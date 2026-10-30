use crate::WorkAttempt;
use crate::WorkCache;
use crate::cache_state::CacheState;
use crate::work_certificate::WorkCertificate;
use eyre::OptionExt;
use eyre::Result;
use eyre::bail;
use eyre::eyre;
use std::sync::Arc;

/// Records semantic child admission, including children dropped before polling.
pub(crate) struct WorkChildGuard {
    parent: WorkAttempt,
    completed: bool,
}

impl WorkChildGuard {
    pub fn admit(cache: &WorkCache, parent: &WorkAttempt, reusable: bool) -> Result<Self> {
        if !Arc::ptr_eq(&cache.state, &parent.cache.state) {
            // Poison through the original owner without ever holding two owners'
            // state locks. Catching this admission failure cannot memoize a fallback.
            let mut state = parent.cache.lock_state()?;
            parent.entry(&parent.cache, &state)?;
            parent.mark_nonreusable(&mut state);
            bail!("cache attempt belongs to a different cache owner");
        }
        let mut state = cache.lock_state()?;
        parent.entry(cache, &state)?;
        let attempt = state
            .attempts
            .get_mut(&parent.id)
            .ok_or_eyre("cache initializer attempt has already completed")?;
        if attempt.frozen || !reusable {
            parent.mark_nonreusable(&mut state);
            bail!("a reusable initializer cannot admit frozen or nonreusable semantic child work");
        }
        let Some(pending) = attempt.pending_children.checked_add(1) else {
            parent.mark_nonreusable(&mut state);
            return Err(eyre!("too many active semantic child requests"));
        };
        attempt.pending_children = pending;
        Ok(Self {
            parent: parent.clone(),
            completed: false,
        })
    }

    /// Called with state held at the same point that the child's result is admitted.
    pub fn complete(
        &mut self,
        state: &mut CacheState,
        certificate: &WorkCertificate,
    ) -> Result<()> {
        self.completed = true;
        let Some(parent) = state.attempts.get_mut(&self.parent.id) else {
            bail!("semantic child completed after its initializer parent");
        };
        parent.pending_children -= 1;
        let mut ancestor = Some(self.parent.id);
        while let Some(id) = ancestor {
            let Some(attempt) = state.attempts.get(&id) else {
                break;
            };
            if certificate
                .observations()
                .any(|entry| entry.identity == attempt.identity)
            {
                self.parent.mark_nonreusable(state);
                bail!("cached semantic dependency cycle");
            }
            ancestor = attempt.parent;
        }
        if !certificate.is_reusable(state) {
            self.parent.mark_nonreusable(state);
        }
        let parent = state
            .attempts
            .get_mut(&self.parent.id)
            .ok_or_eyre("semantic child completed after its initializer parent")?;
        if parent.frozen {
            self.parent.mark_nonreusable(state);
            bail!("semantic child completed after its parent's dependency ledger was frozen");
        }
        parent
            .dependencies
            .extend(certificate.observations().cloned());
        Ok(())
    }
}

impl Drop for WorkChildGuard {
    fn drop(&mut self) {
        if !self.completed {
            let mut state = self
                .parent
                .cache
                .state
                .registry
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(parent) = state.attempts.get_mut(&self.parent.id) {
                parent.pending_children -= 1;
            }
            self.parent.mark_nonreusable(&mut state);
        }
    }
}

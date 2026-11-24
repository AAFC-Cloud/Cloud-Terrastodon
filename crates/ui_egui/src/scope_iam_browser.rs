use crate::scope_deny_assignment_row::ScopeDenyAssignmentRow;
use crate::scope_iam_inventory::ScopeIamInventory;
use crate::scope_iam_principals::ScopeIamPrincipals;
use crate::scope_iam_section::ScopeIamSection;
use crate::scope_iam_snapshot::ScopeIamSnapshot;
use crate::scope_iam_thumbnails::ScopeIamThumbnails;
use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_command::app_work::Loadable;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;

/// Current account's IAM data, retained independently of any work-cache owner.
#[derive(Default)]
pub(crate) struct ScopeIamBrowser {
    pub inventory: Loadable<Arc<ScopeIamInventory>>,
    pub snapshots: HashMap<ScopeImpl, Arc<ScopeIamSnapshot>>,
    pub denies: HashMap<ScopeImpl, Loadable<Arc<Vec<ScopeDenyAssignmentRow>>>>,
    pub principals: ScopeIamPrincipals,
    pub thumbnails: Arc<Mutex<ScopeIamThumbnails>>,
    pub compared: HashSet<ScopeImpl>,
    /// Scope whose ARM comparison and principal lookups should also refresh.
    pub refresh_scope: Option<ScopeImpl>,
    sections: HashMap<ScopeImpl, ScopeIamSection>,
}

impl ScopeIamBrowser {
    pub fn section(&self, scope: &ScopeImpl) -> ScopeIamSection {
        self.sections.get(scope).copied().unwrap_or_default()
    }

    pub fn select_section(&mut self, scope: &ScopeImpl, section: ScopeIamSection) {
        self.sections.insert(scope.clone(), section);
    }

    pub fn can_refresh(&self, scope: &ScopeImpl) -> bool {
        match self.section(scope) {
            ScopeIamSection::DenyAssignments => {
                !matches!(self.denies.get(scope), Some(Loadable::Loading { .. }))
            }
            _ => !matches!(self.inventory, Loadable::Loading { .. }),
        }
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.inventory, Loadable::Loading { .. })
            || self
                .denies
                .values()
                .any(|load| matches!(load, Loadable::Loading { .. }))
            || self.principals.is_loading()
    }
}

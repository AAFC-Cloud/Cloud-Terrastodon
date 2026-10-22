use crate::SourceRepositoryAuthorSummary;
use crate::SourceRepositoryHistorySummary;
use chrono::DateTime;
use eyre::Result;
use eyre::WrapErr;
use gix::Reference;
use gix::Repository;
use gix::actor::SignatureRef;
use gix::hash::ObjectId;
use gix::head::Kind as HeadKind;
use gix::objs::Kind as ObjectKind;
use gix::prelude::ReferenceExt;
use gix::refs::TargetRef;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::str;
use teamy_cancellation::CancellationScope;

/// Aggregate each reachable commit once, using its author rather than committer.
///
/// All local refs contribute roots, including remote-tracking refs and tags;
/// detached HEAD contributes a root as well. Reflogs and unreachable objects do
/// not contribute. Shallow boundaries intentionally stop parent traversal.
/// No transport, mailmap, working-tree files, or historical file blobs are read.
pub(crate) fn read_repository_history(
    repository: &Repository,
    cancellation: &CancellationScope,
) -> Result<SourceRepositoryHistorySummary> {
    // The pinned gix API documents shallow commits as roots whose parents are
    // intentionally unavailable, rather than corrupt/missing history objects.
    // https://docs.rs/gix/0.84.0/gix/struct.Repository.html#method.shallow_commits
    let shallow = repository
        .shallow_commits()
        .wrap_err("Reading Git shallow-history boundaries")?;
    let shallow_boundaries: HashSet<_> = shallow
        .iter()
        .flat_map(|commits| commits.iter().copied())
        .collect();
    let mut pending = Vec::new();
    let head = repository.head().wrap_err("Reading Git HEAD")?;
    let head_target = match head.kind {
        HeadKind::Unborn(_) => None,
        HeadKind::Detached { target, .. } => Some(target),
        HeadKind::Symbolic(reference) => Some(reference_target(
            repository,
            reference.attach(repository),
            cancellation,
        )?),
    };
    if let Some(target) = head_target
        && let Some(commit) = commit_tip(repository, target, cancellation)?
    {
        pending.push(commit);
    }
    // Finish enumeration before resolving symbolic references. Keeping a packed
    // reference iterator alive while reacquiring its buffer can deadlock.
    // https://docs.rs/gix/0.84.0/gix/reference/iter/struct.Iter.html#method.peeled
    let references = repository
        .references()
        .wrap_err("Opening Git reference inventory")?
        .all()
        .wrap_err("Enumerating Git references")?
        .map(|reference| {
            cancellation.bail_if_cancelled()?;
            reference
                .map_err(|error| eyre::eyre!(error))
                .wrap_err("Reading Git reference")
        })
        .collect::<Result<Vec<_>>>()?;
    for reference in references {
        cancellation.bail_if_cancelled()?;
        let target = reference_target(repository, reference, cancellation)?;
        if let Some(commit) = commit_tip(repository, target, cancellation)? {
            pending.push(commit);
        }
    }
    let mut seen = HashSet::new();
    let mut authors: BTreeMap<(String, String), SourceRepositoryAuthorSummary> = BTreeMap::new();
    let mut total_commits = 0;
    while let Some(id) = pending.pop() {
        cancellation.bail_if_cancelled()?;
        if !seen.insert(id) {
            continue;
        }
        let commit = repository
            .find_commit(id)
            .wrap_err_with(|| format!("Reading Git commit {id}"))?;
        let decoded = commit
            .decode()
            .wrap_err_with(|| format!("Decoding Git commit {id}"))?;
        // Parsing the raw author header preserves the exact name/email pair;
        // CommitRef::author() additionally trims identity whitespace.
        let author = SignatureRef::from_bytes(decoded.author.as_ref())
            .wrap_err_with(|| format!("Reading author of Git commit {id}"))?;
        let name = str::from_utf8(author.name.as_ref())
            .wrap_err_with(|| format!("Git commit {id} author name is not UTF-8"))?
            .to_owned();
        let email = str::from_utf8(author.email.as_ref())
            .wrap_err_with(|| format!("Git commit {id} author email is not UTF-8"))?
            .to_owned();
        let timestamp = author
            .time()
            .wrap_err_with(|| format!("Reading authored timestamp of Git commit {id}"))?;
        // Git stores epoch seconds independently of the displayed timezone
        // offset. DateTime::from_timestamp therefore yields the UTC instant.
        let authored_at = DateTime::from_timestamp(timestamp.seconds, 0).ok_or_else(|| {
            eyre::eyre!(
                "Git commit {id} authored timestamp {} is outside the supported datetime range",
                timestamp.seconds
            )
        })?;
        let summary = authors
            .entry((name, email))
            .or_insert_with_key(|(name, email)| SourceRepositoryAuthorSummary {
                name: name.clone(),
                email: email.clone(),
                first_commit_at: authored_at,
                last_commit_at: authored_at,
                commit_count: 0,
            });
        summary.first_commit_at = summary.first_commit_at.min(authored_at);
        summary.last_commit_at = summary.last_commit_at.max(authored_at);
        summary.commit_count += 1;
        total_commits += 1;
        if !shallow_boundaries.contains(&id) {
            // Every parent contributes, including both sides of merges. The
            // visited set deduplicates commits shared by refs or merge parents.
            pending.extend(decoded.parents());
        }
    }
    cancellation.bail_if_cancelled()?;
    Ok(SourceRepositoryHistorySummary {
        authors: authors.into_values().collect(),
        total_commits,
        is_shallow: shallow.is_some(),
    })
}

fn reference_target<'repo>(
    repository: &'repo Repository,
    mut reference: Reference<'repo>,
    cancellation: &CancellationScope,
) -> Result<ObjectId> {
    let mut seen = HashSet::new();
    loop {
        cancellation.bail_if_cancelled()?;
        eyre::ensure!(
            seen.insert(reference.name().as_bstr().to_owned()),
            "Git symbolic-reference cycle at {}",
            reference.name().as_bstr()
        );
        match reference.target() {
            TargetRef::Object(id) => return Ok(id.to_owned()),
            TargetRef::Symbolic(name) => {
                reference = repository
                    .find_reference(name)
                    .wrap_err_with(|| format!("Resolving Git symbolic reference {name}"))?;
            }
        }
    }
}

/// Peel annotated tags using headers so tags of trees/blobs need no file data.
fn commit_tip(
    repository: &Repository,
    mut id: ObjectId,
    cancellation: &CancellationScope,
) -> Result<Option<ObjectId>> {
    let mut seen = HashSet::new();
    loop {
        cancellation.bail_if_cancelled()?;
        eyre::ensure!(seen.insert(id), "Git annotated-tag cycle at {id}");
        match repository
            .find_header(id)
            .wrap_err_with(|| format!("Reading Git reference target {id}"))?
            .kind()
        {
            ObjectKind::Commit => return Ok(Some(id)),
            ObjectKind::Tree | ObjectKind::Blob => return Ok(None),
            ObjectKind::Tag => {
                let tag = repository
                    .find_tag(id)
                    .wrap_err_with(|| format!("Reading Git annotated tag {id}"))?;
                id = tag
                    .decode()
                    .wrap_err_with(|| format!("Decoding Git annotated tag {id}"))?
                    .target();
            }
        }
    }
}

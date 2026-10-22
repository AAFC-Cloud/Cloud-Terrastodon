use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use facet::Facet;

/// Commits attributed to one exact Git author name and email pair.
///
/// Dates are the minimum and maximum authored timestamps in available reachable
/// history, expressed in UTC. Committer identities and dates are not used.
/// See [Git's commit object format](https://git-scm.com/docs/git-commit-tree).
#[derive(Debug, Clone, PartialEq, Eq, Facet)]
pub struct SourceRepositoryAuthorSummary {
    pub name: String,
    pub email: String,
    pub first_commit_at: DateTime<Utc>,
    pub last_commit_at: DateTime<Utc>,
    pub commit_count: u64,
}

impl<'a> Arbitrary<'a> for SourceRepositoryAuthorSummary {
    fn arbitrary(input: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let identity: u64 = input.arbitrary()?;
        let commit_count = input.int_in_range(1..=1_000_000)?;
        let first = DateTime::<Utc>::arbitrary(input)?;
        let last = if commit_count == 1 {
            first
        } else {
            DateTime::<Utc>::arbitrary(input)?
        };
        Ok(Self {
            name: format!("Author {identity}"),
            email: format!("author{identity}@example.invalid"),
            first_commit_at: first.min(last),
            last_commit_at: first.max(last),
            commit_count,
        })
    }
}

cloud_terrastodon_registry::register_thing!(SourceRepositoryAuthorSummary);
cloud_terrastodon_registry::register_arbitrary!(SourceRepositoryAuthorSummary);
cloud_terrastodon_registry::register_thing!(Vec<SourceRepositoryAuthorSummary>);
cloud_terrastodon_registry::register_arbitrary!(Vec<SourceRepositoryAuthorSummary>);

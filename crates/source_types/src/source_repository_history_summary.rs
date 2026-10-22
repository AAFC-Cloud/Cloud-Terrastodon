use crate::SourceRepositoryAuthorSummary;
use arbitrary::Arbitrary;
use facet::Facet;

/// Available commit history reachable from HEAD and locally stored references.
///
/// Each commit is counted once within the repository, across all branches,
/// remote-tracking references, and tags. No history is fetched. Shallow clones
/// contain only their locally available history.
/// See [Git revision reachability](https://git-scm.com/docs/gitrevisions) and
/// [shallow repositories](https://git-scm.com/docs/shallow).
#[derive(Debug, Clone, PartialEq, Eq, Facet)]
pub struct SourceRepositoryHistorySummary {
    pub authors: Vec<SourceRepositoryAuthorSummary>,
    pub total_commits: u64,
    pub is_shallow: bool,
}

impl<'a> Arbitrary<'a> for SourceRepositoryHistorySummary {
    fn arbitrary(input: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let author_count = input.int_in_range(0..=8)?;
        let mut authors = Vec::new();
        for index in 0..author_count {
            let mut author = SourceRepositoryAuthorSummary::arbitrary(input)?;
            // Distinct emails keep the generated history partitioned by identity.
            author.email = format!("author{index}@example.invalid");
            authors.push(author);
        }
        Ok(Self {
            total_commits: authors.iter().map(|author| author.commit_count).sum(),
            authors,
            is_shallow: input.arbitrary()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arbitrary::Unstructured;
    use std::collections::BTreeSet;

    #[test]
    fn generated_history_preserves_author_partitions_and_date_bounds() {
        for seed in 0..=32 {
            let bytes = vec![seed; 2048];
            let history = SourceRepositoryHistorySummary::arbitrary(&mut Unstructured::new(&bytes))
                .expect("fixture has enough bytes for a bounded history");
            assert_eq!(
                history.total_commits,
                history
                    .authors
                    .iter()
                    .map(|author| author.commit_count)
                    .sum()
            );
            let identities: BTreeSet<_> = history
                .authors
                .iter()
                .map(|author| (&author.name, &author.email))
                .collect();
            assert_eq!(identities.len(), history.authors.len());
            for author in history.authors {
                assert!(author.commit_count > 0);
                assert!(author.first_commit_at <= author.last_commit_at);
                if author.commit_count == 1 {
                    assert_eq!(author.first_commit_at, author.last_commit_at);
                }
            }
        }
    }
}

cloud_terrastodon_registry::register_thing!(SourceRepositoryHistorySummary);
cloud_terrastodon_registry::register_arbitrary!(SourceRepositoryHistorySummary);

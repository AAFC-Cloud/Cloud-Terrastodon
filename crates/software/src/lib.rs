use eyre::Context;
use teamy_mft::cli::command::query::QueryArgs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoftwareQuery {
    pub pattern: &'static str,
    pub query: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoftwareQuerySummary {
    pub pattern: &'static str,
    pub query: &'static str,
    pub result_count: usize,
}

pub const SOFTWARE_QUERIES: [SoftwareQuery; 3] = [
    SoftwareQuery {
        pattern: ".git",
        query: ".git$",
    },
    SoftwareQuery {
        pattern: "package.json",
        query: "package.json$",
    },
    SoftwareQuery {
        pattern: "Cargo.toml",
        query: "Cargo.toml$",
    },
];

#[must_use]
pub fn software_queries() -> &'static [SoftwareQuery] {
    &SOFTWARE_QUERIES
}

/// # Errors
///
/// Returns an error if the underlying teamy-mft query fails.
pub fn list_software_counts() -> eyre::Result<Vec<SoftwareQuerySummary>> {
    list_software_counts_with(|software_query| {
        Ok(QueryArgs::new(software_query.query).collect_rows()?.len())
    })
}

/// # Errors
///
/// Returns an error if the provided counter fails for any query.
pub fn list_software_counts_with(
    mut count_query: impl FnMut(&SoftwareQuery) -> eyre::Result<usize>,
) -> eyre::Result<Vec<SoftwareQuerySummary>> {
    software_queries()
        .iter()
        .copied()
        .map(|software_query| {
            let result_count = count_query(&software_query)
                .wrap_err_with(|| format!("failed to query {}", software_query.pattern))?;
            Ok(SoftwareQuerySummary {
                pattern: software_query.pattern,
                query: software_query.query,
                result_count,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::SOFTWARE_QUERIES;
    use super::list_software_counts_with;
    use std::collections::HashMap;

    #[test]
    fn mvp_query_list_is_stable() {
        let queries = SOFTWARE_QUERIES
            .iter()
            .map(|query| (query.pattern, query.query))
            .collect::<Vec<_>>();

        assert_eq!(
            queries,
            vec![
                (".git", ".git$"),
                ("package.json", "package.json$"),
                ("Cargo.toml", "Cargo.toml$"),
            ]
        );
    }

    #[test]
    fn summaries_preserve_pattern_order() {
        let counts = HashMap::from([
            (".git$", 2_usize),
            ("package.json$", 5_usize),
            ("Cargo.toml$", 3_usize),
        ]);

        let summaries = list_software_counts_with(|software_query| {
            counts
                .get(software_query.query)
                .copied()
                .ok_or_else(|| eyre::eyre!("missing count for {}", software_query.query))
        })
        .expect("summary generation should succeed");

        let rendered = summaries
            .into_iter()
            .map(|summary| (summary.pattern, summary.result_count))
            .collect::<Vec<_>>();

        assert_eq!(
            rendered,
            vec![(".git", 2), ("package.json", 5), ("Cargo.toml", 3)]
        );
    }
}

use arbitrary::Arbitrary;
use facet::Facet;

/// A named remote from a repository's local Git configuration.
///
/// URLs are opaque Git transport strings, including SCP-style SSH addresses and
/// local paths. They are read without connecting or applying URL rewrite rules.
/// Missing `push_url` means no explicit `remote.<name>.pushurl` is configured.
/// See [Git's remote configuration](https://git-scm.com/docs/git-config#Documentation/git-config.txt-remoteltnamegturl).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
pub struct SourceRepositoryRemote {
    pub name: String,
    pub fetch_url: Option<String>,
    pub push_url: Option<String>,
}

cloud_terrastodon_registry::register_thing!(SourceRepositoryRemote);
cloud_terrastodon_registry::register_arbitrary!(SourceRepositoryRemote);

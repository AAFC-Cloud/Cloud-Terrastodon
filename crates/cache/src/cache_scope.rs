use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use eyre::Report;
use eyre::Result;
use eyre::ensure;
use facet::Facet;

/// A nonempty hierarchy of exact logical cache identity components.
///
/// Components are opaque strings, not filesystem paths. Separators, whitespace,
/// case, and Unicode are preserved without normalization. Empty components and
/// the exact traversal markers `.` and `..` are rejected. Physical encoding is
/// a separate storage responsibility; raw components must never be joined onto
/// a filesystem root.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Facet)]
#[facet(proxy = Vec<String>)]
pub struct CacheScope {
    // Private storage prevents mutation from bypassing hierarchy validation.
    components: Vec<String>,
}

impl CacheScope {
    /// Validate explicit components without interpreting any of them as paths.
    pub fn try_from_components<I, S>(components: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::try_from(components.into_iter().map(Into::into).collect::<Vec<_>>())
    }

    /// Inspect the exact validated components without permitting mutation.
    pub fn components(&self) -> &[String] {
        &self.components
    }

    /// Append exactly one logical component, preserving this scope.
    pub fn join(&self, component: impl Into<String>) -> Result<Self> {
        let mut components = self.components.clone();
        components.push(component.into());
        Self::try_from(components)
    }

    /// Whether this scope is the supplied scope or one of its descendants.
    ///
    /// Prefixes are compared by complete components, never string prefixes.
    pub fn starts_with(&self, prefix: &Self) -> bool {
        self.components.starts_with(&prefix.components)
    }
}

impl TryFrom<Vec<String>> for CacheScope {
    type Error = Report;

    fn try_from(components: Vec<String>) -> Result<Self> {
        ensure!(
            !components.is_empty(),
            "cache scope must contain a component"
        );
        for (index, component) in components.iter().enumerate() {
            ensure!(
                !component.is_empty(),
                "cache scope component {index} must not be empty"
            );
            ensure!(
                component != "." && component != "..",
                "cache scope component {index} must not be the traversal marker {component:?}"
            );
        }
        Ok(Self { components })
    }
}

impl From<&CacheScope> for Vec<String> {
    fn from(scope: &CacheScope) -> Self {
        scope.components.clone()
    }
}

impl<'a> Arbitrary<'a> for CacheScope {
    fn arbitrary(bytes: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        // Bound generated fixture size, not the domain's accepted hierarchy.
        let count = bytes.int_in_range(1..=8)?;
        let mut components = Vec::with_capacity(count);
        for _ in 0..count {
            let component = String::arbitrary(bytes)?;
            components.push(match component.as_str() {
                "" | "." | ".." => "component".to_owned(),
                _ => component,
            });
        }
        Self::try_from(components).map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

cloud_terrastodon_registry_values::register_thing!(CacheScope);
cloud_terrastodon_registry_values::register_arbitrary!(CacheScope);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_hierarchies_and_traversal_markers() {
        assert!(CacheScope::try_from(Vec::new()).is_err());
        for invalid in ["", ".", ".."] {
            assert!(CacheScope::try_from_components([invalid]).is_err());
            assert!(
                CacheScope::try_from(vec!["operation".to_owned(), invalid.to_owned()]).is_err()
            );
            let scope = CacheScope::try_from_components(["operation"]).unwrap();
            assert!(scope.join(invalid).is_err());
        }
    }

    #[test]
    fn facet_decoding_cannot_bypass_scope_validation() {
        let invalid_components = [
            Vec::new(),
            vec!["operation", ""],
            vec!["operation", "."],
            vec!["operation", ".."],
        ];
        for components in invalid_components {
            let encoded = facet_json::to_string(&components).unwrap();
            assert!(facet_json::from_str::<CacheScope>(&encoded).is_err());
        }
    }

    #[test]
    fn components_preserve_exact_identity_instead_of_path_semantics() {
        let components = [
            "operations",
            " A ",
            "a",
            "a/b",
            r"C:\tenant\..\scope",
            "../x",
            "é%",
        ];
        let scope = CacheScope::try_from_components(components).unwrap();
        assert_eq!(scope.components(), components);
        assert_eq!(Vec::<String>::from(&scope), components);
        assert_ne!(
            CacheScope::try_from_components(["a/b"]).unwrap(),
            CacheScope::try_from_components(["a", "b"]).unwrap()
        );
        assert_ne!(
            CacheScope::try_from_components(["A"]).unwrap(),
            CacheScope::try_from_components(["a"]).unwrap()
        );
    }

    #[test]
    fn hierarchy_prefixes_require_whole_components() {
        let parent = CacheScope::try_from_components(["projects", "sample"]).unwrap();
        let child = parent.join("page/1").unwrap();
        assert!(parent.starts_with(&parent));
        assert!(child.starts_with(&parent));
        assert!(!parent.starts_with(&child));
        assert!(
            !CacheScope::try_from_components(["projects", "sample-other"])
                .unwrap()
                .starts_with(&parent)
        );
        assert!(
            !CacheScope::try_from_components(["projects/sample", "page/1"])
                .unwrap()
                .starts_with(&parent)
        );
    }

    #[test]
    fn arbitrary_generation_preserves_scope_invariants() {
        for value in 0..=u8::MAX {
            let input = [value; 256];
            if let Ok(scope) = CacheScope::arbitrary(&mut Unstructured::new(&input)) {
                assert_eq!(
                    CacheScope::try_from(Vec::<String>::from(&scope)).unwrap(),
                    scope
                );
            }
        }
        // Empty entropy still produces a valid scope, rather than an empty one.
        let scope = CacheScope::arbitrary(&mut Unstructured::new(&[])).unwrap();
        assert!(!scope.components().is_empty());
    }
}

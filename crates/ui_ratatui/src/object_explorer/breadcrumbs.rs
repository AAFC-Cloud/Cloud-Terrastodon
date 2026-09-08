use super::breadcrumb::Breadcrumb;

#[derive(Clone, Debug, Default, Eq, PartialEq, arbitrary::Arbitrary, facet::Facet)]
#[repr(C)]
pub(crate) struct Breadcrumbs {
    /// Ordered, composable projection/filter operations. This is the query
    /// program itself, never an evaluated result collection.
    operations: Vec<Breadcrumb>,
}

impl Breadcrumbs {
    pub(crate) fn new(operations: Vec<Breadcrumb>) -> Self {
        Self { operations }
    }

    pub(crate) fn operations(&self) -> &[Breadcrumb] {
        &self.operations
    }

    pub(crate) fn into_operations(self) -> Vec<Breadcrumb> {
        self.operations
    }

    pub(crate) fn push(&mut self, operation: Breadcrumb) {
        self.operations.push(operation);
    }

    pub(crate) fn remove(&mut self, index: usize) -> Breadcrumb {
        self.operations.remove(index)
    }

    pub(crate) fn replace(&mut self, index: usize, operation: Breadcrumb) -> Option<Breadcrumb> {
        let existing = self.operations.get_mut(index)?;
        Some(std::mem::replace(existing, operation))
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
}

cloud_terrastodon_registry::register_thing!(Breadcrumbs);
cloud_terrastodon_registry::register_arbitrary!(Breadcrumbs);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::produce_json_request::ProduceJsonRequest;
    use crate::object_explorer::tab::Tab;
    use cloud_terrastodon_registry::ArbitraryBytes;
    use facet::Facet;

    fn generated<T: Facet<'static> + Send + 'static>(bytes: Vec<u8>) -> T {
        let constructor =
            cloud_terrastodon_registry::functions_from_to(ArbitraryBytes::SHAPE, T::SHAPE)
                .into_iter()
                .find(|function| function.output_shape.is_shape(T::SHAPE))
                .expect("the exact arbitrary constructor is registered");
        *constructor
            .invoke_mut_boxed(&mut ArbitraryBytes::new(bytes))
            .expect("the deterministic query fixture should generate")
            .downcast::<T>()
            .expect("the constructor returns its declared type")
    }

    #[test]
    fn registered_query_generators_create_round_trippable_values_without_execution()
    -> eyre::Result<()> {
        for first in [0, 1] {
            let mut bytes = vec![0; 4096];
            bytes[0] = first;
            let breadcrumbs = generated::<Breadcrumbs>(bytes.clone());
            assert_eq!(
                facet_json::from_str::<Breadcrumbs>(&facet_json::to_string(&breadcrumbs)?)?,
                breadcrumbs
            );
            if first == 1 {
                assert!(
                    !breadcrumbs.is_empty(),
                    "query generation is not an empty-only placeholder"
                );
            }

            let tab = generated::<Tab>(bytes.clone());
            assert_eq!(
                facet_json::from_str::<Tab>(&facet_json::to_string(&tab)?)?,
                tab
            );

            if first == 1 {
                assert!(
                    !tab.breadcrumbs().is_empty(),
                    "tab generation must exercise a nonempty query"
                );
                // ProduceJsonRequest consumes its u64 filename suffix first;
                // the next byte controls its breadcrumb vector continuation.
                bytes[8] = 1;
            }

            let request = generated::<ProduceJsonRequest>(bytes);
            if first == 1 {
                assert!(
                    !request.breadcrumbs().is_empty(),
                    "export-request generation must exercise a nonempty query"
                );
            }
            let filename = std::path::Path::new(request.filename());
            assert!(!filename.is_absolute());
            assert_eq!(filename.components().count(), 1);
            assert!(matches!(
                filename.components().next(),
                Some(std::path::Component::Normal(_))
            ));
            assert_eq!(
                filename
                    .extension()
                    .and_then(|extension| extension.to_str()),
                Some("json")
            );
            assert!(request.filename().starts_with("object-explorer-"));
            let decoded =
                facet_json::from_str::<ProduceJsonRequest>(&facet_json::to_string(&request)?)?;
            assert_eq!(decoded.filename(), request.filename());
            assert_eq!(decoded.breadcrumbs(), request.breadcrumbs());
        }
        Ok(())
    }
}

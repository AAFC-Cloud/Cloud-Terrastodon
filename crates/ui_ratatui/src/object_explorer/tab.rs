use super::breadcrumbs::Breadcrumbs;

/// An ordinary arena-owned query definition.
///
/// Query results are never fields of Tab. Opening a tab in the UI stores only
/// its SlotId; evaluation reads these breadcrumbs explicitly and lazily.
#[derive(Clone, Debug, Eq, PartialEq, facet::Facet)]
#[repr(C)]
pub(crate) struct Tab {
    name: String,
    #[facet(default)]
    breadcrumbs: Breadcrumbs,
}

impl Tab {
    pub(crate) fn new(name: impl Into<String>, breadcrumbs: Breadcrumbs) -> Self {
        Self {
            name: name.into(),
            breadcrumbs,
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) const fn breadcrumbs(&self) -> &Breadcrumbs {
        &self.breadcrumbs
    }

    pub(crate) fn apply(&mut self, update: super::tab_update::TabUpdate) -> Result<(), String> {
        match update {
            super::tab_update::TabUpdate::Rename(name) => self.name = name,
            super::tab_update::TabUpdate::PushBreadcrumb(breadcrumb) => {
                self.breadcrumbs.push(breadcrumb)
            }
            super::tab_update::TabUpdate::ReplaceBreadcrumb { index, breadcrumb } => {
                if self.breadcrumbs.replace(index, breadcrumb).is_none() {
                    return Err(format!("breadcrumb {index} does not exist"));
                }
            }
            super::tab_update::TabUpdate::RemoveBreadcrumb(index) => {
                if index >= self.breadcrumbs.operations().len() {
                    return Err(format!("breadcrumb {index} does not exist"));
                }
                self.breadcrumbs.remove(index);
            }
            super::tab_update::TabUpdate::ReplaceBreadcrumbs(breadcrumbs) => {
                self.breadcrumbs = breadcrumbs
            }
        }
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(Tab);

#[cfg(test)]
mod tests {
    use super::*;
    use facet::Facet;
    use facet::Type;
    use facet::UserType;

    #[test]
    fn tab_shape_contains_query_not_materialized_results() {
        let Type::User(UserType::Struct(tab)) = Tab::SHAPE.ty else {
            panic!("Tab must remain a reflected struct");
        };
        let fields = tab
            .fields
            .iter()
            .map(|field| field.effective_name())
            .collect::<Vec<_>>();

        assert_eq!(fields, ["name", "breadcrumbs"]);
        assert!(!fields.contains(&"entries"));
    }
}

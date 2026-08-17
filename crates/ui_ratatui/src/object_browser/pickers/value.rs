use crate::object_explorer::SlotId;
use crate::object_explorer::ValueAddress;
use crate::object_explorer::ValueCandidate;
use crate::object_explorer::ValueCandidateWindow;
use crate::object_explorer::arbitrary_constructor_for;
use cloud_terrastodon_registry::Function;
use cloud_terrastodon_registry::ProductionKind;
use cloud_terrastodon_registry::default_production_plan;
use cloud_terrastodon_registry::describe_function;
use cloud_terrastodon_registry::describe_shape;
use cloud_terrastodon_registry::functions_to;
use facet::Facet;
use facet::Shape;
use std::fmt;

#[derive(Clone)]
pub(crate) enum ValuePickerChoice {
    CreateNewOwned { shape: &'static Shape },
    Candidate { address: ValueAddress },
    CreateProducer { function: &'static Function },
    InvokeDefaultProducer { function: &'static Function },
    InvokeArbitraryProducer { function: &'static Function },
}

impl fmt::Debug for ValuePickerChoice {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreateNewOwned { shape } => formatter
                .debug_struct("CreateNewOwned")
                .field("shape", &describe_shape(shape))
                .finish(),
            Self::Candidate { address } => formatter
                .debug_struct("Candidate")
                .field("address", address)
                .finish(),
            Self::CreateProducer { function } => formatter
                .debug_struct("CreateProducer")
                .field("function", &describe_function(function))
                .finish(),
            Self::InvokeDefaultProducer { function } => formatter
                .debug_struct("InvokeDefaultProducer")
                .field("function", &describe_function(function))
                .finish(),
            Self::InvokeArbitraryProducer { function } => formatter
                .debug_struct("InvokeArbitraryProducer")
                .field("function", &describe_function(function))
                .finish(),
        }
    }
}

impl PartialEq for ValuePickerChoice {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::CreateNewOwned { shape: left }, Self::CreateNewOwned { shape: right }) => {
                left.is_shape(right)
            }
            (Self::Candidate { address: left }, Self::Candidate { address: right }) => {
                left == right
            }
            (Self::CreateProducer { function: left }, Self::CreateProducer { function: right })
            | (
                Self::InvokeDefaultProducer { function: left },
                Self::InvokeDefaultProducer { function: right },
            )
            | (
                Self::InvokeArbitraryProducer { function: left },
                Self::InvokeArbitraryProducer { function: right },
            ) => std::ptr::eq(*left, *right),
            _ => false,
        }
    }
}

impl Eq for ValuePickerChoice {}

struct ProducerChoiceMetadata {
    function: &'static Function,
    has_default: bool,
    has_arbitrary: bool,
    is_arbitrary_constructor: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ValuePickerRow {
    label: String,
    choice: ValuePickerChoice,
}

impl ValuePickerRow {
    pub(crate) fn label(&self) -> &str {
        &self.label
    }

    pub(crate) const fn choice(&self) -> &ValuePickerChoice {
        &self.choice
    }
}

/// Picker-local state over one bounded candidate window.
pub(crate) struct FieldValuePicker {
    destination: SlotId,
    field: usize,
    field_name: String,
    target_shape: &'static Shape,
    query: String,
    candidates: Vec<ValueCandidate>,
    producers: Vec<ProducerChoiceMetadata>,
    has_before: bool,
    has_after: bool,
    rows: Vec<ValuePickerRow>,
    selected: usize,
    received_window: bool,
}

impl FieldValuePicker {
    pub(crate) fn new(
        destination: SlotId,
        field: usize,
        field_name: impl Into<String>,
        target_shape: &'static Shape,
    ) -> Self {
        let source_shape =
            cloud_terrastodon_registry::RuntimeValue::preferred_field_source_shape(target_shape);
        let arbitrary_constructor = arbitrary_constructor_for(source_shape);
        let producers = functions_to(source_shape)
            .into_iter()
            .filter(|function| {
                function.production_kind(source_shape) == Some(ProductionKind::Exact)
            })
            .map(|function| ProducerChoiceMetadata {
                function,
                has_default: default_production_plan(function.input_shape).is_some(),
                has_arbitrary: arbitrary_constructor_for(function.input_shape).is_some(),
                is_arbitrary_constructor: arbitrary_constructor
                    .is_some_and(|candidate| std::ptr::eq(candidate, function)),
            })
            .collect();
        let mut result = Self {
            destination,
            field,
            field_name: field_name.into(),
            target_shape,
            query: String::new(),
            candidates: Vec::new(),
            producers,
            has_before: false,
            has_after: false,
            rows: Vec::new(),
            selected: 0,
            received_window: false,
        };
        result.rebuild(None);
        result
    }

    pub(crate) const fn destination(&self) -> SlotId {
        self.destination
    }

    pub(crate) const fn field(&self) -> usize {
        self.field
    }

    pub(crate) fn field_name(&self) -> &str {
        &self.field_name
    }

    pub(crate) const fn target_shape(&self) -> &'static Shape {
        self.target_shape
    }

    pub(crate) fn query(&self) -> &str {
        &self.query
    }

    pub(crate) fn rows(&self) -> &[ValuePickerRow] {
        &self.rows
    }

    pub(crate) const fn selected_index(&self) -> usize {
        self.selected
    }

    pub(crate) fn selected(&self) -> Option<&ValuePickerRow> {
        self.rows.get(self.selected)
    }

    pub(crate) const fn has_before(&self) -> bool {
        self.has_before
    }

    pub(crate) const fn has_after(&self) -> bool {
        self.has_after
    }

    pub(crate) fn replace_window(&mut self, window: ValueCandidateWindow) {
        let selected = self
            .received_window
            .then(|| self.selected().map(|row| row.choice.clone()))
            .flatten();
        self.candidates = window.candidates().to_vec();
        self.has_before = window.has_before();
        self.has_after = window.has_after();
        self.rebuild(selected.as_ref());
        self.received_window = true;
    }

    pub(crate) fn push(&mut self, character: char) {
        self.query.push(character);
        self.rebuild(None);
    }

    pub(crate) fn pop(&mut self) {
        self.query.pop();
        self.rebuild(None);
    }

    pub(crate) fn move_next(&mut self) {
        if !self.rows.is_empty() {
            self.selected = (self.selected + 1).min(self.rows.len() - 1);
        }
    }

    pub(crate) fn move_previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    fn rebuild(&mut self, preserve: Option<&ValuePickerChoice>) {
        let source_shape = cloud_terrastodon_registry::RuntimeValue::preferred_field_source_shape(
            self.target_shape,
        );
        let create = ValuePickerRow {
            label: format!(
                "+ create new owned {}",
                cloud_terrastodon_registry::describe_shape(source_shape)
            ),
            choice: ValuePickerChoice::CreateNewOwned {
                shape: source_shape,
            },
        };
        let candidate_rows = self
            .candidates
            .iter()
            .filter(|candidate| fuzzy_tokens_match(&candidate.display_label(), &self.query))
            .map(|candidate| ValuePickerRow {
                label: candidate.display_label(),
                choice: ValuePickerChoice::Candidate {
                    address: candidate.address().clone(),
                },
            })
            .collect::<Vec<_>>();
        let mut default_producers = Vec::new();
        let mut arbitrary_default_producers = Vec::new();
        let mut create_producers = Vec::new();
        let mut arbitrary_create_producers = Vec::new();
        let mut arbitrary_producers = Vec::new();
        for producer in &self.producers {
            let input = describe_shape(producer.function.input_shape);
            let target = describe_shape(source_shape);
            if producer.has_default {
                let row = ValuePickerRow {
                    label: format!("+ invoke default {input} for {target}"),
                    choice: ValuePickerChoice::InvokeDefaultProducer {
                        function: producer.function,
                    },
                };
                if fuzzy_tokens_match(&row.label, &self.query) {
                    if producer.is_arbitrary_constructor {
                        arbitrary_default_producers.push(row);
                    } else {
                        default_producers.push(row);
                    }
                }
            }
            let row = ValuePickerRow {
                label: format!(
                    "+ create {input} for {target} via {}",
                    describe_function(producer.function)
                ),
                choice: ValuePickerChoice::CreateProducer {
                    function: producer.function,
                },
            };
            if fuzzy_tokens_match(&row.label, &self.query) {
                if producer.is_arbitrary_constructor {
                    arbitrary_create_producers.push(row);
                } else {
                    create_producers.push(row);
                }
            }
            if producer.has_arbitrary {
                let row = ValuePickerRow {
                    label: format!("+ invoke arbitrary {input} for {target}"),
                    choice: ValuePickerChoice::InvokeArbitraryProducer {
                        function: producer.function,
                    },
                };
                if fuzzy_tokens_match(&row.label, &self.query) {
                    arbitrary_producers.push(row);
                }
            }
        }
        default_producers.sort_by(|left, right| left.label.cmp(&right.label));
        arbitrary_default_producers.sort_by(|left, right| left.label.cmp(&right.label));
        create_producers.sort_by(|left, right| left.label.cmp(&right.label));
        arbitrary_create_producers.sort_by(|left, right| left.label.cmp(&right.label));
        arbitrary_producers.sort_by(|left, right| left.label.cmp(&right.label));
        let create = fuzzy_tokens_match(&create.label, &self.query).then_some(create);
        if self.target_shape.is_shape(String::SHAPE) {
            self.rows = create
                .into_iter()
                .chain(candidate_rows)
                .chain(default_producers)
                .chain(create_producers)
                .chain(arbitrary_default_producers)
                .chain(arbitrary_create_producers)
                .chain(arbitrary_producers)
                .collect();
        } else {
            self.rows = candidate_rows
                .into_iter()
                .chain(default_producers)
                .chain(create)
                .chain(create_producers)
                .chain(arbitrary_default_producers)
                .chain(arbitrary_create_producers)
                .chain(arbitrary_producers)
                .collect();
        }
        self.selected = preserve
            .and_then(|choice| self.rows.iter().position(|row| &row.choice == choice))
            .unwrap_or(0)
            .min(self.rows.len().saturating_sub(1));
    }
}

fn fuzzy_tokens_match(candidate: &str, query: &str) -> bool {
    let candidate = candidate.to_lowercase();
    query
        .to_lowercase()
        .split_whitespace()
        .all(|token| candidate.contains(token))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_browser::ObjectBrowserController;
    use crate::object_explorer::ArenaQueryContext;
    use crate::object_explorer::Breadcrumbs;
    use crate::object_explorer::ExplorerEngine;
    use crate::object_explorer::QueryProgressState;
    use std::borrow::Cow;
    use std::num::NonZeroUsize;

    #[test]
    fn cow_field_picker_offers_generic_default_manual_and_arbitrary_producers() {
        use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
        use cloud_terrastodon_azure_devops::DefaultAzureDevOpsOrganizationUrlRequest;

        // Referencing the request also makes its distributed registry entries
        // part of this test binary.
        let _ = std::any::TypeId::of::<DefaultAzureDevOpsOrganizationUrlRequest>();
        let picker = FieldValuePicker::new(
            SlotId::new(4),
            0,
            "org_url",
            <Cow<'static, AzureDevOpsOrganizationUrl>>::SHAPE,
        );
        let labels = picker
            .rows()
            .iter()
            .map(ValuePickerRow::label)
            .collect::<Vec<_>>();

        assert!(labels.iter().any(|label| {
            label.contains("invoke default DefaultAzureDevOpsOrganizationUrlRequest")
        }));
        assert!(
            labels
                .iter()
                .any(|label| { label.contains("create DefaultAzureDevOpsOrganizationUrlRequest") })
        );
        assert!(labels.iter().any(|label| {
            label.contains("invoke arbitrary DefaultAzureDevOpsOrganizationUrlRequest")
        }));
        assert_eq!(
            labels.first().copied(),
            Some(
                "+ invoke default DefaultAzureDevOpsOrganizationUrlRequest for AzureDevOpsOrganizationUrl"
            ),
            "a domain default producer should outrank ArbitraryBytes-based construction"
        );
        assert!(matches!(
            picker.selected().unwrap().choice(),
            ValuePickerChoice::InvokeDefaultProducer { .. }
        ));
    }

    #[test]
    fn string_field_pickers_default_to_creating_a_new_owned_value() {
        let mut picker = FieldValuePicker::new(SlotId::new(4), 1, "filename", String::SHAPE);
        picker.replace_window(ValueCandidateWindow::new(Vec::new(), false, false));

        assert_eq!(picker.selected_index(), 0);
        assert_eq!(
            picker.selected().unwrap().label(),
            "+ create new owned String"
        );
        assert!(matches!(
            picker.selected().unwrap().choice(),
            ValuePickerChoice::CreateNewOwned { shape } if shape.is_shape(String::SHAPE)
        ));
    }

    #[tokio::test]
    async fn picker_rows_preserve_engine_supplied_field_owner_provenance() {
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(8);
        let client = async move {
            let controller = ObjectBrowserController::bootstrap(context).await.unwrap();
            controller
                .begin_value_candidates(Breadcrumbs::SHAPE)
                .await
                .unwrap();
            let progress = controller
                .fill_value_candidates(None, 16, NonZeroUsize::new(4).unwrap())
                .await
                .unwrap();
            let QueryProgressState::Ready(window) = progress.into_state() else {
                panic!("expected a bounded Breadcrumbs candidate window");
            };
            let mut picker =
                FieldValuePicker::new(SlotId::new(9), 0, "breadcrumbs", Breadcrumbs::SHAPE);

            picker.replace_window(window);

            assert!(
                picker.rows()[0]
                    .label()
                    .contains("field breadcrumbs of slot")
            );
            assert!(picker.rows()[0].label().contains("(Tab)"));
            assert!(matches!(
                picker.rows()[0].choice(),
                ValuePickerChoice::Candidate { .. }
            ));
            assert_eq!(picker.selected_index(), 0);
            assert!(matches!(
                picker.selected().unwrap().choice(),
                ValuePickerChoice::Candidate { .. }
            ));
            controller.end_value_candidates().await.unwrap();
            controller.close().await.unwrap();
        };
        let (_engine, ()) = tokio::join!(engine.run(inbox), client);
    }
}

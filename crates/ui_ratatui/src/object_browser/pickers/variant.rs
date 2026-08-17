use facet::{Shape, Type, UserType};

pub(crate) struct VariantPicker {
    slot: crate::object_explorer::SlotId,
    shape: &'static Shape,
    labels: Vec<String>,
    query: String,
    matches: Vec<usize>,
    selected: usize,
}

impl VariantPicker {
    pub(crate) fn new(slot: crate::object_explorer::SlotId, shape: &'static Shape) -> Option<Self> {
        let Type::User(UserType::Enum(object)) = shape.ty else {
            return None;
        };
        let labels = object
            .variants
            .iter()
            .map(|variant| variant.name.to_owned())
            .collect::<Vec<_>>();
        Some(Self {
            slot,
            shape,
            matches: (0..labels.len()).collect(),
            labels,
            query: String::new(),
            selected: 0,
        })
    }

    pub(crate) const fn slot(&self) -> crate::object_explorer::SlotId {
        self.slot
    }

    pub(crate) const fn shape(&self) -> &'static Shape {
        self.shape
    }

    pub(crate) fn query(&self) -> &str {
        &self.query
    }

    pub(crate) fn matches(&self) -> impl Iterator<Item = (usize, &str)> {
        self.matches
            .iter()
            .map(|index| (*index, self.labels[*index].as_str()))
    }

    pub(crate) fn selected_variant(&self) -> Option<usize> {
        self.matches.get(self.selected).copied()
    }

    pub(crate) const fn selected_index(&self) -> usize {
        self.selected
    }

    pub(crate) fn push(&mut self, character: char) {
        self.query.push(character);
        self.rebuild();
    }

    pub(crate) fn pop(&mut self) {
        self.query.pop();
        self.rebuild();
    }

    pub(crate) fn move_next(&mut self) {
        if !self.matches.is_empty() {
            self.selected = (self.selected + 1).min(self.matches.len() - 1);
        }
    }

    pub(crate) fn move_previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    fn rebuild(&mut self) {
        let query = self.query.to_lowercase();
        self.matches = self
            .labels
            .iter()
            .enumerate()
            .filter_map(|(index, label)| label.to_lowercase().contains(&query).then_some(index))
            .collect();
        self.selected = self.selected.min(self.matches.len().saturating_sub(1));
    }
}

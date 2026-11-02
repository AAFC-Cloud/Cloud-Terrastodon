use crate::DefaultProductionPlan;

pub struct DefaultProductionField {
    pub field_index: usize,
    pub field_name: &'static str,
    pub plan: Box<DefaultProductionPlan>,
}

use crate::DefaultProductionField;
use crate::DefaultProductionPlan;
use crate::Function;

pub enum DefaultProductionPlanKind<'a> {
    Default,
    Struct(&'a [DefaultProductionField]),
    Invoke {
        function: Function,
        input: &'a DefaultProductionPlan,
    },
}

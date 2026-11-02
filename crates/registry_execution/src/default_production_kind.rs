use crate::DefaultProductionField;
use crate::DefaultProductionPlan;
use crate::Function;

pub(crate) enum DefaultProductionKind {
    Default,
    Struct {
        fields: Vec<DefaultProductionField>,
    },
    Invoke {
        function: Function,
        input: Box<DefaultProductionPlan>,
    },
}

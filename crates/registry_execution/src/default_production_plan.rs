use crate::DefaultProductionPlanKind;
use crate::default_production_kind::DefaultProductionKind;
use facet::Shape;

/// A registry-discovered recipe for producing a value without user-provided
/// input. The recipe is deliberately type-erased: callers can inspect its
/// target shape, while the registry remains responsible for deciding which
/// defaults and registered producers form a valid chain.
pub struct DefaultProductionPlan {
    // Construction is restricted to the planner so each recipe matches its target shape.
    pub(crate) shape: &'static Shape,
    pub(crate) kind: DefaultProductionKind,
}

impl DefaultProductionPlan {
    pub fn shape(&self) -> &'static Shape {
        self.shape
    }

    pub fn kind(&self) -> DefaultProductionPlanKind<'_> {
        match &self.kind {
            DefaultProductionKind::Default => DefaultProductionPlanKind::Default,
            DefaultProductionKind::Struct { fields } => DefaultProductionPlanKind::Struct(fields),
            DefaultProductionKind::Invoke { function, input } => {
                DefaultProductionPlanKind::Invoke {
                    function: *function,
                    input,
                }
            }
        }
    }
}

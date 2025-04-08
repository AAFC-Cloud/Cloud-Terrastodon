use crate::app::MyApp;
use std::fmt::Debug;

pub trait StateMutator: Debug + Send + Sync {
    fn mutate_state(self: Box<Self>, state: &mut MyApp);
}

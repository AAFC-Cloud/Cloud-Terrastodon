use crate::app::MyApp;

pub trait StateMutator: Send + Sync + std::fmt::Debug {
    fn mutate_state(self: Box<Self>, state: &mut MyApp);
}
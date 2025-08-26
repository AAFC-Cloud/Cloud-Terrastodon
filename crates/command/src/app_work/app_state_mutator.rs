pub trait StateMutator<T>: Send + std::fmt::Debug + 'static {
    fn mutate_state(self: Box<Self>, state: &mut T);
}

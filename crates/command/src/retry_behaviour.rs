/// Controls whether a failed command may enter the built-in retry paths.
#[derive(Clone, Copy, Default, Debug)]
pub enum RetryBehaviour {
    Fail,
    #[default]
    Retry,
}

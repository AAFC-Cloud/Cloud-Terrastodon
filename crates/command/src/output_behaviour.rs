/// Controls whether a command captures output or inherits terminal streams.
#[derive(Clone, Copy, Default, Debug)]
pub enum OutputBehaviour {
    Display,
    #[default]
    Capture,
}

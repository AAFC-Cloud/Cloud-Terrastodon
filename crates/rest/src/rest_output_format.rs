/// Presentation of a retained REST response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RestOutputFormat {
    #[default]
    Text,
    Json,
}

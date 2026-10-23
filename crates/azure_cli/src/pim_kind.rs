use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

pub(crate) enum PimKind {
    Entra,
    AzureRM,
}
impl Display for PimKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            PimKind::Entra => "entra",
            PimKind::AzureRM => "azurerm",
        })
    }
}

use crate::AddressPrefix;
use crate::AddressPrefixes;
use facet::Facet;

/// Wire representation of the singular and plural fields that form an [AddressPrefixes] value.
#[derive(Debug, Clone, PartialEq, Eq, Facet)]
pub struct AddressPrefixesProxy {
    #[facet(rename = "addressPrefix", default)]
    single: Option<String>,
    #[facet(rename = "addressPrefixes", default)]
    multiple: Option<Vec<String>>,
}

impl From<AddressPrefixesProxy> for AddressPrefixes {
    fn from(value: AddressPrefixesProxy) -> Self {
        let address_prefixes = value
            .single
            .into_iter()
            .chain(value.multiple.into_iter().flatten())
            .map(|prefix| AddressPrefix::from(prefix.as_str()))
            .collect();
        Self { address_prefixes }
    }
}

impl From<&AddressPrefixes> for AddressPrefixesProxy {
    fn from(value: &AddressPrefixes) -> Self {
        Self {
            single: None,
            multiple: Some(value.address_prefixes.iter().map(String::from).collect()),
        }
    }
}

use crate::AddressPrefix;

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(opaque, proxy = AddressPrefixesProxy)]
pub struct AddressPrefixes {
    pub address_prefixes: Vec<AddressPrefix>,
}

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
pub struct AddressPrefixesProxy {
    #[facet(rename = "addressPrefix", default)]
    single: Option<AddressPrefix>,
    #[facet(
        rename = "addressPrefixes",
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<AddressPrefix>
    )]
    multiple: Vec<AddressPrefix>,
}

impl From<AddressPrefixesProxy> for AddressPrefixes {
    fn from(value: AddressPrefixesProxy) -> Self {
        let mut result = Vec::new();
        if let Some(single) = value.single {
            result.push(single);
        }
        result.extend(value.multiple);
        AddressPrefixes {
            address_prefixes: result,
        }
    }
}

impl From<&AddressPrefixes> for AddressPrefixesProxy {
    fn from(value: &AddressPrefixes) -> Self {
        Self {
            single: None,
            multiple: value.address_prefixes.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trips() -> eyre::Result<()> {
        let prefixes = AddressPrefixes {
            address_prefixes: vec!["10.0.0.0/24".parse()?, "AzureLoadBalancer".parse()?],
        };
        let json = facet_json::to_string(&prefixes)?;
        let reparsed = facet_json::from_str::<AddressPrefixes>(&json)?;
        assert_eq!(prefixes, reparsed);
        Ok(())
    }

    #[test]
    fn parses_single_address_prefix_json() -> eyre::Result<()> {
        let prefixes: AddressPrefixes = facet_json::from_str(r#"{"addressPrefix":"10.0.0.0/24"}"#)?;
        assert_eq!(prefixes.address_prefixes, vec!["10.0.0.0/24".parse()?]);
        Ok(())
    }
}

use crate::AddressPrefix;
use crate::address_prefixes_proxy::AddressPrefixesProxy;
use facet::Facet;

#[derive(Debug, Clone, PartialEq, Eq, Facet)]
#[facet(proxy = AddressPrefixesProxy)]
pub struct AddressPrefixes {
    pub address_prefixes: Vec<AddressPrefix>,
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

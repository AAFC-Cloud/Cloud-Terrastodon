use hcl::edit::expr::Expression;
use hcl::edit::parser;
use hcl::edit::structure::Attribute;
use hcl::edit::structure::BlockBuilder;
use hcl_primitives::Ident;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TofuProviderKind {
    AzureRM,
    AzureAD,
    Other(String),
}
impl TofuProviderKind {
    pub fn provider_prefix(&self) -> &str {
        match self {
            TofuProviderKind::AzureRM => "azurerm",
            TofuProviderKind::AzureAD => "azuread",
            TofuProviderKind::Other(s) => s.as_str(),
        }
    }
}
impl std::fmt::Display for TofuProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.provider_prefix())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TofuProviderReference {
    Alias {
        kind: TofuProviderKind,
        name: String,
    },
    Default {
        kind: TofuProviderKind,
    },
    Inherited,
}
impl TofuProviderReference {
    pub fn kind(&self) -> Option<&TofuProviderKind> {
        match self {
            TofuProviderReference::Alias { kind, .. } => Some(kind),
            TofuProviderReference::Default { kind } => Some(kind),
            TofuProviderReference::Inherited => None,
        }
    }
    pub fn try_as_expression(&self) -> Option<Result<Expression, parser::Error>> {
        match self {
            TofuProviderReference::Alias { kind, name } => {
                Some(format!("{kind}.{name}").parse::<Expression>())
            }
            TofuProviderReference::Default { kind } => {
                Some(format!("{kind}").parse::<Expression>())
            }
            TofuProviderReference::Inherited => None,
        }
    }
    pub fn apply_to_builder(&self, builder: BlockBuilder) -> Result<BlockBuilder, parser::Error> {
        Ok(match self.try_as_expression() {
            Some(expr) => {
                let expr = expr?;
                let attr = Attribute::new(Ident::new("provider"), expr);
                builder.attribute(attr)
            }
            None => builder,
        })
    }
}

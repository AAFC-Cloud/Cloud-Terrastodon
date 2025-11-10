use crate::prelude::ProviderKind;
use eyre::bail;
use hcl::edit::expr::Expression;
use hcl::edit::expr::TraversalOperator;
use hcl::edit::parser;
use hcl::edit::structure::Attribute;
use hcl::edit::structure::Block;
use hcl::edit::structure::BlockBuilder;
use hcl_primitives::Ident;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum HclProviderReference {
    Alias { kind: ProviderKind, name: String },
    Default { kind: ProviderKind },
    Inherited,
}
impl HclProviderReference {
    pub fn kind(&self) -> Option<&ProviderKind> {
        match self {
            HclProviderReference::Alias { kind, .. } => Some(kind),
            HclProviderReference::Default { kind } => Some(kind),
            HclProviderReference::Inherited => None,
        }
    }
    pub fn try_as_expression(&self) -> Option<Result<Expression, parser::Error>> {
        match self {
            HclProviderReference::Alias { kind, name } => {
                Some(format!("{kind}.{name}").parse::<Expression>())
            }
            HclProviderReference::Default { kind } => Some(format!("{kind}").parse::<Expression>()),
            HclProviderReference::Inherited => None,
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
impl TryFrom<Block> for HclProviderReference {
    type Error = eyre::Error;
    // TODO: optimize this to take a reference instead of cloning the block; make it a custom trait
    fn try_from(block: Block) -> Result<Self, Self::Error> {
        let Some(provider_attr) = block.body.get_attribute("provider") else {
            match block.ident.to_string().as_str() {
                "resource" | "data" | "import" => {
                    return Ok(HclProviderReference::Inherited);
                }
                ident => {
                    bail!("Provider attribute not present in block with invalid ident {ident}");
                }
            }
        };
        let Some(traversal) = provider_attr.value.as_traversal() else {
            bail!("Provider attribute value isn't a traversal");
        };
        let Some(provider_kind) = traversal.expr.as_variable() else {
            bail!("Provider attribute traversal isn't a variable");
        };
        let provider_kind: ProviderKind = provider_kind.parse()?;
        let provider_alias =
            traversal
                .operators
                .first()
                .and_then(|operator| match operator.value() {
                    TraversalOperator::GetAttr(attr) => Some(attr.to_string()),
                    _ => None,
                });
        Ok(match provider_alias {
            Some(alias) => HclProviderReference::Alias {
                kind: provider_kind,
                name: alias,
            },
            None => HclProviderReference::Default {
                kind: provider_kind,
            },
        })
    }
}

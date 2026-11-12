#[cfg(test)]
use arbitrary::Arbitrary;
use cloud_terrastodon::app::App;
use cloud_terrastodon::app::ApplicationCli;
use cloud_terrastodon::app::CliOutput;
use cloud_terrastodon::app::GlobalArgs;
use cloud_terrastodon::app::Result;
use cloud_terrastodon::app::facet;
use cloud_terrastodon::app::figue;
use cloud_terrastodon::auth::CredentialAuthenticationProvider;
use cloud_terrastodon::azure::AzureResourceGroup;
use cloud_terrastodon::azure::AzureResourceGroupListRequest;
use cloud_terrastodon::azure::AzureResourceGroupName;
use cloud_terrastodon::execution::ExecutableRequest;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use facet::Facet;
use std::sync::Arc;

/// Audit Azure resource groups with shared application startup.
/// The commands and their data belong to this consumer, not Cloud Terrastodon.
#[derive(Facet)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Cli {
    #[facet(flatten)]
    pub global_args: GlobalArgs,
    #[facet(figue::subcommand)]
    pub command: Command,
}

impl ApplicationCli for Cli {
    fn global_args(&self) -> &GlobalArgs {
        &self.global_args
    }
}

#[derive(Facet)]
#[cfg_attr(test, derive(Arbitrary))]
#[repr(u8)]
pub enum Command {
    /// Partition resource groups by whether their names contain exactly five vowels.
    Audit(AuditArgs),
}

/// Select the tenant whose resource groups will be audited.
#[derive(Facet)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct AuditArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

/// Resource groups partitioned by the number of vowels in their names.
#[derive(Facet)]
pub struct ResourceGroupAudit {
    /// Groups whose names have exactly five ASCII vowels (`a`, `e`, `i`, `o`, `u`), ignoring case.
    pub with_five_vowels: Vec<Arc<AzureResourceGroup>>,
    /// All other resource groups, including names with more than five vowels.
    pub without_five_vowels: Vec<Arc<AzureResourceGroup>>,
}

fn has_five_vowels(name: &AzureResourceGroupName) -> bool {
    name.chars()
        .filter(|character| matches!(character.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u'))
        .count()
        == 5
}

fn main() -> Result<()> {
    App::new("field-notes", env!("CARGO_PKG_VERSION"))
        .authentication_provider(Arc::new(CredentialAuthenticationProvider::configured()))
        .run(|cli: Cli, context| async move {
            let Command::Audit(args) = cli.command;

            let executor = context.executor();
            let resource_groups = AzureResourceGroupListRequest {
                authentication: args.authentication.resolve(&executor).await?,
            }
            .execute(&executor)
            .await?;

            let mut audit = ResourceGroupAudit {
                with_five_vowels: Vec::new(),
                without_five_vowels: Vec::new(),
            };
            for resource_group in resource_groups {
                // This synchronous loop cannot yield to App's automatic poll checks.
                context.cancellation.bail_if_cancelled()?;
                if has_five_vowels(&resource_group.name) {
                    audit.with_five_vowels.push(resource_group);
                } else {
                    audit.without_five_vowels.push(resource_group);
                }
            }
            Ok(CliOutput::facet(audit))
        })
}

#[cfg(test)]
mod tests {
    use super::AzureResourceGroupName;
    use super::Cli;
    use super::has_five_vowels;
    use cloud_terrastodon::app::Result;
    use cloud_terrastodon::app::figue;

    #[test]
    fn to_args_consistency() -> Result<()> {
        figue::assert_to_args_consistency::<Cli>(Default::default())?;
        Ok(())
    }

    #[test]
    fn to_args_roundtrip() -> Result<()> {
        figue::assert_to_args_roundtrip::<Cli>(figue::TestToArgsRoundTrip {
            success_count_per_leaf: 128,
            max_attempts_per_leaf: 10_000,
            success_count_global: 128,
            max_attempts_global: 10_000,
            ..Default::default()
        })?;
        Ok(())
    }

    #[test]
    fn vowel_audit_counts_exactly_five_ascii_vowels_ignoring_case() {
        for (name, expected) in [
            ("rg-aeiou", true),
            ("RG-AEIOU", true),
            ("rg-aaaaa", true),
            ("rg-aEiOu-y-123", true),
            ("rg-aeio", false),
            ("rg-aeioua", false),
            ("rg-rhythm-123", false),
            ("rg-äëïöü", false),
        ] {
            let name = AzureResourceGroupName::try_from(name)
                .expect("valid synthetic resource group name");
            assert_eq!(has_five_vowels(&name), expected, "{name}");
        }
    }
}

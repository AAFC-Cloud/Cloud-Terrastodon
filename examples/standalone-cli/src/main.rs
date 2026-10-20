use cloud_terrastodon::app::App;
use cloud_terrastodon::app::ApplicationCli;
use cloud_terrastodon::app::CliOutput;
use cloud_terrastodon::app::GlobalArgs;
use cloud_terrastodon::app::Result;
use cloud_terrastodon::app::facet;
use cloud_terrastodon::app::figue;
use cloud_terrastodon::azure::AzureTenantArgument;
use cloud_terrastodon::azure::AzureTenantArgumentExt;
use cloud_terrastodon::azure::ResourceGroup;
use cloud_terrastodon::azure::ResourceGroupName;
use cloud_terrastodon::azure::fetch_all_resource_groups;

/// Audit Azure resource groups with shared application startup.
/// The commands and their data belong to this consumer, not Cloud Terrastodon.
#[derive(facet::Facet)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
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

#[derive(facet::Facet)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
#[repr(u8)]
pub enum Command {
    /// Partition resource groups by whether their names contain exactly five vowels.
    Audit(AuditArgs),
}

/// Select the tenant whose resource groups will be audited.
#[derive(facet::Facet)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
pub struct AuditArgs {
    /// Tenant ID, tracked alias, or `default`.
    /// Defaults to the authentication source's tenant or the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

/// Resource groups partitioned by the number of vowels in their names.
#[derive(facet::Facet)]
pub struct ResourceGroupAudit {
    /// Groups whose names have exactly five ASCII vowels (`a`, `e`, `i`, `o`, `u`), ignoring case.
    pub with_five_vowels: Vec<ResourceGroup>,
    /// All other resource groups, including names with more than five vowels.
    pub without_five_vowels: Vec<ResourceGroup>,
}

fn has_five_vowels(name: &ResourceGroupName) -> bool {
    name.chars()
        .filter(|character| matches!(character.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u'))
        .count()
        == 5
}

fn main() -> Result<()> {
    App::new("field-notes", env!("CARGO_PKG_VERSION")).run(|cli: Cli, context| async move {
        let Command::Audit(args) = cli.command;

        let auth = args
            .tenant
            .bind_auth_context(&context.auth, &context.cancellation)
            .await?;
        let resource_groups = fetch_all_resource_groups(&auth, &context.cancellation).await?;

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
        CliOutput::facet(audit).emit(cli.global_args.output_format, &context.cancellation)
    })
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use super::ResourceGroupName;
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
            let name =
                ResourceGroupName::try_from(name).expect("valid synthetic resource group name");
            assert_eq!(has_five_vowels(&name), expected, "{name}");
        }
    }
}

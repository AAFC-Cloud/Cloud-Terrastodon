use cloud_terrastodon::app::{App, ApplicationCli, CliOutput, GlobalArgs, Result, facet, figue};
use cloud_terrastodon::azure::{
    AzureTenantArgument, AzureTenantArgumentExt, ResourceGroup, ResourceGroupName,
    fetch_all_resource_groups,
};

/// Audit Azure resource groups with shared application startup.
/// The commands and their data belong to this consumer, not Cloud Terrastodon.
#[derive(facet::Facet)]
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
#[repr(u8)]
pub enum Command {
    /// Partition resource groups by whether their names contain exactly five vowels.
    Audit(AuditArgs),
}

/// Select the tenant whose resource groups will be audited.
#[derive(facet::Facet)]
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
        // Check before synchronous rendering, including when there were no groups.
        context.cancellation.bail_if_cancelled()?;
        CliOutput::facet(audit).emit(cli.global_args.output_format)
    })
}

#[cfg(test)]
mod tests {
    use super::ResourceGroupName;
    use super::has_five_vowels;

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

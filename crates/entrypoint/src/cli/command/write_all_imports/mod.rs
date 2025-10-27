use crate::noninteractive::prelude::write_imports_for_all_resource_groups;
use crate::noninteractive::prelude::write_imports_for_all_role_assignments;
use crate::noninteractive::prelude::write_imports_for_all_security_groups;
use clap::Args;
use eyre::Result;

/// Write Terraform import definitions for all supported resources.
#[derive(Args, Debug, Clone, Default)]
pub struct WriteAllImportsArgs;

impl WriteAllImportsArgs {
    pub async fn invoke(self) -> Result<()> {
        write_imports_for_all_resource_groups().await?;
        write_imports_for_all_security_groups().await?;
        write_imports_for_all_role_assignments().await?;
        Ok(())
    }
}

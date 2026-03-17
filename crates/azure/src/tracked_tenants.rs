use crate::prelude::az_account_list;
use cloud_terrastodon_azure_types::prelude::Account;
use cloud_terrastodon_azure_types::prelude::AzureTenantArgument;
use cloud_terrastodon_azure_types::prelude::AzureTenantAlias;
use cloud_terrastodon_azure_types::prelude::TenantId;
use cloud_terrastodon_pathing::AppDir;
use eyre::Context;
use eyre::bail;
use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;
use tracing::warn;

const ALIASES_FILE_NAME: &str = "aliases.txt";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TrackedTenant {
    pub tenant_id: TenantId,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TrackedTenantAlias {
    pub tenant_id: TenantId,
    pub alias: AzureTenantAlias,
    pub path: PathBuf,
}

pub fn tracked_tenants_dir() -> PathBuf {
    AppDir::Tenants.as_path_buf()
}

pub fn tracked_tenant_dir(tenant_id: TenantId) -> PathBuf {
    tracked_tenants_dir().join(tenant_id.to_string())
}

pub fn tracked_tenant_aliases_file(tenant_id: TenantId) -> PathBuf {
    tracked_tenant_dir(tenant_id).join(ALIASES_FILE_NAME)
}

pub fn tracked_tenant_aliases_file_for_alias(tenant_id: TenantId) -> PathBuf {
    tracked_tenant_aliases_file(tenant_id)
}

pub async fn list_tracked_tenants() -> eyre::Result<Vec<TrackedTenant>> {
    list_tracked_tenants_in(&tracked_tenants_dir()).await
}

pub async fn get_tracked_tenant(tenant_id: TenantId) -> eyre::Result<Option<TrackedTenant>> {
    get_tracked_tenant_in(&tracked_tenants_dir(), tenant_id).await
}

pub async fn add_tracked_tenant(tenant_id: TenantId) -> eyre::Result<TrackedTenant> {
    add_tracked_tenant_in(&tracked_tenants_dir(), tenant_id).await
}

pub async fn forget_tracked_tenant(tenant_id: TenantId) -> eyre::Result<Option<TrackedTenant>> {
    forget_tracked_tenant_in(&tracked_tenants_dir(), tenant_id).await
}

pub async fn resolve_tracked_tenant_argument(
    arg: AzureTenantArgument<'_>,
) -> eyre::Result<TenantId> {
    match arg {
        AzureTenantArgument::Id(id) => resolve_tracked_tenant_id(id).await,
        AzureTenantArgument::IdRef(id) => resolve_tracked_tenant_id(*id).await,
        AzureTenantArgument::Alias(alias) => resolve_tracked_tenant_alias(&alias).await,
        AzureTenantArgument::AliasRef(alias) => resolve_tracked_tenant_alias(alias).await,
    }
}

pub async fn list_tracked_tenant_aliases()
-> eyre::Result<Vec<(TrackedTenant, Vec<AzureTenantAlias>)>> {
    list_tracked_tenant_aliases_in(&tracked_tenants_dir()).await
}

pub async fn list_tracked_tenant_aliases_for(
    tenant_id: TenantId,
) -> eyre::Result<Vec<TrackedTenantAlias>> {
    ensure_tracked_tenant_exists(tenant_id).await?;
    list_tracked_tenant_aliases_for_in(&tracked_tenants_dir(), tenant_id).await
}

pub async fn add_tracked_tenant_aliases(
    tenant_id: TenantId,
    aliases: &[AzureTenantAlias],
) -> eyre::Result<Vec<TrackedTenantAlias>> {
    ensure_tracked_tenant_exists(tenant_id).await?;
    add_tracked_tenant_aliases_in(&tracked_tenants_dir(), tenant_id, aliases).await
}

pub async fn remove_tracked_tenant_aliases(
    tenant_id: TenantId,
    aliases: &[AzureTenantAlias],
) -> eyre::Result<Vec<TrackedTenantAlias>> {
    ensure_tracked_tenant_exists(tenant_id).await?;
    remove_tracked_tenant_aliases_in(&tracked_tenants_dir(), tenant_id, aliases).await
}

pub async fn discover_and_track_tenants() -> eyre::Result<Vec<TrackedTenant>> {
    let accounts = az_account_list().await?;
    discover_tracked_tenants_from_accounts(accounts).await
}

pub async fn discover_tracked_tenants_from_accounts(
    accounts: Vec<Account>,
) -> eyre::Result<Vec<TrackedTenant>> {
    discover_tracked_tenants_in(
        &tracked_tenants_dir(),
        accounts.into_iter().map(|account| account.tenant_id),
    )
    .await
}

async fn discover_tracked_tenants_in<I>(
    root: &Path,
    tenant_ids: I,
) -> eyre::Result<Vec<TrackedTenant>>
where
    I: IntoIterator<Item = TenantId>,
{
    let mut unique_tenant_ids = tenant_ids
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    unique_tenant_ids.sort_by_key(|tenant_id| tenant_id.to_string());

    let mut discovered = Vec::with_capacity(unique_tenant_ids.len());
    for tenant_id in unique_tenant_ids {
        discovered.push(add_tracked_tenant_in(root, tenant_id).await?);
    }

    Ok(discovered)
}

async fn list_tracked_tenants_in(root: &Path) -> eyre::Result<Vec<TrackedTenant>> {
    if !fs::try_exists(root).await? {
        return Ok(Vec::new());
    }

    let mut entries = fs::read_dir(root)
        .await
        .wrap_err_with(|| format!("Reading tracked tenants in {}", root.display()))?;
    let mut tenants = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let file_type = entry.file_type().await?;
        if !file_type.is_dir() {
            continue;
        }

        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            warn!(path=%path.display(), "Skipping tracked tenant directory with non-UTF-8 name");
            continue;
        };

        match name.parse::<TenantId>() {
            Ok(tenant_id) => tenants.push(TrackedTenant { tenant_id, path }),
            Err(error) => {
                warn!(path=%path.display(), %error, "Skipping tracked tenant directory with invalid tenant id name");
            }
        }
    }

    tenants.sort_by_key(|tenant| tenant.tenant_id.to_string());
    Ok(tenants)
}

async fn get_tracked_tenant_in(
    root: &Path,
    tenant_id: TenantId,
) -> eyre::Result<Option<TrackedTenant>> {
    let path = root.join(tenant_id.to_string());
    if !fs::try_exists(&path).await? {
        return Ok(None);
    }

    let metadata = fs::metadata(&path)
        .await
        .wrap_err_with(|| format!("Reading metadata for {}", path.display()))?;
    if !metadata.is_dir() {
        bail!(
            "Tracked tenant path exists but is not a directory: {}",
            path.display()
        );
    }

    Ok(Some(TrackedTenant { tenant_id, path }))
}

async fn add_tracked_tenant_in(root: &Path, tenant_id: TenantId) -> eyre::Result<TrackedTenant> {
    fs::create_dir_all(root)
        .await
        .wrap_err_with(|| format!("Creating tracked tenants root {}", root.display()))?;

    let path = root.join(tenant_id.to_string());
    if fs::try_exists(&path).await? {
        let metadata = fs::metadata(&path)
            .await
            .wrap_err_with(|| format!("Reading metadata for {}", path.display()))?;
        if !metadata.is_dir() {
            bail!(
                "Tracked tenant path exists but is not a directory: {}",
                path.display()
            );
        }
    } else {
        fs::create_dir_all(&path)
            .await
            .wrap_err_with(|| format!("Creating tracked tenant directory {}", path.display()))?;
    }

    Ok(TrackedTenant { tenant_id, path })
}

async fn forget_tracked_tenant_in(
    root: &Path,
    tenant_id: TenantId,
) -> eyre::Result<Option<TrackedTenant>> {
    let Some(tenant) = get_tracked_tenant_in(root, tenant_id).await? else {
        return Ok(None);
    };

    fs::remove_dir_all(&tenant.path).await.wrap_err_with(|| {
        format!(
            "Removing tracked tenant directory {}",
            tenant.path.display()
        )
    })?;
    Ok(Some(tenant))
}

async fn resolve_tracked_tenant_id(tenant_id: TenantId) -> eyre::Result<TenantId> {
    ensure_tracked_tenant_exists(tenant_id).await?;
    Ok(tenant_id)
}

async fn ensure_tracked_tenant_exists(tenant_id: TenantId) -> eyre::Result<()> {
    if get_tracked_tenant(tenant_id).await?.is_some() {
        Ok(())
    } else {
        bail!("Tracked tenant '{}' was not found.", tenant_id)
    }
}

async fn resolve_tracked_tenant_alias(
    alias: &AzureTenantAlias,
) -> eyre::Result<TenantId> {
    let tracked_tenants = list_tracked_tenant_aliases().await?;

    let exact_matches = tracked_tenants
        .iter()
        .filter(|(_, aliases)| aliases.iter().any(|current| current == alias))
        .collect::<Vec<_>>();

    match exact_matches.len() {
        1 => return Ok(exact_matches[0].0.tenant_id),
        n if n > 1 => {
            let tenant_ids = exact_matches
                .iter()
                .map(|(tenant, _)| tenant.tenant_id.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            bail!(
                "Tracked tenant alias '{}' matched multiple tenants: {}",
                alias,
                tenant_ids
            );
        }
        _ => {}
    }

    let tenant_id_matches = tracked_tenants
        .iter()
        .filter(|(tenant, _)| tenant.tenant_id.to_string().contains(alias.as_str()))
        .collect::<Vec<_>>();

    match tenant_id_matches.len() {
        1 => Ok(tenant_id_matches[0].0.tenant_id),
        0 => bail!("Tracked tenant alias '{}' was not found.", alias),
        _ => {
            let tenant_ids = tenant_id_matches
                .iter()
                .map(|(tenant, _)| tenant.tenant_id.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            bail!(
                "Tracked tenant alias '{}' matched multiple tenant ids: {}",
                alias,
                tenant_ids
            );
        }
    }
}

async fn list_tracked_tenant_aliases_for_in(
    root: &Path,
    tenant_id: TenantId,
) -> eyre::Result<Vec<TrackedTenantAlias>> {
    let aliases = read_tracked_tenant_aliases_in(root, tenant_id).await?;
    let path = tracked_tenant_aliases_file_in(root, tenant_id);
    Ok(aliases
        .into_iter()
        .map(|alias| TrackedTenantAlias {
            tenant_id,
            alias,
            path: path.clone(),
        })
        .collect())
}

async fn add_tracked_tenant_aliases_in(
    root: &Path,
    tenant_id: TenantId,
    aliases: &[AzureTenantAlias],
) -> eyre::Result<Vec<TrackedTenantAlias>> {
    let mut aliases = aliases.to_vec();
    aliases.sort();
    aliases.dedup();

    let mut current_aliases = read_tracked_tenant_aliases_in(root, tenant_id).await?;
    for alias in aliases {
        if current_aliases.iter().any(|current| *current == alias) {
            continue;
        }

        let existing_all = list_tracked_tenant_aliases_in(root).await?;
        if let Some(conflict) = existing_all.iter().find(|(tenant, current_aliases)| {
            tenant.tenant_id != tenant_id && current_aliases.iter().any(|current| *current == alias)
        }) {
            bail!(
                "Tracked tenant alias '{}' already belongs to tenant '{}'.",
                alias,
                conflict.0.tenant_id
            );
        }

        current_aliases.push(alias.clone());
    }

    current_aliases.sort();
    current_aliases.dedup();
    write_tracked_tenant_aliases_in(root, tenant_id, &current_aliases).await?;

    let path = tracked_tenant_aliases_file_in(root, tenant_id);
    Ok(current_aliases
        .into_iter()
        .map(|alias| TrackedTenantAlias {
            tenant_id,
            alias,
            path: path.clone(),
        })
        .collect())
}

async fn remove_tracked_tenant_aliases_in(
    root: &Path,
    tenant_id: TenantId,
    aliases: &[AzureTenantAlias],
) -> eyre::Result<Vec<TrackedTenantAlias>> {
    let mut current_aliases = read_tracked_tenant_aliases_in(root, tenant_id).await?;
    let mut requested = aliases.to_vec();
    requested.sort();
    requested.dedup();

    let mut removed = Vec::with_capacity(requested.len());
    for alias in requested {
        let Some(index) = current_aliases.iter().position(|current| *current == alias) else {
            bail!(
                "Tracked tenant alias '{}' was not found for tenant '{}'.",
                alias,
                tenant_id
            );
        };
        current_aliases.remove(index);
        removed.push(alias);
    }

    write_tracked_tenant_aliases_in(root, tenant_id, &current_aliases).await?;

    let path = tracked_tenant_aliases_file_in(root, tenant_id);
    Ok(removed
        .into_iter()
        .map(|alias| TrackedTenantAlias {
            tenant_id,
            alias,
            path: path.clone(),
        })
        .collect())
}

async fn list_tracked_tenant_aliases_in(
    root: &Path,
) -> eyre::Result<Vec<(TrackedTenant, Vec<AzureTenantAlias>)>> {
    let tenants = list_tracked_tenants_in(root).await?;
    let mut tracked_tenants = Vec::with_capacity(tenants.len());
    for tenant in tenants {
        let mut aliases = read_tracked_tenant_aliases_in(root, tenant.tenant_id).await?;
        aliases.sort();
        aliases.dedup();
        tracked_tenants.push((tenant, aliases));
    }
    Ok(tracked_tenants)
}

fn tracked_tenant_aliases_file_in(root: &Path, tenant_id: TenantId) -> PathBuf {
    root.join(tenant_id.to_string()).join(ALIASES_FILE_NAME)
}

async fn read_tracked_tenant_aliases_in(
    root: &Path,
    tenant_id: TenantId,
) -> eyre::Result<Vec<AzureTenantAlias>> {
    let file = tracked_tenant_aliases_file_in(root, tenant_id);
    if fs::try_exists(&file).await? {
        return read_tracked_tenant_aliases_file(&file).await;
    }

    Ok(Vec::new())
}

async fn read_tracked_tenant_aliases_file(
    file: &Path,
) -> eyre::Result<Vec<AzureTenantAlias>> {
    let content = fs::read_to_string(file)
        .await
        .wrap_err_with(|| format!("Reading tracked tenant aliases from {}", file.display()))?;
    Ok(parse_alias_lines(&content, file))
}

async fn write_tracked_tenant_aliases_in(
    root: &Path,
    tenant_id: TenantId,
    aliases: &[AzureTenantAlias],
) -> eyre::Result<()> {
    let path = tracked_tenant_aliases_file_in(root, tenant_id);
    if aliases.is_empty() {
        if fs::try_exists(&path).await? {
            fs::remove_file(&path).await.wrap_err_with(|| {
                format!("Removing tracked tenant aliases file {}", path.display())
            })?;
        }
        return Ok(());
    }

    fs::create_dir_all(path.parent().unwrap_or(root))
        .await
        .wrap_err_with(|| format!("Creating tracked tenant aliases parent {}", path.display()))?;
    let content = aliases
        .iter()
        .map(|alias| alias.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&path, format!("{content}\n"))
        .await
        .wrap_err_with(|| format!("Writing tracked tenant aliases file {}", path.display()))?;
    Ok(())
}

fn parse_alias_lines(content: &str, file: &Path) -> Vec<AzureTenantAlias> {
    let mut aliases = Vec::new();
    for (line_no, raw_line) in content.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        match line.parse::<AzureTenantAlias>() {
            Ok(alias) => aliases.push(alias),
            Err(error) => {
                warn!(file=%file.display(), line_no=line_no + 1, %error, "Skipping invalid alias line");
            }
        }
    }

    aliases
}

#[cfg(test)]
mod tests {
    use super::add_tracked_tenant_aliases_in;
    use super::add_tracked_tenant_in;
    use super::forget_tracked_tenant_in;
    use super::get_tracked_tenant_in;
    use super::list_tracked_tenant_aliases_for_in;
    use super::list_tracked_tenants_in;
    use super::remove_tracked_tenant_aliases_in;
    use super::resolve_tracked_tenant_alias;
    use super::tracked_tenant_aliases_file_in;
    use crate::tracked_tenants::discover_tracked_tenants_in;
    use cloud_terrastodon_azure_types::prelude::AzureTenantAlias;
    use cloud_terrastodon_azure_types::prelude::TenantId;
    use std::str::FromStr;
    use tempfile::tempdir;

    #[tokio::test]
    async fn it_lists_added_tenants() -> eyre::Result<()> {
        let temp = tempdir()?;
        let tenant_a = TenantId::from_str("11111111-1111-1111-1111-111111111111")?;
        let tenant_b = TenantId::from_str("22222222-2222-2222-2222-222222222222")?;

        add_tracked_tenant_in(temp.path(), tenant_b).await?;
        add_tracked_tenant_in(temp.path(), tenant_a).await?;

        let tenants = list_tracked_tenants_in(temp.path()).await?;
        assert_eq!(tenants.len(), 2);
        assert_eq!(tenants[0].tenant_id.to_string(), tenant_a.to_string());
        assert_eq!(tenants[1].tenant_id.to_string(), tenant_b.to_string());
        Ok(())
    }

    #[tokio::test]
    async fn it_gets_and_forgets_a_tracked_tenant() -> eyre::Result<()> {
        let temp = tempdir()?;
        let tenant_id = TenantId::from_str("33333333-3333-3333-3333-333333333333")?;

        assert!(
            get_tracked_tenant_in(temp.path(), tenant_id)
                .await?
                .is_none()
        );

        let created = add_tracked_tenant_in(temp.path(), tenant_id).await?;
        assert!(created.path.ends_with(tenant_id.to_string()));

        let fetched = get_tracked_tenant_in(temp.path(), tenant_id).await?;
        assert_eq!(fetched, Some(created.clone()));

        let forgotten = forget_tracked_tenant_in(temp.path(), tenant_id).await?;
        assert_eq!(forgotten, Some(created));
        assert!(
            get_tracked_tenant_in(temp.path(), tenant_id)
                .await?
                .is_none()
        );
        Ok(())
    }

    #[tokio::test]
    async fn it_adds_lists_resolves_and_removes_aliases() -> eyre::Result<()> {
        let temp = tempdir()?;
        let tenant_id = TenantId::from_str("44444444-4444-4444-4444-444444444444")?;
        add_tracked_tenant_in(temp.path(), tenant_id).await?;

        let aliases = vec![
            AzureTenantAlias::try_new("Prod")?,
            AzureTenantAlias::try_new("Prod.West")?,
        ];

        let added = add_tracked_tenant_aliases_in(temp.path(), tenant_id, &aliases).await?;
        assert_eq!(added.len(), 2);
        assert_eq!(added[0].alias.to_string(), "prod");
        assert_eq!(added[1].alias.to_string(), "prod.west");
        assert_eq!(
            added[0].path,
            tracked_tenant_aliases_file_in(temp.path(), tenant_id)
        );

        let listed = list_tracked_tenant_aliases_for_in(temp.path(), tenant_id).await?;
        assert_eq!(listed.len(), 2);
        assert_eq!(
            listed[0].path,
            tracked_tenant_aliases_file_in(temp.path(), tenant_id)
        );

        let resolved =
            resolve_tracked_tenant_alias(&AzureTenantAlias::try_new("PROD")?).await;
        assert!(resolved.is_err());

        let substring_resolved =
            resolve_tracked_tenant_alias(&AzureTenantAlias::try_new("4444")?).await;
        assert_eq!(substring_resolved?, tenant_id);

        let removed = remove_tracked_tenant_aliases_in(
            temp.path(),
            tenant_id,
            &[AzureTenantAlias::try_new("prod")?],
        )
        .await?;
        assert_eq!(removed.len(), 1);

        let listed = list_tracked_tenant_aliases_for_in(temp.path(), tenant_id).await?;
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].alias.to_string(), "prod.west");
        Ok(())
    }

    #[tokio::test]
    async fn it_rejects_alias_conflicts_across_tenants() -> eyre::Result<()> {
        let temp = tempdir()?;
        let tenant_a = TenantId::from_str("55555555-5555-5555-5555-555555555555")?;
        let tenant_b = TenantId::from_str("66666666-6666-6666-6666-666666666666")?;
        add_tracked_tenant_in(temp.path(), tenant_a).await?;
        add_tracked_tenant_in(temp.path(), tenant_b).await?;

        let alias = AzureTenantAlias::try_new("shared")?;
        add_tracked_tenant_aliases_in(temp.path(), tenant_a, std::slice::from_ref(&alias)).await?;
        assert!(
            add_tracked_tenant_aliases_in(temp.path(), tenant_b, &[alias])
                .await
                .is_err()
        );
        Ok(())
    }

    #[tokio::test]
    async fn it_discovers_unique_tenants_from_ids() -> eyre::Result<()> {
        let temp = tempdir()?;
        let tenant_a = TenantId::from_str("77777777-7777-7777-7777-777777777777")?;
        let tenant_b = TenantId::from_str("88888888-8888-8888-8888-888888888888")?;

        let discovered =
            discover_tracked_tenants_in(temp.path(), [tenant_a, tenant_b, tenant_a]).await?;

        assert_eq!(discovered.len(), 2);
        assert!(temp.path().join(tenant_a.to_string()).exists());
        assert!(temp.path().join(tenant_b.to_string()).exists());
        Ok(())
    }
}

use crate::GiteaInstanceUrl;
use crate::GiteaTenantAlias;
use crate::GiteaTenantArgument;
use crate::default_tenant::get_default_gitea_instance_url;
use crate::list_gitea_logins;
use cloud_terrastodon_pathing::AppDir;
use eyre::Context;
use eyre::bail;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;
use tracing::warn;

const ALIASES_FILE_NAME: &str = "aliases.txt";
const URL_FILE_NAME: &str = "url.txt";

#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GiteaTrackedTenant {
    pub url: GiteaInstanceUrl,
    pub aliases: Vec<GiteaTenantAlias>,
}

pub fn tracked_tenants_dir() -> PathBuf {
    AppDir::Config.join("gitea").join("tenants")
}

fn tracked_tenant_dir_for(root: &Path, url: &GiteaInstanceUrl) -> PathBuf {
    root.join(url.storage_key())
}

pub async fn list_tracked_tenants() -> eyre::Result<Vec<GiteaTrackedTenant>> {
    list_tracked_tenants_in(&tracked_tenants_dir()).await
}

pub async fn get_tracked_tenant(
    url: &GiteaInstanceUrl,
) -> eyre::Result<Option<GiteaTrackedTenant>> {
    get_tracked_tenant_in(&tracked_tenants_dir(), url).await
}

pub async fn add_tracked_tenant(url: GiteaInstanceUrl) -> eyre::Result<GiteaTrackedTenant> {
    add_tracked_tenant_in(&tracked_tenants_dir(), url).await
}

pub async fn forget_tracked_tenant(
    url: &GiteaInstanceUrl,
) -> eyre::Result<Option<GiteaTrackedTenant>> {
    forget_tracked_tenant_in(&tracked_tenants_dir(), url).await
}

pub async fn discover_and_track_tenants() -> eyre::Result<Vec<GiteaTrackedTenant>> {
    let logins = list_gitea_logins().await?;
    let mut tenants = Vec::with_capacity(logins.len());
    for login in logins {
        tenants.push(add_tracked_tenant(login.url).await?);
    }
    Ok(tenants)
}

#[expect(async_fn_in_trait)]
pub trait GiteaTenantAliasExt {
    async fn resolve(&self) -> eyre::Result<GiteaInstanceUrl>;
}

impl GiteaTenantAliasExt for GiteaTenantAlias {
    async fn resolve(&self) -> eyre::Result<GiteaInstanceUrl> {
        resolve_tracked_tenant_alias(self).await
    }
}

#[expect(async_fn_in_trait)]
pub trait GiteaTenantArgumentExt {
    async fn resolve(&self) -> eyre::Result<GiteaInstanceUrl>;
}

impl GiteaTenantArgumentExt for GiteaTenantArgument<'_> {
    async fn resolve(&self) -> eyre::Result<GiteaInstanceUrl> {
        match self {
            GiteaTenantArgument::Default => get_default_gitea_instance_url().await,
            GiteaTenantArgument::Url(url) => resolve_tracked_tenant_url(url).await,
            GiteaTenantArgument::UrlRef(url) => resolve_tracked_tenant_url(url).await,
            GiteaTenantArgument::Alias(alias) => alias.resolve().await,
            GiteaTenantArgument::AliasRef(alias) => alias.resolve().await,
        }
    }
}

pub async fn list_tracked_tenant_aliases()
-> eyre::Result<HashMap<GiteaInstanceUrl, Vec<GiteaTenantAlias>>> {
    let tenants = list_tracked_tenants().await?;
    Ok(tenants
        .into_iter()
        .map(|tenant| (tenant.url, tenant.aliases))
        .collect())
}

pub async fn list_tracked_tenant_aliases_for(
    url: &GiteaInstanceUrl,
) -> eyre::Result<Vec<GiteaTenantAlias>> {
    ensure_tracked_tenant_exists(url).await?;
    let tenant = get_tracked_tenant(url)
        .await?
        .ok_or_else(|| eyre::eyre!("Tracked Gitea tenant not found after validation"))?;
    Ok(tenant.aliases)
}

pub async fn add_tracked_tenant_aliases(
    url: &GiteaInstanceUrl,
    aliases: &[GiteaTenantAlias],
) -> eyre::Result<Vec<GiteaTenantAlias>> {
    ensure_tracked_tenant_exists(url).await?;
    add_tracked_tenant_aliases_in(&tracked_tenants_dir(), url, aliases).await
}

pub async fn remove_tracked_tenant_aliases(
    url: &GiteaInstanceUrl,
    aliases: &[GiteaTenantAlias],
) -> eyre::Result<Vec<GiteaTenantAlias>> {
    ensure_tracked_tenant_exists(url).await?;
    remove_tracked_tenant_aliases_in(&tracked_tenants_dir(), url, aliases).await
}

async fn list_tracked_tenants_in(root: &Path) -> eyre::Result<Vec<GiteaTrackedTenant>> {
    if !fs::try_exists(root).await? {
        return Ok(Vec::new());
    }

    let mut entries = fs::read_dir(root)
        .await
        .wrap_err_with(|| format!("Reading tracked Gitea tenants in {}", root.display()))?;
    let mut tenants = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        if !entry.file_type().await?.is_dir() {
            continue;
        }
        let path = entry.path();
        match read_tracked_tenant_dir(&path).await {
            Ok(tenant) => tenants.push(tenant),
            Err(error) => {
                warn!(path=%path.display(), %error, "Skipping invalid tracked Gitea tenant directory");
            }
        }
    }

    tenants.sort_by_key(|tenant| tenant.url.to_string());
    Ok(tenants)
}

async fn get_tracked_tenant_in(
    root: &Path,
    url: &GiteaInstanceUrl,
) -> eyre::Result<Option<GiteaTrackedTenant>> {
    let tenants = list_tracked_tenants_in(root).await?;
    Ok(tenants.into_iter().find(|tenant| tenant.url == *url))
}

async fn add_tracked_tenant_in(
    root: &Path,
    url: GiteaInstanceUrl,
) -> eyre::Result<GiteaTrackedTenant> {
    fs::create_dir_all(root)
        .await
        .wrap_err_with(|| format!("Creating tracked Gitea tenants root {}", root.display()))?;

    let path = root.join(url.storage_key());
    fs::create_dir_all(&path)
        .await
        .wrap_err_with(|| format!("Creating tracked Gitea tenant directory {}", path.display()))?;
    fs::write(path.join(URL_FILE_NAME), format!("{url}\n"))
        .await
        .wrap_err_with(|| format!("Writing tracked Gitea tenant URL file {}", path.display()))?;

    Ok(GiteaTrackedTenant {
        url,
        aliases: read_aliases_from_dir(&path).await?,
    })
}

async fn forget_tracked_tenant_in(
    root: &Path,
    url: &GiteaInstanceUrl,
) -> eyre::Result<Option<GiteaTrackedTenant>> {
    let tenants = list_tracked_tenants_in(root).await?;
    let Some(tenant) = tenants.into_iter().find(|tenant| tenant.url == *url) else {
        return Ok(None);
    };
    let path = root.join(url.storage_key());
    if fs::try_exists(&path).await? {
        fs::remove_dir_all(&path).await.wrap_err_with(|| {
            format!("Removing tracked Gitea tenant directory {}", path.display())
        })?;
    }
    Ok(Some(tenant))
}

async fn resolve_tracked_tenant_url(url: &GiteaInstanceUrl) -> eyre::Result<GiteaInstanceUrl> {
    ensure_tracked_tenant_exists(url).await?;
    Ok(url.clone())
}

async fn ensure_tracked_tenant_exists(url: &GiteaInstanceUrl) -> eyre::Result<()> {
    if get_tracked_tenant(url).await?.is_some() {
        Ok(())
    } else {
        bail!(
            "Tracked Gitea tenant '{}' was not found. Use `cloud_terrastodon tea tenant add {url}` to allow Cloud Terrastodon to interact with this instance.",
            url
        )
    }
}

async fn resolve_tracked_tenant_alias(alias: &GiteaTenantAlias) -> eyre::Result<GiteaInstanceUrl> {
    resolve_tracked_tenant_alias_in(&tracked_tenants_dir(), alias).await
}

async fn resolve_tracked_tenant_alias_in(
    root: &Path,
    alias: &GiteaTenantAlias,
) -> eyre::Result<GiteaInstanceUrl> {
    let tenants = list_tracked_tenants_in(root).await?;

    let exact_matches = tenants
        .iter()
        .filter(|tenant| tenant.aliases.iter().any(|current| current == alias))
        .map(|tenant| tenant.url.clone())
        .collect::<Vec<_>>();
    match exact_matches.len() {
        1 => return Ok(exact_matches[0].clone()),
        n if n > 1 => {
            let urls = exact_matches
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            bail!(
                "Tracked Gitea tenant alias '{}' matched multiple tenants: {}",
                alias,
                urls
            );
        }
        _ => {}
    }

    let substring_matches = tenants
        .iter()
        .filter(|tenant| {
            tenant
                .url
                .to_string()
                .to_ascii_lowercase()
                .contains(alias.as_ref())
        })
        .map(|tenant| tenant.url.clone())
        .collect::<Vec<_>>();
    match substring_matches.len() {
        1 => Ok(substring_matches[0].clone()),
        0 => bail!("Tracked Gitea tenant alias '{}' was not found.", alias),
        _ => {
            let urls = substring_matches
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            bail!(
                "Tracked Gitea tenant alias '{}' matched multiple instance URLs: {}",
                alias,
                urls
            );
        }
    }
}

async fn add_tracked_tenant_aliases_in(
    root: &Path,
    url: &GiteaInstanceUrl,
    aliases: &[GiteaTenantAlias],
) -> eyre::Result<Vec<GiteaTenantAlias>> {
    let path = tracked_tenant_dir_for(root, url);
    let mut current_aliases = read_aliases_from_dir(&path).await?;
    let all_aliases = list_tracked_tenants_in(root).await?;

    let mut requested = aliases.to_vec();
    requested.sort();
    requested.dedup();

    for alias in requested {
        if current_aliases.contains(&alias) {
            continue;
        }
        if let Some(conflict) = all_aliases
            .iter()
            .find(|tenant| tenant.url != *url && tenant.aliases.contains(&alias))
        {
            bail!(
                "Tracked Gitea tenant alias '{}' already belongs to '{}'.",
                alias,
                conflict.url
            );
        }
        current_aliases.push(alias);
    }

    current_aliases.sort();
    current_aliases.dedup();
    write_aliases_for_dir(&path, &current_aliases).await?;
    Ok(current_aliases)
}

async fn remove_tracked_tenant_aliases_in(
    _root: &Path,
    url: &GiteaInstanceUrl,
    aliases: &[GiteaTenantAlias],
) -> eyre::Result<Vec<GiteaTenantAlias>> {
    let path = tracked_tenant_dir_for(_root, url);
    let mut current_aliases = read_aliases_from_dir(&path).await?;
    let mut requested = aliases.to_vec();
    requested.sort();
    requested.dedup();

    let mut removed = Vec::with_capacity(requested.len());
    for alias in requested {
        let Some(index) = current_aliases.iter().position(|current| *current == alias) else {
            bail!(
                "Tracked Gitea tenant alias '{}' was not found for '{}'.",
                alias,
                url
            );
        };
        removed.push(current_aliases.remove(index));
    }

    write_aliases_for_dir(&path, &current_aliases).await?;
    Ok(removed)
}

async fn read_tracked_tenant_dir(path: &Path) -> eyre::Result<GiteaTrackedTenant> {
    let url = read_url_from_dir(path).await?;
    let aliases = read_aliases_from_dir(path).await?;
    Ok(GiteaTrackedTenant { url, aliases })
}

async fn read_url_from_dir(path: &Path) -> eyre::Result<GiteaInstanceUrl> {
    let file = path.join(URL_FILE_NAME);
    let content = fs::read_to_string(&file)
        .await
        .wrap_err_with(|| format!("Reading tracked Gitea tenant URL from {}", file.display()))?;
    content.trim().parse()
}

async fn read_aliases_from_dir(path: &Path) -> eyre::Result<Vec<GiteaTenantAlias>> {
    let file = path.join(ALIASES_FILE_NAME);
    if !fs::try_exists(&file).await? {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&file).await.wrap_err_with(|| {
        format!(
            "Reading tracked Gitea tenant aliases from {}",
            file.display()
        )
    })?;
    Ok(parse_alias_lines(&content, &file))
}

async fn write_aliases_for_dir(path: &Path, aliases: &[GiteaTenantAlias]) -> eyre::Result<()> {
    let file = path.join(ALIASES_FILE_NAME);
    if aliases.is_empty() {
        if fs::try_exists(&file).await? {
            fs::remove_file(&file).await.wrap_err_with(|| {
                format!("Removing tracked Gitea alias file {}", file.display())
            })?;
        }
        return Ok(());
    }
    let content = aliases
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&file, format!("{content}\n"))
        .await
        .wrap_err_with(|| format!("Writing tracked Gitea alias file {}", file.display()))?;
    Ok(())
}

fn parse_alias_lines(content: &str, file: &Path) -> Vec<GiteaTenantAlias> {
    let mut aliases = Vec::new();
    for (line_no, raw_line) in content.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match line.parse::<GiteaTenantAlias>() {
            Ok(alias) => aliases.push(alias),
            Err(error) => {
                warn!(file=%file.display(), line_no=line_no + 1, %error, "Skipping invalid Gitea tenant alias");
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
    use super::list_tracked_tenants_in;
    use super::remove_tracked_tenant_aliases_in;
    use crate::GiteaInstanceUrl;
    use crate::GiteaTenantAlias;
    use tempfile::tempdir;

    #[tokio::test]
    async fn it_adds_lists_and_forgets_tenants() -> eyre::Result<()> {
        let temp = tempdir()?;
        let url = GiteaInstanceUrl::try_new("https://gitea.example.com")?;
        add_tracked_tenant_in(temp.path(), url.clone()).await?;

        let tenants = list_tracked_tenants_in(temp.path()).await?;
        assert_eq!(tenants.len(), 1);
        assert_eq!(tenants[0].url, url);

        let forgotten = forget_tracked_tenant_in(temp.path(), &url).await?;
        assert!(forgotten.is_some());
        assert!(list_tracked_tenants_in(temp.path()).await?.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn it_adds_and_removes_aliases() -> eyre::Result<()> {
        let temp = tempdir()?;
        let url = GiteaInstanceUrl::try_new("https://gitea.example.com")?;
        add_tracked_tenant_in(temp.path(), url.clone()).await?;

        let aliases = vec![
            GiteaTenantAlias::try_new("prod")?,
            GiteaTenantAlias::try_new("main")?,
        ];
        let added = add_tracked_tenant_aliases_in(temp.path(), &url, &aliases).await?;
        assert_eq!(
            added,
            vec![
                GiteaTenantAlias::try_new("main")?,
                GiteaTenantAlias::try_new("prod")?,
            ]
        );

        let removed = remove_tracked_tenant_aliases_in(
            temp.path(),
            &url,
            &[GiteaTenantAlias::try_new("prod")?],
        )
        .await?;
        assert_eq!(removed, vec![GiteaTenantAlias::try_new("prod")?]);
        Ok(())
    }

    #[tokio::test]
    async fn it_resolves_aliases() -> eyre::Result<()> {
        let temp = tempdir()?;
        let url = GiteaInstanceUrl::try_new("https://gitea.example.com")?;
        add_tracked_tenant_in(temp.path(), url.clone()).await?;
        add_tracked_tenant_aliases_in(temp.path(), &url, &[GiteaTenantAlias::try_new("prod")?])
            .await?;

        let tenants = list_tracked_tenants_in(temp.path()).await?;
        assert_eq!(tenants[0].aliases, vec![GiteaTenantAlias::try_new("prod")?]);
        let resolved = super::resolve_tracked_tenant_alias_in(
            temp.path(),
            &GiteaTenantAlias::try_new("prod")?,
        )
        .await?;
        assert_eq!(resolved, url);
        Ok(())
    }
}

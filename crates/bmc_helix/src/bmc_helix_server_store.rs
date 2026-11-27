use crate::BmcHelixServerCatalog;
use cloud_terrastodon_bmc_helix_types::BmcHelixServer;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerName;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerUrl;
use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
use eyre::WrapErr;
use eyre::ensure;
use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::io::Write;
use std::path::PathBuf;

/// Stores saved Helix endpoints independently of account credentials.
///
/// Mutations take an OS lock and atomically replace the catalogue, so a CLI
/// command and a GUI instance can share this store.
#[derive(Debug, Clone)]
pub struct BmcHelixServerStore {
    root: PathBuf,
}

impl BmcHelixServerStore {
    pub fn configured() -> Self {
        Self::new(AppDir::Config.join("bmc-helix"))
    }

    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn load(&self) -> Result<BmcHelixServerCatalog> {
        let content = match std::fs::read(self.root.join("servers.json")) {
            Ok(content) => content,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return Ok(BmcHelixServerCatalog::default());
            }
            Err(error) => return Err(error).wrap_err("reading the Helix server catalogue"),
        };
        let catalogue: BmcHelixServerCatalog =
            facet_json::from_slice(&content).wrap_err("decoding the Helix server catalogue")?;
        catalogue.validate()?;
        Ok(catalogue)
    }

    pub fn list(&self) -> Result<Vec<BmcHelixServer>> {
        let catalogue = self.load()?;
        catalogue
            .servers
            .keys()
            .map(|name| catalogue.get(Some(name)))
            .collect()
    }

    pub fn get(&self, name: Option<&BmcHelixServerName>) -> Result<BmcHelixServer> {
        self.load()?.get(name)
    }

    pub fn add(&self, server: BmcHelixServer) -> Result<BmcHelixServer> {
        self.update(|catalogue| {
            ensure!(
                !catalogue.servers.contains_key(&server.name),
                "Helix server {:?} is already registered",
                server.name
            );
            catalogue
                .servers
                .insert(server.name.clone(), server.url.clone());
            if let Some(url) = &server.innovation_suite_url {
                catalogue
                    .innovation_suite_urls
                    .insert(server.name.clone(), url.clone());
            }
            Ok(server)
        })
    }

    pub fn remove(&self, name: &BmcHelixServerName) -> Result<BmcHelixServer> {
        self.update(|catalogue| {
            let server = catalogue.get(Some(name))?;
            catalogue.servers.remove(name);
            catalogue.innovation_suite_urls.remove(name);
            if catalogue.default.as_ref() == Some(name) {
                catalogue.default = None;
            }
            Ok(server)
        })
    }

    pub fn set_default(&self, name: &BmcHelixServerName) -> Result<BmcHelixServer> {
        self.update(|catalogue| {
            let server = catalogue.get(Some(name))?;
            catalogue.default = Some(name.clone());
            Ok(server)
        })
    }

    pub fn clear_default(&self) -> Result<Option<BmcHelixServer>> {
        self.update(|catalogue| {
            let previous = catalogue
                .default
                .as_ref()
                .map(|name| catalogue.get(Some(name)))
                .transpose()?;
            catalogue.default = None;
            Ok(previous)
        })
    }

    /// Choose the exact Innovation Suite HTTPS origin allowed to receive the
    /// AR System token. Clearing it limits authorization to the issuer again.
    pub fn set_innovation_suite_url(
        &self,
        name: &BmcHelixServerName,
        url: Option<BmcHelixServerUrl>,
    ) -> Result<BmcHelixServer> {
        self.update(|catalogue| {
            let mut server = catalogue.get(Some(name))?;
            match &url {
                Some(url) => {
                    catalogue
                        .innovation_suite_urls
                        .insert(name.clone(), url.clone());
                }
                None => {
                    catalogue.innovation_suite_urls.remove(name);
                }
            }
            server.innovation_suite_url = url;
            Ok(server)
        })
    }

    fn update<T>(&self, update: impl FnOnce(&mut BmcHelixServerCatalog) -> Result<T>) -> Result<T> {
        std::fs::create_dir_all(&self.root)
            .wrap_err("creating the Helix server catalogue directory")?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.root.join("servers.lock"))
            .wrap_err("opening the Helix server catalogue lock")?;
        lock.lock().wrap_err("locking the Helix server catalogue")?;
        let mut catalogue = self.load()?;
        let output = update(&mut catalogue)?;
        catalogue.validate()?;
        let mut temporary = tempfile::NamedTempFile::new_in(&self.root)?;
        temporary.write_all(&facet_json::to_vec(&catalogue)?)?;
        temporary.as_file().sync_all()?;
        temporary
            .persist(self.root.join("servers.json"))
            .map_err(|error| error.error)
            .wrap_err("publishing the Helix server catalogue")?;
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::BmcHelixServerStore;
    use cloud_terrastodon_bmc_helix_types::BmcHelixServer;

    #[test]
    fn resolves_only_explicit_server_defaults_and_clears_a_removed_default() -> eyre::Result<()> {
        let directory = tempfile::tempdir()?;
        let store = BmcHelixServerStore::new(directory.path().to_owned());
        let server = BmcHelixServer {
            name: "test".parse()?,
            url: "https://helix.example.invalid".parse()?,
            innovation_suite_url: None,
        };

        store.add(server.clone())?;
        assert!(store.get(None).is_err());
        assert_eq!(store.get(Some(&server.name))?, server);
        store.set_default(&server.name)?;
        assert_eq!(store.get(None)?, server);
        assert_eq!(store.remove(&server.name)?, server);
        assert!(store.load()?.default.is_none());
        Ok(())
    }

    #[test]
    fn another_name_cannot_register_the_same_origin() -> eyre::Result<()> {
        let directory = tempfile::tempdir()?;
        let store = BmcHelixServerStore::new(directory.path().to_owned());
        let server = BmcHelixServer {
            name: "test".parse()?,
            url: "https://helix.example.invalid".parse()?,
            innovation_suite_url: None,
        };
        store.add(server.clone())?;
        let duplicate = BmcHelixServer {
            name: "duplicate".parse()?,
            url: "https://HELIX.example.invalid:443/".parse()?,
            innovation_suite_url: None,
        };

        assert!(store.add(duplicate).is_err());
        assert_eq!(store.list()?, vec![server]);
        Ok(())
    }

    #[test]
    fn innovation_suite_origin_is_scoped_to_the_registered_ar_server() -> eyre::Result<()> {
        let directory = tempfile::tempdir()?;
        let store = BmcHelixServerStore::new(directory.path().to_owned());
        let server = store.add(BmcHelixServer {
            name: "test".parse()?,
            url: "https://ar.example.invalid".parse()?,
            innovation_suite_url: None,
        })?;
        let target = "https://innovation.example.invalid".parse()?;
        assert_eq!(server.innovation_suite_origin(), &server.url);

        store.set_innovation_suite_url(&server.name, Some(target))?;
        let configured = store.get(Some(&server.name))?;
        assert_eq!(configured.url, server.url);
        let catalogue = store.load()?;
        assert_eq!(
            catalogue.innovation_suite_url_for_origin(&server.url),
            Some(configured.innovation_suite_origin())
        );
        assert!(
            catalogue
                .innovation_suite_url_for_origin(&"https://other.example.invalid".parse()?)
                .is_none()
        );
        store.set_innovation_suite_url(&server.name, None)?;
        let cleared = store.get(Some(&server.name))?;
        assert_eq!(cleared.innovation_suite_origin(), &server.url);
        Ok(())
    }
}

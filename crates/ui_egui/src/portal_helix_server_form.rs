use cloud_terrastodon_bmc_helix_types::BmcHelixServer;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerUrl;
use eframe::egui::Ui;
use eyre::Result;
use eyre::WrapErr;

/// A draft holds editable text until the server registration validates.
#[derive(Default)]
pub(crate) struct PortalHelixServerForm {
    name: String,
    url: String,
    innovation_suite_url: String,
}

impl PortalHelixServerForm {
    pub fn show(&mut self, ui: &mut Ui) -> Result<Option<BmcHelixServer>> {
        let name_label = ui.label("Server name");
        ui.text_edit_singleline(&mut self.name)
            .labelled_by(name_label.id);
        let url_label = ui.label("AR System HTTPS origin");
        ui.text_edit_singleline(&mut self.url)
            .labelled_by(url_label.id);
        let innovation_suite_label = ui.label("Innovation Suite HTTPS origin (optional)");
        ui.text_edit_singleline(&mut self.innovation_suite_url)
            .labelled_by(innovation_suite_label.id);
        if !ui.button("Save server").clicked() {
            return Ok(None);
        }
        Ok(Some(BmcHelixServer {
            name: self
                .name
                .trim()
                .parse()
                .wrap_err("Enter a valid Helix server name")?,
            url: self
                .url
                .trim()
                .parse()
                .wrap_err("Enter a valid Helix server URL")?,
            innovation_suite_url: parse_innovation_suite_url(&self.innovation_suite_url)?,
        }))
    }
}

pub(crate) fn parse_innovation_suite_url(value: &str) -> Result<Option<BmcHelixServerUrl>> {
    let value = value.trim();
    (!value.is_empty())
        .then(|| value.parse())
        .transpose()
        .wrap_err("Enter a valid Innovation Suite HTTPS origin")
}

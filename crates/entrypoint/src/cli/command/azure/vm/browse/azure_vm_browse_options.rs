use strum::Display;
use strum::VariantArray;

#[derive(VariantArray, Display)]
pub enum AzureVmBrowseOption {
    Resources,
    Skus,
    Publishers,
}

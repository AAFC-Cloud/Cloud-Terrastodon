mod commands_config;
mod config;
mod egui_config;
mod work_dirs_config;

pub use commands_config::*;
pub use config::*;
pub use egui_config::*;
pub use work_dirs_config::*;
cloud_terrastodon_registry::register_thing!(CommandsConfig);
cloud_terrastodon_registry::register_thing!(EguiConfig);
cloud_terrastodon_registry::register_thing!(WorkDirsConfig);


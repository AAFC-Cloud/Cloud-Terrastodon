// Preserve the public entrypoint path while sharing the actual type with consumers.
pub use cloud_terrastodon_app::GlobalArgs;
pub use cloud_terrastodon_app::OutputFormat;

cloud_terrastodon_registry::register_thing!(GlobalArgs);
cloud_terrastodon_registry::register_arbitrary!(GlobalArgs);
cloud_terrastodon_registry::register_thing!(OutputFormat);
cloud_terrastodon_registry::register_arbitrary!(OutputFormat);

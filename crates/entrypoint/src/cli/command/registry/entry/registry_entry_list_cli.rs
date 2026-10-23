use arbitrary::Arbitrary;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_registry::Effect;
use cloud_terrastodon_registry::FunctionKind;
use cloud_terrastodon_registry::RegistryFunctionEntry;
use cloud_terrastodon_registry::known_functions;
use color_eyre::owo_colors::OwoColorize;
use color_eyre::owo_colors::Stream;
use color_eyre::owo_colors::Style;
use eyre::Result;
use std::io::Write;

/// List registry metadata without invoking any registered operation.
#[derive(Debug, Clone, Arbitrary, facet::Facet)]
pub struct RegistryEntryListArgs {
    /// Include only executable requests, excluding constructors and conversions.
    #[facet(figue::named, default)]
    pub requests_only: bool,

    /// Require this effect; entries declaring several effects match each one.
    #[facet(figue::named)]
    pub effect: Option<Effect>,

    /// Match input, output, label, trait contract, or source file, ignoring case.
    #[facet(figue::named)]
    pub filter: Option<String>,
}

impl RegistryEntryListArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        let filter = self.filter.map(|filter| filter.to_lowercase());
        let mut entries = Vec::new();
        for function in known_functions() {
            cancellation.bail_if_cancelled()?;
            if self.requests_only && function.kind != FunctionKind::AsyncInvoke {
                continue;
            }
            if self
                .effect
                .is_some_and(|effect| !function.effects.contains(&effect))
            {
                continue;
            }
            let entry = RegistryFunctionEntry::from(function);
            if let Some(filter) = &filter
                && ![
                    entry.input.as_str(),
                    entry.output.as_str(),
                    entry.label.as_str(),
                    entry.trait_contract.as_str(),
                    entry.registration_file.as_str(),
                ]
                .into_iter()
                .any(|value| value.to_lowercase().contains(filter.as_str()))
            {
                continue;
            }
            entries.push(entry);
        }
        // Include the full signature and registration site in the ordering.
        // Distinct registrations remain separate rows, including duplicates.
        entries.sort_by_cached_key(|entry| {
            (
                entry.input.clone(),
                entry.output.clone(),
                format!("{:?}", entry.kind),
                format!("{:?}", entry.receiver_mode),
                entry.label.clone(),
                entry.trait_contract.to_string(),
                entry.registration_file.clone(),
                entry.registration_line,
                format!("{:?}", entry.effects),
            )
        });
        cancellation.bail_if_cancelled()?;
        Ok(CliOutput::facet_with_text(
            entries,
            |entries, writer, cancellation| write_registry_entries(entries, writer, cancellation),
        ))
    }
}

fn write_registry_entries(
    entries: &[RegistryFunctionEntry],
    writer: &mut dyn Write,
    cancellation: &CancellationScope,
) -> Result<()> {
    if entries.is_empty() {
        writeln!(
            writer,
            "{}",
            "No registry entries matched.".if_supports_color(Stream::Stdout, |text| text.dimmed())
        )?;
        return Ok(());
    }
    writeln!(
        writer,
        "{}",
        format!("{:<20}  {:<12}  Input -> Output", "Effects", "Kind")
            .if_supports_color(Stream::Stdout, |text| text
                .style(Style::new().cyan().bold()))
    )?;
    for entry in entries {
        cancellation.bail_if_cancelled()?;
        let effects_width = if entry.effects.is_empty() {
            let (label, style) = if entry.kind == FunctionKind::AsyncInvoke {
                ("UNCLASSIFIED", Style::new().red().bold())
            } else {
                ("none", Style::new().dimmed())
            };
            write!(
                writer,
                "{}",
                label.if_supports_color(Stream::Stdout, |text| text.style(style))
            )?;
            label.len()
        } else {
            let mut width = 0;
            for (index, effect) in entry.effects.iter().enumerate() {
                if index > 0 {
                    write!(writer, ", ")?;
                    width += 2;
                }
                let label = format!("{effect:?}");
                let style = match effect {
                    Effect::Read => Style::new().green(),
                    Effect::Write => Style::new().red().bold(),
                    Effect::Solicit => Style::new().yellow().bold(),
                };
                write!(
                    writer,
                    "{}",
                    label.if_supports_color(Stream::Stdout, |text| text.style(style))
                )?;
                width += label.len();
            }
            width
        };
        let kind = format!("{:?}", entry.kind);
        // Pad visible labels rather than strings containing ANSI escapes.
        let effects_padding = 20_usize.saturating_sub(effects_width);
        let kind_padding = 12_usize.saturating_sub(kind.len());
        writeln!(
            writer,
            "{:effects_padding$}  {}{:kind_padding$}  {} {} {}",
            "",
            kind.if_supports_color(Stream::Stdout, |text| text.magenta()),
            "",
            entry.input.if_supports_color(Stream::Stdout, |text| text
                .style(Style::new().cyan().bold())),
            "->".if_supports_color(Stream::Stdout, |text| text.dimmed()),
            entry
                .output
                .if_supports_color(Stream::Stdout, |text| text.cyan()),
        )?;
        let details = format!(
            "  {}:{} | {} ({}, {:?})",
            entry.registration_file,
            entry.registration_line,
            entry.label,
            entry.trait_contract,
            entry.receiver_mode,
        );
        writeln!(
            writer,
            "{}",
            details.if_supports_color(Stream::Stdout, |text| text.dimmed())
        )?;
    }
    Ok(())
}

cloud_terrastodon_registry::register_thing!(RegistryEntryListArgs);
cloud_terrastodon_registry::register_arbitrary!(RegistryEntryListArgs);

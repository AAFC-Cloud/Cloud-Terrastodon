use super::dump_plan_report::DumpPlanReport;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_registry::DiscoveryPlanNodeKind;
use color_eyre::owo_colors::OwoColorize;
use color_eyre::owo_colors::Stream;
use color_eyre::owo_colors::Style;
use eyre::Result;
use std::io::Write;

pub(super) fn write_dump_plan(
    report: &DumpPlanReport,
    writer: &mut dyn Write,
    cancellation: &CancellationScope,
) -> Result<()> {
    writeln!(
        writer,
        "Offline inventory plan (auth policy: {})",
        report.auth_source
    )?;
    if let Some(org) = &report.organization {
        writeln!(writer, "Organization: {org}")?;
    }
    if let Some(tenant) = &report.tenant {
        writeln!(writer, "Tenant: {tenant}")?;
    }
    for node in &report.plan.nodes {
        cancellation.bail_if_cancelled()?;
        if !node.target && node.kind != DiscoveryPlanNodeKind::Request {
            continue;
        }
        let status = if node.reachable {
            "possible"
        } else {
            "blocked"
        };
        let style = if node.reachable {
            Style::new().green()
        } else {
            Style::new().red()
        };
        writeln!(
            writer,
            "{}  {}{}",
            status.if_supports_color(Stream::Stdout, |text| text.style(style)),
            if node.target { "target: " } else { "request: " },
            node.label
        )?;
        for input in &node.missing_inputs {
            writeln!(writer, "    needs: {input}")?;
        }
    }
    for diagnostic in &report.plan.diagnostics {
        writeln!(writer, "Note: {diagnostic}")?;
    }
    if let Some(path) = &report.diagram {
        writeln!(writer, "Diagram: {}", path.display())?;
    }
    Ok(())
}

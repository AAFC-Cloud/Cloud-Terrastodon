//! Render the enabled icon implementations discovered through the registry.
//!
//! Run from the workspace with:
//! `cargo run -p cloud_terrastodon_iconography_cli --example gallery --offline`.
//! The optional positional argument selects the HTML output path.

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use cloud_terrastodon_iconography::registered_svg_functions;
// Link both collections so their enabled registrations reach the executable.
use cloud_terrastodon_iconography_azure as _;
use cloud_terrastodon_iconography_azure_devops as _;
use cloud_terrastodon_registry::Function;
use cloud_terrastodon_registry::FunctionTraitContract;
use cloud_terrastodon_registry::RuntimeValue;
use cloud_terrastodon_registry::describe_shape;
use cloud_terrastodon_registry::known_thing_for_shape;
use eyre::Result;
use eyre::WrapErr;
use eyre::ensure;
use facet::Shape;
use std::any::Any;
use std::borrow::Cow;
use std::fmt::Write;
use std::path::PathBuf;

fn main() -> Result<()> {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/iconography-gallery.html"));
    let functions = registered_svg_functions().collect::<Vec<_>>();
    let mut icons = Vec::new();
    for svg_function in functions
        .iter()
        .copied()
        .filter(|function| function.trait_contract == FunctionTraitContract::Other("SvgIcon::svg"))
    {
        let input_name = describe_shape(svg_function.input_shape);
        let thing = known_thing_for_shape(svg_function.input_shape)
            .ok_or_else(|| eyre::eyre!("Icon input {input_name} is not registered"))?;
        // Generated icons are default-constructible unit types. No constructors
        // that could consult configuration or services participate in discovery.
        let input = thing
            .runtime_into_boxed(
                RuntimeValue::from_default(svg_function.input_shape)
                    .wrap_err_with(|| format!("Constructing icon input {input_name}"))?,
            )
            .wrap_err_with(|| format!("Boxing icon input {input_name}"))?;
        let name = invoke_text(
            projection(&functions, svg_function.input_shape, "SvgIcon::name")?,
            input.as_ref(),
        )?;
        let category = invoke_text(
            projection(&functions, svg_function.input_shape, "SvgIcon::category")?,
            input.as_ref(),
        )?;
        let svg = invoke_text(svg_function, input.as_ref())?;
        icons.push((category, name, input_name, svg));
    }
    icons.sort_by(|left, right| (&left.0, &left.1, &left.2).cmp(&(&right.0, &right.1, &right.2)));

    let mut html = String::from(
        r#"<!doctype html>
<html lang="en">
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Cloud Terrastodon iconography</title>
<style>
body{font:16px system-ui,sans-serif;max-width:64rem;margin:3rem auto;padding:0 1rem;background:#f5f7fa;color:#172033}
main{display:flex;gap:1rem;flex-wrap:wrap}
figure{margin:0;padding:1.5rem;background:white;border:1px solid #d8e0eb;border-radius:.75rem;min-width:10rem;text-align:center}
img{width:96px;height:96px;object-fit:contain}
figcaption{margin-top:1rem}small{display:block;color:#536078;margin-top:.4rem}
</style>
<h1>Cloud Terrastodon iconography</h1>
<p>Enabled icons discovered from the linked collections, rendered offline.</p>
<main>
"#,
    );
    for (category, name, input_name, svg) in icons {
        let name = escape_html(&name);
        let category = escape_html(&category);
        let input_name = escape_html(&input_name);
        // Each document has its own image context, so upstream SVG IDs cannot collide.
        writeln!(
            html,
            r#"<figure><img alt="{}" src="data:image/svg+xml;base64,{}"><figcaption>{}<small>{} · {}</small></figcaption></figure>"#,
            name,
            BASE64_STANDARD.encode(svg.as_bytes()),
            name,
            input_name,
            category,
        )?;
    }
    html.push_str("</main>\n</html>\n");
    if let Some(parent) = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output, html)?;
    println!("Wrote {}", output.display());
    Ok(())
}

fn projection(
    functions: &[&'static Function],
    input_shape: &'static Shape,
    contract: &'static str,
) -> Result<&'static Function> {
    let candidates = functions
        .iter()
        .copied()
        .filter(|function| {
            function.input_shape.is_shape(input_shape)
                && function.trait_contract == FunctionTraitContract::Other(contract)
        })
        .collect::<Vec<_>>();
    ensure!(
        candidates.len() == 1,
        "Expected one {contract} projection for {}, found {}",
        describe_shape(input_shape),
        candidates.len()
    );
    Ok(candidates[0])
}

fn invoke_text(function: &Function, input: &(dyn Any + Send)) -> Result<Cow<'static, str>> {
    let output = function.invoke_ref_boxed(input).wrap_err_with(|| {
        format!(
            "Invoking {} for {}",
            function.trait_contract,
            describe_shape(function.input_shape),
        )
    })?;
    output
        .downcast::<Cow<'static, str>>()
        .map(|text| *text)
        .map_err(|_| {
            eyre::eyre!(
                "{} returned an unexpected output for {}",
                function.trait_contract,
                describe_shape(function.input_shape),
            )
        })
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

use crate::IconCatalogue;
use crate::IconCatalogueEntry;
use crate::IconFamily;
use eyre::Result;
use eyre::WrapErr;
use eyre::bail;
use eyre::ensure;
use quick_xml::NsReader;
use quick_xml::events::Event;
use quick_xml::name::ResolveResult;
use sha2::Digest;
use sha2::Sha256;
use std::collections::BTreeSet;
use std::path::Component;
use std::path::Path;

/// Check the generator's own input invariants without accessing files or services.
pub fn validate_catalogue(catalogue: &IconCatalogue) -> Result<()> {
    ensure!(
        catalogue.schema_version == 1,
        "unsupported icon catalogue schema version {}",
        catalogue.schema_version
    );
    let mut variants = BTreeSet::new();
    let mut files = BTreeSet::new();
    let mut aliases = BTreeSet::new();
    let mut logical_names = BTreeSet::new();
    let mut generated_types = BTreeSet::new();
    let mut generated_modules = BTreeSet::new();
    for entry in &catalogue.icons {
        syn::parse_str::<syn::Ident>(&entry.variant)
            .wrap_err_with(|| format!("invalid Rust variant {:?}", entry.variant))?;
        ensure!(
            entry.variant.chars().next().is_some_and(char::is_uppercase)
                && !entry.variant.starts_with("r#")
                && !["ALL", "svg", "name", "category", "for_resource_type"]
                    .contains(&entry.variant.as_str()),
            "variant {:?} must start with an uppercase letter and avoid generated API names",
            entry.variant
        );
        ensure!(
            variants.insert((entry.family, entry.variant.clone())),
            "duplicate {} variant {:?}",
            entry.family.as_str(),
            entry.variant
        );
        let generated_type = entry.family.icon_type_name(&entry.variant);
        let generated_module = entry.family.icon_module_name(&entry.variant);
        syn::parse_str::<syn::Ident>(&generated_type)
            .wrap_err_with(|| format!("invalid generated icon type {generated_type:?}"))?;
        syn::parse_str::<syn::Ident>(&generated_module)
            .wrap_err_with(|| format!("invalid generated icon module {generated_module:?}"))?;
        ensure!(
            generated_types.insert((entry.family, generated_type)),
            "conflicting generated icon type for {}",
            entry.variant
        );
        ensure!(
            generated_modules.insert((entry.family, generated_module)),
            "conflicting generated icon module for {}",
            entry.variant
        );
        ensure!(
            !entry.name.trim().is_empty(),
            "{} has an empty display name",
            entry.variant
        );
        ensure!(
            !entry.logical_name.trim().is_empty(),
            "{} has an empty upstream logical name",
            entry.variant
        );
        ensure!(
            logical_names.insert((entry.family, entry.logical_name.clone())),
            "duplicate {} upstream logical name {:?}",
            entry.family.as_str(),
            entry.logical_name
        );
        ensure!(
            valid_category(&entry.category),
            "invalid or reserved Cargo category {:?} for {}",
            entry.category,
            entry.variant
        );
        let mut components = Path::new(&entry.file).components();
        ensure!(
            matches!(components.next(), Some(Component::Normal(_)))
                && components.next().is_none()
                && entry.file.ends_with(".svg")
                && !entry
                    .file
                    .chars()
                    .any(|character| character.is_control() || "<>:\"/\\|?*".contains(character))
                && !entry.file.ends_with([' ', '.']),
            "SVG filename must be a single safe .svg component: {:?}",
            entry.file
        );
        ensure!(
            files.insert((entry.family, entry.file.to_ascii_lowercase())),
            "duplicate {} SVG filename {:?}",
            entry.family.as_str(),
            entry.file
        );
        ensure!(
            entry.sha256.len() == 64
                && entry
                    .sha256
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
            "{} must have a lowercase SHA-256 hex digest",
            entry.variant
        );
        for url in [&entry.source_url, &entry.discovery_url, &entry.terms_url] {
            ensure!(
                matches!(url.scheme(), "http" | "https")
                    && url.host_str().is_some()
                    && url.username().is_empty()
                    && url.password().is_none(),
                "{} provenance URL must be an HTTP(S) URL without credentials: {}",
                entry.variant,
                url
            );
        }
        ensure!(
            entry.family == IconFamily::Azure || entry.resource_types.is_empty(),
            "Azure DevOps asset {} cannot map Azure resource types",
            entry.variant
        );
        for alias in &entry.resource_types {
            ensure!(
                !alias.trim().is_empty() && alias == alias.trim() && alias.contains('/'),
                "invalid Azure resource-type alias {:?} for {}",
                alias,
                entry.variant
            );
            ensure!(
                aliases.insert(alias.to_ascii_lowercase()),
                "conflicting Azure resource-type alias {:?}",
                alias
            );
        }
    }
    Ok(())
}

fn valid_category(category: &str) -> bool {
    !["default", "all", "resource-types", "registry"].contains(&category)
        && category
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_lowercase)
        && category
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && !category.ends_with('-')
}

/// Verify the exact catalogue checksum and the self-contained SVG contract.
pub fn validate_asset(entry: &IconCatalogueEntry, bytes: &[u8]) -> Result<()> {
    let actual = format!("{:x}", Sha256::digest(bytes));
    ensure!(
        actual == entry.sha256,
        "SHA-256 mismatch for {}: catalogue {}, file {}",
        entry.variant,
        entry.sha256,
        actual
    );
    validate_svg(bytes, &entry.variant)
}

/// Validate artwork before it is embedded in an offline HTML consumer.
///
/// These are library packaging constraints, not upstream SVG platform guarantees:
/// reject active content, document types, and references that require external data.
/// CSS image()/image-set() functions, including -webkit-image-set(), are outside
/// this packaging subset because their string arguments can load external images.
/// Original accepted SVG bytes are preserved without sanitizing or rewriting them.
pub fn validate_svg(bytes: &[u8], label: &str) -> Result<()> {
    let text = std::str::from_utf8(bytes).wrap_err_with(|| format!("{label} SVG is not UTF-8"))?;
    let mut reader = NsReader::from_str(text);
    reader.config_mut().expand_empty_elements = true;
    let mut depth = 0usize;
    let mut saw_root = false;
    let mut style = None::<(usize, String)>;
    loop {
        match reader
            .read_event()
            .wrap_err_with(|| format!("parsing {label} SVG"))?
        {
            Event::Start(element) => {
                ensure!(
                    style.is_none(),
                    "{label} contains an element nested inside CSS style text"
                );
                let local = element.local_name();
                let name = std::str::from_utf8(local.as_ref())?;
                if depth == 0 {
                    ensure!(
                        !saw_root && name == "svg",
                        "{label} must contain exactly one SVG root element"
                    );
                    ensure!(
                        matches!(reader.resolver().resolve_element(element.name()).0,
                        ResolveResult::Bound(namespace) if namespace.as_ref() == b"http://www.w3.org/2000/svg"),
                        "{label} root must use the SVG namespace"
                    );
                    saw_root = true;
                }
                ensure!(
                    ![
                        "script",
                        "foreignObject",
                        "iframe",
                        "object",
                        "embed",
                        "image",
                        "animate",
                        "animateMotion",
                        "animateTransform",
                        "set"
                    ]
                    .contains(&name),
                    "{label} contains unsupported active or external-capable element {name}"
                );
                for attribute in element.attributes() {
                    let attribute =
                        attribute.wrap_err_with(|| format!("reading {label} SVG attribute"))?;
                    let local_key = attribute.key.local_name();
                    let key = std::str::from_utf8(local_key.as_ref())?;
                    let value = attribute.decode_and_unescape_value(reader.decoder())?;
                    ensure!(
                        !key.to_ascii_lowercase().starts_with("on"),
                        "{label} contains active event attribute {key}"
                    );
                    ensure!(
                        key != "href" || value.starts_with('#'),
                        "{label} contains an external href reference {value:?}"
                    );
                    ensure!(
                        attribute.key.as_ref() != b"xml:base",
                        "{label} contains xml:base which changes local reference resolution"
                    );
                    validate_css_references(&value, label)?;
                }
                depth += 1;
                if name == "style" {
                    style = Some((depth, String::new()));
                }
            }
            Event::End(_) => {
                if style
                    .as_ref()
                    .is_some_and(|(style_depth, _)| *style_depth == depth)
                {
                    validate_css_references(&style.take().expect("checked style depth").1, label)?;
                }
                depth = depth
                    .checked_sub(1)
                    .ok_or_else(|| eyre::eyre!("{label} SVG has an unmatched closing element"))?;
            }
            Event::DocType(_) => {
                bail!("{label} SVG contains a document type");
            }
            Event::PI(_) => {
                bail!("{label} SVG contains a processing instruction");
            }
            Event::Text(text) => {
                if depth == 0 {
                    ensure!(
                        text.xml_content()?.trim().is_empty(),
                        "{label} SVG contains text outside its root"
                    );
                }
                if let Some((_, css)) = &mut style {
                    css.push_str(&text.xml_content()?);
                }
            }
            Event::CData(text) => {
                ensure!(depth > 0, "{label} SVG contains CDATA outside its root");
                if let Some((_, css)) = &mut style {
                    css.push_str(&text.xml_content()?);
                }
            }
            Event::GeneralRef(reference) => {
                ensure!(
                    depth > 0,
                    "{label} SVG contains a reference outside its root"
                );
                let escaped = format!("&{};", reference.decode()?);
                let decoded = quick_xml::escape::unescape(&escaped)?;
                if let Some((_, css)) = &mut style {
                    css.push_str(&decoded);
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    ensure!(
        saw_root && depth == 0,
        "{label} SVG has no complete SVG root"
    );
    Ok(())
}

fn validate_css_references(value: &str, label: &str) -> Result<()> {
    let lower = value.to_ascii_lowercase();
    ensure!(
        !lower.contains("@import") && !lower.contains('\\'),
        "{label} contains unsupported CSS imports or escapes"
    );
    // String image sources do not require url() notation:
    // https://www.w3.org/TR/css-images-4/#image-set-notation
    // The image-set substring also rejects -webkit-image-set().
    ensure!(
        !["image(", "image-set("]
            .iter()
            .any(|function| lower.contains(function)),
        "{label} contains unsupported CSS string-image functions"
    );
    let mut tail = lower.as_str();
    while let Some(index) = tail.find("url(") {
        tail = &tail[index + 4..];
        let end = tail
            .find(')')
            .ok_or_else(|| eyre::eyre!("{label} contains an unterminated CSS URL"))?;
        let target = tail[..end].trim().trim_matches(['\'', '"']);
        ensure!(
            target.starts_with('#'),
            "{label} contains external CSS URL {target:?}"
        );
        tail = &tail[end + 1..];
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_asset;
    use super::validate_catalogue;
    use super::validate_svg;
    use crate::IconCatalogue;
    use crate::IconCatalogueEntry;
    use crate::IconFamily;
    use chrono::DateTime;
    use sha2::Digest;
    use sha2::Sha256;

    fn entry(variant: &str) -> IconCatalogueEntry {
        IconCatalogueEntry {
            family: IconFamily::Azure,
            variant: variant.into(),
            name: variant.into(),
            category: "resources".into(),
            file: format!("{}.svg", variant.to_ascii_lowercase()),
            source_url: "https://example.com/icon.svg".parse().unwrap(),
            logical_name: variant.into(),
            discovery_url: "https://example.com/manifest".parse().unwrap(),
            sha256: "0".repeat(64),
            retrieved_at: DateTime::from_timestamp(0, 0).unwrap(),
            terms_url: "https://example.com/terms".parse().unwrap(),
            resource_types: Vec::new(),
        }
    }

    #[test]
    fn resource_type_aliases_cannot_disagree_by_case() {
        let mut first = entry("ResourceGroup");
        first.resource_types = vec!["Microsoft.Resources/resourceGroups".into()];
        let mut second = entry("Subscription");
        second.resource_types = vec!["microsoft.resources/RESOURCEGROUPS".into()];
        let catalogue = IconCatalogue {
            schema_version: 1,
            icons: vec![first, second],
        };
        assert!(
            validate_catalogue(&catalogue)
                .unwrap_err()
                .to_string()
                .contains("conflicting")
        );
    }

    #[test]
    fn generated_names_and_paths_cannot_escape_their_contract() {
        let mut catalogue = IconCatalogue {
            schema_version: 1,
            icons: vec![entry("ResourceGroup")],
        };
        assert!(validate_catalogue(&catalogue).is_ok());
        for file in [
            "../outside.svg",
            "nested/file.svg",
            "image.svg:stream",
            "nested\\file.svg",
        ] {
            catalogue.icons[0].file = file.into();
            assert!(validate_catalogue(&catalogue).is_err(), "accepted {file:?}");
        }
        catalogue.icons[0].file = "resource-group.svg".into();
        for variant in ["Self", "ALL", "svg", "Invalid-Name"] {
            catalogue.icons[0].variant = variant.into();
            assert!(
                validate_catalogue(&catalogue).is_err(),
                "accepted {variant:?}"
            );
        }
        catalogue.icons[0].variant = "ResourceGroup".into();
        for category in [
            "all",
            "default",
            "resource-types",
            "registry",
            "Bad",
            "dep:crate",
        ] {
            catalogue.icons[0].category = category.into();
            assert!(
                validate_catalogue(&catalogue).is_err(),
                "accepted {category:?}"
            );
        }
    }

    #[test]
    fn distinct_icon_identities_must_not_generate_the_same_source_module() {
        let catalogue = IconCatalogue {
            schema_version: 1,
            icons: vec![entry("DNSZone"), entry("DnsZone")],
        };
        assert!(
            validate_catalogue(&catalogue)
                .unwrap_err()
                .to_string()
                .contains("generated icon module")
        );
    }

    #[test]
    fn local_gradients_and_inert_portal_css_are_valid_offline_artwork() {
        let svg = br##"<s:svg xmlns:s="http://www.w3.org/2000/svg"><s:style>.a { fill: url('#gradient'); } .a&gt;path { fill: red }</s:style><s:defs><s:linearGradient id="gradient"/></s:defs><s:path fill="url(#gradient)"/></s:svg>"##;
        assert!(validate_svg(svg, "local artwork").is_ok());
        let mut icon = entry("ResourceGroup");
        icon.sha256 = format!("{:x}", Sha256::digest(svg));
        assert!(validate_asset(&icon, svg).is_ok());
        assert!(
            validate_asset(&icon, b"changed")
                .unwrap_err()
                .to_string()
                .contains("SHA-256 mismatch")
        );
    }

    #[test]
    fn active_content_external_dependencies_and_broken_xml_are_rejected() {
        for body in [
            r#"<script>alert(1)</script>"#,
            r#"<path onclick="alert(1)"/>"#,
            r#"<use href="https://example.com/icon.svg#x"/>"#,
            r#"<path fill="url(https://example.com/paint)"/>"#,
            r#"<style>@import "https://example.com/style.css";</style>"#,
            r#"<style>.x { background-image: image-set("https://example.com/image.png" 1x); }</style>"#,
            r#"<path style="background-image: image('https://example.com/image.png')"/>"#,
            r#"<style>.x { fill: u&#114;l(https://example.com/paint) }</style>"#,
            r#"<style>.x { fill: ur<![CDATA[l(https://example.com/paint)]]> }</style>"#,
            r#"<path xml:base="https://example.com/" fill="url(#x)"/>"#,
            r#"<path></g>"#,
        ] {
            let svg = format!(r#"<svg xmlns="http://www.w3.org/2000/svg">{body}</svg>"#);
            assert!(
                validate_svg(svg.as_bytes(), "unsafe artwork").is_err(),
                "accepted {body}"
            );
        }
        for svg in [
            "<svg/>",
            r#"<svg xmlns="https://example.com"/>"#,
            r#"<!DOCTYPE svg><svg xmlns="http://www.w3.org/2000/svg"/>"#,
        ] {
            assert!(
                validate_svg(svg.as_bytes(), "invalid root").is_err(),
                "accepted {svg}"
            );
        }
    }
}

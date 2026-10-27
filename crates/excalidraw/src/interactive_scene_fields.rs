use eyre::Context;
use eyre::Result;
use facet::Facet;
use facet_json::RawJson;
use std::collections::BTreeMap;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

// Opaque fields deliberately retain their original JSON representation. An
// interactive edit must not drop future element kinds, images, customData,
// tombstones, or integer precision that this crate does not interpret.
pub(crate) type SceneFields = BTreeMap<String, RawJson<'static>>;

pub(crate) fn string(fields: &SceneFields, name: &str) -> Option<String> {
    // Excalidraw writes null for absent optional fields such as containerId.
    // Decoding directly as String coerces null into "null", which would turn
    // an unbound label into a reference to a nonexistent container.
    facet_json::from_str::<Option<String>>(fields.get(name)?.as_str())
        .ok()
        .flatten()
}

pub(crate) fn number(fields: &SceneFields, name: &str) -> Option<f64> {
    facet_json::from_str(fields.get(name)?.as_str()).ok()
}

pub(crate) fn deleted(fields: &SceneFields) -> bool {
    fields
        .get("isDeleted")
        .and_then(|value| facet_json::from_str::<bool>(value.as_str()).ok())
        .unwrap_or(false)
}

/// Generated log rows are output, even when an event quotes a control label.
/// Their native grouping is presentation ownership, not an input identifier.
pub(crate) fn log_output(fields: &SceneFields) -> bool {
    fields
        .get("groupIds")
        .and_then(|value| facet_json::from_str::<Vec<String>>(value.as_str()).ok())
        .is_some_and(|groups| groups.iter().any(|group| group.starts_with("ct-log-")))
}

pub(crate) fn set<'a, T: Facet<'a>>(fields: &mut SceneFields, name: &str, value: &T) -> Result<()> {
    fields.insert(
        name.into(),
        RawJson::from_owned(facet_json::to_string(value)?),
    );
    Ok(())
}

pub(crate) fn text(fields: &SceneFields) -> Option<String> {
    (string(fields, "type").as_deref() == Some("text"))
        .then(|| string(fields, "text"))
        .flatten()
}

/// Apply an element edit without losing opaque fields or bumping an unchanged
/// element's version. Validate the next version before replacing the original.
pub(crate) fn edit(
    element: &mut SceneFields,
    change: impl FnOnce(&mut SceneFields) -> Result<()>,
) -> Result<bool> {
    let mut updated = element.clone();
    change(&mut updated)?;
    if updated == *element {
        return Ok(false);
    }
    let version = element
        .get("version")
        .map(|value| facet_json::from_str::<u64>(value.as_str()))
        .transpose()
        .wrap_err("edited element version must be a nonnegative integer")?
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| {
            eyre::eyre!("edited element version cannot be incremented beyond u64::MAX")
        })?;
    set(&mut updated, "version", &version)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    set(&mut updated, "updated", &(now.as_millis() as u64))?;
    set(
        &mut updated,
        "versionNonce",
        &((now.as_nanos() as u32 ^ version as u32) & 0x7fff_ffff),
    )?;
    *element = updated;
    Ok(true)
}

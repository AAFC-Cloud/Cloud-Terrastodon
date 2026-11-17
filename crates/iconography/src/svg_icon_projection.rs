use crate::SvgIcon;
use eyre::Result;
use std::borrow::Cow;

/// Adapt SVG content to the registry's fallible projection contract.
#[doc(hidden)]
pub fn svg_icon_svg<T: SvgIcon>(icon: &T) -> Result<Cow<'static, str>> {
    Ok(icon.svg())
}

/// Adapt an icon's display name to the registry's projection contract.
#[doc(hidden)]
pub fn svg_icon_name<T: SvgIcon>(icon: &T) -> Result<Cow<'static, str>> {
    Ok(icon.name())
}

/// Adapt an icon's owning category to the registry's projection contract.
#[doc(hidden)]
pub fn svg_icon_category<T: SvgIcon>(icon: &T) -> Result<Cow<'static, str>> {
    Ok(icon.category())
}

/// Register an icon type and its SVG, name, and category projections.
///
/// The type implements `SvgIcon`, `Facet`, and `Arbitrary`. The registrations
/// use the existing type/function registry and declare no observable effects.
#[macro_export]
macro_rules! register_svg_icon {
    ($icon_ty:ty $(,)?) => {
        $crate::__registry::register_thing!($icon_ty);
        $crate::__registry::register_arbitrary!($icon_ty);
        $crate::__registry::register_fn_ref!(
            $icon_ty => ::std::borrow::Cow<'static, str>,
            kind = $crate::__registry::FunctionKind::Projection,
            label = "SVG",
            trait_contract = $crate::__registry::FunctionTraitContract::Other("SvgIcon::svg"),
            invoke = $crate::svg_icon_svg::<$icon_ty>,
            effects = []
        );
        $crate::__registry::register_fn_ref!(
            $icon_ty => ::std::borrow::Cow<'static, str>,
            kind = $crate::__registry::FunctionKind::Projection,
            label = "Name",
            trait_contract = $crate::__registry::FunctionTraitContract::Other("SvgIcon::name"),
            invoke = $crate::svg_icon_name::<$icon_ty>,
            effects = []
        );
        $crate::__registry::register_fn_ref!(
            $icon_ty => ::std::borrow::Cow<'static, str>,
            kind = $crate::__registry::FunctionKind::Projection,
            label = "Category",
            trait_contract = $crate::__registry::FunctionTraitContract::Other("SvgIcon::category"),
            invoke = $crate::svg_icon_category::<$icon_ty>,
            effects = []
        );
    };
}

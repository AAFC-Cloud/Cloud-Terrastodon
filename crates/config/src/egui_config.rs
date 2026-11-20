use crate::config::Config;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use emath::Pos2;
use emath::Rect;
use emath::Vec2;
use facet::Facet;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Facet, Clone, PartialEq)]
#[facet(opaque, proxy = EguiConfigProxy)]
pub struct EguiConfig {
    pub starting_points_window: Rect,
    pub open_dirs: HashMap<PathBuf, Rect>,
    /// Last Azure account and source selected in the window.
    pub azure_authentication: Option<AzureBearerAuthenticationSelection>,
    /// Mask email addresses in the account picker.
    pub demo_mode: bool,
}

#[derive(Debug, Arbitrary, Facet, Clone, PartialEq)]
struct EguiConfigProxy {
    pub starting_points_window: RectProxy,
    pub open_dirs: HashMap<PathBuf, RectProxy>,
    #[facet(default)]
    pub azure_authentication: Option<AzureBearerAuthenticationSelection>,
    #[facet(default)]
    pub demo_mode: bool,
}

#[derive(Debug, Arbitrary, Facet, Clone, Copy, PartialEq)]
struct RectProxy {
    pub min: Pos2Proxy,
    pub max: Pos2Proxy,
}

#[derive(Debug, Arbitrary, Facet, Clone, Copy, PartialEq)]
struct Pos2Proxy {
    pub x: f32,
    pub y: f32,
}

impl<'a> Arbitrary<'a> for EguiConfig {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(EguiConfigProxy::arbitrary(u)?.into())
    }
}

impl From<&Rect> for RectProxy {
    fn from(value: &Rect) -> Self {
        Self {
            min: (&value.min).into(),
            max: (&value.max).into(),
        }
    }
}

impl From<RectProxy> for Rect {
    fn from(value: RectProxy) -> Self {
        Rect::from_min_max(value.min.into(), value.max.into())
    }
}

impl From<&Pos2> for Pos2Proxy {
    fn from(value: &Pos2) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<Pos2Proxy> for Pos2 {
    fn from(value: Pos2Proxy) -> Self {
        Pos2::new(value.x, value.y)
    }
}

impl From<EguiConfigProxy> for EguiConfig {
    fn from(value: EguiConfigProxy) -> Self {
        Self {
            starting_points_window: value.starting_points_window.into(),
            open_dirs: value
                .open_dirs
                .into_iter()
                .map(|(path, rect)| (path, rect.into()))
                .collect(),
            azure_authentication: value.azure_authentication,
            demo_mode: value.demo_mode,
        }
    }
}

impl From<&EguiConfig> for EguiConfigProxy {
    fn from(value: &EguiConfig) -> Self {
        Self {
            starting_points_window: (&value.starting_points_window).into(),
            open_dirs: value
                .open_dirs
                .iter()
                .map(|(path, rect)| (path.clone(), rect.into()))
                .collect(),
            azure_authentication: value.azure_authentication,
            demo_mode: value.demo_mode,
        }
    }
}

impl Default for EguiConfig {
    fn default() -> Self {
        Self {
            starting_points_window: Rect::from_min_size(Pos2::new(10., 10.), Vec2::new(500., 500.)),
            open_dirs: HashMap::new(),
            azure_authentication: None,
            demo_mode: false,
        }
    }
}

#[async_trait::async_trait]
impl Config for EguiConfig {
    const FILE_SLUG: &'static str = "egui_ui_state";
}

cloud_terrastodon_registry::register_thing!(EguiConfig);
cloud_terrastodon_registry::register_arbitrary!(EguiConfig);

use crate::config::Config;
use emath::Pos2;
use emath::Rect;
use emath::Vec2;
use std::collections::HashMap;
use std::convert::Infallible;
use std::path::PathBuf;

#[derive(Debug, facet::Facet, Clone, PartialEq)]
#[facet(opaque, proxy = EguiConfigProxy)]
pub struct EguiConfig {
    pub starting_points_window: Rect,
    pub open_dirs: HashMap<PathBuf, Rect>,
}

#[derive(Debug, facet::Facet, Clone, PartialEq)]
struct EguiConfigProxy {
    pub starting_points_window: RectProxy,
    pub open_dirs: HashMap<PathBuf, RectProxy>,
}

#[derive(Debug, facet::Facet, Clone, Copy, PartialEq)]
struct RectProxy {
    pub min: Pos2Proxy,
    pub max: Pos2Proxy,
}

#[derive(Debug, facet::Facet, Clone, Copy, PartialEq)]
struct Pos2Proxy {
    pub x: f32,
    pub y: f32,
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

impl TryFrom<EguiConfigProxy> for EguiConfig {
    type Error = Infallible;

    fn try_from(value: EguiConfigProxy) -> Result<Self, Self::Error> {
        Ok(Self {
            starting_points_window: value.starting_points_window.into(),
            open_dirs: value
                .open_dirs
                .into_iter()
                .map(|(path, rect)| (path, rect.into()))
                .collect(),
        })
    }
}

impl TryFrom<&EguiConfig> for EguiConfigProxy {
    type Error = Infallible;

    fn try_from(value: &EguiConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            starting_points_window: (&value.starting_points_window).into(),
            open_dirs: value
                .open_dirs
                .iter()
                .map(|(path, rect)| (path.clone(), rect.into()))
                .collect(),
        })
    }
}

impl Default for EguiConfig {
    fn default() -> Self {
        Self {
            starting_points_window: Rect::from_min_size(Pos2::new(10., 10.), Vec2::new(500., 500.)),
            open_dirs: HashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl Config for EguiConfig {
    const FILE_SLUG: &'static str = "egui_ui_state";
}

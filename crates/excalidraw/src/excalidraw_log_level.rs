use facet::Facet;

/// Threshold selected by the visible `Log Level` marker on an interactive canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Facet)]
#[repr(u8)]
pub enum ExcalidrawLogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Off,
}

impl ExcalidrawLogLevel {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Debug => "Debug",
            Self::Info => "Info",
            Self::Warn => "Warn",
            Self::Error => "Error",
            Self::Off => "Off",
        }
    }

    pub(crate) fn stroke_color(self) -> &'static str {
        match self {
            Self::Debug => "#1971c2",
            Self::Info => "#2b8a3e",
            Self::Warn => "#9c6500",
            Self::Error => "#c92a2a",
            Self::Off => "#495057",
        }
    }

    pub(crate) fn background_color(self) -> &'static str {
        match self {
            Self::Debug => "#d0ebff",
            Self::Info => "#d3f9d8",
            Self::Warn => "#fff3bf",
            Self::Error => "#ffe3e3",
            Self::Off => "#e9ecef",
        }
    }

    pub(crate) fn from_label(label: &str) -> Option<Self> {
        match label {
            "Debug" => Some(Self::Debug),
            "Info" => Some(Self::Info),
            "Warn" => Some(Self::Warn),
            "Error" => Some(Self::Error),
            "Off" => Some(Self::Off),
            _ => None,
        }
    }
}

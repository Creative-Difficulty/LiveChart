use egui::Vec2;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct LiveChartAppData {
    // #[serde(skip)] // This how you opt-out of serialization of a field
    // value: f32,
    pub pixel_coords: Vec<PixelCoordinate>,
    #[serde(skip)]
    pub view_state: Option<ZoomState>,
}
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct CoordinatePair {
    pub pixels: PixelCoordinate,
    pub real: RealCoordinate,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct PixelCoordinate {
    pub x: f32,
    pub y: f32,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct RealCoordinate {
    pub lat: f64,
    pub lon: f64,
}

impl std::convert::From<egui::Pos2> for PixelCoordinate {
    fn from(value: egui::Pos2) -> Self {
        PixelCoordinate {
            x: value.x,
            y: value.y,
        }
    }
}

impl Default for PixelCoordinate {
    fn default() -> Self {
        PixelCoordinate { x: 0.0, y: 0.0 }
    }
}

impl Default for crate::app::LivechartApp {
    fn default() -> Self {
        Self {
            data: LiveChartAppData {
                pixel_coords: Vec::new(),
                view_state: None,
            },
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct ZoomState {
    pub scale: f32,
    pub offset: Vec2,
}

impl Default for ZoomState {
    fn default() -> Self {
        Self {
            scale: 1.0,
            offset: Vec2 { x: 0.0, y: 0.0 },
        }
    }
}

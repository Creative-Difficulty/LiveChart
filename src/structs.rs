use egui::Vec2;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct LivechartAppData {
    // #[serde(skip)] // This how you opt-out of serialization of a field
    // value: f32,
    pub pixels_coords: Vec<PixelCoordinate>,
    pub zoom_state: Option<ZoomState>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct PixelCoordinate {
    pub x: f32,
    pub y: f32,
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

impl Default for LivechartAppData {
    fn default() -> Self {
        Self {
            pixels_coords: Vec::new(),
            zoom_state: None,
        }
    }
}
#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct ZoomState {
    pub scale: f32,
    pub offset: Vec2,
}

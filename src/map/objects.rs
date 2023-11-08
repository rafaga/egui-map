use egui::{Color32, FontFamily, FontId, Pos2, Stroke};
use std::ops::{Div, Mul};

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct MapStyle {
    pub border: Option<Stroke>,
    pub line: Option<Stroke>,
    pub fill_color: Color32,
    pub text_color: Color32,
    pub font: Option<FontId>,
    pub background_color: Color32,
}

impl MapStyle {
    pub fn new() -> Self {
        MapStyle {
            border: None,
            line: None,
            fill_color: Color32::TRANSPARENT,
            text_color: Color32::TRANSPARENT,
            font: None,
            background_color: Color32::TRANSPARENT,
        }
    }
}

impl Default for MapStyle {
    fn default() -> Self {
        MapStyle::new()
    }
}

impl Mul<i64> for MapStyle {
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn mul(self, rhs: i64) -> Self::Output {
        let mut a = self.clone();
        a.border.unwrap().width *= rhs as f32;
        a.line.unwrap().width *= rhs as f32;
        a.font.as_mut().unwrap().size *= rhs as f32;
        a
    }
}

impl Mul<i32> for MapStyle {
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        let mut a = self.clone();
        a.border.unwrap().width *= rhs as f32;
        a.line.unwrap().width *= rhs as f32;
        a.font.as_mut().unwrap().size *= rhs as f32;
        a
    }
}

impl Mul<f32> for MapStyle {
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        let mut a = self.clone();
        a.border.unwrap().width *= rhs;
        a.line.unwrap().width *= rhs;
        a.font.as_mut().unwrap().size *= rhs;
        a
    }
}

impl Mul<f64> for MapStyle {
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        let mut a = self.clone();
        a.border.unwrap().width *= rhs as f32;
        a.line.unwrap().width *= rhs as f32;
        a.font.as_mut().unwrap().size *= rhs as f32;
        a
    }
}

impl Div<i64> for MapStyle {
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn div(self, rhs: i64) -> Self::Output {
        let mut a = self.clone();
        a.border.unwrap().width /= rhs as f32;
        a.line.unwrap().width /= rhs as f32;
        a.font.as_mut().unwrap().size /= rhs as f32;
        a
    }
}

impl Div<i32> for MapStyle {
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn div(self, rhs: i32) -> Self::Output {
        let mut a = self.clone();
        a.border.unwrap().width /= rhs as f32;
        a.line.unwrap().width /= rhs as f32;
        a.font.as_mut().unwrap().size /= rhs as f32;
        a
    }
}

impl Div<f32> for MapStyle {
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        let mut a = self.clone();
        a.border.unwrap().width /= rhs;
        a.line.unwrap().width /= rhs;
        a.font.as_mut().unwrap().size /= rhs;
        a
    }
}

impl Div<f64> for MapStyle {
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        let mut a = self.clone();
        a.border.unwrap().width /= rhs as f32;
        a.line.unwrap().width /= rhs as f32;
        a.font.as_mut().unwrap().size /= rhs as f32;
        a
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct MapLabel {
    pub text: String,
    pub center: Pos2,
}

impl Default for MapLabel {
    fn default() -> Self {
        MapLabel::new()
    }
}

impl MapLabel {
    pub fn new() -> Self {
        MapLabel {
            text: String::new(),
            center: Pos2::new(0.00, 0.00),
        }
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct MapLine {
    pub points: [Pos2; 2],
}

impl Default for MapLine {
    fn default() -> Self {
        MapLine::new()
    }
}

impl MapLine {
    pub fn new() -> Self {
        MapLine {
            points: [Pos2::new(0.00, 0.00), Pos2::new(0.00, 0.00)],
        }
    }
}

// This can by any object or point with its associated metadata
/// Struct that contains coordinates to help calculate nearest point in space
#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct MapPoint {
    dimension: usize,
    /// coordinates of the Solar System
    pub coords: [f64; 3],
    /// coordinates for lines connecting this point
    pub lines: Vec<[f64; 3]>,
    /// Object Identifier for search propurses
    pub id: usize,
    /// SolarSystem Name
    pub name: String,
}

impl MapPoint {
    /// Creates a new Spatial point with an Id (solarSystemId) and the system's 3D coordinates
    pub fn new(id: usize, coords: Vec<f64>) -> MapPoint {
        let mut point = [0.0f64; 3];
        let size = coords.len();
        point[0] = coords[0];
        point[1] = coords[1];
        if size == 3 {
            point[2] = coords[2];
        }
        MapPoint {
            coords: point,
            dimension: size,
            id,
            lines: Vec::new(),
            name: String::new(),
        }
    }

    /// Get the number of dimensions used in this object
    pub fn get_dimension(self) -> usize {
        self.dimension
    }
}

impl From<std::collections::hash_map::OccupiedEntry<'_, usize, MapPoint>> for MapPoint {
    fn from(value: std::collections::hash_map::OccupiedEntry<'_, usize, MapPoint>) -> Self {
        let k = value.get();
        k.clone()
    }
}

#[derive(Clone)]
pub(crate) struct MapBounds {
    pub min: Pos2,
    pub max: Pos2,
    pub pos: Pos2,
    pub dist: f64,
}

impl MapBounds {
    pub fn new() -> Self {
        MapBounds {
            min: Pos2::new(0.0, 0.0),
            max: Pos2::new(0.0, 0.0),
            pos: Pos2::new(0.0, 0.0),
            dist: 0.0,
        }
    }
}

impl Default for MapBounds {
    fn default() -> Self {
        MapBounds::new()
    }
}

pub struct MapSettings {
    pub max_zoom: f32,
    pub min_zoom: f32,
    pub line_visible_zoom: f32,
    pub label_visible_zoom: f32,
    pub node_text_visibility: VisibilitySetting,
    pub styles: Vec<MapStyle>,
}

impl MapSettings {
    pub fn new() -> Self {
        MapSettings {
            max_zoom: 0.0,
            min_zoom: 0.0,
            line_visible_zoom: 0.0,
            label_visible_zoom: 0.0,
            node_text_visibility: VisibilitySetting::Allways,
            styles: vec![MapStyle::new()],
        }
    }
}

impl Default for MapSettings {
    fn default() -> Self {
        let mut obj = MapSettings {
            max_zoom: 2.0,
            min_zoom: 0.1,
            line_visible_zoom: 0.2,
            label_visible_zoom: 0.58,
            node_text_visibility: VisibilitySetting::Allways,
            styles: Vec::new(),
        };

        // light Theme
        obj.styles.push(MapStyle {
            border: Some(egui::Stroke {
                width: 2f32,
                color: Color32::from_rgb(216, 142, 58),
            }),
            line: Some(egui::Stroke {
                width: 2f32,
                color: Color32::DARK_RED,
            }),
            fill_color: Color32::from_rgb(216, 142, 58),
            text_color: Color32::DARK_GREEN,
            font: Some(FontId::new(12.00, FontFamily::Proportional)),
            background_color: Color32::WHITE,
        });

        // Dark Theme
        obj.styles.push(MapStyle {
            border: Some(egui::Stroke {
                width: 2f32,
                color: Color32::GOLD,
            }),
            line: Some(egui::Stroke {
                width: 2f32,
                color: Color32::LIGHT_RED,
            }),
            fill_color: Color32::GOLD,
            text_color: Color32::LIGHT_GREEN,
            font: Some(FontId::new(12.00, FontFamily::Proportional)),
            background_color: Color32::DARK_GRAY,
        });
        obj
    }
}

#[derive(PartialEq)]
pub enum VisibilitySetting {
    Hidden,
    Hover,
    Allways,
}

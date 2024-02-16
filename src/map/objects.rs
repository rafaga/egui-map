use egui::{Align2, Color32, FontFamily, FontId, Pos2, Stroke};
use std::ops::{Div, Mul};

#[derive(Clone)]
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

    fn mul(mut self, rhs: i64) -> Self::Output {
        self.border.unwrap().width *= rhs as f32;
        self.line.unwrap().width *= rhs as f32;
        self.font.as_mut().unwrap().size *= rhs as f32;
        self
    }
}

impl Mul<i32> for MapStyle {
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn mul(mut self, rhs: i32) -> Self::Output {
        self.border.unwrap().width *= rhs as f32;
        self.line.unwrap().width *= rhs as f32;
        self.font.as_mut().unwrap().size *= rhs as f32;
        self
    }
}

impl Mul<f32> for MapStyle {
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn mul(mut self, rhs: f32) -> Self::Output {
        self.border.unwrap().width *= rhs;
        self.line.unwrap().width *= rhs;
        self.font.as_mut().unwrap().size *= rhs;
        self
    }
}

impl Mul<f64> for MapStyle {
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn mul(mut self, rhs: f64) -> Self::Output {
        self.border.unwrap().width *= rhs as f32;
        self.line.unwrap().width *= rhs as f32;
        self.font.as_mut().unwrap().size *= rhs as f32;
        self
    }
}

impl Div<i64> for MapStyle {
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn div(mut self, rhs: i64) -> Self::Output {
        self.border.unwrap().width /= rhs as f32;
        self.line.unwrap().width /= rhs as f32;
        self.font.as_mut().unwrap().size /= rhs as f32;
        self
    }
}

impl Div<i32> for MapStyle {
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn div(mut self, rhs: i32) -> Self::Output {
        self.border.unwrap().width /= rhs as f32;
        self.line.unwrap().width /= rhs as f32;
        self.font.as_mut().unwrap().size /= rhs as f32;
        self
    }
}

impl Div<f32> for MapStyle {
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn div(mut self, rhs: f32) -> Self::Output {
        self.border.unwrap().width /= rhs;
        self.line.unwrap().width /= rhs;
        self.font.as_mut().unwrap().size /= rhs;
        self
    }
}

impl Div<f64> for MapStyle {
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn div(mut self, rhs: f64) -> Self::Output {
        self.border.unwrap().width /= rhs as f32;
        self.line.unwrap().width /= rhs as f32;
        self.font.as_mut().unwrap().size /= rhs as f32;
        self
    }
}

#[derive(Clone)]
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

#[derive(Clone)]
pub struct MapLine {
    pub points: [Pos2; 2],
}

impl Default for MapLine {
    fn default() -> Self {
        MapLine::new(0.00, 0.00, 0.00, 0.00)
    }
}

impl MapLine {
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        MapLine {
            points: [Pos2::new(x1, y1), Pos2::new(x2, y2)],
        }
    }
}

// This can by any object or point with its associated metadata
/// Struct that contains coordinates to help calculate nearest point in space
#[derive(Clone)]
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

impl Mul<f64> for MapPoint {
    type Output = Self;

    fn mul(mut self, rhs: f64) -> Self::Output {
        self.coords[0] *= rhs;
        self.coords[1] *= rhs;
        self.coords[2] *= rhs;
        for indx in 0..self.lines.len() {
            self.lines[indx][0] *= rhs;
            self.lines[indx][1] *= rhs;
            self.lines[indx][2] *= rhs;
        }
        self
    }
}

impl Mul<f32> for MapPoint {
    type Output = Self;

    fn mul(mut self, rhs: f32) -> Self::Output {
        self.coords[0] *= rhs as f64;
        self.coords[1] *= rhs as f64;
        self.coords[2] *= rhs as f64;
        for indx in 0..self.lines.len() {
            self.lines[indx][0] *= rhs as f64;
            self.lines[indx][1] *= rhs as f64;
            self.lines[indx][2] *= rhs as f64;
        }
        self
    }
}

impl Div<f64> for MapPoint {
    type Output = Self;

    fn div(mut self, rhs: f64) -> Self::Output {
        self.coords[0] /= rhs;
        self.coords[1] /= rhs;
        self.coords[2] /= rhs;
        for indx in 0..self.lines.len() {
            self.lines[indx][0] /= rhs;
            self.lines[indx][1] /= rhs;
            self.lines[indx][2] /= rhs;
        }
        self
    }
}

impl Div<f32> for MapPoint {
    type Output = Self;

    fn div(mut self, rhs: f32) -> Self::Output {
        self.coords[0] /= rhs as f64;
        self.coords[1] /= rhs as f64;
        self.coords[2] /= rhs as f64;
        for indx in 0..self.lines.len() {
            self.lines[indx][0] /= rhs as f64;
            self.lines[indx][1] /= rhs as f64;
            self.lines[indx][2] /= rhs as f64;
        }
        self
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

pub(crate) struct TextSettings {
    pub position: Pos2,
    pub anchor: Align2,
    pub text: String,
    pub size: f32,
    pub family: FontFamily,
    pub text_color: Color32,
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

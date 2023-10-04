use std::ops::{Mul,Div};
use egui::{Stroke,Color32,FontId,FontFamily,Pos2};

#[derive(Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct MapStyle{
    pub border: Option<Stroke>,
    pub line: Option<Stroke>,
    pub fill_color: Color32,
    pub text_color: Color32,
    pub font: Option<FontId>,
}

impl Default for MapStyle{
    fn default() -> Self {
        MapStyle { 
            border: Some(egui::Stroke{ width: 2f32, color: Color32::GOLD}),
            line:  Some(egui::Stroke{ width: 2f32, color: Color32::DARK_RED}),
            fill_color: Color32::GOLD, 
            text_color: Color32::LIGHT_GREEN, 
            font: Some(FontId::new(12.00,FontFamily::Proportional)), 
        }
    }
}

impl MapStyle{
    pub fn new() -> Self {
        MapStyle { 
            border: None,
            line:  None,
            fill_color: Color32::TRANSPARENT, 
            text_color: Color32::TRANSPARENT, 
            font: None, 
        }
    }
}

impl Mul<i64> for MapStyle{
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn mul(self, rhs:i64) -> Self::Output {
        let mut a = self.clone();
        a.border.unwrap().width *= rhs as f32;
        a.line.unwrap().width *= rhs as f32;
        a.font.as_mut().unwrap().size *= rhs as f32;
        a
    }
}

impl Mul<i32> for MapStyle{
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn mul(self, rhs:i32) -> Self::Output {
        let mut a = self.clone();
        a.border.unwrap().width *= rhs as f32;
        a.line.unwrap().width *= rhs as f32;
        a.font.as_mut().unwrap().size *= rhs as f32;
        a
    }
}

impl Mul<f32> for MapStyle{
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn mul(self, rhs:f32) -> Self::Output {
        let mut a = self.clone();
        a.border.unwrap().width *= rhs;
        a.line.unwrap().width *= rhs;
        a.font.as_mut().unwrap().size *= rhs;
        a
    }
}

impl Mul<f64> for MapStyle{
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn mul(self, rhs:f64) -> Self::Output {
        let mut a = self.clone();
        a.border.unwrap().width *= rhs as f32;
        a.line.unwrap().width *= rhs as f32;
        a.font.as_mut().unwrap().size *= rhs as f32;
        a
    }
}

impl Div<i64> for MapStyle{
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn div(self, rhs:i64) -> Self::Output {
        let mut a = self.clone();
        a.border.unwrap().width /= rhs as f32;
        a.line.unwrap().width /= rhs as f32;
        a.font.as_mut().unwrap().size /= rhs as f32;
        a
    }
}

impl Div<i32> for MapStyle{
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn div(self, rhs:i32) -> Self::Output {
        let mut a = self.clone();
        a.border.unwrap().width /= rhs as f32;
        a.line.unwrap().width /= rhs as f32;
        a.font.as_mut().unwrap().size /= rhs as f32;
        a
    }
}

impl Div<f32> for MapStyle{
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn div(self, rhs:f32) -> Self::Output {
        let mut a = self.clone();
        a.border.unwrap().width /= rhs;
        a.line.unwrap().width /= rhs;
        a.font.as_mut().unwrap().size /= rhs;
        a
    }
}

impl Div<f64> for MapStyle{
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn div(self, rhs:f64) -> Self::Output {
        let mut a = self.clone();
        a.border.unwrap().width /= rhs as f32;
        a.line.unwrap().width /= rhs as f32;
        a.font.as_mut().unwrap().size /= rhs as f32;
        a
    }
}

#[derive(Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
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
            text:String::new(), 
            center: Pos2::new(0.00,0.00),
        }
    }
}

#[derive(Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct MapLine {
    pub points: [Pos2;2],
}

impl Default for MapLine{
    fn default() -> Self {
        MapLine::new()
    }
}

impl MapLine {
    pub fn new() -> Self {
        MapLine { 
            points: [Pos2::new(0.00,0.00),Pos2::new(0.00,0.00)], 
        }
    }
}

// This can by any object or point with its associated metadata
/// Struct that contains coordinates to help calculate nearest point in space
#[derive(Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct MapPoint{
    dimension: usize,
    /// coordinates of the Solar System
    pub coords: [f64;3],
    /// coordinates for lines connecting this point
    pub lines: Vec<[f64;3]>,
    /// Object Identifier for search propurses
    pub id: usize,
    /// SolarSystem Name
    pub name: String,
}

impl MapPoint{
    /// Creates a new Spatial point with an Id (solarSystemId) and the system's 3D coordinates
    pub fn new(id: usize, coords: Vec<f64>) -> MapPoint {
        let mut point = [0.0f64;3];
        let size= coords.len();
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


#[derive(Clone)]
pub (crate) struct MapBounds{
    pub min: Pos2,
    pub max: Pos2,
    pub pos: Pos2,
    pub dist: f64,
}

impl MapBounds{
    pub fn new() -> Self {
        MapBounds{
            min: Pos2::new(0.0,0.0),
            max: Pos2::new(0.0,0.0),
            pos: Pos2::new(0.0,0.0),
            dist: 0.0,
        }
    }
}

impl Default for MapBounds {
    fn default() -> Self {
        MapBounds::new()
    }
}

pub(crate) struct MapSettings {
    pub(crate) max_zoom:f32,
    pub(crate) min_zoom:f32,
    pub(crate) line_visible_zoom:f32,
    pub(crate) label_visible_zoom:f32,
}

impl MapSettings {
    pub(crate) fn new() -> Self {
        MapSettings{
            max_zoom:0.1,
            min_zoom:2.0,
            line_visible_zoom:0.2,
            label_visible_zoom:0.58
        }
    }
}
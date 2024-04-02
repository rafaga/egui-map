use egui::{Align2, Color32, FontFamily, FontId, Pos2, Stroke, Ui};
use std::io::ErrorKind;
use std::ops::{Div, DivAssign, Mul, MulAssign, Sub, Add};
use std::convert::{From,Into,TryInto};

#[derive(Copy,Clone)]
pub struct RawPoint{
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl RawPoint{
    pub fn new(x:f32, y:f32, z:f32) -> Self {
        Self {
            x,
            y,
            z
        }
    }

    pub(crate) fn from_pos2(input:Pos2, projected_axis:usize) -> Result<Self,std::io::Error> {
        match projected_axis {
            0 => {
                Ok(RawPoint {
                    x:0.0,
                    y:input.x,
                    z:input.y,
                })
            },
            1 => {
                Ok(RawPoint {
                    x:input.x,
                    y:0.0,
                    z:input.y,
                })
            },
            2 => {
                Ok(RawPoint {
                    x:input.x,
                    y:input.y,
                    z:0.0,
                })
            },
            _ => Err(std::io::Error::new(ErrorKind::Other, "Incorrect Projected Axis Parameter"))
        }
    }
}

impl Default for RawPoint{
    fn default() -> Self {
        Self::new(0.00,0.00,0.00)
    }
}

impl Mul<i64> for RawPoint {
    type Output = Self;

    fn mul(mut self, rhs: i64) -> Self::Output {
        self.x *= rhs as f32;
        self.y *= rhs as f32;
        self.z *= rhs as f32;
        self
    }
}

impl Mul<i32> for RawPoint {
    type Output = Self;

    fn mul(mut self, rhs: i32) -> Self::Output {
        self.x *= rhs as f32;
        self.y *= rhs as f32;
        self.z *= rhs as f32;
        self
    }
}

impl Mul<f32> for RawPoint {
    type Output = Self;

    fn mul(mut self, rhs: f32) -> Self::Output {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
        self
    }
}

impl MulAssign<i64> for RawPoint{
    fn mul_assign(&mut self, rhs: i64) {
        self.x = self.x * rhs as f32;
        self.y = self.y * rhs as f32;
        self.z = self.z * rhs as f32;
    }
}

impl MulAssign<i32> for RawPoint{
    fn mul_assign(&mut self, rhs: i32) {
        self.x = self.x * rhs as f32;
        self.y = self.y * rhs as f32;
        self.z = self.z * rhs as f32;
    }
}

impl MulAssign<f32> for RawPoint{
    fn mul_assign(&mut self, rhs: f32) {
        self.x = self.x * rhs;
        self.y = self.y * rhs;
        self.z = self.z * rhs;
    }
}

impl Div<i64> for RawPoint {
    type Output = Self;

    fn div(mut self, rhs: i64) -> Self::Output {
        self.x /= rhs as f32;
        self.y /= rhs as f32;
        self.z /= rhs as f32;
        self
    }
}

impl Div<i32> for RawPoint {
    type Output = Self;

    fn div(mut self, rhs: i32) -> Self::Output {
        self.x /= rhs as f32;
        self.y /= rhs as f32;
        self.z /= rhs as f32;
        self
    }
}

impl Div<f32> for RawPoint {
    type Output = Self;

    fn div(mut self, rhs: f32) -> Self::Output {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
        self
    }
}

impl DivAssign<i64> for RawPoint{
    fn div_assign(&mut self, rhs: i64) {
        self.x = self.x / rhs as f32;
        self.y = self.y / rhs as f32;
        self.z = self.z / rhs as f32;
    }
}

impl DivAssign<i32> for RawPoint{
    fn div_assign(&mut self, rhs: i32) {
        self.x = self.x / rhs as f32;
        self.y = self.y / rhs as f32;
        self.z = self.z / rhs as f32;
    }
}

impl DivAssign<f32> for RawPoint{
    fn div_assign(&mut self, rhs: f32) {
        self.x = self.x / rhs;
        self.y = self.y / rhs;
        self.z = self.z / rhs;
    }
}

impl Add<RawPoint> for RawPoint{
    type Output = RawPoint;
    fn add(self, rhs: RawPoint) -> Self::Output {
        RawPoint {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub<RawPoint> for RawPoint{
    type Output = RawPoint;
    fn sub(self, rhs: RawPoint) -> Self::Output {
        RawPoint {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Add<&RawPoint> for RawPoint{
    type Output = RawPoint;
    fn add(self, rhs: &RawPoint) -> Self::Output {
        RawPoint {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub<&RawPoint> for RawPoint{
    type Output = RawPoint;
    fn sub(self, rhs: &RawPoint) -> Self::Output {
        RawPoint {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl From<[f32;3]> for RawPoint{
    fn from(value: [f32;3]) -> Self {
        Self {
            x: value[0],
            y: value[1],
            z: value[2]
        }
    }
}

impl From<[i64;3]> for RawPoint{
    fn from(value: [i64;3]) -> Self {
        Self {
            x: value[0] as f32,
            y: value[1] as f32,
            z: value[2] as f32
        }
    }
}

impl From<[i32;3]> for RawPoint{
    fn from(value: [i32;3]) -> Self {
        Self {
            x: value[0] as f32,
            y: value[1] as f32,
            z: value[2] as f32
        }
    }
}

impl From<[i16;3]> for RawPoint{
    fn from(value: [i16;3]) -> Self {
        Self {
            x: value[0] as f32,
            y: value[1] as f32,
            z: value[2] as f32
        }
    }
}

impl From<[i8;3]> for RawPoint{
    fn from(value: [i8;3]) -> Self {
        Self {
            x: value[0] as f32,
            y: value[1] as f32,
            z: value[2] as f32
        }
    }
}

impl Into<[f32;3]> for RawPoint{
    fn into(self) -> [f32;3] {
        [self.x,self.y,self.z]
    }
}

impl TryInto<[i64;3]> for RawPoint{
    type Error = std::io::Error;

    fn try_into(self) -> Result<[i64;3], <RawPoint as TryInto<[i64;3]>>::Error> {
        if self.x > i64::MAX as f32 || self.y > i64::MAX as f32 || self.z > i64::MAX as f32{
            Err(std::io::Error::new(ErrorKind::Other,"Value overflow."))
        } else {
            Ok([self.x as i64,self.y as i64,self.z as i64])
        }
    }
}

impl TryInto<Pos2> for RawPoint{
    type Error = std::io::Error;

    fn try_into(self) -> Result<Pos2, <RawPoint as TryInto<Pos2>>::Error> {
        if self.x != 0.0 && self.y != 0.0 && self.z != 0.0 {
            Err(std::io::Error::new(ErrorKind::InvalidData,""))
        } else {
            if self.x == 0.0 {
                return Ok(Pos2::new(self.y,self.z));
            }
            if self.y == 0.0 {
                Ok(Pos2::new(self.x,self.z))
            } else {
                Ok(Pos2::new(self.x,self.y))
            }
        }
    }
}

impl TryInto<[f32;2]> for RawPoint{
    type Error = std::io::Error;

    fn try_into(self) -> Result<[f32;2], <RawPoint as TryInto<[f32;2]>>::Error> {
        if self.x != 0.0 && self.y != 0.0 && self.z != 0.0 {
            Err(std::io::Error::new(ErrorKind::InvalidData,""))
        } else {
            if self.x == 0.0 {
                return Ok([self.y,self.z]);
            }
            if self.y == 0.0 {
                Ok([self.x,self.z])
            } else {
                Ok([self.x,self.y])
            }
        }
    }
}

#[derive(Copy,Clone)]
pub struct RawLine {
    pub points:[RawPoint;2],
}

impl RawLine{
    pub fn new(a:RawPoint ,b:RawPoint) -> Self {
        Self{
            points:[a,b]
        }
    }

    pub fn distance(self) -> f32 {
        let x = self.points[0].x - self.points[1].x;
        let y = self.points[0].y - self.points[1].y;
        let z = self.points[0].z - self.points[1].z;
        (x.powi(2)+y.powi(2)+z.powi(2)).sqrt()
    }

    pub fn midpoint(self) -> RawPoint {
        let x = (self.points[0].x + self.points[1].x)/2.0;
        let y = (self.points[0].y + self.points[1].y)/2.0;
        let z = (self.points[0].z + self.points[1].z)/2.0;
        RawPoint::new(x,y,z)
    }
}

impl TryInto<[egui::Pos2;2]> for RawLine {
    type Error = std::io::Error;

    fn try_into(self) -> Result<[egui::Pos2; 2], <Self as TryInto<[egui::Pos2; 2]>>::Error> { 
        let position1 = self.points[0].try_into()?;
        let position2 = self.points[1].try_into()?;
        Ok([position1,position2])
    }
}

impl From<[[i64;3];2]> for RawLine {
    fn from(value: [[i64;3];2]) -> Self {
        Self{
            points: [RawPoint::from(value[0]),RawPoint::from(value[1])]
        }
    }
}

#[derive(Clone)]
pub struct MapStyle {
    pub border: Option<Stroke>,
    pub line: Option<Stroke>,
    pub fill_color: Color32,
    pub text_color: Color32,
    pub font: Option<FontId>,
    pub background_color: Color32,
    pub alert_color: Color32,
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
            alert_color: Color32::TRANSPARENT,
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
    pub id:Option<String>,
    pub raw_line: RawLine,
}

impl MapLine {
    pub fn new( point1: RawPoint, point2:RawPoint) -> Self {
        MapLine {
            id:None,
            raw_line: RawLine::new(point1, point2),
        }
    }
}



// This can by any object or point with its associated metadata
/// Struct that contains coordinates to help calculate nearest point in space
#[derive(Clone)]
pub struct MapPoint {
    /// coordinates of the Solar System
    pub raw_point: RawPoint,
    /// coordinates for lines connecting this point
    pub connections: Vec<String>,
    /// Object Identifier for search propurses
    id: usize,
    /// SolarSystem Name
    name: String,
}

impl MapPoint {
    /// Creates a new Spatial point with an Id (solarSystemId) and the system's 3D coordinates
    pub fn new(id: usize, coords: RawPoint) -> MapPoint {
        MapPoint {
            raw_point: coords,
            id,
            connections: Vec::new(),
            name: String::new(),
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn set_name(&mut self, value: String) {
        self.name = value;
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
    pub min: RawPoint,
    pub max: RawPoint,
    pub pos: RawPoint,
    pub dist: f32,
}

impl MapBounds {
    pub fn new() -> Self {
        MapBounds {
            min: RawPoint::default(),
            max: RawPoint::default(),
            pos: RawPoint::default(),
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
    pub position: RawPoint,
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
    pub(crate) projected_index: Option<usize>
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
            projected_index: None,
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
            projected_index: None,
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
            alert_color: Color32::from_rgb(246, 30, 131),
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
            alert_color: Color32::from_rgb(128, 12, 67),
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

pub trait ContextMenuManager {
    fn ui(&self, ui: &mut Ui);
}

pub trait NodeTemplate {
    fn node_ui(&self, ui: &mut Ui, viewport_point: Pos2);
}

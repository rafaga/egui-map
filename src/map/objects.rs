use egui::{Align2, Color32, FontFamily, FontId, Pos2, Stroke, Ui};
use std::ops::{Div, DivAssign, Mul, MulAssign, Sub, Add};
use std::convert::{From,Into};

#[derive(Copy,Clone)]
pub struct RawPoint{
    pub components:[f32;2]
}

impl RawPoint{
    pub fn new(x:f32, y:f32) -> Self {
        Self {
            components:[x,y]
        }
    }
}

impl Default for RawPoint{
    fn default() -> Self {
        Self::new(0.00,0.00)
    }
}

impl Mul<i64> for RawPoint {
    type Output = Self;

    fn mul(self, rhs: i64) -> Self::Output {
        Self {
            components: [self.components[0] * rhs as f32, self.components[1] * rhs as f32],
        }
    }
}

impl Mul<i32> for RawPoint {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self {
            components: [self.components[0] * rhs as f32, self.components[1] * rhs as f32],
        }
    }
}

impl Mul<u64> for RawPoint {
    type Output = Self;

    fn mul(self, rhs: u64) -> Self::Output {
        Self {
            components: [self.components[0] * rhs as f32, self.components[1] * rhs as f32],
        }
    }
}

impl Mul<u32> for RawPoint {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self::Output {
        Self {
            components: [self.components[0] * rhs as f32, self.components[1] * rhs as f32],
        }
    }
}

impl Mul<f32> for RawPoint {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            components: [self.components[0] * rhs, self.components[1] * rhs],
        }
    }
}

impl MulAssign<i64> for RawPoint{
    fn mul_assign(&mut self, rhs: i64) {
        self.components[0] = self.components[0] * rhs as f32;
        self.components[1] = self.components[1] * rhs as f32;
    }
}

impl MulAssign<i32> for RawPoint{
    fn mul_assign(&mut self, rhs: i32) {
        self.components[0] = self.components[0] * rhs as f32;
        self.components[1] = self.components[1] * rhs as f32;
    }
}

impl MulAssign<u64> for RawPoint{
    fn mul_assign(&mut self, rhs: u64) {
        self.components[0] = self.components[0] * rhs as f32;
        self.components[1] = self.components[1] * rhs as f32;
    }
}

impl MulAssign<u32> for RawPoint{
    fn mul_assign(&mut self, rhs: u32) {
        self.components[0] = self.components[0] * rhs as f32;
        self.components[1] = self.components[1] * rhs as f32;
    }
}

impl MulAssign<f32> for RawPoint{
    fn mul_assign(&mut self, rhs: f32) {
        self.components[0] = self.components[0] * rhs;
        self.components[1] = self.components[1] * rhs;
    }
}

impl Div<i64> for RawPoint {
    type Output = Self;

    fn div(self, rhs: i64) -> Self::Output {
        Self {
            components: [self.components[0] / rhs as f32, self.components[1] / rhs as f32],
        }
    }
}

impl Div<i32> for RawPoint {
    type Output = Self;

    fn div(self, rhs: i32) -> Self::Output {
        Self {
            components: [self.components[0] / rhs as f32, self.components[1] / rhs as f32],
        }
    }
}

impl Div<u64> for RawPoint {
    type Output = Self;

    fn div(self, rhs: u64) -> Self::Output {
        Self {
            components: [self.components[0] / rhs as f32, self.components[1] / rhs as f32],
        }
    }
}

impl Div<u32> for RawPoint {
    type Output = Self;

    fn div(self, rhs: u32) -> Self::Output {
        Self {
            components: [self.components[0] / rhs as f32, self.components[1] / rhs as f32],
        }
    }
}

impl Div<f32> for RawPoint {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            components: [self.components[0] / rhs, self.components[1] / rhs],
        }
    }
}

impl DivAssign<i64> for RawPoint{
    fn div_assign(&mut self, rhs: i64) {
        self.components[0] = self.components[0] / rhs as f32;
        self.components[1] = self.components[1] / rhs as f32;
    }
}

impl DivAssign<i32> for RawPoint{
    fn div_assign(&mut self, rhs: i32) {
        self.components[0] = self.components[0] / rhs as f32;
        self.components[1] = self.components[1] / rhs as f32;
    }
}

impl DivAssign<u64> for RawPoint{
    fn div_assign(&mut self, rhs: u64) {
        self.components[0] = self.components[0] / rhs as f32;
        self.components[1] = self.components[1] / rhs as f32;
    }
}

impl DivAssign<u32> for RawPoint{
    fn div_assign(&mut self, rhs: u32) {
        self.components[0] = self.components[0] / rhs as f32;
        self.components[1] = self.components[1] / rhs as f32;
    }
}


impl DivAssign<f32> for RawPoint{
    fn div_assign(&mut self, rhs: f32) {
        self.components[0] = self.components[0] / rhs;
        self.components[1] = self.components[1] / rhs;
    }
}

impl Add<RawPoint> for RawPoint{
    type Output = RawPoint;
    fn add(self, rhs: RawPoint) -> Self::Output {
        Self {
            components: [self.components[0] + rhs.components[0], self.components[1] + rhs.components[1]],
        }
    }
}

impl Sub<RawPoint> for RawPoint{
    type Output = RawPoint;
    fn sub(self, rhs: RawPoint) -> Self::Output {
        Self {
            components: [self.components[0] - rhs.components[0], self.components[1] - rhs.components[1]],
        }
    }
}

impl Add<&RawPoint> for RawPoint{
    type Output = RawPoint;
    fn add(self, rhs: &RawPoint) -> Self::Output {
        Self {
            components: [self.components[0] + rhs.components[0], self.components[1] + rhs.components[1]],
        }
    }
}

impl Sub<&RawPoint> for RawPoint{
    type Output = RawPoint;
    fn sub(self, rhs: &RawPoint) -> Self::Output {
        Self {
            components: [self.components[0] - rhs.components[0], self.components[1] - rhs.components[1]],
        }
    }
}

impl From<[f32;2]> for RawPoint{ 
    fn from(value: [f32;2]) -> Self {
        Self {
            components:value  
        }
    }
}

impl From<Pos2> for RawPoint{
    fn from(value: Pos2) -> Self {
        Self {
            components:[value.x,value.y]
        }
    }
}

impl From<[i64;2]> for RawPoint{
    fn from(value: [i64;2]) -> Self {
        Self {
            components:[value[0] as f32,value[1] as f32]
        }
    }
}

impl From<[i32;2]> for RawPoint{
    fn from(value: [i32;2]) -> Self {
        Self {
            components:[value[0] as f32,value[1] as f32]
        }
    }
}

impl From<[i16;2]> for RawPoint{
    fn from(value: [i16;2]) -> Self {
        Self {
            components:[value[0] as f32,value[1] as f32]
        }
    }
}

impl From<[i8;2]> for RawPoint{
    fn from(value: [i8;2]) -> Self {
        Self {
            components:[value[0] as f32,value[1] as f32]
        }
    }
}

impl Into<[f32;2]> for RawPoint{
    fn into(self) -> [f32;2] {
        [self.components[0],self.components[1]]
    }
}

impl Into<Pos2> for RawPoint{
    fn into(self) -> Pos2 {
        Pos2::from(self.components)
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
        let x = self.points[0].components[0] - self.points[1].components[0];
        let y = self.points[0].components[1] - self.points[1].components[1];
        (x.powi(2)+y.powi(2)).sqrt()
    }

    pub fn midpoint(self) -> RawPoint {
        let x = (self.points[0].components[0] + self.points[1].components[0])/2.0;
        let y = (self.points[0].components[1] + self.points[1].components[1])/2.0;
        RawPoint::new(x,y)
    }
}

impl Into<[Pos2;2]> for RawLine {

    fn into(self) -> [Pos2; 2] { 
        let position1 = self.points[0].into();
        let position2 = self.points[1].into();
        [position1,position2]
    }
}

impl From<[[i64;2];2]> for RawLine {
    fn from(value: [[i64;2];2]) -> Self {
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
                width: 2.0,
                color: Color32::from_rgb(216, 142, 58),
            }),
            line: Some(egui::Stroke {
                width: 2.0,
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
                width: 2.0,
                color: Color32::GOLD,
            }),
            line: Some(egui::Stroke {
                width: 2.0,
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

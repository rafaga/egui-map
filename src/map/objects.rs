use egui::{Align2, Color32, FontFamily, FontId, Pos2, Stroke, Ui};
use std::convert::{From, Into};
use std::ops::{Add, Div, DivAssign, Mul, MulAssign, Sub};
use std::time::Instant;

#[derive(Copy, Clone, Debug)]
pub struct RawPoint {
    pub components: [f32; 2],
}

impl RawPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self { components: [x, y] }
    }
}

impl Default for RawPoint {
    fn default() -> Self {
        Self::new(0.00, 0.00)
    }
}

impl Mul<i64> for RawPoint {
    type Output = Self;

    fn mul(self, rhs: i64) -> Self::Output {
        Self {
            components: [
                self.components[0] * rhs as f32,
                self.components[1] * rhs as f32,
            ],
        }
    }
}

impl Mul<i32> for RawPoint {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self {
            components: [
                self.components[0] * rhs as f32,
                self.components[1] * rhs as f32,
            ],
        }
    }
}

impl Mul<u64> for RawPoint {
    type Output = Self;

    fn mul(self, rhs: u64) -> Self::Output {
        Self {
            components: [
                self.components[0] * rhs as f32,
                self.components[1] * rhs as f32,
            ],
        }
    }
}

impl Mul<u32> for RawPoint {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self::Output {
        Self {
            components: [
                self.components[0] * rhs as f32,
                self.components[1] * rhs as f32,
            ],
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

impl MulAssign<i64> for RawPoint {
    fn mul_assign(&mut self, rhs: i64) {
        self.components[0] = self.components[0] * rhs as f32;
        self.components[1] = self.components[1] * rhs as f32;
    }
}

impl MulAssign<i32> for RawPoint {
    fn mul_assign(&mut self, rhs: i32) {
        self.components[0] = self.components[0] * rhs as f32;
        self.components[1] = self.components[1] * rhs as f32;
    }
}

impl MulAssign<u64> for RawPoint {
    fn mul_assign(&mut self, rhs: u64) {
        self.components[0] = self.components[0] * rhs as f32;
        self.components[1] = self.components[1] * rhs as f32;
    }
}

impl MulAssign<u32> for RawPoint {
    fn mul_assign(&mut self, rhs: u32) {
        self.components[0] = self.components[0] * rhs as f32;
        self.components[1] = self.components[1] * rhs as f32;
    }
}

impl MulAssign<f32> for RawPoint {
    fn mul_assign(&mut self, rhs: f32) {
        self.components[0] = self.components[0] * rhs;
        self.components[1] = self.components[1] * rhs;
    }
}

impl Div<i64> for RawPoint {
    type Output = Self;

    fn div(self, rhs: i64) -> Self::Output {
        Self {
            components: [
                self.components[0] / rhs as f32,
                self.components[1] / rhs as f32,
            ],
        }
    }
}

impl Div<i32> for RawPoint {
    type Output = Self;

    fn div(self, rhs: i32) -> Self::Output {
        Self {
            components: [
                self.components[0] / rhs as f32,
                self.components[1] / rhs as f32,
            ],
        }
    }
}

impl Div<u64> for RawPoint {
    type Output = Self;

    fn div(self, rhs: u64) -> Self::Output {
        Self {
            components: [
                self.components[0] / rhs as f32,
                self.components[1] / rhs as f32,
            ],
        }
    }
}

impl Div<u32> for RawPoint {
    type Output = Self;

    fn div(self, rhs: u32) -> Self::Output {
        Self {
            components: [
                self.components[0] / rhs as f32,
                self.components[1] / rhs as f32,
            ],
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

impl DivAssign<i64> for RawPoint {
    fn div_assign(&mut self, rhs: i64) {
        self.components[0] = self.components[0] / rhs as f32;
        self.components[1] = self.components[1] / rhs as f32;
    }
}

impl DivAssign<i32> for RawPoint {
    fn div_assign(&mut self, rhs: i32) {
        self.components[0] = self.components[0] / rhs as f32;
        self.components[1] = self.components[1] / rhs as f32;
    }
}

impl DivAssign<u64> for RawPoint {
    fn div_assign(&mut self, rhs: u64) {
        self.components[0] = self.components[0] / rhs as f32;
        self.components[1] = self.components[1] / rhs as f32;
    }
}

impl DivAssign<u32> for RawPoint {
    fn div_assign(&mut self, rhs: u32) {
        self.components[0] = self.components[0] / rhs as f32;
        self.components[1] = self.components[1] / rhs as f32;
    }
}

impl DivAssign<f32> for RawPoint {
    fn div_assign(&mut self, rhs: f32) {
        self.components[0] = self.components[0] / rhs;
        self.components[1] = self.components[1] / rhs;
    }
}

impl Add<RawPoint> for RawPoint {
    type Output = RawPoint;
    fn add(self, rhs: RawPoint) -> Self::Output {
        Self {
            components: [
                self.components[0] + rhs.components[0],
                self.components[1] + rhs.components[1],
            ],
        }
    }
}

impl Sub<RawPoint> for RawPoint {
    type Output = RawPoint;
    fn sub(self, rhs: RawPoint) -> Self::Output {
        Self {
            components: [
                self.components[0] - rhs.components[0],
                self.components[1] - rhs.components[1],
            ],
        }
    }
}

impl Add<&RawPoint> for RawPoint {
    type Output = RawPoint;
    fn add(self, rhs: &RawPoint) -> Self::Output {
        Self {
            components: [
                self.components[0] + rhs.components[0],
                self.components[1] + rhs.components[1],
            ],
        }
    }
}

impl Sub<&RawPoint> for RawPoint {
    type Output = RawPoint;
    fn sub(self, rhs: &RawPoint) -> Self::Output {
        Self {
            components: [
                self.components[0] - rhs.components[0],
                self.components[1] - rhs.components[1],
            ],
        }
    }
}

impl From<[f32; 2]> for RawPoint {
    fn from(value: [f32; 2]) -> Self {
        Self { components: value }
    }
}

impl From<Pos2> for RawPoint {
    fn from(value: Pos2) -> Self {
        Self {
            components: [value.x, value.y],
        }
    }
}

impl From<[i64; 2]> for RawPoint {
    fn from(value: [i64; 2]) -> Self {
        Self {
            components: [value[0] as f32, value[1] as f32],
        }
    }
}

impl From<[i32; 2]> for RawPoint {
    fn from(value: [i32; 2]) -> Self {
        Self {
            components: [value[0] as f32, value[1] as f32],
        }
    }
}

impl From<[i16; 2]> for RawPoint {
    fn from(value: [i16; 2]) -> Self {
        Self {
            components: [value[0] as f32, value[1] as f32],
        }
    }
}

impl From<[i8; 2]> for RawPoint {
    fn from(value: [i8; 2]) -> Self {
        Self {
            components: [value[0] as f32, value[1] as f32],
        }
    }
}

impl From<RawPoint> for [f32; 2] {
    fn from(val: RawPoint) -> Self {
        [val.components[0], val.components[1]]
    }
}

impl From<RawPoint> for Pos2 {
    fn from(val: RawPoint) -> Self {
        Pos2::from(val.components)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RawLine {
    pub points: [RawPoint; 2],
}

impl RawLine {
    pub fn new(a: RawPoint, b: RawPoint) -> Self {
        Self { points: [a, b] }
    }

    pub fn distance(self) -> f32 {
        let x = self.points[0].components[0] - self.points[1].components[0];
        let y = self.points[0].components[1] - self.points[1].components[1];
        (x.powi(2) + y.powi(2)).sqrt()
    }

    pub fn midpoint(self) -> RawPoint {
        let x = (self.points[0].components[0] + self.points[1].components[0]) / 2.0;
        let y = (self.points[0].components[1] + self.points[1].components[1]) / 2.0;
        RawPoint::new(x, y)
    }
}

impl From<RawLine> for [Pos2; 2] {
    fn from(val: RawLine) -> Self {
        let position1 = val.points[0].into();
        let position2 = val.points[1].into();
        [position1, position2]
    }
}

impl From<[[i64; 2]; 2]> for RawLine {
    fn from(value: [[i64; 2]; 2]) -> Self {
        Self {
            points: [RawPoint::from(value[0]), RawPoint::from(value[1])],
        }
    }
}

#[derive(Clone, Debug)]
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
        self.border.as_mut().unwrap().width *= rhs as f32;
        self.line.as_mut().unwrap().width *= rhs as f32;
        self.font.as_mut().unwrap().size *= rhs as f32;
        self
    }
}

impl Mul<i32> for MapStyle {
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn mul(mut self, rhs: i32) -> Self::Output {
        self.border.as_mut().unwrap().width *= rhs as f32;
        self.line.as_mut().unwrap().width *= rhs as f32;
        self.font.as_mut().unwrap().size *= rhs as f32;
        self
    }
}

impl Mul<f32> for MapStyle {
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn mul(mut self, rhs: f32) -> Self::Output {
        self.border.as_mut().unwrap().width *= rhs;
        self.line.as_mut().unwrap().width *= rhs;
        self.font.as_mut().unwrap().size *= rhs;
        self
    }
}

impl Mul<f64> for MapStyle {
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn mul(mut self, rhs: f64) -> Self::Output {
        self.border.as_mut().unwrap().width *= rhs as f32;
        self.line.as_mut().unwrap().width *= rhs as f32;
        self.font.as_mut().unwrap().size *= rhs as f32;
        self
    }
}

impl Div<i64> for MapStyle {
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn div(mut self, rhs: i64) -> Self::Output {
        self.border.as_mut().unwrap().width /= rhs as f32;
        self.line.as_mut().unwrap().width /= rhs as f32;
        self.font.as_mut().unwrap().size /= rhs as f32;
        self
    }
}

impl Div<i32> for MapStyle {
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn div(mut self, rhs: i32) -> Self::Output {
        self.border.as_mut().unwrap().width /= rhs as f32;
        self.line.as_mut().unwrap().width /= rhs as f32;
        self.font.as_mut().unwrap().size /= rhs as f32;
        self
    }
}

impl Div<f32> for MapStyle {
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn div(mut self, rhs: f32) -> Self::Output {
        self.border.as_mut().unwrap().width /= rhs;
        self.line.as_mut().unwrap().width /= rhs;
        self.font.as_mut().unwrap().size /= rhs;
        self
    }
}

impl Div<f64> for MapStyle {
    // The multiplication of rational numbers is a closed operation.
    type Output = Self;

    fn div(mut self, rhs: f64) -> Self::Output {
        self.border.as_mut().unwrap().width /= rhs as f32;
        self.line.as_mut().unwrap().width /= rhs as f32;
        self.font.as_mut().unwrap().size /= rhs as f32;
        self
    }
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct MapLine {
    pub id: Option<String>,
    pub raw_line: RawLine,
}

impl MapLine {
    pub fn new(point1: RawPoint, point2: RawPoint) -> Self {
        MapLine {
            id: None,
            raw_line: RawLine::new(point1, point2),
        }
    }
}

// This can by any object or point with its associated metadata
/// Struct that contains coordinates to help calculate nearest point in space
#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug, PartialEq)]
pub enum VisibilitySetting {
    Hidden,
    Hover,
    Allways,
}

pub trait ContextMenuManager {
    fn ui(&self, ui: &mut Ui);
}

pub trait NodeTemplate {
    fn node_ui(&self, ui: &mut Ui, _viewport_point: Pos2, _zoom: f32, _system: &MapPoint);
    fn selection_ui(&self, ui: &mut Ui, _viewport_point: Pos2, _zoom: f32);
    fn notification_ui(
        &self,
        ui: &mut Ui,
        _viewport_point: Pos2,
        _zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool;
    fn marker_ui(&self, ui: &mut Ui, _viewport_point: Pos2, _zoom: f32);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // ---------- RawPoint ----------

    #[test]
    fn raw_point_new() {
        let p = RawPoint::new(3.5, -2.0);
        assert_eq!(p.components, [3.5, -2.0]);
    }

    #[test]
    fn raw_point_default() {
        let p = RawPoint::default();
        assert_eq!(p.components, [0.0, 0.0]);
    }

    #[test]
    fn raw_point_mul_i64() {
        let p = RawPoint::new(2.0, -3.0) * 3i64;
        assert_eq!(p.components, [6.0, -9.0]);
    }

    #[test]
    fn raw_point_mul_i32() {
        let p = RawPoint::new(2.0, -3.0) * 3i32;
        assert_eq!(p.components, [6.0, -9.0]);
    }

    #[test]
    fn raw_point_mul_u64() {
        let p = RawPoint::new(2.0, -3.0) * 3u64;
        assert_eq!(p.components, [6.0, -9.0]);
    }

    #[test]
    fn raw_point_mul_u32() {
        let p = RawPoint::new(2.0, -3.0) * 3u32;
        assert_eq!(p.components, [6.0, -9.0]);
    }

    #[test]
    fn raw_point_mul_f32() {
        let p = RawPoint::new(2.0, -3.0) * 0.5f32;
        assert_eq!(p.components, [1.0, -1.5]);
    }

    #[test]
    fn raw_point_mul_assign_i64() {
        let mut p = RawPoint::new(2.0, -3.0);
        p *= 3i64;
        assert_eq!(p.components, [6.0, -9.0]);
    }

    #[test]
    fn raw_point_mul_assign_i32() {
        let mut p = RawPoint::new(2.0, -3.0);
        p *= 3i32;
        assert_eq!(p.components, [6.0, -9.0]);
    }

    #[test]
    fn raw_point_mul_assign_u64() {
        let mut p = RawPoint::new(2.0, -3.0);
        p *= 3u64;
        assert_eq!(p.components, [6.0, -9.0]);
    }

    #[test]
    fn raw_point_mul_assign_u32() {
        let mut p = RawPoint::new(2.0, -3.0);
        p *= 3u32;
        assert_eq!(p.components, [6.0, -9.0]);
    }

    #[test]
    fn raw_point_mul_assign_f32() {
        let mut p = RawPoint::new(2.0, -3.0);
        p *= 0.5f32;
        assert_eq!(p.components, [1.0, -1.5]);
    }

    #[test]
    fn raw_point_div_i64() {
        let p = RawPoint::new(6.0, -9.0) / 3i64;
        assert_eq!(p.components, [2.0, -3.0]);
    }

    #[test]
    fn raw_point_div_i32() {
        let p = RawPoint::new(6.0, -9.0) / 3i32;
        assert_eq!(p.components, [2.0, -3.0]);
    }

    #[test]
    fn raw_point_div_u64() {
        let p = RawPoint::new(6.0, -9.0) / 3u64;
        assert_eq!(p.components, [2.0, -3.0]);
    }

    #[test]
    fn raw_point_div_u32() {
        let p = RawPoint::new(6.0, -9.0) / 3u32;
        assert_eq!(p.components, [2.0, -3.0]);
    }

    #[test]
    fn raw_point_div_f32() {
        let p = RawPoint::new(1.0, -1.5) / 0.5f32;
        assert_eq!(p.components, [2.0, -3.0]);
    }

    #[test]
    fn raw_point_div_assign_i64() {
        let mut p = RawPoint::new(6.0, -9.0);
        p /= 3i64;
        assert_eq!(p.components, [2.0, -3.0]);
    }

    #[test]
    fn raw_point_div_assign_i32() {
        let mut p = RawPoint::new(6.0, -9.0);
        p /= 3i32;
        assert_eq!(p.components, [2.0, -3.0]);
    }

    #[test]
    fn raw_point_div_assign_u64() {
        let mut p = RawPoint::new(6.0, -9.0);
        p /= 3u64;
        assert_eq!(p.components, [2.0, -3.0]);
    }

    #[test]
    fn raw_point_div_assign_u32() {
        let mut p = RawPoint::new(6.0, -9.0);
        p /= 3u32;
        assert_eq!(p.components, [2.0, -3.0]);
    }

    #[test]
    fn raw_point_div_assign_f32() {
        let mut p = RawPoint::new(1.0, -1.5);
        p /= 0.5f32;
        assert_eq!(p.components, [2.0, -3.0]);
    }

    #[test]
    fn raw_point_add() {
        let a = RawPoint::new(1.0, 2.0);
        let b = RawPoint::new(3.0, -4.0);
        let c = a + b;
        assert_eq!(c.components, [4.0, -2.0]);
    }

    #[test]
    fn raw_point_sub() {
        let a = RawPoint::new(1.0, 2.0);
        let b = RawPoint::new(3.0, -4.0);
        let c = a - b;
        assert_eq!(c.components, [-2.0, 6.0]);
    }

    #[test]
    fn raw_point_add_ref() {
        let a = RawPoint::new(1.0, 2.0);
        let b = RawPoint::new(3.0, -4.0);
        let c = a + &b;
        assert_eq!(c.components, [4.0, -2.0]);
        // b sigue siendo usable tras la suma por referencia
        assert_eq!(b.components, [3.0, -4.0]);
    }

    #[test]
    fn raw_point_sub_ref() {
        let a = RawPoint::new(1.0, 2.0);
        let b = RawPoint::new(3.0, -4.0);
        let c = a - &b;
        assert_eq!(c.components, [-2.0, 6.0]);
        assert_eq!(b.components, [3.0, -4.0]);
    }

    #[test]
    fn raw_point_from_f32_array() {
        let p = RawPoint::from([1.5f32, -2.5f32]);
        assert_eq!(p.components, [1.5, -2.5]);
    }

    #[test]
    fn raw_point_from_i64_array() {
        let p = RawPoint::from([3i64, -4i64]);
        assert_eq!(p.components, [3.0, -4.0]);
    }

    #[test]
    fn raw_point_from_i32_array() {
        let p = RawPoint::from([3i32, -4i32]);
        assert_eq!(p.components, [3.0, -4.0]);
    }

    #[test]
    fn raw_point_from_i16_array() {
        let p = RawPoint::from([3i16, -4i16]);
        assert_eq!(p.components, [3.0, -4.0]);
    }

    #[test]
    fn raw_point_from_i8_array() {
        let p = RawPoint::from([3i8, -4i8]);
        assert_eq!(p.components, [3.0, -4.0]);
    }

    #[test]
    fn raw_point_from_pos2() {
        let p = RawPoint::from(Pos2::new(7.0, 8.0));
        assert_eq!(p.components, [7.0, 8.0]);
    }

    #[test]
    fn raw_point_into_f32_array() {
        let arr: [f32; 2] = RawPoint::new(7.0, 8.0).into();
        assert_eq!(arr, [7.0, 8.0]);
    }

    #[test]
    fn raw_point_into_pos2() {
        let pos: Pos2 = RawPoint::new(7.0, 8.0).into();
        assert_eq!(pos, Pos2::new(7.0, 8.0));
    }

    // ---------- RawLine ----------

    #[test]
    fn raw_line_new() {
        let a = RawPoint::new(1.0, 2.0);
        let b = RawPoint::new(3.0, 4.0);
        let line = RawLine::new(a, b);
        assert_eq!(line.points[0].components, [1.0, 2.0]);
        assert_eq!(line.points[1].components, [3.0, 4.0]);
    }

    #[test]
    fn raw_line_distance() {
        // triángulo 3-4-5
        let line = RawLine::new(RawPoint::new(0.0, 0.0), RawPoint::new(3.0, 4.0));
        assert_eq!(line.distance(), 5.0);
    }

    #[test]
    fn raw_line_distance_zero() {
        let line = RawLine::new(RawPoint::new(2.0, 2.0), RawPoint::new(2.0, 2.0));
        assert_eq!(line.distance(), 0.0);
    }

    #[test]
    fn raw_line_midpoint() {
        let line = RawLine::new(RawPoint::new(0.0, 0.0), RawPoint::new(4.0, 6.0));
        let mid = line.midpoint();
        assert_eq!(mid.components, [2.0, 3.0]);
    }

    #[test]
    fn raw_line_into_pos2_array() {
        let line = RawLine::new(RawPoint::new(1.0, 2.0), RawPoint::new(3.0, 4.0));
        let arr: [Pos2; 2] = line.into();
        assert_eq!(arr, [Pos2::new(1.0, 2.0), Pos2::new(3.0, 4.0)]);
    }

    #[test]
    fn raw_line_from_i64_arrays() {
        let line = RawLine::from([[1i64, 2i64], [3i64, 4i64]]);
        assert_eq!(line.points[0].components, [1.0, 2.0]);
        assert_eq!(line.points[1].components, [3.0, 4.0]);
    }

    // ---------- MapStyle ----------

    fn full_style() -> MapStyle {
        MapStyle {
            border: Some(Stroke::new(2.0, Color32::RED)),
            line: Some(Stroke::new(4.0, Color32::BLUE)),
            fill_color: Color32::GREEN,
            text_color: Color32::WHITE,
            font: Some(FontId::new(10.0, FontFamily::Proportional)),
            background_color: Color32::BLACK,
            alert_color: Color32::YELLOW,
        }
    }

    #[test]
    fn map_style_new() {
        let s = MapStyle::new();
        assert!(s.border.is_none());
        assert!(s.line.is_none());
        assert!(s.font.is_none());
        assert_eq!(s.fill_color, Color32::TRANSPARENT);
        assert_eq!(s.text_color, Color32::TRANSPARENT);
        assert_eq!(s.background_color, Color32::TRANSPARENT);
        assert_eq!(s.alert_color, Color32::TRANSPARENT);
    }

    #[test]
    fn map_style_default_equals_new() {
        let s = MapStyle::default();
        assert!(s.border.is_none());
        assert!(s.line.is_none());
        assert!(s.font.is_none());
    }

    #[test]
    fn map_style_mul_i64() {
        let s = full_style() * 2i64;
        assert_eq!(s.border.unwrap().width, 4.0);
        assert_eq!(s.line.unwrap().width, 8.0);
        assert_eq!(s.font.unwrap().size, 20.0);
    }

    #[test]
    fn map_style_mul_i32() {
        let s = full_style() * 2i32;
        assert_eq!(s.border.unwrap().width, 4.0);
        assert_eq!(s.line.unwrap().width, 8.0);
        assert_eq!(s.font.unwrap().size, 20.0);
    }

    #[test]
    fn map_style_mul_f32() {
        let s = full_style() * 0.5f32;
        assert_eq!(s.border.unwrap().width, 1.0);
        assert_eq!(s.line.unwrap().width, 2.0);
        assert_eq!(s.font.unwrap().size, 5.0);
    }

    #[test]
    fn map_style_mul_f64() {
        let s = full_style() * 0.5f64;
        assert_eq!(s.border.unwrap().width, 1.0);
        assert_eq!(s.line.unwrap().width, 2.0);
        assert_eq!(s.font.unwrap().size, 5.0);
    }

    #[test]
    fn map_style_div_i64() {
        let s = full_style() / 2i64;
        assert_eq!(s.border.unwrap().width, 1.0);
        assert_eq!(s.line.unwrap().width, 2.0);
        assert_eq!(s.font.unwrap().size, 5.0);
    }

    #[test]
    fn map_style_div_i32() {
        let s = full_style() / 2i32;
        assert_eq!(s.border.unwrap().width, 1.0);
        assert_eq!(s.line.unwrap().width, 2.0);
        assert_eq!(s.font.unwrap().size, 5.0);
    }

    #[test]
    fn map_style_div_f32() {
        let s = full_style() / 0.5f32;
        assert_eq!(s.border.unwrap().width, 4.0);
        assert_eq!(s.line.unwrap().width, 8.0);
        assert_eq!(s.font.unwrap().size, 20.0);
    }

    #[test]
    fn map_style_div_f64() {
        let s = full_style() / 0.5f64;
        assert_eq!(s.border.unwrap().width, 4.0);
        assert_eq!(s.line.unwrap().width, 8.0);
        assert_eq!(s.font.unwrap().size, 20.0);
    }

    // ---------- MapLabel ----------

    #[test]
    fn map_label_new() {
        let l = MapLabel::new();
        assert_eq!(l.text, String::new());
        assert_eq!(l.center, Pos2::new(0.0, 0.0));
    }

    #[test]
    fn map_label_default_equals_new() {
        let l = MapLabel::default();
        assert_eq!(l.text, String::new());
        assert_eq!(l.center, Pos2::new(0.0, 0.0));
    }

    // ---------- MapLine ----------

    #[test]
    fn map_line_new() {
        let a = RawPoint::new(1.0, 2.0);
        let b = RawPoint::new(3.0, 4.0);
        let line = MapLine::new(a, b);
        assert!(line.id.is_none());
        assert_eq!(line.raw_line.points[0].components, [1.0, 2.0]);
        assert_eq!(line.raw_line.points[1].components, [3.0, 4.0]);
    }

    // ---------- MapPoint ----------

    #[test]
    fn map_point_new() {
        let p = MapPoint::new(42, RawPoint::new(1.0, 2.0));
        assert_eq!(p.get_id(), 42);
        assert_eq!(p.raw_point.components, [1.0, 2.0]);
        assert!(p.connections.is_empty());
        assert_eq!(p.get_name(), String::new());
    }

    #[test]
    fn map_point_set_and_get_name() {
        let mut p = MapPoint::new(1, RawPoint::default());
        p.set_name("Jita".to_string());
        assert_eq!(p.get_name(), "Jita");
    }

    #[test]
    fn map_point_from_occupied_entry() {
        let mut map: HashMap<usize, MapPoint> = HashMap::new();
        let mut original = MapPoint::new(7, RawPoint::new(5.0, 6.0));
        original.set_name("Amarr".to_string());
        map.insert(7, original);

        use std::collections::hash_map::Entry;
        if let Entry::Occupied(entry) = map.entry(7) {
            let cloned = MapPoint::from(entry);
            assert_eq!(cloned.get_id(), 7);
            assert_eq!(cloned.get_name(), "Amarr");
            assert_eq!(cloned.raw_point.components, [5.0, 6.0]);
        } else {
            panic!("se esperaba una entrada ocupada");
        }
    }

    // ---------- MapBounds ----------

    #[test]
    fn map_bounds_new() {
        let b = MapBounds::new();
        assert_eq!(b.min.components, [0.0, 0.0]);
        assert_eq!(b.max.components, [0.0, 0.0]);
        assert_eq!(b.pos.components, [0.0, 0.0]);
        assert_eq!(b.dist, 0.0);
    }

    #[test]
    fn map_bounds_default_equals_new() {
        let b = MapBounds::default();
        assert_eq!(b.dist, 0.0);
        assert_eq!(b.pos.components, [0.0, 0.0]);
    }

    // ---------- MapSettings ----------

    #[test]
    fn map_settings_new() {
        let s = MapSettings::new();
        assert_eq!(s.max_zoom, 0.0);
        assert_eq!(s.min_zoom, 0.0);
        assert_eq!(s.line_visible_zoom, 0.0);
        assert_eq!(s.label_visible_zoom, 0.0);
        assert_eq!(s.node_text_visibility, VisibilitySetting::Allways);
        assert_eq!(s.styles.len(), 1);
    }

    #[test]
    fn map_settings_default() {
        let s = MapSettings::default();
        assert_eq!(s.max_zoom, 2.0);
        assert_eq!(s.min_zoom, 0.1);
        assert_eq!(s.line_visible_zoom, 0.2);
        assert_eq!(s.label_visible_zoom, 0.58);
        assert_eq!(s.node_text_visibility, VisibilitySetting::Allways);
        // light + dark themes
        assert_eq!(s.styles.len(), 2);
        // light theme
        assert_eq!(s.styles[0].background_color, Color32::WHITE);
        assert!(s.styles[0].border.is_some());
        assert!(s.styles[0].line.is_some());
        assert!(s.styles[0].font.is_some());
        // dark theme
        assert_eq!(s.styles[1].background_color, Color32::DARK_GRAY);
        assert!(s.styles[1].border.is_some());
        assert!(s.styles[1].line.is_some());
        assert!(s.styles[1].font.is_some());
    }

    // ---------- VisibilitySetting ----------

    #[test]
    fn visibility_setting_equality() {
        assert_eq!(VisibilitySetting::Hidden, VisibilitySetting::Hidden);
        assert_eq!(VisibilitySetting::Hover, VisibilitySetting::Hover);
        assert_eq!(VisibilitySetting::Allways, VisibilitySetting::Allways);
        assert_ne!(VisibilitySetting::Hidden, VisibilitySetting::Hover);
        assert_ne!(VisibilitySetting::Hover, VisibilitySetting::Allways);
        assert_ne!(VisibilitySetting::Hidden, VisibilitySetting::Allways);
    }
}

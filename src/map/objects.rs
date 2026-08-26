//! Data types consumed by the [`Map`](super::Map) widget.
//!
//! This module contains the geometry primitives ([`RawPoint`], [`RawLine`]),
//! the map content types ([`MapPoint`], [`MapSegment`], [`MapLabel`]) and the
//! customization points of the widget: [`MapSettings`],
//! [`Style`](super::theme::Style), [`VisibilitySetting`],
//! [`ContextMenuManager`] and [`NodeTemplate`]. The color palette a `Style`
//! paints with lives in [`super::theme`], via [`MapTheme`](super::theme::MapTheme).

use crate::map::theme::Style;
use egui::{Align2, Color32, FontFamily, FontId, Painter, Pos2, Ui};
use rstar::AABB;
use std::convert::{From, Into};
use std::ops::{Add, Div, DivAssign, Mul, MulAssign, Sub};
use std::time::Instant;

/// A point (or vector) in 2D map coordinates.
///
/// `RawPoint` supports component-wise arithmetic: [`Mul`], [`Div`],
/// [`MulAssign`] and [`DivAssign`] with `f32` and the common integer types, and
/// [`Add`]/[`Sub`] with other points (by value or by reference). It also
/// converts from and to `[f32; 2]`, integer arrays and [`egui::Pos2`].
///
/// # Examples
///
/// ```
/// use egui_map::map::objects::RawPoint;
///
/// let a = RawPoint::new(1.0, 2.0);
/// let b = RawPoint::new(3.0, -1.0);
///
/// assert_eq!((a + b).components, [4.0, 1.0]);
/// assert_eq!((a * 2.0f32).components, [2.0, 4.0]);
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RawPoint {
    /// The `x` and `y` components of the point.
    pub components: [f32; 2],
}

impl RawPoint {
    /// Creates a point from its `x` and `y` coordinates.
    pub fn new(x: f32, y: f32) -> Self {
        Self { components: [x, y] }
    }
}

impl rstar::Point for RawPoint {
    type Scalar = f32;
    const DIMENSIONS: usize = 2;

    fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
        let mut components = [0.0; 2];
        for (i, component) in components.iter_mut().enumerate() {
            *component = generator(i);
        }
        Self { components }
    }

    fn nth(&self, index: usize) -> Self::Scalar {
        self.components[index]
    }

    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        &mut self.components[index]
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

/// A straight line segment between two [`RawPoint`]s.
#[derive(Copy, Clone, Debug)]
pub struct RawLine {
    /// The two end points of the segment.
    pub points: [RawPoint; 2],
}

impl RawLine {
    /// Creates a segment between `a` and `b`.
    pub fn new(a: RawPoint, b: RawPoint) -> Self {
        Self { points: [a, b] }
    }

    /// Returns the Euclidean distance between the two end points.
    pub fn distance(self) -> f32 {
        let x = self.points[0].components[0] - self.points[1].components[0];
        let y = self.points[0].components[1] - self.points[1].components[1];
        (x.powi(2) + y.powi(2)).sqrt()
    }

    /// Returns the point halfway between the two end points.
    pub fn midpoint(self) -> RawPoint {
        let x = (self.points[0].components[0] + self.points[1].components[0]) / 2.0;
        let y = (self.points[0].components[1] + self.points[1].components[1]) / 2.0;
        RawPoint::new(x, y)
    }

    /// Returns the Euclidean distance from `point` to the closest point on
    /// this segment.
    ///
    /// The closest point is the perpendicular projection of `point` onto the
    /// segment's supporting line, clamped to the segment itself; for a
    /// zero-length segment it is simply the distance to the endpoint.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_map::map::objects::{RawLine, RawPoint};
    ///
    /// let line = RawLine::new(RawPoint::new(0.0, 0.0), RawPoint::new(10.0, 0.0));
    /// assert_eq!(line.distance_to_point(RawPoint::new(5.0, 3.0)), 3.0);
    /// // Beyond the end of the segment, the endpoint is the closest point.
    /// assert_eq!(line.distance_to_point(RawPoint::new(14.0, 0.0)), 4.0);
    /// ```
    pub fn distance_to_point(self, point: RawPoint) -> f32 {
        let [a, b] = self.points;
        let ab = b - a;
        let ap = point - a;
        let len_sq = ab.components[0].powi(2) + ab.components[1].powi(2);
        if len_sq == 0.0 {
            // Degenerate segment: both endpoints coincide.
            return (ap.components[0].powi(2) + ap.components[1].powi(2)).sqrt();
        }
        let t = ((ap.components[0] * ab.components[0] + ap.components[1] * ab.components[1])
            / len_sq)
            .clamp(0.0, 1.0);
        let closest = a + ab * t;
        let d = point - closest;
        (d.components[0].powi(2) + d.components[1].powi(2)).sqrt()
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

/// A free-floating text label drawn on the map.
///
/// Labels are installed with [`Map::add_labels`](super::Map::add_labels).
#[derive(Clone, Debug)]
pub struct MapLabel {
    /// The text to display.
    pub text: String,
    /// The position of the label's center.
    pub center: Pos2,
}

impl Default for MapLabel {
    fn default() -> Self {
        MapLabel::new()
    }
}

impl MapLabel {
    /// Creates an empty label centered at the origin.
    pub fn new() -> Self {
        MapLabel {
            text: String::new(),
            center: Pos2::new(0.00, 0.00),
        }
    }
}

/// A connection line between two points on the map, ready to be stored in an
/// [`rstar::RTree`].
///
/// Mirrors `sde::objects::SdeSegment`'s shape (`id`, `point1`, `point2`) so
/// callers that already hold `sde` connection data can build one with a
/// straight field-for-field copy; the only structural difference is `f32`
/// instead of `f64` for the coordinates, matching `egui`'s own coordinate
/// type (`egui` — and therefore this widget — doesn't work in `f64`).
///
/// Lines are installed with [`Map::add_hashmap_lines`](super::Map::add_hashmap_lines),
/// keyed by an id that nodes reference through [`MapPoint::connections`].
/// The bounding box the R-tree uses for broad-phase viewport culling and
/// hit-testing is computed on demand from `point1`/`point2` in
/// [`envelope`](rstar::RTreeObject::envelope) rather than cached on the
/// struct, same as `SdeSegment`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapSegment {
    /// Identifier shared with the line key (and with the
    /// [`MapPoint::connections`] of the endpoint nodes).
    pub id: (usize, usize),
    /// One endpoint of the segment, in map coordinates.
    pub point1: [f32; 2],
    /// The other endpoint of the segment, in map coordinates.
    pub point2: [f32; 2],
}

impl MapSegment {
    /// Creates a segment for `id` between `point1` and `point2`.
    pub fn new(id: (usize, usize), point1: [f32; 2], point2: [f32; 2]) -> Self {
        Self { id, point1, point2 }
    }

    /// The segment geometry as a [`RawLine`], for the distance/midpoint math
    /// callers already get from that type.
    pub(crate) fn raw_line(&self) -> RawLine {
        RawLine::new(RawPoint::from(self.point1), RawPoint::from(self.point2))
    }
}

impl rstar::RTreeObject for MapSegment {
    type Envelope = AABB<[f32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(
            [
                self.point1[0].min(self.point2[0]),
                self.point1[1].min(self.point2[1]),
            ],
            [
                self.point1[0].max(self.point2[0]),
                self.point1[1].max(self.point2[1]),
            ],
        )
    }
}

/// A node on the map: an id, a 2D position and an optional display name.
///
/// Mirrors `sde::objects::SdePoint`'s shape (public `coords`/`id`/`name`/
/// `connections` fields) so callers that already hold `sde` map-query data
/// can build one with a straight field-for-field copy. Structural
/// differences from `SdePoint`:
///
/// - `f32` instead of `f64` for `coords`, matching `egui`'s own coordinate
///   type.
/// - 2 components instead of 3: this widget only ever renders a 2D map, so
///   there's no third axis to carry.
/// - `id` is a plain `usize`, not `Option<usize>`. `SdePoint` uses `None`
///   for a bare coordinate with no entity behind it (e.g. a bounding-box
///   corner); every `MapPoint` loaded into the widget represents a real,
///   placed node whose id is used directly as the point-set `HashMap` key
///   and the kd-tree payload (see [`Map::add_hashmap_points`]), so an
///   optional id would just push an `.unwrap()` (or a silently dropped
///   node) into those call sites with no caller ever passing `None`.
///
/// Nodes are loaded into the widget through
/// [`Map::add_hashmap_points`](super::Map::add_hashmap_points), keyed by their
/// id.
#[derive(Clone, Debug, PartialEq)]
pub struct MapPoint {
    /// Position of the node, in map coordinates.
    pub coords: [f32; 2],
    /// Node identifier, used for lookups, notifications and markers.
    pub id: usize,
    /// Display name shown next to the node; `None` if it was never set.
    pub name: Option<String>,
    /// Ids of the lines connecting this node with others.
    ///
    /// Each entry must match a key of the map passed to
    /// [`Map::add_hashmap_lines`](super::Map::add_hashmap_lines) (and
    /// [`MapSegment::id`]). The usual pattern is to push the same pair into
    /// the `connections` of **both** endpoint nodes. Line visibility is
    /// computed from the segment bounding boxes (R-tree), not from node
    /// visibility, so a line is drawn whenever its bounding box intersects
    /// the viewport.
    pub connections: Vec<(usize, usize)>,
    /// Persistent fill color for this node's default circle, in place of
    /// [`NodeStyle::fill_color`](super::NodeStyle::fill_color). `None`
    /// (the default) keeps today's behavior of every node sharing the
    /// same style color.
    ///
    /// Only consulted by the built-in circle drawn when no
    /// [`NodeTemplate`] is installed -- a custom template receives this
    /// same `MapPoint` and decides for itself whether/how to use `color`.
    pub color: Option<Color32>,
}

impl MapPoint {
    /// Creates a node with the given `id` at the given map coordinates.
    pub fn new(id: usize, coords: [f32; 2]) -> MapPoint {
        MapPoint {
            coords,
            id,
            connections: Vec::new(),
            name: None,
            color: None,
        }
    }

    /// Returns the node identifier.
    pub fn get_id(&self) -> usize {
        self.id
    }

    /// Returns the node display name (empty if it was never set).
    pub fn get_name(&self) -> String {
        self.name.clone().unwrap_or_default()
    }

    /// Sets the node display name.
    pub fn set_name(&mut self, value: String) {
        self.name = Some(value);
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

/// Configuration of a [`Map`](super::Map) widget.
///
/// [`MapSettings::default()`] provides sensible zoom limits plus a light and a
/// dark theme; the widget picks the style to apply based on
/// [`egui::Visuals::dark_mode`], using `styles[0]` in light mode and
/// `styles[1]` in dark mode.
#[derive(Clone, Debug)]
pub struct MapSettings {
    /// Maximum zoom factor.
    pub max_zoom: f32,
    /// Minimum zoom factor.
    pub min_zoom: f32,
    /// Zoom threshold above which connection lines become visible.
    pub line_visible_zoom: f32,
    /// Zoom threshold above which node names become visible when
    /// [`node_text_visibility`](Self::node_text_visibility) is
    /// [`VisibilitySetting::Always`].
    pub label_visible_zoom: f32,
    /// Controls when node names are displayed.
    pub node_text_visibility: VisibilitySetting,
    /// Effect drawn on nodes registered with
    /// [`Map::update_marker`](super::Map::update_marker).
    ///
    /// Persistent, so it keeps the app repainting for as long as a marker
    /// exists. Ignored when a [`NodeTemplate`] is installed.
    ///
    /// Node *state* set through [`NodeHandle`](super::NodeHandle) picks its own
    /// effect per node and does not read this field.
    pub marker_animation: SteadyAnimation,
    /// Font size, **in screen pixels**, of the node names.
    ///
    /// This is a screen-space size: it deliberately does *not* scale with the
    /// zoom factor, so a name stays exactly as readable when the map is zoomed
    /// all the way out as when it is zoomed in. Because the nodes pack closer
    /// together as you zoom out while the names keep their size, names take up
    /// proportionally more of the view down there — use
    /// [`label_visible_zoom`](Self::label_visible_zoom) or
    /// [`node_text_visibility`](Self::node_text_visibility) to control when
    /// they are worth showing at all.
    pub node_text_size: f32,
    /// Font size, **in screen pixels**, of the free-floating [`MapLabel`]s.
    ///
    /// Screen-space, exactly like [`node_text_size`](Self::node_text_size).
    pub label_text_size: f32,
    /// Per-mode styles; index `0` is used in light mode, index `1` in dark
    /// mode. Their colors are kept in sync with the active
    /// [`MapTheme`](super::theme::MapTheme) -- see
    /// [`Map::set_theme`](super::Map::set_theme) -- rather than set here.
    pub styles: Vec<Style>,
}

impl MapSettings {
    /// Creates settings with all zoom thresholds set to `0.0` and a single
    /// transparent style.
    ///
    /// Prefer [`MapSettings::default()`] unless you really need to build the
    /// configuration from scratch.
    pub fn new() -> Self {
        MapSettings {
            max_zoom: 0.0,
            min_zoom: 0.0,
            line_visible_zoom: 0.0,
            label_visible_zoom: 0.0,
            node_text_visibility: VisibilitySetting::Always,
            marker_animation: SteadyAnimation::Blink,
            node_text_size: 12.0,
            label_text_size: 24.0,
            styles: vec![Style::new()],
        }
    }
}

impl Default for MapSettings {
    /// Returns the default configuration: zoom from `0.1` to `2.0`, connection
    /// lines visible above `0.2`, node names above `0.58`, and built-in light
    /// and dark themes.
    fn default() -> Self {
        let mut obj = MapSettings {
            max_zoom: 2.0,
            min_zoom: 0.1,
            line_visible_zoom: 0.2,
            label_visible_zoom: 0.58,
            node_text_visibility: VisibilitySetting::Always,
            marker_animation: SteadyAnimation::Blink,
            node_text_size: 12.0,
            label_text_size: 24.0,
            styles: Vec::new(),
        };

        // The background color below is a placeholder, overwritten by
        // `Map::assign_visual_style` from egui's own visuals on the first
        // frame. `Style` carries no color of its own -- every color the
        // widget paints with comes live from the default `MapTheme` (see
        // `Map::set_theme`/`Map::theme_colors`), so there is nothing here to
        // keep in sync with a `Theme`.

        // light style
        obj.styles.push(Style {
            line_width: Some(2.0),
            font: Some(FontId::new(12.00, FontFamily::Proportional)),
            background_color: Color32::WHITE,
        });

        // dark style
        obj.styles.push(Style {
            line_width: Some(2.0),
            font: Some(FontId::new(12.00, FontFamily::Proportional)),
            background_color: Color32::DARK_GRAY,
        });
        obj
    }
}

/// A built-in effect that plays once and ends.
///
/// Anchored to the [`Instant`] an event happened, these are the animations
/// reached through [`NodeHandle`](super::NodeHandle): `map.node(id)?.ripple(t)`.
/// The widget drops the notification and stops repainting once the effect
/// finishes. See [`crate::map::animation`] for what each looks like and how
/// long it runs.
///
/// Ignored when a [`NodeTemplate`] is installed — the template's
/// `notification_ui` takes over. The effects stay reachable there through
/// [`Animation`](crate::map::animation::Animation).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NodeAnimation {
    /// Expanding, fading disc. Reads as "one thing happened here".
    #[default]
    Pulse,
    /// Three staggered expanding rings. Reads as "activity is ongoing".
    Ripple,
    /// A ring that empties clockwise. Reads as "how old is this information".
    CountdownArc,
    /// A disc that overshoots its size and settles. For nodes that just appeared.
    ScaleIn,
    /// Four ticks converging on the node. Reads as "target acquired".
    Crosshair,
}

/// A built-in effect that runs until it is cleared.
///
/// Named after how long it lasts rather than after who uses it, because it has
/// two consumers: node state set through [`NodeHandle`](super::NodeHandle)
/// (`map.node(id)?.halo()`), and markers registered with
/// [`Map::update_marker`](super::Map::update_marker), which pick their look
/// with [`MapSettings::marker_animation`].
///
/// These never end, so the widget keeps requesting repaints for as long as one
/// is active. That is fine for the handful of elements they are meant for, but
/// it does keep the app redrawing — see [`crate::map::animation`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SteadyAnimation {
    /// Thick ring blinking on and off. The long-standing marker look.
    #[default]
    Blink,
    /// Ring whose opacity breathes in and out. Calmer than [`Self::Blink`].
    Halo,
    /// A dot circling the node. Reads as "under observation".
    Orbit,
}

/// Which endpoint a [`SegmentAnimation::Comet`] pass starts from.
///
/// A segment's own endpoint order (`a`, `b` as loaded through
/// [`Map::add_lines`](super::Map::add_lines)) is not usually meaningful to a
/// caller — naming the two ends [`Self::Forward`]/[`Self::Reverse`] instead
/// keeps the choice about the animation's direction, not about internal
/// storage order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CometDirection {
    /// From the segment's first endpoint to its second.
    #[default]
    Forward,
    /// From the segment's second endpoint to its first.
    Reverse,
}

/// A built-in effect that plays once and ends, for a segment.
///
/// Anchored to the [`Instant`] an event happened, these are the animations
/// reached through [`SegmentHandle`](super::SegmentHandle):
/// `map.segment(id)?.flash(t)`. The widget drops the notification and stops
/// repainting once the effect finishes. See [`crate::map::animation`] for
/// what each looks like and how long it runs.
///
/// Ignored when a [`SegmentTemplate`] is installed — the template's
/// `segment_notification_ui` takes over. The effect stays reachable there
/// through [`Animation`](crate::map::animation::Animation).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SegmentAnimation {
    /// A brief bright flash on the line that fades back out. The segment
    /// analogue of [`NodeAnimation::Pulse`] — reads as "something happened on
    /// this route".
    #[default]
    FlashDecay,
    /// A single dot pass from one endpoint to the other, then gone — the
    /// event-driven counterpart to [`SteadySegmentAnimation::Comet`]. Reads
    /// as "one thing moved along this route just now", direction included,
    /// rather than "traffic keeps flowing this way".
    Comet(CometDirection),
    /// The line drawing itself in from the first endpoint to the second,
    /// then gone. Reads as "this route was just established" rather than
    /// "something travelled along it".
    Wipe,
}

/// A built-in effect that runs until it is cleared, for a segment.
///
/// Reached through node state set on [`SegmentHandle`](super::SegmentHandle)
/// (`map.segment(id)?.comet()` / `.dash()`). Like [`SteadyAnimation`], these
/// never end, so the widget keeps requesting repaints for as long as one is
/// active — fine for a handful of highlighted routes, not for every segment
/// on the map.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SteadySegmentAnimation {
    /// A dot travelling from one endpoint to the other and looping. Reads as
    /// "this is the direction of flow".
    #[default]
    Comet,
    /// A dashed line whose pattern slides along the segment ("marching
    /// ants"). Reads as "route", classic for a path someone might follow.
    Dash,
    /// A localized band of brightness travelling the length of the segment
    /// and looping, fading out before it reaches either end rather than
    /// snapping back. Reads as "flow", softer and less busy than [`Self::Dash`].
    GlowBand,
    /// A row of arrow shapes sliding along the segment, pointing the way.
    /// Reads as "direction of travel", more explicit than [`Self::Comet`]'s
    /// single dot.
    Chevrons,
}

/// Controls when the name of a node is displayed next to it.
#[derive(Clone, Debug, PartialEq)]
pub enum VisibilitySetting {
    /// Never show node names.
    Hidden,
    /// Only show the name of the node closest to the mouse pointer.
    Hover,
    /// Always show node names, subject to [`MapSettings::label_visible_zoom`].
    Always,
}

/// Provides the contents of the widget's right-click context menu.
///
/// Install an implementation with
/// [`Map::set_context_manager`](super::Map::set_context_manager).
///
/// # Examples
///
/// ```
/// use egui_map::map::objects::ContextMenuManager;
///
/// struct MyMenu;
///
/// impl ContextMenuManager for MyMenu {
///     fn ui(&self, ui: &mut egui::Ui) {
///         ui.label("Hello from the map!");
///     }
/// }
/// ```
pub trait ContextMenuManager {
    /// Builds the menu contents; called every frame while the menu is open.
    fn ui(&self, ui: &mut Ui);
}

/// Customizes how nodes and their visual effects are rendered.
///
/// When a template is installed with
/// [`Map::set_node_template`](super::Map::set_node_template), the widget
/// delegates all node painting to it instead of using the built-in shapes and
/// animations — including the node name labels, so draw the name yourself in
/// [`NodeTemplate::node_ui`] if you need it.
///
/// The positions passed to these methods are in screen coordinates: already
/// scaled by `zoom` and translated to the viewport origin. Multiply every size
/// by `zoom` so your shapes scale together with the map.
///
/// # Animation idioms
///
/// egui only repaints on demand, so any method that animates (a blinking
/// marker, a fading notification, ...) must call
/// [`ui.ctx().request_repaint()`](egui::Context::request_repaint) to keep the
/// frames coming. Time-driven effects are usually computed from
/// [`Instant::now()`] (see `initial_time` in
/// [`NodeTemplate::notification_ui`]) or from the system clock.
///
/// # Examples
///
/// A node drawn as a rounded box with its name inside, plus a notification
/// animation that expands and fades out over two seconds:
///
/// ```
/// use egui_map::map::objects::{MapPoint, NodeContext, NodeTemplate, NotificationContext, MarkerContext, SelectionContext};
/// use egui::{Align2, Color32, CornerRadius, FontId, Pos2, Rect, Stroke, Ui, Vec2};
/// use std::time::Instant;
///
/// struct BoxedNodes;
///
/// impl NodeTemplate for BoxedNodes {
///     fn node_ui(&self, ui: &mut Ui, ctx: NodeContext) {
///         // Multiply every size by `ctx.zoom` so the node scales with the map.
///         let rect = Rect::from_center_size(ctx.position, Vec2::new(90.0 * ctx.zoom, 35.0 * ctx.zoom));
///         let rounding = CornerRadius::same((10.0 * ctx.zoom) as u8);
///         let painter = ui.painter();
///         // `ctx.color` is already resolved: `ctx.point.color` if the node has
///         // its own override, otherwise the active theme's node color.
///         painter.rect_filled(rect, rounding, ctx.color);
///         painter.rect_stroke(
///             rect,
///             rounding,
///             Stroke::new(4.0 * ctx.zoom, Color32::WHITE),
///             egui::StrokeKind::Middle,
///         );
///         painter.text(
///             ctx.position,
///             Align2::CENTER_CENTER,
///             ctx.point.get_name(),
///             FontId::proportional(12.0 * ctx.zoom),
///             Color32::WHITE,
///         );
///     }
///
///     fn notification_ui(&self, ui: &mut Ui, ctx: NotificationContext) -> bool {
///         let secs = Instant::now().duration_since(ctx.initial_time).as_secs_f32();
///         // Expand the stroke and fade the color out over 2 seconds.
///         let alpha = (1.0 - secs / 2.0).clamp(0.0, 1.0);
///         let fading = Color32::from_rgba_unmultiplied(
///             ctx.color.r(),
///             ctx.color.g(),
///             ctx.color.b(),
///             (255.0 * alpha) as u8,
///         );
///         let rect = Rect::from_center_size(ctx.position, Vec2::new(90.0 * ctx.zoom, 35.0 * ctx.zoom));
///         ui.painter().rect_stroke(
///             rect,
///             CornerRadius::same((10.0 * ctx.zoom) as u8),
///             Stroke::new((4.0 + 25.0 * secs) * ctx.zoom, fading),
///             egui::StrokeKind::Middle,
///         );
///         // Keep the animation frames coming.
///         ui.ctx().request_repaint();
///         // Returning `false` removes the notification.
///         secs < 2.0
///     }
///     # fn selection_ui(&self, ui: &mut Ui, ctx: SelectionContext) {
///     #     let rect = Rect::from_center_size(ctx.position, Vec2::new(94.0 * ctx.zoom, 39.0 * ctx.zoom));
///     #     ui.painter().rect_stroke(
///     #         rect,
///     #         CornerRadius::same((10.0 * ctx.zoom) as u8),
///     #         Stroke::new(3.0 * ctx.zoom, ctx.color),
///     #         egui::StrokeKind::Middle,
///     #     );
///     # }
///     # fn marker_ui(&self, ui: &mut Ui, ctx: MarkerContext) {
///     #     ui.painter().circle_stroke(ctx.position, 6.0 * ctx.zoom, Stroke::new(2.0 * ctx.zoom, Color32::LIGHT_GREEN));
///     #     ui.ctx().request_repaint();
///     # }
/// }
/// ```
///
/// # Note on `NodeAnimation`/`SteadyAnimation` in the examples above
///
/// Every method here takes a context struct -- [`NodeContext`],
/// [`SelectionContext`], [`NotificationContext`] or [`MarkerContext`] -- each
/// `#[non_exhaustive]` so a future field can be added without another
/// breaking change to `NodeTemplate` itself.
pub trait NodeTemplate {
    /// Draws a node, replacing the default filled circle.
    ///
    /// Called every frame for each visible node. The widget no longer draws
    /// the node name once a template is installed, so render it here (e.g.
    /// with [`Painter::text`](egui::Painter::text)) if you need it. See
    /// [`NodeContext`] for the fields available, in particular `ctx.color`
    /// -- the color already resolved for this node, so you don't have to
    /// repeat the `point.color.unwrap_or(...)` fallback (or reach for the
    /// active theme yourself) to honor a per-node color override.
    fn node_ui(&self, ui: &mut Ui, ctx: NodeContext);

    /// Draws the highlight over the node closest to the mouse pointer.
    ///
    /// The nearest node is only computed while the pointer is over the map and
    /// [`MapSettings::node_text_visibility`] is [`VisibilitySetting::Hover`].
    /// See [`SelectionContext`] for the fields available, in particular
    /// `ctx.point` (which node is being highlighted) and `ctx.color` (the
    /// active theme's selection color, resolved for you).
    fn selection_ui(&self, ui: &mut Ui, ctx: SelectionContext);

    /// Draws the notification effect of a node notified at
    /// `ctx.initial_time`.
    ///
    /// Called every frame for each node passed to
    /// [`Map::notify`](super::Map::notify) or animated through
    /// [`Map::node`](super::Map::node)'s event methods (`pulse`, `ripple`,
    /// ...). `ctx.kind` is which of those was requested and `ctx.node_id` is
    /// the id of the node it belongs to -- use them to dispatch to the
    /// matching built-in [`Animation`](crate::map::animation::Animation)
    /// function (or your own effect) instead of reimplementing every
    /// animation by hand. See [`NotificationContext`] for the rest of the
    /// fields. Should return `true` while the animation is still playing —
    /// remember to call
    /// [`ui.ctx().request_repaint()`](egui::Context::request_repaint) —;
    /// once it returns `false` the notification is discarded.
    fn notification_ui(&self, ui: &mut Ui, ctx: NotificationContext) -> bool;

    /// Draws a marker over the given node.
    ///
    /// Called every frame for two different things -- see [`MarkerContext`]
    /// for what `ctx.kind`/`ctx.node_id` mean in each case. For animated
    /// markers (e.g. a blinking light), drive the effect from the system
    /// clock and call
    /// [`ui.ctx().request_repaint()`](egui::Context::request_repaint).
    fn marker_ui(&self, ui: &mut Ui, ctx: MarkerContext);
}

/// The context passed to [`NodeTemplate::node_ui`].
///
/// `#[non_exhaustive]`, like [`NotificationContext`]/[`MarkerContext`], so a
/// future field can be added here without another breaking change.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct NodeContext<'a> {
    /// The node's screen position: already scaled by `zoom` and translated
    /// to the viewport origin.
    pub position: Pos2,
    /// Multiply every size you draw by this so it scales with the map.
    pub zoom: f32,
    /// The node being painted -- its id, name, coordinates and connections.
    pub point: &'a MapPoint,
    /// The color requested for this node: [`point.color`](MapPoint::color)
    /// if the node has its own override, otherwise the active
    /// [`MapTheme`](super::theme::MapTheme)'s
    /// [`ThemeColors::node`](super::theme::ThemeColors::node) for the
    /// current color mode -- the same fallback the built-in circle uses when
    /// no template is installed, resolved once here so every `NodeTemplate`
    /// doesn't need to repeat it.
    pub color: Color32,
}

/// The context passed to [`NodeTemplate::selection_ui`].
///
/// `#[non_exhaustive]`, like [`NodeContext`]/[`NotificationContext`]/
/// [`MarkerContext`], so a future field can be added here without another
/// breaking change.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct SelectionContext<'a> {
    /// The node's screen position: already scaled by `zoom` and translated
    /// to the viewport origin.
    pub position: Pos2,
    /// Multiply every size you draw by this so it scales with the map.
    pub zoom: f32,
    /// The node the highlight belongs to -- the one closest to the mouse
    /// pointer. Its id, name, coordinates and connections.
    pub point: &'a MapPoint,
    /// The active [`MapTheme`](super::theme::MapTheme)'s
    /// [`ThemeColors::selected`](super::theme::ThemeColors::selected) for
    /// the current color mode -- resolved once here so every `NodeTemplate`
    /// doesn't need to reach for the theme itself.
    pub color: Color32,
}

/// The context passed to [`NodeTemplate::notification_ui`].
///
/// `#[non_exhaustive]` so a future field can be added here without another
/// breaking change to [`NodeTemplate`].
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct NotificationContext {
    /// The node's screen position: already scaled by `zoom` and translated
    /// to the viewport origin.
    pub position: Pos2,
    /// Multiply every size you draw by this so it scales with the map.
    pub zoom: f32,
    /// When the notification started -- usually fed into a progress
    /// computation like `Instant::now().duration_since(initial_time)`.
    pub initial_time: Instant,
    /// The color requested for this notification (the node's own color, or
    /// the current style's `alert_color` if none was set).
    pub color: Color32,
    /// Which built-in event effect was requested (`pulse`, `ripple`, ...).
    /// Match on this to dispatch to the corresponding
    /// [`Animation`](crate::map::animation::Animation) function instead of
    /// reimplementing the lookup yourself.
    pub kind: NodeAnimation,
    /// The id of the node this notification belongs to.
    pub node_id: usize,
}

/// The context passed to [`NodeTemplate::marker_ui`].
///
/// `#[non_exhaustive]`, like [`NotificationContext`], so a future field can
/// be added here without another breaking change.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct MarkerContext {
    /// The node's screen position: already scaled by `zoom` and translated
    /// to the viewport origin.
    pub position: Pos2,
    /// Multiply every size you draw by this so it scales with the map.
    pub zoom: f32,
    /// Which persistent effect to draw. For a node's own lasting state (set
    /// through [`Map::node`](super::Map::node)'s `halo`/`blink`/`orbit`)
    /// this is whichever of those was requested; for a plain marker
    /// (registered with [`Map::update_marker`](super::Map::update_marker))
    /// it is always [`MapSettings::marker_animation`], since every marker
    /// shares that one setting. There is no way from inside this hook to
    /// tell the two *cases* apart -- only which `SteadyAnimation` to draw
    /// for whichever one it is.
    pub kind: SteadyAnimation,
    /// The id of the node the state/marker belongs to (for a marker: the id
    /// it points at, not the marker's own id).
    pub node_id: usize,
}

/// Customizes how segments and their visual effects are rendered.
///
/// When a template is installed with
/// [`Map::set_segment_template`](super::Map::set_segment_template), the widget
/// delegates all segment painting to it instead of using the built-in stroke
/// and animations.
///
/// Unlike [`NodeTemplate`], these methods receive a bare [`&Painter`](Painter)
/// rather than `&mut Ui`. Segments are visited in bulk, every frame, after the
/// R-tree viewport culling in `paint_map_lines`; going through `Ui` would cost
/// a layout pass per segment, on top of what the culling already had to
/// discard. Use [`Painter::ctx`] to reach the [`egui::Context`] — for example
/// to call `request_repaint()`.
///
/// The positions passed to these methods are in screen coordinates: already
/// scaled by `zoom` and translated to the viewport origin, same as
/// [`NodeTemplate`]'s. Multiply every size by `zoom` so your shapes scale
/// together with the map.
///
/// # Examples
///
/// A segment drawn as a dashed line, plus a notification that briefly
/// thickens and brightens it:
///
/// ```
/// use egui_map::map::objects::{MapSegment, SegmentTemplate};
/// use egui::{Color32, Painter, Pos2, Stroke};
/// use std::time::Instant;
///
/// struct DashedRoutes;
///
/// impl SegmentTemplate for DashedRoutes {
///     fn segment_ui(&self, painter: &Painter, a: Pos2, b: Pos2, zoom: f32, _segment: &MapSegment) {
///         // A crude dash: short strokes along the segment, spaced in screen
///         // pixels so they don't stretch as the map zooms.
///         let dir = b - a;
///         let len = dir.length();
///         let step = 10.0 * zoom;
///         let mut travelled = 0.0;
///         while travelled < len {
///             let start = a + dir * (travelled / len);
///             let end = a + dir * ((travelled + step * 0.6).min(len) / len);
///             painter.line_segment([start, end], Stroke::new(2.0 * zoom, Color32::GRAY));
///             travelled += step;
///         }
///     }
///
///     fn segment_notification_ui(
///         &self,
///         painter: &Painter,
///         a: Pos2,
///         b: Pos2,
///         zoom: f32,
///         initial_time: Instant,
///         color: Color32,
///     ) -> bool {
///         let secs = Instant::now().duration_since(initial_time).as_secs_f32();
///         let alpha = (1.0 - secs).clamp(0.0, 1.0);
///         let fading =
///             Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), (255.0 * alpha) as u8);
///         painter.line_segment([a, b], Stroke::new(5.0 * zoom, fading));
///         painter.ctx().request_repaint();
///         secs < 1.0
///     }
///
///     fn segment_state_ui(&self, painter: &Painter, a: Pos2, b: Pos2, zoom: f32, time: f32, color: Color32) {
///         let t = (time / 1.6).rem_euclid(1.0);
///         painter.circle_filled(a + (b - a) * t, 4.0 * zoom, color);
///         painter.ctx().request_repaint();
///     }
/// }
/// ```
pub trait SegmentTemplate {
    /// Draws a segment, replacing the default stroked line.
    ///
    /// Called every frame for each segment that survives the R-tree viewport
    /// culling in `paint_map_lines`.
    fn segment_ui(
        &self,
        painter: &Painter,
        pos_a: Pos2,
        pos_b: Pos2,
        zoom: f32,
        segment: &MapSegment,
    );

    /// Draws the notification effect of a segment notified through
    /// [`Map::segment`](super::Map::segment).
    ///
    /// Called every frame for each segment carrying an event-driven effect
    /// (see [`SegmentHandle`](super::SegmentHandle)). Should return `true`
    /// while the animation is still playing — remember to call
    /// [`Painter::ctx`]`().request_repaint()` — once it returns `false` the
    /// notification is discarded.
    fn segment_notification_ui(
        &self,
        painter: &Painter,
        pos_a: Pos2,
        pos_b: Pos2,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool;

    /// Draws the lasting state effect of a segment (e.g. a travelling dot).
    ///
    /// Called every frame for each segment with lasting state set through
    /// [`Map::segment`](super::Map::segment). `time` is the frame time in
    /// seconds (`ui.input(|i| i.time)`), so every element animated this frame
    /// shares one clock. For animated state, remember to call
    /// [`Painter::ctx`]`().request_repaint()`.
    fn segment_state_ui(
        &self,
        painter: &Painter,
        pos_a: Pos2,
        pos_b: Pos2,
        zoom: f32,
        time: f32,
        color: Color32,
    );
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

    // ---------- MapSegment ----------

    #[test]
    fn map_segment_new_computes_tight_aabb() {
        let seg = MapSegment::new((1, 2), [10.0, -5.0], [-2.0, 7.0]);
        assert_eq!(seg.id, (1, 2));
        let envelope: AABB<[f32; 2]> = rstar::RTreeObject::envelope(&seg);
        assert_eq!(envelope.lower(), [-2.0, -5.0]);
        assert_eq!(envelope.upper(), [10.0, 7.0]);
        assert_eq!(seg.raw_line().points[0].components, [10.0, -5.0]);
        assert_eq!(seg.raw_line().points[1].components, [-2.0, 7.0]);
    }

    #[test]
    fn map_segment_envelope_returns_its_aabb() {
        let seg = MapSegment::new((1, 2), [0.0, 0.0], [4.0, 2.0]);
        let envelope: AABB<[f32; 2]> = rstar::RTreeObject::envelope(&seg);
        assert_eq!(envelope.lower(), [0.0, 0.0]);
        assert_eq!(envelope.upper(), [4.0, 2.0]);
    }

    #[test]
    fn map_segment_degenerate_line_has_point_aabb() {
        // A zero-length segment must still produce a valid (empty-area) AABB.
        let seg = MapSegment::new((1, 2), [3.0, 3.0], [3.0, 3.0]);
        let envelope: AABB<[f32; 2]> = rstar::RTreeObject::envelope(&seg);
        assert_eq!(envelope.lower(), [3.0, 3.0]);
        assert_eq!(envelope.upper(), [3.0, 3.0]);
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
    #[allow(clippy::op_ref)] // se prueba a propósito la impl Add<&RawPoint>
    fn raw_point_add_ref() {
        let a = RawPoint::new(1.0, 2.0);
        let b = RawPoint::new(3.0, -4.0);
        let c = a + &b;
        assert_eq!(c.components, [4.0, -2.0]);
        // b sigue siendo usable tras la suma por referencia
        assert_eq!(b.components, [3.0, -4.0]);
    }

    #[test]
    #[allow(clippy::op_ref)] // se prueba a propósito la impl Sub<&RawPoint>
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
    fn raw_line_distance_to_point_on_segment() {
        let line = RawLine::new(RawPoint::new(0.0, 0.0), RawPoint::new(10.0, 0.0));
        assert_eq!(line.distance_to_point(RawPoint::new(5.0, 3.0)), 3.0);
        assert_eq!(line.distance_to_point(RawPoint::new(5.0, 0.0)), 0.0);
    }

    #[test]
    fn raw_line_distance_to_point_beyond_endpoints() {
        let line = RawLine::new(RawPoint::new(0.0, 0.0), RawPoint::new(10.0, 0.0));
        // Past the end of the segment, the closest point is the endpoint.
        assert_eq!(line.distance_to_point(RawPoint::new(14.0, 0.0)), 4.0);
        assert_eq!(line.distance_to_point(RawPoint::new(-3.0, -4.0)), 5.0);
    }

    #[test]
    fn raw_line_distance_to_point_degenerate_segment() {
        let line = RawLine::new(RawPoint::new(1.0, 1.0), RawPoint::new(1.0, 1.0));
        assert_eq!(line.distance_to_point(RawPoint::new(4.0, 5.0)), 5.0);
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

    fn full_style() -> Style {
        Style {
            line_width: Some(4.0),
            font: Some(FontId::new(10.0, FontFamily::Proportional)),
            background_color: Color32::BLACK,
        }
    }

    #[test]
    fn map_style_new() {
        let s = Style::new();
        assert!(s.line_width.is_none());
        assert!(s.font.is_none());
        assert_eq!(s.background_color, Color32::TRANSPARENT);
    }

    #[test]
    fn map_style_default_equals_new() {
        let s = Style::default();
        assert!(s.line_width.is_none());
        assert!(s.font.is_none());
    }

    #[test]
    fn map_style_mul_i64() {
        let s = full_style() * 2i64;
        assert_eq!(s.line_width.unwrap(), 8.0);
        assert_eq!(s.font.unwrap().size, 20.0);
    }

    #[test]
    fn map_style_mul_i32() {
        let s = full_style() * 2i32;
        assert_eq!(s.line_width.unwrap(), 8.0);
        assert_eq!(s.font.unwrap().size, 20.0);
    }

    #[test]
    fn map_style_mul_f32() {
        let s = full_style() * 0.5f32;
        assert_eq!(s.line_width.unwrap(), 2.0);
        assert_eq!(s.font.unwrap().size, 5.0);
    }

    #[test]
    fn map_style_mul_f64() {
        let s = full_style() * 0.5f64;
        assert_eq!(s.line_width.unwrap(), 2.0);
        assert_eq!(s.font.unwrap().size, 5.0);
    }

    #[test]
    fn map_style_div_i64() {
        let s = full_style() / 2i64;
        assert_eq!(s.line_width.unwrap(), 2.0);
        assert_eq!(s.font.unwrap().size, 5.0);
    }

    #[test]
    fn map_style_div_i32() {
        let s = full_style() / 2i32;
        assert_eq!(s.line_width.unwrap(), 2.0);
        assert_eq!(s.font.unwrap().size, 5.0);
    }

    #[test]
    fn map_style_div_f32() {
        let s = full_style() / 0.5f32;
        assert_eq!(s.line_width.unwrap(), 8.0);
        assert_eq!(s.font.unwrap().size, 20.0);
    }

    #[test]
    fn map_style_div_f64() {
        let s = full_style() / 0.5f64;
        assert_eq!(s.line_width.unwrap(), 8.0);
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

    // ---------- MapPoint ----------

    #[test]
    fn map_point_new() {
        let p = MapPoint::new(42, [1.0, 2.0]);
        assert_eq!(p.get_id(), 42);
        assert_eq!(p.coords, [1.0, 2.0]);
        assert!(p.connections.is_empty());
        assert_eq!(p.name, None);
        assert_eq!(p.get_name(), String::new());
    }

    #[test]
    fn map_point_set_and_get_name() {
        let mut p = MapPoint::new(1, [0.0, 0.0]);
        p.set_name("Jita".to_string());
        assert_eq!(p.name, Some("Jita".to_string()));
        assert_eq!(p.get_name(), "Jita");
    }

    #[test]
    fn map_point_from_occupied_entry() {
        let mut map: HashMap<usize, MapPoint> = HashMap::new();
        let mut original = MapPoint::new(7, [5.0, 6.0]);
        original.set_name("Amarr".to_string());
        map.insert(7, original);

        use std::collections::hash_map::Entry;
        if let Entry::Occupied(entry) = map.entry(7) {
            let cloned = MapPoint::from(entry);
            assert_eq!(cloned.get_id(), 7);
            assert_eq!(cloned.get_name(), "Amarr");
            assert_eq!(cloned.coords, [5.0, 6.0]);
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
        assert_eq!(s.node_text_visibility, VisibilitySetting::Always);
        assert_eq!(s.marker_animation, SteadyAnimation::Blink);
        assert_eq!(s.node_text_size, 12.0);
        assert_eq!(s.label_text_size, 24.0);
        assert_eq!(s.styles.len(), 1);
    }

    #[test]
    fn map_settings_default() {
        let s = MapSettings::default();
        assert_eq!(s.max_zoom, 2.0);
        assert_eq!(s.min_zoom, 0.1);
        assert_eq!(s.line_visible_zoom, 0.2);
        assert_eq!(s.label_visible_zoom, 0.58);
        assert_eq!(s.node_text_visibility, VisibilitySetting::Always);
        assert_eq!(s.marker_animation, SteadyAnimation::Blink);
        assert_eq!(s.node_text_size, 12.0);
        assert_eq!(s.label_text_size, 24.0);
        // light + dark themes
        assert_eq!(s.styles.len(), 2);
        // light theme
        assert_eq!(s.styles[0].background_color, Color32::WHITE);
        assert!(s.styles[0].line_width.is_some());
        assert!(s.styles[0].font.is_some());
        // dark theme
        assert_eq!(s.styles[1].background_color, Color32::DARK_GRAY);
        assert!(s.styles[1].line_width.is_some());
        assert!(s.styles[1].font.is_some());
    }

    // ---------- VisibilitySetting ----------

    #[test]
    fn visibility_setting_equality() {
        assert_eq!(VisibilitySetting::Hidden, VisibilitySetting::Hidden);
        assert_eq!(VisibilitySetting::Hover, VisibilitySetting::Hover);
        assert_eq!(VisibilitySetting::Always, VisibilitySetting::Always);
        assert_ne!(VisibilitySetting::Hidden, VisibilitySetting::Hover);
        assert_ne!(VisibilitySetting::Hover, VisibilitySetting::Always);
        assert_ne!(VisibilitySetting::Hidden, VisibilitySetting::Always);
    }
}

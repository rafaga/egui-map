//! Interactive map widget and the data types it renders.
//!
//! [`Map`] is an [`egui::Widget`] that draws a 2D set of nodes
//! ([`objects::MapPoint`]), the connection lines between them
//! ([`objects::MapSegment`]) and free-floating text labels
//! ([`objects::MapLabel`]). Nodes are indexed in a kd-tree so that only the
//! ones inside the current viewport are painted each frame.
//!
//! ## Coordinate model
//!
//! The widget works with two coordinate spaces:
//!
//! - **Map coordinates**: the logical position of your nodes, as loaded through
//!   [`Map::add_hashmap_points`].
//! - **Screen coordinates**: positions inside the widget's rectangle on screen.
//!
//! Both are related by the current zoom factor and viewport origin:
//! `screen = map * zoom - origin`. Use [`Map::set_zoom`], [`Map::set_pos`] and
//! [`Map::set_pos_from_nodeid`] to control the visible region.
//!
//! ## Connecting nodes with lines
//!
//! Lines are wired up in three steps:
//!
//! 1. Create the nodes as a [`HashMap`] keyed by node id.
//! 2. For every connection, choose a unique `(usize, usize)` id -- typically
//!    the pair of node ids it joins -- and push it into
//!    [`MapPoint::connections`] of **both** endpoint nodes.
//! 3. Load the nodes with [`Map::add_hashmap_points`], then load a
//!    [`HashMap`] of [`MapSegment`] keyed by those same connection ids and
//!    add it to the widget with [`Map::add_hashmap_lines`].
//!
//! ```
//! use egui_map::map::Map;
//! use egui_map::map::objects::{MapPoint, MapSegment};
//! use std::collections::HashMap;
//!
//! // 1. Create the nodes.
//! let mut points: HashMap<usize, MapPoint> = HashMap::new();
//! points.insert(1, MapPoint::new(1, [0.0, 0.0]));
//! points.insert(2, MapPoint::new(2, [10.0, 10.0]));
//!
//! // 2. Register the connection id on both endpoints.
//! for id in [1, 2] {
//!     points.get_mut(&id).unwrap().connections.push((1, 2));
//! }
//!
//! let mut map = Map::new();
//! map.add_hashmap_points(points);
//!
//! // 3. Provide the line geometry keyed by the same connection id.
//! let mut lines: HashMap<(usize, usize), MapSegment> = HashMap::new();
//! lines.insert((1, 2), MapSegment::new((1, 2), [0.0, 0.0], [10.0, 10.0]));
//! map.add_hashmap_lines(lines);
//! ```
//!
//! A line is only drawn while the zoom level is above
//! [`MapSettings::line_visible_zoom`] and its bounding box intersects the
//! viewport. Segments are culled broad-phase with an R-tree built by
//! [`Map::add_lines`], so long lines crossing the view are drawn even when
//! both endpoints lie outside of it.
//!
//! ## Custom node rendering
//!
//! Install a [`NodeTemplate`] implementation with [`Map::set_node_template`]
//! to take over the rendering of nodes, selection highlights, notification
//! animations and markers. Note that this replaces
//! *all* built-in node rendering, including the node name labels: draw them
//! yourself in [`NodeTemplate::node_ui`] if you need them.
//!
//! ## Animating nodes and segments
//!
//! [`Map::node`] and [`Map::segment`] borrow a node or a segment already
//! loaded into the widget and return a handle -- [`NodeHandle`] /
//! [`SegmentHandle`] -- with one method per built-in effect. Effects come in
//! two families: event-driven ones (`pulse`, `flash`, ...) play once from an
//! [`Instant`] and stop on their own; lasting ones (`halo`, `comet`, ...) run
//! until [`NodeHandle::clear`] / [`SegmentHandle::clear`] and keep the app
//! repainting the whole time they're active.
//!
//! ```
//! use egui_map::map::Map;
//! use egui_map::map::objects::{MapPoint, MapSegment};
//! use std::time::Instant;
//!
//! let mut map = Map::new();
//! map.add_points(vec![MapPoint::new(1, [0.0, 0.0])]);
//! map.add_lines(vec![MapSegment::new((1, 1), [0.0, 0.0], [10.0, 0.0])]);
//!
//! if let Some(node) = map.node(1) {
//!     node.pulse(Instant::now());
//! }
//! if let Some(segment) = map.segment((1, 1)) {
//!     segment.comet();
//! }
//! ```
//!
//! To fully replace how an effect looks, install a [`objects::NodeTemplate`] /
//! [`objects::SegmentTemplate`] and implement its `notification_ui` /
//! `segment_notification_ui` and `marker_ui` / `segment_state_ui` hooks -- or
//! call [`animation::Animation`]'s functions directly from either template if
//! you only want to reuse the built-in look.

use crate::map::animation::Animation;
use crate::map::objects::{
    CometDirection, ContextMenuManager, HitContext, LabelContext, MapBounds, MapLabel, MapPoint,
    MapSegment, MapSettings, MarkerContext, NodeAnimation, NodeContext, NotificationContext,
    RawLine, RawPoint, RegionLabel, SegmentAnimation, SegmentContext, SegmentNotificationContext,
    SegmentStateContext, SelectionContext, SteadyAnimation, SteadySegmentAnimation, TextSettings,
    VisibilitySetting,
};
use crate::map::theme::{ColorMode, MapTheme, Theme, ThemeColors};
use egui::text::Galley;
use egui::{widgets::*, *};
use kdtree::KdTree;
use kdtree::distance::squared_euclidean;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use self::objects::{LabelTemplate, NodeTemplate, SegmentTemplate};

pub mod animation;
pub mod objects;
pub mod outline;
pub mod theme;

/// How much more opaque a segment effect (`comet`, `dash`, `flash`, ...)
/// stays than the segment's own zoom-based fade-in, in `paint_map_lines` --
/// see `line_fade`/`effect_fade` there. Effects track the line's fade rather
/// than ignoring it, but always keep this much of a head start so they read
/// as at least as visible as the line beneath them instead of fading out in
/// lockstep and risking disappearing into it.
const SEGMENT_EFFECT_ALPHA_BOOST: f32 = 0.2;

/// How far outside the visible rect a [`RegionLabel`]'s *center* point
/// may still fall and be painted, expressed as a multiple of its current
/// (zoom-scaled) font size.
///
/// The viewport cull below only has each label's projected center point
/// available -- its actual painted width depends on the region's name
/// and isn't known until the text is laid out, which is exactly the
/// cost this cull exists to skip for labels nobody will see. `6.0` is a
/// generous stand-in for "half the width of a typical region name plus
/// some slack", picked to avoid visible pop-in at the edge of the
/// screen rather than computed from an exact bound; it costs nothing to
/// be generous here since the whole point is discarding the labels far
/// outside the viewport, and a modest few false positives near the edge
/// do not undermine that.
const REGION_LABEL_CULL_MARGIN_FACTOR: f32 = 6.0;

/// Returns `color` with its alpha multiplied by `factor` (clamped to
/// `0.0..=1.0`), preserving whatever RGB the caller already set rather than
/// replacing it outright.
///
/// `Color32`'s `r()`/`g()`/`b()` accessors return the *premultiplied*
/// bytes it stores internally, not the original unmultiplied channels --
/// feeding those straight back into [`Color32::from_rgba_unmultiplied`]
/// with a new alpha would premultiply them a second time and darken the
/// color instead of just fading it. Going through
/// [`Color32::to_srgba_unmultiplied`] first recovers the true unmultiplied
/// RGB so only the alpha actually changes.
fn scale_alpha(color: Color32, factor: f32) -> Color32 {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    Color32::from_rgba_unmultiplied(r, g, b, (a as f32 * factor.clamp(0.0, 1.0)).round() as u8)
}

/// An interactive 2D map widget.
///
/// `Map` renders a set of nodes ([`objects::MapPoint`]), connection lines
/// ([`objects::MapSegment`]) and text labels ([`objects::MapLabel`]). The user can
/// pan the view by dragging and zoom with the mouse wheel (hold `Ctrl` — or
/// `Cmd` on macOS — to zoom faster), or use the built-in zoom slider drawn at
/// the top-right corner of the widget.
///
/// The map is fed through [`Map::add_hashmap_points`], which also builds the
/// internal kd-tree used for viewport culling and nearest-node hover queries.
/// Behavior and appearance are configured through the public
/// [`settings`](Map::settings) field (see [`objects::MapSettings`]).
///
/// Rendering of nodes and their visual effects (selection highlight,
/// notifications and markers) can be fully customized by installing a
/// [`objects::NodeTemplate`] implementation with [`Map::set_node_template`],
/// segments likewise with [`objects::SegmentTemplate`] and
/// [`Map::set_segment_template`], and region labels
/// ([`objects::RegionLabel`], added with [`Map::add_region_labels`]) with
/// [`objects::LabelTemplate`] and [`Map::set_label_template`]; a right-click
/// context menu can be provided with [`Map::set_context_manager`].
///
/// # Examples
///
/// ```no_run
/// # fn example(ui: &mut egui::Ui) {
/// use egui_map::map::Map;
/// use egui_map::map::objects::MapPoint;
/// use std::collections::HashMap;
///
/// let mut points = HashMap::new();
/// points.insert(1, MapPoint::new(1, [0.0, 0.0]));
///
/// let mut map = Map::new();
/// map.add_hashmap_points(points);
///
/// // Every frame, inside your egui update logic:
/// ui.add(&mut map);
/// # }
/// ```
#[derive(Clone)]
pub struct Map {
    zoom: f32,
    previous_zoom: f32,
    points: Option<HashMap<usize, MapPoint>>,
    segments: Option<rstar::RTree<MapSegment>>,
    labels: Vec<MapLabel>,
    region_labels: Vec<RegionLabel>,
    /// Layout cache for the built-in [`RegionLabel`] renderer, keyed by
    /// `(text, rounded screen size, font family)` so a repeated frame at a
    /// steady zoom and [`Style::region_label_font`](theme::Style::region_label_font) is a cache lookup rather
    /// than a relayout -- see [`objects::LabelTemplate::label_ui`]'s doc.
    /// Cleared whenever [`Map::add_region_labels`] replaces the label set.
    region_label_cache: HashMap<(String, i32, FontFamily), Arc<Galley>>,
    tree: Option<KdTree<f32, usize, [f32; 2]>>,
    visible_points: Vec<isize>,
    map_area: Rect,
    reference: MapBounds,
    current: MapBounds,
    /// Whether the widget last painted with `ui`'s `Visuals` in dark mode --
    /// drives [`Map::color_mode`], kept up to date by
    /// [`Map::assign_visual_style`].
    dark_mode: bool,
    notifications: HashMap<usize, Notification>,
    node_states: HashMap<usize, NodeState>,
    segment_notifications: HashMap<(usize, usize), SegmentNotification>,
    segment_states: HashMap<(usize, usize), SegmentState>,
    /// Ids of the segments currently loaded, kept alongside the R-tree so
    /// [`Map::segment`] can check whether an id exists in O(1) instead of
    /// scanning it.
    segment_ids: HashSet<(usize, usize)>,
    min_size: (Option<f32>, Option<f32>),
    max_size: (Option<f32>, Option<f32>),
    /// Behavior and appearance configuration (zoom limits, visibility
    /// thresholds and per-theme styles). See [`objects::MapSettings`].
    pub settings: MapSettings,
    menu_manager: Option<Rc<dyn ContextMenuManager>>,
    node_template: Option<Rc<dyn NodeTemplate>>,
    segment_template: Option<Rc<dyn SegmentTemplate>>,
    label_template: Option<Rc<dyn LabelTemplate>>,
    markers: HashMap<usize, usize>,
    /// Fade of [`NodeContext::marker`](objects::NodeContext::marker), keyed
    /// by node id. An entry stays after fading out (at most one per node).
    marker_presence: HashMap<usize, MarkerPresence>,
    /// The node under the pointer in the last frame; see
    /// [`Map::hovered_node`].
    hovered_node: Option<usize>,
    /// The farthest a drawn node's template outline has reached from its
    /// position, in map units: part of `find_hovered_node`'s search radius.
    /// Reset when the template changes.
    outline_reach: std::cell::Cell<f32>,
    /// The active color palette. See [`Map::set_theme`].
    theme: Rc<dyn MapTheme>,
}

/// A one-off effect attached to a node, with the moment it started.
#[derive(Clone, Copy, Debug)]
struct Notification {
    started: Instant,
    animation: NodeAnimation,
    /// `None` falls back to the active theme's `ThemeColors::alert`.
    color: Option<Color32>,
    /// When a lasting notification ([`NodeHandle::lasting`]) ends; `None`
    /// plays the effect once.
    until: Option<Instant>,
}

impl Notification {
    /// Whether a lasting notification has run its course at `now`.
    fn expired(&self, now: Instant) -> bool {
        self.until.is_some_and(|until| now >= until)
    }

    /// How much of a lasting notification is left at `now`, from `1.0` (just
    /// triggered) to `0.0` (over); always `1.0` for a one-off one. Its color
    /// fades by this factor, so a lasting alert grows fainter as it ages.
    fn remaining(&self, now: Instant) -> f32 {
        let Some(until) = self.until else {
            return 1.0;
        };
        let total = until.saturating_duration_since(self.started).as_secs_f32();
        if total <= 0.0 {
            return 0.0;
        }
        (until.saturating_duration_since(now).as_secs_f32() / total).clamp(0.0, 1.0)
    }
}

/// Lasting state attached to a node, drawn until it is cleared.
#[derive(Clone, Copy, Debug)]
struct NodeState {
    animation: SteadyAnimation,
    /// `None` falls back to the active theme's `ThemeColors::marker` -- this
    /// is the persistent "flagged" state `marker_ui` paints, not a one-off
    /// event, so it uses `marker` rather than `alert`.
    color: Option<Color32>,
}

/// Fade of a node's [`NodeContext::marker`](objects::NodeContext::marker):
/// in while `on`, out otherwise, since `since`. Turning it around halfway
/// shifts `since` back so the level carries on from where it was.
#[derive(Clone, Copy, Debug)]
struct MarkerPresence {
    on: bool,
    since: Instant,
}

impl MarkerPresence {
    /// `0.0` (no marker) to `1.0` (fully in) at `now`.
    fn level(&self, now: Instant) -> f32 {
        let progress =
            now.saturating_duration_since(self.since).as_secs_f32() / objects::MARKER_FADE_SECS;
        if self.on {
            progress.min(1.0)
        } else {
            (1.0 - progress).max(0.0)
        }
    }

    /// The fade after switching to `on` at `now`, starting from the level
    /// `previous` had reached.
    fn switch(previous: Option<MarkerPresence>, on: bool, now: Instant) -> Self {
        let level = previous.map_or(0.0, |presence| presence.level(now));
        let head_start = if on { level } else { 1.0 - level } * objects::MARKER_FADE_SECS;
        let since = now
            .checked_sub(std::time::Duration::from_secs_f32(head_start))
            .unwrap_or(now);
        Self { on, since }
    }

    /// Whether the level is still changing at `now`.
    fn fading(&self, now: Instant) -> bool {
        let level = self.level(now);
        if self.on { level < 1.0 } else { level > 0.0 }
    }
}

/// A one-off effect attached to a segment, with the moment it started.
#[derive(Clone, Copy, Debug)]
struct SegmentNotification {
    started: Instant,
    animation: SegmentAnimation,
    /// `None` falls back to the active theme's `ThemeColors::alert`.
    color: Option<Color32>,
}

/// Lasting state attached to a segment, drawn until it is cleared.
#[derive(Clone, Copy, Debug)]
struct SegmentState {
    animation: SteadySegmentAnimation,
    /// `None` falls back to the active theme's `ThemeColors::alert`.
    color: Option<Color32>,
}

/// A borrowed node, obtained from [`Map::node`], that an animation can be
/// attached to.
///
/// Modifiers such as [`NodeHandle::color`] come first; the effect method is the
/// terminal call that writes everything at once. A modifier on its own does
/// nothing, so there is no way to configure a notification that does not exist.
///
/// Effects come in two families, and which one you call decides how it ends:
///
/// - [`pulse`](Self::pulse), [`ripple`](Self::ripple),
///   [`countdown`](Self::countdown), [`scale_in`](Self::scale_in) and
///   [`crosshair`](Self::crosshair) play once from the [`Instant`] you pass and
///   stop on their own.
/// - [`halo`](Self::halo), [`blink`](Self::blink) and [`orbit`](Self::orbit)
///   are lasting state: they run until [`clear`](Self::clear), and keep the app
///   repainting the whole time.
///
/// A node can carry one of each at once; the state is drawn underneath the
/// event.
pub struct NodeHandle<'a> {
    map: &'a mut Map,
    id: usize,
    color: Option<Color32>,
    lasting: Option<std::time::Duration>,
}

impl NodeHandle<'_> {
    /// Overrides the colour of the effect about to be attached.
    ///
    /// Without this, the effect falls back to the active theme's
    /// `ThemeColors::alert` for a one-off event (`pulse`, `ripple`, ...) or
    /// `ThemeColors::marker` for lasting state (`halo`, `blink`, `orbit`) --
    /// whichever terminal method is called after this one.
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Makes the event effect about to be attached (`pulse`, `ripple`, ...)
    /// last `duration` from the moment it is triggered, repeating it every
    /// cycle, instead of playing it once. Its color fades progressively over
    /// that time, so a recent notification stands out from an old one (a
    /// [`NodeTemplate`] gets it already faded in
    /// [`NotificationContext::color`](objects::NotificationContext::color)).
    /// Lasting state (`halo`, `blink`, `orbit`) ignores it: it already runs
    /// until cleared.
    ///
    /// ```
    /// # use egui_map::map::Map;
    /// # use egui_map::map::objects::MapPoint;
    /// # use std::time::{Duration, Instant};
    /// # let mut map = Map::new();
    /// # map.add_points(vec![MapPoint::new(1, [0.0, 0.0])]);
    /// if let Some(node) = map.node(1) {
    ///     node.lasting(Duration::from_secs(240)).pulse(Instant::now());
    /// }
    /// ```
    pub fn lasting(mut self, duration: std::time::Duration) -> Self {
        self.lasting = Some(duration);
        self
    }

    fn notify_with(self, animation: NodeAnimation, at: Instant) {
        self.map.notifications.insert(
            self.id,
            Notification {
                started: at,
                animation,
                color: self.color,
                until: self.lasting.map(|duration| at + duration),
            },
        );
    }

    fn set_state(self, animation: SteadyAnimation) {
        self.map.node_states.insert(
            self.id,
            NodeState {
                animation,
                color: self.color,
            },
        );
    }

    /// Expanding, fading disc. Reads as "one thing happened here".
    pub fn pulse(self, at: Instant) {
        self.notify_with(NodeAnimation::Pulse, at);
    }

    /// Three staggered expanding rings. Reads as "activity is ongoing".
    pub fn ripple(self, at: Instant) {
        self.notify_with(NodeAnimation::Ripple, at);
    }

    /// A ring emptying clockwise. Reads as "how old is this information".
    pub fn countdown(self, at: Instant) {
        self.notify_with(NodeAnimation::CountdownArc, at);
    }

    /// A disc that overshoots and settles. For a node that just appeared.
    pub fn scale_in(self, at: Instant) {
        self.notify_with(NodeAnimation::ScaleIn, at);
    }

    /// Four ticks converging on the node. Reads as "target acquired".
    pub fn crosshair(self, at: Instant) {
        self.notify_with(NodeAnimation::Crosshair, at);
    }

    /// Lasting ring whose opacity breathes. Runs until [`Self::clear`].
    pub fn halo(self) {
        self.set_state(SteadyAnimation::Halo);
    }

    /// Lasting thick ring blinking on and off. Runs until [`Self::clear`].
    pub fn blink(self) {
        self.set_state(SteadyAnimation::Blink);
    }

    /// Lasting dot circling the node. Runs until [`Self::clear`].
    pub fn orbit(self) {
        self.set_state(SteadyAnimation::Orbit);
    }

    /// Removes both the notification and the lasting state of this node.
    pub fn clear(self) {
        self.map.notifications.remove(&self.id);
        self.map.node_states.remove(&self.id);
    }
}

/// A borrowed segment, obtained from [`Map::segment`], that an animation can
/// be attached to.
///
/// Mirrors [`NodeHandle`], with the same modifier-then-terminal shape:
///
/// - [`flash`](Self::flash), [`comet_once`](Self::comet_once) and
///   [`wipe`](Self::wipe) play once from the [`Instant`] you pass and stop on
///   their own.
/// - [`comet`](Self::comet), [`dash`](Self::dash), [`glow_band`](Self::glow_band)
///   and [`chevrons`](Self::chevrons) are lasting state: they run until
///   [`clear`](Self::clear), and keep the app repainting the whole time.
///
/// A segment can carry one of each at once; the state is drawn underneath the
/// event, same as node effects.
pub struct SegmentHandle<'a> {
    map: &'a mut Map,
    id: (usize, usize),
    color: Option<Color32>,
}

impl SegmentHandle<'_> {
    /// Overrides the colour of the effect about to be attached.
    ///
    /// Without this, the effect falls back to the active theme's
    /// `ThemeColors::alert`, for either a one-off notification or lasting
    /// state -- segments don't have a `marker` role like a node's lasting
    /// state does.
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Brief flash that fades back out. The segment analogue of
    /// [`NodeHandle::pulse`] — reads as "something happened on this route".
    pub fn flash(self, at: Instant) {
        self.map.segment_notifications.insert(
            self.id,
            SegmentNotification {
                started: at,
                animation: SegmentAnimation::FlashDecay,
                color: self.color,
            },
        );
    }

    /// Single dot pass from one endpoint to the other, then gone. The
    /// event-driven counterpart to [`Self::comet`] — reads as "one thing
    /// moved along this route just now" rather than "traffic keeps flowing
    /// this way". `direction` picks which endpoint it starts from.
    pub fn comet_once(self, at: Instant, direction: CometDirection) {
        self.map.segment_notifications.insert(
            self.id,
            SegmentNotification {
                started: at,
                animation: SegmentAnimation::Comet(direction),
                color: self.color,
            },
        );
    }

    /// Line drawing itself in from the first endpoint to the second, then
    /// gone — reads as "this route was just established" rather than
    /// "something travelled along it".
    pub fn wipe(self, at: Instant) {
        self.map.segment_notifications.insert(
            self.id,
            SegmentNotification {
                started: at,
                animation: SegmentAnimation::Wipe,
                color: self.color,
            },
        );
    }

    /// Lasting dot travelling along the segment. Runs until
    /// [`Self::clear`].
    pub fn comet(self) {
        self.map.segment_states.insert(
            self.id,
            SegmentState {
                animation: SteadySegmentAnimation::Comet,
                color: self.color,
            },
        );
    }

    /// Lasting dashed line, its pattern sliding along the segment
    /// ("marching ants"). Runs until [`Self::clear`].
    pub fn dash(self) {
        self.map.segment_states.insert(
            self.id,
            SegmentState {
                animation: SteadySegmentAnimation::Dash,
                color: self.color,
            },
        );
    }

    /// Lasting band of brightness travelling the length of the segment and
    /// looping. Reads as "flow", calmer than [`Self::dash`]. Runs until
    /// [`Self::clear`].
    pub fn glow_band(self) {
        self.map.segment_states.insert(
            self.id,
            SegmentState {
                animation: SteadySegmentAnimation::GlowBand,
                color: self.color,
            },
        );
    }

    /// Lasting row of arrow shapes sliding along the segment, pointing the
    /// way. Runs until [`Self::clear`].
    pub fn chevrons(self) {
        self.map.segment_states.insert(
            self.id,
            SegmentState {
                animation: SteadySegmentAnimation::Chevrons,
                color: self.color,
            },
        );
    }

    /// Removes both the notification and the lasting state of this segment.
    pub fn clear(self) {
        self.map.segment_notifications.remove(&self.id);
        self.map.segment_states.remove(&self.id);
    }
}

impl Default for Map {
    /// Creates an empty map; equivalent to [`Map::new`].
    fn default() -> Self {
        Map::new()
    }
}

impl Widget for &mut Map {
    /// Renders the map, handling panning (drag), zooming (mouse wheel) and the
    /// right-click context menu if one was installed.
    fn ui(self, ui: &mut egui::Ui) -> Response {
        let rect = self.calculate_widget_dimensions(ui);

        // we define the initial coordinate as the center of such rectangle
        let reference_dist = rect.distance();
        // `reference.dist` is refreshed here every frame, but `current.dist` --
        // the value the viewport cull actually queries with -- is only derived
        // from it inside `adjust_bounds`, which used to run on a zoom change or
        // a `set_pos` and nothing else. Resizing the window therefore left the
        // cull radius stale until the next zoom, so the node set was culled
        // against the *old* widget size (too few nodes after growing the
        // window, too many after shrinking it). Tracked here so the bounds can
        // be recomputed below, next to the zoom-change branch.
        let resized = reference_dist != self.reference.dist;
        self.reference.dist = reference_dist;

        self.assign_visual_style(ui);

        let canvas = egui::Frame::canvas(ui.style()).inner_margin(Margin::symmetric(3, 5));

        // The frame consumes `total_margin` (inner margin + stroke width +
        // outer margin) around whatever is drawn inside it. `map_area` is the
        // widget's *whole* footprint, so the painter may only claim what is
        // left after that margin. Allocating `map_area.size()` inside the
        // frame made the frame grow to `map_area.size() + 2 * total_margin`
        // and spill past the space the widget was given, which left the
        // visible drawable region truncated on the right/bottom -- so its
        // centre no longer matched the point `set_pos`/`set_pos_from_nodeid`
        // centre on, and nodes were drawn half a margin off.
        let frame_margin = canvas.total_margin().sum();
        let painter_size = (self.map_area.size() - frame_margin).max(Vec2::ZERO);

        let inner_response = canvas.show(ui, |ui| {
            let _span = tracing::info_span!("paint_map").entered();

            if ui.is_rect_visible(self.map_area) {
                let (resp, paint) =
                    ui.allocate_painter(painter_size, egui::Sense::click_and_drag());
                // `paint` is already clipped to the drawable area, but custom
                // templates (and the built-in marker effects) paint through
                // `ui.painter()`, whose clip was still the parent's -- so a
                // node near the edge was drawn over the frame and past the
                // widget. Clip the whole content `Ui` the same way.
                ui.set_clip_rect(paint.clip_rect());
                let vec = resp.drag_delta();
                if vec.length() != 0.0 {
                    let _span = tracing::info_span!("calculating_points_in_visible_area").entered();

                    let coords = RawPoint::from(vec.to_pos2());
                    let new_pos = self.reference.pos - (coords / self.zoom);
                    self.set_pos(new_pos.into());
                }
                // Centre on the rect we actually paint into. Now that the
                // painter is sized to the frame's content area this is exactly
                // `map_area.center()`, but deriving it from `resp.rect` keeps
                // projection, hover hit-testing and the frame in agreement if
                // the frame's margins ever change. Computed *before* the labels
                // below so they can reuse the same map -> screen projection as
                // the nodes/markers instead of being pinned to a screen pixel.
                let rect_midpoint = RawPoint::from(resp.rect.center());
                let min_point = self.current.pos - rect_midpoint;

                if !self.region_labels.is_empty() {
                    // Region labels are the deepest layer: painted first so
                    // every other element (connection lines, nodes,
                    // free-floating `MapLabel`s) draws over them. See
                    // `RegionLabel`'s own doc for how this differs from
                    // `MapLabel`.
                    let theme = self.theme_colors();
                    // A more transparent color than ordinary text, so region
                    // labels read as a backdrop rather than competing with
                    // foreground content.
                    let color = scale_alpha(theme.text, self.settings.region_label_alpha);
                    let zoom = self.zoom;
                    // `Style::region_label_font` configures only the built-in
                    // renderer below; a custom `LabelTemplate` picks its own
                    // font, the same way `NodeTemplate`/`SegmentTemplate`
                    // implementations pick their own fonts freely. It is
                    // mandatory (no `Option`), so there is no fallback to
                    // resolve here: `size` (still scaled *by* zoom instead
                    // of staying screen-constant, unlike
                    // `node_text_size`/`label_text_size`) and `family` come
                    // straight from it. Each read is its own statement so
                    // the immutable borrow of `self` ends before the
                    // `&mut self.region_label_cache` borrow the loop below
                    // needs.
                    let region_label_size = self.settings.style.region_label_font.size * zoom;
                    let region_label_family = self.settings.style.region_label_font.family.clone();
                    let region_label_font = FontId::new(region_label_size, region_label_family);
                    // Every label's projected `position` is checked against this
                    // before any layout/paint work happens. Without it, all of
                    // `self.region_labels` (113 for the whole-galaxy view this
                    // type ships for) were laid out and painted every frame
                    // regardless of whether they were anywhere near the screen --
                    // and at high zoom `region_label_size` grows right along with
                    // it (see that field's own doc), making each one of those
                    // off-screen-but-still-evaluated labels progressively more
                    // expensive too. See `REGION_LABEL_CULL_MARGIN_FACTOR` for why
                    // this uses a margin instead of the label's exact (not yet
                    // known) rendered size.
                    let visible_rect = resp
                        .rect
                        .expand(region_label_size * REGION_LABEL_CULL_MARGIN_FACTOR);
                    let template = self.label_template.clone();
                    if let Some(template) = &template {
                        for label in &self.region_labels {
                            let position: Pos2 =
                                (RawPoint::from(label.center) * zoom - min_point).into();
                            if !visible_rect.contains(position) {
                                continue;
                            }
                            template.label_ui(
                                &paint,
                                LabelContext {
                                    position,
                                    zoom,
                                    label,
                                    size: region_label_size,
                                    color,
                                    theme,
                                },
                            );
                        }
                    } else {
                        // `paint_region_label` needs `&mut self` (it caches into
                        // `self.region_label_cache`), so it can't be called while
                        // `self.region_labels` is still borrowed -- each iteration
                        // extracts what it needs (`position`, an owned `text`) in a
                        // block that ends that borrow before the method call.
                        for i in 0..self.region_labels.len() {
                            let (position, text) = {
                                let label = &self.region_labels[i];
                                let position: Pos2 =
                                    (RawPoint::from(label.center) * zoom - min_point).into();
                                (position, label.text.clone())
                            };
                            if !visible_rect.contains(position) {
                                continue;
                            }
                            self.paint_region_label(
                                &paint,
                                position,
                                text,
                                region_label_font.clone(),
                                color,
                            );
                        }
                    }
                }

                if self.zoom < self.settings.line_visible_zoom {
                    // filling text settings
                    let mut text_settings = TextSettings {
                        // Screen-space size: unlike the map geometry this is
                        // NOT multiplied by the zoom, so the label stays just
                        // as readable however far the map is zoomed out.
                        size: self.settings.label_text_size,
                        anchor: Align2::CENTER_CENTER,
                        family: FontFamily::Proportional,
                        text: String::new(),
                        position: RawPoint::default(),
                        // The active theme's own text color, not egui's
                        // surrounding-UI text color -- so labels stay
                        // legible against a custom `MapTheme`'s palette
                        // instead of silently following the host app's
                        // light/dark mode.
                        text_color: self.theme_colors().text,
                    };
                    for label in &self.labels {
                        text_settings.text.clone_from(&label.text);
                        // `MapLabel::center` is in map coordinates, so project
                        // it exactly like the nodes do (`coords * zoom -
                        // min_point`). Using it verbatim (as this used to)
                        // pinned every label to a fixed screen pixel that
                        // ignored pan and zoom entirely.
                        text_settings.position =
                            RawPoint::from(label.center) * self.zoom - min_point;
                        self.paint_label(&paint, &text_settings);
                    }
                }

                let vec_points = &self.visible_points;
                let hashm = &self.points;

                // Safety net: drop stale notifications even if their node/
                // segment is outside the viewport and never finishes its
                // animation.
                let now = Instant::now();
                self.notifications.retain(|_, n| match n.until {
                    Some(until) => now < until,
                    None => now.duration_since(n.started).as_secs_f32() < 10.0,
                });
                self.segment_notifications
                    .retain(|_, n| now.duration_since(n.started).as_secs_f32() < 10.0);

                for segment in self.paint_map_lines(&paint, &min_point) {
                    self.segment_notifications.remove(&segment);
                }

                self.hovered_node = self.find_hovered_node(&resp, &min_point);

                if let Ok(nodes_to_remove) =
                    self.paint_map_points(vec_points, hashm, &paint, ui, &min_point, &resp)
                {
                    for node in nodes_to_remove {
                        self.notifications.remove(&node);
                    }
                }

                for marker in &self.markers {
                    // A marker can arrive before its node does (the nodes
                    // aren't loaded yet, or never will be): it is kept and
                    // drawn once the node exists.
                    if let Some(point) =
                        self.points.as_ref().and_then(|points| points.get(marker.1))
                    {
                        let adjusted_point = RawPoint::from(point.coords) * self.zoom - min_point;
                        // Plain markers have no color setting of their own to
                        // override, unlike a node's lasting state -- both
                        // fall back to the active theme's `marker` color,
                        // the same persistent "this is flagged" role the
                        // `node_states` branch below uses, distinct from the
                        // one-off `alert` color transient notifications use.
                        let color = self.theme_colors().marker;
                        if let Some(template) = &self.node_template {
                            template.marker_ui(
                                ui,
                                MarkerContext {
                                    position: adjusted_point.into(),
                                    zoom: self.zoom,
                                    kind: self.settings.marker_animation,
                                    node_id: *marker.1,
                                    point,
                                    color,
                                    theme: self.theme_colors(),
                                    animation: self.settings.animation,
                                },
                            );
                        } else {
                            // Frame time, so every marker in this frame shares
                            // one clock instead of each sampling the wall clock
                            // at a slightly different moment.
                            let time = ui.input(|i| i.time) as f32;
                            let effect = self
                                .settings
                                .animation
                                .state(self.settings.marker_animation);
                            effect(ui.painter(), adjusted_point.into(), self.zoom, time, color);
                            // Persistent effects never finish on their own.
                            ui.ctx().request_repaint();
                        }
                    }
                }

                self.paint_sub_components(ui, self.map_area);

                self.capture_mouse_events(ui, &resp);

                if self.zoom != self.previous_zoom || resized {
                    let _span = tracing::info_span!("calculating viewport with zoom").entered();
                    self.adjust_bounds();
                    self.calculate_visible_points();
                    self.previous_zoom = self.zoom;
                }

                if let Some(menu_mon) = &mut self.menu_manager {
                    resp.context_menu(|ui| {
                        menu_mon.ui(ui);
                    });
                }

                #[cfg(feature = "debug_overlay")]
                self.print_debug_info(ui, &resp);
            }
        });
        // `Frame::show` already allocated the frame's outer rect in the parent
        // `Ui` (that is what `inner_response.response.rect` reports), so the
        // widget must not allocate `map_area` a second time -- doing so made
        // it consume twice its own height in the surrounding layout.
        inner_response.response
    }
}

impl Map {
    /// Creates an empty map widget with default [`MapSettings`].
    ///
    /// The widget displays nothing until nodes are loaded with
    /// [`Map::add_hashmap_points`].
    pub fn new() -> Self {
        let settings = MapSettings::default();
        Self {
            zoom: 1.0,
            previous_zoom: 1.0,
            map_area: Rect::NOTHING,
            tree: None,
            points: None,
            labels: Vec::new(),
            region_labels: Vec::new(),
            region_label_cache: HashMap::new(),
            visible_points: Vec::new(),
            current: MapBounds::default(),
            reference: MapBounds::default(),
            settings,
            min_size: (None, None),
            max_size: (None, None),
            dark_mode: false,
            notifications: HashMap::new(),
            node_states: HashMap::new(),
            segment_notifications: HashMap::new(),
            segment_states: HashMap::new(),
            segment_ids: HashSet::new(),
            menu_manager: None,
            node_template: None,
            segment_template: None,
            label_template: None,
            markers: HashMap::new(),
            marker_presence: HashMap::new(),
            hovered_node: None,
            outline_reach: std::cell::Cell::new(0.0),
            segments: None,
            theme: Rc::new(Theme::default()),
        }
    }

    fn calculate_widget_dimensions(&mut self, ui: &mut Ui) -> RawLine {
        let available = ui.available_rect_before_wrap();
        let mut size = available.size();
        if let Some(max_width) = self.max_size.0 {
            size.x = size.x.min(max_width);
        }
        if let Some(max_height) = self.max_size.1 {
            size.y = size.y.min(max_height);
        }
        if let Some(min_width) = self.min_size.0 {
            size.x = size.x.max(min_width);
        }
        if let Some(min_height) = self.min_size.1 {
            size.y = size.y.max(min_height);
        }
        self.map_area = Rect::from_min_size(available.min, size);
        RawLine::new(
            RawPoint::from(self.map_area.left_top()),
            RawPoint::from(self.map_area.right_bottom()),
        )
    }

    fn calculate_visible_points(&mut self) {
        let _span = tracing::info_span!("calculate_visible_points").entered();
        if self.current.dist > 0.0
            && self.current.dist < f32::INFINITY
            && let Some(tree) = &self.tree
        {
            let center = self.current.pos / self.zoom;
            // `current.dist` is the *full* diagonal of the visible area
            // expressed in map units (`reference.dist` is `RawLine::distance()`
            // over the widget rect, divided by the zoom). A circle centred on
            // the viewport only has to reach its corners to cover it, so the
            // radius is the *half* diagonal -- the rect's circumradius.
            // Querying with the full diagonal doubled the radius, i.e. covered
            // 4x the area, and with the circle-over-rectangle slack that fed
            // roughly 6x more nodes to the paint pass than are actually on
            // screen. Halved here rather than at the `reference.dist`
            // assignments so `dist` keeps meaning "diagonal" for the debug
            // overlay and for `adjust_bounds`.
            let radius = (self.current.dist / 2.0).powi(2);
            let point: [f32; 2] = center.into();
            let vis_pos = tree.within(&point, radius, &squared_euclidean).unwrap();
            self.visible_points.clear();
            for point in vis_pos {
                self.visible_points.push(point.1.cast_signed());
            }
        }
    }

    /// Loads the node set and (re)builds the spatial index.
    ///
    /// This replaces any previously loaded points, computes the bounding box of
    /// the whole set, centers the view on its midpoint and refreshes the list
    /// of visible nodes. It must be called at least once before the widget can
    /// display anything.
    ///
    /// The kd-tree built here is what enables viewport culling and
    /// nearest-neighbor hover lookups, so calling this method on every frame is
    /// discouraged; call it only when the node set changes.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_map::map::Map;
    /// use egui_map::map::objects::MapPoint;
    ///
    /// let mut points = Vec::new();
    /// points.push(MapPoint::new(1, [0.0, 0.0]));
    /// points.push(MapPoint::new(2, [10.0, 10.0]));
    ///
    /// let mut map = Map::new();
    /// map.add_points(points);
    ///
    /// // The view is centered on the midpoint of the loaded nodes.
    /// assert_eq!(map.get_pos(), [5.0, 5.0]);
    /// ```
    pub fn add_points(&mut self, points: Vec<MapPoint>) {
        let mut tree = KdTree::<f32, usize, [f32; 2]>::new(2);
        let mut hash_map = HashMap::new();
        let mut min = RawPoint::new(f32::INFINITY, f32::INFINITY);
        let mut max = RawPoint::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
        for entry in points {
            for i in 0..min.components.len() {
                if entry.coords[i] < min.components[i] {
                    min.components[i] = entry.coords[i];
                }
                if entry.coords[i] > max.components[i] {
                    max.components[i] = entry.coords[i];
                }
            }
            let _result = tree.add(entry.coords, entry.get_id());
            hash_map.insert(entry.get_id(), entry);
        }
        // We stablish the max and min coordinates in this map, this wont change until we change the point hash map
        self.reference.min = min;
        self.reference.max = max;
        self.points = Some(hash_map);
        self.tree = Some(tree);
        self.reference.pos = RawLine::new(min, max).midpoint();
        // we create a rect that include every node in the map
        // Stupid fix because rect area could be infinite
        // I need to implement a more elegant fix
        if self.map_area.area() == 0.0 {
            self.reference.dist = 3000.00;
        } else {
            let rect = RawLine::new(
                RawPoint::from(self.map_area.left_top()),
                RawPoint::from(self.map_area.right_bottom()),
            );
            self.reference.dist = rect.distance();
        }
        self.current = self.reference.clone();
        self.calculate_visible_points();
    }

    /// Loads the node set and (re)builds the spatial index.
    ///
    /// This replaces any previously loaded points, computes the bounding box of
    /// the whole set, centers the view on its midpoint and refreshes the list
    /// of visible nodes. It must be called at least once before the widget can
    /// display anything.
    ///
    /// The kd-tree built here is what enables viewport culling and
    /// nearest-neighbor hover lookups, so calling this method on every frame is
    /// discouraged; call it only when the node set changes.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_map::map::Map;
    /// use egui_map::map::objects::MapPoint;
    /// use std::collections::HashMap;
    ///
    /// let mut points = HashMap::new();
    /// points.insert(1, MapPoint::new(1, [0.0, 0.0]));
    /// points.insert(2, MapPoint::new(2, [10.0, 10.0]));
    ///
    /// let mut map = Map::new();
    /// map.add_hashmap_points(points);
    ///
    /// // The view is centered on the midpoint of the loaded nodes.
    /// assert_eq!(map.get_pos(), [5.0, 5.0]);
    /// ```
    //#[deprecated(since="0.2.3", note="please use `add_points` instead")]
    pub fn add_hashmap_points(&mut self, hash_map: HashMap<usize, MapPoint>) {
        let _span = tracing::info_span!("add_hashmap_points").entered();
        let mut min = RawPoint::new(f32::INFINITY, f32::INFINITY);
        let mut max = RawPoint::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
        let mut tree = KdTree::<f32, usize, [f32; 2]>::new(2);

        for entry in hash_map.iter() {
            for i in 0..min.components.len() {
                if entry.1.coords[i] < min.components[i] {
                    min.components[i] = entry.1.coords[i];
                }
                if entry.1.coords[i] > max.components[i] {
                    max.components[i] = entry.1.coords[i];
                }
            }
            let _result = tree.add(entry.1.coords, *entry.0);
        }

        // We stablish the max and min coordinates in this map, this wont change until we change the point hash map
        self.reference.min = min;
        self.reference.max = max;
        self.points = Some(hash_map);
        self.tree = Some(tree);
        self.reference.pos = RawLine::new(min, max).midpoint();
        // we create a rect that include every node in the map
        // Stupid fix because rect area could be infinite
        // I need to implement a more elegant fix
        if self.map_area.area() == 0.0 {
            self.reference.dist = 3000.00;
        } else {
            let rect = RawLine::new(
                RawPoint::from(self.map_area.left_top()),
                RawPoint::from(self.map_area.right_bottom()),
            );
            self.reference.dist = rect.distance();
        }
        self.current = self.reference.clone();
        self.calculate_visible_points();
    }

    /// Centers the view on the node with the given id.
    ///
    /// Returns `true` if the view moved. Returns `false` — leaving the view
    /// untouched — when no points have been loaded yet or when `node_id` is
    /// not among them; that case also emits a `tracing` warning, since a
    /// silently ignored id is otherwise indistinguishable from a node that
    /// was centered but drawn in the wrong place.
    ///
    /// A `false` here usually means the id belongs to a different set than
    /// the one loaded through [`Map::add_hashmap_points`] — for example a
    /// map showing only part of the universe, or ids coming from a different
    /// query than the one that produced the nodes.
    ///
    /// ```
    /// use egui_map::map::Map;
    /// use egui_map::map::objects::MapPoint;
    ///
    /// let mut map = Map::new();
    /// map.add_points(vec![MapPoint::new(1, [10.0, 20.0])]);
    ///
    /// assert!(map.set_pos_from_nodeid(1));
    /// assert_eq!(map.get_pos(), [10.0, 20.0]);
    ///
    /// // Unknown id: the view stays where it was.
    /// assert!(!map.set_pos_from_nodeid(999));
    /// assert_eq!(map.get_pos(), [10.0, 20.0]);
    /// ```
    pub fn set_pos_from_nodeid(&mut self, node_id: usize) -> bool {
        let _span = tracing::info_span!("set_pos_from_nodeid").entered();
        if let Some(hash_map) = &self.points
            && let Some(map_point) = hash_map.get(&node_id)
        {
            self.reference.pos = RawPoint::from(map_point.coords);
            self.adjust_bounds();
            self.calculate_visible_points();
            true
        } else {
            tracing::warn!(
                node_id,
                loaded_nodes = self.points.as_ref().map_or(0, |p| p.len()),
                "set_pos_from_nodeid: unknown node id, the view was left unchanged"
            );
            false
        }
    }

    /// Centers the view on the given map coordinates.
    pub fn set_pos(&mut self, position: [f32; 2]) {
        let _span = tracing::info_span!("set_pos").entered();
        let point = RawPoint::from(position);
        self.reference.pos = point;
        self.adjust_bounds();
        self.calculate_visible_points();
    }

    /// Returns the map coordinates the view is currently centered on.
    pub fn get_pos(&self) -> [f32; 2] {
        let _span = tracing::info_span!("get_pos").entered();
        self.reference.pos.into()
    }

    /// Replaces the set of free-floating text labels drawn on the map.
    ///
    /// Labels are only rendered while the zoom level is below
    /// [`MapSettings::line_visible_zoom`].
    pub fn add_labels(&mut self, labels: Vec<MapLabel>) {
        let _span = tracing::info_span!("add_labels").entered();
        self.labels = labels;
    }

    /// Replaces the set of region labels drawn on the map.
    ///
    /// Unlike [`Map::add_labels`], a [`RegionLabel`] is not a fixed
    /// on-screen annotation -- see its own doc for how it differs (font
    /// size that scales with zoom, background z-order, a more transparent
    /// color) and [`Map::set_label_template`] for customizing how it is
    /// drawn. Also clears the built-in renderer's layout cache, so call this
    /// when the label *set* changes rather than every frame.
    pub fn add_region_labels(&mut self, labels: Vec<RegionLabel>) {
        let _span = tracing::info_span!("add_region_labels").entered();
        self.region_labels = labels;
        self.region_label_cache.clear();
    }

    /// Replaces the set of connection lines between nodes.
    ///
    /// Lines are keyed by a connection id that the endpoint nodes must
    /// reference through [`MapPoint::connections`] — push each line's key into
    /// the `connections` of the nodes it joins. The segments are stored in an
    /// R-tree keyed by bounding box: a line is drawn while its bounding box
    /// intersects the viewport and the zoom level is above
    /// [`MapSettings::line_visible_zoom`].
    ///
    /// See the [module-level example](self#connecting-nodes-with-lines) for
    /// the complete wiring.
    pub fn add_lines(&mut self, segments: Vec<MapSegment>) {
        let _span = tracing::info_span!("add_lines").entered();
        // Intern the keys as Rc<str> and build the broad-phase spatial index
        // over the line bounding boxes, so viewport culling and hit-testing
        // discard whole regions without touching every segment.

        self.segment_ids = segments.iter().map(|s| s.id).collect();
        self.segments = Some(rstar::RTree::bulk_load(segments));
    }

    /// Replaces the set of connection lines between nodes, from a map keyed
    /// by the same `(usize, usize)` id used in [`MapSegment::id`] and
    /// referenced by [`MapPoint::connections`].
    ///
    /// Equivalent to [`add_lines`](Self::add_lines) but avoids callers having
    /// to collect their segments into a `Vec` first when they already have
    /// them keyed in a `HashMap` (e.g. straight from an adapter that mirrors
    /// them 1:1 by id, with no intermediate ordering to preserve).
    pub fn add_hashmap_lines(&mut self, segments: HashMap<(usize, usize), MapSegment>) {
        let _span = tracing::info_span!("add_hashmap_lines").entered();
        let segments: Vec<MapSegment> = segments.into_values().collect();
        self.segment_ids = segments.iter().map(|s| s.id).collect();
        self.segments = Some(rstar::RTree::bulk_load(segments));
    }

    fn adjust_bounds(&mut self) {
        let _span = tracing::info_span!("adjust_bounds").entered();
        self.current.max = self.reference.max * self.zoom;
        self.current.min = self.reference.min * self.zoom;
        self.current.dist = self.reference.dist / self.zoom;
        self.current.pos = self.reference.pos * self.zoom;
    }

    fn capture_mouse_events(&mut self, ui: &Ui, _resp: &Response) {
        let _span = tracing::info_span!("capture_mouse_events").entered();
        // capture MouseWheel Event for Zoom control change
        if ui.rect_contains_pointer(self.map_area) {
            ui.input(|x| {
                let _span = tracing::info_span!("capture_mouse_events_input").entered();

                if !x.events.is_empty() {
                    for event in &x.events {
                        match event {
                            Event::MouseWheel {
                                unit: _,
                                delta,
                                modifiers,
                                phase: _,
                            } => {
                                #[cfg(target_os = "macos")]
                                let zoom_modifier = if modifiers.mac_cmd {
                                    delta.y / 80.00
                                } else {
                                    delta.y / 400.00
                                };

                                #[cfg(not(target_os = "macos"))]
                                let zoom_modifier = if modifiers.ctrl {
                                    delta.y / 8.00
                                } else {
                                    delta.y / 40.00
                                };

                                let mut pre_zoom = self.zoom + zoom_modifier;
                                if pre_zoom > self.settings.max_zoom {
                                    pre_zoom = self.settings.max_zoom;
                                }
                                if pre_zoom < self.settings.min_zoom {
                                    pre_zoom = self.settings.min_zoom;
                                }
                                self.zoom = pre_zoom;
                            }
                            _ => {
                                continue;
                            }
                        };
                    }
                }
            });
        }
    }

    /// Sets the zoom factor.
    ///
    /// Values outside the [`MapSettings::min_zoom`]..=[`MapSettings::max_zoom`]
    /// range are ignored.
    pub fn set_zoom(&mut self, value: f32) {
        if value >= self.settings.min_zoom && value <= self.settings.max_zoom {
            self.zoom = value;
        }
    }

    /// Returns the current zoom factor.
    pub fn get_zoom(&mut self) -> f32 {
        self.zoom
    }

    fn assign_visual_style(&mut self, ui_obj: &mut Ui) {
        let dark_mode = ui_obj.visuals().dark_mode;

        if self.dark_mode != dark_mode {
            let _span = tracing::info_span!("asign_visual_style").entered();

            self.dark_mode = dark_mode;
        }
    }

    /// The [`ColorMode`] the widget is currently painting with -- `Dark`
    /// when `dark_mode` is `true`, `Light` otherwise.
    fn color_mode(&self) -> ColorMode {
        ColorMode::from_dark_mode(self.dark_mode)
    }

    /// Resolves the color palette the widget paints with right now: the
    /// active [`MapTheme`]'s colors for the current [`ColorMode`]. This is
    /// the single, canonical source for every color the widget paints --
    /// unlike `settings.style`, it can never drift out of sync with the
    /// installed theme because nothing caches it.
    fn theme_colors(&self) -> ThemeColors {
        self.theme.colors(self.color_mode())
    }

    /// Floating debug read-out, compiled in only under the `debug_overlay`
    /// feature.
    ///
    /// Deliberately unobtrusive: it renders as a collapsed `dbg` toggle in the
    /// map's top-left corner with no background of its own, so it costs a few
    /// dim pixels until a developer clicks it open. egui remembers the
    /// open/closed state per widget instance, so it stays open across frames
    /// once expanded.
    #[cfg(feature = "debug_overlay")]
    fn print_debug_info(&mut self, ui: &mut Ui, resp: &Response) {
        let _span = tracing::info_span!("printing debug data").entered();

        let p = |v: f32| format!("{v:.2}");
        let mut rows: Vec<(String, Color32)> = vec![
            (
                format!(
                    "MIN {}, {}",
                    p(self.current.min.components[0]),
                    p(self.current.min.components[1])
                ),
                Color32::LIGHT_GREEN,
            ),
            (
                format!(
                    "MAX {}, {}",
                    p(self.current.max.components[0]),
                    p(self.current.max.components[1])
                ),
                Color32::LIGHT_GREEN,
            ),
            (
                format!(
                    "CUR {}, {}",
                    p(self.current.pos.components[0]),
                    p(self.current.pos.components[1])
                ),
                Color32::LIGHT_GREEN,
            ),
            (
                format!("DST {}", p(self.current.dist)),
                Color32::LIGHT_GREEN,
            ),
            (format!("ZOM {}", self.zoom), Color32::GREEN),
            (
                format!(
                    "REC {}, {} .. {}, {}",
                    p(self.map_area.left_top().x),
                    p(self.map_area.left_top().y),
                    p(self.map_area.right_bottom().x),
                    p(self.map_area.right_bottom().y)
                ),
                Color32::LIGHT_GREEN,
            ),
        ];
        if let Some(points) = &self.points {
            rows.push((format!("NUM {}", points.len()), Color32::LIGHT_GREEN));
        }
        if !self.visible_points.is_empty() {
            rows.push((
                format!("VIS {}", self.visible_points.len()),
                Color32::LIGHT_GREEN,
            ));
        }
        if let Some(pointer_pos) = resp.hover_pos() {
            rows.push((
                format!("HVR {}, {}", p(pointer_pos.x), p(pointer_pos.y)),
                Color32::LIGHT_BLUE,
            ));
        }
        let drag = resp.drag_delta();
        if drag.length() != 0.0 {
            rows.push((format!("DRG {}, {}", p(drag.x), p(drag.y)), Color32::GOLD));
        }

        // Drawn into a *detached* child `Ui` in the map's own layer.
        //
        // `new_child` alone does not call `advance_cursor_after_rect`, so the
        // overlay never contributes to the parent's `min_rect`. That matters:
        // the canvas `Frame` sizes itself from its content's `min_rect`, and
        // letting this grow it would push the frame past the space the widget
        // was given -- exactly the overflow that used to knock the map
        // off-centre. Being laid out after the map's painter also means the
        // toggle wins pointer input over the pan/zoom surface underneath.
        let overlay_rect = Rect::from_min_max(
            self.map_area.left_top() + Vec2::new(6.0, 6.0),
            self.map_area.right_bottom(),
        );
        let mut overlay_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(overlay_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        // No frame and no header background: the map shows straight through,
        // so this costs a few dim pixels until someone opens it.
        CollapsingHeader::new(RichText::new("dbg").monospace().small().weak())
            .id_salt("egui_map_debug_overlay")
            .default_open(false)
            .show_background(false)
            .show(&mut overlay_ui, |ui| {
                for (text, color) in rows {
                    ui.label(RichText::new(text).monospace().small().color(color));
                }
            });
    }

    fn paint_sub_components(&mut self, ui_obj: &mut Ui, rect: Rect) {
        let _span = tracing::info_span!("map_ui_paint_sub_components").entered();
        let zoom_slider = egui::Slider::new(
            &mut self.zoom,
            self.settings.min_zoom..=self.settings.max_zoom,
        )
        .show_value(false)
        .orientation(SliderOrientation::Vertical);
        let mut pos1 = rect.right_top();
        let mut pos2 = rect.right_top();
        pos1.x -= 80.0;
        pos1.y += 120.0;
        pos2.x -= 60.0;
        pos2.y += 240.0;

        let sub_rect = egui::Rect::from_two_pos(pos1, pos2);
        let ui_builder = egui::UiBuilder::new().clone().max_rect(sub_rect);
        ui_obj.scope_builder(ui_builder, |ui_obj| {
            ui_obj.add(zoom_slider);
        });
    }

    fn paint_map_points(
        &self,
        vec_points: &Vec<isize>,
        hashm: &Option<HashMap<usize, MapPoint>>,
        paint: &Painter,
        ui_obj: &mut Ui,
        min_point: &RawPoint,
        resp: &Response,
    ) -> Result<Vec<usize>, ()> {
        // One span for the whole node pass, rather than one per node inside
        // the loop below. A per-node span made the *instrumentation* the
        // dominant cost: measured against this widget, a field-less Tracy zone
        // came to ~10.6 us per node, so at ~107 visible nodes it burned ~1.1 ms
        // of every frame while measuring nothing that a single zone around the
        // loop does not already report.
        let _span = tracing::info_span!("paint_map_points").entered();
        let mut nearest_id = None;
        let mut nodes_to_remove = Vec::new();
        let mut shape_vec = vec![];

        if hashm.is_none() {
            return Err(());
        }
        if vec_points.is_empty() {
            return Err(());
        }
        // detecting the nearest hover node
        if self.settings.node_text_visibility == VisibilitySetting::Hover
            && resp.hovered()
            && let Some(point) = resp.hover_pos()
        {
            let raw_point = RawPoint::from(point);
            let hovered_map_point = (*min_point + raw_point) / self.zoom;
            if let Ok(nearest_node) = self.tree.as_ref().unwrap().nearest(
                &hovered_map_point.components,
                1,
                &squared_euclidean,
            ) {
                nearest_id = Some(nearest_node.first().unwrap().1);
            }
        }
        // Resolved once for the whole batch of nodes below, instead of once
        // per role per node (`selected`, `marker`, `alert`, `node`, plus a
        // whole `theme: self.theme_colors()` copy for up to four separate
        // context structs) -- `Theme::colors` is cheap (a `const fn` over
        // plain `Color32` literals, no allocation), but there is no reason
        // to repeat it dozens of times a frame when the active theme and
        // color mode can't change mid-frame. Same reasoning for
        // `background_color`: the surrounding UI's visuals don't change
        // node to node either.
        let theme = self.theme_colors();
        let background_color = ui_obj.ctx().theme().default_visuals().extreme_bg_color;
        // One clock for every node's marker fade this frame.
        let now = Instant::now();

        // filling text settings
        let mut text_settings = TextSettings {
            // Screen-space size: unlike the map geometry this is NOT
            // multiplied by the zoom, so a node name stays just as readable
            // when the map is zoomed all the way out.
            size: self.settings.node_text_size,
            anchor: Align2::LEFT_BOTTOM,
            family: FontFamily::Proportional,
            text: String::new(),
            position: RawPoint::default(),
            // Same reasoning as the free-floating label above: the active
            // theme's text color, so node names honor a custom `MapTheme`.
            text_color: theme.text,
        };

        // Drawing Points
        for temp_point in vec_points {
            let parsed_point = temp_point.cast_unsigned();
            if let Some(system) = hashm.as_ref().unwrap().get(&parsed_point) {
                let viewport_point = RawPoint::from(system.coords) * self.zoom - min_point;
                if let Some(node_template) = &self.node_template {
                    if nearest_id.unwrap_or(&0usize) == &system.get_id() {
                        node_template.selection_ui(
                            ui_obj,
                            SelectionContext {
                                position: viewport_point.into(),
                                zoom: self.zoom,
                                point: system,
                                color: theme.selected,
                                theme,
                            },
                        );
                    }
                } else if self.zoom > self.settings.label_visible_zoom
                    && self.settings.node_text_visibility == VisibilitySetting::Always
                    || (self.settings.node_text_visibility == VisibilitySetting::Hover
                        && nearest_id.unwrap_or(&0usize) == &system.get_id())
                {
                    let mut viewport_text = viewport_point;
                    viewport_text.components[0] += 3.0 * self.zoom;
                    viewport_text.components[1] -= 3.0 * self.zoom;
                    text_settings.position = viewport_text;
                    text_settings.text = system.get_name();
                    self.paint_label(paint, &text_settings);
                }

                let system_id = system.get_id();

                // Persistent node state is drawn first so a notification --
                // the *event* -- sits on top of the *state*.
                if let Some(state) = self.node_states.get(&system_id) {
                    let color = state.color.unwrap_or(theme.marker);
                    if let Some(template) = &self.node_template {
                        // There is no dedicated template hook for node state:
                        // `marker_ui` is the persistent-visual one, so state and
                        // markers share it -- `kind` is enough for a template to
                        // pick the right built-in effect either way, but the
                        // hook still cannot tell *which* of the two call sites
                        // (state vs. a `Map::update_marker` marker) it is.
                        template.marker_ui(
                            ui_obj,
                            MarkerContext {
                                position: viewport_point.into(),
                                zoom: self.zoom,
                                kind: state.animation,
                                node_id: system_id,
                                point: system,
                                color,
                                theme,
                                animation: self.settings.animation,
                            },
                        );
                    } else {
                        let effect = self.settings.animation.state(state.animation);
                        // Frame time, so every element animated this frame
                        // shares one clock instead of sampling its own.
                        let time = ui_obj.input(|i| i.time) as f32;
                        effect(paint, viewport_point.into(), self.zoom, time, color);
                    }
                    // Persistent effects never finish on their own.
                    ui_obj.ctx().request_repaint();
                }

                if let Some(notification) = self.notifications.get(&system_id) {
                    let color = notification
                        .color
                        .unwrap_or(theme.alert)
                        .gamma_multiply(notification.remaining(now));
                    if notification.expired(now) {
                        nodes_to_remove.push(system_id);
                    } else if let Some(template) = &self.node_template {
                        let running = template.notification_ui(
                            ui_obj,
                            NotificationContext {
                                position: viewport_point.into(),
                                zoom: self.zoom,
                                initial_time: notification.started,
                                color,
                                kind: notification.animation,
                                until: notification.until,
                                node_id: system_id,
                                point: system,
                                theme,
                                animation: self.settings.animation,
                            },
                        );
                        if !running {
                            nodes_to_remove.push(system_id);
                        }
                    } else {
                        let effect = self.settings.animation.event(notification.animation);
                        // A lasting notification restarts the effect every
                        // cycle until `until`; a plain one plays it once.
                        let started = if notification.until.is_some() {
                            animation::cycle_start(
                                notification.started,
                                now,
                                self.settings
                                    .animation
                                    .event_duration(notification.animation),
                            )
                        } else {
                            notification.started
                        };
                        let running =
                            effect(paint, viewport_point.into(), self.zoom, started, color);
                        if running || notification.until.is_some() {
                            ui_obj.ctx().request_repaint();
                        } else {
                            nodes_to_remove.push(system_id);
                        }
                    }
                }
                // The color requested for this node: its own override if it
                // has one, otherwise the active theme's node color -- the
                // single fallback both the built-in circle and a
                // `NodeTemplate` (via `NodeContext::color`) paint with. The
                // active theme's full palette is handed over separately
                // (`NodeContext::theme`) so a template can tell the
                // two apart.
                let node_color = system.color.unwrap_or(theme.node);
                if let Some(node_template) = &self.node_template {
                    let (marker, marker_fading) = self.marker_level(system_id, now);
                    if marker_fading {
                        ui_obj.ctx().request_repaint();
                    }
                    let node_context = NodeContext {
                        position: viewport_point.into(),
                        zoom: self.zoom,
                        point: system,
                        color: node_color,
                        background_color,
                        theme,
                        marker,
                        animation: self.settings.animation,
                    };
                    // Learn how far this template's nodes reach, in map
                    // units, for `find_hovered_node`'s search radius.
                    let reach = node_template
                        .outline(node_context.hit())
                        .extent_from(node_context.position)
                        / self.zoom;
                    if reach > self.outline_reach.get() {
                        self.outline_reach.set(reach);
                    }
                    node_template.node_ui(ui_obj, node_context);
                } else {
                    shape_vec.push(Shape::circle_filled(
                        viewport_point.into(),
                        4.00 * self.zoom,
                        node_color,
                    ));
                }
            }
        }
        paint.extend(shape_vec);

        Ok(nodes_to_remove)
    }

    /// Draws the connection lines, plus any segment effects, returning the ids
    /// of segment notifications that finished this frame so the caller can
    /// drop them (same pattern as [`Map::paint_map_points`]'s return value).
    fn paint_map_lines(&self, painter: &Painter, min_point: &RawPoint) -> Vec<(usize, usize)> {
        let _span = tracing::info_span!("paint_map_lines").entered();
        let mut segments_to_remove = Vec::new();

        if self.zoom <= self.settings.line_visible_zoom {
            return segments_to_remove;
        }
        let Some(segments) = &self.segments else {
            return segments_to_remove;
        };

        // How far into its own zoom-based fade-in the *default* line stroke
        // has ramped: `0.0` right at `line_visible_zoom`, linearly up to
        // `1.0` once the zoom is 0.80 units past it, then staying there.
        // Segment effects reuse this below (`effect_fade`) so they fade in
        // step with the line they sit on instead of ignoring the zoom
        // entirely.
        let line_fade = ((self.zoom - self.settings.line_visible_zoom) / 0.80).clamp(0.0, 1.0);

        // `style.line_width == None` only turns off the *default* stroke -- a
        // `SegmentTemplate` or a segment effect installed through
        // `Map::segment` still needs to run, e.g. for a consumer who draws
        // lines entirely on their own and only wants the built-in effects.
        let line_width = self.settings.style.line_width;

        // Broad-phase: query the segment R-tree with the viewport AABB (in
        // map coordinates), padded by the stroke width -- when there is one
        // -- so lines at the very edge are not clipped prematurely.
        let center = self.current.pos / self.zoom;
        let padding = line_width.unwrap_or(0.0) / self.zoom;
        let half = RawPoint::new(
            self.map_area.width() / 2.0 / self.zoom + padding,
            self.map_area.height() / 2.0 / self.zoom + padding,
        );
        let query = rstar::AABB::from_corners((center - half).into(), (center + half).into());

        // Segment effects should always read as at least as visible as the
        // line they animate, not fade out in lockstep with it and risk
        // disappearing into it -- so their alpha tracks `line_fade` with a
        // fixed head start rather than mirroring it exactly.
        let effect_fade = (line_fade + SEGMENT_EFFECT_ALPHA_BOOST).min(1.0);

        for segment in segments.locate_in_envelope_intersecting(query) {
            let raw_line = segment.raw_line();
            let pos_a: Pos2 = (raw_line.points[0] * self.zoom - min_point).into();
            let pos_b: Pos2 = (raw_line.points[1] * self.zoom - min_point).into();

            // The default stroke is painted right away, not batched up and
            // flushed once after the whole loop -- an opaque line painted
            // *after* its own effect would completely cover it. That's fine
            // for nodes (their effects extend beyond the node's own small
            // circle, so the base shape painted last only covers the
            // center) but not for segments, where the effect runs along the
            // exact same path as the line underneath it.
            // The color resolved per segment: its own override if it has
            // one, otherwise the active theme's segment color -- the same
            // value handed to a `SegmentTemplate` as `SegmentContext::color`,
            // and the one the default stroke paints with, so an override
            // actually shows up.
            let segment_color = scale_alpha(
                segment.color.unwrap_or(self.theme_colors().segment),
                line_fade,
            );
            if let Some(template) = &self.segment_template {
                template.segment_ui(
                    painter,
                    SegmentContext {
                        pos_a,
                        pos_b,
                        zoom: self.zoom,
                        segment,
                        color: segment_color,
                        theme: self.theme_colors(),
                    },
                );
            } else if let Some(width) = line_width {
                painter.add(Shape::line_segment(
                    [pos_a, pos_b],
                    Stroke::new(width, segment_color),
                ));
            }

            // Persistent segment state is drawn first so a notification --
            // the *event* -- sits on top of the *state*, same ordering as
            // node effects.
            if let Some(state) = self.segment_states.get(&segment.id) {
                let color = scale_alpha(
                    state.color.unwrap_or(self.theme_colors().alert),
                    effect_fade,
                );
                let time = painter.ctx().input(|i| i.time) as f32;
                if let Some(template) = &self.segment_template {
                    template.segment_state_ui(
                        painter,
                        SegmentStateContext {
                            pos_a,
                            pos_b,
                            zoom: self.zoom,
                            segment,
                            time,
                            color,
                            theme: self.theme_colors(),
                            kind: state.animation,
                        },
                    );
                } else {
                    let effect = match state.animation {
                        SteadySegmentAnimation::Comet => Animation::comet,
                        SteadySegmentAnimation::Dash => Animation::dash,
                        SteadySegmentAnimation::GlowBand => Animation::glow_band,
                        SteadySegmentAnimation::Chevrons => Animation::chevrons,
                    };
                    effect(painter, pos_a, pos_b, self.zoom, time, color);
                }
                // Persistent effects never finish on their own.
                painter.ctx().request_repaint();
            }

            if let Some(notification) = self.segment_notifications.get(&segment.id) {
                let color = scale_alpha(
                    notification.color.unwrap_or(self.theme_colors().alert),
                    effect_fade,
                );
                let still_playing = if let Some(template) = &self.segment_template {
                    template.segment_notification_ui(
                        painter,
                        SegmentNotificationContext {
                            pos_a,
                            pos_b,
                            zoom: self.zoom,
                            segment,
                            initial_time: notification.started,
                            color,
                            theme: self.theme_colors(),
                            kind: notification.animation,
                        },
                    )
                } else {
                    match notification.animation {
                        SegmentAnimation::FlashDecay => Animation::flash_decay(
                            painter,
                            pos_a,
                            pos_b,
                            self.zoom,
                            notification.started,
                            color,
                        ),
                        SegmentAnimation::Comet(direction) => Animation::comet_once(
                            painter,
                            pos_a,
                            pos_b,
                            self.zoom,
                            notification.started,
                            color,
                            direction,
                        ),
                        SegmentAnimation::Wipe => Animation::wipe(
                            painter,
                            pos_a,
                            pos_b,
                            self.zoom,
                            notification.started,
                            color,
                        ),
                    }
                };
                if still_playing {
                    painter.ctx().request_repaint();
                } else {
                    segments_to_remove.push(segment.id);
                }
            }
        }
        segments_to_remove
    }

    fn paint_label(&self, paint: &Painter, text_settings: &TextSettings) {
        let _span = tracing::info_span!("paint_label").entered();
        paint.text(
            text_settings.position.into(),
            text_settings.anchor,
            text_settings.text.clone(),
            FontId::new(text_settings.size, text_settings.family.clone()),
            text_settings.text_color,
        );
    }

    /// Paints one [`RegionLabel`] with the built-in renderer.
    ///
    /// Lays the text out once per distinct `(text, rounded size, family)`
    /// triple and caches the resulting `Arc<Galley>` in
    /// `self.region_label_cache`, then applies `color` fresh every call
    /// through [`Painter::galley_with_override_text_color`] -- so a cache
    /// hit skips `fonts_mut` entirely, and a theme or alpha change (which
    /// only changes `color`) never invalidates the cache. Rounding `size`
    /// to the nearest pixel before hashing keeps the cache useful while the
    /// map sits at a steady zoom, at the cost of relaying out on every zoom
    /// step that crosses a pixel boundary.
    ///
    /// Centers the text on `position` the same way [`Painter::text`] does
    /// internally (`anchor.anchor_size(pos, galley.size())`), since a
    /// pre-laid-out galley is painted with [`Painter::galley_with_override_text_color`]
    /// rather than `Painter::text` itself.
    ///
    /// Takes `&mut self` (it mutates `self.region_label_cache`), so the
    /// caller must not still be borrowing `self.region_labels` when this is
    /// called -- see the call site above. `text` is taken by value rather
    /// than `&str`: the cache key needs an owned `String` anyway, so the
    /// caller's clone becomes that key directly instead of being cloned a
    /// second time here. `font` bundles what used to be two separate
    /// `size`/`family` parameters -- the caller still needs the raw
    /// `size: f32` on its own (for `LabelContext::size` in the
    /// `LabelTemplate` branch), so this doesn't remove any state, it just
    /// packages what this function receives as the `FontId` it already
    /// conceptually is.
    fn paint_region_label(
        &mut self,
        paint: &Painter,
        position: Pos2,
        text: String,
        font: FontId,
        color: Color32,
    ) {
        let _span = tracing::info_span!("paint_region_label").entered();
        // `font.size` is captured before `font.family` is moved into `key`
        // below, since the cache key rounds the size for hashing but the
        // actual layout call (on a cache miss) still needs the precise,
        // unrounded value.
        let size = font.size;
        let key = (text, size.round() as i32, font.family);
        let galley = match self.region_label_cache.get(&key) {
            Some(galley) => galley.clone(),
            None => {
                let galley =
                    paint.layout_no_wrap(key.0.clone(), FontId::new(size, key.2.clone()), color);
                self.region_label_cache.insert(key, galley.clone());
                galley
            }
        };
        let rect = Align2::CENTER_CENTER.anchor_size(position, galley.size());
        paint.galley_with_override_text_color(rect.min, galley, color);
    }

    /// Triggers a pulsing notification on the node `id_node`.
    ///
    /// # Deprecated
    ///
    /// This only ever played one of the available effects. Use [`Map::node`]
    /// and pick the effect you want:
    ///
    /// ```
    /// # use egui_map::map::Map;
    /// # use egui_map::map::objects::MapPoint;
    /// # use std::time::Instant;
    /// # let mut map = Map::new();
    /// # map.add_points(vec![MapPoint::new(1, [0.0, 0.0])]);
    /// # let time = Instant::now();
    /// if let Some(node) = map.node(1) {
    ///     node.pulse(time);
    /// }
    /// ```
    ///
    /// Note the one behavioural difference: `notify` accepts an id that was
    /// never loaded (the notification simply never draws), while [`Map::node`]
    /// returns `None` for it.
    #[deprecated(
        since = "0.4.0",
        note = "use `map.node(id)` and pick an effect, e.g. `if let Some(n) = map.node(id) { n.pulse(time) }`"
    )]
    pub fn notify(&mut self, id_node: usize, time: Instant) {
        let _span = tracing::info_span!("notify").entered();
        self.notifications.insert(
            id_node,
            Notification {
                started: time,
                animation: NodeAnimation::Pulse,
                color: None,
                until: None,
            },
        );
    }

    /// Borrows the node `id` so an animation can be attached to it.
    ///
    /// Returns `None` when `id` was never loaded through
    /// [`Map::add_points`] / [`Map::add_hashmap_points`], so a stale or
    /// mistyped id is a compile-time-visible case rather than a silent no-op.
    ///
    /// The handle carries optional configuration that must be set *before* the
    /// effect, which is the terminal call:
    ///
    /// ```
    /// # use egui_map::map::Map;
    /// # use egui_map::map::objects::MapPoint;
    /// # use std::time::Instant;
    /// # let mut map = Map::new();
    /// # map.add_points(vec![MapPoint::new(1, [0.0, 0.0])]);
    /// # let time = Instant::now();
    /// // a one-off event
    /// if let Some(node) = map.node(1) {
    ///     node.color(egui::Color32::RED).ripple(time);
    /// }
    ///
    /// // lasting state, until cleared
    /// if let Some(node) = map.node(1) {
    ///     node.halo();
    /// }
    ///
    /// assert!(map.node(999).is_none());
    /// ```
    pub fn node(&mut self, id: usize) -> Option<NodeHandle<'_>> {
        if !self
            .points
            .as_ref()
            .is_some_and(|points| points.contains_key(&id))
        {
            return None;
        }
        Some(NodeHandle {
            map: self,
            id,
            color: None,
            lasting: None,
        })
    }

    /// Borrows the segment `id` so an animation can be attached to it.
    ///
    /// Returns `None` when `id` was never loaded through [`Map::add_lines`] /
    /// [`Map::add_hashmap_lines`], mirroring [`Map::node`]. Use
    /// [`Map::line_at`] to find the id of the segment under a point first,
    /// e.g. to flash the route the mouse is hovering.
    ///
    /// ```
    /// # use egui_map::map::Map;
    /// # use egui_map::map::objects::MapSegment;
    /// # use std::time::Instant;
    /// # let mut map = Map::new();
    /// # map.add_lines(vec![MapSegment::new((1, 2), [0.0, 0.0], [10.0, 0.0])]);
    /// # let time = Instant::now();
    /// // a one-off event
    /// if let Some(segment) = map.segment((1, 2)) {
    ///     segment.color(egui::Color32::RED).flash(time);
    /// }
    ///
    /// // lasting state, until cleared
    /// if let Some(segment) = map.segment((1, 2)) {
    ///     segment.comet();
    /// }
    ///
    /// assert!(map.segment((404, 404)).is_none());
    /// ```
    pub fn segment(&mut self, id: (usize, usize)) -> Option<SegmentHandle<'_>> {
        if !self.segment_ids.contains(&id) {
            return None;
        }
        Some(SegmentHandle {
            map: self,
            id,
            color: None,
        })
    }

    /// Returns the id of the line closest to `point`, in map coordinates,
    /// when it lies within `tolerance` map units of the segment.
    ///
    /// Broad-phase candidates are taken from the segment R-tree built by
    /// [`Map::add_lines`]; the exact point-to-segment distance is then
    /// computed against the line geometry and the closest match wins. Returns
    /// `None` when no lines are loaded or every segment is farther than
    /// `tolerance`. A negative `tolerance` behaves like `0.0`.
    ///
    /// To hit-test a mouse click, convert the screen position to map
    /// coordinates first (`map = (screen + origin) / zoom`, see the
    /// [coordinate model](self#coordinate-model)) and pick a tolerance scaled
    /// by `1.0 / zoom` so it stays constant in screen pixels.
    pub fn line_at(&self, point: [f32; 2], tolerance: f32) -> Option<(usize, usize)> {
        let _span = tracing::info_span!("line_at").entered();
        let segments = self.segments.as_ref()?;
        let tolerance = tolerance.max(0.0);

        let center = RawPoint::from(point);
        let padding = RawPoint::new(tolerance, tolerance);
        let query = rstar::AABB::from_corners((center - padding).into(), (center + padding).into());

        let mut closest: Option<(f32, (usize, usize))> = None;
        for segment in segments.locate_in_envelope_intersecting(query) {
            let distance = segment.raw_line().distance_to_point(center);
            if distance <= tolerance && closest.as_ref().is_none_or(|(best, _)| distance < *best) {
                closest = Some((distance, segment.id));
            }
        }
        closest.map(|(_, id)| id)
    }

    /// Installs a right-click context menu whose contents are built by the
    /// given [`ContextMenuManager`] implementation.
    pub fn set_context_manager(&mut self, manager: Rc<dyn ContextMenuManager>) {
        self.menu_manager = Some(manager);
    }

    /// Replaces the built-in node rendering with a custom [`NodeTemplate`]
    /// implementation.
    ///
    /// The template takes over the drawing of nodes, selection highlights,
    /// notification animations and markers — including the node name labels,
    /// which the widget no longer draws once a template is installed. See the
    /// [`NodeTemplate`] examples for custom shapes and animations.
    pub fn set_node_template(&mut self, template: Rc<dyn NodeTemplate>) {
        self.node_template = Some(template);
        self.outline_reach.set(0.0);
    }

    /// Replaces the built-in segment rendering with a custom
    /// [`SegmentTemplate`] implementation.
    ///
    /// The template takes over the drawing of segments and their effects. See
    /// the [`SegmentTemplate`] examples for a custom line style and animation.
    pub fn set_segment_template(&mut self, template: Rc<dyn SegmentTemplate>) {
        self.segment_template = Some(template);
    }

    /// Replaces the built-in [`RegionLabel`] rendering with a custom
    /// [`LabelTemplate`] implementation.
    ///
    /// The template takes over drawing every region label installed with
    /// [`Map::add_region_labels`]. See the [`LabelTemplate`] example for a
    /// custom look.
    pub fn set_label_template(&mut self, template: Rc<dyn LabelTemplate>) {
        self.label_template = Some(template);
    }

    /// Installs the color palette used to paint the map, replacing the
    /// default [`Theme::default`].
    ///
    /// Accepts any [`MapTheme`] implementation, including a built-in
    /// [`Theme`] variant -- e.g. `map.set_theme(Rc::new(Theme::ArticCyan))` --
    /// or a custom palette. The new colors are resolved live from
    /// `new_theme` on the very next frame, in whichever light/dark mode is
    /// active then -- there is nothing to eagerly refresh, since
    /// [`Style`](theme::Style) never caches theme colors.
    pub fn set_theme(&mut self, new_theme: Rc<dyn MapTheme>) {
        self.theme = new_theme;
    }

    /// Adds the marker `id`, or moves it, so it points to the node `node_id`.
    ///
    /// Markers are drawn as a blinking ring around the target node unless a
    /// custom [`objects::NodeTemplate::marker_ui`] is installed.
    pub fn update_marker(&mut self, id: usize, node_id: usize) {
        let previous = self.markers.insert(id, node_id);
        if let Some(previous) = previous
            && previous != node_id
        {
            self.refresh_marker_presence(previous);
        }
        self.refresh_marker_presence(node_id);
    }

    /// Removes the marker `id`, if there is one, and returns the node it
    /// pointed to. Removing a marker that doesn't exist does nothing.
    pub fn remove_marker(&mut self, id: usize) -> Option<usize> {
        let removed = self.markers.remove(&id);
        if let Some(node_id) = removed {
            self.refresh_marker_presence(node_id);
        }
        removed
    }

    /// The node under the pointer in the last frame the widget was drawn, or
    /// `None` if the pointer isn't over the map or over any node.
    ///
    /// A node counts as under the pointer when the pointer is inside its hit
    /// area: [`NodeTemplate::contains`] with
    /// a template installed, a small circle around the node without one.
    /// Worked out in every [`VisibilitySetting`].
    ///
    /// Useful to attach a tooltip to some nodes only, from the `Response` the
    /// widget returns:
    ///
    /// ```no_run
    /// # use egui_map::map::Map;
    /// # fn show(ui: &mut egui::Ui, map: &mut Map) {
    /// let response = ui.add(&mut *map);
    /// if let Some(id) = map.hovered_node()
    ///     && id == 30000142
    /// {
    ///     response.on_hover_text_at_pointer("Jita");
    /// }
    /// # }
    /// ```
    pub fn hovered_node(&self) -> Option<usize> {
        self.hovered_node
    }

    /// Finds the node under the pointer. Candidates are every node whose
    /// center lies within the largest hit area a node can have
    /// ([`NodeTemplate::hit_extent`](objects::NodeTemplate::hit_extent), or
    /// the default radius without a template) of the pointer -- so a large
    /// area (a template drawing boxes) is never missed because other centers
    /// are nearer. Among the candidates whose hit area contains the pointer,
    /// the one painted last -- the one on top -- wins.
    fn find_hovered_node(&self, resp: &Response, min_point: &RawPoint) -> Option<usize> {
        if !resp.hovered() {
            return None;
        }
        let pointer = resp.hover_pos()?;
        let points = self.points.as_ref()?;
        let tree = self.tree.as_ref()?;
        let extent = match &self.node_template {
            Some(template) => template
                .hit_extent(self.zoom)
                .max(self.outline_reach.get() * self.zoom),
            None => objects::default_hit_extent(self.zoom),
        };
        let map_point = (*min_point + RawPoint::from(pointer)) / self.zoom;
        // The tree measures squared map units.
        let radius = extent.max(0.0) / self.zoom;
        let candidates = tree
            .within(&map_point.components, radius * radius, &squared_euclidean)
            .ok()?;
        candidates
            .into_iter()
            .filter_map(|(_, id)| {
                let point = points.get(id)?;
                let position: Pos2 = (RawPoint::from(point.coords) * self.zoom - *min_point).into();
                let ctx = HitContext {
                    position,
                    zoom: self.zoom,
                    point,
                };
                let hit = match &self.node_template {
                    Some(template) => template.contains(ctx, pointer),
                    None => ctx.within_default_radius(pointer),
                };
                hit.then_some(*id)
            })
            .max_by_key(|id| self.paint_order(*id))
    }

    /// Where `id` falls in this frame's painting order (later is on top);
    /// `None` for a node that isn't painted (outside the viewport).
    fn paint_order(&self, id: usize) -> Option<usize> {
        self.visible_points
            .iter()
            .rposition(|visible| visible.cast_unsigned() == id)
    }

    /// Starts fading `node_id`'s [`NodeContext::marker`](objects::NodeContext::marker)
    /// in or out to match whether any marker still points at it.
    fn refresh_marker_presence(&mut self, node_id: usize) {
        let on = self.markers.values().any(|node| *node == node_id);
        let previous = self.marker_presence.get(&node_id).copied();
        if previous.is_some_and(|presence| presence.on == on) || (previous.is_none() && !on) {
            return;
        }
        self.marker_presence.insert(
            node_id,
            MarkerPresence::switch(previous, on, Instant::now()),
        );
    }

    /// Current [`NodeContext::marker`](objects::NodeContext::marker) level
    /// of `node_id`, and whether it is still fading.
    fn marker_level(&self, node_id: usize, now: Instant) -> (f32, bool) {
        self.marker_presence
            .get(&node_id)
            .map_or((0.0, false), |presence| {
                (presence.level(now), presence.fading(now))
            })
    }

    /// Sets the minimum width and/or height the widget should occupy, in egui
    /// points. `None` leaves the corresponding dimension unconstrained.
    pub fn allocate_at_least(&mut self, width: Option<f32>, height: Option<f32>) {
        self.min_size = (width, height);
    }

    /// Sets the maximum width and/or height the widget should occupy, in egui
    /// points. `None` leaves the corresponding dimension unconstrained.
    pub fn allocate_at_most(&mut self, width: Option<f32>, height: Option<f32>) {
        self.max_size = (width, height);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn sample_points() -> Vec<MapPoint> {
        vec![
            MapPoint::new(1, [0.0, 0.0]),
            MapPoint::new(2, [10.0, 10.0]),
            MapPoint::new(3, [-10.0, -10.0]),
        ]
    }

    // ---------- construcción ----------

    #[test]
    fn map_new_initial_state() {
        let map = Map::new();
        assert_eq!(map.zoom, 1.0);
        assert_eq!(map.previous_zoom, 1.0);
        assert!(map.points.is_none());
        assert!(map.segments.is_none());
        assert!(map.tree.is_none());
        assert!(map.labels.is_empty());
        assert!(map.visible_points.is_empty());
        assert!(map.markers.is_empty());
        assert!(map.notifications.is_empty());
        assert!(map.node_states.is_empty());
        assert!(map.segment_notifications.is_empty());
        assert!(map.segment_states.is_empty());
        assert!(map.segment_ids.is_empty());
        assert_eq!(map.min_size, (None, None));
        assert_eq!(map.max_size, (None, None));
        assert!(!map.dark_mode);
    }

    #[test]
    fn map_default_equals_new() {
        let map = Map::default();
        assert_eq!(map.zoom, 1.0);
        assert!(map.points.is_none());
    }

    // ---------- zoom ----------

    #[test]
    fn set_zoom_within_range() {
        let mut map = Map::new();
        map.set_zoom(1.5);
        assert_eq!(map.get_zoom(), 1.5);
    }

    #[test]
    fn set_zoom_at_exact_limits() {
        let mut map = Map::new();
        map.set_zoom(map.settings.min_zoom);
        assert_eq!(map.get_zoom(), 0.1);
        map.set_zoom(map.settings.max_zoom);
        assert_eq!(map.get_zoom(), 2.0);
    }

    #[test]
    fn set_zoom_out_of_range_is_ignored() {
        let mut map = Map::new();
        let initial = map.get_zoom();
        map.set_zoom(0.05); // por debajo de min_zoom
        assert_eq!(map.get_zoom(), initial);
        map.set_zoom(2.5); // por encima de max_zoom
        assert_eq!(map.get_zoom(), initial);
    }

    // ---------- puntos ----------

    #[test]
    fn add_hashmap_points_computes_bounds() {
        let mut map = Map::new();
        map.add_points(sample_points());

        assert_eq!(map.reference.min.components, [-10.0, -10.0]);
        assert_eq!(map.reference.max.components, [10.0, 10.0]);
        // pos es el punto medio del rectángulo que contiene todos los puntos
        assert_eq!(map.reference.pos.components, [0.0, 0.0]);
        // map_area tiene área 0 antes de renderizar, así que dist es el valor fijo
        assert_eq!(map.reference.dist, 3000.0);
        // current se inicializa como copia de reference
        assert_eq!(map.current.min.components, map.reference.min.components);
        assert_eq!(map.current.max.components, map.reference.max.components);
        assert_eq!(map.current.pos.components, map.reference.pos.components);
        assert_eq!(map.current.dist, map.reference.dist);
        assert!(map.points.is_some());
        assert!(map.tree.is_some());
        assert_eq!(map.points.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn add_hashmap_points_populates_visible_points() {
        let mut map = Map::new();
        map.add_points(sample_points());
        // todos los puntos de muestra caen dentro del radio por defecto
        assert_eq!(map.visible_points.len(), 3);
    }

    /// Renders one frame of `map` in a 500x500 viewport and returns the
    /// painted line segments.
    fn render_line_segments(map: &mut Map) -> Vec<[egui::Pos2; 2]> {
        use egui::{Context, RawInput, Shape};
        let ctx = Context::default();
        let input = RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(500.0, 500.0),
            )),
            ..RawInput::default()
        };
        let mut output = ctx.run_ui(input, |ui| {
            ui.add(&mut *map);
        });
        // `TexturesDelta` panics on drop if it still holds an unhandled
        // delta (e.g. the font atlas uploaded on the first frame) -- clear
        // it explicitly instead of letting `output` fall out of scope with
        // it untouched. Same fix as `render_with` in
        // `tests/segment_animations.rs`.
        output.textures_delta.clear();
        output
            .shapes
            .iter()
            .filter_map(|cs| match cs.shape {
                Shape::LineSegment { points, .. } => Some(points),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn segment_crossing_viewport_is_painted_even_with_far_endpoints() {
        // With the old endpoint-based rule this line was culled: both
        // endpoints sit beyond the point-culling radius. With the R-tree the
        // segment AABB intersects the viewport, so it is painted — no points
        // needed at all.
        let mut map = Map::new();
        map.set_zoom(1.0);
        let lines = vec![MapSegment::new((1, 2), [-4000.0, -1.0], [4000.0, 1.0])];
        map.add_lines(lines);
        map.set_pos([0.0, 0.0]);

        let segments = render_line_segments(&mut map);
        assert_eq!(segments.len(), 1);
    }

    #[test]
    fn segment_outside_viewport_is_not_painted() {
        let mut map = Map::new();
        map.set_zoom(1.0);
        let lines = vec![MapSegment::new(
            (1, 2),
            [10_000.0, 10_000.0],
            [10_100.0, 10_100.0],
        )];
        map.add_lines(lines);
        map.set_pos([0.0, 0.0]);

        assert!(render_line_segments(&mut map).is_empty());
    }

    #[test]
    fn add_lines_builds_segment_tree() {
        let mut map = Map::new();
        map.add_points(sample_points());
        let lines = vec![MapSegment::new((1, 2), [0.0, 0.0], [10.0, 10.0])];
        map.add_lines(lines);

        let tree = map
            .segments
            .as_ref()
            .expect("add_lines must build the segment tree");
        assert_eq!(tree.size(), 1);

        // Broad-phase query: a viewport containing (0,0) must hit the segment;
        // a far-away viewport must not.
        let hit_query = rstar::AABB::from_corners([-1.0, -1.0], [1.0, 1.0]);
        let hits: Vec<_> = tree.locate_in_envelope_intersecting(hit_query).collect();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, (1, 2));

        let miss_query = rstar::AABB::from_corners([100.0, 100.0], [200.0, 200.0]);
        assert_eq!(tree.locate_in_envelope_intersecting(miss_query).count(), 0);
    }

    #[test]
    fn add_lines_populates_segment_ids() {
        let mut map = Map::new();
        map.add_lines(vec![
            MapSegment::new((1, 2), [0.0, 0.0], [10.0, 10.0]),
            MapSegment::new((3, 4), [1.0, 1.0], [2.0, 2.0]),
        ]);
        assert!(map.segment_ids.contains(&(1, 2)));
        assert!(map.segment_ids.contains(&(3, 4)));
        assert_eq!(map.segment_ids.len(), 2);

        // Replacing the set of lines replaces the id index too.
        map.add_hashmap_lines(HashMap::from([(
            (5, 6),
            MapSegment::new((5, 6), [0.0, 0.0], [1.0, 1.0]),
        )]));
        assert_eq!(map.segment_ids, HashSet::from([(5, 6)]));
    }

    #[test]
    fn map_check_line_is_painted_on_first_frame() {
        use egui::{Context, RawInput, Shape};

        // --- arrange ---
        let mut map = Map::new();
        map.set_zoom(1.0);

        let mut point_a = MapPoint::new(0, [0.0, 0.0]);
        point_a.connections.push((0, 1));
        let mut point_b = MapPoint::new(1, [50.0, 50.0]);
        point_b.connections.push((0, 1));

        let lines = vec![MapSegment::new((0, 1), point_a.coords, point_b.coords)];

        let points = vec![point_a, point_b];
        // Load points before lines — the natural order shown in the examples.
        map.add_points(points);
        map.add_lines(lines);

        map.set_pos([25.0, 25.0]);

        // --- act: 1st frame (no CentralPanel — run_ui creates the root Ui) ---
        let ctx = Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(500.0, 500.0));
        let input = RawInput {
            screen_rect: Some(screen),
            ..RawInput::default()
        };

        let mut output1 = ctx.run_ui(input.clone(), |ui| {
            ui.add(&mut map);
        });

        let segments1: Vec<[egui::Pos2; 2]> = output1
            .shapes
            .iter()
            .filter_map(|cs| match cs.shape {
                Shape::LineSegment { points, .. } => Some(points),
                _ => None,
            })
            .collect();
        // `TexturesDelta` panics on drop if it still holds an unhandled
        // delta (e.g. the font atlas uploaded on the first frame) -- clear
        // it explicitly rather than letting `output1` fall out of scope
        // with it untouched.
        output1.textures_delta.clear();

        assert!(
            !segments1.is_empty(),
            "Frame 1: no LineSegment shapes painted (map lines did not draw)"
        );

        // Expected projection of (0,0)->(50,50) with zoom=1, center=(25,25),
        // viewport 500x500: pos_a = (225, 225), pos_b = (275, 275). Tolerance ±2 px.
        let expected_a = egui::pos2(225.0, 225.0);
        let expected_b = egui::pos2(275.0, 275.0);
        let tolerance = 2.0;
        let found_on_frame1 = segments1.iter().any(|[p1, p2]| {
            let d_a1 = p1.distance(expected_a);
            let d_b1 = p2.distance(expected_b);
            let d_a2 = p2.distance(expected_a);
            let d_b2 = p1.distance(expected_b);
            (d_a1 < tolerance && d_b1 < tolerance) || (d_a2 < tolerance && d_b2 < tolerance)
        });
        assert!(
            found_on_frame1,
            "Frame 1: no LineSegment matches expected endpoints (~225,225 -> ~275,275); got {:?}",
            segments1
        );

        // --- act: 2nd frame (unchanged) — detect duplicate-lines regression ---
        let mut output2 = ctx.run_ui(input, |ui| {
            ui.add(&mut map);
        });

        let segments2: Vec<[egui::Pos2; 2]> = output2
            .shapes
            .iter()
            .filter_map(|cs| match cs.shape {
                Shape::LineSegment { points, .. } => Some(points),
                _ => None,
            })
            .collect();
        output2.textures_delta.clear();

        assert_eq!(
            segments1.len(),
            segments2.len(),
            "Frame 2: expected {} line segments (no duplication across frames), got {}",
            segments1.len(),
            segments2.len()
        );
    }

    // ---------- posición ----------

    #[test]
    fn set_pos_and_get_pos_roundtrip() {
        let mut map = Map::new();
        map.set_pos([25.0, -35.0]);
        assert_eq!(map.get_pos(), [25.0, -35.0]);
    }

    #[test]
    fn set_pos_from_nodeid_with_valid_id() {
        let mut map = Map::new();
        map.add_points(sample_points());
        assert!(map.set_pos_from_nodeid(2));
        assert_eq!(map.get_pos(), [10.0, 10.0]);
    }

    /// Regression: the widget used to allocate a painter of the full
    /// `map_area.size()` *inside* the canvas frame, so the frame grew by
    /// `2 * total_margin` and spilled out of the space the widget was given.
    /// The drawable region was then truncated on the right/bottom and its
    /// centre no longer matched `map_area.center()`, leaving a centred node
    /// visibly off-centre by half a margin.
    #[test]
    fn centered_node_is_painted_at_the_middle_of_the_drawable_area() {
        use egui::{Context, RawInput, Shape};

        for screen_size in [egui::vec2(500.0, 500.0), egui::vec2(800.0, 400.0)] {
            for zoom in [1.0, 2.0] {
                let mut map = Map::new();
                map.set_zoom(zoom);
                map.add_points(vec![MapPoint::new(7, [123.0, 456.0])]);
                map.set_pos_from_nodeid(7);

                let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, screen_size);
                let ctx = Context::default();
                let mut widget_rect = egui::Rect::NOTHING;
                let mut output = ctx.run_ui(
                    RawInput {
                        screen_rect: Some(screen),
                        ..RawInput::default()
                    },
                    |ui| {
                        widget_rect = ui.add(&mut map).rect;
                    },
                );

                // The widget must stay inside the space it was handed.
                assert!(
                    screen.contains_rect(widget_rect),
                    "{screen_size:?} zoom {zoom}: widget rect {widget_rect:?} overflows {screen:?}"
                );

                // The centred node must land at the middle of the region the
                // map actually paints into (the painter's clip rect).
                let (node_center, drawable) = output
                    .shapes
                    .iter()
                    .find_map(|cs| match &cs.shape {
                        Shape::Circle(circle) => Some((circle.center, cs.clip_rect)),
                        _ => None,
                    })
                    .expect("the node must be painted");

                assert!(
                    node_center.distance(drawable.center()) < 0.5,
                    "{screen_size:?} zoom {zoom}: node painted at {node_center:?} but the \
                     drawable area {drawable:?} is centred at {:?}",
                    drawable.center()
                );

                output.textures_delta.clear();
            }
        }
    }

    #[test]
    fn set_pos_from_nodeid_with_invalid_id_keeps_position() {
        let mut map = Map::new();
        map.add_points(sample_points());
        let before = map.reference.pos.components;
        // An unknown id must report the failure instead of silently no-op'ing.
        assert!(!map.set_pos_from_nodeid(999));
        assert_eq!(map.reference.pos.components, before);
    }

    #[test]
    fn set_pos_from_nodeid_without_points_does_nothing() {
        let mut map = Map::new();
        assert!(!map.set_pos_from_nodeid(1));
        assert_eq!(map.reference.pos.components, [0.0, 0.0]);
    }

    // ---------- etiquetas y líneas ----------

    #[test]
    fn add_labels_stores_labels() {
        let mut map = Map::new();
        let label = MapLabel {
            text: "Region".to_string(),
            center: Pos2::new(1.0, 2.0),
        };
        map.add_labels(vec![label]);
        assert_eq!(map.labels.len(), 1);
        assert_eq!(map.labels[0].text, "Region");
    }

    #[test]
    fn add_region_labels_stores_labels() {
        let mut map = Map::new();
        let label = RegionLabel {
            text: "Domain".to_string(),
            center: Pos2::new(3.0, 4.0),
            color: None,
        };
        map.add_region_labels(vec![label]);
        assert_eq!(map.region_labels.len(), 1);
        assert_eq!(map.region_labels[0].text, "Domain");
    }

    /// `add_region_labels` clears the built-in renderer's layout cache, so a
    /// stale `Arc<Galley>` for text that no longer exists in the new label
    /// set doesn't linger in memory forever.
    #[test]
    fn add_region_labels_clears_the_stale_layout_cache() {
        use egui::{Context, RawInput};

        let mut map = Map::new();
        map.add_region_labels(vec![RegionLabel {
            text: "Old".to_string(),
            center: Pos2::new(0.0, 0.0),
            color: None,
        }]);

        let ctx = Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400.0, 400.0));
        let mut output = ctx.run_ui(
            RawInput {
                screen_rect: Some(screen),
                ..RawInput::default()
            },
            |ui| {
                ui.add(&mut map);
            },
        );
        output.textures_delta.clear();

        assert_eq!(
            map.region_label_cache.len(),
            1,
            "expected one cached galley after painting one region label"
        );

        map.add_region_labels(vec![RegionLabel {
            text: "New".to_string(),
            center: Pos2::new(0.0, 0.0),
            color: None,
        }]);

        assert!(
            map.region_label_cache.is_empty(),
            "add_region_labels must clear the previous label set's cached galleys, found {:?}",
            map.region_label_cache.keys().collect::<Vec<_>>()
        );
    }

    /// Region labels whose projected position falls well outside the
    /// visible rect are skipped before any layout happens -- confirmed
    /// here via `region_label_cache` staying empty, the same signal
    /// `add_region_labels_clears_the_stale_layout_cache` uses to prove a
    /// label *did* get painted. Without this cull, every entry in
    /// `region_labels` (113 for EVE's regions, in the app this crate
    /// ships for) is laid out and painted every frame regardless of
    /// whether it is anywhere near the screen.
    #[test]
    fn region_labels_outside_viewport_are_culled() {
        use egui::{Context, RawInput};

        let mut map = Map::new();
        map.add_region_labels(vec![RegionLabel {
            text: "Far Away".to_string(),
            center: Pos2::new(10_000.0, 10_000.0),
            color: None,
        }]);

        let ctx = Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400.0, 400.0));
        let mut output = ctx.run_ui(
            RawInput {
                screen_rect: Some(screen),
                ..RawInput::default()
            },
            |ui| {
                ui.add(&mut map);
            },
        );
        output.textures_delta.clear();

        assert!(
            map.region_label_cache.is_empty(),
            "a region label far outside the viewport must not be laid out/painted, found {:?}",
            map.region_label_cache.keys().collect::<Vec<_>>()
        );
    }

    /// Positive control for `region_labels_outside_viewport_are_culled`:
    /// a label at the map's center must still be painted -- the cull
    /// must not be so aggressive it starts dropping labels that are
    /// actually visible.
    #[test]
    fn region_labels_within_viewport_are_painted() {
        use egui::{Context, RawInput};

        let mut map = Map::new();
        map.add_region_labels(vec![RegionLabel {
            text: "Domain".to_string(),
            center: Pos2::new(0.0, 0.0),
            color: None,
        }]);

        let ctx = Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400.0, 400.0));
        let mut output = ctx.run_ui(
            RawInput {
                screen_rect: Some(screen),
                ..RawInput::default()
            },
            |ui| {
                ui.add(&mut map);
            },
        );
        output.textures_delta.clear();

        assert_eq!(
            map.region_label_cache.len(),
            1,
            "expected one cached galley after painting one visible region label"
        );
    }

    /// Exact-byte counterpart to `tests/region_labels.rs`'s
    /// `region_label_color_is_theme_text_faded_by_alpha`, which can only
    /// assert an approximate color from outside the crate. From in here the
    /// expected value can go through `scale_alpha` itself, the same way
    /// `steady_segment_effect_alpha_tracks_the_lines_zoom_fade_with_a_head_start`
    /// does for segment effects, avoiding a hand-derived byte value that
    /// would be fragile against `Color32`'s premultiplied-alpha rounding.
    #[test]
    fn region_label_color_is_exactly_scale_alpha_of_theme_text() {
        use egui::{Context, RawInput, Shape};

        let mut map = Map::new();
        map.settings.region_label_alpha = 0.4;
        map.add_region_labels(vec![RegionLabel {
            text: "Domain".to_string(),
            center: Pos2::new(0.0, 0.0),
            color: None,
        }]);

        let ctx = Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400.0, 400.0));
        let mut output = ctx.run_ui(
            RawInput {
                screen_rect: Some(screen),
                ..RawInput::default()
            },
            |ui| {
                ui.add(&mut map);
            },
        );
        output.textures_delta.clear();

        // Read *after* the frame ran, once `assign_visual_style` has settled
        // `dark_mode` to whatever light/dark mode this `Context` actually
        // painted with -- reading it beforehand would compare against the
        // wrong mode's text color.
        let expected = scale_alpha(map.theme_colors().text, 0.4);

        let painted_color = output
            .shapes
            .iter()
            .find_map(|cs| match &cs.shape {
                Shape::Text(t) if t.galley.text() == "Domain" => t.override_text_color,
                _ => None,
            })
            .expect("region label was not painted");

        assert_eq!(painted_color, expected);
    }

    #[test]
    fn add_lines_stores_lines() {
        let mut map = Map::new();
        let lines = vec![MapSegment::new((1, 2), [0.0, 0.0], [1.0, 1.0])];
        map.add_lines(lines);
        let tree = map.segments.as_ref().unwrap();
        assert_eq!(tree.size(), 1);
        assert_eq!(
            tree.locate_in_envelope_intersecting(rstar::AABB::from_corners(
                [-1.0, -1.0],
                [2.0, 2.0],
            ))
            .next()
            .unwrap()
            .id,
            (1, 2)
        );
    }

    // ---------- notificaciones y marcadores ----------

    #[test]
    fn line_at_returns_closest_line_within_tolerance() {
        let mut map = Map::new();
        map.add_points(sample_points());
        let lines = vec![
            MapSegment::new((1, 2), [0.0, 0.0], [10.0, 0.0]),
            MapSegment::new((3, 4), [20.0, -5.0], [20.0, 5.0]),
        ];
        map.add_lines(lines);

        // 1.5 units above the horizontal segment.
        let hit = map.line_at([5.0, 1.5], 2.0).expect("line must be hit");
        assert_eq!(hit, (1, 2));

        // Closest to the vertical segment.
        let hit = map.line_at([19.0, 0.0], 2.0).expect("line must be hit");
        assert_eq!(hit, (3, 4));
    }

    #[test]
    fn line_at_returns_none_beyond_tolerance() {
        let mut map = Map::new();
        map.add_points(sample_points());
        let lines = vec![MapSegment::new((1, 2), [0.0, 0.0], [10.0, 10.0])];
        map.add_lines(lines);

        // Distance from (5,4) to the diagonal segment (0,0)-(10,10) is
        // |5-4|/sqrt(2) ~= 0.707.
        assert!(map.line_at([5.0, 4.0], 0.8).is_some());
        assert!(map.line_at([5.0, 4.0], 0.5).is_none());
        assert!(map.line_at([100.0, 100.0], 5.0).is_none());
    }

    #[test]
    fn line_at_returns_none_without_lines() {
        let map = Map::new();
        assert!(map.line_at([0.0, 0.0], 10.0).is_none());
    }

    #[test]
    fn line_at_negative_tolerance_behaves_like_zero() {
        let mut map = Map::new();
        map.add_points(sample_points());
        let lines = vec![MapSegment::new((1, 2), [0.0, 0.0], [10.0, 10.0])];
        map.add_lines(lines);

        // Exact point on the segment is hit even with tolerance clamped to 0.
        assert!(map.line_at([5.0, 5.0], -1.0).is_some());
        assert!(map.line_at([5.0, 5.1], -1.0).is_none());
    }

    /// The deprecated shortcut must keep behaving exactly as it did: a pulse,
    /// restarted on every call, and tolerant of ids that were never loaded.
    #[test]
    #[allow(deprecated)]
    fn deprecated_notify_still_records_a_pulse() {
        let mut map = Map::new();
        let t1 = Instant::now();
        map.notify(5, t1);
        let recorded = map.notifications.get(&5).expect("notify must record");
        assert_eq!(recorded.started, t1);
        assert_eq!(recorded.animation, NodeAnimation::Pulse);
        assert_eq!(recorded.color, None);

        let t2 = t1 + Duration::from_secs(1);
        map.notify(5, t2);
        assert_eq!(map.notifications.get(&5).unwrap().started, t2);
        assert_eq!(map.notifications.len(), 1);
    }

    // ---------- NodeHandle ----------

    fn map_with_nodes() -> Map {
        let mut map = Map::new();
        map.add_points(vec![
            MapPoint::new(1, [0.0, 0.0]),
            MapPoint::new(2, [10.0, 10.0]),
        ]);
        map
    }

    #[test]
    fn node_returns_none_for_an_unknown_id() {
        let mut map = map_with_nodes();
        assert!(map.node(1).is_some());
        assert!(map.node(999).is_none());
        // and with nothing loaded at all
        assert!(Map::new().node(1).is_none());
    }

    #[test]
    fn each_event_effect_records_its_own_animation() {
        let now = Instant::now();
        for (apply, expected) in [
            (
                Box::new(|n: NodeHandle| n.pulse(now)) as Box<dyn FnOnce(NodeHandle)>,
                NodeAnimation::Pulse,
            ),
            (
                Box::new(|n: NodeHandle| n.ripple(now)),
                NodeAnimation::Ripple,
            ),
            (
                Box::new(|n: NodeHandle| n.countdown(now)),
                NodeAnimation::CountdownArc,
            ),
            (
                Box::new(|n: NodeHandle| n.scale_in(now)),
                NodeAnimation::ScaleIn,
            ),
            (
                Box::new(|n: NodeHandle| n.crosshair(now)),
                NodeAnimation::Crosshair,
            ),
        ] {
            let mut map = map_with_nodes();
            apply(map.node(1).unwrap());
            let recorded = map.notifications.get(&1).expect("effect must be recorded");
            assert_eq!(recorded.animation, expected);
            assert_eq!(recorded.started, now);
            // an event effect must not leave lasting state behind
            assert!(map.node_states.is_empty());
        }
    }

    #[test]
    fn each_steady_effect_records_lasting_state() {
        for (apply, expected) in [
            (
                Box::new(|n: NodeHandle| n.halo()) as Box<dyn FnOnce(NodeHandle)>,
                SteadyAnimation::Halo,
            ),
            (Box::new(|n: NodeHandle| n.blink()), SteadyAnimation::Blink),
            (Box::new(|n: NodeHandle| n.orbit()), SteadyAnimation::Orbit),
        ] {
            let mut map = map_with_nodes();
            apply(map.node(1).unwrap());
            assert_eq!(map.node_states.get(&1).unwrap().animation, expected);
            // lasting state must not masquerade as a notification
            assert!(map.notifications.is_empty());
        }
    }

    #[test]
    fn color_modifier_reaches_both_families() {
        let mut map = map_with_nodes();
        map.node(1)
            .unwrap()
            .color(Color32::RED)
            .pulse(Instant::now());
        map.node(2).unwrap().color(Color32::BLUE).halo();

        assert_eq!(map.notifications.get(&1).unwrap().color, Some(Color32::RED));
        assert_eq!(map.node_states.get(&2).unwrap().color, Some(Color32::BLUE));
    }

    #[test]
    fn a_node_can_carry_state_and_a_notification_at_once() {
        let mut map = map_with_nodes();
        map.node(1).unwrap().halo();
        map.node(1).unwrap().ripple(Instant::now());

        assert!(map.node_states.contains_key(&1));
        assert!(map.notifications.contains_key(&1));
    }

    #[test]
    fn clear_removes_both_families_for_that_node_only() {
        let mut map = map_with_nodes();
        map.node(1).unwrap().halo();
        map.node(1).unwrap().ripple(Instant::now());
        map.node(2).unwrap().halo();

        map.node(1).unwrap().clear();

        assert!(!map.node_states.contains_key(&1));
        assert!(!map.notifications.contains_key(&1));
        assert!(map.node_states.contains_key(&2), "node 2 must be untouched");
    }

    #[test]
    fn re_triggering_replaces_the_previous_effect() {
        let mut map = map_with_nodes();
        map.node(1).unwrap().pulse(Instant::now());
        map.node(1).unwrap().crosshair(Instant::now());

        assert_eq!(map.notifications.len(), 1);
        assert_eq!(
            map.notifications.get(&1).unwrap().animation,
            NodeAnimation::Crosshair
        );
    }

    // ---------- SegmentHandle ----------

    fn map_with_segments() -> Map {
        let mut map = Map::new();
        map.add_lines(vec![
            MapSegment::new((1, 2), [0.0, 0.0], [10.0, 0.0]),
            MapSegment::new((3, 4), [0.0, 10.0], [10.0, 10.0]),
        ]);
        map
    }

    #[test]
    fn segment_returns_none_for_an_unknown_id() {
        let mut map = map_with_segments();
        assert!(map.segment((1, 2)).is_some());
        assert!(map.segment((404, 404)).is_none());
        // and with nothing loaded at all
        assert!(Map::new().segment((1, 2)).is_none());
    }

    #[test]
    fn flash_records_a_segment_notification() {
        let mut map = map_with_segments();
        let now = Instant::now();
        map.segment((1, 2)).unwrap().flash(now);

        let recorded = map
            .segment_notifications
            .get(&(1, 2))
            .expect("flash must be recorded");
        assert_eq!(recorded.animation, SegmentAnimation::FlashDecay);
        assert_eq!(recorded.started, now);
        // an event effect must not leave lasting state behind
        assert!(map.segment_states.is_empty());
    }

    #[test]
    fn each_event_segment_effect_records_its_own_notification() {
        for (apply, expected) in [
            (
                Box::new(|s: SegmentHandle, at: Instant| s.flash(at))
                    as Box<dyn FnOnce(SegmentHandle, Instant)>,
                SegmentAnimation::FlashDecay,
            ),
            (
                Box::new(|s: SegmentHandle, at: Instant| s.comet_once(at, CometDirection::Forward)),
                SegmentAnimation::Comet(CometDirection::Forward),
            ),
            (
                Box::new(|s: SegmentHandle, at: Instant| s.comet_once(at, CometDirection::Reverse)),
                SegmentAnimation::Comet(CometDirection::Reverse),
            ),
            (
                Box::new(|s: SegmentHandle, at: Instant| s.wipe(at)),
                SegmentAnimation::Wipe,
            ),
        ] {
            let mut map = map_with_segments();
            let now = Instant::now();
            apply(map.segment((1, 2)).unwrap(), now);

            let recorded = map
                .segment_notifications
                .get(&(1, 2))
                .expect("the effect must be recorded");
            assert_eq!(recorded.animation, expected);
            assert_eq!(recorded.started, now);
            // an event effect must not leave lasting state behind
            assert!(map.segment_states.is_empty());
        }
    }

    #[test]
    fn each_steady_segment_effect_records_lasting_state() {
        for (apply, expected) in [
            (
                Box::new(|s: SegmentHandle| s.comet()) as Box<dyn FnOnce(SegmentHandle)>,
                SteadySegmentAnimation::Comet,
            ),
            (
                Box::new(|s: SegmentHandle| s.dash()),
                SteadySegmentAnimation::Dash,
            ),
            (
                Box::new(|s: SegmentHandle| s.glow_band()),
                SteadySegmentAnimation::GlowBand,
            ),
            (
                Box::new(|s: SegmentHandle| s.chevrons()),
                SteadySegmentAnimation::Chevrons,
            ),
        ] {
            let mut map = map_with_segments();
            apply(map.segment((1, 2)).unwrap());
            assert_eq!(map.segment_states.get(&(1, 2)).unwrap().animation, expected);
            // lasting state must not masquerade as a notification
            assert!(map.segment_notifications.is_empty());
        }
    }

    #[test]
    fn steady_segment_effect_alpha_tracks_the_lines_zoom_fade_with_a_head_start() {
        // At zoom 0.6 with the default `line_visible_zoom` of 0.2, the line's
        // own fade (`line_fade`) is exactly half-way through its 0.80-unit
        // ramp: `(0.6 - 0.2) / 0.80 == 0.5`. `comet` paints `color` on a
        // `Shape::Circle` with no alpha adjustment of its own, so whatever
        // alpha lands on screen must be exactly `scale_alpha`'s output --
        // `effect_fade = (0.5 + SEGMENT_EFFECT_ALPHA_BOOST).min(1.0) = 0.7`.
        let mut map = map_with_segments();
        map.set_zoom(0.6);
        let color = Color32::from_rgba_unmultiplied(10, 20, 30, 200);
        map.segment((1, 2)).unwrap().color(color).comet();

        // `scale_alpha` goes through `Color32::to_srgba_unmultiplied` before
        // reconstructing the color, so the expected fill is whatever that
        // round trip actually produces -- not a hand-derived byte value,
        // which would be fragile against the gamma-aware LUT egui's
        // premultiply table uses internally.
        let expected_fill = scale_alpha(color, 0.7);
        assert_eq!(
            expected_fill.a(),
            140,
            "sanity-check the hand-derived alpha"
        );

        let ctx = Context::default();
        let screen_rect = Rect::from_min_size(Pos2::ZERO, vec2(400.0, 300.0));
        let mut out = ctx.run_ui(
            RawInput {
                screen_rect: Some(screen_rect),
                ..RawInput::default()
            },
            |ui| {
                ui.add(&mut map);
            },
        );
        // Font-atlas texture deltas are dropped safely only when explicitly
        // cleared first -- see the identical pattern in `render_line_segments`
        // and `tests/segment_animations.rs`'s `render_with`.
        out.textures_delta.clear();

        let comet_circle = out
            .shapes
            .iter()
            .find_map(|cs| match &cs.shape {
                Shape::Circle(c) => Some(*c),
                _ => None,
            })
            .expect("comet must paint a filled circle");

        assert_eq!(
            comet_circle.fill, expected_fill,
            "the comet's fill must be exactly scale_alpha(color, effect_fade) -- the line's \
             zoom fade with the documented head start applied"
        );
    }

    #[test]
    fn color_modifier_reaches_both_segment_families() {
        let mut map = map_with_segments();
        map.segment((1, 2))
            .unwrap()
            .color(Color32::RED)
            .flash(Instant::now());
        map.segment((3, 4)).unwrap().color(Color32::BLUE).comet();

        assert_eq!(
            map.segment_notifications.get(&(1, 2)).unwrap().color,
            Some(Color32::RED)
        );
        assert_eq!(
            map.segment_states.get(&(3, 4)).unwrap().color,
            Some(Color32::BLUE)
        );
    }

    #[test]
    fn a_segment_can_carry_state_and_a_notification_at_once() {
        let mut map = map_with_segments();
        map.segment((1, 2)).unwrap().comet();
        map.segment((1, 2)).unwrap().flash(Instant::now());

        assert!(map.segment_states.contains_key(&(1, 2)));
        assert!(map.segment_notifications.contains_key(&(1, 2)));
    }

    #[test]
    fn clear_removes_both_segment_families_for_that_id_only() {
        let mut map = map_with_segments();
        map.segment((1, 2)).unwrap().comet();
        map.segment((1, 2)).unwrap().flash(Instant::now());
        map.segment((3, 4)).unwrap().comet();

        map.segment((1, 2)).unwrap().clear();

        assert!(!map.segment_states.contains_key(&(1, 2)));
        assert!(!map.segment_notifications.contains_key(&(1, 2)));
        assert!(
            map.segment_states.contains_key(&(3, 4)),
            "segment (3, 4) must be untouched"
        );
    }

    #[test]
    fn re_flashing_a_segment_restarts_it() {
        let mut map = map_with_segments();
        let t1 = Instant::now();
        map.segment((1, 2)).unwrap().flash(t1);
        let t2 = t1 + Duration::from_secs(1);
        map.segment((1, 2)).unwrap().flash(t2);

        assert_eq!(map.segment_notifications.len(), 1);
        assert_eq!(map.segment_notifications.get(&(1, 2)).unwrap().started, t2);
    }

    #[test]
    fn update_marker_inserts_and_updates() {
        let mut map = Map::new();
        map.update_marker(1, 100);
        assert_eq!(map.markers.get(&1), Some(&100));
        map.update_marker(1, 200);
        assert_eq!(map.markers.get(&1), Some(&200));
        assert_eq!(map.markers.len(), 1);
    }

    #[test]
    fn lasting_notifications_cycle_and_fade() {
        let started = Instant::now();
        let at = |secs: f32| started + std::time::Duration::from_secs_f32(secs);
        // 7.5 s into a 3.5 s cycle: the third cycle began 0.5 s ago.
        let cycle = animation::cycle_start(started, at(7.5), 3.5);
        assert!((at(7.5).duration_since(cycle).as_secs_f32() - 0.5).abs() < 1e-3);

        let lasting = Notification {
            started,
            animation: NodeAnimation::Pulse,
            color: None,
            until: Some(at(100.0)),
        };
        assert_eq!(lasting.remaining(started), 1.0);
        assert!((lasting.remaining(at(25.0)) - 0.75).abs() < 1e-3);
        assert_eq!(lasting.remaining(at(200.0)), 0.0);
        assert!(!lasting.expired(at(99.0)));
        assert!(lasting.expired(at(100.0)));

        let once = Notification {
            until: None,
            ..lasting
        };
        assert_eq!(once.remaining(at(50.0)), 1.0);
        assert!(!once.expired(at(1000.0)));
    }

    #[test]
    fn marker_presence_fades_in_and_out() {
        let start = Instant::now();
        let fade = std::time::Duration::from_secs_f32(objects::MARKER_FADE_SECS);
        let presence = MarkerPresence::switch(None, true, start);
        assert_eq!(presence.level(start), 0.0);
        assert!((presence.level(start + fade / 2) - 0.5).abs() < 0.01);
        assert_eq!(presence.level(start + fade * 2), 1.0);
        assert!(presence.fading(start + fade / 2));
        assert!(!presence.fading(start + fade * 2));

        // Turned off halfway: it fades out from where it was, not from 1.0.
        let half = start + fade / 2;
        let off = MarkerPresence::switch(Some(presence), false, half);
        assert!((off.level(half) - 0.5).abs() < 0.01);
        assert_eq!(off.level(half + fade), 0.0);
    }

    #[test]
    fn marker_presence_follows_the_markers() {
        let later = || Instant::now() + std::time::Duration::from_secs(10);
        let mut map = Map::new();
        map.update_marker(1, 100);
        map.update_marker(2, 100);
        assert_eq!(map.marker_level(100, later()).0, 1.0);

        // One of two markers moves away: the node still has one.
        map.update_marker(1, 200);
        assert_eq!(map.marker_level(100, later()).0, 1.0);
        assert_eq!(map.marker_level(200, later()).0, 1.0);

        map.remove_marker(2);
        assert_eq!(map.marker_level(100, later()).0, 0.0);
        assert_eq!(map.marker_level(300, later()), (0.0, false));
    }

    #[test]
    fn remove_marker_removes_only_that_marker() {
        let mut map = Map::new();
        map.update_marker(1, 100);
        map.update_marker(2, 200);
        assert_eq!(map.remove_marker(1), Some(100));
        assert_eq!(map.markers.get(&1), None);
        assert_eq!(map.markers.get(&2), Some(&200));
        // Removing it again, or a marker that never existed, is a no-op.
        assert_eq!(map.remove_marker(1), None);
        assert_eq!(map.remove_marker(99), None);
        assert_eq!(map.markers.len(), 1);
    }

    // ---------- tamaño ----------

    #[test]
    fn allocate_at_least_sets_min_size() {
        let mut map = Map::new();
        map.allocate_at_least(Some(100.0), None);
        assert_eq!(map.min_size, (Some(100.0), None));
    }

    #[test]
    fn allocate_at_most_sets_max_size() {
        let mut map = Map::new();
        map.allocate_at_most(None, Some(200.0));
        assert_eq!(map.max_size, (None, Some(200.0)));
    }

    // ---------- bounds ----------

    #[test]
    fn adjust_bounds_scales_with_zoom() {
        let mut map = Map::new();
        map.reference.min = RawPoint::new(-10.0, -20.0);
        map.reference.max = RawPoint::new(10.0, 20.0);
        map.reference.pos = RawPoint::new(5.0, 5.0);
        map.reference.dist = 100.0;
        map.set_zoom(2.0);
        map.adjust_bounds();

        assert_eq!(map.current.max.components, [20.0, 40.0]);
        assert_eq!(map.current.min.components, [-20.0, -40.0]);
        assert_eq!(map.current.pos.components, [10.0, 10.0]);
        assert_eq!(map.current.dist, 50.0);
    }

    // ---------- theme ----------

    #[test]
    fn theme_colors_are_resolved_live_from_the_installed_theme() {
        // `Style` no longer caches any color (see `theme.rs`), so there is
        // nothing for `set_theme` to eagerly refresh -- `theme_colors()`
        // must reflect the newly installed theme immediately, in whichever
        // light/dark mode is active, without waiting for a mode flip.
        let mut map = Map::new();
        map.set_theme(Rc::new(Theme::ArticCyan));

        let light = Theme::ArticCyan.colors(ColorMode::Light);
        let dark = Theme::ArticCyan.colors(ColorMode::Dark);

        assert!(!map.dark_mode, "Map::new starts in light mode");
        assert_eq!(map.theme_colors(), light);

        // Simulate the app flipping to dark mode: still the very same
        // installed theme, resolved for the other `ColorMode`.
        map.dark_mode = true;
        assert_eq!(map.theme_colors(), dark);
    }

    #[test]
    fn a_custom_map_theme_reaches_the_painted_node() {
        // End-to-end: a `MapTheme` installed through `set_theme` must be the
        // color a plain (un-templated, un-colored) node is actually painted
        // with, not just a value sitting in `settings.style`.
        use crate::map::theme::ThemeColors;
        use egui::{Context, RawInput, Shape};

        struct FixedPalette;
        impl MapTheme for FixedPalette {
            fn colors(&self, _mode: ColorMode) -> ThemeColors {
                ThemeColors {
                    node: Color32::from_rgb(1, 2, 3),
                    segment: Color32::from_rgb(4, 5, 6),
                    selected: Color32::from_rgb(7, 8, 9),
                    alert: Color32::from_rgb(10, 11, 12),
                    marker: Color32::from_rgb(16, 17, 18),
                    text: Color32::from_rgb(13, 14, 15),
                    background: Color32::from_rgb(19, 20, 21),
                }
            }
        }

        let mut map = Map::new();
        map.set_theme(Rc::new(FixedPalette));
        map.add_points(vec![MapPoint::new(1, [0.0, 0.0])]);

        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(200.0, 200.0));
        let ctx = Context::default();
        let mut output = ctx.run_ui(
            RawInput {
                screen_rect: Some(screen),
                ..RawInput::default()
            },
            |ui| {
                ui.add(&mut map);
            },
        );
        // `TexturesDelta` panics on drop if left unhandled.
        output.textures_delta.clear();

        let fill = output
            .shapes
            .iter()
            .find_map(|cs| match &cs.shape {
                Shape::Circle(circle) => Some(circle.fill),
                _ => None,
            })
            .expect("the node must be painted");

        assert_eq!(
            fill,
            Color32::from_rgb(1, 2, 3),
            "the node fill must come from the installed MapTheme, in either color mode"
        );
    }

    #[test]
    fn a_custom_map_theme_reaches_the_selection_highlight() {
        // End-to-end: `SelectionContext::color` must be the installed
        // `MapTheme`'s `selected` color -- previously defined on every
        // `ThemeColors` but never actually consumed anywhere in painting --
        // and `SelectionContext::point` must be the node the widget
        // actually computed as nearest to the pointer.
        use crate::map::theme::ThemeColors;
        use egui::{Context, Event, RawInput};
        use std::cell::RefCell;

        struct FixedPalette;
        impl MapTheme for FixedPalette {
            fn colors(&self, _mode: ColorMode) -> ThemeColors {
                ThemeColors {
                    node: Color32::from_rgb(1, 2, 3),
                    segment: Color32::from_rgb(4, 5, 6),
                    selected: Color32::from_rgb(7, 8, 9),
                    alert: Color32::from_rgb(10, 11, 12),
                    marker: Color32::from_rgb(16, 17, 18),
                    text: Color32::from_rgb(13, 14, 15),
                    background: Color32::from_rgb(19, 20, 21),
                }
            }
        }

        #[derive(Default)]
        struct RecordingTemplate {
            seen: RefCell<Option<(usize, Color32)>>,
        }
        impl NodeTemplate for RecordingTemplate {
            fn node_ui(&self, _ui: &mut Ui, _ctx: NodeContext) {}
            fn selection_ui(&self, _ui: &mut Ui, ctx: SelectionContext) {
                *self.seen.borrow_mut() = Some((ctx.point.get_id(), ctx.color));
            }
            fn notification_ui(&self, _ui: &mut Ui, _ctx: NotificationContext) -> bool {
                false
            }
            fn marker_ui(&self, _ui: &mut Ui, _ctx: MarkerContext) {}
        }

        let mut map = Map::new();
        map.set_theme(Rc::new(FixedPalette));
        map.settings.node_text_visibility = VisibilitySetting::Hover;
        map.add_points(vec![MapPoint::new(9, [0.0, 0.0])]);
        let template = Rc::new(RecordingTemplate::default());
        map.set_node_template(template.clone());

        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(200.0, 200.0));
        let ctx = Context::default();
        // Two passes: egui needs a frame to lay the widget out before its
        // `Response::hovered()` reflects a pointer position landed in the
        // same frame (same reasoning as `tests/debug_overlay.rs`).
        for pass in 0..2 {
            let events = if pass == 1 {
                vec![Event::PointerMoved(screen.center())]
            } else {
                Vec::new()
            };
            let mut output = ctx.run_ui(
                RawInput {
                    screen_rect: Some(screen),
                    events,
                    ..RawInput::default()
                },
                |ui| {
                    ui.add(&mut map);
                },
            );
            // `TexturesDelta` panics on drop if left unhandled.
            output.textures_delta.clear();
        }

        let (id, color) = template
            .seen
            .borrow()
            .expect("selection_ui must be called while the pointer hovers the map");
        assert_eq!(
            id, 9,
            "the highlighted node must be the one under the pointer"
        );
        assert_eq!(
            color,
            Color32::from_rgb(7, 8, 9),
            "the highlight color must come from the installed MapTheme's `selected`"
        );
    }

    #[test]
    fn contexts_carry_the_settings_animation() {
        // The `animation` a template receives must be the one on
        // `settings.animation`, so a template drawing an effect itself uses
        // the same tuning the built-in path would.
        #[derive(Default)]
        struct Recorder {
            marker: std::cell::RefCell<Option<Animation>>,
            notification: std::cell::RefCell<Option<Animation>>,
            node: std::cell::RefCell<Option<Animation>>,
        }
        impl NodeTemplate for Recorder {
            fn node_ui(&self, _ui: &mut Ui, ctx: NodeContext) {
                *self.node.borrow_mut() = Some(ctx.animation);
            }
            fn notification_ui(&self, _ui: &mut Ui, ctx: NotificationContext) -> bool {
                *self.notification.borrow_mut() = Some(ctx.animation);
                false
            }
            fn marker_ui(&self, _ui: &mut Ui, ctx: MarkerContext) {
                *self.marker.borrow_mut() = Some(ctx.animation);
            }
        }

        let expected = Animation::default().with(|a| a.pulse.spread = 123.4);
        let mut map = Map::new();
        map.add_points(vec![MapPoint::new(1, [0.0, 0.0])]);
        map.settings.animation = expected;
        let template = Rc::new(Recorder::default());
        map.set_node_template(template.clone());
        map.node(1).unwrap().pulse(Instant::now());
        map.update_marker(42, 1);

        let ctx = Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(200.0, 200.0));
        let mut output = ctx.run_ui(
            RawInput {
                screen_rect: Some(screen),
                ..RawInput::default()
            },
            |ui| {
                ui.add(&mut map);
            },
        );
        output.textures_delta.clear();

        assert_eq!(template.notification.borrow().as_ref(), Some(&expected));
        assert_eq!(template.marker.borrow().as_ref(), Some(&expected));
        assert_eq!(template.node.borrow().as_ref(), Some(&expected));
    }
}

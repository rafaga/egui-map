//! Custom `NodeTemplate` that draws its own diamond-shaped nodes, but reuses
//! the crate's built-in `Animation::*` functions for its effects instead of
//! hand-rolling new ones -- contrast with `examples/custom_template.rs`,
//! which hand-rolls a single effect of its own from scratch.
//!
//! As of egui-map 0.5.0, [`NodeTemplate::notification_ui`] and
//! [`NodeTemplate::marker_ui`] are told which built-in animation was
//! requested -- `kind`, a
//! [`NodeAnimation`](egui_map::map::objects::NodeAnimation) or
//! [`SteadyAnimation`](egui_map::map::objects::SteadyAnimation) respectively
//! -- and which node it belongs to (`node_id`), so a template can dispatch
//! straight to the matching `Animation::*` function instead of
//! reimplementing the lookup itself:
//!
//! - `notification_ui`'s `kind` picks between `Pulse`/`Ripple`/
//!   `CountdownArc`/`ScaleIn`/`Crosshair` -- see the `match` below.
//! - `marker_ui`'s `kind` picks between `Halo`/`Blink`/`Orbit`. It is still
//!   the one hook shared by two different things: persistent node state
//!   (`halo`/`blink`/`orbit`, requested per node through `Map::node`) and
//!   plain markers registered with `Map::update_marker` (which all share one
//!   `MapSettings::marker_animation`, so their `kind` is always that one
//!   setting). `kind` is enough to draw the right effect either way, but the
//!   hook still cannot tell the two *call sites* apart, and still receives
//!   no color -- this template picks its own (`MARKER_COLOR`) for both.
//!
//! (Earlier versions of this example, before 0.5.0, worked around the
//! missing `kind` by giving each event effect its own exact, dedicated color
//! when firing it and matching on that -- a real but narrow trick that only
//! worked because the example controlled every call site's color, and could
//! not help `marker_ui` at all since it never even received a color. That
//! workaround is gone now that `kind` is passed directly; the per-effect
//! colors below are kept only for visual variety, not for dispatch.)
//!
//! Segments are not templated here -- they use the widget's default segment
//! rendering and animation dispatch, same as `examples/animations.rs` (which
//! already showcases the full segment catalog on its own). This example just
//! wires the nodes into a small connected network -- a ring plus a few
//! chords, so most nodes have more than one neighbor -- rather than the
//! isolated pairs `animations.rs` uses, so the persistent segment effects
//! (`comet`/`dash`/`glow_band`/`chevrons`, cycled across the edges) read as
//! a live network instead of a handful of disconnected demo lines.
//!
//! Run with: cargo run --example node_template_animations

use eframe::egui::{self, Align2, Color32, Pos2, Shape, Stroke, Ui, Vec2};
use egui_map::map::Map;
use egui_map::map::animation::Animation;
use egui_map::map::objects::{
    MapPoint, MapSegment, MarkerContext, NodeAnimation, NodeTemplate, NotificationContext,
    SteadyAnimation, VisibilitySetting,
};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

// -------------------------------------------------------------- demo colors

/// One color per event effect, purely so they read apart on screen -- with
/// `kind` passed directly to `notification_ui` these no longer double as a
/// dispatch key (see the module docs above).
const PULSE_COLOR: Color32 = Color32::from_rgb(80, 160, 255);
const RIPPLE_COLOR: Color32 = Color32::from_rgb(255, 170, 60);
const COUNTDOWN_COLOR: Color32 = Color32::from_rgb(170, 110, 255);
const SCALE_IN_COLOR: Color32 = Color32::from_rgb(90, 210, 140);
const CROSSHAIR_COLOR: Color32 = Color32::from_rgb(255, 90, 90);
/// Color [`DiamondNodes::marker_ui`] draws with -- unlike `notification_ui`,
/// it is not handed a color at all, so the template has to pick one itself.
const MARKER_COLOR: Color32 = Color32::from_rgb(120, 220, 255);

// -------------------------------------------------------------- NodeTemplate

struct DiamondNodes;

impl NodeTemplate for DiamondNodes {
    /// Custom node shape: a filled diamond with the node name above it.
    fn node_ui(&self, ui: &mut Ui, position: Pos2, zoom: f32, point: &MapPoint) {
        let size = 9.0 * zoom;
        let diamond = vec![
            Pos2::new(position.x, position.y - size),
            Pos2::new(position.x + size, position.y),
            Pos2::new(position.x, position.y + size),
            Pos2::new(position.x - size, position.y),
        ];
        let painter = ui.painter();
        painter.add(Shape::convex_polygon(
            diamond,
            Color32::from_rgb(60, 70, 90),
            Stroke::new(1.5 * zoom, Color32::from_rgb(150, 170, 200)),
        ));
        painter.text(
            position + Vec2::new(0.0, -size - 3.0 * zoom),
            Align2::CENTER_BOTTOM,
            point.get_name(),
            egui::FontId::proportional(11.0 * zoom),
            ui.visuals().text_color(),
        );
    }

    /// Highlight ring over the node closest to the mouse pointer.
    fn selection_ui(&self, ui: &mut Ui, position: Pos2, zoom: f32) {
        ui.painter().circle_stroke(
            position,
            14.0 * zoom,
            Stroke::new(2.0 * zoom, Color32::YELLOW),
        );
    }

    /// Reuses one of the five built-in event effects, picked directly from
    /// `ctx.kind` -- no more color-matching, see the module docs above.
    fn notification_ui(&self, ui: &mut Ui, ctx: NotificationContext) -> bool {
        let NotificationContext {
            position,
            zoom,
            initial_time,
            color,
            kind,
            ..
        } = ctx;
        let painter = ui.painter();
        let still_playing = match kind {
            NodeAnimation::Pulse => Animation::pulse(painter, position, zoom, initial_time, color),
            NodeAnimation::Ripple => {
                Animation::ripple(painter, position, zoom, initial_time, color)
            }
            NodeAnimation::CountdownArc => {
                Animation::countdown_arc(painter, position, zoom, initial_time, color)
            }
            NodeAnimation::ScaleIn => {
                Animation::scale_in(painter, position, zoom, initial_time, color)
            }
            NodeAnimation::Crosshair => {
                Animation::crosshair(painter, position, zoom, initial_time, color)
            }
        };
        ui.ctx().request_repaint();
        still_playing
    }

    /// Reuses the matching built-in persistent effect, picked directly from
    /// `ctx.kind` -- `Halo`/`Blink`/`Orbit` now render as themselves instead
    /// of every node with lasting state or a marker collapsing onto the same
    /// `Animation::halo` call (see the module docs above). `marker_ui` still
    /// gets no color, so this template still picks its own.
    fn marker_ui(&self, ui: &mut Ui, ctx: MarkerContext) {
        let time = ui.input(|i| i.time) as f32;
        let effect = match ctx.kind {
            SteadyAnimation::Halo => Animation::halo,
            SteadyAnimation::Blink => Animation::blink,
            SteadyAnimation::Orbit => Animation::orbit,
        };
        effect(ui.painter(), ctx.position, ctx.zoom, time, MARKER_COLOR);
        ui.ctx().request_repaint();
    }
}

// ---------------------------------------------------------------- node fire

fn fire_pulse(map: &mut Map, id: usize, at: Instant) {
    if let Some(node) = map.node(id) {
        node.color(PULSE_COLOR).pulse(at);
    }
}
fn fire_ripple(map: &mut Map, id: usize, at: Instant) {
    if let Some(node) = map.node(id) {
        node.color(RIPPLE_COLOR).ripple(at);
    }
}
fn fire_countdown(map: &mut Map, id: usize, at: Instant) {
    if let Some(node) = map.node(id) {
        node.color(COUNTDOWN_COLOR).countdown(at);
    }
}
fn fire_scale_in(map: &mut Map, id: usize, at: Instant) {
    if let Some(node) = map.node(id) {
        node.color(SCALE_IN_COLOR).scale_in(at);
    }
}
fn fire_crosshair(map: &mut Map, id: usize, at: Instant) {
    if let Some(node) = map.node(id) {
        node.color(CROSSHAIR_COLOR).crosshair(at);
    }
}

// ------------------------------------------------------------- repeaters

/// Re-fires the same node event effect on a timer of its own -- same
/// struct as `examples/animations.rs` uses.
struct NodeRepeater {
    node_id: usize,
    period: Duration,
    last_fired: Instant,
    fire: fn(&mut Map, usize, Instant),
}

impl NodeRepeater {
    fn new(
        node_id: usize,
        period: Duration,
        first_delay: Duration,
        fire: fn(&mut Map, usize, Instant),
    ) -> Self {
        Self {
            node_id,
            period,
            last_fired: Instant::now() - period + first_delay,
            fire,
        }
    }

    fn tick(&mut self, map: &mut Map, now: Instant) {
        if now.duration_since(self.last_fired) < self.period {
            return;
        }
        (self.fire)(map, self.node_id, now);
        self.last_fired = now;
    }
}

fn main() -> eframe::Result<()> {
    // 1. Nodes, arranged in a ring so the network reads clearly on screen.
    let mut points: HashMap<usize, MapPoint> = HashMap::new();
    for (id, name, x, y) in [
        (1, "Pulse", 200.0, 0.0),
        (2, "Ripple", 173.2, 100.0),
        (3, "Countdown", 100.0, 173.2),
        (4, "ScaleIn", 0.0, 200.0),
        (5, "Crosshair", -100.0, 173.2),
        (6, "Halo", -173.2, 100.0),
        (7, "Blink", -200.0, 0.0),
        (8, "Orbit", -173.2, -100.0),
        (9, "MarkerA", -100.0, -173.2),
        (10, "MarkerB", 0.0, -200.0),
        (11, "Hub1", 100.0, -173.2),
        (12, "Hub2", 173.2, -100.0),
    ] {
        let mut point = MapPoint::new(id, [x, y]);
        point.set_name(name.to_string());
        points.insert(id, point);
    }

    // 2. Edges: the full ring, plus three chords across it -- most nodes end
    // up with two or three neighbors instead of the isolated demo pairs
    // `animations.rs` uses.
    for (line_id, endpoints) in [
        ((1, 2), [1, 2]),
        ((2, 3), [2, 3]),
        ((3, 4), [3, 4]),
        ((4, 5), [4, 5]),
        ((5, 6), [5, 6]),
        ((6, 7), [6, 7]),
        ((7, 8), [7, 8]),
        ((8, 9), [8, 9]),
        ((9, 10), [9, 10]),
        ((10, 11), [10, 11]),
        ((11, 12), [11, 12]),
        ((12, 1), [12, 1]),
        ((1, 7), [1, 7]),
        ((3, 9), [3, 9]),
        ((5, 11), [5, 11]),
    ] {
        for id in endpoints {
            points.get_mut(&id).unwrap().connections.push(line_id);
        }
    }

    let mut map = Map::new();
    map.add_hashmap_points(points);
    map.add_lines(vec![
        MapSegment::new((1, 2), [200.0, 0.0], [173.2, 100.0]),
        MapSegment::new((2, 3), [173.2, 100.0], [100.0, 173.2]),
        MapSegment::new((3, 4), [100.0, 173.2], [0.0, 200.0]),
        MapSegment::new((4, 5), [0.0, 200.0], [-100.0, 173.2]),
        MapSegment::new((5, 6), [-100.0, 173.2], [-173.2, 100.0]),
        MapSegment::new((6, 7), [-173.2, 100.0], [-200.0, 0.0]),
        MapSegment::new((7, 8), [-200.0, 0.0], [-173.2, -100.0]),
        MapSegment::new((8, 9), [-173.2, -100.0], [-100.0, -173.2]),
        MapSegment::new((9, 10), [-100.0, -173.2], [0.0, -200.0]),
        MapSegment::new((10, 11), [0.0, -200.0], [100.0, -173.2]),
        MapSegment::new((11, 12), [100.0, -173.2], [173.2, -100.0]),
        MapSegment::new((12, 1), [173.2, -100.0], [200.0, 0.0]),
        MapSegment::new((1, 7), [200.0, 0.0], [-200.0, 0.0]),
        MapSegment::new((3, 9), [100.0, 173.2], [-100.0, -173.2]),
        MapSegment::new((5, 11), [-100.0, 173.2], [100.0, -173.2]),
    ]);

    // 3. Segments are not templated, so they keep the widget's own default
    // rendering and effect dispatch -- cycle the four persistent segment
    // effects across the edges so the network looks alive.
    for (id, effect) in [
        ((1, 2), 0),
        ((2, 3), 1),
        ((3, 4), 2),
        ((4, 5), 3),
        ((5, 6), 0),
        ((6, 7), 1),
        ((7, 8), 2),
        ((8, 9), 3),
        ((9, 10), 0),
        ((10, 11), 1),
        ((11, 12), 2),
        ((12, 1), 3),
        ((1, 7), 0),
        ((3, 9), 1),
        ((5, 11), 2),
    ] {
        let segment = map.segment(id).expect("edge is loaded");
        match effect {
            0 => segment.comet(),
            1 => segment.dash(),
            2 => segment.glow_band(),
            _ => segment.chevrons(),
        }
    }

    // 4. Lasting node state and markers -- `DiamondNodes::marker_ui` now
    // dispatches on `kind`, so Halo/Blink/Orbit each render as themselves
    // (see the module docs).
    map.node(6).expect("Halo is loaded").halo();
    map.node(7).expect("Blink is loaded").blink();
    map.node(8).expect("Orbit is loaded").orbit();
    map.update_marker(0, 9);
    map.update_marker(1, 10);

    map.set_node_template(Rc::new(DiamondNodes));
    // Show node names on hover so selection_ui gets called.
    map.settings.node_text_visibility = VisibilitySetting::Hover;

    // 5. Event effects: each on its own independent, non-synchronized timer,
    // each with the dedicated color `DiamondNodes::notification_ui` uses to
    // pick which built-in effect to reuse.
    let mut node_repeaters = vec![
        NodeRepeater::new(1, Duration::from_millis(2200), Duration::ZERO, fire_pulse),
        NodeRepeater::new(
            2,
            Duration::from_millis(2600),
            Duration::from_millis(400),
            fire_ripple,
        ),
        NodeRepeater::new(
            3,
            Duration::from_millis(3000),
            Duration::from_millis(900),
            fire_countdown,
        ),
        NodeRepeater::new(
            4,
            Duration::from_millis(1800),
            Duration::from_millis(200),
            fire_scale_in,
        ),
        NodeRepeater::new(
            5,
            Duration::from_millis(2400),
            Duration::from_millis(700),
            fire_crosshair,
        ),
    ];

    eframe::run_ui_native(
        "egui-map: NodeTemplate reusing the built-in animations",
        eframe::NativeOptions::default(),
        move |ui, _frame| {
            let now = Instant::now();
            for repeater in &mut node_repeaters {
                repeater.tick(&mut map, now);
            }
            ui.add(&mut map);
        },
    )
}

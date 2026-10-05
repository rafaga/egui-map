//! Custom rendering example: install a `NodeTemplate` to draw your own node
//! shape, and let the widget do the rest.
//!
//! Only two things are written here: [`NodeTemplate::node_ui`], which paints
//! the node, and [`NodeTemplate::outline`], which says what shape it has. The
//! hit area (`Map::hovered_node`), the selection ring, the notification
//! effects (`pulse`, `ripple`, ...) and the marker are the defaults of
//! `NodeTemplate`, drawn along that outline -- so they follow the circle drawn
//! below, at any zoom, without a line of code for each.
//!
//! - **A notification that lasts.** `NodeHandle::lasting(duration)` makes an
//!   event effect repeat every cycle for `duration`, fading out progressively,
//!   instead of playing once. Beta is notified for a few seconds every so
//!   often.
//! - **A marker.** `Map::update_marker` puts the default lasting effect on
//!   Gamma (it blinks around the node's outline).
//!
//! Every hook has a default except `node_ui`: override `selection_ui`,
//! `notification_ui`, `marker_ui` or `contains` only to draw something else.
//! `examples/node_template_animations.rs` has a node of another shape, and
//! `examples/svg_template.rs` hand-writes those hooks.
//!
//! Run with: cargo run --example custom_template

use eframe::egui::{self, Align2, Ui, Vec2};
use egui_map::map::Map;
use egui_map::map::objects::{
    HitContext, MapPoint, NodeContext, NodeOutline, NodeTemplate, VisibilitySetting,
};
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Radius of a node before the zoom: the circle `node_ui` paints and the one
/// `outline` declares.
const NODE_RADIUS: f32 = 8.0;

/// How long Beta's notification lasts each time, and how often it starts.
const NOTIFY_FOR: Duration = Duration::from_secs(6);
const NOTIFY_EVERY: Duration = Duration::from_secs(10);

struct CircleNodes;

impl NodeTemplate for CircleNodes {
    /// Custom node shape: a circle with the node name above it, filled with
    /// `ctx.color` -- the node's own color override if it set one, otherwise
    /// the active theme's node color.
    fn node_ui(&self, ui: &mut Ui, ctx: NodeContext) {
        let radius = NODE_RADIUS * ctx.zoom;
        let painter = ui.painter();
        painter.circle_filled(ctx.position, radius, ctx.color);
        painter.text(
            ctx.position + Vec2::new(0.0, -radius),
            Align2::CENTER_BOTTOM,
            ctx.point.get_name(),
            egui::FontId::proportional(11.0 * ctx.zoom),
            ctx.theme.text,
        );
    }

    /// The shape `node_ui` draws, in screen coordinates. Everything else --
    /// the hit area, the selection ring, the notifications and the marker --
    /// is derived from it by the default hooks.
    fn outline(&self, ctx: HitContext) -> NodeOutline {
        NodeOutline::Circle {
            center: ctx.position,
            radius: NODE_RADIUS * ctx.zoom,
        }
    }
}

fn main() -> eframe::Result<()> {
    let mut points = Vec::new();
    for (id, name, x, y) in [
        (1, "Alpha", 0.0, 0.0),
        (2, "Beta", 100.0, 50.0),
        (3, "Gamma", 50.0, -80.0),
    ] {
        let mut point = MapPoint::new(id, [x, y]);
        point.set_name(name.to_string());
        points.push(point);
    }

    let mut map = Map::new();
    map.add_points(points);
    map.set_node_template(Rc::new(CircleNodes));
    // Show node names on hover, so the selection ring is drawn too.
    map.settings.node_text_visibility = VisibilitySetting::Hover;
    map.update_marker(0, 3);

    // Start Beta's notification right away, then every `NOTIFY_EVERY`.
    let mut last_notified = Instant::now() - NOTIFY_EVERY;

    eframe::run_ui_native(
        "egui-map: custom template",
        eframe::NativeOptions::default(),
        move |ui, _frame| {
            if last_notified.elapsed() >= NOTIFY_EVERY {
                if let Some(node) = map.node(2) {
                    node.lasting(NOTIFY_FOR).pulse(Instant::now());
                }
                last_notified = Instant::now();
            }
            ui.add(&mut map);
        },
    )
}

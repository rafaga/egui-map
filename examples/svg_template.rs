//! SVG rendering example: a NodeTemplate that draws each node as an SVG icon
//! using egui's image loader pipeline (which rasterizes and caches the
//! textures automatically).
//!
//! The template writes two methods: [`NodeTemplate::node_ui`], which paints
//! the icon and its name, and [`NodeTemplate::outline`], which says the node
//! is the rounded square around them. The selection ring, the marker and the
//! notification effects are the defaults of `NodeTemplate`, drawn along that
//! outline, and `Map::hovered_node` uses it as the hit area -- here, to show a
//! tooltip.
//!
//! - **A notification that lasts.** `NodeHandle::lasting` makes the pulse on
//!   switch-01 repeat and fade out for a few seconds every so often.
//! - **A region label.** `Map::add_region_labels` adds a `RegionLabel` behind
//!   the network: unlike node names, its size scales *with* zoom, and it is
//!   always painted first, so it reads as a backdrop naming the whole rack
//!   rather than competing with the icons and lines drawn over it.
//!
//! Run with: cargo run --example svg_template

use eframe::egui::{self, Align2, Vec2};
use egui_map::map::Map;
use egui_map::map::objects::{
    HitContext, MapPoint, MapSegment, NodeContext, NodeOutline, NodeTemplate, RegionLabel,
};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Side of the icon, before the zoom.
const ICON_SIZE: f32 = 24.0;
/// How long the notification on switch-01 lasts each time, and how often it
/// starts.
const NOTIFY_FOR: Duration = Duration::from_secs(5);
const NOTIFY_EVERY: Duration = Duration::from_secs(8);

struct SvgNodes;

impl NodeTemplate for SvgNodes {
    /// Custom node shape: an SVG icon with the node name below it. The icon
    /// has its own fixed colors, so this template has no use for `ctx.color`
    /// -- see `custom_template.rs` for one that paints with it.
    fn node_ui(&self, ui: &mut egui::Ui, ctx: NodeContext) {
        let size = ICON_SIZE * ctx.zoom;
        let source = match ctx.point.get_id() {
            1 => egui::include_image!("router_pool.svg"),
            2 => egui::include_image!("switch_pool.svg"),
            _ => egui::include_image!("server_mango.svg"),
        };
        let rect = egui::Rect::from_center_size(ctx.position, Vec2::splat(size));
        ui.put(
            rect,
            egui::Image::new(source).fit_to_exact_size(Vec2::splat(size)),
        );
        ui.painter().text(
            ctx.position + Vec2::new(0.0, size / 2.0),
            Align2::CENTER_TOP,
            ctx.point.get_name(),
            egui::FontId::proportional(11.0 * ctx.zoom),
            ctx.theme.text,
        );
    }

    /// The square the icon fills, with rounded corners. The selection ring,
    /// the marker and the notifications are derived from it by the default
    /// hooks.
    fn outline(&self, ctx: HitContext) -> NodeOutline {
        let size = ICON_SIZE * ctx.zoom;
        NodeOutline::RoundedRect {
            rect: egui::Rect::from_center_size(ctx.position, Vec2::splat(size)),
            corner_radius: 4.0 * ctx.zoom,
        }
    }
}

fn main() -> eframe::Result<()> {
    // 1. The nodes, keyed by id. `names` is kept apart for the tooltip.
    let mut points: HashMap<usize, MapPoint> = HashMap::new();
    let mut names: HashMap<usize, &str> = HashMap::new();
    for (id, name, x, y) in [
        (1, "router-01", 0.0, 0.0),
        (2, "switch-01", 100.0, 50.0),
        (3, "Zeus", 100.0, 130.0),
    ] {
        let mut point = MapPoint::new(id, [x, y]);
        point.set_name(name.to_string());
        points.insert(id, point);
        names.insert(id, name);
    }

    // 2. Register each connection id on BOTH endpoint nodes, and build the
    //    line geometry keyed by the same id.
    let mut segments = Vec::new();
    for line_id in [(1, 2), (2, 3)] {
        for endpoint in [line_id.0, line_id.1] {
            points
                .get_mut(&endpoint)
                .expect("the endpoint is one of the nodes above")
                .connections
                .push(line_id);
        }
        let (from, to) = (points[&line_id.0].coords, points[&line_id.1].coords);
        segments.push(MapSegment::new(line_id, from, to));
    }

    // 3. Load the nodes, then the lines, the label and the template.
    let mut map = Map::new();
    map.add_hashmap_points(points);
    map.add_lines(segments);
    map.add_region_labels(vec![RegionLabel {
        text: "Rack A".to_string(),
        center: egui::pos2(60.0, 60.0), // roughly the centroid of the three nodes above
        color: None,                    // default: the active theme's text color, faded
    }]);
    map.set_node_template(Rc::new(SvgNodes));
    map.update_marker(0, 3);

    // Start the notification on node 2 right away, then every `NOTIFY_EVERY`.
    let mut last_notified = Instant::now() - NOTIFY_EVERY;
    let mut loaders_installed = false;

    eframe::run_ui_native(
        "egui-map: svg template",
        eframe::NativeOptions::default(),
        move |ui, _frame| {
            if !loaders_installed {
                // Installs the SVG loader (among others); idempotent.
                egui_extras::install_image_loaders(ui.ctx());
                loaders_installed = true;
            }
            if last_notified.elapsed() >= NOTIFY_EVERY {
                if let Some(node) = map.node(2) {
                    node.lasting(NOTIFY_FOR).pulse(Instant::now());
                }
                last_notified = Instant::now();
            }
            let response = ui.add(&mut map);
            // The node under the pointer: a tooltip with its name and id.
            if let Some(id) = map.hovered_node() {
                let name = names.get(&id).copied().unwrap_or("unknown");
                response.on_hover_text_at_pointer(format!("{name} (node {id})"));
            }
        },
    )
}

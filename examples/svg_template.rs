//! SVG rendering example: a NodeTemplate that draws each node as an SVG icon
//! using egui's image loader pipeline (which rasterizes and caches the
//! textures automatically), plus a marker and a repeating notification pulse.
//!
//! Run with: cargo run --example svg_template

use eframe::egui::{self, Align2, Color32, Stroke, Ui, Vec2};
use egui_map::map::Map;
use egui_map::map::objects::{
    MapPoint, MapSegment, MarkerContext, NodeContext, NodeTemplate, NotificationContext,
    SelectionContext, VisibilitySetting,
};
use std::rc::Rc;
use std::time::Instant;

struct SvgNodes;

impl NodeTemplate for SvgNodes {
    /// Custom node shape: an SVG icon with the node name below it. The icon
    /// has its own fixed colors, so this template has no use for `ctx.color`
    /// -- see `custom_template.rs` for one that paints with it.
    fn node_ui(&self, ui: &mut Ui, ctx: NodeContext) {
        let size = 24.0 * ctx.zoom;
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
            ui.visuals().text_color(),
        );
    }

    /// Highlight ring over the node closest to the mouse pointer, outlined in
    /// `ctx.color` -- the active theme's selection color.
    fn selection_ui(&self, ui: &mut Ui, ctx: SelectionContext) {
        ui.painter().circle_stroke(
            ctx.position,
            16.0 * ctx.zoom,
            Stroke::new(2.0 * ctx.zoom, ctx.color),
        );
    }

    /// Animated notification: an expanding ring that fades out over 2 seconds.
    fn notification_ui(&self, ui: &mut Ui, ctx: NotificationContext) -> bool {
        let secs = ctx.initial_time.elapsed().as_secs_f32();
        let alpha = (1.0 - secs / 2.0).clamp(0.0, 1.0);
        let fading = Color32::from_rgba_unmultiplied(
            ctx.color.r(),
            ctx.color.g(),
            ctx.color.b(),
            (255.0 * alpha) as u8,
        );
        ui.painter().circle_stroke(
            ctx.position,
            (16.0 + 30.0 * secs) * ctx.zoom,
            Stroke::new(3.0 * ctx.zoom, fading),
        );
        ui.ctx().request_repaint(); // keep the animation frames coming
        secs < 2.0 // returning false removes the notification
    }

    /// Static marker ring drawn over the marked node.
    fn marker_ui(&self, ui: &mut Ui, ctx: MarkerContext) {
        ui.painter().circle_stroke(
            ctx.position,
            20.0 * ctx.zoom,
            Stroke::new(2.0 * ctx.zoom, Color32::LIGHT_GREEN),
        );
    }
}

fn main() -> eframe::Result<()> {
    let mut points = Vec::new();
    for (id, name, x, y) in [
        (1, "router-01", 0.0, 0.0),
        (2, "switch-01", 100.0, 50.0),
        (3, "Zeus", 100.0, 130.0),
    ] {
        let mut point = MapPoint::new(id, [x, y]);
        point.set_name(name.to_string());
        match point.get_id() {
            1 => point.connections.push((0, 1)),
            2 => {
                point.connections.push((0, 1));
                point.connections.push((1, 2));
            }
            _ => point.connections.push((1, 2)),
        }
        points.push(point);
    }
    let mut vec_segmnents = Vec::new();
    let ids = vec![[1, 2], [2, 3]];
    let mut cont = 0;
    for id in ids {
        let point1 = points.get(id[0]).unwrap();
        let point2 = points.get(id[1]).unwrap();
        let line = MapSegment::new((cont, cont + 1), point1.coords, point2.coords);
        vec_segmnents.push(line);
        cont += 1;
    }
    let mut map = Map::new();
    map.add_points(points);
    map.add_lines(vec_segmnents);
    map.set_node_template(Rc::new(SvgNodes));
    // Show node names on hover so selection_ui gets called.
    map.settings.node_text_visibility = VisibilitySetting::Hover;
    map.update_marker(0, 3);

    // Re-trigger the notification on node 2 every 3 seconds.
    let mut last_pulse = Instant::now() - std::time::Duration::from_secs(3);
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
            if last_pulse.elapsed().as_secs() >= 3 {
                if let Some(node) = map.node(2) {
                    node.pulse(Instant::now());
                }
                last_pulse = Instant::now();
            }
            ui.add(&mut map);
        },
    )
}

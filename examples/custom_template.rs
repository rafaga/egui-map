//! Custom rendering example: install a NodeTemplate to draw your own node
//! shapes and animations, plus a marker and a repeating notification pulse.
//!
//! Run with: cargo run --example custom_template

use eframe::egui::{self, Align2, Color32, Pos2, Stroke, Ui, Vec2};
use egui_map::map::Map;
use egui_map::map::objects::{MapPoint, NodeTemplate, RawPoint, VisibilitySetting};
use std::rc::Rc;
use std::time::Instant;

struct CircleNodes;

impl NodeTemplate for CircleNodes {
    /// Custom node shape: a blue circle with the node name above it.
    /// `position` is the node's screen position; scale every size by `zoom`.
    fn node_ui(&self, ui: &mut Ui, position: Pos2, zoom: f32, point: &MapPoint) {
        let radius = 8.0 * zoom;
        let painter = ui.painter();
        painter.circle_filled(position, radius, Color32::from_rgb(80, 160, 255));
        painter.text(
            position + Vec2::new(0.0, -radius),
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
            11.0 * zoom,
            Stroke::new(2.0 * zoom, Color32::YELLOW),
        );
    }

    /// Animated notification: an expanding ring that fades out over 2 seconds.
    fn notification_ui(
        &self,
        ui: &mut Ui,
        position: Pos2,
        zoom: f32,
        start: Instant,
        color: Color32,
    ) -> bool {
        let secs = start.elapsed().as_secs_f32();
        let alpha = (1.0 - secs / 2.0).clamp(0.0, 1.0);
        let fading =
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), (255.0 * alpha) as u8);
        ui.painter().circle_stroke(
            position,
            (8.0 + 30.0 * secs) * zoom,
            Stroke::new(3.0 * zoom, fading),
        );
        ui.ctx().request_repaint(); // keep the animation frames coming
        secs < 2.0 // returning false removes the notification
    }

    /// Static marker ring drawn over the marked node.
    fn marker_ui(&self, ui: &mut Ui, position: Pos2, zoom: f32) {
        ui.painter().circle_stroke(
            position,
            14.0 * zoom,
            Stroke::new(2.0 * zoom, Color32::LIGHT_GREEN),
        );
    }
}

fn main() -> eframe::Result<()> {
    let mut points = Vec::new();
    for (id, name, x, y) in [
        (1, "Alpha", 0.0, 0.0),
        (2, "Beta", 100.0, 50.0),
        (3, "Gamma", 50.0, -80.0),
    ] {
        let mut point = MapPoint::new(id, RawPoint::new(x, y));
        point.set_name(name.to_string());
        points.push(point);
    }

    let mut map = Map::new();
    map.add_points(points);
    map.set_node_template(Rc::new(CircleNodes));
    // Show node names on hover so selection_ui gets called.
    map.settings.node_text_visibility = VisibilitySetting::Hover;
    map.update_marker(0, 3);

    // Re-trigger the notification on node 2 every 3 seconds.
    let mut last_pulse = Instant::now() - std::time::Duration::from_secs(3);

    eframe::run_ui_native(
        "egui-map: custom template",
        eframe::NativeOptions::default(),
        move |ui, _frame| {
            if last_pulse.elapsed().as_secs() >= 3 {
                map.notify(2, Instant::now());
                last_pulse = Instant::now();
            }
            ui.add(&mut map);
        },
    )
}

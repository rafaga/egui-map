//! Tutorial 2: a map with a `NodeTemplate`.
//!
//! The same five systems as `examples/tutorial_map.rs`, but each node is a
//! rounded box with its name inside instead of a dot with the name beside it.
//! A `NodeTemplate` is how you draw the nodes yourself. Read it top to
//! bottom, the steps are numbered; the README has the same tutorial in prose.
//!
//! Run with: cargo run --example tutorial_node_template

use eframe::egui::{self, Align2, CornerRadius, FontId, Pos2, Rect, Stroke, StrokeKind, Ui, Vec2};
use egui_map::map::Map;
use egui_map::map::objects::VisibilitySetting;
use egui_map::map::objects::{
    HitContext, MapPoint, MapSegment, NodeContext, NodeOutline, NodeTemplate,
};
use std::rc::Rc;

/// Step 1: the template. It is a type of your own that implements
/// `NodeTemplate`; it can hold whatever settings your nodes need. The size
/// below is what a box measures at the standard zoom (100%); every method
/// multiplies it by the zoom, so the boxes grow and shrink with the map.
struct BoxNodes {
    size: Vec2,
    corner_radius: f32,
}

impl BoxNodes {
    /// The box of a node whose centre is at `position`, at `zoom`.
    ///
    /// `node_ui` paints this rectangle and `outline` declares it, so they can
    /// not disagree about where the node is. This is a method and not a free
    /// function for the same reason as everything else: it needs the size.
    fn rect(&self, position: Pos2, zoom: f32) -> Rect {
        Rect::from_center_size(position, self.size * zoom)
    }
}

impl NodeTemplate for BoxNodes {
    /// Step 2: paint the node -- the only method you have to write.
    ///
    /// `ctx.position` is the node's centre on the screen, already placed and
    /// scaled; `ctx.zoom` is the scale to apply to every size you draw. The
    /// colors come from the active theme, so the nodes follow it when it
    /// changes: `ctx.color` is the node's own color (or the theme's node
    /// color if it has none) and `ctx.theme` is the whole palette.
    ///
    /// Once a template is installed the widget no longer writes node names:
    /// this is where they are drawn.
    fn node_ui(&self, ui: &mut Ui, ctx: NodeContext) {
        let rect = self.rect(ctx.position, ctx.zoom);
        let radius = CornerRadius::same((self.corner_radius * ctx.zoom) as u8);
        let painter = ui.painter();
        painter.rect_filled(rect, radius, ctx.theme.background);
        painter.rect_stroke(
            rect,
            radius,
            Stroke::new(1.5 * ctx.zoom, ctx.color),
            StrokeKind::Inside,
        );
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            ctx.point.get_name(),
            FontId::proportional(11.0 * ctx.zoom),
            ctx.theme.text,
        );
    }

    /// Step 3: say what shape the node has. This is optional but worth the
    /// three lines: from the outline the widget derives where the pointer
    /// hits the node (so `Map::hovered_node` and clicks work), the selection
    /// ring and the shape the notification effects wrap around.
    fn outline(&self, ctx: HitContext) -> NodeOutline {
        NodeOutline::RoundedRect {
            rect: self.rect(ctx.position, ctx.zoom),
            corner_radius: self.corner_radius * ctx.zoom,
        }
    }
}

fn main() -> eframe::Result<()> {
    // The data is built exactly as in the first tutorial: nodes with a unique
    // id, a name and a position, and lines whose id is listed in the
    // `connections` of both of their nodes. A template changes how the nodes
    // look, not how they are loaded.
    let mut points = Vec::new();
    for (id, name, x, y) in [
        (1, "Alpha", 0.0, 0.0),
        (2, "Beta", 120.0, 40.0),
        (3, "Gamma", 60.0, -90.0),
        (4, "Delta", -110.0, -30.0),
        (5, "Epsilon", -40.0, 100.0),
    ] {
        let mut point = MapPoint::new(id, [x, y]);
        point.set_name(name.to_string());
        points.push(point);
    }
    let mut segments = Vec::new();
    for line_id in [(1, 2), (1, 3), (1, 4), (4, 5)] {
        let mut ends = [[0.0; 2]; 2];
        for (end, node_id) in [line_id.0, line_id.1].into_iter().enumerate() {
            let point = points
                .iter_mut()
                .find(|point| point.get_id() == node_id)
                .expect("the line joins two of the nodes above");
            point.connections.push(line_id);
            ends[end] = point.coords;
        }
        segments.push(MapSegment::new(line_id, ends[0], ends[1]));
    }

    let mut map = Map::new();
    map.add_points(points);
    map.add_lines(segments);

    // Step 4: install the template. From now on the map calls it to draw
    // every node that is in view. The map keeps it behind an `Rc`.
    map.set_node_template(Rc::new(BoxNodes {
        size: Vec2::new(72.0, 22.0),
        corner_radius: 6.0,
    }));
    // The template draws the names, so `node_text_visibility` -- the rule for
    // the widget's own names -- has nothing left to decide: it stays out of
    // the way. Hover and selection still work, through the outline.
    map.settings.node_text_visibility = VisibilitySetting::Hover;

    // Step 5: show it, exactly as in the first tutorial.
    eframe::run_ui_native(
        "egui-map: tutorial 2, a map with a NodeTemplate",
        eframe::NativeOptions::default(),
        move |ui, _frame| {
            egui::CentralPanel::default().show(ui, |ui| {
                ui.add(&mut map);
            });
        },
    )
}

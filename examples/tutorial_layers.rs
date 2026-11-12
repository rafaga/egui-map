//! Tutorial 5: layers.
//!
//! A layer draws something the map knows nothing about -- here a disc under
//! each node and a ring around the one under the pointer -- at a place of the
//! paint order you choose. Read it top to bottom, the steps are numbered.
//! `docs/tutorial-5-layers.md` has the same tutorial in prose.
//!
//! Run with: cargo run --example tutorial_layers

use eframe::egui;
use egui::{Color32, Painter, Stroke};
use egui_map::map::Map;
use egui_map::map::layer::{LayerContext, LayerSlot, LayerTemplate};
use egui_map::map::objects::{MapPoint, SELECTION_GAP, VisibilitySetting};
use std::cell::Cell;
use std::rc::Rc;

/// 1. A layer is a type implementing `LayerTemplate`. This one paints a soft
///    disc under every node that is on screen. `ctx.points` are those nodes,
///    each with its position in *screen* coordinates, so you only multiply
///    sizes by `ctx.zoom`.
struct Discs {
    /// Radius in map units, shared with the sliders of the application.
    radius: Rc<Cell<f32>>,
}

impl LayerTemplate for Discs {
    fn layer_ui(&self, painter: &Painter, ctx: LayerContext) {
        for point in ctx.points {
            painter.circle_filled(
                point.position,
                self.radius.get() * ctx.zoom,
                ctx.theme.node.gamma_multiply(0.2),
            );
        }
    }
}

/// 2. A ring over everything around the node under the pointer. Every
///    `LayerPoint` has the `outline` of its node (the shape its template
///    reports, a circle when there is none), so the ring follows the shape.
struct Ring {
    hovered: Rc<Cell<Option<usize>>>,
}

impl LayerTemplate for Ring {
    fn layer_ui(&self, painter: &Painter, ctx: LayerContext) {
        let Some(id) = self.hovered.get() else {
            return;
        };
        if let Some(point) = ctx.points.iter().find(|point| point.point.id == id) {
            painter.add(
                point
                    .outline
                    .grown(SELECTION_GAP)
                    .stroke_shape(Stroke::new(2.0, Color32::GOLD)),
            );
        }
    }
}

fn main() -> eframe::Result<()> {
    let mut map = Map::new();
    let mut points = Vec::new();
    for id in 1..=12_usize {
        let angle = id as f32 * 2.399_963;
        let radius = 40.0 * (id as f32).sqrt();
        let mut point = MapPoint::new(id, [radius * angle.cos(), radius * angle.sin()]);
        point.set_name(format!("Node {id}"));
        points.push(point);
    }
    map.add_points(points);
    map.settings.node_text_visibility = VisibilitySetting::Hover;

    // 3. Install them. The slot says where in the paint order a layer goes
    //    (`BelowLines`, `BelowNodes`, `AboveNodes`); layers of one slot are
    //    painted by ascending `z`. The map keeps no data of a layer: share it
    //    with `Rc<Cell<..>>` as here.
    let radius = Rc::new(Cell::new(35.0));
    let hovered = Rc::new(Cell::new(None));
    map.add_layer(
        "discs",
        LayerSlot::BelowLines,
        0,
        Rc::new(Discs {
            radius: radius.clone(),
        }),
    );
    map.add_layer(
        "ring",
        LayerSlot::AboveNodes,
        0,
        Rc::new(Ring {
            hovered: hovered.clone(),
        }),
    );

    let mut show_discs = true;
    eframe::run_ui_native(
        "egui-map: tutorial 5, layers",
        eframe::NativeOptions::default(),
        move |ui, _frame| {
            egui::Panel::top("controls").show(ui, |ui| {
                ui.horizontal(|ui| {
                    // 4. Layers can be hidden without removing them.
                    if ui.checkbox(&mut show_discs, "Discs").changed() {
                        map.set_layer_visible("discs", show_discs);
                    }
                    let mut value = radius.get();
                    if ui
                        .add(egui::Slider::new(&mut value, 10.0..=80.0).text("radius"))
                        .changed()
                    {
                        radius.set(value);
                    }
                });
            });
            egui::CentralPanel::default().show(ui, |ui| {
                ui.add(&mut map);
                hovered.set(map.hovered_node());
            });
        },
    )
}

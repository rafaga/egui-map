//! Layers example: three `LayerTemplate`s drawing things the map knows
//! nothing about, in three different places of its paint order, over a network
//! that mixes straight and curved connections.
//!
//! - **Ownership** (`LayerSlot::BelowLines`): a soft disc around every node,
//!   colored by who owns it. Painted under the connection lines.
//! - **Heat** (`LayerSlot::BelowNodes`): a radial glow per node, from blue
//!   (quiet) to red (busy), built as one `Mesh` per frame. Painted over the
//!   lines and under the nodes.
//! - **Heat tint** (`LayerSlot::OverNodeBody`): the same data as a tint inside
//!   each node's opaque box, between its background and its name. Only a
//!   `NodeTemplate` can place it there, by calling `ctx.layers.paint(..)`.
//! - **Focus ring** (`LayerSlot::AboveNodes`): a ring around the node under
//!   the pointer, over everything.
//!
//! A layer keeps its own data, shared with the application through an
//! `Rc<RefCell<..>>`: the map never sees it. Toggle each layer with the
//! checkboxes (`Map::set_layer_visible`) and press "New data" to change the
//! heat without touching the map.
//!
//! Run with: cargo run --example layers

use eframe::egui;
use egui::epaint::Mesh;
use egui::{Align2, Color32, FontId, Painter, Pos2, Shape, Stroke, Ui, vec2};
use egui_map::map::Map;
use egui_map::map::layer::{LayerContext, LayerNodeContext, LayerSlot, LayerTemplate};
use egui_map::map::objects::{
    HitContext, MapPoint, MapSegment, NodeContext, NodeOutline, NodeTemplate, SELECTION_GAP,
};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

/// A color ramp from quiet to busy.
struct HeatRamp {
    stops: [Color32; 4],
}

impl Default for HeatRamp {
    fn default() -> Self {
        Self {
            stops: [
                Color32::from_rgb(40, 90, 200),
                Color32::from_rgb(40, 200, 170),
                Color32::from_rgb(240, 210, 60),
                Color32::from_rgb(230, 50, 40),
            ],
        }
    }
}

impl HeatRamp {
    /// The color at `heat`, from 0.0 to 1.0.
    fn color_at(&self, heat: f32) -> Color32 {
        let scaled = heat.clamp(0.0, 1.0) * (self.stops.len() - 1) as f32;
        let index = (scaled.floor() as usize).min(self.stops.len() - 2);
        self.stops[index].lerp_to_gamma(self.stops[index + 1], scaled - index as f32)
    }
}

/// The heat of every node, from 0.0 to 1.0, shared by the two heat layers.
type SharedHeat = Rc<RefCell<HashMap<usize, f32>>>;

/// A radial glow per node.
struct HeatHalo {
    heat: SharedHeat,
    ramp: HeatRamp,
    /// Radius of a glow, in map units.
    radius: Rc<Cell<f32>>,
    max_alpha: u8,
}

impl LayerTemplate for HeatHalo {
    fn layer_ui(&self, painter: &Painter, ctx: LayerContext) {
        const SEGMENTS: u32 = 16;
        let heat = self.heat.borrow();
        let mut mesh = Mesh::default();
        for point in ctx.points {
            let level = heat.get(&point.point.id).copied().unwrap_or(0.0);
            if level <= 0.0 {
                continue;
            }
            let radius = self.radius.get() * ctx.zoom * (0.4 + 0.6 * level);
            let color = self.ramp.color_at(level);
            let centre = Color32::from_rgba_unmultiplied(
                color.r(),
                color.g(),
                color.b(),
                (f32::from(self.max_alpha) * level) as u8,
            );
            let rim = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 0);
            let first = mesh.vertices.len() as u32;
            mesh.colored_vertex(point.position, centre);
            for step in 0..SEGMENTS {
                let angle = std::f32::consts::TAU * step as f32 / SEGMENTS as f32;
                mesh.colored_vertex(
                    point.position + vec2(angle.cos(), angle.sin()) * radius,
                    rim,
                );
            }
            for step in 0..SEGMENTS {
                mesh.add_triangle(first, first + 1 + step, first + 1 + (step + 1) % SEGMENTS);
            }
        }
        painter.add(Shape::mesh(mesh));
    }

    /// The tint inside the node's box (`LayerSlot::OverNodeBody`): the same
    /// data, drawn in the shape the node template reported.
    fn node_ui(&self, painter: &Painter, ctx: LayerNodeContext) {
        let level = self
            .heat
            .borrow()
            .get(&ctx.point.id)
            .copied()
            .unwrap_or(0.0);
        if level > 0.0 {
            let color = self.ramp.color_at(level).gamma_multiply(0.55);
            painter.rect_filled(ctx.outline.bounding_rect(), 5.0 * ctx.zoom, color);
        }
    }
}

/// A soft disc per node, in the color of its owner.
struct Ownership {
    owners: HashMap<usize, Color32>,
}

impl LayerTemplate for Ownership {
    fn layer_ui(&self, painter: &Painter, ctx: LayerContext) {
        for point in ctx.points {
            if let Some(color) = self.owners.get(&point.point.id) {
                painter.circle_filled(point.position, 70.0 * ctx.zoom, color.gamma_multiply(0.18));
            }
        }
    }
}

/// A ring around the node the pointer is over, following the shape of the
/// node: a rounded box for the chips of this example, a circle for a round
/// node.
struct FocusRing {
    hovered: Rc<Cell<Option<usize>>>,
}

impl LayerTemplate for FocusRing {
    fn layer_ui(&self, painter: &Painter, ctx: LayerContext) {
        let Some(id) = self.hovered.get() else {
            return;
        };
        if let Some(point) = ctx.points.iter().find(|point| point.point.id == id) {
            painter.add(
                point
                    .outline
                    .grown(SELECTION_GAP)
                    .stroke_shape(Stroke::new(2.0, ctx.theme.selected)),
            );
        }
    }
}

/// An opaque box with the node's name, so the nodes hide what is under them
/// and the heat has to be drawn inside (`LayerSlot::OverNodeBody`).
struct Chip;

impl Chip {
    fn outline_at(position: Pos2, zoom: f32) -> NodeOutline {
        NodeOutline::RoundedRect {
            rect: egui::Rect::from_center_size(position, vec2(62.0, 22.0) * zoom),
            corner_radius: 5.0 * zoom,
        }
    }
}

impl NodeTemplate for Chip {
    fn node_ui(&self, ui: &mut Ui, ctx: NodeContext) {
        let outline = Self::outline_at(ctx.position, ctx.zoom);
        ui.painter().rect_filled(
            outline.bounding_rect(),
            5.0 * ctx.zoom,
            ctx.background_color,
        );
        // The layers go here: over the background, under the name.
        ctx.layers.paint(ui.painter(), &outline);
        ui.painter().text(
            ctx.position,
            Align2::CENTER_CENTER,
            ctx.point.get_name(),
            FontId::proportional((11.0 * ctx.zoom).max(6.0)),
            ctx.theme.text,
        );
    }

    fn outline(&self, ctx: HitContext) -> NodeOutline {
        Self::outline_at(ctx.position, ctx.zoom)
    }
}

/// The example's data, derived from a seed so "New data" can change it.
struct Scene;

impl Scene {
    const COLUMNS: usize = 6;
    const ROWS: usize = 4;

    fn heat(seed: u32) -> HashMap<usize, f32> {
        (0..Self::COLUMNS * Self::ROWS)
            .map(|index| {
                let wave = ((index as u32 * 7919 + seed * 104_729) % 1000) as f32 / 1000.0;
                (index + 1, if wave < 0.35 { 0.0 } else { wave })
            })
            .collect()
    }

    fn owner(index: usize) -> Color32 {
        match index % Self::COLUMNS {
            0..=1 => Color32::from_rgb(70, 130, 230),
            2..=3 => Color32::from_rgb(230, 150, 50),
            _ => Color32::from_rgb(170, 90, 210),
        }
    }

    fn map() -> Map {
        let mut points: HashMap<usize, MapPoint> = HashMap::new();
        for index in 0..Self::COLUMNS * Self::ROWS {
            let (column, row) = (index % Self::COLUMNS, index / Self::COLUMNS);
            let stagger = if row % 2 == 0 { 0.0 } else { 38.0 };
            let mut point = MapPoint::new(
                index + 1,
                [column as f32 * 100.0 + stagger, row as f32 * 90.0],
            );
            point.set_name(format!("Sys {}", index + 1));
            points.insert(index + 1, point);
        }
        // A grid of connections; every third one is a curve, the others are
        // straight. Both kinds share the map.
        let mut segments = Vec::new();
        for index in 0..Self::COLUMNS * Self::ROWS {
            let (column, row) = (index % Self::COLUMNS, index / Self::COLUMNS);
            let mut neighbours = Vec::new();
            if column + 1 < Self::COLUMNS {
                neighbours.push(index + 1);
            }
            if row + 1 < Self::ROWS {
                neighbours.push(index + Self::COLUMNS);
            }
            for other in neighbours {
                let line = (index + 1, other + 1);
                for end in [line.0, line.1] {
                    points
                        .get_mut(&end)
                        .expect("both ends are nodes of the grid")
                        .connections
                        .push(line);
                }
                let (from, to) = (points[&line.0].coords, points[&line.1].coords);
                segments.push(if (line.0 + line.1) % 3 == 0 {
                    MapSegment::curved(line, from, to, None)
                } else {
                    MapSegment::straight(line, from, to)
                });
            }
        }
        let mut map = Map::new();
        map.add_hashmap_points(points);
        map.add_lines(segments);
        map.settings.line_visible_zoom = 0.0;
        map
    }
}

fn main() -> eframe::Result<()> {
    let mut map = Scene::map();
    let heat: SharedHeat = Rc::new(RefCell::new(Scene::heat(1)));
    let radius = Rc::new(Cell::new(75.0));
    let hovered = Rc::new(Cell::new(None));

    map.set_node_template(Rc::new(Chip));
    let owners = (0..Scene::COLUMNS * Scene::ROWS)
        .map(|index| (index + 1, Scene::owner(index)))
        .collect();
    map.add_layer(
        "ownership",
        LayerSlot::BelowLines,
        0,
        Rc::new(Ownership { owners }),
    );
    // One template, two slots: the halo under the nodes and the tint inside
    // their boxes read the same data.
    let halo = Rc::new(HeatHalo {
        heat: Rc::clone(&heat),
        ramp: HeatRamp::default(),
        radius: Rc::clone(&radius),
        max_alpha: 190,
    });
    map.add_layer("heat", LayerSlot::BelowNodes, 0, halo.clone());
    map.add_layer("heat-tint", LayerSlot::OverNodeBody, 0, halo);
    map.add_layer(
        "focus",
        LayerSlot::AboveNodes,
        0,
        Rc::new(FocusRing {
            hovered: Rc::clone(&hovered),
        }),
    );

    let mut seed = 1;
    eframe::run_ui_native(
        "egui-map: layers",
        eframe::NativeOptions::default(),
        move |ui, _frame| {
            egui::Panel::top("controls").show(ui, |ui| {
                ui.horizontal(|ui| {
                    for (id, label) in [
                        ("ownership", "Ownership"),
                        ("heat", "Heat halo"),
                        ("heat-tint", "Heat tint"),
                        ("focus", "Focus ring"),
                    ] {
                        let mut shown = map.layer_visible(id).unwrap_or(false);
                        if ui.checkbox(&mut shown, label).changed() {
                            map.set_layer_visible(id, shown);
                        }
                    }
                    ui.separator();
                    let mut value = radius.get();
                    if ui
                        .add(egui::Slider::new(&mut value, 20.0..=160.0).text("Halo radius"))
                        .changed()
                    {
                        radius.set(value);
                    }
                    if ui.button("New data").clicked() {
                        seed += 1;
                        *heat.borrow_mut() = Scene::heat(seed);
                    }
                    ui.separator();
                    ui.add(
                        egui::Slider::new(&mut map.settings.default_curvature, -0.6..=0.6)
                            .text("Curvature"),
                    );
                });
            });
            egui::CentralPanel::default().show(ui, |ui| {
                ui.add(&mut map);
                hovered.set(map.hovered_node());
            });
        },
    )
}

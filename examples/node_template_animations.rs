//! A small network drawn with a custom `NodeTemplate`: hexagonal nodes with
//! their name inside, a glowing node and a segment with "marching ants".
//!
//! Six nodes: four form a ring, and the two others hang from one node of the
//! ring each, on opposite sides of it.
//!
//! - **A hexagon.** [`NodeTemplate::node_ui`] paints it and
//!   [`NodeTemplate::outline`] declares it as a
//!   [`NodeOutline::Polygon`], so the hit area (`Map::hovered_node`) and the
//!   selection ring follow the hexagon, at any zoom.
//! - **A glow.** `NodeContext::marker` says how present a `Map::update_marker`
//!   marker is on the node, fading in and out on its own. `node_ui` hands it
//!   to `Animation::glow_outline` over a larger hexagon, painted behind the
//!   node. The marker is the glow, so `marker_ui` draws nothing else.
//! - **Marching ants.** One of the ring's segments is dashed, and its pattern
//!   slides along it (`Map::segment(..).dash()`). Segments are not templated:
//!   they use the widget's own rendering.
//! - **The basics of `examples/basic.rs`.** A combo box with the built-in
//!   themes next to egui's light/dark/system buttons (`Map::set_theme`), a
//!   tooltip on the node under the pointer (`Map::hovered_node`) and a button
//!   that sets and clears the marker.
//!
//! See `examples/custom_template.rs` for the shortest template, which leaves
//! the selection and the marker to the defaults.
//!
//! Run with: cargo run --example node_template_animations

use eframe::egui::{self, Align2, Pos2, Shape, Stroke, Ui, Vec2};
use egui_map::map::Map;
use egui_map::map::objects::{
    HitContext, MapPoint, MapSegment, MarkerContext, NodeContext, NodeOutline, NodeTemplate,
};
use egui_map::map::theme::Theme;
use std::collections::HashMap;
use std::rc::Rc;

/// Every built-in theme, in the order the combo box lists them.
const THEMES: [Theme; 15] = [
    Theme::SystemDefault,
    Theme::SlateOcean,
    Theme::NebulaViolet,
    Theme::TerminalGreen,
    Theme::EmberForge,
    Theme::SolarAmber,
    Theme::ArticCyan,
    Theme::CrimsonSignal,
    Theme::MidnightIndigo,
    Theme::CopperRose,
    Theme::LimeCircuit,
    Theme::CoralReef,
    Theme::GraphiteMono,
    Theme::PlumStatic,
    Theme::SandstoneTrail,
];

/// Distance from the center of a hexagon to its corners, before the zoom.
const HEX_RADIUS: f32 = 26.0;
/// How much larger than the node the glowing hexagon behind it is.
const GLOW_SCALE: f32 = 1.45;

/// Id of the marker the button below toggles, and the node it points to.
const MARKER: usize = 0;
const MARKED_NODE: usize = 5;

/// The corners of a flat-topped hexagon, in screen coordinates.
fn hexagon(center: Pos2, radius: f32) -> Vec<Pos2> {
    (0..6)
        .map(|corner| {
            let angle = std::f32::consts::FRAC_PI_3 * corner as f32;
            center + Vec2::angled(angle) * radius
        })
        .collect()
}

struct HexNodes;

impl NodeTemplate for HexNodes {
    /// A hexagon with the node name inside, outlined in `ctx.color` -- the
    /// node's own color override if it set one, otherwise the active theme's
    /// node color -- over the theme's background.
    ///
    /// While a marker points here, a larger hexagon glows behind it, with
    /// `ctx.marker` -- `0.0..=1.0`, fading in and out as the marker is set or
    /// cleared -- as the strength of the effect.
    fn node_ui(&self, ui: &mut Ui, ctx: NodeContext) {
        let radius = HEX_RADIUS * ctx.zoom;
        if ctx.marker > 0.0 {
            let time = ui.input(|input| input.time) as f32;
            let outline = NodeOutline::Polygon(hexagon(ctx.position, radius * GLOW_SCALE));
            ctx.animation
                .glow_outline(ui.painter(), &outline, time, ctx.theme.marker, ctx.marker);
            ui.ctx().request_repaint();
        }
        let painter = ui.painter();
        painter.add(Shape::convex_polygon(
            hexagon(ctx.position, radius),
            ctx.theme.background,
            Stroke::new(2.0 * ctx.zoom, ctx.color),
        ));
        painter.text(
            ctx.position,
            Align2::CENTER_CENTER,
            ctx.point.get_name(),
            egui::FontId::proportional(11.0 * ctx.zoom),
            ctx.theme.text,
        );
    }

    /// The hexagon `node_ui` draws, border included. The hit area and the
    /// selection ring are derived from it by the default hooks.
    fn outline(&self, ctx: HitContext) -> NodeOutline {
        NodeOutline::Polygon(hexagon(ctx.position, HEX_RADIUS * ctx.zoom))
    }

    /// The glow in `node_ui` is the marker, so there is no ring on top of it.
    fn marker_ui(&self, _ui: &mut Ui, _ctx: MarkerContext) {}
}

fn main() -> eframe::Result<()> {
    // 1. The nodes, keyed by id: the ring (1 to 4) and the two nodes hanging
    //    from it (5 and 6), one on each side. `names` is kept apart for the
    //    tooltip.
    let mut points: HashMap<usize, MapPoint> = HashMap::new();
    let mut names: HashMap<usize, &str> = HashMap::new();
    for (id, name, x, y) in [
        (1, "Alpha", 0.0, -100.0),
        (2, "Beta", 100.0, 0.0),
        (3, "Gamma", 0.0, 100.0),
        (4, "Delta", -100.0, 0.0),
        (5, "Echo", -250.0, 0.0),
        (6, "Foxtrot", 250.0, 0.0),
    ] {
        let mut point = MapPoint::new(id, [x, y]);
        point.set_name(name.to_string());
        points.insert(id, point);
        names.insert(id, name);
    }

    // 2. Register each connection id on BOTH endpoint nodes, and build the
    //    line geometry keyed by the same id: the ring, then the two spurs.
    let mut segments = Vec::new();
    for line_id in [(1, 2), (2, 3), (3, 4), (4, 1), (4, 5), (2, 6)] {
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

    // 3. Load the nodes, then the lines, and install the template.
    let mut map = Map::new();
    map.add_hashmap_points(points);
    map.add_lines(segments);
    map.set_node_template(Rc::new(HexNodes));

    // A lasting segment effect: the pattern of the dashed line keeps sliding
    // along it ("marching ants").
    map.segment((1, 2))
        .expect("Alpha <-> Beta is loaded")
        .dash();

    // The glow starts on: a marker on Echo.
    map.update_marker(MARKER, MARKED_NODE);

    let mut theme = Theme::default();
    let mut marked = true;

    eframe::run_ui_native(
        "egui-map: hexagonal nodes",
        eframe::NativeOptions::default(),
        move |ui, _frame| {
            egui::Panel::top("controls").show(ui, |ui| {
                ui.horizontal(|ui| {
                    egui::ComboBox::from_label("Theme")
                        .selected_text(format!("{theme:?}"))
                        .show_ui(ui, |ui| {
                            for choice in THEMES {
                                let label = format!("{choice:?}");
                                if ui.selectable_value(&mut theme, choice, label).changed() {
                                    map.set_theme(Rc::new(choice));
                                }
                            }
                        });
                    // Light, dark or the system's: the map resolves its colors
                    // for whichever mode egui is in.
                    egui::widgets::global_theme_preference_buttons(ui);
                    ui.separator();
                    let label = if marked {
                        "Clear the glow"
                    } else {
                        "Make Echo glow"
                    };
                    if ui.button(label).clicked() {
                        // `remove_marker` returns the node the marker pointed
                        // to, or `None` when there was none: then set it.
                        marked = map.remove_marker(MARKER).is_none();
                        if marked {
                            map.update_marker(MARKER, MARKED_NODE);
                        }
                    }
                });
            });
            egui::CentralPanel::default().show(ui, |ui| {
                let response = ui.add(&mut map);
                // The node under the pointer, whatever the node text
                // visibility is: here, a tooltip with its name.
                if let Some(id) = map.hovered_node() {
                    let name = names.get(&id).copied().unwrap_or("unknown");
                    response.on_hover_text_at_pointer(format!("{name} (node {id})"));
                }
            });
        },
    )
}

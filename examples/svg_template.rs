//! SVG rendering example: a simulated computer network drawn with a
//! `NodeTemplate` that paints each node as an SVG icon (egui's image loader
//! rasterizes and caches the textures), with a path of "marching ants" from one
//! computer to another through the switches and routers between them.
//!
//! The network has three routers in a triangle, four switches and eight
//! computers. Pick the two computers in the combo boxes: the shortest path
//! between them is found and the segments it uses get a `dash` effect, sliding
//! from the first computer to the second one.
//!
//! - **The template** writes two methods: [`NodeTemplate::node_ui`], which
//!   paints the icon and its name, and [`NodeTemplate::outline`], which says
//!   the node is the rounded square around the icon. The selection ring and
//!   `Map::hovered_node`, which shows a tooltip here, follow that outline.
//! - **The direction of the ants.** A dash slides from the first endpoint of a
//!   segment to its second, so a hop of the path taken the other way is
//!   started with `Map::segment(..).direction(CometDirection::Reverse)`.
//!   Each time the path changes, the old effects are cleared and the new ones
//!   started (`Map::segment(..).dash()`).
//! - **The rest of the window** is the one of `examples/basic.rs`: a combo box
//!   with the built-in themes next to egui's light/dark/system buttons
//!   (`Map::set_theme`).
//!
//! Run with: cargo run --example svg_template

use eframe::egui::{self, Align2, Vec2};
use egui_map::map::Map;
use egui_map::map::objects::{
    CometDirection, HitContext, MapPoint, MapSegment, NodeContext, NodeOutline, NodeTemplate,
};
use egui_map::map::theme::Theme;
use std::collections::{HashMap, HashSet, VecDeque};
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

/// Side of the icon, before the zoom.
const ICON_SIZE: f32 = 24.0;

/// What a node is, which decides its icon.
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Router,
    Switch,
    Computer,
}

/// Id, name, kind and position of every node.
const NODES: [(usize, &str, Kind, f32, f32); 15] = [
    (1, "R1", Kind::Router, -100.0, -10.0),
    (2, "R2", Kind::Router, 100.0, -10.0),
    (3, "R3", Kind::Router, 0.0, 60.0),
    (4, "SW1", Kind::Switch, -150.0, -70.0),
    (5, "SW2", Kind::Switch, 150.0, -70.0),
    (6, "SW3", Kind::Switch, 0.0, 120.0),
    (7, "SW4", Kind::Switch, 190.0, 10.0),
    (8, "PC-A", Kind::Computer, -215.0, -125.0),
    (9, "PC-B", Kind::Computer, -110.0, -130.0),
    (10, "PC-C", Kind::Computer, 100.0, -130.0),
    (11, "PC-D", Kind::Computer, 195.0, -130.0),
    (12, "PC-E", Kind::Computer, 235.0, -10.0),
    (13, "PC-F", Kind::Computer, 235.0, 55.0),
    (14, "PC-G", Kind::Computer, -65.0, 175.0),
    (15, "PC-H", Kind::Computer, 65.0, 175.0),
];

/// The cables, each one as the id of its segment: the two nodes it joins.
const LINKS: [(usize, usize); 15] = [
    // The core: three routers in a triangle.
    (1, 2),
    (2, 3),
    (3, 1),
    // A switch under each router, and one more under SW2.
    (1, 4),
    (2, 5),
    (3, 6),
    (5, 7),
    // The computers.
    (4, 8),
    (4, 9),
    (5, 10),
    (5, 11),
    (7, 12),
    (7, 13),
    (6, 14),
    (6, 15),
];

struct NetworkNodes;

impl NodeTemplate for NetworkNodes {
    /// Custom node shape: an SVG icon with the node name below it. The icons
    /// have their own fixed colors, so this template has no use for
    /// `ctx.color` -- see `custom_template.rs` for one that paints with it.
    fn node_ui(&self, ui: &mut egui::Ui, ctx: NodeContext) {
        let size = ICON_SIZE * ctx.zoom;
        let source = match kind_of(ctx.point.get_id()) {
            Kind::Router => egui::include_image!("router_pool.svg"),
            Kind::Switch => egui::include_image!("switch_pool.svg"),
            Kind::Computer => egui::include_image!("server_mango.svg"),
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

    /// The square the icon fills, with rounded corners. The selection ring is
    /// derived from it by the default hook.
    fn outline(&self, ctx: HitContext) -> NodeOutline {
        let size = ICON_SIZE * ctx.zoom;
        NodeOutline::RoundedRect {
            rect: egui::Rect::from_center_size(ctx.position, Vec2::splat(size)),
            corner_radius: 4.0 * ctx.zoom,
        }
    }
}

fn kind_of(id: usize) -> Kind {
    NODES
        .iter()
        .find(|node| node.0 == id)
        .map_or(Kind::Computer, |node| node.2)
}

/// The shortest path from `from` to `to`, both included, or an empty `Vec`
/// when there is none.
fn shortest_path(from: usize, to: usize) -> Vec<usize> {
    let mut came_from: HashMap<usize, usize> = HashMap::new();
    let mut seen = HashSet::from([from]);
    let mut queue = VecDeque::from([from]);
    while let Some(node) = queue.pop_front() {
        if node == to {
            let mut path = vec![to];
            while let Some(&previous) = came_from.get(path.last().expect("it has `to`")) {
                path.push(previous);
            }
            path.reverse();
            return path;
        }
        for &(a, b) in &LINKS {
            let next = match node {
                n if n == a => b,
                n if n == b => a,
                _ => continue,
            };
            if seen.insert(next) {
                came_from.insert(next, node);
                queue.push_back(next);
            }
        }
    }
    Vec::new()
}

/// Clears the marching ants of the previous path and starts them on the
/// segments of `path`, sliding the way the data goes.
fn show_path(map: &mut Map, path: &[usize]) {
    for id in LINKS {
        if let Some(segment) = map.segment(id) {
            segment.clear();
        }
    }
    for hop in path.windows(2) {
        let (a, b) = (hop[0], hop[1]);
        // A segment is stored as `(first, second)`, and its ants slide from
        // the first to the second: a hop taken the other way runs `Reverse`.
        let (id, direction) = if LINKS.contains(&(a, b)) {
            ((a, b), CometDirection::Forward)
        } else {
            ((b, a), CometDirection::Reverse)
        };
        map.segment(id)
            .expect("the path follows the links")
            .direction(direction)
            .dash();
    }
}

fn main() -> eframe::Result<()> {
    // 1. The nodes, keyed by id. `names` is kept apart for the tooltip and the
    //    combo boxes.
    let mut points: HashMap<usize, MapPoint> = HashMap::new();
    let mut names: HashMap<usize, &str> = HashMap::new();
    for (id, name, _, x, y) in NODES {
        let mut point = MapPoint::new(id, [x, y]);
        point.set_name(name.to_string());
        points.insert(id, point);
        names.insert(id, name);
    }

    // 2. Register each connection id on BOTH endpoint nodes, and build the
    //    line geometry keyed by the same id.
    let mut segments = Vec::new();
    for line_id in LINKS {
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
    map.set_node_template(Rc::new(NetworkNodes));

    let computers: Vec<usize> = NODES
        .iter()
        .filter(|node| node.2 == Kind::Computer)
        .map(|node| node.0)
        .collect();
    let (mut from, mut to) = (8, 12);
    let mut path = shortest_path(from, to);
    show_path(&mut map, &path);

    let mut theme = Theme::default();
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
                    let mut changed = false;
                    for (label, endpoint) in [("From", &mut from), ("To", &mut to)] {
                        ui.label(label);
                        egui::ComboBox::from_id_salt(label)
                            .selected_text(names[endpoint])
                            .show_ui(ui, |ui| {
                                for &id in &computers {
                                    changed |=
                                        ui.selectable_value(endpoint, id, names[&id]).changed();
                                }
                            });
                    }
                    if changed {
                        path = shortest_path(from, to);
                        show_path(&mut map, &path);
                    }
                });
                let route: Vec<&str> = path.iter().map(|id| names[id]).collect();
                ui.label(if route.is_empty() {
                    "No path".to_string()
                } else {
                    route.join(" -> ")
                });
            });
            egui::CentralPanel::default().show(ui, |ui| {
                let response = ui.add(&mut map);
                // The node under the pointer: a tooltip with its name and id.
                if let Some(id) = map.hovered_node() {
                    let name = names.get(&id).copied().unwrap_or("unknown");
                    response.on_hover_text_at_pointer(format!("{name} (node {id})"));
                }
            });
        },
    )
}

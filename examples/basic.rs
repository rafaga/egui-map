//! Basic egui-map example: nodes, connection lines and the widget's built-in
//! style, plus the three things an application usually reaches for first.
//!
//! - **Themes.** [`Map::set_theme`] swaps the palette live. The map follows
//!   the light/dark mode egui is in, so the three buttons next to the combo
//!   box (system, dark, light) change the look of every theme at once.
//! - **A tooltip on some nodes.** [`Map::hovered_node`] says which node the
//!   pointer was over in the last frame, so a tooltip can be attached to the
//!   `Response` the widget returns.
//! - **A marker.** [`Map::update_marker`] puts one on a node and
//!   [`Map::remove_marker`] takes it away. The marker fades in and out on its
//!   own instead of switching on and off.
//! - **A segment animation.** The line between Alpha and Beta is a dashed
//!   line whose pattern slides along it ("marching ants"), started with
//!   `Map::segment(..).dash()`. See `examples/animations.rs` for the whole
//!   catalog of node and segment effects.
//! - **A segment animation.** The line between Alpha and Beta is a dashed
//!   line whose pattern slides along it ("marching ants"), started with
//!   `Map::segment(..).dash()`. See `examples/animations.rs` for the whole
//!   catalog of node and segment effects.
//!
//! Run with: cargo run --example basic

use eframe::egui;
use egui_map::map::Map;
use egui_map::map::objects::{MapPoint, MapSegment};
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

/// Id of the marker the button below toggles, and the node it points to.
const MARKER: usize = 0;
const MARKED_NODE: usize = 2;

fn main() -> eframe::Result<()> {
    // 1. The nodes, keyed by id. `names` is kept apart for the tooltip.
    let mut points: HashMap<usize, MapPoint> = HashMap::new();
    let mut names: HashMap<usize, &str> = HashMap::new();
    for (id, name, x, y) in [
        (1, "Alpha", 0.0, 0.0),
        (2, "Beta", 100.0, 50.0),
        (3, "Gamma", 50.0, -80.0),
    ] {
        let mut point = MapPoint::new(id, [x, y]);
        point.set_name(name.to_string());
        points.insert(id, point);
        names.insert(id, name);
    }

    // 2. Register each connection id on BOTH endpoint nodes, and build the
    //    line geometry keyed by the same id.
    let mut segments = Vec::new();
    for line_id in [(1, 2), (1, 3)] {
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

    // 3. Load the nodes, then the lines.
    let mut map = Map::new();
    map.add_hashmap_points(points);
    map.add_lines(segments);

    // A lasting segment effect: Alpha <-> Beta becomes a dashed line whose
    // pattern keeps sliding along it ("marching ants"), until `clear` is
    // called on the same handle.
    map.segment((1, 2))
        .expect("Alpha <-> Beta is loaded")
        .dash();

    let mut theme = Theme::default();
    let mut marked = false;

    eframe::run_ui_native(
        "egui-map: basic",
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
                        "Clear the marker"
                    } else {
                        "Mark Beta"
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

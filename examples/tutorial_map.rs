//! Tutorial 1: your first map.
//!
//! The smallest complete program that shows a map: five systems joined by
//! lines, pan with click & drag, zoom with the mouse wheel or the slider. Read
//! it top to bottom, the steps are numbered. `docs/tutorial-1-first-map.md` has
//! the same tutorial in prose; `examples/tutorial_node_template.rs` is the
//! next step, drawing the nodes yourself.
//!
//! Run with: cargo run --example tutorial_map

use eframe::egui;
use egui_map::map::Map;
use egui_map::map::objects::{MapPoint, MapSegment, VisibilitySetting};

fn main() -> eframe::Result<()> {
    // 1. Create the nodes. A node is a `MapPoint`: an id you choose (it has
    //    to be unique) and its position in *map coordinates* -- any units you
    //    like, the widget scales them with the zoom. The name is optional and
    //    is what the widget writes next to the node.
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

    // 2. Connect them. Every line has an id of its own -- by convention the
    //    pair of node ids it joins -- and that id must be listed in the
    //    `connections` of BOTH of its nodes. The line itself, a `MapSegment`,
    //    carries that id and the coordinates of its two ends.
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

    // 3. Load them into the map: the nodes first, then the lines.
    let mut map = Map::new();
    map.add_points(points);
    map.add_lines(segments);

    // 4. Adjust it, if you want. Everything lives in `map.settings`; the
    //    defaults are a good start, so every line here is optional.
    //
    //    - Node names show when the pointer is over the node (the default is
    //      `Always`, which also needs the map to be zoomed in a bit).
    //    - A line is drawn `style.line_width` pixels wide at the standard
    //      zoom (100%) and scales with it: twice as wide at 200%.
    //    - Lines are hidden below `line_visible_zoom` and fade in above it
    //      (`line_fade`); `LineFade::None` draws them always, fully opaque.
    //    - The minimap is off by default.
    map.settings.node_text_visibility = VisibilitySetting::Hover;
    map.settings.style.line_width = Some(2.0);
    map.settings.minimap.enabled = true;

    // 5. Show it. The map is a widget: add it to any `Ui`, on every frame.
    //    Keep the same `Map` between frames -- it holds the zoom and the
    //    position the user has panned to.
    eframe::run_ui_native(
        "egui-map: tutorial 1, your first map",
        eframe::NativeOptions::default(),
        move |ui, _frame| {
            egui::CentralPanel::default().show(ui, |ui| {
                ui.add(&mut map);
            });
        },
    )
}

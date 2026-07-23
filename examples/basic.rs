//! Basic egui-map example: nodes, connection lines and a label, all rendered
//! with the widget's built-in style.
//!
//! Run with: cargo run --example basic

use eframe::egui;
use egui_map::map::Map;
use egui_map::map::objects::{MapLabel, MapPoint, RawLine, RawPoint};
use std::collections::HashMap;

fn main() -> eframe::Result<()> {
    // 1. Create the nodes, keyed by id.
    let mut points = HashMap::new();
    for (id, name, x, y) in [
        (1, "Alpha", 0.0, 0.0),
        (2, "Beta", 100.0, 50.0),
        (3, "Gamma", 50.0, -80.0),
    ] {
        let mut point = MapPoint::new(id, RawPoint::new(x, y));
        point.set_name(name.to_string());
        points.insert(id, point);
    }

    // 2. Register each connection id on BOTH endpoint nodes.
    for (line_id, endpoints) in [("1-2", [1, 2]), ("1-3", [1, 3])] {
        for id in endpoints {
            points
                .get_mut(&id)
                .unwrap()
                .connections
                .push(line_id.to_string());
        }
    }

    // 3. Load the nodes, then the line geometry keyed by the same ids.
    let mut map = Map::new();
    map.add_hashmap_points(points);
    map.add_lines(HashMap::from([
        (
            "1-2".to_string(),
            RawLine::new(RawPoint::new(0.0, 0.0), RawPoint::new(100.0, 50.0)),
        ),
        (
            "1-3".to_string(),
            RawLine::new(RawPoint::new(0.0, 0.0), RawPoint::new(50.0, -80.0)),
        ),
    ]));

    // A free-floating label. Its position is in screen coordinates.
    map.add_labels(vec![MapLabel {
        text: "Example region".to_string(),
        center: egui::pos2(100.0, 100.0),
    }]);

    eframe::run_ui_native(
        "egui-map: basic",
        eframe::NativeOptions::default(),
        move |ui, _frame| {
            ui.add(&mut map);
        },
    )
}

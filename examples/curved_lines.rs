//! Curved connection lines example: straight and curved lines on the same
//! map, with the effects following the curve.
//!
//! - A line is straight or curved from the moment it is built:
//!   `MapSegment::straight` (what `MapSegment::new` is) or
//!   `MapSegment::curved(id, from, to, curvature)`.
//! - `curved(.., None)` bends by `MapSettings::default_curvature`, which the
//!   slider changes live; the straight lines never move.
//! - `curved(.., Some(amount))` has its own curvature, and
//!   `MapSegment::set_curvature` changes only that amount afterwards.
//! - Positive curvature bends to the right of the way from the first node to
//!   the second, negative to the left: two connections between the same pair
//!   of nodes (here, the pair on the right) can be told apart.
//! - The segment effects (dash, comet, glow band, chevrons) follow the curve.
//!
//! Run with: cargo run --example curved_lines

use eframe::egui;
use egui_map::map::Map;
use egui_map::map::objects::{MapPoint, MapSegment};
use std::collections::HashMap;

/// The example's map.
struct Network;

impl Network {
    fn map() -> Map {
        let nodes = [
            (1, "A", 0.0, 0.0),
            (2, "B", 160.0, 0.0),
            (3, "C", 0.0, 140.0),
            (4, "D", 160.0, 140.0),
            (5, "E", 0.0, 280.0),
            (6, "F", 160.0, 280.0),
            (7, "G", 340.0, 0.0),
            (8, "H", 340.0, 140.0),
        ];
        let mut points: HashMap<usize, MapPoint> = HashMap::new();
        for (id, name, x, y) in nodes {
            let mut point = MapPoint::new(id, [x, y]);
            point.set_name(name.to_string());
            points.insert(id, point);
        }
        // (from, to, curvature): `None` is a straight line, `Some(None)` a
        // curve with the default curvature, `Some(Some(x))` one of its own.
        let lines: [((usize, usize), Option<Option<f32>>); 8] = [
            ((1, 2), None),
            ((3, 4), Some(None)),
            ((5, 6), Some(Some(0.5))),
            ((1, 3), Some(Some(-0.3))),
            ((2, 4), Some(Some(0.3))),
            ((7, 8), Some(Some(0.35))),
            ((8, 7), Some(Some(0.35))),
            ((4, 8), None),
        ];
        let mut segments = Vec::new();
        for (line, shape) in lines {
            for end in [line.0, line.1] {
                points
                    .get_mut(&end)
                    .expect("both ends are nodes above")
                    .connections
                    .push(line);
            }
            let (from, to) = (points[&line.0].coords, points[&line.1].coords);
            segments.push(match shape {
                None => MapSegment::straight(line, from, to),
                Some(curvature) => MapSegment::curved(line, from, to, curvature),
            });
        }
        let mut map = Map::new();
        map.add_hashmap_points(points);
        map.add_lines(segments);
        map.settings.line_visible_zoom = 0.0;

        // The effects run along the curve, not along the chord.
        map.segment((3, 4)).expect("C <-> D is loaded").dash();
        map.segment((5, 6)).expect("E <-> F is loaded").comet();
        map.segment((2, 4)).expect("B <-> D is loaded").chevrons();
        map.segment((7, 8)).expect("G -> H is loaded").glow_band();
        map
    }
}

fn main() -> eframe::Result<()> {
    let mut map = Network::map();
    eframe::run_ui_native(
        "egui-map: curved lines",
        eframe::NativeOptions::default(),
        move |ui, _frame| {
            egui::Panel::top("controls").show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Default curvature (only C <-> D follows it)");
                    ui.add(egui::Slider::new(
                        &mut map.settings.default_curvature,
                        -1.0..=1.0,
                    ));
                });
            });
            egui::CentralPanel::default().show(ui, |ui| {
                ui.add(&mut map);
            });
        },
    )
}

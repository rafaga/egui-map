//! Demonstrates the built-in node and segment animations reached through
//! `Map::node` and `Map::segment` -- no custom template involved, just the
//! effects the widget already knows how to draw.
//!
//! - Node `Beta` carries a lasting `halo`, node `Delta` a lasting `orbit`,
//!   and the `Gamma <-> Delta` segment a lasting `comet` -- all visible
//!   immediately and forever, since lasting effects only stop when cleared.
//! - Every few seconds the example also fires a one-off event effect: it
//!   cycles `Alpha` through all five node effects (`pulse`, `ripple`,
//!   `countdown`, `scale_in`, `crosshair`) and flashes the
//!   `Alpha <-> Beta` segment at the same time, so every built-in animation
//!   shows up in turn.
//!
//! Run with: cargo run --example animations

use egui_map::map::Map;
use egui_map::map::objects::MapPoint;
use egui_map::map::objects::MapSegment;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// One event effect per step of the cycle described above.
fn fire_node_event(map: &mut Map, step: usize, at: Instant) {
    let Some(node) = map.node(1) else { return };
    match step % 5 {
        0 => node.pulse(at),
        1 => node.ripple(at),
        2 => node.countdown(at),
        3 => node.scale_in(at),
        _ => node.crosshair(at),
    }
}

fn main() -> eframe::Result<()> {
    // 1. Nodes, keyed by id.
    let mut points: HashMap<usize, MapPoint> = HashMap::new();
    for (id, name, x, y) in [
        (1, "Alpha", 0.0, 0.0),
        (2, "Beta", 120.0, 40.0),
        (3, "Gamma", 60.0, -90.0),
        (4, "Delta", -80.0, -40.0),
    ] {
        let mut point = MapPoint::new(id, [x, y]);
        point.set_name(name.to_string());
        points.insert(id, point);
    }

    // 2. Register each connection id on both endpoint nodes.
    for (line_id, endpoints) in [((1, 2), [1, 2]), ((3, 4), [3, 4])] {
        for id in endpoints {
            points.get_mut(&id).unwrap().connections.push(line_id);
        }
    }

    let mut map = Map::new();
    map.add_hashmap_points(points);
    map.add_lines(vec![
        MapSegment::new((1, 2), [0.0, 0.0], [120.0, 40.0]),
        MapSegment::new((3, 4), [60.0, -90.0], [-80.0, -40.0]),
    ]);

    // Lasting effects: set once, visible for as long as the app runs.
    map.node(2).expect("Beta is loaded").halo();
    map.node(4).expect("Delta is loaded").orbit();
    map.segment((3, 4))
        .expect("Gamma <-> Delta is loaded")
        .comet();

    // One-off effects: re-triggered on a timer, cycling through the five
    // node effects. The segment flash rides along on the same timer.
    let mut step = 0usize;
    let mut last_fired = Instant::now() - Duration::from_secs(3);

    eframe::run_ui_native(
        "egui-map: node and segment animations",
        eframe::NativeOptions::default(),
        move |ui, _frame| {
            if last_fired.elapsed().as_secs_f32() >= 2.5 {
                let now = Instant::now();
                fire_node_event(&mut map, step, now);
                if let Some(segment) = map.segment((1, 2)) {
                    segment.flash(now);
                }
                step += 1;
                last_fired = now;
            }
            ui.add(&mut map);
        },
    )
}

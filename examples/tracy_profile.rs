//! Demonstrates wiring the crate's `tracing` spans up to the
//! [Tracy](https://github.com/wolfpld/tracy) profiler.
//!
//! `egui-map` itself never installs a subscriber -- as a library it just
//! emits `tracing::info_span!` spans on its hot paths (painting, viewport
//! culling, point/line loading, ...), which are zero-cost no-ops until
//! *something* in the final binary installs a subscriber. This example is
//! that "something": it builds a `tracing_subscriber::registry()` with a
//! `tracing_tracy::TracyLayer` and installs it as the global default before
//! running the same widget as the `basic` example, so every span the crate
//! emits shows up as a zone in Tracy's timeline.
//!
//! Run with a Tracy capture window/GUI already listening, then:
//!
//!     cargo run --example tracy_profile --features tracy
//!
//! Pan and zoom the map to see `paint_map`, `calculate_visible_points`,
//! `capture_mouse_events` and friends show up as zones per frame.

use egui_map::map::Map;
use egui_map::map::objects::{MapPoint, MapSegment};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

fn main() -> eframe::Result<()> {
    // Install a Tracy-backed subscriber as the process-wide default. This is
    // the "consumer" side of the crate's instrumentation: `egui-map` only
    // ever calls `tracing::info_span!`, it never touches a subscriber
    // itself, so this has to happen in the binary that embeds the widget.
    tracing_subscriber::registry()
        .with(tracing_tracy::TracyLayer::default())
        .init();

    // Same node/line setup as the `basic` example.
    let mut points = Vec::new();
    for (id, name, x, y) in [
        (1, "Alpha", 0.0, 0.0),
        (2, "Beta", 100.0, 50.0),
        (3, "Gamma", 50.0, -80.0),
    ] {
        let mut point = MapPoint::new(id, [x, y]);
        point.set_name(name.to_string());
        points.push(point);
    }
    for (line_id, endpoints) in [((1, 2), [1, 2]), ((1, 3), [1, 3])] {
        for id in endpoints {
            points.get_mut(id).unwrap().connections.push(line_id);
        }
    }

    let mut map = Map::new();
    map.add_points(points);
    map.add_lines(vec![
        MapSegment::new((1, 2), [0.0, 0.0], [100.0, 50.0]),
        MapSegment::new((1, 3), [0.0, 0.0], [50.0, -80.0]),
    ]);

    eframe::run_ui_native(
        "egui-map: tracy profiling",
        eframe::NativeOptions::default(),
        move |ui, _frame| {
            ui.add(&mut map);
        },
    )
}

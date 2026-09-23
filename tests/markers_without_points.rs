//! A map whose nodes were never loaded (the data source is missing or still
//! being built) must still draw when it has markers. Up to 0.9.1 the marker
//! loop unwrapped the node map and panicked on the first frame.

use egui::{Context, RawInput, Rect, pos2, vec2};
use egui_map::map::Map;
use egui_map::map::objects::MapPoint;

/// Draws `map` once in a headless context.
fn draw(map: &mut Map) {
    let ctx = Context::default();
    let mut out = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(400.0, 300.0))),
            ..RawInput::default()
        },
        |ui| {
            ui.add(&mut *map);
        },
    );
    out.textures_delta.clear();
}

#[test]
fn a_marker_on_a_map_without_nodes_is_not_drawn() {
    let mut map = Map::new();
    map.update_marker(1, 42);
    draw(&mut map);
}

#[test]
fn a_marker_on_a_node_that_is_not_loaded_is_not_drawn() {
    let mut map = Map::new();
    map.add_points(vec![
        MapPoint::new(1, [0.0, 0.0]),
        MapPoint::new(2, [10.0, 10.0]),
    ]);
    map.update_marker(1, 42);
    draw(&mut map);
}

#[test]
fn a_marker_waits_for_its_node_to_be_loaded() {
    let mut map = Map::new();
    map.update_marker(1, 2);
    draw(&mut map);

    map.add_points(vec![
        MapPoint::new(1, [0.0, 0.0]),
        MapPoint::new(2, [10.0, 10.0]),
    ]);
    draw(&mut map);
}

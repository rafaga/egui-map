//! Tests de integración para la API pública de `egui-map`.

use egui_map::map::Map;
use egui_map::map::objects::{
    MapLabel, MapPoint, MapSegment, MapSettings, RawPoint, VisibilitySetting,
};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

fn sample_points() -> HashMap<usize, MapPoint> {
    let mut map = HashMap::new();
    map.insert(1, MapPoint::new(1, RawPoint::new(0.0, 0.0)));
    map.insert(2, MapPoint::new(2, RawPoint::new(10.0, 10.0)));
    map.insert(3, MapPoint::new(3, RawPoint::new(-10.0, -10.0)));
    map
}

// ---------- flujo completo de Map (API pública) ----------

#[test]
fn map_initial_position_is_points_midpoint() {
    let mut map = Map::new();
    map.add_hashmap_points(sample_points());
    // el punto medio de (-10,-10)..(10,10) es el origen
    assert_eq!(map.get_pos(), [0.0, 0.0]);
}

#[test]
fn map_position_workflow() {
    let mut map = Map::new();
    map.add_hashmap_points(sample_points());

    map.set_pos_from_nodeid(2);
    assert_eq!(map.get_pos(), [10.0, 10.0]);

    map.set_pos([5.0, -5.0]);
    assert_eq!(map.get_pos(), [5.0, -5.0]);
}

#[test]
fn map_zoom_workflow() {
    let mut map = Map::new();
    assert_eq!(map.get_zoom(), 1.0);

    map.set_zoom(1.75);
    assert_eq!(map.get_zoom(), 1.75);

    // valores fuera de rango se ignoran
    map.set_zoom(99.0);
    assert_eq!(map.get_zoom(), 1.75);
    map.set_zoom(0.001);
    assert_eq!(map.get_zoom(), 1.75);
}

#[test]
fn map_add_labels_and_lines() {
    let mut map = Map::new();
    map.add_hashmap_points(sample_points());

    map.add_labels(vec![MapLabel {
        text: "The Forge".to_string(),
        center: egui::Pos2::new(3.0, 4.0),
    }]);

    let mut lines = Vec::new();
    lines.push(MapSegment::new(
        Rc::from("1-2"),
        RawPoint::new(0.0, 0.0),
        RawPoint::new(10.0, 10.0),
    ));
    map.add_lines(lines);
}

#[test]
fn map_notify_and_markers() {
    let mut map = Map::new();
    map.add_hashmap_points(sample_points());

    map.notify(1, Instant::now());
    map.update_marker(0, 2);
    map.update_marker(1, 3);
}

// ---------- tipos públicos desde fuera del crate ----------

#[test]
fn raw_point_arithmetic_from_outside_crate() {
    let a = RawPoint::new(1.0, 2.0);
    let b = RawPoint::new(3.0, 4.0);
    let sum = a + b;
    assert_eq!(sum.components, [4.0, 6.0]);

    let scaled = a * 2.0f32;
    assert_eq!(scaled.components, [2.0, 4.0]);

    let divided = b / 2.0f32;
    assert_eq!(divided.components, [1.5, 2.0]);
}

#[test]
fn raw_line_geometry_from_outside_crate() {
    let segment = MapSegment::new(
        Rc::from("test"),
        RawPoint::new(0.0, 0.0),
        RawPoint::new(6.0, 8.0),
    );
    assert_eq!(segment.raw_line.distance(), 10.0);
    assert_eq!(segment.raw_line.midpoint().components, [3.0, 4.0]);
}

#[test]
fn map_point_api_from_outside_crate() {
    let mut point = MapPoint::new(30000142, RawPoint::new(1.0, 2.0));
    assert_eq!(point.get_id(), 30000142);
    point.set_name("Jita".to_string());
    assert_eq!(point.get_name(), "Jita");
    point.connections.push("1-2".to_string());
    assert_eq!(point.connections.len(), 1);
}

#[test]
fn map_settings_default_from_outside_crate() {
    let settings = MapSettings::default();
    assert_eq!(settings.max_zoom, 2.0);
    assert_eq!(settings.min_zoom, 0.1);
    assert_eq!(settings.node_text_visibility, VisibilitySetting::Always);
    assert_eq!(settings.styles.len(), 2);
}

//! A map loaded with no nodes (a region of the SDE without systems) is
//! centered on the origin and can be drawn: its bounds used to stay at
//! infinity, and the midpoint of those is not a number.

use egui_map::map::Map;
use egui_map::map::objects::MapPoint;
use std::collections::HashMap;

#[test]
fn a_map_loaded_with_no_hashmap_nodes_is_centered_on_the_origin() {
    let mut map = Map::new();
    map.add_hashmap_points(HashMap::new());
    assert_eq!(map.get_pos(), [0.0, 0.0]);
}

#[test]
fn a_map_loaded_with_no_nodes_is_centered_on_the_origin() {
    let mut map = Map::new();
    map.add_points(Vec::new());
    assert_eq!(map.get_pos(), [0.0, 0.0]);
}

#[test]
fn an_empty_map_takes_nodes_after_being_loaded() {
    let mut map = Map::new();
    map.add_hashmap_points(HashMap::new());
    map.insert_point(MapPoint::new(1, [10.0, 20.0]));
    assert!(map.contains_point(1));
}

#[test]
fn a_map_reloaded_with_no_nodes_forgets_the_old_ones() {
    let mut map = Map::new();
    map.add_points(vec![MapPoint::new(1, [10.0, 20.0])]);
    map.add_points(Vec::new());
    assert!(!map.contains_point(1));
    assert_eq!(map.get_pos(), [0.0, 0.0]);
}

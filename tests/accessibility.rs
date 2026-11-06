//! What the map tells a screen reader, read back from the AccessKit tree that
//! egui builds for every frame.
//!
//! The tests run the map in an `egui_kittest` harness, which also checks on its
//! own, after every run, that no input widget on screen is left without an
//! accessible name. A control added to the map without a name fails every test
//! here, not just the one that is about it.

use egui::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use egui_map::map::Map;
use egui_map::map::objects::{MapPoint, MapSegment};
use egui_map::map::overlay::Corner;
use std::collections::HashMap;

/// A small map: three nodes and the two lines that join them.
fn small_map() -> Map {
    let mut points: HashMap<usize, MapPoint> = HashMap::new();
    let mut first = MapPoint::new(1, [0.0, 0.0]);
    first.connections.push((1, 2));
    let mut second = MapPoint::new(2, [100.0, 0.0]);
    second.connections.extend([(1, 2), (2, 3)]);
    let mut third = MapPoint::new(3, [100.0, 100.0]);
    third.connections.push((2, 3));
    points.insert(1, first);
    points.insert(2, second);
    points.insert(3, third);
    let mut map = Map::new();
    map.add_hashmap_points(points);
    map.add_hashmap_lines(HashMap::from([
        ((1, 2), MapSegment::new((1, 2), [0.0, 0.0], [100.0, 0.0])),
        (
            (2, 3),
            MapSegment::new((2, 3), [100.0, 0.0], [100.0, 100.0]),
        ),
    ]));
    map
}

fn harness(map: Map) -> Harness<'static, Map> {
    Harness::builder()
        .with_size(egui::vec2(600.0, 400.0))
        .build_ui_state(
            |ui, map: &mut Map| {
                ui.add(map);
            },
            map,
        )
}

#[test]
fn the_map_is_a_canvas_named_map() {
    let mut harness = harness(small_map());
    harness.run();

    let canvas = harness.get_by_role_and_label(Role::Canvas, "Map");
    assert!(!canvas.accesskit_node().is_hidden());
}

#[test]
fn the_zoom_slider_is_named_and_carries_the_zoom() {
    let mut map = small_map();
    map.set_zoom(1.5);
    let mut harness = harness(map);
    harness.run();

    let slider = harness.get_by_role_and_label(Role::Slider, "Zoom");
    assert_eq!(slider.accesskit_node().numeric_value(), Some(1.5));
}

#[test]
fn the_zoom_value_follows_the_zoom() {
    let mut harness = harness(small_map());
    harness.run();
    harness.state_mut().set_zoom(0.5);
    harness.run();

    let slider = harness.get_by_role_and_label(Role::Slider, "Zoom");
    assert_eq!(slider.accesskit_node().numeric_value(), Some(0.5));
}

#[test]
fn a_screen_reader_can_set_the_zoom_through_the_slider() {
    let mut harness = harness(small_map());
    harness.run();
    harness
        .get_by_role_and_label(Role::Slider, "Zoom")
        .set_value_accesskit(1.25);
    harness.run();

    assert!((harness.state_mut().get_zoom() - 1.25).abs() < 1e-6);
}

#[test]
fn a_screen_reader_can_step_the_zoom_up_and_down() {
    let mut harness = harness(small_map());
    harness.state_mut().set_zoom(1.0);
    harness.run();
    harness
        .get_by_role_and_label(Role::Slider, "Zoom")
        .increment_accesskit();
    harness.run();
    let raised = harness.state_mut().get_zoom();
    assert!(raised > 1.0, "{raised}");

    harness
        .get_by_role_and_label(Role::Slider, "Zoom")
        .decrement_accesskit();
    harness.run();
    assert!(harness.state_mut().get_zoom() < raised);
}

#[test]
fn a_hidden_slider_leaves_the_tree() {
    let mut map = small_map();
    map.settings.slider.visible = false;
    let mut harness = harness(map);
    harness.run();

    assert!(harness.query_by_role(Role::Slider).is_none());
    harness.get_by_role_and_label(Role::Canvas, "Map");
}

#[test]
fn the_minimap_is_in_the_tree_only_when_it_is_on() {
    let mut harness = harness(small_map());
    harness.run();
    assert!(harness.query_by_label("Minimap").is_none());

    harness.state_mut().settings.minimap.enabled = true;
    harness.run();
    harness.get_by_role_and_label(Role::Canvas, "Minimap");
    harness.get_by_role_and_label(Role::Canvas, "Map");
}

#[test]
fn every_control_can_be_renamed() {
    let mut map = small_map();
    map.settings.minimap.enabled = true;
    map.settings.accessibility.map = "Mapa de la galaxia".to_owned();
    map.settings.accessibility.zoom_slider = "Acercamiento".to_owned();
    map.settings.accessibility.minimap = "Minimapa".to_owned();
    let mut harness = harness(map);
    harness.run();

    harness.get_by_role_and_label(Role::Canvas, "Mapa de la galaxia");
    harness.get_by_role_and_label(Role::Slider, "Acercamiento");
    harness.get_by_role_and_label(Role::Canvas, "Minimapa");
    for default in ["Map", "Zoom", "Minimap"] {
        assert!(harness.query_by_label(default).is_none(), "{default}");
    }
}

#[test]
fn the_default_names_are_in_english() {
    let labels = egui_map::map::accessibility::AccessibilityLabels::default();
    assert_eq!(labels.map, "Map");
    assert_eq!(labels.zoom_slider, "Zoom");
    assert_eq!(labels.minimap, "Minimap");
}

#[test]
fn every_corner_and_orientation_keeps_the_slider_named() {
    for corner in [
        Corner::TopLeft,
        Corner::TopRight,
        Corner::BottomLeft,
        Corner::BottomRight,
    ] {
        for orientation in [
            egui::SliderOrientation::Vertical,
            egui::SliderOrientation::Horizontal,
        ] {
            let mut map = small_map();
            map.settings.slider.anchor = corner;
            map.settings.slider.orientation = orientation;
            map.settings.minimap.enabled = true;
            let mut harness = harness(map);
            harness.run();

            harness.get_by_role_and_label(Role::Slider, "Zoom");
        }
    }
}

#[test]
fn a_map_with_nothing_loaded_still_has_its_controls_named() {
    let mut harness = harness(Map::new());
    harness.run();

    harness.get_by_role_and_label(Role::Canvas, "Map");
    harness.get_by_role_and_label(Role::Slider, "Zoom");
}

/// A map and every node activation it reported, one entry per frame in which
/// there was one. `Harness::step` can run more than one frame, and the
/// activation lasts only one, so reading it afterwards would miss it.
struct Recorded {
    map: Map,
    activations: Vec<usize>,
}

fn recording(map: Map) -> Harness<'static, Recorded> {
    Harness::builder()
        .with_size(egui::vec2(600.0, 400.0))
        .build_ui_state(
            |ui, state: &mut Recorded| {
                ui.add(&mut state.map);
                if let Some(id) = state.map.activated_node() {
                    state.activations.push(id);
                }
            },
            Recorded {
                map,
                activations: Vec::new(),
            },
        )
}

/// Three nodes in a row, two of them named, on a 600 x 400 screen.
fn named_map() -> Map {
    let mut map = Map::new();
    let mut jita = MapPoint::new(1, [0.0, 0.0]);
    jita.name = Some("Jita".to_owned());
    let mut perimeter = MapPoint::new(2, [40.0, 0.0]);
    perimeter.name = Some("Perimeter".to_owned());
    map.add_points(vec![jita, perimeter, MapPoint::new(3, [-40.0, 0.0])]);
    map.set_pos([0.0, 0.0]);
    map.set_zoom(1.0);
    map
}

#[test]
fn every_node_on_screen_is_a_button_named_after_it() {
    let mut harness = harness(named_map());
    harness.run();

    harness.get_by_role_and_label(Role::Button, "Jita");
    harness.get_by_role_and_label(Role::Button, "Perimeter");
}

#[test]
fn a_node_without_a_name_is_called_node_and_its_id() {
    let mut harness = harness(named_map());
    harness.run();

    harness.get_by_role_and_label(Role::Button, "Node 3");
}

#[test]
fn the_name_of_an_unnamed_node_can_be_translated() {
    let mut map = named_map();
    map.settings.accessibility.unnamed_node = "Sistema".to_owned();
    let mut harness = harness(map);
    harness.run();

    harness.get_by_role_and_label(Role::Button, "Sistema 3");
    assert!(harness.query_by_label("Node 3").is_none());
}

#[test]
fn a_blank_name_counts_as_no_name() {
    let mut map = Map::new();
    let mut node = MapPoint::new(9, [0.0, 0.0]);
    node.name = Some("   ".to_owned());
    map.add_points(vec![node]);
    let mut harness = harness(map);
    harness.run();

    harness.get_by_role_and_label(Role::Button, "Node 9");
}

#[test]
fn nodes_off_screen_are_not_in_the_tree() {
    let mut map = Map::new();
    let mut jita = MapPoint::new(1, [0.0, 0.0]);
    jita.name = Some("Jita".to_owned());
    map.add_points(vec![jita, MapPoint::new(50, [100_000.0, 100_000.0])]);
    map.set_pos([0.0, 0.0]);
    map.set_zoom(1.0);
    let mut harness = harness(map);
    harness.run();

    assert!(harness.query_by_label("Node 50").is_none());
    harness.get_by_role_and_label(Role::Button, "Jita");
}

#[test]
fn nothing_is_activated_until_the_user_does_it() {
    let mut harness = recording(named_map());
    harness.run();

    assert!(harness.state().activations.is_empty());
    assert_eq!(harness.state().map.activated_node(), None);
}

#[test]
fn a_screen_reader_click_activates_the_node() {
    let mut harness = recording(named_map());
    harness.run();
    harness
        .get_by_role_and_label(Role::Button, "Perimeter")
        .click_accesskit();
    harness.run();

    assert_eq!(harness.state().activations, [2]);
}

#[test]
fn the_activation_lasts_one_frame() {
    let mut harness = recording(named_map());
    harness.run();
    harness
        .get_by_role_and_label(Role::Button, "Jita")
        .click_accesskit();
    harness.run_steps(5);

    assert_eq!(harness.state().activations, [1]);
    assert_eq!(harness.state().map.activated_node(), None);
}

#[test]
fn enter_and_space_activate_the_focused_node() {
    for key in [egui::Key::Enter, egui::Key::Space] {
        let mut harness = recording(named_map());
        harness.run();
        harness.get_by_role_and_label(Role::Button, "Jita").focus();
        harness.run();
        harness.key_press(key);
        harness.run();

        assert_eq!(harness.state().activations, [1], "{key:?}");
    }
}

#[test]
fn enter_does_nothing_while_no_node_has_the_focus() {
    let mut harness = recording(named_map());
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();

    assert!(harness.state().activations.is_empty());
}

#[test]
fn the_focus_moves_between_the_nodes_with_tab() {
    let mut harness = harness(named_map());
    harness.run();
    harness.get_by_role_and_label(Role::Button, "Jita").focus();
    harness.run();
    assert!(
        harness
            .get_by_role_and_label(Role::Button, "Jita")
            .is_focused()
    );

    harness.key_press(egui::Key::Tab);
    harness.run();
    assert!(
        !harness
            .get_by_role_and_label(Role::Button, "Jita")
            .is_focused()
    );
}

#[test]
fn dragging_from_a_node_still_pans_the_map() {
    // A map bigger than the viewport: one that fits in it stays centered.
    let mut map = named_map();
    map.insert_point(MapPoint::new(4, [-1000.0, 0.0]));
    map.insert_point(MapPoint::new(5, [1000.0, 0.0]));
    let mut harness = harness(map);
    harness.run();
    let node = harness
        .get_by_role_and_label(Role::Button, "Jita")
        .rect()
        .center();
    let before = harness.state().get_pos();

    harness.drag_at(node);
    harness.run();
    harness.hover_at(node + egui::vec2(60.0, 30.0));
    harness.run();
    harness.drop_at(node + egui::vec2(60.0, 30.0));
    harness.run();

    let after = harness.state().get_pos();
    assert_ne!(before, after, "the drag did not move the map");
}

#[test]
fn a_node_is_still_hovered_through_its_button() {
    let mut harness = harness(named_map());
    harness.run();
    let node = harness
        .get_by_role_and_label(Role::Button, "Jita")
        .rect()
        .center();
    harness.hover_at(node);
    harness.run();

    assert_eq!(harness.state().hovered_node(), Some(1));
}

#[test]
fn without_a_screen_reader_the_nodes_are_not_exposed() {
    use egui::{Context, Pos2, RawInput, Rect};
    let ctx = Context::default();
    let mut map = named_map();
    let screen = Rect::from_min_size(Pos2::ZERO, egui::vec2(600.0, 400.0));
    for _ in 0..3 {
        let mut output = ctx.run_ui(
            RawInput {
                screen_rect: Some(screen),
                events: vec![egui::Event::Key {
                    key: egui::Key::Enter,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                }],
                ..RawInput::default()
            },
            |ui| {
                ui.add(&mut map);
            },
        );
        output.textures_delta.clear();
    }

    assert_eq!(map.activated_node(), None);
}

#[test]
fn a_nodes_button_is_centered_on_the_node_and_big_enough_to_aim_at() {
    let mut harness = harness(named_map());
    harness.run();
    let rect = harness.get_by_role_and_label(Role::Button, "Jita").rect();

    assert!(rect.width() >= 16.0 && rect.height() >= 16.0, "{rect:?}");
    // The map centers on (0, 0) and the node is there.
    let screen = harness.get_by_role_and_label(Role::Canvas, "Map").rect();
    assert!((rect.center() - screen.center()).length() < 4.0);
}

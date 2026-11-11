//! The mouse and keyboard side of the map: zooming with the wheel, panning
//! by dragging and the context menu, driven through an `egui_kittest` harness.

use egui::{Event, Modifiers, MouseWheelUnit, PointerButton, Pos2, Role, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use egui_map::map::Map;
use egui_map::map::objects::{ContextMenuManager, MapPoint};
use std::cell::Cell;
use std::rc::Rc;

/// Two nodes, the view on the first one at zoom 1. Two more far away make the
/// map bigger than the viewport: a map that fits in it can't be panned (it
/// stays centered), and these tests are about panning.
fn map() -> Map {
    let mut map = Map::new();
    map.add_points(vec![
        MapPoint::new(1, [0.0, 0.0]),
        MapPoint::new(2, [50.0, 0.0]),
        MapPoint::new(3, [-1000.0, -1000.0]),
        MapPoint::new(4, [1000.0, 1000.0]),
    ]);
    map.set_pos([0.0, 0.0]);
    map.set_zoom(1.0);
    map
}

fn harness(map: Map) -> Harness<'static, Map> {
    Harness::builder()
        .with_size(vec2(600.0, 400.0))
        .build_ui_state(
            |ui, map: &mut Map| {
                ui.add(map);
            },
            map,
        )
}

/// The wheel modifier that zooms faster: Ctrl, or Cmd on macOS.
fn fast() -> Modifiers {
    if cfg!(target_os = "macos") {
        Modifiers::MAC_CMD
    } else {
        Modifiers::CTRL
    }
}

fn wheel(harness: &Harness<'_, Map>, at: Pos2, delta_y: f32, modifiers: Modifiers) {
    harness.hover_at(at);
    harness.event(Event::MouseWheel {
        unit: MouseWheelUnit::Line,
        delta: vec2(0.0, delta_y),
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Wheel,
        modifiers,
    });
}

fn center(harness: &Harness<'_, Map>) -> Pos2 {
    harness
        .get_by_role_and_label(Role::Canvas, "Map")
        .rect()
        .center()
}

#[test]
fn the_wheel_zooms_in_and_out() {
    let mut harness = harness(map());
    harness.state_mut().set_zoom(1.0);
    harness.run();
    let at = center(&harness);

    wheel(&harness, at, 4.0, Modifiers::NONE);
    harness.run();
    let zoomed_in = harness.state_mut().get_zoom();
    assert!(zoomed_in > 1.0, "{zoomed_in}");

    wheel(&harness, at, -4.0, Modifiers::NONE);
    harness.run();
    assert!(harness.state_mut().get_zoom() < zoomed_in);
}

#[test]
fn the_wheel_with_ctrl_or_cmd_zooms_faster() {
    let mut slow = harness(map());
    slow.run();
    let at = center(&slow);
    wheel(&slow, at, 4.0, Modifiers::NONE);
    slow.run();

    let mut quick = harness(map());
    quick.run();
    wheel(&quick, at, 4.0, fast());
    quick.run();

    let slow = slow.state_mut().get_zoom() - 1.0;
    let quick = quick.state_mut().get_zoom() - 1.0;
    assert!(slow > 0.0, "{slow}");
    assert!(quick > slow * 2.0, "{quick} against {slow}");
}

#[test]
fn the_wheel_stops_at_the_zoom_limits() {
    // Without the zoom slider, which would clamp the value on its own.
    let mut map = map();
    map.settings.slider.visible = false;
    let mut harness = harness(map);
    harness.run();
    let at = center(&harness);
    let (min, max) = {
        let settings = &harness.state_mut().settings;
        (settings.min_zoom, settings.max_zoom)
    };

    wheel(&harness, at, 100_000.0, fast());
    harness.run_steps(12);
    assert_eq!(harness.state_mut().get_zoom(), max);

    wheel(&harness, at, -100_000.0, fast());
    harness.run_steps(12);
    assert_eq!(harness.state_mut().get_zoom(), min);
}

#[test]
fn the_wheel_outside_the_map_does_nothing() {
    let mut map = map();
    map.allocate_at_most(Some(200.0), Some(150.0));
    let mut harness = harness(map);
    harness.run();

    wheel(&harness, Pos2::new(580.0, 380.0), 4.0, Modifiers::NONE);
    harness.run();
    assert_eq!(harness.state_mut().get_zoom(), 1.0);

    wheel(&harness, Pos2::new(100.0, 75.0), 4.0, Modifiers::NONE);
    harness.run();
    assert!(harness.state_mut().get_zoom() > 1.0);
}

fn press(harness: &Harness<'_, Map>, at: Pos2, button: PointerButton, pressed: bool) {
    harness.event(Event::PointerButton {
        pos: at,
        button,
        pressed,
        modifiers: Modifiers::NONE,
    });
}

/// Drags the empty part of the map by `by`, in the steps a mouse makes.
fn drag(harness: &mut Harness<'_, Map>, from: Pos2, by: egui::Vec2) {
    harness.hover_at(from);
    harness.run();
    press(harness, from, PointerButton::Primary, true);
    harness.run();
    for step in 1..=4 {
        harness.hover_at(from + by * (step as f32 / 4.0));
        harness.run();
    }
    press(harness, from + by, PointerButton::Primary, false);
    harness.run();
}

#[test]
fn dragging_the_map_moves_the_view_the_other_way() {
    let mut harness = harness(map());
    harness.run();
    let from = center(&harness) + vec2(100.0, 80.0);

    drag(&mut harness, from, vec2(60.0, -40.0));

    let [x, y] = harness.state_mut().get_pos();
    assert!((x + 60.0).abs() < 1.0, "x {x}");
    assert!((y - 40.0).abs() < 1.0, "y {y}");
}

#[test]
fn a_drag_covers_less_of_the_map_when_zoomed_in() {
    let mut harness = harness(map());
    harness.state_mut().set_zoom(2.0);
    harness.run();
    let from = center(&harness) + vec2(100.0, 80.0);

    drag(&mut harness, from, vec2(60.0, 0.0));

    let [x, _] = harness.state_mut().get_pos();
    assert!((x + 30.0).abs() < 1.0, "x {x}");
}

#[test]
fn a_press_without_moving_leaves_the_view_alone() {
    let mut harness = harness(map());
    harness.run();
    let from = center(&harness) + vec2(100.0, 80.0);

    drag(&mut harness, from, vec2(0.0, 0.0));

    assert_eq!(harness.state_mut().get_pos(), [0.0, 0.0]);
}

/// Counts the frames in which the menu was built.
#[derive(Default)]
struct Menu {
    built: Cell<usize>,
}

impl ContextMenuManager for Menu {
    fn ui(&self, ui: &mut egui::Ui) {
        self.built.set(self.built.get() + 1);
        ui.label("The menu");
    }
}

#[test]
fn a_right_click_opens_the_context_menu() {
    let menu = Rc::new(Menu::default());
    let mut map = map();
    map.set_context_manager(menu.clone());
    let mut harness = harness(map);
    harness.run();
    assert_eq!(menu.built.get(), 0, "the menu is built before it is asked");

    harness
        .get_by_role_and_label(Role::Canvas, "Map")
        .click_secondary();
    harness.run();

    assert!(menu.built.get() > 0);
    harness.get_by_label("The menu");
}

#[test]
fn a_left_click_does_not_open_the_context_menu() {
    let menu = Rc::new(Menu::default());
    let mut map = map();
    map.set_context_manager(menu.clone());
    let mut harness = harness(map);
    harness.run();

    harness.get_by_role_and_label(Role::Canvas, "Map").click();
    harness.run();

    assert_eq!(menu.built.get(), 0);
}

#[test]
fn without_a_context_manager_a_right_click_opens_nothing() {
    let mut harness = harness(map());
    harness.run();
    harness
        .get_by_role_and_label(Role::Canvas, "Map")
        .click_secondary();
    harness.run();

    assert!(harness.query_by_label("The menu").is_none());
}

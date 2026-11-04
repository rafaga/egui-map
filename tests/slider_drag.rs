//! The zoom slider keeps following the mouse while the nodes on screen change.
//!
//! A node template may add widgets of its own (an `Image` drawn by `ui.put`).
//! Dragging the slider zooms the map, so nodes come into view and leave it
//! while the handle is held; the slider used to take its id from how many
//! widgets had been added before it, got a new one in the middle of the drag,
//! and froze until the button was released.

use egui::{Event, Modifiers, PointerButton, Role, Sense, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use egui_map::map::Map;
use egui_map::map::objects::{HitContext, MapPoint, NodeContext, NodeOutline, NodeTemplate};
use std::rc::Rc;

/// Adds a widget of its own for every node it draws, the way an image does.
struct Widgets(Sense);

impl NodeTemplate for Widgets {
    fn node_ui(&self, ui: &mut egui::Ui, ctx: NodeContext) {
        let size = 30.0 * ctx.zoom;
        let rect = egui::Rect::from_center_size(ctx.position, egui::Vec2::splat(size));
        ui.allocate_rect(rect, self.0);
    }

    fn outline(&self, ctx: HitContext) -> NodeOutline {
        let size = 30.0 * ctx.zoom;
        NodeOutline::RoundedRect {
            rect: egui::Rect::from_center_size(ctx.position, egui::Vec2::splat(size)),
            corner_radius: 4.0,
        }
    }
}

fn harness(template: Option<Sense>, nodes: &[[f32; 2]]) -> Harness<'static, Map> {
    let mut map = Map::new();
    map.add_points(
        nodes
            .iter()
            .enumerate()
            .map(|(index, coords)| MapPoint::new(index + 1, *coords))
            .collect(),
    );
    map.set_pos([0.0, 0.0]);
    map.set_zoom(1.0);
    if let Some(sense) = template {
        map.set_node_template(Rc::new(Widgets(sense)));
    }
    Harness::builder()
        .with_size(vec2(600.0, 400.0))
        .build_ui_state(
            |ui, map: &mut Map| {
                ui.add(map);
            },
            map,
        )
}

/// Holds the slider handle and drags it up by 40 points, in four steps.
fn drag_the_slider_up(harness: &mut Harness<'_, Map>) -> (f32, f32) {
    let from = harness
        .get_by_role_and_label(Role::Slider, "Zoom")
        .rect()
        .center();
    let before = harness.state_mut().get_zoom();
    let button = |pressed: bool, pos| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    harness.hover_at(from);
    harness.run();
    harness.event(button(true, from));
    harness.run();
    for step in 1..=4 {
        harness.hover_at(from + vec2(0.0, -10.0 * step as f32));
        harness.run();
    }
    harness.event(button(false, from + vec2(0.0, -40.0)));
    harness.run();
    (before, harness.state_mut().get_zoom())
}

/// A node under the slider, in map coordinates (the view is on (0, 0)).
fn under_the_slider(harness: &Harness<'_, Map>) -> [f32; 2] {
    let slider = harness
        .get_by_role_and_label(Role::Slider, "Zoom")
        .rect()
        .center();
    let canvas = harness
        .get_by_role_and_label(Role::Canvas, "Map")
        .rect()
        .center();
    let offset = slider - canvas;
    [offset.x, offset.y]
}

#[test]
fn the_slider_follows_the_whole_drag_without_nodes_that_add_widgets() {
    let mut harness = harness(None, &[[0.0, 0.0]]);
    harness.run();

    let (before, after) = drag_the_slider_up(&mut harness);

    assert!(after > before + 0.6, "{before} -> {after}");
}

#[test]
fn the_slider_follows_the_whole_drag_when_a_node_leaves_the_view() {
    for sense in [Sense::hover(), Sense::click(), Sense::click_and_drag()] {
        // The node sits under the slider, and the zoom pushes it out of view
        // in the middle of the drag.
        let mut probe = harness(None, &[[0.0, 0.0]]);
        probe.run();
        let node = under_the_slider(&probe);
        let mut harness = harness(Some(sense), &[[0.0, 0.0], node]);
        harness.run();

        let (before, after) = drag_the_slider_up(&mut harness);

        assert!(after > before + 0.6, "{sense:?}: {before} -> {after}");
    }
}

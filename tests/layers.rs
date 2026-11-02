//! Verifies the layers: each slot is painted, in order, only while the layer
//! is visible, and `OverNodeBody` layers are painted by the node template.

use egui::{Context, Painter, RawInput, Ui};
use egui_map::map::Map;
use egui_map::map::layer::{LayerContext, LayerNodeContext, LayerSlot, LayerTemplate};
use egui_map::map::objects::{
    MapPoint, MarkerContext, NodeContext, NodeOutline, NodeTemplate, NotificationContext,
    SelectionContext,
};
use std::cell::RefCell;
use std::rc::Rc;

/// Pushes its name to a shared log each time it is painted, and how many
/// points it was given.
struct Recorder {
    name: &'static str,
    log: Rc<RefCell<Vec<String>>>,
}

impl LayerTemplate for Recorder {
    fn layer_ui(&self, _painter: &Painter, ctx: LayerContext) {
        self.log
            .borrow_mut()
            .push(format!("{}:{}", self.name, ctx.points.len()));
    }
    fn node_ui(&self, _painter: &Painter, ctx: LayerNodeContext) {
        self.log
            .borrow_mut()
            .push(format!("{}@{}", self.name, ctx.point.id));
    }
}

/// A template that paints its layers, then notes it did.
struct LayeredTemplate {
    log: Rc<RefCell<Vec<String>>>,
}

impl NodeTemplate for LayeredTemplate {
    fn node_ui(&self, ui: &mut Ui, ctx: NodeContext) {
        self.log.borrow_mut().push(format!("body@{}", ctx.point.id));
        ctx.layers.paint(
            ui.painter(),
            &NodeOutline::Circle {
                center: ctx.position,
                radius: 4.0,
            },
        );
        self.log
            .borrow_mut()
            .push(format!("label@{}", ctx.point.id));
    }
    fn selection_ui(&self, _ui: &mut Ui, _ctx: SelectionContext) {}
    fn notification_ui(&self, _ui: &mut Ui, _ctx: NotificationContext) -> bool {
        false
    }
    fn marker_ui(&self, _ui: &mut Ui, _ctx: MarkerContext) {}
}

fn render_once(map: &mut Map) {
    let ctx = Context::default();
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400.0, 300.0));
    let mut out = ctx.run_ui(
        RawInput {
            screen_rect: Some(screen),
            ..RawInput::default()
        },
        |ui| {
            ui.add(&mut *map);
        },
    );
    out.textures_delta.clear();
}

fn map_with_two_points() -> Map {
    let mut map = Map::new();
    map.add_points(vec![
        MapPoint::new(1, [0.0, 0.0]),
        MapPoint::new(2, [50.0, 0.0]),
    ]);
    map
}

fn recorder(name: &'static str, log: &Rc<RefCell<Vec<String>>>) -> Rc<Recorder> {
    Rc::new(Recorder {
        name,
        log: Rc::clone(log),
    })
}

#[test]
fn the_slots_are_painted_from_back_to_front_with_the_visible_points() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut map = map_with_two_points();
    map.add_layer("above", LayerSlot::AboveNodes, 0, recorder("above", &log));
    map.add_layer("below", LayerSlot::BelowNodes, 0, recorder("below", &log));
    map.add_layer("lines", LayerSlot::BelowLines, 0, recorder("lines", &log));
    render_once(&mut map);
    assert_eq!(*log.borrow(), ["lines:2", "below:2", "above:2"]);
}

#[test]
fn layers_of_a_slot_follow_z_then_arrival() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut map = map_with_two_points();
    map.add_layer("late", LayerSlot::BelowNodes, 5, recorder("late", &log));
    map.add_layer("first", LayerSlot::BelowNodes, 1, recorder("first", &log));
    map.add_layer("second", LayerSlot::BelowNodes, 1, recorder("second", &log));
    render_once(&mut map);
    assert_eq!(*log.borrow(), ["first:2", "second:2", "late:2"]);
}

#[test]
fn a_hidden_or_removed_layer_is_not_painted_and_adding_again_replaces() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut map = map_with_two_points();
    map.add_layer("heat", LayerSlot::BelowNodes, 0, recorder("old", &log));
    map.add_layer("heat", LayerSlot::BelowNodes, 0, recorder("new", &log));
    assert_eq!(map.layer_visible("heat"), Some(true));
    assert!(map.set_layer_visible("heat", false));
    render_once(&mut map);
    assert!(log.borrow().is_empty());

    assert!(map.set_layer_visible("heat", true));
    render_once(&mut map);
    assert_eq!(*log.borrow(), ["new:2"]);

    assert!(map.remove_layer("heat"));
    assert!(!map.remove_layer("heat"));
    assert!(!map.set_layer_visible("heat", true));
    assert_eq!(map.layer_visible("heat"), None);
}

#[test]
fn over_node_body_layers_are_painted_by_the_template_between_body_and_label() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut map = map_with_two_points();
    map.set_node_template(Rc::new(LayeredTemplate {
        log: Rc::clone(&log),
    }));
    map.add_layer("tint", LayerSlot::OverNodeBody, 0, recorder("tint", &log));
    render_once(&mut map);
    let log = log.borrow();
    for id in [1, 2] {
        let body = log.iter().position(|e| *e == format!("body@{id}")).unwrap();
        assert_eq!(log[body + 1], format!("tint@{id}"));
        assert_eq!(log[body + 2], format!("label@{id}"));
    }
    // The once-per-frame method is never called for this slot.
    assert!(!log.iter().any(|entry| entry.starts_with("tint:")));
}

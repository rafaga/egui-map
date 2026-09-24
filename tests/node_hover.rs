//! Verifies `Map::hovered_node`: the node under the pointer, hit-tested with
//! the default radius or with a template's own `NodeTemplate::contains`.

use egui::{Context, Event, Pos2, RawInput, Rect, Ui, vec2};
use egui_map::map::Map;
use egui_map::map::objects::{
    HitContext, MapPoint, MarkerContext, NodeContext, NodeTemplate, NotificationContext,
    SelectionContext,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Records where each node was painted, and optionally hit-tests a box.
#[derive(Default)]
struct Template {
    positions: RefCell<HashMap<usize, Pos2>>,
    /// Half size of the hit box; `None` keeps the default radius.
    half_box: Option<egui::Vec2>,
}

impl NodeTemplate for Template {
    fn node_ui(&self, _ui: &mut Ui, ctx: NodeContext) {
        self.positions
            .borrow_mut()
            .insert(ctx.point.id, ctx.position);
    }
    fn selection_ui(&self, _ui: &mut Ui, _ctx: SelectionContext) {}
    fn notification_ui(&self, _ui: &mut Ui, _ctx: NotificationContext) -> bool {
        false
    }
    fn marker_ui(&self, _ui: &mut Ui, _ctx: MarkerContext) {}
    fn contains(&self, ctx: HitContext, pointer: Pos2) -> bool {
        match self.half_box {
            Some(half) => Rect::from_center_size(ctx.position, half * 2.0).contains(pointer),
            None => ctx.within_default_radius(pointer),
        }
    }
    fn hit_extent(&self, zoom: f32) -> f32 {
        match self.half_box {
            Some(half) => half.length(),
            None => egui_map::map::objects::default_hit_extent(zoom),
        }
    }
}

fn frame(ctx: &Context, map: &mut Map, pointer: Option<Pos2>) {
    let screen = Rect::from_min_size(Pos2::ZERO, vec2(400.0, 300.0));
    let events = pointer
        .map(|pos| vec![Event::PointerMoved(pos)])
        .unwrap_or_default();
    let mut out = ctx.run_ui(
        RawInput {
            screen_rect: Some(screen),
            events,
            ..RawInput::default()
        },
        |ui| {
            ui.add(&mut *map);
        },
    );
    out.textures_delta.clear();
}

fn map_with(template: Rc<Template>) -> Map {
    let mut map = Map::new();
    map.add_points(vec![
        MapPoint::new(1, [0.0, 0.0]),
        MapPoint::new(2, [100.0, 0.0]),
    ]);
    map.set_node_template(template);
    map
}

#[test]
fn the_node_under_the_pointer_is_reported() {
    let template = Rc::new(Template::default());
    let mut map = map_with(template.clone());
    let ctx = Context::default();
    frame(&ctx, &mut map, None);
    assert_eq!(map.hovered_node(), None);

    let node = template.positions.borrow()[&1];
    // Twice: egui hit-tests the pointer against the previous frame's widgets.
    frame(&ctx, &mut map, Some(node));
    frame(&ctx, &mut map, Some(node));
    assert_eq!(map.hovered_node(), Some(1));

    // Just off the default radius: nothing.
    let away = node + vec2(30.0, 0.0);
    frame(&ctx, &mut map, Some(away));
    frame(&ctx, &mut map, Some(away));
    assert_eq!(map.hovered_node(), None);
}

#[test]
fn a_template_hit_area_is_used() {
    let template = Rc::new(Template {
        half_box: Some(vec2(40.0, 10.0)),
        ..Template::default()
    });
    let mut map = map_with(template.clone());
    let ctx = Context::default();
    frame(&ctx, &mut map, None);

    // Inside the box but well outside the default radius.
    let pointer = template.positions.borrow()[&1] + vec2(30.0, 0.0);
    frame(&ctx, &mut map, Some(pointer));
    frame(&ctx, &mut map, Some(pointer));
    assert_eq!(map.hovered_node(), Some(1));
}

/// Hit-tests a wide box around node 1 only; every other node keeps the
/// default radius.
struct WideBox {
    positions: RefCell<HashMap<usize, Pos2>>,
    order: RefCell<Vec<usize>>,
    half: egui::Vec2,
    boxed: Vec<usize>,
}

impl NodeTemplate for WideBox {
    fn node_ui(&self, _ui: &mut Ui, ctx: NodeContext) {
        self.positions
            .borrow_mut()
            .insert(ctx.point.id, ctx.position);
        self.order.borrow_mut().push(ctx.point.id);
    }
    fn selection_ui(&self, _ui: &mut Ui, _ctx: SelectionContext) {}
    fn notification_ui(&self, _ui: &mut Ui, _ctx: NotificationContext) -> bool {
        false
    }
    fn marker_ui(&self, _ui: &mut Ui, _ctx: MarkerContext) {}
    fn contains(&self, ctx: HitContext, pointer: Pos2) -> bool {
        if self.boxed.contains(&ctx.point.id) {
            Rect::from_center_size(ctx.position, self.half * 2.0).contains(pointer)
        } else {
            ctx.within_default_radius(pointer)
        }
    }
    fn hit_extent(&self, _zoom: f32) -> f32 {
        self.half.length()
    }
}

#[test]
fn a_large_hit_area_is_found_past_nearer_centers() {
    // Node 1 has a 300 px wide box; five nodes sit closer to the pointer
    // (the box's right end) than node 1's center, none of them under it.
    let template = Rc::new(WideBox {
        positions: RefCell::new(HashMap::new()),
        order: RefCell::new(Vec::new()),
        half: vec2(150.0, 10.0),
        boxed: vec![1],
    });
    let mut map = Map::new();
    let mut points = vec![MapPoint::new(1, [0.0, 0.0])];
    for (i, id) in (2..=6).enumerate() {
        points.push(MapPoint::new(id, [120.0 + 5.0 * i as f32, 40.0]));
    }
    map.add_points(points);
    map.set_node_template(template.clone());
    let ctx = Context::default();
    frame(&ctx, &mut map, None);

    let pointer = template.positions.borrow()[&1] + vec2(140.0, 0.0);
    frame(&ctx, &mut map, Some(pointer));
    frame(&ctx, &mut map, Some(pointer));
    assert_eq!(map.hovered_node(), Some(1));
}

#[test]
fn where_areas_overlap_the_node_on_top_wins() {
    let template = Rc::new(WideBox {
        positions: RefCell::new(HashMap::new()),
        order: RefCell::new(Vec::new()),
        half: vec2(60.0, 20.0),
        boxed: vec![1, 2],
    });
    let mut map = Map::new();
    map.add_points(vec![
        MapPoint::new(1, [0.0, 0.0]),
        MapPoint::new(2, [30.0, 0.0]),
    ]);
    map.set_node_template(template.clone());
    let ctx = Context::default();
    frame(&ctx, &mut map, None);

    // Inside both boxes, nearer node 1's center than node 2's.
    let positions = template.positions.borrow().clone();
    let pointer = positions[&1] + vec2(5.0, 0.0);
    template.order.borrow_mut().clear();
    frame(&ctx, &mut map, Some(pointer));
    template.order.borrow_mut().clear();
    frame(&ctx, &mut map, Some(pointer));
    let on_top = *template.order.borrow().last().unwrap();
    assert_eq!(map.hovered_node(), Some(on_top));
}

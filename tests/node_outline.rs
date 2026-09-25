//! Verifies the `NodeTemplate::outline`-driven defaults: a template that only
//! draws its node and declares its shape gets a matching hit area and
//! built-in effects that follow that shape.

use egui::epaint::ClippedShape;
use egui::{Context, Event, Pos2, RawInput, Rect, Shape, Ui, Vec2, vec2};
use egui_map::map::Map;
use egui_map::map::objects::{HitContext, MapPoint, NodeContext, NodeOutline, NodeTemplate};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

const HALF: Vec2 = Vec2::new(45.0, 17.5);

/// Only `node_ui` and `outline`: every other hook is the default.
#[derive(Default)]
struct Boxes {
    positions: RefCell<HashMap<usize, Pos2>>,
}

impl NodeTemplate for Boxes {
    fn node_ui(&self, _ui: &mut Ui, ctx: NodeContext) {
        self.positions
            .borrow_mut()
            .insert(ctx.point.id, ctx.position);
    }

    fn outline(&self, ctx: HitContext) -> NodeOutline {
        NodeOutline::RoundedRect {
            rect: Rect::from_center_size(ctx.position, HALF * 2.0 * ctx.zoom),
            corner_radius: 10.0 * ctx.zoom,
        }
    }
}

fn frame(ctx: &Context, map: &mut Map, pointer: Option<Pos2>) -> Vec<ClippedShape> {
    let events = pointer
        .map(|pos| vec![Event::PointerMoved(pos)])
        .unwrap_or_default();
    let mut out = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 300.0))),
            events,
            ..RawInput::default()
        },
        |ui| {
            ui.add(&mut *map);
        },
    );
    out.textures_delta.clear();
    out.shapes
}

fn setup() -> (Map, Rc<Boxes>) {
    let template = Rc::new(Boxes::default());
    let mut map = Map::new();
    map.add_points(vec![MapPoint::new(1, [0.0, 0.0])]);
    map.set_node_template(template.clone());
    (map, template)
}

#[test]
fn the_hit_area_follows_the_outline() {
    let (mut map, template) = setup();
    let ctx = Context::default();
    frame(&ctx, &mut map, None);
    // Near the box's right end: far outside the default radius, and with no
    // `contains`/`hit_extent` of the template's own.
    let pointer = template.positions.borrow()[&1] + vec2(40.0, 0.0);
    frame(&ctx, &mut map, Some(pointer));
    frame(&ctx, &mut map, Some(pointer));
    assert_eq!(map.hovered_node(), Some(1));
}

#[test]
fn the_default_pulse_keeps_the_node_shape() {
    let (mut map, _template) = setup();
    let ctx = Context::default();
    frame(&ctx, &mut map, None);
    let zoom = 1.0;
    // A second into the pulse: the box grown by 40 points on every side.
    map.node(1)
        .unwrap()
        .pulse(Instant::now() - Duration::from_secs(1));
    let shapes = frame(&ctx, &mut map, None);
    let grown = (HALF + Vec2::splat(40.0 * zoom)) * 2.0;
    let found = shapes.iter().any(|clipped| match &clipped.shape {
        Shape::Rect(rect) => (rect.rect.size() - grown).length() < 1.0,
        _ => false,
    });
    assert!(found, "a rounded rect the size of the grown box is drawn");
}

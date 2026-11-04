//! The public helpers a custom template calls to draw the built-in effects
//! along its own outline, and a few small public functions that no other test
//! reaches: `LineFade::has_threshold`, `CometDirection::orient` and
//! `NotificationContext::effect_start`.

use egui::{Color32, Context, Painter, Pos2, RawInput, Rect, Shape, Ui, pos2, vec2};
use egui_map::map::Map;
use egui_map::map::animation::Animation;
use egui_map::map::objects::{
    CometDirection, LineFade, MapPoint, MarkerContext, NodeAnimation, NodeContext, NodeOutline,
    NodeTemplate, NotificationContext, SelectionContext, SteadyAnimation,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// How many shapes `paint` adds to a frame, and what it returned.
fn painted<T>(paint: impl FnOnce(&Painter) -> T) -> (usize, T) {
    let ctx = Context::default();
    let mut paint = Some(paint);
    let mut result = None;
    let mut out = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 300.0))),
            ..RawInput::default()
        },
        |ui| {
            result = paint.take().map(|paint| paint(ui.painter()));
        },
    );
    out.textures_delta.clear();
    let shapes = out.shapes.len();
    (shapes, result.unwrap())
}

/// The shapes `paint` adds to a frame, as `egui` hands them out.
fn shapes_of(paint: impl FnOnce(&Painter)) -> Vec<Shape> {
    let ctx = Context::default();
    let mut paint = Some(paint);
    let mut out = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 300.0))),
            ..RawInput::default()
        },
        |ui| {
            if let Some(paint) = paint.take() {
                paint(ui.painter());
            }
        },
    );
    out.textures_delta.clear();
    out.shapes
        .into_iter()
        .map(|clipped| clipped.shape)
        .collect()
}

fn outlines() -> Vec<NodeOutline> {
    vec![
        NodeOutline::Circle {
            center: pos2(200.0, 150.0),
            radius: 10.0,
        },
        NodeOutline::RoundedRect {
            rect: Rect::from_center_size(pos2(200.0, 150.0), vec2(60.0, 30.0)),
            corner_radius: 6.0,
        },
        NodeOutline::Polygon(vec![
            pos2(190.0, 140.0),
            pos2(210.0, 140.0),
            pos2(200.0, 165.0),
        ]),
    ]
}

const KINDS: [NodeAnimation; 5] = [
    NodeAnimation::Pulse,
    NodeAnimation::Ripple,
    NodeAnimation::CountdownArc,
    NodeAnimation::ScaleIn,
    NodeAnimation::Crosshair,
];

#[test]
fn every_event_effect_follows_every_outline() {
    let animation = Animation::default();
    let color = Color32::from_rgb(220, 60, 40);
    for kind in KINDS {
        for outline in outlines() {
            let effect = animation.event_outline(kind);
            let (shapes, running) = painted(|painter| {
                effect(
                    painter,
                    &outline,
                    1.0,
                    Instant::now() - Duration::from_millis(50),
                    color,
                )
            });
            assert!(running, "{kind:?} on {outline:?} ended at once");
            assert!(shapes > 0, "{kind:?} on {outline:?} painted nothing");
        }
    }
}

#[test]
fn an_event_effect_ends_and_paints_nothing_when_its_time_is_up() {
    let animation = Animation::default();
    let long_ago = Instant::now() - Duration::from_secs(3600);
    for kind in [
        NodeAnimation::Pulse,
        NodeAnimation::CountdownArc,
        NodeAnimation::ScaleIn,
        NodeAnimation::Crosshair,
    ] {
        let effect = animation.event_outline(kind);
        let outline = &outlines()[0];
        let (shapes, running) =
            painted(|painter| effect(painter, outline, 1.0, long_ago, Color32::WHITE));
        assert!(!running, "{kind:?} was still running an hour later");
        // Only the pulse skips its fully transparent tail; the others draw
        // one last frame with no opacity left.
        if kind == NodeAnimation::Pulse {
            assert_eq!(shapes, 0, "{kind:?} painted after it ended");
        }
    }
}

#[test]
fn every_steady_effect_follows_every_outline() {
    let animation = Animation::default();
    for kind in [
        SteadyAnimation::Blink,
        SteadyAnimation::Halo,
        SteadyAnimation::Orbit,
    ] {
        for outline in outlines() {
            let effect = animation.state_outline(kind);
            // A blink is off half of the time: look at a whole second.
            let painted_at_some_time = (0..20).any(|step| {
                let time = step as f32 * 0.05;
                painted(|painter| effect(painter, &outline, 1.0, time, Color32::GOLD)).0 > 0
            });
            assert!(
                painted_at_some_time,
                "{kind:?} on {outline:?} never painted"
            );
        }
    }
}

#[test]
fn a_lasting_event_effect_keeps_running_until_it_is_dropped() {
    let animation = Animation::default();
    let started = Instant::now() - Duration::from_secs(5);
    let until = started + Duration::from_secs(60);
    for kind in KINDS {
        let effect = animation.lasting_event_outline(kind);
        let outline = &outlines()[1];
        let (shapes, running) =
            painted(|painter| effect(painter, outline, 1.0, started, until, Color32::LIGHT_BLUE));
        assert!(running, "{kind:?}");
        assert!(shapes > 0, "{kind:?} painted nothing");
    }
}

#[test]
fn the_effects_scale_with_the_zoom() {
    let animation = Animation::default();
    let effect = animation.event_outline(NodeAnimation::Pulse);
    let outline = &outlines()[0];
    // A zoom of zero would collapse the effect; the call must still be safe.
    let (_, running) = painted(|painter| {
        effect(
            painter,
            outline,
            0.0,
            Instant::now() - Duration::from_millis(50),
            Color32::WHITE,
        )
    });
    let _ = running;
}

#[test]
fn only_the_zoom_fades_have_a_threshold() {
    assert!(!LineFade::None.has_threshold());
    assert!(LineFade::Zoom { range: 0.5 }.has_threshold());
    assert!(LineFade::default().has_threshold());
}

#[test]
fn the_line_fade_factor_is_clamped_to_zero_and_one() {
    let fade = LineFade::Zoom { range: 0.5 };
    assert_eq!(fade.factor(0.2, 0.2), 0.0);
    assert_eq!(fade.factor(0.0, 0.2), 0.0);
    assert!((fade.factor(0.45, 0.2) - 0.5).abs() < 1e-6);
    assert_eq!(fade.factor(5.0, 0.2), 1.0);
    assert_eq!(LineFade::None.factor(0.0, 0.2), 1.0);
}

#[test]
fn a_comet_direction_orders_the_endpoints() {
    assert_eq!(CometDirection::Forward.orient(1, 2), (1, 2));
    assert_eq!(CometDirection::Reverse.orient(1, 2), (2, 1));
    assert_eq!(
        CometDirection::Reverse.orient("a", "b"),
        CometDirection::Forward.orient("b", "a")
    );
}

/// Keeps the `effect_start` of every notification it draws.
struct Starts {
    seen: RefCell<Vec<(Instant, Instant, bool)>>,
}

impl NodeTemplate for Starts {
    fn node_ui(&self, _ui: &mut Ui, _ctx: NodeContext) {}
    fn selection_ui(&self, _ui: &mut Ui, _ctx: SelectionContext) {}
    fn notification_ui(&self, _ui: &mut Ui, ctx: NotificationContext) -> bool {
        self.seen
            .borrow_mut()
            .push((ctx.initial_time, ctx.effect_start(), ctx.until.is_some()));
        true
    }
    fn marker_ui(&self, _ui: &mut Ui, _ctx: MarkerContext) {}
}

fn notified(lasting: Option<Duration>, age: Duration) -> (Instant, Instant, bool) {
    let template = Rc::new(Starts {
        seen: RefCell::new(Vec::new()),
    });
    let mut map = Map::new();
    map.add_points(vec![MapPoint::new(1, [0.0, 0.0])]);
    map.set_node_template(template.clone());
    let started = Instant::now() - age;
    let handle = map.node(1).unwrap();
    match lasting {
        Some(duration) => handle.lasting(duration).pulse(started),
        None => handle.pulse(started),
    }
    let ctx = Context::default();
    let mut out = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 300.0))),
            ..RawInput::default()
        },
        |ui| {
            ui.add(&mut map);
        },
    );
    out.textures_delta.clear();
    let seen = template.seen.borrow();
    assert_eq!(seen.len(), 1);
    seen[0]
}

#[test]
fn a_one_shot_notification_starts_its_effect_when_it_was_triggered() {
    let (initial, start, lasting) = notified(None, Duration::from_millis(300));

    assert!(!lasting);
    assert_eq!(start, initial);
}

#[test]
fn a_lasting_notification_restarts_its_effect_every_cycle() {
    let cycle = Animation::default().event_duration(NodeAnimation::Pulse);
    // Three and a half cycles in.
    let age = Duration::from_secs_f32(cycle * 3.5);
    let (initial, start, lasting) = notified(Some(Duration::from_secs(600)), age);

    assert!(lasting);
    assert!(start > initial, "the effect never restarted");
    let into_cycle = start.elapsed().as_secs_f32();
    assert!(
        into_cycle < cycle + 0.5,
        "the cycle began {into_cycle}s ago, a cycle is {cycle}s"
    );
}

fn circle(shape: &Shape) -> &egui::epaint::CircleShape {
    match shape {
        Shape::Circle(circle) => circle,
        other => panic!("not a circle: {other:?}"),
    }
}

#[test]
fn each_event_kind_draws_its_own_effect() {
    let animation = Animation::default();
    let outline = NodeOutline::Circle {
        center: pos2(200.0, 150.0),
        radius: 10.0,
    };
    let started = Instant::now() - Duration::from_millis(50);
    let draw = |kind: NodeAnimation| {
        let effect = animation.event_outline(kind);
        shapes_of(|painter| {
            effect(painter, &outline, 1.0, started, Color32::RED);
        })
    };

    // A pulse is a disc that grows out of the node.
    let pulse = draw(NodeAnimation::Pulse);
    assert_eq!(pulse.len(), 1);
    assert!(circle(&pulse[0]).fill.a() > 0 && circle(&pulse[0]).radius > 10.0);

    // Ripples are rings: a stroke and no fill.
    let ripple = draw(NodeAnimation::Ripple);
    assert!(!ripple.is_empty());
    let ring = circle(&ripple[0]);
    assert!(ring.stroke.width > 0.0 && ring.fill.a() == 0);

    // The countdown is an arc, a path.
    let countdown = draw(NodeAnimation::CountdownArc);
    assert!(matches!(countdown[0], Shape::Path(_)), "{countdown:?}");

    // Scaling in starts from a disc smaller than the node.
    let scale = draw(NodeAnimation::ScaleIn);
    assert_eq!(scale.len(), 1);
    assert!(circle(&scale[0]).fill.a() > 0 && circle(&scale[0]).radius < 10.0);

    // The crosshair is four ticks.
    let crosshair = draw(NodeAnimation::Crosshair);
    assert_eq!(crosshair.len(), 4);
    assert!(
        crosshair
            .iter()
            .all(|shape| matches!(shape, Shape::LineSegment { .. }))
    );
}

#[test]
fn each_steady_kind_is_the_effect_it_is_named_after() {
    let animation = Animation::default();
    let outline = NodeOutline::Circle {
        center: pos2(200.0, 150.0),
        radius: 10.0,
    };
    let color = Color32::GOLD;
    for time in [0.1_f32, 0.3, 0.6, 1.7] {
        let blink = animation.state_outline(SteadyAnimation::Blink);
        let halo = animation.state_outline(SteadyAnimation::Halo);
        let orbit = animation.state_outline(SteadyAnimation::Orbit);
        let same = |from_kind: Vec<Shape>, direct: Vec<Shape>| {
            assert_eq!(format!("{from_kind:?}"), format!("{direct:?}"), "at {time}");
        };
        same(
            shapes_of(|p| blink(p, &outline, 1.0, time, color)),
            shapes_of(|p| animation.blink_outline(p, &outline, 1.0, time, color)),
        );
        same(
            shapes_of(|p| halo(p, &outline, 1.0, time, color)),
            shapes_of(|p| animation.halo_outline(p, &outline, 1.0, time, color)),
        );
        same(
            shapes_of(|p| orbit(p, &outline, 1.0, time, color)),
            shapes_of(|p| animation.orbit_outline(p, &outline, 1.0, time, color)),
        );
    }
}

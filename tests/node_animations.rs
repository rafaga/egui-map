//! Node animations attached through `Map::node`, driven end to end.
//!
//! The unit tests in `map.rs` cover what the handle *records*; these cover what
//! actually reaches the screen — that the recorded choice is the one painted,
//! that event effects stop on their own and lasting state does not, and that
//! the two stack in the documented order.

use egui::{Color32, Context, RawInput, Shape};
use egui_map::map::Map;
use egui_map::map::objects::{MapPoint, MapSettings, SteadyAnimation};
use std::time::{Duration, Instant};

/// A coarse description of a painted shape, enough to tell the effects apart.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Kind {
    FilledCircle,
    StrokedCircle,
    Line,
    Path,
    Other,
}

fn render(map: &mut Map) -> Vec<Kind> {
    let ctx = Context::default();
    render_with(&ctx, map)
}

/// Renders one frame on an existing context, so frame time keeps advancing
/// across calls (`RawInput::default()` leaves `time` unset and egui advances it
/// by `predicted_dt`).
fn render_with(ctx: &Context, map: &mut Map) -> Vec<Kind> {
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
    let kinds = out
        .shapes
        .iter()
        .map(|cs| match &cs.shape {
            Shape::Circle(c) if c.fill.a() > 0 => Kind::FilledCircle,
            Shape::Circle(_) => Kind::StrokedCircle,
            Shape::LineSegment { .. } => Kind::Line,
            Shape::Path(_) => Kind::Path,
            _ => Kind::Other,
        })
        .collect();
    out.textures_delta.clear();
    kinds
}

/// A named way of attaching one effect, so the tables below stay readable.
type Effect = (&'static str, Box<dyn Fn(&mut Map)>);

fn map_with_one_node() -> Map {
    let mut map = Map::new();
    map.settings = MapSettings::default();
    map.add_points(vec![MapPoint::new(1, [0.0, 0.0])]);
    map
}

/// How many shapes the animation added on top of a plain map.
fn extra(with: &[Kind], without: &[Kind]) -> usize {
    with.len().saturating_sub(without.len())
}

#[test]
fn every_event_effect_paints_something() {
    let baseline = render(&mut map_with_one_node());
    let now = Instant::now();

    let effects: Vec<Effect> = vec![
        (
            "pulse",
            Box::new(move |m: &mut Map| m.node(1).unwrap().pulse(now)),
        ),
        (
            "ripple",
            Box::new(move |m: &mut Map| m.node(1).unwrap().ripple(now)),
        ),
        (
            "countdown",
            Box::new(move |m: &mut Map| m.node(1).unwrap().countdown(now)),
        ),
        (
            "scale_in",
            Box::new(move |m: &mut Map| m.node(1).unwrap().scale_in(now)),
        ),
        (
            "crosshair",
            Box::new(move |m: &mut Map| m.node(1).unwrap().crosshair(now)),
        ),
    ];

    for (name, apply) in effects {
        let mut map = map_with_one_node();
        apply(&mut map);
        let shapes = render(&mut map);
        assert!(
            extra(&shapes, &baseline) > 0,
            "{name} painted nothing on top of the plain map: {shapes:?}"
        );
    }
}

#[test]
fn event_effects_stop_painting_once_finished() {
    // Past the longest effect (countdown, 5s) but inside the widget's own 10s
    // orphan sweep, so what removes the notification is the effect reporting it
    // finished -- not the sweep.
    let long_ago = Instant::now() - Duration::from_secs(6);
    let baseline = render(&mut map_with_one_node()).len();

    let effects: Vec<Effect> = vec![
        (
            "pulse",
            Box::new(move |m: &mut Map| m.node(1).unwrap().pulse(long_ago)),
        ),
        (
            "ripple",
            Box::new(move |m: &mut Map| m.node(1).unwrap().ripple(long_ago)),
        ),
        (
            "countdown",
            Box::new(move |m: &mut Map| m.node(1).unwrap().countdown(long_ago)),
        ),
        (
            "scale_in",
            Box::new(move |m: &mut Map| m.node(1).unwrap().scale_in(long_ago)),
        ),
        (
            "crosshair",
            Box::new(move |m: &mut Map| m.node(1).unwrap().crosshair(long_ago)),
        ),
    ];

    for (name, apply) in effects {
        let mut map = map_with_one_node();
        apply(&mut map);
        let _ = render(&mut map); // the frame that notices it is over
        assert_eq!(
            render(&mut map).len(),
            baseline,
            "{name} kept painting after it should have finished"
        );
    }
}

/// If the dispatch ignored the recorded choice these would coincide.
#[test]
fn the_effect_painted_is_the_one_requested() {
    let now = Instant::now();

    let mut arc = map_with_one_node();
    arc.node(1).unwrap().countdown(now);
    let arc = render(&mut arc);

    let mut cross = map_with_one_node();
    cross.node(1).unwrap().crosshair(now);
    let cross = render(&mut cross);

    assert!(
        arc.contains(&Kind::Path),
        "countdown paints an arc as a Path, got {arc:?}"
    );
    assert!(
        cross.iter().filter(|k| **k == Kind::Line).count() >= 4,
        "crosshair paints four ticks, got {cross:?}"
    );
    assert_ne!(arc, cross);
}

#[test]
fn lasting_state_keeps_painting_and_clear_stops_it() {
    let baseline = render(&mut map_with_one_node()).len();

    for animation in [
        SteadyAnimation::Blink,
        SteadyAnimation::Halo,
        SteadyAnimation::Orbit,
    ] {
        let mut map = map_with_one_node();
        match animation {
            SteadyAnimation::Blink => map.node(1).unwrap().blink(),
            SteadyAnimation::Halo => map.node(1).unwrap().halo(),
            SteadyAnimation::Orbit => map.node(1).unwrap().orbit(),
        }

        // unlike a notification, this must survive many frames
        for frame in 0..5 {
            assert!(
                render(&mut map).len() > baseline,
                "{animation:?} stopped painting at frame {frame}"
            );
        }

        map.node(1).unwrap().clear();
        assert_eq!(
            render(&mut map).len(),
            baseline,
            "{animation:?} kept painting after clear()"
        );
    }
}

#[test]
fn state_and_notification_stack_on_the_same_node() {
    let baseline = render(&mut map_with_one_node()).len();

    let mut only_state = map_with_one_node();
    only_state.node(1).unwrap().halo();
    let state_only = render(&mut only_state).len();

    let mut both = map_with_one_node();
    both.node(1).unwrap().halo();
    both.node(1).unwrap().pulse(Instant::now());
    let both = render(&mut both).len();

    assert!(state_only > baseline);
    assert!(
        both > state_only,
        "a notification must add to the lasting state, not replace it"
    );
}

#[test]
fn color_modifier_reaches_the_painted_shape() {
    // `pulse` paints a filled disc, so the colour is readable straight off it.
    let mut map = map_with_one_node();
    map.node(1)
        .unwrap()
        .color(Color32::RED)
        .pulse(Instant::now());

    let ctx = Context::default();
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400.0, 300.0));
    let mut out = ctx.run_ui(
        RawInput {
            screen_rect: Some(screen),
            ..RawInput::default()
        },
        |ui| {
            ui.add(&mut map);
        },
    );
    let reds = out
        .shapes
        .iter()
        .filter(|cs| match &cs.shape {
            Shape::Circle(c) => c.fill.r() == 255 && c.fill.g() == 0 && c.fill.b() == 0,
            _ => false,
        })
        .count();
    out.textures_delta.clear();

    assert!(
        reds > 0,
        "the requested colour never reached a painted shape"
    );
}

#[test]
fn orbiting_state_moves_over_time() {
    // The orbiting dot is the one effect whose *position* is time-driven, so it
    // is the cheapest end-to-end check that frame time reaches the effect.
    let ctx = Context::default();
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400.0, 300.0));
    let mut map = map_with_one_node();
    map.node(1).unwrap().orbit();

    let mut frames: Vec<Vec<(i32, i32)>> = Vec::new();
    for _ in 0..40 {
        let mut out = ctx.run_ui(
            RawInput {
                screen_rect: Some(screen),
                ..RawInput::default()
            },
            |ui| {
                ui.add(&mut map);
            },
        );
        // The node's own dot never moves, so any change here is the orbit.
        let mut centers: Vec<(i32, i32)> = out
            .shapes
            .iter()
            .filter_map(|cs| match &cs.shape {
                Shape::Circle(c) if c.fill.a() > 0 => {
                    Some(((c.center.x * 10.0) as i32, (c.center.y * 10.0) as i32))
                }
                _ => None,
            })
            .collect();
        centers.sort_unstable();
        frames.push(centers);
        out.textures_delta.clear();
    }

    assert!(
        frames.iter().any(|f| *f != frames[0]),
        "the orbiting dot never moved across 40 frames: {:?}",
        frames[0]
    );
}

#[test]
#[allow(deprecated)]
fn deprecated_notify_still_paints_a_pulse() {
    let baseline = render(&mut map_with_one_node());

    let mut map = map_with_one_node();
    map.notify(1, Instant::now());
    let shapes = render(&mut map);

    assert_eq!(
        extra(&shapes, &baseline),
        1,
        "notify should still paint exactly one pulse disc: {shapes:?}"
    );
    assert!(shapes.contains(&Kind::FilledCircle));

    // and it still tolerates an id that was never loaded
    let mut map = map_with_one_node();
    map.notify(999, Instant::now());
    assert_eq!(render(&mut map).len(), baseline.len());
}

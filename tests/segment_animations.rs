//! Segment animations attached through `Map::segment`, driven end to end.
//!
//! Mirrors `tests/node_animations.rs`: these check what actually reaches the
//! screen -- that `flash`/`comet_once` stop on their own, that `comet` does
//! not, and that state and notification stack in the documented order --
//! rather than just what the handle records (that part is covered by the
//! unit tests in `map.rs`).

use egui::{Color32, Context, RawInput, Shape};
use egui_map::map::Map;
use egui_map::map::animation::CHEVRON_SPEED;
use egui_map::map::objects::{CometDirection, MapSegment};
use std::time::{Duration, Instant};

/// A coarse description of a painted shape, enough to tell the effects apart.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Kind {
    FilledCircle,
    Line,
    Mesh,
    Other,
}

fn render(map: &mut Map) -> Vec<Kind> {
    let ctx = Context::default();
    render_with(&ctx, map)
}

/// Renders one frame on an existing context, so frame time keeps advancing
/// across calls (`RawInput::default()` leaves `time` unset and egui advances
/// it by `predicted_dt`).
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
            Shape::LineSegment { .. } => Kind::Line,
            Shape::Mesh(_) => Kind::Mesh,
            _ => Kind::Other,
        })
        .collect();
    out.textures_delta.clear();
    kinds
}

fn map_with_one_segment() -> Map {
    let mut map = Map::new();
    map.add_lines(vec![MapSegment::new((1, 2), [0.0, 0.0], [50.0, 0.0])]);
    map
}

/// How many shapes the animation added on top of a plain map.
fn extra(with: &[Kind], without: &[Kind]) -> usize {
    with.len().saturating_sub(without.len())
}

#[test]
fn flash_paints_something() {
    let baseline = render(&mut map_with_one_segment());
    let mut map = map_with_one_segment();
    map.segment((1, 2)).unwrap().flash(Instant::now());
    let shapes = render(&mut map);
    assert!(
        extra(&shapes, &baseline) > 0,
        "flash painted nothing on top of the plain map: {shapes:?}"
    );
}

#[test]
fn flash_stops_painting_once_finished() {
    // Past FLASH_DECAY_DURATION but inside the widget's own 10s orphan sweep,
    // so what removes the notification is the effect reporting it finished --
    // not the sweep.
    let long_ago = Instant::now() - Duration::from_secs(2);
    let baseline = render(&mut map_with_one_segment()).len();

    let mut map = map_with_one_segment();
    map.segment((1, 2)).unwrap().flash(long_ago);
    let _ = render(&mut map); // the frame that notices it is over
    assert_eq!(
        render(&mut map).len(),
        baseline,
        "flash kept painting after it should have finished"
    );
}

#[test]
fn comet_once_paints_something_and_stops_on_its_own() {
    let baseline = render(&mut map_with_one_segment());
    let mut map = map_with_one_segment();
    map.segment((1, 2))
        .unwrap()
        .comet_once(Instant::now(), CometDirection::Forward);
    let shapes = render(&mut map);
    assert!(
        extra(&shapes, &baseline) > 0,
        "comet_once painted nothing on top of the plain map: {shapes:?}"
    );
}

#[test]
fn comet_once_stops_painting_once_finished() {
    let long_ago = Instant::now() - Duration::from_secs(2);
    let baseline = render(&mut map_with_one_segment()).len();

    let mut map = map_with_one_segment();
    map.segment((1, 2))
        .unwrap()
        .comet_once(long_ago, CometDirection::Forward);
    let _ = render(&mut map); // the frame that notices it is over
    assert_eq!(
        render(&mut map).len(),
        baseline,
        "comet_once kept painting after it should have finished"
    );
}

#[test]
fn comet_once_direction_starts_from_the_chosen_endpoint() {
    // At the very start of the animation the dot must sit on the starting
    // endpoint -- close to the segment's first rendered point for
    // `Forward`, close to its second for `Reverse` -- not partway along the
    // line. Positions are read in screen space, where the widget centers and
    // offsets the map, so this compares the circle against the *rendered*
    // line endpoints from the same frame rather than assuming the raw
    // map-space coordinates carry over unchanged.
    fn line_and_circle(ctx: &Context, map: &mut Map) -> ([egui::Pos2; 2], egui::Pos2) {
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
        let line = out
            .shapes
            .iter()
            .find_map(|cs| match &cs.shape {
                Shape::LineSegment { points, .. } => Some(*points),
                _ => None,
            })
            .expect("expected the default segment line to be painted");
        let circle = out
            .shapes
            .iter()
            .find_map(|cs| match &cs.shape {
                Shape::Circle(c) if c.fill.a() > 0 => Some(c.center),
                _ => None,
            })
            .expect("expected a filled circle to be painted");
        out.textures_delta.clear();
        (line, circle)
    }

    let ctx = Context::default();
    let mut forward = map_with_one_segment();
    forward
        .segment((1, 2))
        .unwrap()
        .comet_once(Instant::now(), CometDirection::Forward);
    let ([a, b], forward_pos) = line_and_circle(&ctx, &mut forward);

    let ctx = Context::default();
    let mut reverse = map_with_one_segment();
    reverse
        .segment((1, 2))
        .unwrap()
        .comet_once(Instant::now(), CometDirection::Reverse);
    let (_, reverse_pos) = line_and_circle(&ctx, &mut reverse);

    let dist = |p: egui::Pos2, q: egui::Pos2| (p - q).length();
    assert!(
        dist(forward_pos, a) < dist(forward_pos, b),
        "Forward must start closer to the first endpoint {a:?}, got {forward_pos:?} (other endpoint {b:?})"
    );
    assert!(
        dist(reverse_pos, b) < dist(reverse_pos, a),
        "Reverse must start closer to the second endpoint {b:?}, got {reverse_pos:?} (other endpoint {a:?})"
    );
}

#[test]
fn wipe_paints_something() {
    let baseline = render(&mut map_with_one_segment());
    let mut map = map_with_one_segment();
    map.segment((1, 2)).unwrap().wipe(Instant::now());
    let shapes = render(&mut map);
    assert!(
        extra(&shapes, &baseline) > 0,
        "wipe painted nothing on top of the plain map: {shapes:?}"
    );
}

#[test]
fn wipe_stops_painting_once_finished() {
    let long_ago = Instant::now() - Duration::from_secs(2);
    let baseline = render(&mut map_with_one_segment()).len();

    let mut map = map_with_one_segment();
    map.segment((1, 2)).unwrap().wipe(long_ago);
    let _ = render(&mut map); // the frame that notices it is over
    assert_eq!(
        render(&mut map).len(),
        baseline,
        "wipe kept painting after it should have finished"
    );
}

#[test]
fn comet_keeps_painting_and_clear_stops_it() {
    let baseline = render(&mut map_with_one_segment()).len();
    let mut map = map_with_one_segment();
    map.segment((1, 2)).unwrap().comet();

    // unlike a notification, this must survive many frames
    for frame in 0..5 {
        assert!(
            render(&mut map).len() > baseline,
            "comet stopped painting at frame {frame}"
        );
    }

    map.segment((1, 2)).unwrap().clear();
    assert_eq!(
        render(&mut map).len(),
        baseline,
        "comet kept painting after clear()"
    );
}

#[test]
fn dash_paints_a_mesh_and_keeps_painting_until_cleared() {
    let baseline = render(&mut map_with_one_segment());
    let mut map = map_with_one_segment();
    map.segment((1, 2)).unwrap().dash();

    let shapes = render(&mut map);
    assert!(
        shapes.contains(&Kind::Mesh),
        "dash must paint a textured mesh, got {shapes:?}"
    );

    // unlike a notification, this must survive many frames
    for frame in 0..5 {
        assert!(
            render(&mut map).len() > baseline.len(),
            "dash stopped painting at frame {frame}"
        );
    }

    map.segment((1, 2)).unwrap().clear();
    assert_eq!(
        render(&mut map).len(),
        baseline.len(),
        "dash kept painting after clear()"
    );
}

#[test]
fn dash_mesh_is_painted_after_the_default_line_so_it_stays_on_top() {
    // Paint order is stacking order: if the segment's own opaque line were
    // painted after (on top of) its effect, the mesh would be all but
    // invisible even though `dash_paints_a_mesh_and_keeps_painting_until_cleared`
    // above still sees it in `out.shapes`. This is the paint-*order*
    // regression test: it fails if the default line's index ever moves back
    // to after the effect's, as it did when the line was batched into a
    // `shape_vec` and flushed only once after the whole segment loop.
    let mut map = map_with_one_segment();
    map.segment((1, 2)).unwrap().dash();

    let shapes = render(&mut map);
    let line_index = shapes
        .iter()
        .position(|k| *k == Kind::Line)
        .expect("the segment's own line must still be painted");
    let mesh_index = shapes
        .iter()
        .position(|k| *k == Kind::Mesh)
        .expect("dash must paint a textured mesh");

    assert!(
        mesh_index > line_index,
        "the segment's own line must be painted before (under) its effect, got {shapes:?}"
    );
}

#[test]
fn glow_band_paints_a_mesh_and_keeps_painting_until_cleared() {
    let baseline = render(&mut map_with_one_segment());
    let mut map = map_with_one_segment();
    map.segment((1, 2)).unwrap().glow_band();

    let shapes = render(&mut map);
    assert!(
        shapes.contains(&Kind::Mesh),
        "glow_band must paint a textured mesh, got {shapes:?}"
    );

    // unlike a notification, this must survive many frames
    for frame in 0..5 {
        assert!(
            render(&mut map).len() > baseline.len(),
            "glow_band stopped painting at frame {frame}"
        );
    }

    map.segment((1, 2)).unwrap().clear();
    assert_eq!(
        render(&mut map).len(),
        baseline.len(),
        "glow_band kept painting after clear()"
    );
}

#[test]
fn chevrons_paint_a_mesh_and_keep_painting_until_cleared() {
    let baseline = render(&mut map_with_one_segment());
    let mut map = map_with_one_segment();
    map.segment((1, 2)).unwrap().chevrons();

    let shapes = render(&mut map);
    assert!(
        shapes.contains(&Kind::Mesh),
        "chevrons must paint a textured mesh, got {shapes:?}"
    );

    // unlike a notification, this must survive many frames
    for frame in 0..5 {
        assert!(
            render(&mut map).len() > baseline.len(),
            "chevrons stopped painting at frame {frame}"
        );
    }

    map.segment((1, 2)).unwrap().clear();
    assert_eq!(
        render(&mut map).len(),
        baseline.len(),
        "chevrons kept painting after clear()"
    );
}

#[test]
fn chevrons_pattern_slides_toward_the_second_endpoint() {
    // Regression test: the chevrons must visually travel the same way the
    // arrowheads point (`a` -> `b`, matching how `comet`'s `Forward` reads),
    // not crawl backwards while the arrows point forward -- exactly the bug
    // reported and fixed here. The mesh's own `uv.x` at the `a` end (`u0`)
    // must equal `-(time * CHEVRON_SPEED) mod 1`; the sign is what keeps the
    // pattern's motion in agreement with which way the arrows point (see the
    // comment on `Animation::chevrons`), so this pins the sign down instead
    // of only checking that *something* moves.
    fn render_and_capture(ctx: &Context, map: &mut Map) -> (f32, f32) {
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
        let time = ctx.input(|i| i.time) as f32;
        let u0 = out
            .shapes
            .iter()
            .find_map(|cs| match &cs.shape {
                Shape::Mesh(mesh) => mesh.vertices.first().map(|v| v.uv.x),
                _ => None,
            })
            .expect("expected chevrons to paint a textured mesh");
        out.textures_delta.clear();
        (time, u0)
    }

    let ctx = Context::default();
    let mut map = map_with_one_segment();
    map.segment((1, 2)).unwrap().chevrons();

    // The first frame's `ctx.input(|i| i.time)` can still be `0.0`; render
    // once to get a real, nonzero frame time before checking the formula.
    let _ = render_and_capture(&ctx, &mut map);
    let (time, u0) = render_and_capture(&ctx, &mut map);

    let expected = (-(time * CHEVRON_SPEED)).rem_euclid(1.0);
    assert!(
        (u0 - expected).abs() < 1e-4,
        "chevrons' texture coordinate at the first endpoint must be \
         `-(time * CHEVRON_SPEED) mod 1` so the pattern travels toward the \
         second endpoint, got {u0}, expected {expected} at time {time}"
    );
}

#[test]
fn state_and_notification_stack_on_the_same_segment() {
    let baseline = render(&mut map_with_one_segment()).len();

    let mut only_state = map_with_one_segment();
    only_state.segment((1, 2)).unwrap().comet();
    let state_only = render(&mut only_state).len();

    let mut both = map_with_one_segment();
    both.segment((1, 2)).unwrap().comet();
    both.segment((1, 2)).unwrap().flash(Instant::now());
    let both_len = render(&mut both).len();

    assert!(state_only > baseline);
    assert!(
        both_len > state_only,
        "a notification must add to the lasting state, not replace it"
    );
}

#[test]
fn color_modifier_reaches_the_painted_shape() {
    // `flash` paints the segment as a `Shape::LineSegment`, so the colour is
    // readable straight off its stroke.
    let mut map = map_with_one_segment();
    map.segment((1, 2))
        .unwrap()
        .color(Color32::RED)
        .flash(Instant::now());

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
    // Same premultiply caveat as the node colour test: hue survives at any
    // alpha (green/blue stay exactly 0), the exact red channel does not.
    let reds = out
        .shapes
        .iter()
        .filter(|cs| match &cs.shape {
            Shape::LineSegment { stroke, .. } => {
                stroke.color.r() > 0 && stroke.color.g() == 0 && stroke.color.b() == 0
            }
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
fn comet_moves_over_time() {
    // The travelling dot is the one effect whose *position* is time-driven,
    // so it is the cheapest end-to-end check that frame time reaches it.
    // `render`/`render_with` only classify shapes, so this grabs positions
    // directly instead.
    let ctx = Context::default();
    let mut map = map_with_one_segment();
    map.segment((1, 2)).unwrap().comet();

    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400.0, 300.0));
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
        "the comet never moved across 40 frames: {:?}",
        frames[0]
    );
}

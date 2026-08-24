//! Segment animations attached through `Map::segment`, driven end to end.
//!
//! Mirrors `tests/node_animations.rs`: these check what actually reaches the
//! screen -- that `flash` stops on its own, that `comet` does not, and that
//! the two stack in the documented order -- rather than just what the handle
//! records (that part is covered by the unit tests in `map.rs`).

use egui::{Color32, Context, RawInput, Shape};
use egui_map::map::Map;
use egui_map::map::objects::MapSegment;
use std::time::{Duration, Instant};

/// A coarse description of a painted shape, enough to tell the effects apart.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Kind {
    FilledCircle,
    Line,
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

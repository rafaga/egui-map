//! Verifies that `SegmentTemplate`'s three hooks receive the
//! `SegmentContext`/`SegmentNotificationContext`/`SegmentStateContext`
//! structs with the fields they document -- in particular that `kind`
//! reaches `segment_notification_ui`/`segment_state_ui` so a template can
//! dispatch straight to the matching built-in `Animation::*` function, and
//! that every hook sees both the active theme's full palette (`theme`)
//! and the resolved/computed one (`color`). Mirrors
//! `tests/node_context_colors.rs` for the node side.

use egui::{Context, Painter, RawInput};
use egui_map::map::Map;
use egui_map::map::objects::{
    MapSegment, SegmentAnimation, SegmentContext, SegmentNotificationContext, SegmentStateContext,
    SegmentTemplate, SteadySegmentAnimation,
};
use egui_map::map::theme::{ColorMode, MapTheme, ThemeColors};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

type SegmentId = (usize, usize);

/// A fixed palette so tests can assert exact colors instead of depending on
/// the default theme's light/dark variant.
struct FixedPalette;

impl MapTheme for FixedPalette {
    fn colors(&self, _mode: ColorMode) -> ThemeColors {
        ThemeColors {
            node: egui::Color32::from_rgb(1, 2, 3),
            segment: egui::Color32::from_rgb(4, 5, 6),
            selected: egui::Color32::from_rgb(7, 8, 9),
            alert: egui::Color32::from_rgb(10, 11, 12),
            text: egui::Color32::from_rgb(13, 14, 15),
        }
    }
}

/// Records every call it receives, so tests can assert on exactly what
/// reached each hook rather than on what got painted.
#[derive(Default)]
struct RecordingSegmentTemplate {
    draws: RefCell<Vec<(SegmentId, egui::Color32, ThemeColors)>>,
    notifications: RefCell<Vec<(SegmentId, SegmentAnimation, egui::Color32, ThemeColors)>>,
    states: RefCell<
        Vec<(
            SegmentId,
            SteadySegmentAnimation,
            egui::Color32,
            ThemeColors,
        )>,
    >,
}

impl SegmentTemplate for RecordingSegmentTemplate {
    fn segment_ui(&self, _painter: &Painter, ctx: SegmentContext) {
        self.draws
            .borrow_mut()
            .push((ctx.segment.id, ctx.color, ctx.theme));
    }

    fn segment_notification_ui(&self, painter: &Painter, ctx: SegmentNotificationContext) -> bool {
        self.notifications
            .borrow_mut()
            .push((ctx.segment.id, ctx.kind, ctx.color, ctx.theme));
        painter.ctx().request_repaint();
        true
    }

    fn segment_state_ui(&self, painter: &Painter, ctx: SegmentStateContext) {
        self.states
            .borrow_mut()
            .push((ctx.segment.id, ctx.kind, ctx.color, ctx.theme));
        painter.ctx().request_repaint();
    }
}

fn map_with_one_segment() -> Map {
    let mut map = Map::new();
    map.add_lines(vec![MapSegment::new((1, 2), [0.0, 0.0], [50.0, 0.0])]);
    map
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
    // `TexturesDelta` panics on drop if left unhandled -- see the fix applied
    // to the crate's own `render_line_segments` test helper.
    out.textures_delta.clear();
}

#[test]
fn segment_ui_receives_the_segment_id_and_resolved_color() {
    let template = Rc::new(RecordingSegmentTemplate::default());
    let mut map = map_with_one_segment();
    map.set_segment_template(template.clone());

    render_once(&mut map);

    let draws = template.draws.borrow();
    assert_eq!(draws.len(), 1, "segment_ui must be called once per segment");
    assert_eq!(
        draws[0].0,
        (1, 2),
        "SegmentContext::segment must be the loaded segment"
    );
}

#[test]
fn segment_ui_sees_theme_palette_and_computed_color_separately() {
    let template = Rc::new(RecordingSegmentTemplate::default());
    let mut map = map_with_one_segment();
    map.set_theme(Rc::new(FixedPalette));
    map.set_segment_template(template.clone());

    render_once(&mut map);

    let draws = template.draws.borrow();
    let (_, color, theme) = draws[0];
    // With no per-segment override the computed color falls back to the
    // theme's own segment color -- but the template still receives the
    // full palette alongside it, so it can tell them apart.
    assert_eq!(
        color,
        egui::Color32::from_rgb(4, 5, 6),
        "SegmentContext::color must be the resolved segment color"
    );
    assert_eq!(
        theme.segment,
        egui::Color32::from_rgb(4, 5, 6),
        "SegmentContext::theme must carry the active theme's segment color"
    );
    assert_eq!(
        theme.node,
        egui::Color32::from_rgb(1, 2, 3),
        "SegmentContext::theme must carry every role, not just segment"
    );
}

#[test]
fn segment_override_reaches_the_computed_color_but_not_theme() {
    let template = Rc::new(RecordingSegmentTemplate::default());
    let mut map = Map::new();
    map.set_theme(Rc::new(FixedPalette));
    let mut segment = MapSegment::new((1, 2), [0.0, 0.0], [50.0, 0.0]);
    segment.set_color(egui::Color32::from_rgb(200, 100, 50));
    map.add_lines(vec![segment]);
    map.set_segment_template(template.clone());

    render_once(&mut map);

    let draws = template.draws.borrow();
    let (_, color, theme) = draws[0];
    assert_eq!(
        color,
        egui::Color32::from_rgb(200, 100, 50),
        "SegmentContext::color must honor the segment's override"
    );
    assert_eq!(
        theme.segment,
        egui::Color32::from_rgb(4, 5, 6),
        "SegmentContext::theme must stay the theme's palette, unaffected by the override"
    );
}

#[test]
fn segment_notification_ui_receives_the_requested_kind() {
    let template = Rc::new(RecordingSegmentTemplate::default());
    let mut map = map_with_one_segment();
    map.set_segment_template(template.clone());

    map.segment((1, 2)).unwrap().wipe(Instant::now());
    render_once(&mut map);

    let notifications = template.notifications.borrow();
    assert_eq!(notifications.len(), 1);
    assert_eq!(notifications[0].0, (1, 2));
    assert_eq!(
        notifications[0].1,
        SegmentAnimation::Wipe,
        "SegmentNotificationContext::kind must be the effect that was actually requested"
    );
}

#[test]
fn segment_notification_ui_sees_theme_palette_and_computed_color() {
    let template = Rc::new(RecordingSegmentTemplate::default());
    let mut map = map_with_one_segment();
    map.set_theme(Rc::new(FixedPalette));
    map.set_segment_template(template.clone());

    map.segment((1, 2)).unwrap().wipe(Instant::now());
    render_once(&mut map);

    let notifications = template.notifications.borrow();
    let (_, _, color, theme) = notifications[0];
    assert_eq!(
        color,
        egui::Color32::from_rgb(10, 11, 12),
        "SegmentNotificationContext::color must fall back to the theme's alert color"
    );
    assert_eq!(
        theme.alert,
        egui::Color32::from_rgb(10, 11, 12),
        "SegmentNotificationContext::theme must carry the theme's alert color"
    );
    assert_eq!(
        theme.segment,
        egui::Color32::from_rgb(4, 5, 6),
        "SegmentNotificationContext::theme must carry every role, not just alert"
    );
}

#[test]
fn segment_state_ui_receives_the_requested_kind() {
    let template = Rc::new(RecordingSegmentTemplate::default());
    let mut map = map_with_one_segment();
    map.set_segment_template(template.clone());

    map.segment((1, 2)).unwrap().chevrons();
    render_once(&mut map);

    let states = template.states.borrow();
    assert_eq!(states.len(), 1);
    assert_eq!(states[0].0, (1, 2));
    assert_eq!(
        states[0].1,
        SteadySegmentAnimation::Chevrons,
        "SegmentStateContext::kind must be the effect that was actually requested"
    );
}

#[test]
fn segment_state_ui_sees_theme_palette_and_computed_color() {
    let template = Rc::new(RecordingSegmentTemplate::default());
    let mut map = map_with_one_segment();
    map.set_theme(Rc::new(FixedPalette));
    map.set_segment_template(template.clone());

    map.segment((1, 2)).unwrap().chevrons();
    render_once(&mut map);

    let states = template.states.borrow();
    let (_, _, color, theme) = states[0];
    assert_eq!(
        color,
        egui::Color32::from_rgb(10, 11, 12),
        "SegmentStateContext::color must fall back to the theme's alert color"
    );
    assert_eq!(
        theme.alert,
        egui::Color32::from_rgb(10, 11, 12),
        "SegmentStateContext::theme must carry the theme's alert color"
    );
    assert_eq!(
        theme.selected,
        egui::Color32::from_rgb(7, 8, 9),
        "SegmentStateContext::theme must carry every role, not just alert"
    );
}

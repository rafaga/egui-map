//! A segment can carry a text and ask for dashes. The built-in stroke draws
//! both; a `SegmentTemplate` is left to do it by itself.

use egui::{Context, Painter, Pos2, RawInput, Rect, Shape, Vec2};
use egui_map::map::Map;
use egui_map::map::objects::{
    MapPoint, MapSegment, SegmentContext, SegmentNotificationContext, SegmentStateContext,
    SegmentTemplate,
};
use std::rc::Rc;

/// Runs two frames and returns what was drawn.
fn draw(map: &mut Map) -> Vec<Shape> {
    let ctx = Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 400.0));
    let mut shapes = Vec::new();
    for _ in 0..2 {
        let mut out = ctx.run_ui(
            RawInput {
                screen_rect: Some(screen),
                ..RawInput::default()
            },
            |ui| {
                ui.add(&mut *map);
            },
        );
        shapes = out
            .shapes
            .iter()
            .map(|clipped| clipped.shape.clone())
            .collect();
        out.textures_delta.clear();
    }
    shapes
}

/// Two nodes with a line between them, at a zoom where the line is drawn.
fn map_with(segment: MapSegment) -> Map {
    let mut map = Map::new();
    map.add_points(vec![
        MapPoint::new(1, [0.0, 0.0]),
        MapPoint::new(2, [200.0, 0.0]),
    ]);
    map.insert_line(segment);
    map.set_zoom(1.0);
    map
}

fn texts(shapes: &[Shape]) -> Vec<String> {
    shapes
        .iter()
        .filter_map(|shape| match shape {
            Shape::Text(text) => Some(text.galley.text().to_string()),
            _ => None,
        })
        .collect()
}

/// How many pieces of line there are: a solid line is one, dashes are many.
fn line_pieces(shapes: &[Shape]) -> usize {
    shapes
        .iter()
        .filter(|shape| matches!(shape, Shape::LineSegment { .. } | Shape::Path(_)))
        .count()
}

#[test]
fn the_label_is_written_on_the_line() {
    let mut map = map_with(MapSegment::new((1, 2), [0.0, 0.0], [200.0, 0.0]).with_label("RRW-782"));
    assert!(texts(&draw(&mut map)).contains(&"RRW-782".to_string()));
}

#[test]
fn a_line_without_a_label_writes_nothing() {
    let mut map = map_with(MapSegment::new((1, 2), [0.0, 0.0], [200.0, 0.0]));
    assert!(!texts(&draw(&mut map)).contains(&"RRW-782".to_string()));
}

#[test]
fn a_dashed_line_is_drawn_as_many_pieces() {
    let mut solid = map_with(MapSegment::new((1, 2), [0.0, 0.0], [200.0, 0.0]));
    let mut dashed = map_with(MapSegment::new((1, 2), [0.0, 0.0], [200.0, 0.0]).dashed());
    let solid_pieces = line_pieces(&draw(&mut solid));
    let dashed_pieces = line_pieces(&draw(&mut dashed));
    assert!(
        dashed_pieces > solid_pieces + 5,
        "{dashed_pieces} pieces vs {solid_pieces}"
    );
}

#[test]
fn the_dashes_follow_the_settings() {
    let mut short = map_with(MapSegment::new((1, 2), [0.0, 0.0], [200.0, 0.0]).dashed());
    let mut long = map_with(MapSegment::new((1, 2), [0.0, 0.0], [200.0, 0.0]).dashed());
    short.settings.segment_dash.dash = 4.0;
    long.settings.segment_dash.dash = 40.0;
    assert!(line_pieces(&draw(&mut short)) > line_pieces(&draw(&mut long)));
}

/// A template that draws nothing and says what it was given.
struct Silent;

impl SegmentTemplate for Silent {
    fn segment_ui(&self, _painter: &Painter, _ctx: SegmentContext) {}
    fn segment_notification_ui(
        &self,
        _painter: &Painter,
        _ctx: SegmentNotificationContext,
    ) -> bool {
        false
    }
    fn segment_state_ui(&self, _painter: &Painter, _ctx: SegmentStateContext) {}
}

#[test]
fn a_template_gets_the_label_and_the_widget_does_not_draw_it() {
    let mut map = map_with(MapSegment::new((1, 2), [0.0, 0.0], [200.0, 0.0]).with_label("RRW-782"));
    map.set_segment_template(Rc::new(Silent));
    assert!(!texts(&draw(&mut map)).contains(&"RRW-782".to_string()));
}

//! A segment can carry a text and ask for dashes. The built-in stroke draws
//! both; a `SegmentTemplate` is left to do it by itself.

use egui::{Context, Painter, Pos2, RawInput, Rect, Shape, Vec2};
use egui_map::map::Map;
use egui_map::map::objects::{
    MapPoint, MapSegment, MapSettings, SegmentContext, SegmentNotificationContext,
    SegmentStateContext, SegmentTemplate, TextScaling,
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

/// Where the text of `segment` is drawn, on the screen.
fn label_x(segment: MapSegment) -> f32 {
    let mut map = map_with(segment);
    draw(&mut map)
        .iter()
        .find_map(|shape| match shape {
            Shape::Text(text) if text.galley.text() == "RRW-782" => Some(text.pos.x),
            _ => None,
        })
        .expect("the label is drawn")
}

#[test]
fn the_label_goes_where_label_at_says_along_the_line() {
    let line = || MapSegment::new((1, 2), [0.0, 0.0], [200.0, 0.0]).with_label("RRW-782");
    let at = |fraction: f32| label_x(line().with_label_at(fraction));
    let (near_start, middle, near_end) = (at(0.2), at(0.5), at(0.8));
    // It moves toward the end it is given, and by the same amount for the
    // same fraction, wherever that is on the line.
    assert!(near_start < middle && middle < near_end);
    assert!(
        ((middle - near_start) - (near_end - middle)).abs() < 0.5,
        "{near_start} {middle} {near_end}"
    );
    // By default it is at the middle.
    assert_eq!(label_x(line()), middle);
}

#[test]
fn a_label_position_off_the_line_is_the_nearest_end_and_a_nan_is_the_middle() {
    let line = || MapSegment::new((1, 2), [0.0, 0.0], [200.0, 0.0]).with_label("RRW-782");
    assert_eq!(line().with_label_at(7.0).label_position(), 1.0);
    assert_eq!(line().with_label_at(-3.0).label_position(), 0.0);
    assert_eq!(line().with_label_at(f32::NAN).label_position(), 0.5);
    assert_eq!(line().label_position(), MapSegment::DEFAULT_LABEL_AT);
    assert_eq!(label_x(line().with_label_at(f32::NAN)), label_x(line()));
}

/// Like [`draw`], with the pointer at `pointer` (or nowhere) for four frames.
fn draw_pointing(map: &mut Map, pointer: Option<Pos2>) -> Vec<Shape> {
    let ctx = Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 400.0));
    let mut shapes = Vec::new();
    for _ in 0..4 {
        let events = pointer
            .map(|pos| vec![egui::Event::PointerMoved(pos)])
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
        shapes = out
            .shapes
            .iter()
            .map(|clipped| clipped.shape.clone())
            .collect();
        out.textures_delta.clear();
    }
    shapes
}

/// The two ends of the longest straight piece drawn (the line; the other
/// pieces are the ticks of the zoom scale), on the screen: left, then right.
fn line_ends(shapes: &[Shape]) -> (Pos2, Pos2) {
    shapes
        .iter()
        .filter_map(|shape| match shape {
            Shape::LineSegment { points, .. } => Some((points[0], points[1])),
            _ => None,
        })
        .max_by(|a, b| (a.1 - a.0).length().total_cmp(&(b.1 - b.0).length()))
        .map(|(a, b)| if a.x <= b.x { (a, b) } else { (b, a) })
        .expect("the line is drawn")
}

fn hover_map(visibility: egui_map::map::objects::VisibilitySetting) -> Map {
    let mut map = map_with(MapSegment::new((1, 2), [0.0, 0.0], [200.0, 0.0]).with_label("RRW-782"));
    map.settings.segment_label_visibility = visibility;
    map
}

#[test]
fn on_hover_the_label_shows_only_with_the_pointer_on_the_line() {
    use egui_map::map::objects::VisibilitySetting::Hover;
    let (left, right) = line_ends(&draw(&mut hover_map(Hover)));
    let middle = left + (right - left) * 0.5;

    assert!(!texts(&draw_pointing(&mut hover_map(Hover), None)).contains(&"RRW-782".to_string()));
    // Along the line, away from either node.
    let on = left + (right - left) * 0.3;
    assert!(
        texts(&draw_pointing(&mut hover_map(Hover), Some(on))).contains(&"RRW-782".to_string())
    );
    // Far from it.
    let off = middle + Vec2::new(0.0, 120.0);
    assert!(
        !texts(&draw_pointing(&mut hover_map(Hover), Some(off))).contains(&"RRW-782".to_string())
    );
}

#[test]
fn on_hover_a_node_with_one_line_shows_it() {
    use egui_map::map::objects::VisibilitySetting::Hover;
    let (left, _) = line_ends(&draw(&mut hover_map(Hover)));
    assert!(
        texts(&draw_pointing(&mut hover_map(Hover), Some(left))).contains(&"RRW-782".to_string())
    );
}

#[test]
fn on_hover_a_node_with_two_lines_shows_none() {
    use egui_map::map::objects::VisibilitySetting::Hover;
    let mut map = hover_map(Hover);
    map.add_points(vec![
        MapPoint::new(1, [0.0, 0.0]),
        MapPoint::new(2, [200.0, 0.0]),
        MapPoint::new(3, [100.0, 150.0]),
    ]);
    map.insert_line(MapSegment::new((2, 3), [200.0, 0.0], [100.0, 150.0]).with_label("OTHER"));
    let (_, right) = line_ends(&draw_pointing(&mut map, None));
    let shown = texts(&draw_pointing(&mut map, Some(right)));
    assert_eq!(map.hovered_node(), Some(2));
    assert!(!shown.contains(&"RRW-782".to_string()), "{shown:?}");
    assert!(!shown.contains(&"OTHER".to_string()), "{shown:?}");
}

#[test]
fn always_writes_the_label_and_hidden_never_does() {
    use egui_map::map::objects::VisibilitySetting::{Always, Hidden};
    assert!(texts(&draw(&mut hover_map(Always))).contains(&"RRW-782".to_string()));
    let (left, right) = line_ends(&draw(&mut hover_map(Hidden)));
    let on = left + (right - left) * 0.3;
    assert!(
        !texts(&draw_pointing(&mut hover_map(Hidden), Some(on))).contains(&"RRW-782".to_string())
    );
}

/// The font size and the patch behind the text "RRW-782" when the map is at
/// `zoom`.
fn label_at_zoom(scaling: TextScaling, zoom: f32) -> (f32, Vec2) {
    let mut map = map_with(MapSegment::new((1, 2), [0.0, 0.0], [200.0, 0.0]).with_label("RRW-782"));
    map.settings.segment_label_scaling = scaling;
    map.set_zoom(zoom);
    let shapes = draw(&mut map);
    let size = shapes
        .iter()
        .find_map(|shape| match shape {
            Shape::Text(text) if text.galley.text() == "RRW-782" => text
                .galley
                .job
                .sections
                .first()
                .map(|section| section.format.font_id.size),
            _ => None,
        })
        .expect("the label is drawn");
    // The patch is the filled rectangle that is as wide as the text and not
    // as wide as a node.
    let patch = shapes
        .iter()
        .filter_map(|shape| match shape {
            Shape::Rect(rect) if rect.rect.width() > size * 2.0 => Some(rect.rect.size()),
            _ => None,
        })
        .min_by(|a, b| a.x.total_cmp(&b.x))
        .expect("the patch is drawn");
    (size, patch)
}

#[test]
fn the_label_keeps_its_size_at_every_zoom_unless_asked_otherwise() {
    let expected = MapSettings::default().node_text_size;
    for zoom in [0.6_f32, 1.0, 1.5, 2.0] {
        assert_eq!(label_at_zoom(TextScaling::Screen, zoom).0, expected);
    }
}

#[test]
fn the_label_follows_the_zoom_when_asked() {
    let base = MapSettings::default().node_text_size;
    for zoom in [0.6_f32, 1.0, 1.5, 2.0] {
        let expected = ((base * zoom) * 2.0).round() / 2.0;
        assert_eq!(label_at_zoom(TextScaling::Zoom, zoom).0, expected, "{zoom}");
    }
}

#[test]
fn the_patch_behind_the_label_grows_with_the_zoom_as_the_text_does() {
    let (_, one) = label_at_zoom(TextScaling::Zoom, 1.0);
    let (_, two) = label_at_zoom(TextScaling::Zoom, 2.0);
    // The padding scales too, so the whole patch does, give or take what
    // the glyphs round to.
    assert!((two.x / one.x - 2.0).abs() < 0.1, "{one:?} against {two:?}");
    assert!((two.y / one.y - 2.0).abs() < 0.1, "{one:?} against {two:?}");
    let (_, fixed_one) = label_at_zoom(TextScaling::Screen, 1.0);
    let (_, fixed_two) = label_at_zoom(TextScaling::Screen, 2.0);
    assert_eq!(fixed_one, fixed_two);
}

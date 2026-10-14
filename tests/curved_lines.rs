//! Verifies the curved connection lines: a segment is straight until it is
//! made curved, `MapSettings::default_curvature` only applies to the curves
//! without an amount of their own, and the effects and the hit-test follow the
//! curve.

use egui::{Context, Pos2, RawInput, Rect, Shape, vec2};
use egui_map::map::Map;
use egui_map::map::objects::{LineFade, LineShape, MapPoint, MapSegment};

/// One frame of a map with a single line from `(0, 0)` to `(100, 0)`.
struct Scene;

impl Scene {
    fn map(configure: impl FnOnce(&mut Map)) -> Map {
        let mut map = Map::new();
        map.add_points(vec![
            MapPoint::new(1, [0.0, 0.0]),
            MapPoint::new(2, [100.0, 0.0]),
        ]);
        map.add_lines(vec![MapSegment::new((1, 2), [0.0, 0.0], [100.0, 0.0])]);
        map.settings.line_fade = LineFade::None;
        map.settings.slider.ticks = false;
        configure(&mut map);
        map
    }

    /// The shapes painted in one frame of `map`.
    fn shapes(map: &mut Map) -> Vec<Shape> {
        let mut out = Context::default().run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
                ..RawInput::default()
            },
            |ui| {
                ui.add(&mut *map);
            },
        );
        out.textures_delta.clear();
        out.shapes
            .into_iter()
            .map(|clipped| clipped.shape)
            .collect()
    }
}

#[test]
fn lines_are_straight_by_default() {
    let mut map = Scene::map(|_| {});
    let shapes = Scene::shapes(&mut map);
    assert!(
        shapes
            .iter()
            .any(|s| matches!(s, Shape::LineSegment { .. }))
    );
    assert!(
        !shapes
            .iter()
            .any(|s| matches!(s, Shape::QuadraticBezier(_)))
    );
}

#[test]
fn the_default_curvature_does_not_touch_straight_lines() {
    let mut map = Scene::map(|map| map.settings.default_curvature = 0.8);
    let shapes = Scene::shapes(&mut map);
    assert!(
        shapes
            .iter()
            .any(|s| matches!(s, Shape::LineSegment { .. }))
    );
    assert!(
        !shapes
            .iter()
            .any(|s| matches!(s, Shape::QuadraticBezier(_)))
    );
}

#[test]
fn straight_and_curved_lines_share_a_map() {
    let mut map = Map::new();
    map.add_points(vec![
        MapPoint::new(1, [0.0, 0.0]),
        MapPoint::new(2, [100.0, 0.0]),
        MapPoint::new(3, [0.0, 100.0]),
    ]);
    let curved = MapSegment::curved((1, 3), [0.0, 0.0], [0.0, 100.0], None);
    map.add_lines(vec![
        MapSegment::new((1, 2), [0.0, 0.0], [100.0, 0.0]),
        curved,
    ]);
    map.settings.line_fade = LineFade::None;
    map.settings.slider.ticks = false;
    let shapes = Scene::shapes(&mut map);
    assert_eq!(
        shapes
            .iter()
            .filter(|s| matches!(s, Shape::LineSegment { .. }))
            .count(),
        1,
        "the straight line"
    );
    assert_eq!(
        shapes
            .iter()
            .filter(|s| matches!(s, Shape::QuadraticBezier(_)))
            .count(),
        1,
        "the curved line"
    );
}

#[test]
fn a_curve_without_an_amount_uses_the_default_curvature() {
    let control_offset = |default: f32| {
        let mut map = Scene::map(|map| {
            map.settings.default_curvature = default;
            let segment = MapSegment::curved((1, 2), [0.0, 0.0], [100.0, 0.0], None);
            assert_eq!(segment.shape(), LineShape::Curved(None));
            map.add_lines(vec![segment]);
        });
        Scene::shapes(&mut map)
            .iter()
            .find_map(|s| match s {
                Shape::QuadraticBezier(curve) => Some(curve.points[1].y - curve.points[0].y),
                _ => None,
            })
            .expect("the line is a curve")
    };
    let gentle = control_offset(0.1);
    let deep = control_offset(0.4);
    assert!(gentle > 0.0);
    assert!(
        deep > gentle * 3.0,
        "{deep} should be about four times {gentle}"
    );
}

#[test]
fn a_curves_own_amount_ignores_the_default() {
    let mut map = Scene::map(|map| {
        map.settings.default_curvature = 0.9;
        let segment = MapSegment::curved((1, 2), [0.0, 0.0], [100.0, 0.0], Some(-0.2));
        map.add_lines(vec![segment]);
    });
    let offset = Scene::shapes(&mut map)
        .iter()
        .find_map(|s| match s {
            Shape::QuadraticBezier(curve) => Some(curve.points[1].y - curve.points[0].y),
            _ => None,
        })
        .expect("the line is a curve");
    // Negative bends to the left of an eastward line, which is up; and by
    // `-0.2` of the 100 units between the nodes, not by the default.
    assert!(offset < 0.0);
    assert!(offset.abs() < 30.0, "{offset}");
}

#[test]
fn the_shape_is_decided_when_the_segment_is_built() {
    let straight = MapSegment::new((1, 2), [0.0, 0.0], [100.0, 0.0]);
    assert_eq!(straight.shape(), LineShape::Straight);
    assert_eq!(
        straight,
        MapSegment::straight((1, 2), [0.0, 0.0], [100.0, 0.0])
    );
    let curved = MapSegment::curved((1, 2), [0.0, 0.0], [100.0, 0.0], Some(0.3));
    assert_eq!(curved.shape(), LineShape::Curved(Some(0.3)));
    let default = MapSegment::curved((1, 2), [0.0, 0.0], [100.0, 0.0], None);
    assert_eq!(default.shape(), LineShape::Curved(None));
}

#[test]
fn only_a_curve_can_change_its_curvature() {
    let mut curved = MapSegment::curved((1, 2), [0.0, 0.0], [100.0, 0.0], Some(0.3));
    assert!(curved.set_curvature(Some(-0.1)));
    assert_eq!(curved.shape(), LineShape::Curved(Some(-0.1)));
    // `None` goes back to the default of the map, and it stays a curve.
    assert!(curved.set_curvature(None));
    assert_eq!(curved.shape(), LineShape::Curved(None));

    // A straight line stays straight: its shape is decided when it is built.
    let mut straight = MapSegment::new((1, 2), [0.0, 0.0], [100.0, 0.0]);
    assert!(!straight.set_curvature(Some(0.3)));
    assert_eq!(straight.shape(), LineShape::Straight);
    let mut map = Scene::map(|map| map.add_lines(vec![straight]));
    let shapes = Scene::shapes(&mut map);
    assert!(
        !shapes
            .iter()
            .any(|s| matches!(s, Shape::QuadraticBezier(_)))
    );
}

#[test]
fn opposite_curvatures_bend_to_opposite_sides() {
    let control_y = |curvature: f32| {
        let mut map = Scene::map(|map| {
            let segment = MapSegment::curved((1, 2), [0.0, 0.0], [100.0, 0.0], Some(curvature));
            map.add_lines(vec![segment]);
        });
        Scene::shapes(&mut map)
            .iter()
            .find_map(|s| match s {
                Shape::QuadraticBezier(curve) => Some(curve.points[1].y - curve.points[0].y),
                _ => None,
            })
            .expect("the line is a curve")
    };
    assert!(control_y(0.3) > 0.0);
    assert!(control_y(-0.3) < 0.0);
}

#[test]
fn a_dash_follows_the_curve() {
    let mut straight = Scene::map(|map| {
        map.segment((1, 2)).expect("segment is loaded").dash();
    });
    let mut curved = Scene::map(|map| {
        let segment = MapSegment::curved((1, 2), [0.0, 0.0], [100.0, 0.0], Some(0.5));
        map.add_lines(vec![segment]);
        map.segment((1, 2)).expect("segment is loaded").dash();
    });
    let ribbon = |shapes: Vec<Shape>| {
        shapes
            .into_iter()
            .find_map(|s| match s {
                Shape::Mesh(mesh) => Some(mesh),
                _ => None,
            })
            .expect("the dash paints a mesh")
    };
    let straight = ribbon(Scene::shapes(&mut straight));
    let curved = ribbon(Scene::shapes(&mut curved));
    // A straight ribbon is a single quad; a curved one has a pair of
    // vertices per piece of the curve, and leaves the line of the chord.
    assert_eq!(straight.vertices.len(), 4);
    assert!(curved.vertices.len() > 4);
    let (min, max) = curved
        .vertices
        .iter()
        .fold((f32::MAX, f32::MIN), |(min, max), v| {
            (min.min(v.pos.y), max.max(v.pos.y))
        });
    assert!(max - min > 10.0, "the ribbon bows away from the chord");
}

#[test]
fn line_at_measures_against_the_curve_only_for_curves() {
    let map = Scene::map(|map| {
        let segment = MapSegment::curved((1, 2), [0.0, 0.0], [100.0, 0.0], Some(0.5));
        map.add_lines(vec![segment]);
    });
    // The peak of the curve sits 25 map units off the chord, at its middle.
    assert_eq!(map.line_at([50.0, 25.0], 2.0), Some((1, 2)));
    // The chord itself is nowhere near the curve any more.
    assert_eq!(map.line_at([50.0, 0.0], 2.0), None);

    // A straight line is still measured against its chord, whatever the
    // default curvature of the map.
    let map = Scene::map(|map| map.settings.default_curvature = 0.5);
    assert_eq!(map.line_at([50.0, 0.0], 2.0), Some((1, 2)));
    assert_eq!(map.line_at([50.0, 25.0], 2.0), None);
}

#[test]
fn a_curve_outside_the_view_is_not_painted() {
    let mut map = Map::new();
    map.add_points(vec![
        MapPoint::new(1, [0.0, 0.0]),
        MapPoint::new(2, [100.0, 0.0]),
        MapPoint::new(3, [100_000.0, 100_000.0]),
        MapPoint::new(4, [100_100.0, 100_000.0]),
    ]);
    let far = MapSegment::curved(
        (3, 4),
        [100_000.0, 100_000.0],
        [100_100.0, 100_000.0],
        Some(0.4),
    );
    map.add_lines(vec![MapSegment::new((1, 2), [0.0, 0.0], [100.0, 0.0]), far]);
    map.settings.line_fade = LineFade::None;
    map.settings.slider.ticks = false;
    let shapes = Scene::shapes(&mut map);
    assert!(
        !shapes
            .iter()
            .any(|s| matches!(s, Shape::QuadraticBezier(_)))
    );
}

//! Verifies the width of the connection lines: `Style::line_width` is the width
//! at the standard zoom, and the lines scale with the zoom from there.

use egui::{Context, Pos2, RawInput, Rect, Shape, vec2};
use egui_map::map::Map;
use egui_map::map::objects::{LineFade, MapPoint, MapSegment};

/// One frame of a map with a single line, at `zoom`.
struct Scene;

impl Scene {
    /// The stroke width of the line, if it is drawn.
    fn width(zoom: f32, configure: impl FnOnce(&mut Map)) -> Option<f32> {
        let mut map = Map::new();
        map.add_points(vec![
            MapPoint::new(1, [0.0, 0.0]),
            MapPoint::new(2, [100.0, 0.0]),
        ]);
        map.add_lines(vec![MapSegment::new((1, 2), [0.0, 0.0], [100.0, 0.0])]);
        map.settings.line_fade = LineFade::None;
        map.settings.slider.ticks = false;
        configure(&mut map);
        map.set_zoom(zoom);

        let mut out = Context::default().run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
                ..RawInput::default()
            },
            |ui| {
                ui.add(&mut map);
            },
        );
        out.textures_delta.clear();
        out.shapes.iter().find_map(|clipped| match &clipped.shape {
            Shape::LineSegment { stroke, .. } => Some(stroke.width),
            _ => None,
        })
    }
}

#[test]
fn the_default_line_is_two_pixels_at_the_standard_zoom() {
    assert_eq!(Scene::width(1.0, |_| {}), Some(2.0));
}

#[test]
fn the_line_grows_and_shrinks_with_the_zoom() {
    assert_eq!(Scene::width(2.0, |_| {}), Some(4.0));
    assert_eq!(Scene::width(1.5, |_| {}), Some(3.0));
}

#[test]
fn zoomed_out_the_line_gets_thinner_than_a_pixel() {
    assert_eq!(Scene::width(0.5, |_| {}), Some(1.0));
    assert_eq!(Scene::width(0.25, |_| {}), Some(0.5));
}

#[test]
fn the_configured_width_is_the_one_at_the_standard_zoom() {
    let thick = |map: &mut Map| map.settings.style.line_width = Some(5.0);
    assert_eq!(Scene::width(1.0, thick), Some(5.0));
    assert_eq!(Scene::width(2.0, thick), Some(10.0));
}

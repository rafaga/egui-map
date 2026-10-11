//! Verifies `MapSettings::line_fade`: the default ramp is unchanged, and
//! `LineFade::None` draws the lines fully opaque as soon as they appear.

use egui::{Color32, Context, Pos2, RawInput, Rect, Shape, vec2};
use egui_map::map::Map;
use egui_map::map::objects::{LineFade, MapPoint, MapSegment};
use egui_map::map::theme::{ColorMode, MapTheme, ThemeColors};
use std::rc::Rc;

const SEGMENT: Color32 = Color32::from_rgb(200, 60, 60);

/// A palette whose segment color is opaque, so any alpha seen on screen comes
/// from the fade.
struct Palette;

impl MapTheme for Palette {
    fn colors(&self, _mode: ColorMode) -> ThemeColors {
        ThemeColors {
            node: Color32::from_rgb(10, 200, 30),
            segment: SEGMENT,
            selected: Color32::from_rgb(30, 60, 220),
            alert: Color32::from_rgb(250, 150, 0),
            marker: Color32::from_rgb(250, 0, 250),
            text: Color32::WHITE,
            background: Color32::from_rgb(20, 20, 40),
        }
    }
}

/// A map with one segment, drawn at `zoom`, and the color it was stroked with.
struct Scene;

impl Scene {
    fn line_color(fade: Option<LineFade>, zoom: f32) -> Option<Color32> {
        let mut map = Map::new();
        map.add_points(vec![
            MapPoint::new(1, [0.0, 0.0]),
            MapPoint::new(2, [100.0, 0.0]),
        ]);
        map.add_lines(vec![MapSegment::new((1, 2), [0.0, 0.0], [100.0, 0.0])]);
        map.set_theme(Rc::new(Palette));
        if let Some(fade) = fade {
            map.settings.line_fade = fade;
        }
        map.set_zoom(zoom);
        // The slider's tick marks are line segments too; look at the map's.
        map.settings.slider.ticks = false;

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
        out.shapes.iter().find_map(|clipped| match &clipped.shape {
            Shape::LineSegment { stroke, .. } => Some(stroke.color),
            _ => None,
        })
    }
}

#[test]
fn by_default_the_lines_fade_in_with_the_zoom() {
    // `line_visible_zoom` is 0.2 and the ramp 0.8 long: zoom 0.6 is half way.
    let half = Scene::line_color(None, 0.6).expect("the line is drawn");
    assert!(
        (i32::from(half.a()) - 128).abs() <= 2,
        "alpha {} should be about half",
        half.a()
    );
    let full = Scene::line_color(None, 1.0).expect("the line is drawn");
    assert_eq!(full, SEGMENT);
}

#[test]
fn without_a_fade_the_lines_are_opaque() {
    for zoom in [0.25, 0.6, 1.0] {
        assert_eq!(
            Scene::line_color(Some(LineFade::None), zoom),
            Some(SEGMENT),
            "zoom {zoom}"
        );
    }
}

#[test]
fn without_a_fade_there_is_no_threshold_either() {
    // At and below `line_visible_zoom` (0.2) the lines are still drawn.
    for zoom in [0.2, 0.15, 0.1] {
        assert_eq!(
            Scene::line_color(Some(LineFade::None), zoom),
            Some(SEGMENT),
            "zoom {zoom}"
        );
    }
}

#[test]
fn with_a_fade_the_lines_are_hidden_at_the_threshold() {
    assert_eq!(Scene::line_color(None, 0.2), None);
    assert_eq!(
        Scene::line_color(Some(LineFade::Zoom { range: 0.4 }), 0.2),
        None
    );
}

#[test]
fn a_custom_range_changes_the_ramp() {
    // A range of 0.4: zoom 0.4 is half way up from 0.2.
    let half = Scene::line_color(Some(LineFade::Zoom { range: 0.4 }), 0.4).unwrap();
    assert!((i32::from(half.a()) - 128).abs() <= 2, "alpha {}", half.a());
}

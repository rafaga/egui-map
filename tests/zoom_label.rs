//! Verifies the zoom read-out under the built-in zoom slider.

use egui::{Color32, Context, Pos2, RawInput, Rect, Shape, vec2};
use egui_map::map::Map;
use egui_map::map::objects::MapPoint;
use egui_map::map::theme::{ColorMode, MapTheme, ThemeColors};
use std::rc::Rc;

const TEXT: Color32 = Color32::from_rgb(240, 230, 120);

/// A palette whose text color is easy to tell from every other.
struct Palette;

impl MapTheme for Palette {
    fn colors(&self, _mode: ColorMode) -> ThemeColors {
        ThemeColors {
            node: Color32::from_rgb(10, 200, 30),
            segment: Color32::from_rgb(200, 60, 60),
            selected: Color32::from_rgb(30, 60, 220),
            alert: Color32::from_rgb(250, 150, 0),
            marker: Color32::from_rgb(250, 0, 250),
            text: TEXT,
            background: Color32::from_rgb(20, 20, 40),
        }
    }
}

/// A map drawn at a zoom, and what was painted.
struct Scene {
    widget: Rect,
    texts: Vec<(String, Pos2, Color32)>,
}

impl Scene {
    fn at_zoom(zoom: f32) -> Self {
        let mut map = Map::new();
        map.add_points(vec![MapPoint::new(1, [0.0, 0.0])]);
        map.set_theme(Rc::new(Palette));
        map.set_zoom(zoom);

        let mut widget = Rect::NOTHING;
        let mut out = Context::default().run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
                ..RawInput::default()
            },
            |ui| {
                widget = ui.add(&mut map).rect;
            },
        );
        out.textures_delta.clear();
        let texts = out
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                Shape::Text(text) => Some((
                    text.galley.text().to_string(),
                    text.pos,
                    text.override_text_color.unwrap_or(text.fallback_color),
                )),
                _ => None,
            })
            .collect();
        Self { widget, texts }
    }

    fn label(&self, text: &str) -> Option<&(String, Pos2, Color32)> {
        self.texts.iter().find(|(shown, ..)| shown == text)
    }
}

#[test]
fn the_zoom_is_shown_as_a_percentage() {
    for (zoom, label) in [(1.0, "100%"), (0.5, "50%"), (2.0, "200%"), (0.1, "10%")] {
        let scene = Scene::at_zoom(zoom);
        assert!(
            scene.label(label).is_some(),
            "zoom {zoom}: no `{label}` in {:?}",
            scene.texts
        );
    }
}

#[test]
fn the_label_sits_under_the_slider_in_the_top_right_corner() {
    let scene = Scene::at_zoom(1.0);
    let (_, pos, _) = scene.label("100%").expect("the zoom label");
    // The control is 44 px wide and starts 10 px from the corner, and the
    // label is under the 100 px rail, centred on it.
    assert!(
        pos.x > scene.widget.right() - 60.0 && pos.x < scene.widget.right() - 10.0,
        "{pos:?} is not near the right edge of {:?}",
        scene.widget
    );
    assert!(
        pos.y > scene.widget.top() + 100.0 && pos.y < scene.widget.top() + 140.0,
        "{pos:?} is not right under the rail of a control at the top of {:?}",
        scene.widget
    );
}

#[test]
fn the_label_uses_the_theme_text_color() {
    let scene = Scene::at_zoom(1.0);
    let (_, _, color) = scene.label("100%").expect("the zoom label");
    assert_eq!(*color, TEXT);
}

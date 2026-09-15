//! Node names and free-floating labels must be painted with the active
//! [`MapTheme`]'s [`ThemeColors::text`], not with egui's own surrounding-UI
//! text color -- otherwise a custom theme's `text` role has no visible
//! effect anywhere. Regression test for that gap: before this fix, both
//! label paths read `ui.visuals().text_color()` instead (and
//! `NodeContext::text_color`, when a `NodeTemplate` is installed, used a
//! hardcoded black/white).

use egui::{Color32, Context, RawInput, Shape};
use egui_map::map::Map;
use egui_map::map::objects::{MapLabel, MapPoint, MapSettings, VisibilitySetting};
use egui_map::map::theme::{ColorMode, MapTheme, ThemeColors};

const THEME_TEXT: Color32 = Color32::from_rgb(200, 20, 200);

struct FixedPalette;

impl MapTheme for FixedPalette {
    fn colors(&self, _mode: ColorMode) -> ThemeColors {
        ThemeColors {
            node: Color32::from_rgb(1, 2, 3),
            segment: Color32::from_rgb(4, 5, 6),
            selected: Color32::from_rgb(7, 8, 9),
            alert: Color32::from_rgb(10, 11, 12),
            text: THEME_TEXT,
        }
    }
}

/// Renders one frame and returns the color of every non-empty text shape
/// drawn.
fn drawn_text_colors(map: &mut Map) -> Vec<Color32> {
    let ctx = Context::default();
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(600.0, 400.0));
    let mut out = ctx.run_ui(
        RawInput {
            screen_rect: Some(screen),
            ..RawInput::default()
        },
        |ui| {
            ui.add(&mut *map);
        },
    );
    let mut colors = Vec::new();
    for cs in &out.shapes {
        if let Shape::Text(t) = &cs.shape {
            if t.galley.text().is_empty() {
                continue;
            }
            if let Some(section) = t.galley.job.sections.first() {
                colors.push(section.format.color);
            }
        }
    }
    out.textures_delta.clear();
    colors
}

#[test]
fn node_name_label_uses_the_active_theme_text_color() {
    let mut map = Map::new();
    map.set_theme(std::rc::Rc::new(FixedPalette));
    map.settings = MapSettings {
        node_text_visibility: VisibilitySetting::Always,
        ..Default::default()
    };
    let mut point = MapPoint::new(1, [0.0, 0.0]);
    point.set_name("Amarr".to_string());
    map.add_points(vec![point]);
    // Above `label_visible_zoom` (0.58, the default) so `Always` draws the
    // name -- `Map::new()`'s default zoom of 1.0 already satisfies this.

    let colors = drawn_text_colors(&mut map);
    assert!(
        colors.contains(&THEME_TEXT),
        "the node name must be painted in ThemeColors::text, got {colors:?}"
    );
}

#[test]
fn free_label_uses_the_active_theme_text_color() {
    let mut map = Map::new();
    map.set_theme(std::rc::Rc::new(FixedPalette));
    map.settings = MapSettings::default();
    map.add_points(vec![MapPoint::new(1, [0.0, 0.0])]);
    map.add_labels(vec![MapLabel {
        text: "Domain".to_string(),
        center: egui::pos2(300.0, 200.0),
    }]);
    // `MapLabel`s only paint below `line_visible_zoom` (0.2).
    map.set_zoom(0.15);

    let colors = drawn_text_colors(&mut map);
    assert!(
        colors.contains(&THEME_TEXT),
        "the free label must be painted in ThemeColors::text, got {colors:?}"
    );
}

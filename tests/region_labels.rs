//! `RegionLabel`s are anchored to an area of the map rather than to a single
//! node, and -- unlike node names and `MapLabel`s (see
//! `tests/text_scaling.rs`, which asserts the *opposite* behavior) -- their
//! font size scales *with* the map's zoom instead of staying
//! screen-constant. They are also always painted first, so every other
//! layer draws over them, and with a more transparent color than the
//! theme's own text. Mirrors `tests/segment_template_context.rs` for the
//! `LabelTemplate` hook coverage.

use egui::{Color32, Context, FontFamily, FontId, Painter, RawInput, Shape};
use egui_map::map::Map;
use egui_map::map::objects::{
    LabelContext, LabelTemplate, MapPoint, MapSegment, MapSettings, RegionLabel,
};
use egui_map::map::theme::{ColorMode, MapTheme, ThemeColors};
use std::cell::RefCell;
use std::rc::Rc;

/// A fixed palette so color/alpha assertions don't depend on which of the
/// default theme's light/dark variants happens to be active.
struct FixedPalette;

impl MapTheme for FixedPalette {
    fn colors(&self, _mode: ColorMode) -> ThemeColors {
        ThemeColors {
            node: Color32::from_rgb(1, 2, 3),
            segment: Color32::from_rgb(4, 5, 6),
            selected: Color32::from_rgb(7, 8, 9),
            alert: Color32::from_rgb(10, 11, 12),
            marker: Color32::from_rgb(16, 17, 18),
            text: Color32::from_rgba_unmultiplied(200, 100, 50, 255),
            background: Color32::from_rgb(19, 20, 21),
        }
    }
}

/// Renders one frame and returns `(text, font_size, color)` for every
/// `Shape::Text` painted, in paint order. `color` is the text's effective
/// paint color -- `override_text_color` when set (as the built-in region
/// label renderer always sets it), otherwise `fallback_color`.
fn drawn_texts(map: &mut Map) -> Vec<(String, f32, Color32)> {
    let ctx = Context::default();
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(600.0, 400.0));
    let mut output = ctx.run_ui(
        RawInput {
            screen_rect: Some(screen),
            ..RawInput::default()
        },
        |ui| {
            ui.add(&mut *map);
        },
    );
    let mut found = Vec::new();
    for cs in &output.shapes {
        if let Shape::Text(t) = &cs.shape {
            let text = t.galley.text().to_string();
            if text.is_empty() {
                continue;
            }
            let size = t
                .galley
                .job
                .sections
                .first()
                .map(|s| s.format.font_id.size)
                .unwrap_or(f32::NAN);
            let color = t.override_text_color.unwrap_or(t.fallback_color);
            found.push((text, size, color));
        }
    }
    output.textures_delta.clear();
    found
}

/// Like `drawn_texts`, but also returns each painted `Shape::Text`'s font
/// family, to check `Style::region_label_font` reaches the built-in
/// renderer.
fn drawn_text_families(map: &mut Map) -> Vec<(String, FontFamily)> {
    let ctx = Context::default();
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(600.0, 400.0));
    let mut output = ctx.run_ui(
        RawInput {
            screen_rect: Some(screen),
            ..RawInput::default()
        },
        |ui| {
            ui.add(&mut *map);
        },
    );
    let mut found = Vec::new();
    for cs in &output.shapes {
        if let Shape::Text(t) = &cs.shape {
            let text = t.galley.text().to_string();
            if text.is_empty() {
                continue;
            }
            let family = t
                .galley
                .job
                .sections
                .first()
                .map(|s| s.format.font_id.family.clone())
                .unwrap_or(FontFamily::Proportional);
            found.push((text, family));
        }
    }
    output.textures_delta.clear();
    found
}

fn map_with_region_label(text: &str, zoom: f32) -> Map {
    let mut map = Map::new();
    map.settings = MapSettings::default();
    map.set_theme(Rc::new(FixedPalette));
    map.add_region_labels(vec![RegionLabel {
        text: text.to_string(),
        center: egui::pos2(300.0, 200.0),
        color: None,
    }]);
    map.set_zoom(zoom);
    map
}

fn find<'a>(texts: &'a [(String, f32, Color32)], needle: &str) -> &'a (String, f32, Color32) {
    texts
        .iter()
        .find(|(t, ..)| t == needle)
        .unwrap_or_else(|| panic!("{needle:?} was not drawn; got {texts:?}"))
}

#[test]
fn region_label_font_size_scales_with_zoom() {
    // Opposite of `node_name_size_does_not_change_with_zoom`/
    // `free_label_size_does_not_change_with_zoom` in `tests/text_scaling.rs`.
    let base = MapSettings::default().styles[0].region_label_font.size;
    for zoom in [0.2_f32, 0.5, 1.0, 1.8] {
        let mut map = map_with_region_label("Domain", zoom);
        let texts = drawn_texts(&mut map);
        let (_, size, _) = find(&texts, "Domain");
        let expected = base * zoom;
        assert!(
            (*size - expected).abs() < 0.01,
            "at zoom {zoom}: expected size {expected}, got {size}"
        );
    }
}

#[test]
fn region_label_font_size_is_configurable() {
    let mut map = map_with_region_label("Domain", 2.0);
    for style in &mut map.settings.styles {
        style.region_label_font.size = 10.0;
    }
    let texts = drawn_texts(&mut map);
    let (_, size, _) = find(&texts, "Domain");
    assert_eq!(*size, 20.0);
}

#[test]
fn region_label_color_is_theme_text_faded_by_alpha() {
    let mut map = map_with_region_label("Domain", 1.0);
    map.settings.region_label_alpha = 0.4;
    let texts = drawn_texts(&mut map);
    let (_, _, color) = find(&texts, "Domain");
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    // Within 1 of the theme's RGB and the expected alpha: `Color32` stores
    // premultiplied bytes internally, so a fade-and-recover round trip can
    // be off by a rounding unit -- see `scale_alpha`'s own doc in
    // `src/map.rs` and the exact-byte unit test next to it, which asserts
    // this precisely with access to `scale_alpha` itself.
    for (actual, expected) in [(r, 200), (g, 100), (b, 50)] {
        assert!(
            actual.abs_diff(expected) <= 1,
            "region label must keep (approximately) the theme text's RGB, only fading alpha; \
             got {color:?}"
        );
    }
    let expected_alpha = (255.0_f32 * 0.4).round() as u8;
    assert!(
        a.abs_diff(expected_alpha) <= 1,
        "expected alpha ~{expected_alpha}, got {a}"
    );
}

#[test]
fn region_label_alpha_defaults_to_more_transparent_than_full_theme_text() {
    let mut map = map_with_region_label("Domain", 1.0);
    let texts = drawn_texts(&mut map);
    let (_, _, color) = find(&texts, "Domain");
    assert!(
        color.a() < 255,
        "default region_label_alpha should fade the theme's text color, got alpha {}",
        color.a()
    );
}

#[test]
fn region_label_font_family_defaults_to_proportional() {
    let mut map = map_with_region_label("Domain", 1.0);
    let families = drawn_text_families(&mut map);
    let (_, family) = families
        .iter()
        .find(|(t, _)| t == "Domain")
        .expect("region label was not painted");
    assert_eq!(*family, FontFamily::Proportional);
}

#[test]
fn region_label_font_family_is_configurable() {
    let mut map = map_with_region_label("Domain", 1.0);
    for style in &mut map.settings.styles {
        style.region_label_font = FontId::new(30.0, FontFamily::Monospace);
    }
    let families = drawn_text_families(&mut map);
    let (_, family) = families
        .iter()
        .find(|(t, _)| t == "Domain")
        .expect("region label was not painted");
    assert_eq!(*family, FontFamily::Monospace);
}

#[test]
fn region_labels_paint_before_lines_and_nodes() {
    let mut map = Map::new();
    map.settings = MapSettings::default();
    map.add_points(vec![MapPoint::new(1, [0.0, 0.0])]);
    map.add_lines(vec![MapSegment::new((1, 2), [0.0, 0.0], [50.0, 0.0])]);
    map.add_region_labels(vec![RegionLabel {
        text: "Domain".to_string(),
        center: egui::pos2(300.0, 200.0),
        color: None,
    }]);
    map.set_zoom(1.0);

    let ctx = Context::default();
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(600.0, 400.0));
    let mut output = ctx.run_ui(
        RawInput {
            screen_rect: Some(screen),
            ..RawInput::default()
        },
        |ui| {
            ui.add(&mut map);
        },
    );

    let label_index = output
        .shapes
        .iter()
        .position(|cs| matches!(&cs.shape, Shape::Text(t) if t.galley.text() == "Domain"))
        .expect("region label was not painted");
    let line_index = output
        .shapes
        .iter()
        .position(|cs| matches!(&cs.shape, Shape::LineSegment { .. }))
        .expect("line was not painted");
    let node_index = output
        .shapes
        .iter()
        .position(|cs| matches!(&cs.shape, Shape::Circle(c) if c.fill.a() > 0))
        .expect("node was not painted");
    output.textures_delta.clear();

    assert!(
        label_index < line_index,
        "region label (index {label_index}) must paint before the line (index {line_index})"
    );
    assert!(
        label_index < node_index,
        "region label (index {label_index}) must paint before the node (index {node_index})"
    );
}

/// One recorded call to [`LabelTemplate::label_ui`], mirroring [`LabelContext`]'s
/// fields (minus the borrowed `label`, copied out as an owned `text`).
struct RecordedLabelCall {
    text: String,
    #[allow(dead_code)]
    position: egui::Pos2,
    zoom: f32,
    size: f32,
    color: Color32,
    theme: ThemeColors,
}

/// Records every call it receives, so tests can assert on exactly what
/// reached the hook rather than on what got painted.
#[derive(Default)]
struct RecordingLabelTemplate {
    calls: RefCell<Vec<RecordedLabelCall>>,
}

impl LabelTemplate for RecordingLabelTemplate {
    fn label_ui(&self, _painter: &Painter, ctx: LabelContext) {
        self.calls.borrow_mut().push(RecordedLabelCall {
            text: ctx.label.text.clone(),
            position: ctx.position,
            zoom: ctx.zoom,
            size: ctx.size,
            color: ctx.color,
            theme: ctx.theme,
        });
    }
}

#[test]
fn label_template_replaces_the_built_in_renderer_and_receives_the_documented_context() {
    let template = Rc::new(RecordingLabelTemplate::default());
    let mut map = Map::new();
    map.settings = MapSettings::default();
    map.settings.region_label_alpha = 0.5;
    map.set_theme(Rc::new(FixedPalette));
    map.set_label_template(template.clone());
    map.add_region_labels(vec![RegionLabel {
        text: "Domain".to_string(),
        center: egui::pos2(0.0, 0.0),
        color: None,
    }]);
    map.set_zoom(1.5);

    // No built-in text should be painted once a template is installed.
    let texts = drawn_texts(&mut map);
    assert!(
        texts.iter().all(|(t, ..)| t != "Domain"),
        "built-in renderer must not run once a LabelTemplate is installed; got {texts:?}"
    );

    let calls = template.calls.borrow();
    assert_eq!(calls.len(), 1, "label_ui should run once per region label");
    let call = &calls[0];
    assert_eq!(call.text, "Domain");
    assert_eq!(call.zoom, 1.5);
    assert_eq!(
        call.size,
        MapSettings::default().styles[0].region_label_font.size * 1.5,
        "ctx.size must already be scaled by zoom"
    );
    let [r, g, b, a] = call.color.to_srgba_unmultiplied();
    // See the identical tolerance note in
    // `region_label_color_is_theme_text_faded_by_alpha` above.
    for (actual, expected) in [(r, 200), (g, 100), (b, 50)] {
        assert!(actual.abs_diff(expected) <= 1, "got {:?}", call.color);
    }
    let expected_alpha = (255.0_f32 * 0.5).round() as u8;
    assert!(a.abs_diff(expected_alpha) <= 1, "got {:?}", call.color);
    assert_eq!(call.theme.text, FixedPalette.colors(ColorMode::Light).text);
}

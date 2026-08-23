//! Node names and free-floating labels are sized in **screen pixels**, so they
//! stay readable no matter how far the map is zoomed out.

use egui::{Context, RawInput, Shape};
use egui_map::map::Map;
use egui_map::map::objects::{MapLabel, MapPoint, MapSettings, VisibilitySetting};

/// Renders one frame and returns `(text, font_size)` for every string drawn.
fn drawn_text(map: &mut Map) -> Vec<(String, f32)> {
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
    let mut found = Vec::new();
    for cs in &out.shapes {
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
            found.push((text, size));
        }
    }
    out.textures_delta.clear();
    found
}

fn map_with_named_node() -> Map {
    let mut map = Map::new();
    map.settings = MapSettings {
        node_text_visibility: VisibilitySetting::Always,
        ..Default::default()
    };
    let mut point = MapPoint::new(1, [0.0, 0.0]);
    point.set_name("Amarr".to_string());
    map.add_points(vec![point]);
    map
}

fn size_of(texts: &[(String, f32)], needle: &str) -> f32 {
    texts
        .iter()
        .find(|(t, _)| t == needle)
        .unwrap_or_else(|| panic!("{needle:?} was not drawn; got {texts:?}"))
        .1
}

#[test]
fn node_name_size_does_not_change_with_zoom() {
    // Above `label_visible_zoom` (0.58) so `Always` actually draws the name.
    let mut sizes = Vec::new();
    for zoom in [0.6_f32, 1.0, 1.5, 2.0] {
        let mut map = map_with_named_node();
        map.set_zoom(zoom);
        sizes.push((zoom, size_of(&drawn_text(&mut map), "Amarr")));
    }

    let expected = MapSettings::default().node_text_size;
    for (zoom, size) in &sizes {
        assert_eq!(
            *size, expected,
            "node name must keep its screen size; at zoom {zoom} it was {size} (all: {sizes:?})"
        );
    }
}

#[test]
fn node_name_size_is_configurable() {
    let mut map = map_with_named_node();
    map.settings.node_text_size = 21.0;
    map.set_zoom(0.6);

    assert_eq!(size_of(&drawn_text(&mut map), "Amarr"), 21.0);
}

#[test]
fn free_label_size_does_not_change_with_zoom() {
    // `MapLabel`s are drawn below `line_visible_zoom` (0.2).
    let mut sizes = Vec::new();
    for zoom in [0.1_f32, 0.15, 0.19] {
        let mut map = Map::new();
        map.settings = MapSettings::default();
        map.add_points(vec![MapPoint::new(1, [0.0, 0.0])]);
        map.add_labels(vec![MapLabel {
            text: "Domain".to_string(),
            center: egui::pos2(300.0, 200.0),
        }]);
        map.set_zoom(zoom);
        sizes.push((zoom, size_of(&drawn_text(&mut map), "Domain")));
    }

    let expected = MapSettings::default().label_text_size;
    for (zoom, size) in &sizes {
        assert_eq!(
            *size, expected,
            "free label must keep its screen size; at zoom {zoom} it was {size} (all: {sizes:?})"
        );
    }
}

#[test]
fn free_label_size_is_configurable() {
    let mut map = Map::new();
    map.settings = MapSettings::default();
    map.settings.label_text_size = 33.0;
    map.add_points(vec![MapPoint::new(1, [0.0, 0.0])]);
    map.add_labels(vec![MapLabel {
        text: "Domain".to_string(),
        center: egui::pos2(300.0, 200.0),
    }]);
    map.set_zoom(0.15);

    assert_eq!(size_of(&drawn_text(&mut map), "Domain"), 33.0);
}

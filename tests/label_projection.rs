//! `MapLabel::center` is in map coordinates: labels must follow the view
//! through pan and zoom, exactly like nodes, instead of being pinned to a
//! fixed screen pixel (the historical bug this guards).

use egui::{Context, Pos2, RawInput, Rect, Shape, Vec2};
use egui_map::map::Map;
use egui_map::map::objects::{MapLabel, MapPoint};

/// Renders a couple of frames (so a `set_zoom`/resize has settled) and returns
/// the screen position of every drawn text.
fn drawn_text_positions(map: &mut Map) -> Vec<(String, Pos2)> {
    let ctx = Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 400.0));

    let mut result = Vec::new();
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
        result.clear();
        for cs in &out.shapes {
            if let Shape::Text(t) = &cs.shape {
                let text = t.galley.text().to_string();
                if !text.is_empty() {
                    result.push((text, t.pos));
                }
            }
        }
        out.textures_delta.clear();
    }
    result
}

fn pos_of(texts: &[(String, Pos2)], needle: &str) -> Pos2 {
    texts
        .iter()
        .find(|(t, _)| t == needle)
        .unwrap_or_else(|| panic!("{needle:?} was not drawn; got {texts:?}"))
        .1
}

/// One node at the origin (so the view's reference point is `[0, 0]`) plus two
/// labels 100 map units apart, drawn at a zoom below `line_visible_zoom`.
fn label_map() -> Map {
    let mut map = Map::new();
    map.add_points(vec![MapPoint::new(1, [0.0, 0.0])]);
    map.add_labels(vec![
        MapLabel {
            text: "L1".to_string(),
            center: egui::pos2(0.0, 0.0),
        },
        MapLabel {
            text: "L2".to_string(),
            center: egui::pos2(100.0, 0.0),
        },
    ]);
    map.set_zoom(0.15);
    map
}

#[test]
fn label_spacing_scales_with_zoom() {
    let mut map = label_map();
    let texts = drawn_text_positions(&mut map);
    let l1 = pos_of(&texts, "L1");
    let l2 = pos_of(&texts, "L2");

    // 100 map units apart at zoom 0.15 => 15 px on screen, wherever the view is.
    assert!(
        (l2.x - l1.x - 15.0).abs() < 0.5,
        "labels must be one zoom-scaled distance apart, got l1={l1:?} l2={l2:?}"
    );
    assert!(
        (l2.y - l1.y).abs() < 0.5,
        "labels must share the same row, got l1={l1:?} l2={l2:?}"
    );
}

#[test]
fn label_follows_pan() {
    let mut map = label_map();
    let before = pos_of(&drawn_text_positions(&mut map), "L1");

    // Pan the view 50 map units to the right; at zoom 0.15 the label must
    // slide 50 * 0.15 = 7.5 px to the left.
    map.set_pos([50.0, 0.0]);
    let after = pos_of(&drawn_text_positions(&mut map), "L1");

    assert!(
        (after.x - before.x + 7.5).abs() < 0.5,
        "label must move with pan, got before={before:?} after={after:?}"
    );
    assert!(
        (after.y - before.y).abs() < 0.5,
        "pan on X must not move the label vertically, got before={before:?} after={after:?}"
    );
}

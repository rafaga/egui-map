//! Behaviour of the `debug_overlay` feature's read-out.
//!
//! The overlay is meant to be unobtrusive: a collapsed `dbg` toggle with no
//! background of its own, that a developer opens when they need the numbers.
#![cfg(feature = "debug_overlay")]

use egui::{Context, Event, PointerButton, RawInput, Shape};
use egui_map::map::Map;
use egui_map::map::objects::MapPoint;

/// Renders the map for enough frames to settle, optionally clicking once at
/// `click`. Returns the drawn text strings and the opaque rects.
fn render(click: Option<egui::Pos2>) -> (Vec<String>, Vec<egui::Rect>) {
    let ctx = Context::default();
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(500.0, 400.0));

    let mut map = Map::new();
    map.add_points(vec![
        MapPoint::new(1, [0.0, 0.0]),
        MapPoint::new(2, [50.0, 50.0]),
    ]);

    let mut texts = Vec::new();
    let mut rects = Vec::new();
    // Several passes: egui needs a frame to lay out before input lands, and
    // the collapsing body animates open.
    for pass in 0..40 {
        let mut events = Vec::new();
        if pass == 1 && let Some(pos) = click {
            events.push(Event::PointerMoved(pos));
            for pressed in [true, false] {
                events.push(Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                });
            }
        }
        let mut out = ctx.run_ui(
            RawInput {
                screen_rect: Some(screen),
                events,
                ..RawInput::default()
            },
            |ui| {
                ui.add(&mut map);
            },
        );
        texts.clear();
        rects.clear();
        for cs in &out.shapes {
            match &cs.shape {
                Shape::Text(t) => texts.push(t.galley.text().to_string()),
                Shape::Rect(r) if r.fill.a() > 0 => rects.push(r.rect),
                _ => {}
            }
        }
        out.textures_delta.clear();
    }
    (texts, rects)
}

/// The largest opaque rect is the canvas frame's background.
fn canvas_frame(rects: &[egui::Rect]) -> egui::Rect {
    *rects
        .iter()
        .max_by(|a, b| a.area().partial_cmp(&b.area()).unwrap())
        .expect("the canvas frame must be painted")
}

#[test]
fn overlay_starts_collapsed_and_shows_only_its_toggle() {
    let (texts, _) = render(None);

    assert!(
        texts.iter().any(|t| t.contains("dbg")),
        "the `dbg` toggle must be visible; got {texts:?}"
    );
    for key in ["MIN", "MAX", "CUR", "DST", "ZOM", "REC", "NUM", "VIS"] {
        assert!(
            !texts.iter().any(|t| t.contains(key)),
            "collapsed overlay must not draw {key}; got {texts:?}"
        );
    }
}

#[test]
fn overlay_expands_when_its_toggle_is_clicked() {
    let (texts, _) = render(Some(egui::pos2(30.0, 15.0)));

    for key in ["MIN", "MAX", "CUR", "DST", "ZOM", "REC"] {
        assert!(
            texts.iter().any(|t| t.contains(key)),
            "expanded overlay must draw {key}; got {texts:?}"
        );
    }
}

/// Regression guard: the overlay lives in a detached child `Ui`, so it must
/// never feed the canvas `Frame`'s `min_rect`. If it did, the frame would grow
/// past the space the widget was given -- the same overflow that used to knock
/// a centered node off the middle of the map.
#[test]
fn overlay_never_changes_the_canvas_frame() {
    let (_, collapsed) = render(None);
    let (_, expanded) = render(Some(egui::pos2(30.0, 15.0)));

    let collapsed = canvas_frame(&collapsed);
    let expanded = canvas_frame(&expanded);

    assert_eq!(
        collapsed, expanded,
        "opening the overlay must not resize the canvas frame"
    );
    assert!(
        expanded.max.x <= 500.0 && expanded.max.y <= 400.0,
        "the canvas frame must stay inside the viewport, got {expanded:?}"
    );
}

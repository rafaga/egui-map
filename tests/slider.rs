//! Verifies where the zoom slider (and the read-out under it) is drawn: pinned
//! to its corner, stacked clear of the minimap, and hideable.

use egui::{Color32, Context, Pos2, RawInput, Rect, Shape, SliderOrientation, vec2};
use egui_map::map::Map;
use egui_map::map::objects::MapPoint;
use egui_map::map::overlay::Corner;
use egui_map::map::theme::{ColorMode, MapTheme, ThemeColors};
use std::rc::Rc;

const TEXT: Color32 = Color32::from_rgb(240, 230, 120);
const BACKGROUND: Color32 = Color32::from_rgb(20, 20, 40);

/// A palette whose text and background are easy to tell from every other.
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
            background: BACKGROUND,
        }
    }
}

/// One drawn frame: where the zoom read-out and the minimap panel ended up.
struct Scene {
    /// The area the map paints into.
    area: Rect,
    label: Option<Pos2>,
    minimap: Option<Rect>,
}

impl Scene {
    fn with(configure: impl FnOnce(&mut Map)) -> Self {
        let mut map = Map::new();
        map.add_points(vec![
            MapPoint::new(1, [0.0, 0.0]),
            MapPoint::new(2, [2000.0, 1000.0]),
        ]);
        map.set_theme(Rc::new(Palette));
        configure(&mut map);
        let opacity = map.settings.minimap.opacity;

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

        let mut area = Rect::NOTHING;
        let (mut label, mut minimap) = (None, None);
        for clipped in &out.shapes {
            match &clipped.shape {
                Shape::Text(text) if text.galley.text() == "100%" => label = Some(text.pos),
                Shape::Rect(rect) if rect.fill == BACKGROUND.gamma_multiply(opacity) => {
                    minimap = Some(rect.rect);
                    area = clipped.clip_rect;
                }
                _ => {}
            }
        }
        if area == Rect::NOTHING {
            // No minimap to take the painter's rectangle from: it is the clip
            // rectangle of the read-out, which paints inside the same area.
            area = out
                .shapes
                .iter()
                .find(|clipped| matches!(&clipped.shape, Shape::Text(_)))
                .map_or(Rect::NOTHING, |clipped| clipped.clip_rect);
        }
        Self {
            area,
            label,
            minimap,
        }
    }
}

#[test]
fn by_default_the_slider_is_in_the_top_right_corner() {
    let scene = Scene::with(|_| {});
    let label = scene.label.expect("the zoom label");
    assert!(label.x > scene.area.right() - 60.0 && label.x < scene.area.right() - 10.0);
    assert!(label.y < scene.area.top() + 140.0, "{label:?}");
}

#[test]
fn the_slider_can_be_pinned_to_any_corner() {
    for anchor in [
        Corner::TopLeft,
        Corner::TopRight,
        Corner::BottomLeft,
        Corner::BottomRight,
    ] {
        let scene = Scene::with(|map| map.settings.slider.anchor = anchor);
        let label = scene.label.expect("the zoom label");
        let (center_x, center_y) = (scene.area.center().x, scene.area.center().y);
        assert_eq!(
            label.x > center_x,
            matches!(anchor, Corner::TopRight | Corner::BottomRight)
        );
        assert_eq!(label.y < center_y, anchor.is_top(), "{anchor:?}: {label:?}");
    }
}

#[test]
fn the_slider_can_be_hidden() {
    let scene = Scene::with(|map| map.settings.slider.visible = false);
    assert!(scene.label.is_none());
}

#[test]
fn a_minimap_in_another_corner_leaves_the_slider_where_it_is() {
    let alone = Scene::with(|_| {});
    let with_minimap = Scene::with(|map| map.settings.minimap.enabled = true);
    assert!(with_minimap.minimap.is_some());
    assert_eq!(alone.label, with_minimap.label);
}

#[test]
fn sharing_a_corner_the_slider_stacks_above_the_minimap() {
    let scene = Scene::with(|map| {
        map.settings.minimap.enabled = true;
        map.settings.slider.anchor = Corner::BottomRight;
    });
    let (label, minimap) = (scene.label.expect("label"), scene.minimap.expect("minimap"));
    assert!(
        label.y < minimap.top(),
        "{label:?} is not above {minimap:?}"
    );
    // On the same edge, so the two line up.
    assert!(label.x > minimap.right() - ZOOM_CONTROL_WIDTH && label.x < minimap.right());
}

/// The width of the control, as documented on `SliderSettings::BREADTH`.
const ZOOM_CONTROL_WIDTH: f32 = egui_map::map::slider::SliderSettings::BREADTH;

const TEXT_RGB: [u8; 3] = [240, 230, 120];

/// The tick lines and the handle drawn in one frame.
struct Marks {
    ticks: Vec<[Pos2; 2]>,
    /// Centres of the small round or square shapes: the handle is one.
    handles: Vec<Pos2>,
}

impl Marks {
    /// Whether `color` is the theme's text colour at some opacity (fading
    /// premultiplies it, so the channels only match within rounding).
    fn is_text_colored(color: Color32) -> bool {
        let [r, g, b, a] = color.to_srgba_unmultiplied();
        a > 20
            && [r, g, b]
                .iter()
                .zip(TEXT_RGB)
                .all(|(got, want)| got.abs_diff(want) <= 6)
    }

    fn with(configure: impl FnOnce(&mut Map)) -> Self {
        Self::frame(None, configure)
    }

    /// A frame where the mouse is at `pointer`, if given.
    fn frame(pointer: Option<Pos2>, configure: impl FnOnce(&mut Map)) -> Self {
        let mut map = Map::new();
        map.set_theme(Rc::new(Palette));
        configure(&mut map);
        let events = pointer
            .map(|pos| vec![egui::Event::PointerMoved(pos)])
            .unwrap_or_default();
        let mut out = Context::default().run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
                events,
                ..RawInput::default()
            },
            |ui| {
                ui.add(&mut map);
            },
        );
        out.textures_delta.clear();
        let (mut ticks, mut handles) = (Vec::new(), Vec::new());
        for clipped in &out.shapes {
            match &clipped.shape {
                Shape::LineSegment { points, stroke } if Self::is_text_colored(stroke.color) => {
                    ticks.push(*points);
                }
                Shape::Rect(rect) if rect.rect.width() < 20.0 && rect.rect.height() < 20.0 => {
                    handles.push(rect.rect.center());
                }
                Shape::Circle(circle) if circle.radius < 10.0 => handles.push(circle.center),
                _ => {}
            }
        }
        Self { ticks, handles }
    }
}

#[test]
fn the_slider_has_five_marks_without_labels() {
    let marks = Marks::with(|_| {});
    assert_eq!(marks.ticks.len(), 5);
    // The only text is the zoom read-out.
    let scene = Scene::with(|_| {});
    assert!(scene.label.is_some());
}

#[test]
fn the_marks_can_be_turned_off() {
    let marks = Marks::with(|map| map.settings.slider.ticks = false);
    assert!(marks.ticks.is_empty());
}

#[test]
fn the_vertical_marks_are_horizontal_lines_left_of_a_right_corner_slider() {
    let marks = Marks::with(|_| {});
    for [a, b] in &marks.ticks {
        assert_eq!(a.y, b.y);
    }
    let handle = marks.handles.first().expect("the handle");
    assert!(marks.ticks.iter().all(|[a, _]| a.x < handle.x));
}

#[test]
fn the_horizontal_marks_are_vertical_lines() {
    let marks = Marks::with(|map| {
        map.settings.slider.orientation = SliderOrientation::Horizontal;
        map.settings.slider.anchor = Corner::BottomLeft;
    });
    assert_eq!(marks.ticks.len(), 5);
    for [a, b] in &marks.ticks {
        assert_eq!(a.x, b.x);
    }
    let handle = marks.handles.first().expect("the handle");
    // In a bottom corner the marks are above the slider.
    assert!(marks.ticks.iter().all(|[a, _]| a.y < handle.y));
}

#[test]
fn the_handle_lines_up_with_the_first_and_last_marks() {
    for (zoom, index) in [(0.1, 0), (2.0, 4)] {
        let marks = Marks::with(|map| map.set_zoom(zoom));
        let handle = marks.handles.first().expect("the handle");
        let mark = marks.ticks[index][0];
        assert!(
            (handle.y - mark.y).abs() <= 1.0,
            "zoom {zoom}: handle {handle:?}, mark {mark:?}"
        );
    }
}

#[test]
fn the_horizontal_handle_lines_up_with_the_marks() {
    for (zoom, index) in [(0.1, 0), (2.0, 4)] {
        let marks = Marks::with(|map| {
            map.settings.slider.orientation = SliderOrientation::Horizontal;
            map.set_zoom(zoom);
        });
        let handle = marks.handles.first().expect("the handle");
        let mark = marks.ticks[index][0];
        assert!(
            (handle.x - mark.x).abs() <= 1.0,
            "zoom {zoom}: handle {handle:?}, mark {mark:?}"
        );
    }
}

/// The rounded rectangles of one frame whose fill is the theme's background
/// at some opacity: the slider's panel, when it is drawn.
fn panels(pointer: Option<Pos2>, configure: impl FnOnce(&mut Map)) -> Vec<(Rect, u8)> {
    let mut map = Map::new();
    map.set_theme(Rc::new(Palette));
    configure(&mut map);
    let events = pointer
        .map(|pos| vec![egui::Event::PointerMoved(pos)])
        .unwrap_or_default();
    let mut out = Context::default().run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
            events,
            ..RawInput::default()
        },
        |ui| {
            ui.add(&mut map);
        },
    );
    out.textures_delta.clear();
    out.shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            Shape::Rect(rect)
                if rect.corner_radius != egui::CornerRadius::ZERO
                    && rect.fill.a() > 0
                    && rect.fill.to_srgba_unmultiplied()[..3]
                        .iter()
                        .zip([20u8, 20, 40])
                        .all(|(got, want)| got.abs_diff(want) <= 3) =>
            {
                Some((rect.rect, rect.fill.a()))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn the_slider_panel_is_invisible_until_the_mouse_is_over_it() {
    assert!(panels(None, |_| {}).is_empty());
    assert!(panels(Some(Pos2::new(100.0, 500.0)), |_| {}).is_empty());
}

#[test]
fn under_the_mouse_the_panel_is_semi_opaque_and_holds_the_whole_control() {
    let marks = Marks::with(|_| {});
    let probe = marks.ticks[2][0];
    let shown = panels(Some(probe), |_| {});
    assert_eq!(shown.len(), 1, "{shown:?}");
    let (panel, alpha) = shown[0];
    assert!(alpha > 100 && alpha < 200, "alpha {alpha}");
    // It holds the marks and the handle.
    for [a, b] in &marks.ticks {
        assert!(panel.contains(*a) && panel.contains(*b), "{panel:?}");
    }
    assert!(panel.contains(*marks.handles.first().expect("the handle")));
}

#[test]
fn the_panel_opacities_are_configurable() {
    let probe = Marks::with(|_| {}).ticks[2][0];
    let solid = panels(Some(probe), |map| {
        map.settings.slider.panel_hover_opacity = 1.0
    });
    assert_eq!(solid[0].1, 255);
    let idle = panels(None, |map| map.settings.slider.panel_idle_opacity = 1.0);
    assert_eq!(idle.len(), 1);
}

#[test]
fn the_panel_lines_up_with_the_minimap_on_the_same_edge() {
    let minimap = Scene::with(|map| map.settings.minimap.enabled = true)
        .minimap
        .expect("minimap");
    let probe = Marks::with(|_| {}).ticks[2][0];
    let (panel, _) = panels(Some(probe), |_| {})[0];
    assert_eq!(panel.right(), minimap.right());
}

//! Verifies the minimap overlay: it is opt-in, pinned where it is told to be,
//! painted with the active theme, and recentres the main view when pressed.

use egui::epaint::ClippedShape;
use egui::{
    Color32, Context, Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Shape, Vec2, vec2,
};
use egui_map::map::Map;
use egui_map::map::minimap::MinimapSettings;
use egui_map::map::objects::{MapPoint, MapSegment};
use egui_map::map::overlay::Corner;
use egui_map::map::theme::{ColorMode, MapTheme, ThemeColors};
use std::rc::Rc;

/// Map extent used throughout: `0..=2000` by `0..=1000`.
const WORLD: [f32; 2] = [2000.0, 1000.0];

/// A palette whose roles are all distinguishable from each other.
struct Palette;

impl MapTheme for Palette {
    fn colors(&self, _mode: ColorMode) -> ThemeColors {
        ThemeColors {
            node: Color32::from_rgb(10, 200, 30),
            segment: Color32::from_rgb(200, 60, 60),
            selected: Color32::from_rgb(30, 60, 220),
            alert: Color32::from_rgb(250, 150, 0),
            marker: Color32::from_rgb(250, 0, 250),
            text: Color32::WHITE,
            background: Color32::from_rgb(20, 20, 40),
        }
    }
}

/// A map, the egui context that runs it, and the helpers to drive frames.
struct Scene {
    ctx: Context,
    map: Map,
}

impl Scene {
    fn new(enabled: bool) -> Self {
        let mut map = Map::new();
        map.add_points(vec![
            MapPoint::new(1, [0.0, 0.0]),
            MapPoint::new(2, WORLD),
            MapPoint::new(3, [1000.0, 500.0]),
        ]);
        map.add_lines(vec![MapSegment::new((1, 2), [0.0, 0.0], WORLD)]);
        map.set_theme(Rc::new(Palette));
        map.settings.minimap.enabled = enabled;
        Self {
            ctx: Context::default(),
            map,
        }
    }

    fn frame(&mut self, events: Vec<Event>) -> Vec<ClippedShape> {
        let map = &mut self.map;
        let mut out = self.ctx.run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
                events,
                ..RawInput::default()
            },
            |ui| {
                ui.add(&mut *map);
            },
        );
        out.textures_delta.clear();
        out.shapes
    }

    /// The minimap's panel (the rectangle filled with the theme background at
    /// the minimap's opacity) and the area the map paints into, which is that
    /// shape's clip rectangle.
    fn panel(shapes: &[ClippedShape], settings: &MinimapSettings) -> Option<(Rect, Rect)> {
        let fill = Palette
            .colors(ColorMode::Dark)
            .background
            .gamma_multiply(settings.opacity);
        shapes.iter().find_map(|clipped| match &clipped.shape {
            Shape::Rect(rect) if rect.fill == fill => Some((rect.rect, clipped.clip_rect)),
            _ => None,
        })
    }

    /// Runs a frame and returns the minimap's panel and the map's paint area.
    fn shown_panel(&mut self) -> (Rect, Rect) {
        let shapes = self.frame(vec![]);
        Self::panel(&shapes, &self.map.settings.minimap).expect("minimap panel")
    }

    fn press_events(pos: Pos2) -> Vec<Event> {
        vec![
            Event::PointerMoved(pos),
            Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ]
    }

    /// Frames to let egui register the widgets, then press at `pos`.
    fn press_at(&mut self, pos: Pos2) {
        self.frame(vec![]);
        self.frame(vec![Event::PointerMoved(pos)]);
        self.frame(Self::press_events(pos));
        self.frame(vec![]);
    }

    /// Screen pixels per map unit of the minimap's fit of [`WORLD`] into `rect`
    /// (minus its padding), the same as the overlay computes.
    fn scale(rect: Rect) -> f32 {
        ((rect.width() - 12.0) / WORLD[0]).min((rect.height() - 12.0) / WORLD[1])
    }
}

#[test]
fn a_map_does_not_draw_a_minimap_unless_asked() {
    let mut off = Scene::new(false);
    let shapes = off.frame(vec![]);
    assert!(Scene::panel(&shapes, &off.map.settings.minimap).is_none());

    let mut on = Scene::new(true);
    let shapes = on.frame(vec![]);
    assert!(Scene::panel(&shapes, &on.map.settings.minimap).is_some());
}

#[test]
fn the_minimap_is_pinned_to_the_requested_corner() {
    for anchor in [
        Corner::TopLeft,
        Corner::TopRight,
        Corner::BottomLeft,
        Corner::BottomRight,
    ] {
        let mut scene = Scene::new(true);
        scene.map.settings.minimap.anchor = anchor;
        let (rect, area) = scene.shown_panel();
        let settings = scene.map.settings.minimap;
        assert_eq!(
            rect,
            settings.fitted_rect(area, vec2(WORLD[0], WORLD[1])),
            "{anchor:?}"
        );
        assert!(settings.rect_in(area).contains_rect(rect), "{anchor:?}");
        let (full, corner) = (settings.rect_in(area), settings.margin);
        match anchor {
            Corner::TopLeft => assert_eq!(rect.left_top(), full.left_top()),
            Corner::TopRight => assert_eq!(rect.right_top(), full.right_top()),
            Corner::BottomLeft => assert_eq!(rect.left_bottom(), full.left_bottom()),
            Corner::BottomRight => {
                assert_eq!(
                    rect.right_bottom(),
                    area.right_bottom() - vec2(corner, corner)
                );
            }
        }
    }
}

#[test]
fn the_minimap_is_painted_with_the_active_theme() {
    let mut scene = Scene::new(true);
    scene.map.settings.minimap.opacity = 0.5;
    let shapes = scene.frame(vec![]);
    let theme = Palette.colors(ColorMode::Dark);
    let tint = |color: Color32| color.gamma_multiply(0.5);

    let (rect, area) = Scene::panel(&shapes, &scene.map.settings.minimap).expect("minimap panel");
    let inside = |clipped: &ClippedShape| clipped.clip_rect == rect.intersect(area);

    let panel_fill = shapes.iter().find_map(|clipped| match &clipped.shape {
        Shape::Rect(r) if r.rect == rect && r.fill != Color32::TRANSPARENT => Some(r.fill),
        _ => None,
    });
    assert_eq!(panel_fill, Some(tint(theme.background)));

    let nodes: Vec<_> = shapes
        .iter()
        .filter(|clipped| inside(clipped))
        .filter_map(|clipped| match &clipped.shape {
            Shape::Rect(node) if node.rect.size() == Vec2::splat(1.0) => Some(node.fill),
            _ => None,
        })
        .collect();
    assert_eq!(nodes, vec![tint(theme.node); 3]);

    let segments: Vec<_> = shapes
        .iter()
        .filter(|clipped| inside(clipped))
        .filter_map(|clipped| match &clipped.shape {
            Shape::LineSegment { stroke, .. } => Some(stroke.color),
            _ => None,
        })
        .collect();
    assert_eq!(segments, vec![tint(theme.segment)]);
}

#[test]
fn the_viewport_outline_can_be_turned_off() {
    // Zoomed in, so the viewport is a part of the world and not all of it.
    let outlines = |show: bool| {
        let mut scene = Scene::new(true);
        scene.map.set_zoom(2.0);
        scene.map.settings.minimap.show_viewport = show;
        let shapes = scene.frame(vec![]);
        let (rect, _) = Scene::panel(&shapes, &scene.map.settings.minimap).expect("minimap panel");
        shapes
            .iter()
            .filter(|clipped| match &clipped.shape {
                Shape::Rect(r) => {
                    r.rect != rect && r.stroke.width >= 1.5 && rect.contains_rect(r.rect)
                }
                _ => false,
            })
            .count()
    };
    assert_eq!(outlines(false), 0);
    assert!(outlines(true) > 0);
}

#[test]
fn the_viewport_outline_is_centred_on_the_view() {
    let mut scene = Scene::new(true);
    scene.map.set_zoom(2.0);
    scene.map.set_pos([500.0, 250.0]);
    let shapes = scene.frame(vec![]);
    let (rect, area) = Scene::panel(&shapes, &scene.map.settings.minimap).expect("minimap panel");

    let outline = shapes
        .iter()
        .find_map(|clipped| match &clipped.shape {
            Shape::Rect(r)
                if r.rect != rect && r.stroke.width >= 1.5 && rect.contains_rect(r.rect) =>
            {
                Some(r.rect)
            }
            _ => None,
        })
        .expect("viewport outline");

    let expected = rect.center() + (vec2(500.0, 250.0) - vec2(1000.0, 500.0)) * Scene::scale(rect);
    assert!(
        outline.center().distance(expected) < 0.5,
        "{:?} vs {expected:?}",
        outline.center()
    );
    // It spans `area / zoom` map units, so it has the proportions of `area`.
    let aspect = outline.width() / outline.height();
    assert!((aspect - area.width() / area.height()).abs() < 0.05);
}

#[test]
fn pressing_the_minimap_recentres_the_view() {
    let mut scene = Scene::new(true);
    scene.map.set_pos([100.0, 100.0]);
    let (rect, _) = scene.shown_panel();

    // The middle of the minimap is the middle of the world.
    scene.press_at(rect.center());
    let [x, y] = scene.map.get_pos();
    assert!(
        (x - 1000.0).abs() < 1.0 && (y - 500.0).abs() < 1.0,
        "{x},{y}"
    );
}

#[test]
fn pressing_off_centre_goes_to_the_matching_map_position() {
    let mut scene = Scene::new(true);
    let (rect, _) = scene.shown_panel();

    let offset = vec2(30.0, -20.0);
    scene.press_at(rect.center() + offset);
    let scale = Scene::scale(rect);
    let [x, y] = scene.map.get_pos();
    assert!((x - (1000.0 + offset.x / scale)).abs() < 1.0, "{x}");
    assert!((y - (500.0 + offset.y / scale)).abs() < 1.0, "{y}");
}

#[test]
fn a_press_outside_the_minimap_leaves_the_view_alone() {
    let mut scene = Scene::new(true);
    scene.map.set_pos([100.0, 100.0]);
    let (rect, area) = scene.shown_panel();
    assert!(!rect.contains(area.center()));

    scene.press_at(area.center());
    assert_eq!(scene.map.get_pos(), [100.0, 100.0]);
}

#[test]
fn a_disabled_minimap_ignores_the_pointer() {
    let (rect, _) = Scene::new(true).shown_panel();

    let mut off = Scene::new(false);
    off.map.set_pos([100.0, 100.0]);
    off.press_at(rect.center());
    assert_eq!(off.map.get_pos(), [100.0, 100.0]);
}

#[test]
fn a_map_without_nodes_draws_no_minimap() {
    let mut scene = Scene::new(true);
    scene.map = Map::new();
    scene.map.settings.minimap.enabled = true;
    let shapes = scene.frame(vec![]);
    assert!(Scene::panel(&shapes, &scene.map.settings.minimap).is_none());
}

#[test]
fn nodes_are_crisp_pixel_aligned_squares() {
    let mut scene = Scene::new(true);
    let shapes = scene.frame(vec![]);
    let (rect, _) = Scene::panel(&shapes, &scene.map.settings.minimap).expect("minimap panel");

    let nodes: Vec<Rect> = shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            Shape::Rect(node) if node.rect.size() == Vec2::splat(1.0) => Some(node.rect),
            _ => None,
        })
        .collect();
    assert_eq!(nodes.len(), 3);
    for node in nodes {
        assert!(rect.contains_rect(node), "{node:?} outside {rect:?}");
        // Whole pixels at the default scale: nothing left to anti-alias.
        assert_eq!(node.min, node.min.round(), "{node:?}");
    }
}

#[test]
fn the_node_size_is_configurable() {
    let mut scene = Scene::new(true);
    scene.map.settings.minimap.point_size = 3.0;
    let shapes = scene.frame(vec![]);
    let nodes = shapes
        .iter()
        .filter(|clipped| match &clipped.shape {
            Shape::Rect(node) => node.rect.size() == Vec2::splat(3.0),
            _ => false,
        })
        .count();
    assert_eq!(nodes, 3);
}

#[test]
fn the_minimap_has_the_proportions_of_the_map() {
    let mut scene = Scene::new(true);
    let (rect, _) = scene.shown_panel();
    // The map is twice as wide as tall; the panel keeps that, less its padding.
    let ratio = (rect.width() - 12.0) / (rect.height() - 12.0);
    assert!((ratio - 2.0).abs() < 0.05, "{rect:?} has ratio {ratio}");
    let (_, area) = scene.shown_panel();
    let boxed = scene.map.settings.minimap.rect_in(area);
    assert!(boxed.contains_rect(rect));
    // The whole box was not used: that is what the proportions are for.
    assert!(rect.height() < boxed.height());
}

#[test]
fn the_proportions_can_be_left_out() {
    let mut scene = Scene::new(true);
    scene.map.settings.minimap.keep_aspect_ratio = false;
    let (rect, area) = scene.shown_panel();
    assert_eq!(rect, scene.map.settings.minimap.rect_in(area));
}

#[test]
fn the_minimap_is_a_fraction_of_the_widget() {
    let mut scene = Scene::new(true);
    scene.map.settings.minimap.keep_aspect_ratio = false;
    scene.map.settings.minimap.scale = 0.25;
    let (rect, area) = scene.shown_panel();
    assert!(
        (rect.width() - area.width() * 0.25).abs() < 0.5,
        "{rect:?} in {area:?}"
    );
    assert!(
        (rect.height() - area.height() * 0.25).abs() < 0.5,
        "{rect:?} in {area:?}"
    );
}

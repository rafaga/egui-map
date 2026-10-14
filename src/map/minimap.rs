//! A small overview of the whole map, pinned to a corner of the widget.
//!
//! The minimap is part of [`Map`] rather than a separate widget,
//! so it reuses the nodes, segments and theme the map already holds. It is
//! **off by default**: turn it on with [`MinimapSettings::enabled`] through
//! [`MapSettings::minimap`](super::objects::MapSettings::minimap).
//!
//! ```
//! use egui_map::map::Map;
//! use egui_map::map::overlay::Corner;
//!
//! let mut map = Map::new();
//! map.settings.minimap.enabled = true;
//! map.settings.minimap.anchor = Corner::BottomLeft;
//! ```
//!
//! The minimap keeps the proportions of the map it shows (see
//! [`MinimapSettings::keep_aspect_ratio`]), so its panel is as wide and tall
//! as the map is, within the box [`MinimapSettings::scale`] gives it.
//!
//! While it is on, the minimap draws every node and segment scaled to fit,
//! plus (with [`MinimapSettings::show_viewport`]) the part of the map the main
//! view currently shows. Pressing or dragging inside it recentres the main
//! view on that point; the pointer is then owned by the minimap, so the main
//! map does not pan underneath it.
//!
//! All its colors come from the active [`MapTheme`](super::theme::MapTheme):
//! the background, the node and segment colors, and `selected` for the
//! viewport rectangle.

use super::Map;
use super::overlay::Corner;
use super::path::SegmentPath;
use egui::emath::GuiRounding;
use egui::{
    Color32, CornerRadius, CursorIcon, Painter, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui,
    Vec2, pos2, vec2,
};

/// Configuration of the minimap overlay; see the [module](self) docs.
///
/// Reached as [`MapSettings::minimap`](super::objects::MapSettings::minimap).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MinimapSettings {
    /// Whether the minimap is drawn and reacts to the pointer. `false` by
    /// default, so a map that does not ask for it looks and behaves as before.
    pub enabled: bool,
    /// How large the minimap is, as a fraction of the map widget: `0.2` makes
    /// the box it has to fit in a fifth of the widget's width and height,
    /// whatever size the widget is resized to. Clamped to `0.0..=1.0`, and
    /// shrunk further when the widget is too small to hold it together with
    /// [`margin`](Self::margin). With
    /// [`keep_aspect_ratio`](Self::keep_aspect_ratio) the panel is then
    /// trimmed to the proportions of the map.
    pub scale: f32,
    /// Whether the minimap keeps the proportions of the map it shows. `true`
    /// by default: the panel shrinks to the largest rectangle of the map's
    /// shape that fits in the box [`scale`](Self::scale) gives, so a wide map
    /// gets a wide minimap instead of one with empty bands above and below.
    /// `false` always uses all of that box, however different its shape is
    /// from the map's.
    pub keep_aspect_ratio: bool,
    /// Corner of the map it is pinned to.
    pub anchor: Corner,
    /// Gap, in screen pixels, between the minimap and the edges of the map.
    pub margin: f32,
    /// Opacity of the minimap, `0.0` (invisible) to `1.0` (opaque); values
    /// outside that range are clamped.
    pub opacity: f32,
    /// Whether to outline the part of the map the main view shows.
    pub show_viewport: bool,
    /// Side, in screen pixels, of the square each node is drawn as. Squares
    /// are snapped to the pixel grid so they stay crisp; the size is never
    /// drawn below one physical pixel. Lower it for a dense map, raise it for
    /// a sparse one.
    pub point_size: f32,
}

impl Default for MinimapSettings {
    /// Disabled, a fifth of the widget (`scale` `0.2`) keeping the map's
    /// proportions, bottom-right, a `10` pixel margin, `85%` opaque, with the viewport
    /// outlined and nodes one pixel wide.
    fn default() -> Self {
        Self {
            enabled: false,
            scale: 0.2,
            keep_aspect_ratio: true,
            anchor: Corner::BottomRight,
            margin: 10.0,
            opacity: 0.85,
            show_viewport: true,
            point_size: 1.0,
        }
    }
}

impl MinimapSettings {
    /// The largest rectangle the minimap can take inside `area`, the region
    /// the map paints into: `area` scaled by [`scale`](Self::scale), capped so
    /// that it, plus the margin on both sides, fits in `area`.
    ///
    /// This is the box [`fitted_rect`](Self::fitted_rect) fits the map's shape
    /// into; it is the minimap itself only when
    /// [`keep_aspect_ratio`](Self::keep_aspect_ratio) is off.
    pub fn rect_in(&self, area: Rect) -> Rect {
        self.anchored(area, self.capped_size(area))
    }

    /// The rectangle the minimap occupies inside `area` for a map whose nodes
    /// span `world` map units.
    ///
    /// With [`keep_aspect_ratio`](Self::keep_aspect_ratio) it is the largest
    /// rectangle of the map's proportions that fits in
    /// [`rect_in`](Self::rect_in), pinned to the same corner. Otherwise, or
    /// when `world` has no extent on one axis (a single node, or nodes in a
    /// line), it is `rect_in` itself.
    pub fn fitted_rect(&self, area: Rect, world: Vec2) -> Rect {
        let capped = self.capped_size(area);
        if !self.keep_aspect_ratio || !(world.x > 0.0 && world.y > 0.0) {
            return self.anchored(area, capped);
        }
        let padding = Vec2::splat(2.0 * MinimapTransform::PADDING);
        let available = capped - padding;
        if !(available.x > 0.0 && available.y > 0.0) {
            return self.anchored(area, capped);
        }
        let scale = (available.x / world.x).min(available.y / world.y);
        // Whole pixels, so the panel's border stays crisp.
        let size = (world * scale + padding).round().min(capped);
        self.anchored(area, size)
    }

    /// `area` scaled by [`scale`](Self::scale), capped to what fits in it next
    /// to the margin.
    fn capped_size(&self, area: Rect) -> Vec2 {
        let margin = self.margin.max(0.0);
        (area.size() * self.scale.clamp(0.0, 1.0))
            .min(area.size() - Vec2::splat(2.0 * margin))
            .max(Vec2::ZERO)
    }

    /// A rectangle of `size` pinned to this settings' corner of `area`.
    fn anchored(&self, area: Rect, size: Vec2) -> Rect {
        self.anchor.place(area, size, self.margin.max(0.0))
    }

    /// The opacity actually painted with: [`opacity`](Self::opacity) clamped
    /// to `0.0..=1.0`.
    fn alpha(&self) -> f32 {
        self.opacity.clamp(0.0, 1.0)
    }
}

/// Maps between map coordinates and the minimap's screen rectangle: the whole
/// node set, centred and uniformly scaled to fit inside [`Self::rect`].
#[derive(Clone, Copy, Debug)]
pub(crate) struct MinimapTransform {
    rect: Rect,
    /// Centre of the node set, in map coordinates.
    center: [f32; 2],
    /// Screen pixels per map unit.
    scale: f32,
}

impl MinimapTransform {
    /// Space kept free between the node set and the edge of the minimap.
    const PADDING: f32 = 6.0;

    /// Fits the bounds `min..=max` (map coordinates) into `rect`.
    ///
    /// `None` when `rect` is too small to draw anything in.
    pub(crate) fn fit(rect: Rect, min: [f32; 2], max: [f32; 2]) -> Option<Self> {
        let available = rect.size() - Vec2::splat(2.0 * Self::PADDING);
        if !(available.x > 0.0 && available.y > 0.0) {
            return None;
        }
        // A zero span (a single node, or nodes in a line) makes that axis
        // unconstraining rather than dividing by zero.
        let fit_axis = |available: f32, span: f32| {
            if span > 0.0 {
                available / span
            } else {
                f32::INFINITY
            }
        };
        let scale =
            fit_axis(available.x, max[0] - min[0]).min(fit_axis(available.y, max[1] - min[1]));
        Some(Self {
            rect,
            center: [(min[0] + max[0]) / 2.0, (min[1] + max[1]) / 2.0],
            scale: if scale.is_finite() { scale } else { 1.0 },
        })
    }

    /// Where a map coordinate falls on the minimap.
    pub(crate) fn to_screen(self, point: [f32; 2]) -> Pos2 {
        let center = self.rect.center();
        pos2(
            center.x + (point[0] - self.center[0]) * self.scale,
            center.y + (point[1] - self.center[1]) * self.scale,
        )
    }

    /// The map coordinate under a minimap position.
    pub(crate) fn to_world(self, position: Pos2) -> [f32; 2] {
        let center = self.rect.center();
        [
            self.center[0] + (position.x - center.x) / self.scale,
            self.center[1] + (position.y - center.y) / self.scale,
        ]
    }
}

impl Map {
    /// The minimap's rectangle and transform for a map painted into `area`, or
    /// `None` when it is off, there is nothing to show or it has no room.
    pub(super) fn minimap_geometry(&self, area: Rect) -> Option<(Rect, MinimapTransform)> {
        if !self.settings.minimap.enabled || self.points.as_ref()?.is_empty() {
            return None;
        }
        let (min, max): ([f32; 2], [f32; 2]) =
            (self.reference.min.into(), self.reference.max.into());
        let world = vec2(max[0] - min[0], max[1] - min[1]);
        let rect = self.settings.minimap.fitted_rect(area, world);
        let transform = MinimapTransform::fit(rect, min, max)?;
        Some((rect, transform))
    }

    /// Recentres the main view on the minimap position being pressed or
    /// dragged. Runs at the start of the frame, before the view is projected,
    /// so the map follows the pointer without a frame of lag.
    ///
    /// Registering the minimap as an interactive widget is also what keeps the
    /// main map from panning, hovering or opening its context menu underneath
    /// it: egui gives the pointer to the topmost widget.
    pub(super) fn handle_minimap_input(&mut self, ui: &mut Ui, area: Rect) {
        let Some((rect, transform)) = self.minimap_geometry(area) else {
            return;
        };
        let response = ui.interact(
            rect,
            ui.id().with("egui_map_minimap"),
            Sense::click_and_drag(),
        );
        let pressed = ui.input(|input| input.pointer.primary_down());
        if pressed
            && response.is_pointer_button_down_on()
            && let Some(position) = response.interact_pointer_pos()
        {
            let _span = tracing::info_span!("minimap_recentre").entered();
            let position = position.clamp(rect.min, rect.max);
            self.set_pos(transform.to_world(position));
        }
        response.on_hover_cursor(CursorIcon::PointingHand);
    }

    /// Paints the minimap: a themed panel, every segment and node scaled to
    /// fit, and the outline of the region the main view shows.
    pub(super) fn paint_minimap(&self, paint: &Painter, area: Rect) {
        let Some((rect, transform)) = self.minimap_geometry(area) else {
            return;
        };
        let _span = tracing::info_span!("paint_minimap").entered();
        let alpha = self.settings.minimap.alpha();
        let theme = self.theme_colors();
        let tint = |color: Color32| color.gamma_multiply(alpha);
        let inside = paint.with_clip_rect(rect.intersect(paint.clip_rect()));

        paint.rect_filled(rect, CornerRadius::same(4), tint(theme.background));

        let mut shapes: Vec<Shape> = Vec::new();
        if let Some(segments) = &self.segments {
            let default_curvature = self.settings.default_curvature;
            shapes.extend(segments.iter().map(|segment| {
                // The minimap's scale is the same on both axes, so the curve
                // of the map is the same curve here, just smaller.
                let path = SegmentPath::curved(
                    transform.to_screen(segment.point1),
                    transform.to_screen(segment.point2),
                    segment.curvature_with(default_curvature),
                );
                path.stroke_shape(Stroke::new(
                    1.0,
                    tint(segment.color.unwrap_or(theme.segment)),
                ))
            }));
        }
        if let Some(points) = &self.points {
            // One pixel-aligned square per node: a circle this small is
            // smeared by anti-aliasing over several pixels, which turns a
            // dense map into a solid blob.
            let pixels_per_point = paint.pixels_per_point();
            let side = Vec2::splat(self.settings.minimap.point_size.max(1.0 / pixels_per_point));
            let color = tint(theme.node);
            shapes.extend(points.values().map(|point| {
                let node = Rect::from_center_size(transform.to_screen(point.coords), side)
                    .round_to_pixels(pixels_per_point);
                Shape::rect_filled(node, CornerRadius::ZERO, color)
            }));
        }
        inside.extend(shapes);

        if self.settings.minimap.show_viewport {
            // The main view shows `map_area / zoom` map units around the
            // position it is centred on (see `Map::ui`).
            let half = self.map_area.size() / (2.0 * self.zoom);
            let center: [f32; 2] = self.reference.pos.into();
            let viewport = Rect::from_two_pos(
                transform.to_screen([center[0] - half.x, center[1] - half.y]),
                transform.to_screen([center[0] + half.x, center[1] + half.y]),
            )
            .intersect(rect);
            if viewport.is_positive() {
                inside.rect_filled(
                    viewport,
                    CornerRadius::ZERO,
                    tint(theme.selected).gamma_multiply(0.2),
                );
                inside.rect_stroke(
                    viewport,
                    CornerRadius::ZERO,
                    Stroke::new(1.5, tint(theme.selected)),
                    StrokeKind::Inside,
                );
            }
        }

        paint.rect_stroke(
            rect,
            CornerRadius::same(4),
            Stroke::new(1.0, tint(theme.segment)),
            StrokeKind::Inside,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area() -> Rect {
        Rect::from_min_size(pos2(100.0, 50.0), vec2(600.0, 400.0))
    }

    #[test]
    fn it_is_off_by_default() {
        assert!(!MinimapSettings::default().enabled);
        assert!(!crate::map::objects::MapSettings::new().minimap.enabled);
        assert!(!crate::map::objects::MapSettings::default().minimap.enabled);
    }

    #[test]
    fn each_anchor_pins_the_minimap_to_its_corner_with_the_margin() {
        let settings = MinimapSettings {
            scale: 0.25,
            margin: 10.0,
            ..MinimapSettings::default()
        };
        let area = area();
        let expected = [
            (Corner::TopLeft, pos2(110.0, 60.0)),
            (Corner::TopRight, pos2(540.0, 60.0)),
            (Corner::BottomLeft, pos2(110.0, 340.0)),
            (Corner::BottomRight, pos2(540.0, 340.0)),
        ];
        for (anchor, min) in expected {
            let rect = MinimapSettings { anchor, ..settings }.rect_in(area);
            assert_eq!(rect.min, min, "{anchor:?}");
            assert_eq!(rect.size(), vec2(150.0, 100.0), "{anchor:?}");
        }
    }

    #[test]
    fn the_minimap_size_follows_the_scale_of_the_widget() {
        let small = MinimapSettings {
            scale: 0.2,
            ..MinimapSettings::default()
        };
        let large = MinimapSettings {
            scale: 0.4,
            ..small
        };
        assert_eq!(small.rect_in(area()).size(), vec2(120.0, 80.0));
        assert_eq!(large.rect_in(area()).size(), vec2(240.0, 160.0));

        // A bigger widget gets a bigger minimap at the same scale.
        let double = Rect::from_min_size(pos2(0.0, 0.0), area().size() * 2.0);
        assert_eq!(small.rect_in(double).size(), vec2(240.0, 160.0));
    }

    #[test]
    fn the_scale_is_clamped_to_the_widget() {
        let area = area();
        for scale in [1.0, 3.0] {
            let settings = MinimapSettings {
                scale,
                margin: 0.0,
                ..MinimapSettings::default()
            };
            assert_eq!(settings.rect_in(area).size(), area.size(), "{scale}");
        }
        for scale in [0.0, -1.0] {
            let settings = MinimapSettings {
                scale,
                ..MinimapSettings::default()
            };
            assert_eq!(settings.rect_in(area).size(), Vec2::ZERO, "{scale}");
        }
    }

    #[test]
    fn the_minimap_keeps_the_proportions_of_the_map_by_default() {
        let settings = MinimapSettings {
            scale: 0.5,
            margin: 10.0,
            ..MinimapSettings::default()
        };
        assert!(settings.keep_aspect_ratio);
        // The box is half the 600 x 400 area.
        assert_eq!(settings.rect_in(area()).size(), vec2(300.0, 200.0));
        let pad = 2.0 * MinimapTransform::PADDING;

        // Twice as wide as tall: the width is the limit.
        let wide = settings.fitted_rect(area(), vec2(2000.0, 1000.0));
        assert_eq!(wide.width(), 300.0);
        assert!(((wide.width() - pad) / (wide.height() - pad) - 2.0).abs() < 0.02);
        // Twice as tall as wide: the height is.
        let tall = settings.fitted_rect(area(), vec2(1000.0, 2000.0));
        assert_eq!(tall.height(), 200.0);
        assert!(((tall.height() - pad) / (tall.width() - pad) - 2.0).abs() < 0.02);
        // Never larger than the box it fits in.
        assert!(settings.rect_in(area()).contains_rect(wide));
        assert!(settings.rect_in(area()).contains_rect(tall));
    }

    #[test]
    fn a_fitted_minimap_stays_pinned_to_its_corner() {
        let world = vec2(2000.0, 1000.0);
        for anchor in [
            Corner::TopLeft,
            Corner::TopRight,
            Corner::BottomLeft,
            Corner::BottomRight,
        ] {
            let settings = MinimapSettings {
                scale: 0.5,
                margin: 10.0,
                anchor,
                ..MinimapSettings::default()
            };
            let (full, fitted) = (
                settings.rect_in(area()),
                settings.fitted_rect(area(), world),
            );
            let corner = |rect: Rect| match anchor {
                Corner::TopLeft => rect.left_top(),
                Corner::TopRight => rect.right_top(),
                Corner::BottomLeft => rect.left_bottom(),
                Corner::BottomRight => rect.right_bottom(),
            };
            assert_eq!(corner(fitted), corner(full), "{anchor:?}");
        }
    }

    #[test]
    fn without_keeping_the_proportions_the_minimap_uses_all_its_size() {
        let settings = MinimapSettings {
            keep_aspect_ratio: false,
            ..MinimapSettings::default()
        };
        assert_eq!(
            settings.fitted_rect(area(), vec2(2000.0, 1000.0)),
            settings.rect_in(area())
        );
    }

    #[test]
    fn a_map_with_no_extent_on_an_axis_uses_the_whole_box() {
        let settings = MinimapSettings::default();
        for world in [vec2(0.0, 0.0), vec2(500.0, 0.0), vec2(0.0, 500.0)] {
            assert_eq!(
                settings.fitted_rect(area(), world),
                settings.rect_in(area()),
                "{world:?}"
            );
        }
    }

    #[test]
    fn the_minimap_shrinks_to_fit_a_small_map() {
        let settings = MinimapSettings {
            scale: 1.0,
            margin: 10.0,
            ..MinimapSettings::default()
        };
        let small = Rect::from_min_size(pos2(0.0, 0.0), vec2(120.0, 90.0));
        let rect = settings.rect_in(small);
        assert_eq!(rect.size(), vec2(100.0, 70.0));
        assert!(small.contains_rect(rect));

        let tiny = Rect::from_min_size(pos2(0.0, 0.0), vec2(10.0, 10.0));
        assert_eq!(settings.rect_in(tiny).size(), Vec2::ZERO);
    }

    #[test]
    fn the_transform_round_trips_and_keeps_the_node_set_inside() {
        let rect = Rect::from_min_size(pos2(10.0, 20.0), vec2(200.0, 100.0));
        let (min, max) = ([-50.0, -10.0], [150.0, 30.0]);
        let transform = MinimapTransform::fit(rect, min, max).unwrap();

        for corner in [min, max, [min[0], max[1]], [max[0], min[1]]] {
            let on_screen = transform.to_screen(corner);
            assert!(rect.contains(on_screen), "{corner:?} -> {on_screen:?}");
            let back = transform.to_world(on_screen);
            assert!((back[0] - corner[0]).abs() < 1e-3 && (back[1] - corner[1]).abs() < 1e-3);
        }
        // The node set is centred on the minimap.
        let middle = transform.to_screen([50.0, 10.0]);
        assert!(middle.distance(rect.center()) < 1e-3);
    }

    #[test]
    fn the_transform_keeps_the_aspect_ratio() {
        let rect = Rect::from_min_size(Pos2::ZERO, vec2(300.0, 300.0));
        let transform = MinimapTransform::fit(rect, [0.0, 0.0], [100.0, 50.0]).unwrap();
        let (a, b) = (
            transform.to_screen([0.0, 0.0]),
            transform.to_screen([100.0, 50.0]),
        );
        assert!(((b.x - a.x) / (b.y - a.y) - 2.0).abs() < 1e-3);
    }

    #[test]
    fn a_degenerate_node_set_still_has_a_finite_transform() {
        let rect = Rect::from_min_size(Pos2::ZERO, vec2(200.0, 100.0));
        for (min, max) in [
            ([5.0, 5.0], [5.0, 5.0]),
            ([0.0, 3.0], [10.0, 3.0]),
            ([3.0, 0.0], [3.0, 10.0]),
        ] {
            let transform = MinimapTransform::fit(rect, min, max).unwrap();
            let on_screen = transform.to_screen(min);
            assert!(on_screen.x.is_finite() && on_screen.y.is_finite());
            assert!(
                rect.contains(on_screen),
                "{min:?}..{max:?} -> {on_screen:?}"
            );
        }
    }

    #[test]
    fn a_minimap_with_no_room_has_no_transform() {
        let rect = Rect::from_min_size(Pos2::ZERO, vec2(8.0, 8.0));
        assert!(MinimapTransform::fit(rect, [0.0, 0.0], [1.0, 1.0]).is_none());
        assert!(MinimapTransform::fit(Rect::ZERO, [0.0, 0.0], [1.0, 1.0]).is_none());
    }
}

//! The zoom slider drawn over the map, with the zoom as a percentage under it.
//!
//! It is pinned to a corner like the [minimap](super::minimap), and the two
//! share the same [`Corner`]s, margin default and edge alignment, so they read
//! as one set of controls. By default the slider takes the top-right corner
//! and the minimap the bottom-right one, a column along the right edge. When
//! both are asked to share a corner the slider stacks on the inner side of the
//! minimap instead of covering it: above it in a bottom corner, below it in a
//! top one.
//!
//! The slider can lie [vertical or horizontal](SliderSettings::orientation)
//! and carries five tick marks, at 0, 25, 50, 75 and 100% of its travel, so
//! the zoom levels can be told apart at a glance. The marks sit on the side of
//! the slider that faces the inside of the map, whatever the corner, and have
//! no labels of their own.
//!
//! The slider, its marks and its read-out are grouped in a rounded panel
//! filled with the theme's `background`. The panel is fully transparent
//! ([`panel_idle_opacity`](SliderSettings::panel_idle_opacity)) until the mouse
//! goes over it (or the slider is dragged), and then it turns half opaque
//! ([`panel_hover_opacity`](SliderSettings::panel_hover_opacity)) so the slider
//! stands out from the map behind it.
//!
//! ```
//! use egui::SliderOrientation;
//! use egui_map::map::Map;
//! use egui_map::map::overlay::Corner;
//!
//! let mut map = Map::new();
//! map.settings.slider.anchor = Corner::BottomLeft;
//! map.settings.slider.orientation = SliderOrientation::Horizontal;
//! // Or drop the marks, or hide it, to drive the zoom with a control of your own:
//! map.settings.slider.ticks = false;
//! map.settings.slider.visible = false;
//! ```

use super::Map;
use super::overlay::Corner;
use egui::emath::GuiRounding;
use egui::style::HandleShape;
use egui::{
    Align2, CornerRadius, Pos2, Rect, Sense, Slider, SliderOrientation, Stroke, TextStyle, Ui,
    UiBuilder, Vec2, lerp, pos2, vec2,
};

/// Configuration of the zoom slider; see the [module](self) docs.
///
/// Reached as [`MapSettings::slider`](super::objects::MapSettings::slider).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderSettings {
    /// Whether the slider, and the zoom read-out under it, are drawn. `true`
    /// by default; turn it off to zoom with a control of your own through
    /// [`Map::set_zoom`] (the mouse wheel keeps working either way).
    pub visible: bool,
    /// Corner of the map it is pinned to.
    pub anchor: Corner,
    /// Gap, in screen pixels, between the slider and the edges of the map.
    pub margin: f32,
    /// Whether the slider stands upright (the default, with the maximum zoom
    /// at the top) or lies down (the maximum at the right).
    pub orientation: SliderOrientation,
    /// Whether the five tick marks (0, 25, 50, 75 and 100% of the travel) are
    /// drawn next to the slider. `true` by default.
    pub ticks: bool,
    /// Opacity of the rounded panel behind the control while the mouse is
    /// away from it, from `0.0` (invisible, the default) to `1.0`.
    pub panel_idle_opacity: f32,
    /// Opacity of the panel while the mouse is over it, or the slider is being
    /// dragged, from `0.0` to `1.0`. `0.6` by default: semi opaque.
    pub panel_hover_opacity: f32,
}

impl Default for SliderSettings {
    /// Visible, top-right, a `10` pixel margin (the same the minimap uses, so
    /// the two line up along an edge), vertical, with tick marks, on a panel that is invisible until the
    /// mouse is over it.
    fn default() -> Self {
        Self {
            visible: true,
            anchor: Corner::TopRight,
            margin: 10.0,
            orientation: SliderOrientation::Vertical,
            ticks: true,
            panel_idle_opacity: 0.0,
            panel_hover_opacity: 0.6,
        }
    }
}

/// Where the parts of the control go inside its rectangle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderLayout {
    /// The slider widget.
    pub rail: Rect,
    /// The top-centre of the zoom read-out.
    pub label: Pos2,
}

impl SliderSettings {
    /// Length of the slider rail, in screen pixels.
    pub const RAIL_LENGTH: f32 = 100.0;
    /// Breadth of a vertical control: the slider with its marks, and the
    /// read-out under it, fit in it.
    pub const BREADTH: f32 = 44.0;
    /// Space between the rail and the read-out, and between the control and a
    /// minimap it is stacked against.
    pub const GAP: f32 = 4.0;
    /// Space between the edge of the panel and what it holds.
    pub const PADDING: f32 = 6.0;
    /// Corner radius of the panel: the minimap's.
    pub const ROUNDING: u8 = 4;
    /// Length of a tick mark.
    pub const TICK_LENGTH: f32 = 5.0;
    /// Space between the rail and its tick marks.
    pub const TICK_GAP: f32 = 3.0;
    /// How many tick marks there are: one every quarter of the travel.
    pub const TICK_COUNT: usize = 5;

    /// The opacity of the panel, given whether it is `active` (the mouse is
    /// over it or the slider is being dragged). Always within `0..=1`.
    pub fn panel_opacity(&self, active: bool) -> f32 {
        let opacity = if active {
            self.panel_hover_opacity
        } else {
            self.panel_idle_opacity
        };
        opacity.clamp(0.0, 1.0)
    }

    /// What the tick marks add to the control across the slider.
    fn tick_band(&self) -> f32 {
        if self.ticks {
            Self::TICK_GAP + Self::TICK_LENGTH
        } else {
            0.0
        }
    }

    /// The size of the whole control, its panel included, for a slider `thickness` thick and a
    /// read-out `label_height` tall.
    pub fn control_size(&self, thickness: f32, label_height: f32) -> Vec2 {
        let across = thickness + self.tick_band();
        let inner = match self.orientation {
            SliderOrientation::Vertical => vec2(
                across.max(Self::BREADTH),
                Self::RAIL_LENGTH + Self::GAP + label_height,
            ),
            SliderOrientation::Horizontal => {
                vec2(Self::RAIL_LENGTH, across + Self::GAP + label_height)
            }
        };
        inner + Vec2::splat(2.0 * Self::PADDING)
    }

    /// Where the slider and the read-out go inside `control`, a rectangle of
    /// [`control_size`](Self::control_size).
    ///
    /// The slider hugs the outer edge of the corner and the marks go on its
    /// inner side. A vertical control keeps the read-out under the slider; a
    /// horizontal one in a bottom corner puts it above, so it is always on the
    /// side of the marks.
    pub fn layout(&self, control: Rect, thickness: f32, label_height: f32) -> SliderLayout {
        let band = self.tick_band();
        // What the panel holds, inside its padding.
        let control = control.shrink(Self::PADDING);
        match self.orientation {
            SliderOrientation::Vertical => {
                let group = thickness + band;
                let left = control.center().x - group / 2.0;
                let x = if self.anchor.is_left() {
                    left
                } else {
                    left + band
                };
                SliderLayout {
                    rail: Rect::from_min_size(
                        pos2(x, control.top()),
                        vec2(thickness, Self::RAIL_LENGTH),
                    ),
                    label: pos2(control.center().x, control.bottom() - label_height),
                }
            }
            SliderOrientation::Horizontal => {
                let (y, label_y) = if self.anchor.is_top() {
                    (control.top(), control.bottom() - label_height)
                } else {
                    (control.bottom() - thickness, control.top())
                };
                SliderLayout {
                    rail: Rect::from_min_size(
                        pos2(control.left(), y),
                        vec2(Self::RAIL_LENGTH, thickness),
                    ),
                    label: pos2(control.center().x, label_y),
                }
            }
        }
    }

    /// The tick marks of a slider drawn in `rail`, as the two ends of each
    /// line, from the minimum zoom to the maximum.
    ///
    /// They are spread over the travel of the slider's handle, not over the
    /// whole rail: egui keeps the handle a little inside the ends of the rail,
    /// by an amount that depends on its `handle_shape`, and the marks have to
    /// line up with the handle at those zooms. Each mark sits on a pixel
    /// centre (`pixels_per_point`) so it is drawn crisp. They are on the inner
    /// side of the anchor: left of a vertical slider in a right corner, below
    /// a horizontal one in a top corner, and so on.
    pub fn tick_marks(
        &self,
        rail: Rect,
        handle_shape: HandleShape,
        pixels_per_point: f32,
    ) -> [[Pos2; 2]; Self::TICK_COUNT] {
        let across = match self.orientation {
            SliderOrientation::Vertical => rail.width(),
            SliderOrientation::Horizontal => rail.height(),
        };
        // The same radius egui derives from the slider's thickness.
        let radius = across / 2.5;
        let reach = match handle_shape {
            HandleShape::Circle => radius,
            HandleShape::Rect { aspect_ratio } => radius * aspect_ratio,
        };
        let (a, b) = (-Self::TICK_GAP, -(Self::TICK_GAP + Self::TICK_LENGTH));
        // `near` and `far` are signed offsets from the rail edge facing the
        // inside of the map.
        let (edge, sign) = match self.orientation {
            SliderOrientation::Vertical if self.anchor.is_left() => (rail.right(), -1.0),
            SliderOrientation::Vertical => (rail.left(), 1.0),
            SliderOrientation::Horizontal if self.anchor.is_top() => (rail.bottom(), -1.0),
            SliderOrientation::Horizontal => (rail.top(), 1.0),
        };
        let (near, far) = (edge + sign * a, edge + sign * b);
        std::array::from_fn(|index| {
            let fraction = index as f32 / (Self::TICK_COUNT - 1) as f32;
            match self.orientation {
                SliderOrientation::Horizontal => {
                    let x = lerp(rail.left() + reach..=rail.right() - reach, fraction)
                        .round_to_pixel_center(pixels_per_point);
                    [pos2(x, near), pos2(x, far)]
                }
                // The largest zoom is at the top.
                SliderOrientation::Vertical => {
                    let y = lerp(rail.bottom() - reach..=rail.top() + reach, fraction)
                        .round_to_pixel_center(pixels_per_point);
                    [pos2(near, y), pos2(far, y)]
                }
            }
        })
    }

    /// Where a control of `size` goes inside `area`, the region the map paints
    /// into, avoiding `minimap` -- the minimap's rectangle, when there is one.
    ///
    /// It is pinned to [`anchor`](Self::anchor). If that would cover the
    /// minimap, it is moved to the minimap's inner side (above it in a bottom
    /// corner, below it in a top one) as long as it still fits in `area`; if
    /// it does not, it stays where it was.
    ///
    /// `None` when the control does not fit in `area` at all.
    pub fn rect_in(&self, area: Rect, size: Vec2, minimap: Option<Rect>) -> Option<Rect> {
        let margin = self.margin.max(0.0);
        let room = area.shrink(margin);
        if size.x > room.width() || size.y > room.height() {
            return None;
        }
        let pinned = self.anchor.place(area, size, margin);
        let Some(minimap) = minimap.filter(|minimap| minimap.intersects(pinned)) else {
            return Some(pinned);
        };
        let top = if self.anchor.is_top() {
            minimap.bottom() + Self::GAP
        } else {
            minimap.top() - Self::GAP - size.y
        };
        let stacked = Rect::from_min_size(pos2(pinned.left(), top), size);
        Some(if room.contains_rect(stacked) {
            stacked
        } else {
            pinned
        })
    }
}

impl Map {
    /// Draws the zoom slider, its tick marks and its read-out, if they are
    /// visible and fit.
    pub(super) fn paint_slider(&mut self, ui: &mut Ui, area: Rect) {
        let settings = self.settings.slider;
        if !settings.visible {
            return;
        }
        let _span = tracing::info_span!("map_ui_paint_slider").entered();
        let label_font = TextStyle::Small.resolve(ui.style());
        let label_height = ui.text_style_height(&TextStyle::Small);
        // An egui slider is `interact_size.y` thick and `slider_width` long.
        let thickness = ui.spacing().interact_size.y;
        let size = settings.control_size(thickness, label_height);
        let minimap = self.minimap_geometry(area).map(|(rect, _)| rect);
        let Some(control) = settings.rect_in(area, size, minimap) else {
            return;
        };
        let layout = settings.layout(control, thickness, label_height);

        // The panel turns semi opaque while the mouse is over the control, or
        // the slider is being dragged (the pointer may leave it meanwhile).
        let dragging_id = ui.id().with("map_slider_dragging");
        let dragging = ui
            .data(|data| data.get_temp::<bool>(dragging_id))
            .unwrap_or(false);
        let over = ui
            .input(|input| input.pointer.hover_pos())
            .is_some_and(|pos| control.contains(pos));
        let colors = self.theme_colors();
        let panel = settings.panel_opacity(over || dragging);
        if panel > 0.0 {
            ui.painter().rect_filled(
                control,
                CornerRadius::same(SliderSettings::ROUNDING),
                colors.background.gamma_multiply(panel),
            );
        }
        // Under the slider, so a press on the panel does not pan the map.
        ui.interact(control, dragging_id.with("panel"), Sense::click_and_drag());

        let slider = Slider::new(
            &mut self.zoom,
            self.settings.min_zoom..=self.settings.max_zoom,
        )
        .show_value(false)
        .orientation(settings.orientation);
        let response = ui
            .scope_builder(UiBuilder::new().max_rect(layout.rail), |ui| {
                ui.spacing_mut().slider_width = SliderSettings::RAIL_LENGTH;
                ui.add(slider)
            })
            .inner;
        ui.data_mut(|data| data.insert_temp(dragging_id, response.dragged()));

        if settings.ticks {
            let stroke = Stroke::new(1.0, colors.text.gamma_multiply(0.6));
            let marks = settings.tick_marks(
                layout.rail,
                ui.visuals().handle_shape,
                ui.pixels_per_point(),
            );
            for mark in marks {
                ui.painter().line_segment(mark, stroke);
            }
        }
        // The zoom at a glance, next to the slider. Drawn after it so it
        // shows the value the slider just set this frame.
        ui.painter().text(
            layout.label,
            Align2::CENTER_TOP,
            self.zoom_label(),
            label_font,
            colors.text,
        );
    }

    /// The zoom factor as a percentage -- `1.0` reads `100%` -- the simplified
    /// form shown next to the zoom slider.
    pub(super) fn zoom_label(&self) -> String {
        format!("{:.0}%", self.zoom * 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area() -> Rect {
        Rect::from_min_size(pos2(100.0, 50.0), vec2(600.0, 400.0))
    }

    fn control() -> Vec2 {
        SliderSettings::default().control_size(18.0, 12.0)
    }

    fn settings(anchor: Corner) -> SliderSettings {
        SliderSettings {
            anchor,
            ..SliderSettings::default()
        }
    }

    #[test]
    fn it_is_visible_in_the_top_right_by_default() {
        let defaults = SliderSettings::default();
        assert!(defaults.visible);
        assert_eq!(defaults.anchor, Corner::TopRight);
        assert_eq!(defaults.margin, 10.0);
    }

    #[test]
    fn it_is_pinned_to_its_corner_with_the_margin() {
        for anchor in [
            Corner::TopLeft,
            Corner::TopRight,
            Corner::BottomLeft,
            Corner::BottomRight,
        ] {
            let rect = settings(anchor).rect_in(area(), control(), None).unwrap();
            assert_eq!(rect, anchor.place(area(), control(), 10.0), "{anchor:?}");
        }
    }

    #[test]
    fn it_does_not_move_for_a_minimap_in_another_corner() {
        let minimap = Corner::BottomRight.place(area(), vec2(180.0, 100.0), 10.0);
        let rect = settings(Corner::TopRight)
            .rect_in(area(), control(), Some(minimap))
            .unwrap();
        assert_eq!(rect, Corner::TopRight.place(area(), control(), 10.0));
    }

    #[test]
    fn in_a_bottom_corner_it_stacks_above_the_minimap() {
        let minimap = Corner::BottomRight.place(area(), vec2(180.0, 100.0), 10.0);
        let rect = settings(Corner::BottomRight)
            .rect_in(area(), control(), Some(minimap))
            .unwrap();
        assert_eq!(rect.bottom(), minimap.top() - SliderSettings::GAP);
        // Same edge as the minimap, and no overlap.
        assert_eq!(rect.right(), minimap.right());
        assert!(!rect.intersects(minimap));
    }

    #[test]
    fn in_a_top_corner_it_stacks_below_the_minimap() {
        let minimap = Corner::TopLeft.place(area(), vec2(180.0, 100.0), 10.0);
        let rect = settings(Corner::TopLeft)
            .rect_in(area(), control(), Some(minimap))
            .unwrap();
        assert_eq!(rect.top(), minimap.bottom() + SliderSettings::GAP);
        assert_eq!(rect.left(), minimap.left());
        assert!(!rect.intersects(minimap));
    }

    #[test]
    fn when_the_stack_does_not_fit_it_stays_pinned() {
        // A minimap nearly as tall as the area leaves no room above it.
        let minimap = Corner::BottomRight.place(area(), vec2(180.0, 370.0), 10.0);
        let rect = settings(Corner::BottomRight)
            .rect_in(area(), control(), Some(minimap))
            .unwrap();
        assert_eq!(rect, Corner::BottomRight.place(area(), control(), 10.0));
    }

    #[test]
    fn it_is_not_drawn_in_a_map_too_small_for_it() {
        let tiny = Rect::from_min_size(pos2(0.0, 0.0), vec2(60.0, 100.0));
        assert!(
            settings(Corner::TopRight)
                .rect_in(tiny, control(), None)
                .is_none()
        );
    }

    fn oriented(anchor: Corner, orientation: SliderOrientation) -> SliderSettings {
        SliderSettings {
            anchor,
            orientation,
            ..SliderSettings::default()
        }
    }

    const ALL: [Corner; 4] = [
        Corner::TopLeft,
        Corner::TopRight,
        Corner::BottomLeft,
        Corner::BottomRight,
    ];

    #[test]
    fn the_panel_is_invisible_when_idle_and_semi_opaque_under_the_mouse() {
        let defaults = SliderSettings::default();
        assert_eq!(defaults.panel_opacity(false), 0.0);
        assert!(defaults.panel_opacity(true) > 0.0 && defaults.panel_opacity(true) < 1.0);
    }

    #[test]
    fn the_panel_opacity_is_kept_within_zero_and_one() {
        let settings = SliderSettings {
            panel_idle_opacity: -1.0,
            panel_hover_opacity: 3.0,
            ..SliderSettings::default()
        };
        assert_eq!(settings.panel_opacity(false), 0.0);
        assert_eq!(settings.panel_opacity(true), 1.0);
    }

    #[test]
    fn it_is_vertical_with_tick_marks_by_default() {
        let defaults = SliderSettings::default();
        assert_eq!(defaults.orientation, SliderOrientation::Vertical);
        assert!(defaults.ticks);
    }

    #[test]
    fn a_horizontal_control_is_wide_and_short() {
        let vertical = oriented(Corner::TopRight, SliderOrientation::Vertical);
        let horizontal = oriented(Corner::TopRight, SliderOrientation::Horizontal);
        let (v, h) = (
            vertical.control_size(18.0, 12.0),
            horizontal.control_size(18.0, 12.0),
        );
        assert!(v.y > v.x);
        assert!(h.x > h.y);
        assert_eq!(
            h.x,
            SliderSettings::RAIL_LENGTH + 2.0 * SliderSettings::PADDING
        );
    }

    #[test]
    fn the_marks_make_the_control_a_little_thicker() {
        let with = SliderSettings {
            orientation: SliderOrientation::Horizontal,
            ..SliderSettings::default()
        };
        let without = SliderSettings {
            ticks: false,
            ..with
        };
        let band = SliderSettings::TICK_GAP + SliderSettings::TICK_LENGTH;
        assert_eq!(
            with.control_size(18.0, 12.0).y - without.control_size(18.0, 12.0).y,
            band
        );
    }

    #[test]
    fn the_parts_stay_inside_the_control_and_do_not_overlap() {
        for orientation in [SliderOrientation::Vertical, SliderOrientation::Horizontal] {
            for anchor in ALL {
                let settings = oriented(anchor, orientation);
                let size = settings.control_size(18.0, 12.0);
                let control = Rect::from_min_size(pos2(40.0, 30.0), size);
                let layout = settings.layout(control, 18.0, 12.0);
                let label = Rect::from_center_size(layout.label + vec2(0.0, 6.0), vec2(26.0, 12.0));
                let ctx = format!("{anchor:?} {orientation:?}");
                assert!(control.contains_rect(layout.rail), "{ctx}");
                assert!(!layout.rail.intersects(label), "{ctx}");
                for mark in settings.tick_marks(layout.rail, HandleShape::Circle, 1.0) {
                    for end in mark {
                        assert!(control.expand(0.01).contains(end), "{ctx}: {end:?}");
                    }
                    assert!(
                        !label.intersects(Rect::from_two_pos(mark[0], mark[1])),
                        "{ctx}"
                    );
                }
            }
        }
    }

    #[test]
    fn five_marks_are_spread_evenly_from_the_minimum_to_the_maximum() {
        let rail = Rect::from_min_size(pos2(0.0, 0.0), vec2(18.0, 100.0));
        let settings = oriented(Corner::TopRight, SliderOrientation::Vertical);
        let marks = settings.tick_marks(rail, HandleShape::Circle, 1.0);
        let ys: Vec<f32> = marks.iter().map(|mark| mark[0].y).collect();
        // Largest zoom (index 4) at the top, smallest at the bottom.
        assert!(ys.windows(2).all(|pair| pair[0] > pair[1]), "{ys:?}");
        let steps: Vec<f32> = ys.windows(2).map(|pair| pair[0] - pair[1]).collect();
        // Pixel rounding may move a mark by up to a pixel.
        assert!(
            steps.iter().all(|step| (step - steps[0]).abs() <= 1.0),
            "{steps:?}"
        );
        // The ends are the handle's travel: the radius (18 / 2.5) inside.
        assert!((ys[0] - (100.0 - 7.2)).abs() <= 0.5 + 0.01, "{ys:?}");
        assert!((ys[4] - 7.2).abs() <= 0.5 + 0.01, "{ys:?}");
    }

    #[test]
    fn horizontal_marks_run_left_to_right() {
        let rail = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 18.0));
        let settings = oriented(Corner::TopLeft, SliderOrientation::Horizontal);
        let marks = settings.tick_marks(rail, HandleShape::Circle, 1.0);
        let xs: Vec<f32> = marks.iter().map(|mark| mark[0].x).collect();
        assert!(xs.windows(2).all(|pair| pair[0] < pair[1]), "{xs:?}");
        for mark in marks {
            assert_eq!(mark[0].x, mark[1].x);
        }
    }

    #[test]
    fn the_marks_are_on_the_side_facing_the_inside_of_the_map() {
        let vertical = Rect::from_min_size(pos2(50.0, 0.0), vec2(18.0, 100.0));
        let horizontal = Rect::from_min_size(pos2(0.0, 50.0), vec2(100.0, 18.0));
        for anchor in ALL {
            let [start, _] = oriented(anchor, SliderOrientation::Vertical).tick_marks(
                vertical,
                HandleShape::Circle,
                1.0,
            )[0];
            // In a left corner the inside is to the right of the slider.
            assert_eq!(
                start.x > vertical.center().x,
                anchor.is_left(),
                "{anchor:?}"
            );
            let [start, _] = oriented(anchor, SliderOrientation::Horizontal).tick_marks(
                horizontal,
                HandleShape::Circle,
                1.0,
            )[0];
            // In a top corner the inside is below it.
            assert_eq!(
                start.y > horizontal.center().y,
                anchor.is_top(),
                "{anchor:?}"
            );
        }
    }

    #[test]
    fn the_marks_start_off_the_rail_and_point_away_from_it() {
        let rail = Rect::from_min_size(pos2(50.0, 0.0), vec2(18.0, 100.0));
        let settings = oriented(Corner::TopRight, SliderOrientation::Vertical);
        let [near, far] = settings.tick_marks(rail, HandleShape::Circle, 1.0)[2];
        assert_eq!(near.x, rail.left() - SliderSettings::TICK_GAP);
        assert_eq!(
            far.x,
            rail.left() - SliderSettings::TICK_GAP - SliderSettings::TICK_LENGTH
        );
    }

    #[test]
    fn a_rect_handle_reaches_less_far_in_than_a_round_one() {
        let rail = Rect::from_min_size(pos2(0.0, 0.0), vec2(18.0, 100.0));
        let settings = oriented(Corner::TopRight, SliderOrientation::Vertical);
        let round = settings.tick_marks(rail, HandleShape::Circle, 1.0)[0][0].y;
        let flat = settings.tick_marks(rail, HandleShape::Rect { aspect_ratio: 0.5 }, 1.0)[0][0].y;
        assert!(flat > round);
    }
}

//! Built-in animation effects for nodes.
//!
//! Two families, distinguished by how they handle time and by when they stop:
//!
//! - **Event-driven** effects ([`Animation::pulse`], [`Animation::ripple`],
//!   [`Animation::countdown_arc`], [`Animation::scale_in`],
//!   [`Animation::crosshair`]) are anchored to the [`Instant`] an event
//!   happened and **terminate**: they return `true` while still playing and
//!   `false` once finished, so the caller can drop the entry and stop
//!   repainting.
//! - **Persistent** effects ([`Animation::halo`], [`Animation::blink`],
//!   [`Animation::orbit`]) never end. They take the **frame time** in seconds
//!   (`ui.input(|i| i.time)`) rather than an `Instant`, so every element
//!   animated in the same frame shares one clock and cannot drift apart.
//!
//! Persistent effects require the caller to keep requesting repaints, which
//! turns an idle app into one redrawing continuously — use them for a handful
//! of elements, not for every node.
//!
//! Both families are reached through
//! [`Map::node`](crate::map::Map::node); see [`NodeHandle`](crate::map::NodeHandle).
//! They are also useful from a custom
//! [`NodeTemplate`](crate::map::objects::NodeTemplate): call them from
//! `notification_ui` / `marker_ui` instead of reimplementing the effect. When
//! you do, remember to call `ui.ctx().request_repaint()` yourself — the widget
//! only does that for its own built-in path.

use egui::{
    Color32, Painter, Pos2, Shape, Stroke,
    epaint::{CircleShape, PathShape},
};
use std::f32::consts::TAU;
use std::time::Instant;

/// How long [`Animation::pulse`] plays, in seconds.
pub const PULSE_DURATION: f32 = 3.5;
/// How long [`Animation::ripple`] plays, in seconds.
pub const RIPPLE_DURATION: f32 = 3.5;
/// How long [`Animation::countdown_arc`] takes to empty, in seconds.
pub const COUNTDOWN_DURATION: f32 = 5.0;
/// How long [`Animation::scale_in`] plays, in seconds.
pub const SCALE_IN_DURATION: f32 = 0.45;
/// How long [`Animation::crosshair`] takes to converge, in seconds.
pub const CROSSHAIR_DURATION: f32 = 0.6;

/// Returns `color` with its alpha replaced by `alpha` (clamped to `0.0..=1.0`).
fn with_alpha(color: Color32, alpha: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(
        color.r(),
        color.g(),
        color.b(),
        (255.0 * alpha.clamp(0.0, 1.0)).round() as u8,
    )
}

/// Seconds elapsed since `initial_time`.
fn elapsed(initial_time: Instant) -> f32 {
    Instant::now().duration_since(initial_time).as_secs_f32()
}

/// A `0 -> 1 -> 0` triangle wave of the given `period`, in seconds.
fn triangle_wave(time: f32, period: f32) -> f32 {
    let phase = (time / period).rem_euclid(1.0);
    1.0 - (2.0 * phase - 1.0).abs()
}

/// Overshooting ease-out, so a scale-in settles with a small bounce.
fn ease_out_back(x: f32) -> f32 {
    const C1: f32 = 1.701_58;
    const C3: f32 = C1 + 1.0;
    let x1 = x - 1.0;
    1.0 + C3 * x1 * x1 * x1 + C1 * x1 * x1
}

/// Factory for the built-in node animations.
///
/// See the [module docs](self) for the difference between the event-driven and
/// persistent families.
pub struct Animation {}

impl Animation {
    // ---------------------------------------------------------------- events

    /// One frame of an expanding, fading disc centred on `center`.
    ///
    /// Reads as *"one thing happened here"*. Plays for [`PULSE_DURATION`].
    /// Returns `true` while still playing.
    pub fn pulse(
        painter: &Painter,
        center: Pos2,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        let secs = elapsed(initial_time);
        let radius = (4.00 + (40.00 * secs)) * zoom;
        let transparency = (1.00 - (secs / PULSE_DURATION).abs()).max(0.0);
        painter.add(Shape::Circle(CircleShape::filled(
            center,
            radius,
            with_alpha(color, transparency),
        )));
        secs < PULSE_DURATION
    }

    /// One frame of three staggered expanding rings.
    ///
    /// Where [`Animation::pulse`] reads as a single event, the repetition here
    /// reads as *"activity is ongoing"*. Plays for [`RIPPLE_DURATION`].
    /// Returns `true` while still playing.
    pub fn ripple(
        painter: &Painter,
        center: Pos2,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        const RINGS: usize = 3;
        let secs = elapsed(initial_time);
        let stagger = RIPPLE_DURATION / RINGS as f32;

        let mut shapes = Vec::with_capacity(RINGS);
        for ring in 0..RINGS {
            let local = secs - ring as f32 * stagger;
            if !(0.0..RIPPLE_DURATION).contains(&local) {
                continue;
            }
            let progress = local / RIPPLE_DURATION;
            shapes.push(Shape::Circle(CircleShape::stroke(
                center,
                (4.0 + 36.0 * progress) * zoom,
                Stroke::new(2.0 * zoom, with_alpha(color, 1.0 - progress)),
            )));
        }
        painter.extend(shapes);
        secs < RIPPLE_DURATION
    }

    /// One frame of a ring that empties clockwise from 12 o'clock.
    ///
    /// The remaining arc is the remaining fraction of [`COUNTDOWN_DURATION`],
    /// which makes it a natural fit for *"how old is this information"*.
    /// Returns `true` while still playing.
    pub fn countdown_arc(
        painter: &Painter,
        center: Pos2,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        // Segments in a full turn; the arc draws a prefix of these.
        const STEPS: usize = 48;
        let secs = elapsed(initial_time);
        let remaining = (1.0 - secs / COUNTDOWN_DURATION).clamp(0.0, 1.0);
        let radius = 10.0 * zoom;

        let count = (STEPS as f32 * remaining).round() as usize;
        if count >= 1 {
            let points = (0..=count)
                .map(|i| {
                    // Start at 12 o'clock and sweep clockwise. Screen y grows
                    // downwards, so a growing angle already turns clockwise.
                    let angle = TAU * (i as f32 / STEPS as f32) - TAU / 4.0;
                    Pos2::new(
                        center.x + radius * angle.cos(),
                        center.y + radius * angle.sin(),
                    )
                })
                .collect();
            painter.add(Shape::Path(PathShape::line(
                points,
                Stroke::new(2.0 * zoom, with_alpha(color, 1.0)),
            )));
        }
        secs < COUNTDOWN_DURATION
    }

    /// One frame of a disc that grows past its final size and settles back.
    ///
    /// Meant for nodes that just appeared. Plays for [`SCALE_IN_DURATION`].
    /// Returns `true` while still playing.
    pub fn scale_in(
        painter: &Painter,
        center: Pos2,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        let secs = elapsed(initial_time);
        let progress = (secs / SCALE_IN_DURATION).clamp(0.0, 1.0);
        let radius = 8.0 * zoom * ease_out_back(progress).max(0.0);
        painter.add(Shape::Circle(CircleShape::filled(
            center,
            radius,
            with_alpha(color, 1.0 - progress),
        )));
        secs < SCALE_IN_DURATION
    }

    /// One frame of four ticks converging onto the node.
    ///
    /// Reads as *"target acquired"*; pairs well with selection. Plays for
    /// [`CROSSHAIR_DURATION`]. Returns `true` while still playing.
    pub fn crosshair(
        painter: &Painter,
        center: Pos2,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        let secs = elapsed(initial_time);
        let progress = (secs / CROSSHAIR_DURATION).clamp(0.0, 1.0);
        // Ticks travel from far away down to just outside the node, and fade
        // out over the last third so they do not linger on top of it.
        let far = (30.0 - 18.0 * progress) * zoom;
        let near = far - 8.0 * zoom;
        let alpha = if progress < 0.66 {
            1.0
        } else {
            1.0 - (progress - 0.66) / 0.34
        };
        let stroke = Stroke::new(2.0 * zoom, with_alpha(color, alpha));

        let mut shapes = Vec::with_capacity(4);
        for (dx, dy) in [(0.0, -1.0), (0.0, 1.0), (-1.0, 0.0), (1.0, 0.0)] {
            shapes.push(Shape::line_segment(
                [
                    Pos2::new(center.x + dx * far, center.y + dy * far),
                    Pos2::new(center.x + dx * near, center.y + dy * near),
                ],
                stroke,
            ));
        }
        painter.extend(shapes);
        secs < CROSSHAIR_DURATION
    }

    // ------------------------------------------------------------ persistent

    /// One frame of a ring whose opacity breathes in and out.
    ///
    /// For lasting state — *"you are here"*, *"this system is camped"*. `time`
    /// is the frame time in seconds (`ui.input(|i| i.time)`).
    ///
    /// The radius has a floor in screen pixels so the halo does not vanish
    /// when the map is zoomed far out.
    pub fn halo(painter: &Painter, center: Pos2, zoom: f32, time: f32, color: Color32) {
        const PERIOD: f32 = 2.0;
        let alpha = 0.30 + 0.45 * triangle_wave(time, PERIOD);
        let radius = (9.0 * zoom).max(5.0);
        painter.add(Shape::Circle(CircleShape::stroke(
            center,
            radius,
            Stroke::new((2.0 * zoom).max(1.5), with_alpha(color, alpha)),
        )));
    }

    /// One frame of a thick ring blinking on and off.
    ///
    /// This is the effect markers have always used, factored out of the widget
    /// so it can be selected and reused like any other. `time` is the frame
    /// time in seconds.
    pub fn blink(painter: &Painter, center: Pos2, zoom: f32, time: f32, color: Color32) {
        const PERIOD: f32 = 2.55;
        painter.add(Shape::Circle(CircleShape::stroke(
            center,
            4.0 * zoom,
            Stroke::new(9.0 * zoom, with_alpha(color, triangle_wave(time, PERIOD))),
        )));
    }

    /// One frame of a dot orbiting the node, with a faint guide ring.
    ///
    /// Reads as *"under observation"*. `time` is the frame time in seconds.
    pub fn orbit(painter: &Painter, center: Pos2, zoom: f32, time: f32, color: Color32) {
        const PERIOD: f32 = 3.0;
        let radius = (12.0 * zoom).max(7.0);
        let angle = TAU * (time / PERIOD).rem_euclid(1.0);
        let dot = Pos2::new(
            center.x + radius * angle.cos(),
            center.y + radius * angle.sin(),
        );
        painter.extend([
            Shape::Circle(CircleShape::stroke(
                center,
                radius,
                Stroke::new(1.0, with_alpha(color, 0.25)),
            )),
            Shape::Circle(CircleShape::filled(
                dot,
                (2.5 * zoom).max(2.0),
                with_alpha(color, 1.0),
            )),
        ]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Context, LayerId, Rect, Vec2};
    use std::time::Duration;

    fn headless_painter() -> Painter {
        Painter::new(
            Context::default(),
            LayerId::background(),
            Rect::from_min_size(Pos2::ZERO, Vec2::new(100.0, 100.0)),
        )
    }

    /// Every event-driven effect, with the duration it is supposed to run for.
    #[allow(clippy::type_complexity)]
    fn event_effects() -> Vec<(
        &'static str,
        fn(&Painter, Pos2, f32, Instant, Color32) -> bool,
        f32,
    )> {
        vec![
            ("pulse", Animation::pulse, PULSE_DURATION),
            ("ripple", Animation::ripple, RIPPLE_DURATION),
            (
                "countdown_arc",
                Animation::countdown_arc,
                COUNTDOWN_DURATION,
            ),
            ("scale_in", Animation::scale_in, SCALE_IN_DURATION),
            ("crosshair", Animation::crosshair, CROSSHAIR_DURATION),
        ]
    }

    #[test]
    fn event_effects_report_running_then_finished() {
        let painter = headless_painter();
        for (name, effect, duration) in event_effects() {
            assert!(
                effect(&painter, Pos2::ZERO, 1.0, Instant::now(), Color32::RED),
                "{name} must report it is still running when it just started"
            );

            let long_past = Instant::now() - Duration::from_secs_f32(duration + 1.0);
            assert!(
                !effect(&painter, Pos2::ZERO, 1.0, long_past, Color32::RED),
                "{name} must report it is finished once its duration has passed"
            );
        }
    }

    #[test]
    fn every_event_effect_stays_under_the_orphan_sweep() {
        // `Map` drops notifications older than 10s as a safety net. An effect
        // that outlived it would be cut off mid-play.
        for (name, _, duration) in event_effects() {
            assert!(
                duration < 10.0,
                "{name} lasts {duration}s, which the 10s orphan sweep would truncate"
            );
        }
    }

    #[test]
    fn persistent_effects_run_at_any_time() {
        let painter = headless_painter();
        for time in [0.0, 0.7, 1.3, 60.0] {
            Animation::halo(&painter, Pos2::ZERO, 1.0, time, Color32::GREEN);
            Animation::blink(&painter, Pos2::ZERO, 1.0, time, Color32::GREEN);
            Animation::orbit(&painter, Pos2::ZERO, 1.0, time, Color32::GREEN);
        }
    }

    #[test]
    fn triangle_wave_goes_up_and_back_down() {
        assert_eq!(triangle_wave(0.0, 2.0), 0.0);
        assert_eq!(triangle_wave(1.0, 2.0), 1.0);
        assert!(triangle_wave(2.0, 2.0).abs() < 1e-6);
        assert!((triangle_wave(3.0, 2.0) - 1.0).abs() < 1e-6);
        for step in 0..200 {
            let v = triangle_wave(step as f32 * 0.05, 2.55);
            assert!((0.0..=1.0).contains(&v), "{v} out of range");
        }
    }

    #[test]
    fn ease_out_back_overshoots_then_settles() {
        assert_eq!(ease_out_back(0.0), 0.0);
        assert!((ease_out_back(1.0) - 1.0).abs() < 1e-5);
        let peak = (0..=100)
            .map(|i| ease_out_back(i as f32 / 100.0))
            .fold(f32::MIN, f32::max);
        assert!(
            peak > 1.0,
            "ease_out_back should overshoot, peaked at {peak}"
        );
    }

    #[test]
    fn with_alpha_clamps_out_of_range_values() {
        assert_eq!(with_alpha(Color32::RED, 2.0).a(), 255);
        assert_eq!(with_alpha(Color32::RED, -1.0).a(), 0);
        assert_eq!(with_alpha(Color32::RED, 1.0), Color32::RED);
    }
}

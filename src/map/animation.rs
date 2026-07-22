//! Built-in animation effects, used when no custom
//! [`NodeTemplate`](crate::map::objects::NodeTemplate) is installed.

use crate::map::objects::RawPoint;
use egui::{Color32, Painter, Shape, epaint::CircleShape};
use std::time::Instant;

/// Factory for the default node notification animation.
pub(crate) struct Animation {}

impl Animation {
    /// Draws one frame of an expanding, fading circle centered on `center`.
    ///
    /// The pulse starts at `initial_time` and plays for about 3.5 seconds,
    /// growing in radius while its transparency decreases. Returns `true`
    /// while the animation is still playing (the caller should request a
    /// repaint) and `false` once it has finished, so the caller can drop the
    /// notification.
    pub(crate) fn pulse(
        painter: &Painter,
        center: RawPoint,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        let current_instant = Instant::now();
        let time_diff = current_instant.duration_since(initial_time);
        let secs_played = time_diff.as_secs_f32();
        let radius = (4.00 + (40.00 * secs_played)) * zoom;
        let mut transparency = 1.00 - (secs_played / 3.50).abs();
        if transparency < 0.00 {
            transparency = 0.00;
        }
        let corrected_color = Color32::from_rgba_unmultiplied(
            color.r(),
            color.g(),
            color.b(),
            (255.00 * transparency).round() as u8,
        );
        let circle = Shape::Circle(CircleShape::filled(center.into(), radius, corrected_color));
        painter.extend(vec![circle]);
        secs_played < 3.50
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Context, LayerId, Pos2, Rect, Vec2};
    use std::time::Duration;

    fn headless_painter() -> Painter {
        Painter::new(
            Context::default(),
            LayerId::background(),
            Rect::from_min_size(Pos2::ZERO, Vec2::new(100.0, 100.0)),
        )
    }

    #[test]
    fn pulse_returns_true_while_running() {
        let painter = headless_painter();
        let result = Animation::pulse(
            &painter,
            RawPoint::default(),
            1.0,
            Instant::now(),
            Color32::RED,
        );
        assert!(result);
    }

    #[test]
    fn pulse_returns_false_when_finished() {
        let painter = headless_painter();
        // la animación dura 3.5 segundos; 4 segundos después ya terminó
        let initial_time = Instant::now() - Duration::from_secs(4);
        let result = Animation::pulse(
            &painter,
            RawPoint::default(),
            1.0,
            initial_time,
            Color32::RED,
        );
        assert!(!result);
    }
}

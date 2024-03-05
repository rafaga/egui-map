use crate::map::Error;
use egui::{epaint::CircleShape, Color32, Painter, Pos2, Shape};
use std::time::Instant;

pub(crate) struct Animation {}

impl Animation {
    pub(crate) fn pulse(
        painter: &Painter,
        center: Pos2,
        zoom: f32,
        initial_time: Instant,
    ) -> Result<bool, Error> {
        let current_instant = Instant::now();
        let mut result = false;
        let time_diff = current_instant.duration_since(initial_time);
        let secs_played = time_diff.as_secs_f32();
        // This is in beta state
        let radius = (4.00 + (40.00 * secs_played)) * zoom;
        let mut transparency = 1.00 - (secs_played / 3.50).abs();
        if transparency < 0.00 {
            transparency = 0.00;
        }
        let color =
            Color32::from_rgba_unmultiplied(128, 12, 67, (255.00 * transparency).round() as u8);
        let circle = Shape::Circle(CircleShape::filled(center, radius, color));
        painter.extend(vec![circle]);
        if secs_played < 3.50 {
            result = true;
        }
        Ok(result)
    }
}

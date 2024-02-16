use crate::map::Error;
use egui::{epaint::CircleShape, Color32, Pos2, Shape, Ui};
use std::time::Instant;

pub(crate) struct Animation {}

impl Animation {
    pub(crate) fn pulse(
        ui: &Ui,
        center: Pos2,
        zoom: f32,
        initial_time: Instant,
    ) -> Result<bool, Error> {
        let time_diff = initial_time - Instant::now();
        let secs_played = time_diff.as_secs_f32();
        let mut result = true;
        let current_instant = Instant::now();
        let duration = current_instant.duration_since(initial_time);
        // This is in beta state
        let radius = (4.00 + (40.00 * duration.as_secs_f32())) * zoom;
        let mut transparency = secs_played / 3.50;
        if transparency > 1.00 {
            transparency = 1.00;
        }
        let color =
            Color32::from_rgba_unmultiplied(128, 12, 67, (255.00 * transparency).round() as u8);
        let circle = Shape::Circle(CircleShape::filled(center, radius, color));
        ui.painter().extend(vec![circle]);
        if secs_played >= 3.50 {
            result = false;
        }
        Ok(result)
    }
}

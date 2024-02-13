use egui::{epaint::CircleShape, Color32, Pos2, Rect, Shape, Ui};
use std::time::{Duration, Instant};

pub(crate) struct AnimationPoint{
    pub position: Pos2,
    pub time: Instant
}

impl AnimationPoint{
    pub(crate) fn new(x:f32,y:f32,time:Instant) -> Self {
        Self {
            position: Pos2::new(x,y),
            time
        }
    }
}

pub struct AnimationManager{
    pub(crate) notifications: Vec<AnimationPoint>,
    rect: Rect,
}

impl AnimationManager {
    pub(crate) fn new() -> Self{
        AnimationManager{
            notifications: vec![],
            rect: Rect::NOTHING,
        }
    }

    pub(crate) fn animation_loop(&self, ui: &mut Ui) {
        for node in &self.notifications {
            Self::star_animation(&ui, node.position, node.time);
        }
    }

    fn star_animation(ui:&Ui, center: Pos2, initial_time:Instant){
        let current_instant = Instant::now();
        let duration = current_instant.duration_since(initial_time);
        let radius = duration.as_secs_f32();
        let color = Color32::from_rgba_unmultiplied(128, 12, 67, 100);
        let circle = Shape::Circle(CircleShape::filled(center, radius, color));
        ui.painter().extend(vec![circle]);
        //ui.allocate_painter(desired_size, sense)
    }
}

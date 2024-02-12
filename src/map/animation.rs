use egui::{epaint::CircleShape, Color32, Pos2, Rect, Shape, Ui};
use std::time::{Duration, Instant};

pub struct AnimationPoint{
    position: Pos2,
    time: Instant
}

impl AnimationPoint{
    pub fn new(x:f32,y:f32,time:Instant) -> Self {
        Self {
            position: Pos2::new(x,y),
            time
        }
    }
}

pub(crate) struct AnimationManager{
    notifications: Vec<AnimationPoint>,
    rect: Rect,
}

impl AnimationManager {
    pub fn new() -> Self{
        AnimationManager{
            notifications: vec![],
            rect: Rect::NOTHING,
        }
    }

    fn star_animation(self, ui:Ui, center: Pos2, initial_time:Instant){
        let current_instant = Instant::now();
        let duration = current_instant.duration_since(initial_time);
        let radius = duration.as_secs_f32();
        let color = Color32::from_rgba_unmultiplied(128, 12, 67, 100);
        let circle = Shape::Circle(CircleShape::filled(center, radius, color));
        //ui.allocate_painter(desired_size, sense)
    }

    pub fn animate_node(ui:Ui, point:Pos2) {
        //ui.ctx();
    }
}

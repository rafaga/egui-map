use egui::{containers::*, widgets::*, *};
pub struct Map {
    pub zoom: f32,
}

impl Default for Map {
    fn default() -> Self {
        Map {
            zoom: 1.0,
        }
    }
}

impl Widget for Map {
    fn ui(self, _: &mut egui::Ui) -> Response { 
        todo!()
    }
}

impl Map {
    pub fn new() -> Map {
        Map {
            zoom: 1.0,
        }
    }
}
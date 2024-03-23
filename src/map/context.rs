use egui::{Response,Ui,Pos2};

pub trait MenuManager{
    fn ui(&mut self,ui: &mut Ui) -> Response;
    fn open(&mut self, response: &Response);
    fn close(&mut self);
}

pub struct ContextMenu{
    opened: bool,
    position: Option<Pos2>,
}

impl ContextMenu{
    pub fn new() -> Self {
        Self {
            opened: false,
            position: None,
        }
    }
}

impl MenuManager for ContextMenu{
    fn ui(&mut self,ui: &mut Ui) -> Response {
        ui.allocate_response(ui.max_rect().size(), egui::Sense::click_and_drag())
    }
    fn open(&mut self, response: &Response) {
        self.opened = true;
        self.position = response.interact_pointer_pos();
    }
    fn close(&mut self) {
        self.opened = false;
    }
} 
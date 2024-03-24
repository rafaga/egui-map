use egui::{Rect,Ui,Vec2,Sense};

pub trait MenuManager{
    fn ui(&mut self,ui: &mut Ui);
    fn open(&mut self, rect: &Rect);
    fn close(&mut self);
}

pub struct ContextMenuManager{
    pub(crate) opened: bool,
    size: Option<Vec2>,
}

impl ContextMenuManager{
    pub fn new() -> Self {
        Self {
            opened: false,
            size: None,
        }
    }
}

impl MenuManager for ContextMenuManager{
    fn ui(&mut self,ui: &mut Ui){
        let resp = ui.allocate_response(self.size.unwrap(),Sense::click());
        resp.context_menu(|ui|{
            if ui.button("Settings").clicked() {

            }
        });
    }
    fn open(&mut self, rect: &Rect) {
        self.opened = true;
        self.size = Some(rect.size());
    }
    fn close(&mut self) {
        self.opened = false;
        self.size = None;
    }
} 
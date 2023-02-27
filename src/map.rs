use egui::{containers::*, widgets::*, *};
use egui::Painter;


pub struct Map {
    pub zoom: f32,
    pub pos: Vec2,
    pub points: Option<Vec<Pos2>>,
}

impl Default for Map {
    fn default() -> Self {
        Map::new()
    }
}

impl Widget for Map {
    fn ui(self, ui_obj: &mut egui::Ui) -> Response {

        // 1. Deciding widget size:
        // You can query the `ui` how much space is available,
        // but in this example we have a fixed size widget based on the height of a standard button:
        let desired_size = ui_obj.spacing().interact_size.y * egui::vec2(16.0, 16.0);

        // 2. Allocating space:
        // This is where we get a region of the screen assigned.
        // We also tell the Ui to sense clicks in the allocated region.
        let (rect, mut response) = ui_obj.allocate_exact_size(desired_size, egui::Sense::click());

        if response.hovered(){
            let position = response.hover_pos();
            response.mark_changed();
        }
        // 4. Paint!
        // Make sure we need to paint:
        if ui_obj.is_rect_visible(rect) {
            // Let's ask for a simple animation from egui.
            // egui keeps track of changes in the boolean associated with the id and
            // returns an animated value in the 0-1 range for how much "on" we are.
            //let how_on = ui_obj.ctx().animate_bool(response.id, *on);

            // We will follow the current style by asking
            // "how should something that is being interacted with be painted?".
            // This will, for instance, give us different colors when the widget is hovered or clicked.
            let visuals = ui_obj.style().interact(&response);

            if let Some(tree) = self.points{
                
            }
            else {
                let rect = ui_obj.ctx().available_rect();
                let mut reply = ui_obj.allocate_at_least(egui::vec2(100.0, 200.0), egui::Sense::click_and_drag());
                let pos = Pos2{x:reply.0.left(), y:reply.0.top() };
                let font_id = FontId{size:20.0, family:FontFamily::Proportional};
                ui_obj.painter().text( pos,Align2::LEFT_TOP, "loading", font_id, Color32::LIGHT_RED);
            }
            
            /*
            // All coordinates are in absolute screen coordinates so we use `rect` to place the elements.
            let rect = rect.expand(visuals.expansion);
            let radius = 0.5 * rect.height();
            ui_obj.painter()
                .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
            // Paint the circle, animating it from left to right with `how_on`:
            //let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
            let center = egui::pos2(circle_x, rect.center().y);
            ui_obj.painter()
                .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
            */
        }
        response
    }
}

impl Map {
    pub fn new() -> Map {
        Map {
            zoom: 1.0,
            pos: Vec2 { x: 0.0f32, y: 0.0f32 },
            points: None,
        }
    }

    fn Paintsystems() -> () {

    }
}
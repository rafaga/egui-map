//! Custom node templates paint through the `Ui` they are handed
//! (`ui.painter()`), so that `Ui` must be clipped to the map's drawable area:
//! otherwise a node near the edge is drawn over the widget's frame and
//! beyond it (onto neighbouring panels).

use egui::{Context, RawInput, Rect, Ui};
use egui_map::map::Map;
use egui_map::map::objects::{
    MapPoint, MarkerContext, NodeContext, NodeTemplate, NotificationContext, SelectionContext,
};
use std::cell::RefCell;
use std::rc::Rc;

/// Records the clip rect of the `Ui` every hook is called with.
#[derive(Default)]
struct ClipRecorder {
    clips: RefCell<Vec<Rect>>,
}

impl NodeTemplate for ClipRecorder {
    fn node_ui(&self, ui: &mut Ui, _ctx: NodeContext) {
        self.clips.borrow_mut().push(ui.painter().clip_rect());
    }
    fn selection_ui(&self, ui: &mut Ui, _ctx: SelectionContext) {
        self.clips.borrow_mut().push(ui.painter().clip_rect());
    }
    fn notification_ui(&self, ui: &mut Ui, _ctx: NotificationContext) -> bool {
        self.clips.borrow_mut().push(ui.painter().clip_rect());
        true
    }
    fn marker_ui(&self, ui: &mut Ui, _ctx: MarkerContext) {
        self.clips.borrow_mut().push(ui.painter().clip_rect());
    }
}

#[test]
fn templates_paint_inside_the_map_area_only() {
    let template = Rc::new(ClipRecorder::default());
    let mut map = Map::new();
    // One node in the middle and one far off to the side, partly outside.
    map.add_points(vec![
        MapPoint::new(1, [0.0, 0.0]),
        MapPoint::new(2, [180.0, 0.0]),
    ]);
    map.set_node_template(template.clone());
    map.update_marker(7, 2);

    let ctx = Context::default();
    let screen = Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400.0, 300.0));
    let mut widget_rect = Rect::NOTHING;
    let mut out = ctx.run_ui(
        RawInput {
            screen_rect: Some(screen),
            ..RawInput::default()
        },
        |ui| {
            // Leave room around the widget, like a map inside a tab.
            ui.add_space(20.0);
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                ui.allocate_ui(egui::vec2(200.0, 150.0), |ui| {
                    widget_rect = ui.add(&mut map).rect;
                });
            });
        },
    );
    out.textures_delta.clear();

    let clips = template.clips.borrow();
    assert!(!clips.is_empty(), "the template must have been called");
    for clip in clips.iter() {
        assert!(
            widget_rect.contains_rect(*clip),
            "template clip {clip:?} must stay inside the map widget {widget_rect:?}"
        );
    }
}

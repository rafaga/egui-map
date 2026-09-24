//! Verifies `NodeContext::marker`: the template sees how present a marker
//! (`Map::update_marker`) is on each node, fading in when one arrives and out
//! when the last one leaves, so it can draw markers as part of the node.

use egui::{Context, RawInput, Ui};
use egui_map::map::Map;
use egui_map::map::objects::{
    MARKER_FADE_SECS, MapPoint, MarkerContext, NodeContext, NodeTemplate, NotificationContext,
    SelectionContext,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

/// Records the last `NodeContext::marker` each node was painted with.
#[derive(Default)]
struct RecordingTemplate {
    markers: RefCell<HashMap<usize, f32>>,
}

impl NodeTemplate for RecordingTemplate {
    fn node_ui(&self, _ui: &mut Ui, ctx: NodeContext) {
        self.markers.borrow_mut().insert(ctx.point.id, ctx.marker);
    }
    fn selection_ui(&self, _ui: &mut Ui, _ctx: SelectionContext) {}
    fn notification_ui(&self, _ui: &mut Ui, _ctx: NotificationContext) -> bool {
        false
    }
    fn marker_ui(&self, _ui: &mut Ui, _ctx: MarkerContext) {}
}

fn render_once(map: &mut Map) {
    let ctx = Context::default();
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400.0, 300.0));
    let mut out = ctx.run_ui(
        RawInput {
            screen_rect: Some(screen),
            ..RawInput::default()
        },
        |ui| {
            ui.add(&mut *map);
        },
    );
    out.textures_delta.clear();
}

fn wait_for_fade() {
    std::thread::sleep(Duration::from_secs_f32(MARKER_FADE_SECS) + Duration::from_millis(50));
}

#[test]
fn node_ui_sees_markers_fade_in_and_out() {
    let template = Rc::new(RecordingTemplate::default());
    let mut map = Map::new();
    map.add_points(vec![
        MapPoint::new(1, [0.0, 0.0]),
        MapPoint::new(2, [50.0, 0.0]),
    ]);
    map.set_node_template(template.clone());

    render_once(&mut map);
    assert_eq!(template.markers.borrow()[&1], 0.0);

    map.update_marker(7, 1);
    render_once(&mut map);
    let arriving = template.markers.borrow()[&1];
    assert!(
        (0.0..1.0).contains(&arriving),
        "still fading in: {arriving}"
    );

    wait_for_fade();
    render_once(&mut map);
    assert_eq!(template.markers.borrow()[&1], 1.0);
    assert_eq!(template.markers.borrow()[&2], 0.0);

    map.remove_marker(7);
    wait_for_fade();
    render_once(&mut map);
    assert_eq!(template.markers.borrow()[&1], 0.0);
}

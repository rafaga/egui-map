//! Verifies that `NodeTemplate::notification_ui`/`marker_ui` are handed the
//! correct `NodeAnimation`/`SteadyAnimation` `kind` and `node_id` -- the
//! capability added in egui-map 0.5.0 (via `NotificationContext`/
//! `MarkerContext`) so a template can dispatch straight to the matching
//! built-in `Animation::*` function instead of reimplementing the lookup, or
//! resorting to a workaround like matching on `color` (as earlier,
//! pre-0.5.0 versions of `examples/node_template_animations.rs` had to).

use egui::{Context, RawInput, Ui};
use egui_map::map::Map;
use egui_map::map::objects::{
    MapPoint, MarkerContext, NodeAnimation, NodeContext, NodeTemplate, NotificationContext,
    SelectionContext, SteadyAnimation,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

/// Records every `notification_ui`/`marker_ui` call it receives, so tests can
/// assert on exactly what reached the hook rather than on what got painted.
#[derive(Default)]
struct RecordingTemplate {
    notifications: RefCell<Vec<(NodeAnimation, usize)>>,
    markers: RefCell<Vec<(SteadyAnimation, usize)>>,
}

impl NodeTemplate for RecordingTemplate {
    fn node_ui(&self, _ui: &mut Ui, _ctx: NodeContext) {}
    fn selection_ui(&self, _ui: &mut Ui, _ctx: SelectionContext) {}

    fn notification_ui(&self, ui: &mut Ui, ctx: NotificationContext) -> bool {
        self.notifications
            .borrow_mut()
            .push((ctx.kind, ctx.node_id));
        ui.ctx().request_repaint();
        true
    }

    fn marker_ui(&self, ui: &mut Ui, ctx: MarkerContext) {
        self.markers.borrow_mut().push((ctx.kind, ctx.node_id));
        ui.ctx().request_repaint();
    }
}

fn map_with_two_nodes() -> Map {
    let mut map = Map::new();
    map.add_points(vec![
        MapPoint::new(1, [0.0, 0.0]),
        MapPoint::new(2, [50.0, 0.0]),
    ]);
    map
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
    // `TexturesDelta` panics on drop if left unhandled -- see the fix applied
    // to the crate's own `render_line_segments` test helper.
    out.textures_delta.clear();
}

#[test]
fn notification_ui_receives_the_requested_kind_and_node_id() {
    let template = Rc::new(RecordingTemplate::default());
    let mut map = map_with_two_nodes();
    map.set_node_template(template.clone());

    map.node(1).unwrap().ripple(Instant::now());
    map.node(2).unwrap().crosshair(Instant::now());
    render_once(&mut map);

    let mut seen = template.notifications.borrow().clone();
    seen.sort_by_key(|(_, id)| *id);
    assert_eq!(
        seen,
        vec![(NodeAnimation::Ripple, 1), (NodeAnimation::Crosshair, 2)],
        "the template must be told exactly which NodeAnimation each node requested"
    );
}

#[test]
fn marker_ui_receives_the_requested_kind_for_persistent_state() {
    let template = Rc::new(RecordingTemplate::default());
    let mut map = map_with_two_nodes();
    map.set_node_template(template.clone());

    map.node(1).unwrap().halo();
    map.node(2).unwrap().orbit();
    render_once(&mut map);

    let mut seen = template.markers.borrow().clone();
    seen.sort_by_key(|(_, id)| *id);
    assert_eq!(
        seen,
        vec![(SteadyAnimation::Halo, 1), (SteadyAnimation::Orbit, 2)],
        "the template must be told exactly which SteadyAnimation each node's state requested"
    );
}

#[test]
fn marker_ui_receives_the_shared_marker_animation_kind_for_update_marker() {
    // Plain markers (`Map::update_marker`) don't carry their own animation
    // choice -- they all share `MapSettings::marker_animation` -- so `kind`
    // here must be that global setting, not a per-marker one.
    let template = Rc::new(RecordingTemplate::default());
    let mut map = map_with_two_nodes();
    map.set_node_template(template.clone());
    map.settings.marker_animation = SteadyAnimation::Orbit;
    map.update_marker(100, 2);

    render_once(&mut map);

    assert_eq!(
        *template.markers.borrow(),
        vec![(SteadyAnimation::Orbit, 2)],
        "a Map::update_marker marker must report MapSettings::marker_animation as its kind, \
         and the id of the node it points at"
    );
}

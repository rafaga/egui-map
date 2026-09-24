//! Verifies lasting notifications (`NodeHandle::lasting`): they reach the
//! template with `until` set and a color that fades as they age, and are
//! dropped once `until` passes -- and that a template returning `false` ends
//! a notification.

use egui::{Color32, Context, Pos2, RawInput, Rect, Ui, vec2};
use egui_map::map::Map;
use egui_map::map::objects::{
    MapPoint, MarkerContext, NodeContext, NodeTemplate, NotificationContext, SelectionContext,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

struct Template {
    calls: RefCell<Vec<(Option<Instant>, Color32)>>,
    running: Cell<bool>,
}

impl NodeTemplate for Template {
    fn node_ui(&self, _ui: &mut Ui, _ctx: NodeContext) {}
    fn selection_ui(&self, _ui: &mut Ui, _ctx: SelectionContext) {}
    fn notification_ui(&self, _ui: &mut Ui, ctx: NotificationContext) -> bool {
        self.calls.borrow_mut().push((ctx.until, ctx.color));
        self.running.get()
    }
    fn marker_ui(&self, _ui: &mut Ui, _ctx: MarkerContext) {}
}

fn setup(running: bool) -> (Map, Rc<Template>) {
    let template = Rc::new(Template {
        calls: RefCell::new(Vec::new()),
        running: Cell::new(running),
    });
    let mut map = Map::new();
    map.add_points(vec![MapPoint::new(1, [0.0, 0.0])]);
    map.set_node_template(template.clone());
    (map, template)
}

fn frame(map: &mut Map) {
    let ctx = Context::default();
    let mut out = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 300.0))),
            ..RawInput::default()
        },
        |ui| {
            ui.add(&mut *map);
        },
    );
    out.textures_delta.clear();
}

#[test]
fn a_lasting_notification_fades_and_ends() {
    let (mut map, template) = setup(true);
    let now = Instant::now();
    // Triggered 30 s ago, lasting a minute: halfway through.
    let started = now - Duration::from_secs(30);
    map.node(1)
        .unwrap()
        .color(Color32::from_rgb(200, 0, 0))
        .lasting(Duration::from_secs(60))
        .pulse(started);
    frame(&mut map);
    {
        let calls = template.calls.borrow();
        assert_eq!(calls.len(), 1);
        let (until, color) = calls[0];
        assert_eq!(until, Some(started + Duration::from_secs(60)));
        // Roughly half faded.
        assert!((100..=160).contains(&color.a()), "alpha {}", color.a());
    }

    // Past its end: dropped without being drawn.
    let (mut map, template) = setup(true);
    map.node(1)
        .unwrap()
        .lasting(Duration::from_secs(60))
        .pulse(now - Duration::from_secs(61));
    frame(&mut map);
    frame(&mut map);
    assert!(template.calls.borrow().is_empty());
}

#[test]
fn a_template_returning_false_ends_the_notification() {
    let (mut map, template) = setup(false);
    map.node(1).unwrap().pulse(Instant::now());
    frame(&mut map);
    frame(&mut map);
    assert_eq!(template.calls.borrow().len(), 1);
}

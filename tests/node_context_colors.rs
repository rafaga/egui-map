//! Verifies that `NodeTemplate::node_ui` receives both the active theme's
//! node color (`NodeContext::theme_color`) and the resolved/computed one
//! (`NodeContext::color`) -- with the computed one honoring a per-node
//! `MapPoint::color` override while `theme_color` stays the theme's own base
//! color. Mirrors `tests/segment_template_context.rs` for the segment side.

use egui::{Color32, Context, RawInput, Ui};
use egui_map::map::Map;
use egui_map::map::objects::{
    MapPoint, MarkerContext, NodeContext, NodeTemplate, NotificationContext, SelectionContext,
};
use egui_map::map::theme::{ColorMode, MapTheme, ThemeColors};
use std::cell::RefCell;
use std::rc::Rc;

/// A fixed palette so tests can assert exact colors instead of depending on
/// the default theme's light/dark variant.
struct FixedPalette;

impl MapTheme for FixedPalette {
    fn colors(&self, _mode: ColorMode) -> ThemeColors {
        ThemeColors {
            node: Color32::from_rgb(1, 2, 3),
            segment: Color32::from_rgb(4, 5, 6),
            selected: Color32::from_rgb(7, 8, 9),
            alert: Color32::from_rgb(10, 11, 12),
            text: Color32::from_rgb(13, 14, 15),
        }
    }
}

/// Records every `node_ui` call it receives, so tests can assert on exactly
/// what reached the hook rather than on what got painted.
#[derive(Default)]
struct RecordingTemplate {
    nodes: RefCell<Vec<(usize, Color32, Color32)>>,
}

impl NodeTemplate for RecordingTemplate {
    fn node_ui(&self, _ui: &mut Ui, ctx: NodeContext) {
        self.nodes
            .borrow_mut()
            .push((ctx.point.id, ctx.color, ctx.theme_color));
    }

    fn selection_ui(&self, _ui: &mut Ui, _ctx: SelectionContext) {}

    fn notification_ui(&self, _ui: &mut Ui, _ctx: NotificationContext) -> bool {
        false
    }

    fn marker_ui(&self, _ui: &mut Ui, _ctx: MarkerContext) {}
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
fn node_ui_sees_theme_color_and_computed_color_without_override() {
    let template = Rc::new(RecordingTemplate::default());
    let mut map = map_with_two_nodes();
    map.set_theme(Rc::new(FixedPalette));
    map.set_node_template(template.clone());

    render_once(&mut map);

    let nodes = template.nodes.borrow();
    assert_eq!(nodes.len(), 2);
    for (_, color, theme_color) in nodes.iter() {
        // With no per-node override the computed color falls back to the
        // theme's own node color -- but the template still receives both
        // values explicitly.
        assert_eq!(*color, Color32::from_rgb(1, 2, 3));
        assert_eq!(*theme_color, Color32::from_rgb(1, 2, 3));
    }
}

#[test]
fn node_override_reaches_the_computed_color_but_not_theme_color() {
    let template = Rc::new(RecordingTemplate::default());
    let mut map = Map::new();
    map.set_theme(Rc::new(FixedPalette));
    let mut point = MapPoint::new(1, [0.0, 0.0]);
    point.color = Some(Color32::from_rgb(200, 100, 50));
    map.add_points(vec![point]);
    map.set_node_template(template.clone());

    render_once(&mut map);

    let nodes = template.nodes.borrow();
    let (id, color, theme_color) = nodes[0];
    assert_eq!(id, 1);
    assert_eq!(
        color,
        Color32::from_rgb(200, 100, 50),
        "NodeContext::color must honor the node's override"
    );
    assert_eq!(
        theme_color,
        Color32::from_rgb(1, 2, 3),
        "NodeContext::theme_color must stay the theme's color, unaffected by the override"
    );
}

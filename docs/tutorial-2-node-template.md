# Tutorial 2: a map with a `NodeTemplate`

[README](../README.md) · Previous: [Tutorial 1: your first map](tutorial-1-first-map.md)

The nodes of [tutorial 1](tutorial-1-first-map.md) are dots with the name next to them. A `NodeTemplate` lets you draw them yourself -- here, a rounded box with the name inside. The data is loaded exactly as before: a template changes how the nodes look, not how they are built.

**1. Write the template.** It is a type of your own, which can hold the settings your nodes need. The size is what a box measures at the standard zoom; every method multiplies it by the zoom.

```rust
use egui::{Align2, CornerRadius, FontId, Pos2, Rect, Stroke, StrokeKind, Ui, Vec2};
use egui_map::map::objects::{HitContext, NodeContext, NodeOutline, NodeTemplate};

struct BoxNodes {
    size: Vec2,
    corner_radius: f32,
}

impl BoxNodes {
    // The box of a node centred at `position`. `node_ui` paints it and
    // `outline` declares it, so they cannot disagree.
    fn rect(&self, position: Pos2, zoom: f32) -> Rect {
        Rect::from_center_size(position, self.size * zoom)
    }
}
```

**2. Paint the node** in `node_ui`, the only method you have to write. `ctx.position` is the node's centre on the screen, already placed and scaled, and `ctx.zoom` the scale to apply to every size. The colors come from the active theme, so the nodes follow it when it changes: `ctx.color` is the node's own color (or the theme's node color) and `ctx.theme` the whole palette. Once a template is installed the widget no longer writes the names: this is where you draw them.

```rust
impl NodeTemplate for BoxNodes {
    fn node_ui(&self, ui: &mut Ui, ctx: NodeContext) {
        let rect = self.rect(ctx.position, ctx.zoom);
        let radius = CornerRadius::same((self.corner_radius * ctx.zoom) as u8);
        let painter = ui.painter();
        painter.rect_filled(rect, radius, ctx.theme.background);
        painter.rect_stroke(
            rect,
            radius,
            Stroke::new(1.5 * ctx.zoom, ctx.color),
            StrokeKind::Inside,
        );
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            ctx.point.get_name(),
            FontId::proportional(11.0 * ctx.zoom),
            ctx.theme.text,
        );
    }
```

**3. Say what shape the node has**, in `outline`. It is optional but worth three lines: from it the widget derives where the pointer hits the node (so `Map::hovered_node` and clicks work), the selection ring and the shape the notification effects wrap around.

```rust
    fn outline(&self, ctx: HitContext) -> NodeOutline {
        NodeOutline::RoundedRect {
            rect: self.rect(ctx.position, ctx.zoom),
            corner_radius: self.corner_radius * ctx.zoom,
        }
    }
}
```

**4. Install it** on the map, behind an `Rc`, and show the map as before.

```rust
map.set_node_template(std::rc::Rc::new(BoxNodes {
    size: Vec2::new(72.0, 22.0),
    corner_radius: 6.0,
}));
```

[`examples/tutorial_node_template.rs`](../examples/tutorial_node_template.rs) is this tutorial with the same five nodes as the first one; run it with `cargo run --example tutorial_node_template`. [`examples/custom_template.rs`](../examples/custom_template.rs) and [`examples/node_template_animations.rs`](../examples/node_template_animations.rs) take it further, with notifications and markers.

## Where to go from here

- The README's [Custom node rendering and animations](../README.md#custom-node-rendering-and-animations) section covers the rest of `NodeTemplate`: selection, notifications and markers.
- [Custom segment rendering and animations](../README.md#custom-segment-rendering-and-animations) is the same idea for the lines.

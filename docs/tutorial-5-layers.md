# Tutorial 5: layers

[README](../README.md) · Previous: [Tutorial 4: effects](tutorial-4-effects.md) · Next: [Tutorial 6: themes and region labels](tutorial-6-themes-and-labels.md)

A **layer** draws something of your own -- an ownership overlay, a heat map, a highlight -- at a place of the map's paint order that you choose. The map keeps no data of a layer: it all lives in your type.

**1. Write the layer.** It implements `LayerTemplate`; `layer_ui` is called once per frame with a `Painter` clipped to the map. `ctx.points` are the nodes inside the viewport, each with its `position` on the *screen*, so you only multiply sizes by `ctx.zoom`.

```rust
use egui::Painter;
use egui_map::map::layer::{LayerContext, LayerTemplate};

struct Discs {
    radius: std::rc::Rc<std::cell::Cell<f32>>, // in map units
}

impl LayerTemplate for Discs {
    fn layer_ui(&self, painter: &Painter, ctx: LayerContext) {
        for point in ctx.points {
            painter.circle_filled(
                point.position,
                self.radius.get() * ctx.zoom,
                ctx.theme.node.gamma_multiply(0.2),
            );
        }
    }
}
```

**2. Follow the shape of the node.** Each `LayerPoint` carries the `outline` of its node: the shape its `NodeTemplate` reports, or a circle when there is none. `grown(SELECTION_GAP)` leaves the same gap the built-in selection does, and `stroke_shape` turns the outline into something you can paint.

```rust
use egui::Stroke;
use egui_map::map::objects::SELECTION_GAP;

painter.add(
    point.outline.grown(SELECTION_GAP).stroke_shape(Stroke::new(2.0, ctx.theme.selected)),
);
```

**3. Install the layers.** The slot says where they go: `BelowLines`, `BelowNodes` or `AboveNodes`. Layers of one slot are painted by ascending `z`, and with an equal `z` in the order they were added. Adding a layer with an id that exists replaces it.

```rust
use egui_map::map::layer::LayerSlot;

map.add_layer("discs", LayerSlot::BelowLines, 0, std::rc::Rc::new(discs));
map.add_layer("ring", LayerSlot::AboveNodes, 0, std::rc::Rc::new(ring));
```

**4. Share the data.** The application changes a layer's data through whatever you shared with it (an `Rc<Cell<..>>` above), and `set_layer_visible` hides a layer without removing it. The ring needs to know the hovered node: set it after showing the map.

```rust
map.set_layer_visible("discs", false);

ui.add(&mut map);
hovered.set(map.hovered_node());
```

[`examples/tutorial_layers.rs`](../examples/tutorial_layers.rs) is this tutorial; run it with `cargo run --example tutorial_layers`. [`examples/layers.rs`](../examples/layers.rs) adds a heat map built as one `Mesh` per frame, and a layer painted *inside* a node's box through `LayerSlot::OverNodeBody` (a `NodeTemplate` places it with `ctx.layers.paint(..)`).

## Where to go from here

- [Tutorial 6](tutorial-6-themes-and-labels.md) changes the colors of the map and names its areas.

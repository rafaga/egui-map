# Tutorial 1: your first map

[README](../README.md) · Next: [Tutorial 2: a map with a `NodeTemplate`](tutorial-2-node-template.md)

A map is an egui widget. You build it once, keep it between frames and add it to a `Ui` on every frame. Building it takes three steps.

**1. Create the nodes.** A node is a `MapPoint`: an id of your choice (unique) and a position in *map coordinates*, in whatever units you like -- the widget scales them with the zoom. The name is optional.

```rust
use egui_map::map::objects::MapPoint;

let mut alpha = MapPoint::new(1, [0.0, 0.0]);
alpha.set_name("Alpha".to_string());
let mut beta = MapPoint::new(2, [120.0, 40.0]);
beta.set_name("Beta".to_string());
let mut points = vec![alpha, beta];
```

**2. Connect them.** Every line has an id of its own -- by convention the pair of node ids it joins -- and that id has to be listed in the `connections` of **both** of its nodes. The line itself, a `MapSegment`, carries the id and the coordinates of its two ends.

```rust
use egui_map::map::objects::MapSegment;

let line_id = (1, 2);
points[0].connections.push(line_id);
points[1].connections.push(line_id);
let segments = vec![MapSegment::new(line_id, [0.0, 0.0], [120.0, 40.0])];
```

**3. Load them and show the map.** Load the nodes first, then the lines.

```rust
use egui_map::map::Map;

let mut map = Map::new();
map.add_points(points);
map.add_lines(segments);

// Then, on every frame of your egui update loop, with the same `map`:
// ui.add(&mut map);
```

Pan with click & drag, zoom with the mouse wheel or the slider. [`examples/tutorial_map.rs`](../examples/tutorial_map.rs) is this tutorial with five nodes, four lines and `eframe`; run it with `cargo run --example tutorial_map`.

## Where to go from here

- [Tutorial 2](tutorial-2-node-template.md) draws the nodes yourself with a `NodeTemplate`.
- The README's [Configuring the map](../README.md#configuring-the-map) section lists the settings people reach for first: zoom limits, when the lines show, how thick they are, node names, the minimap and the slider.
- Every line in this tutorial is straight. [Straight and curved lines](../README.md#straight-and-curved-lines) shows how to make one a curve.

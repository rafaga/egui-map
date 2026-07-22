# egui-map

An [`egui`](https://github.com/emilk/egui) widget that renders an interactive 2D map and displays information about it.

## Features

- Pan with click & drag, and zoom with the mouse wheel (hold `Ctrl` — or `Cmd` on macOS — to zoom faster) or the built-in slider.
- Spatial indexing via kd-tree: only the nodes inside the viewport are painted each frame.
- Node names with configurable visibility rules (always / on hover / hidden).
- Connection lines between nodes and free-floating text labels.
- Pulsing notification effects and blinking markers attached to nodes.
- Custom node rendering and right-click context menus through the `NodeTemplate` and `ContextMenuManager` traits.
- Built-in light and dark themes, customizable through `MapSettings`.

## Usage

Add the dependency:

```toml
[dependencies]
egui-map = "0.0"
```

Feed the map a set of nodes and add it to your UI:

```rust
use egui_map::map::Map;
use egui_map::map::objects::{MapPoint, RawPoint};
use std::collections::HashMap;

// Build the node set, keyed by node id.
let mut points: HashMap<usize, MapPoint> = HashMap::new();
points.insert(1, MapPoint::new(1, RawPoint::new(0.0, 0.0)));
points.insert(2, MapPoint::new(2, RawPoint::new(100.0, 50.0)));

let mut map = Map::new();
map.add_hashmap_points(points);

// Then, on every frame of your egui update loop:
// ui.add(&mut map);
```

### Connecting nodes with lines

Lines are wired in three steps: create the nodes, register a unique connection id in the `connections` of **both** endpoints, and load the line geometry keyed by that same id:

```rust
use egui_map::map::objects::{MapLine, MapPoint, RawPoint};
use std::collections::HashMap;

let mut points: HashMap<usize, MapPoint> = HashMap::new();
points.insert(1, MapPoint::new(1, RawPoint::new(0.0, 0.0)));
points.insert(2, MapPoint::new(2, RawPoint::new(10.0, 10.0)));

// Register the connection id on both endpoints.
for id in [1, 2] {
    points.get_mut(&id).unwrap().connections.push("1-2".to_string());
}
map.add_hashmap_points(points);

// Line geometry, keyed by the same connection id.
let mut lines: HashMap<String, MapLine> = HashMap::new();
lines.insert("1-2".to_string(), MapLine::new(RawPoint::new(0.0, 0.0), RawPoint::new(10.0, 10.0)));
map.add_lines(lines);
```

A line is only drawn while the zoom level is above `MapSettings::line_visible_zoom` and at least one of its endpoints is inside the viewport.

### Custom node rendering and animations

Implement `NodeTemplate` to take over how nodes, selection highlights, notifications and markers are drawn — including the name labels, which the widget no longer paints once a template is installed:

```rust
use egui_map::map::objects::{MapPoint, NodeTemplate};
use egui::{Color32, Pos2, Ui};
use std::time::Instant;

struct MyTemplate;

impl NodeTemplate for MyTemplate {
    fn node_ui(&self, ui: &mut Ui, position: Pos2, zoom: f32, point: &MapPoint) {
        // `point` is the node's screen position; scale every size by `zoom`.
        ui.painter().circle_filled(position, 6.0 * zoom, Color32::GOLD);
    }

    fn notification_ui(&self, ui: &mut Ui, position: Pos2, zoom: f32, start: Instant, color: Color32) -> bool {
        // ... draw a time-driven effect computed from `start.elapsed()` ...
        ui.ctx().request_repaint(); // keep the animation frames coming
        start.elapsed().as_secs_f32() < 2.0 // returning false removes the notification
    }

    fn selection_ui(&self, _ui: &mut Ui, _position: Pos2, _zoom: f32) {}
    fn marker_ui(&self, _ui: &mut Ui, _position: Pos2, _zoom: f32) {}
}

map.set_node_template(std::rc::Rc::new(MyTemplate));
```

See the `NodeTemplate` rustdoc for a complete example with a custom node shape and an animated notification.

## Crate features

- `puffin`: instruments the widget's hot paths with the [`puffin`](https://crates.io/crates/puffin) profiler.

## License

MIT. See [LICENSE.md](LICENSE.md).

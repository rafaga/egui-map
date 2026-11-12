# Tutorial 3: changing the map while it is on screen

[README](../README.md) · Previous: [Tutorial 2: a map with a `NodeTemplate`](tutorial-2-node-template.md) · Next: [Tutorial 4: effects](tutorial-4-effects.md)

Tutorials 1 and 2 load the nodes once. Most real maps change: a system appears, a device goes away. `add_points` and `add_lines` replace the whole set and center the view again, so they are the wrong tool for that. `insert_point`, `remove_point`, `insert_line` and `remove_line` change one thing and leave the view where the user put it.

**1. Keep a copy of your nodes.** The map takes nodes in but does not hand them back, and a line has to be listed in the `connections` of *both* of its nodes, so to join a new node to an old one you need the old one in your hand.

```rust
use egui_map::map::Map;
use egui_map::map::objects::MapPoint;
use std::collections::BTreeMap;

struct Network {
    map: Map,
    nodes: BTreeMap<usize, MapPoint>,
    next_id: usize,
}
```

**2. Add a node and the line that joins it.** `insert_point` also *replaces* a node with the same id, which is how the old node gets its new connection.

```rust
let line_id = (previous.get_id(), id);
let segment = MapSegment::new(line_id, previous.coords, point.coords);
previous.connections.push(line_id);
point.connections.push(line_id);
self.map.insert_point(previous.clone());
self.map.insert_line(segment);
self.map.insert_point(point.clone());
```

**3. Remove a node.** `remove_point` returns the node and takes everything that hung from it: the lines in its `connections` (and the reference to them in the other nodes of the map), its markers, its effects and its notifications. Your own copy is yours to tidy.

```rust
if let Some(removed) = self.map.remove_point(id) {
    self.nodes.remove(&id);
    for neighbour in self.nodes.values_mut() {
        neighbour.connections.retain(|line| !removed.connections.contains(line));
    }
}
```

**4. Ask the response for clicks.** `ui.add(&mut map)` returns a normal egui `Response`, so `clicked()` and `dragged()` work. `Map::hovered_node()` says which node the pointer is over, if any.

```rust
let response = ui.add(&mut network.map);
if response.clicked()
    && let Some(id) = network.map.hovered_node()
{
    network.remove_node(id);
}
```

[`examples/tutorial_dynamic.rs`](../examples/tutorial_dynamic.rs) is this tutorial with an "Add node" button; run it with `cargo run --example tutorial_dynamic`. [`examples/dynamic_nodes.rs`](../examples/dynamic_nodes.rs) is the larger version: about thirty nodes in rings and bridges that grow and shrink by themselves.

## Where to go from here

- [Tutorial 4](tutorial-4-effects.md) makes a node or a line draw attention to itself.
- `Map::contains_point` tells whether an id is on the map; `remove_line` removes a line alone.

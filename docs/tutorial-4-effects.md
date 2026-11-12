# Tutorial 4: animations on nodes and lines

[README](../README.md) · Previous: [Tutorial 3: changing the map while it is on screen](tutorial-3-dynamic-nodes.md) · Next: [Tutorial 5: layers](tutorial-5-layers.md)

A node or a line can carry an animation. They come in two kinds: **one-off** effects play from the `Instant` you give and end on their own, **lasting** effects run until you clear them. The map keeps repainting while one is running, so you never ask egui for frames yourself.

You get a handle with `map.node(id)` or `map.segment(line_id)`; both return an `Option`, because the id may not be on the map. The handle is used once: optional modifiers first (`color(..)`, `lasting(..)`, and `direction(..)` for lines), then one terminal method.

**1. A one-off effect on a node.** `pulse` is an expanding, fading disc: "something happened here". Its siblings are `ripple`, `countdown`, `scale_in` and `crosshair`.

```rust
use std::time::Instant;

if let Some(node) = map.node(3) {
    node.pulse(Instant::now());
}
```

`color(..)` changes the color (the default is the theme's alert color), and `lasting(duration)` makes a one-off effect repeat for that long, its color fading so a recent notification stands out from an old one.

```rust
use egui::Color32;
use std::time::Duration;

if let Some(node) = map.node(3) {
    node.color(Color32::ORANGE).lasting(Duration::from_secs(30)).ripple(Instant::now());
}
```

**2. A lasting effect.** `halo`, `blink` and `orbit` run until `clear()`, which removes both the notification and the lasting state of the node.

```rust
if let Some(node) = map.node(1) {
    node.halo();
}
// later
if let Some(node) = map.node(1) {
    node.clear();
}
```

**3. The same idea on lines.** `flash`, `comet_once` and `wipe` play once; `comet`, `dash`, `glow_band` and `chevrons` are lasting. `direction(CometDirection::Reverse)` runs an effect from the second end of the line to the first.

```rust
if let Some(line) = map.segment((1, 2)) {
    line.comet_once(Instant::now());
}
```

**4. Trigger them from the response.** A click on a node, as in [tutorial 3](tutorial-3-dynamic-nodes.md):

```rust
let response = ui.add(&mut map);
if response.clicked()
    && let Some(id) = map.hovered_node()
    && let Some(node) = map.node(id)
{
    node.pulse(Instant::now());
}
```

[`examples/tutorial_effects.rs`](../examples/tutorial_effects.rs) is this tutorial on a ring of six nodes; run it with `cargo run --example tutorial_effects`. [`examples/animations.rs`](../examples/animations.rs) is a visual catalog of every built-in effect.

## Where to go from here

- With a `NodeTemplate` installed you draw the effects yourself: see [Custom node rendering and animations](../README.md#custom-node-rendering-and-animations).
- [Tutorial 5](tutorial-5-layers.md) draws things the map does not know about.

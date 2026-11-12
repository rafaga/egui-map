# Tutorial 6: themes and region labels

[README](../README.md) · Previous: [Tutorial 5: layers](tutorial-5-layers.md)

**1. Name an area.** A `RegionLabel` is for an area of the map, not for a node. Its center is in *map* coordinates, like a node's. Unlike every other text the widget draws, it grows and shrinks with the zoom, and it is painted behind everything else in a faded color. `color: None` uses the theme's text color.

```rust
use egui_map::map::objects::RegionLabel;

map.add_region_labels(vec![RegionLabel {
    text: "West".to_string(),
    center: egui::pos2(-160.0, 0.0),
    color: None,
}]);
```

`add_region_labels` replaces the whole set. The font comes from `map.settings.style.region_label_font`.

**2. Change the colors.** The palette is a [`Theme`](../THEMES.md) -- there are fifteen, each with a light and a dark variant -- and `set_theme` swaps it at any time. The new colors apply on the next frame, in whichever mode egui is in.

```rust
use egui_map::map::theme::Theme;

map.set_theme(std::rc::Rc::new(Theme::ArticCyan));
```

For your own palette, implement `MapTheme`; the README's [Custom themes](../README.md#custom-themes) section shows how.

**3. Turn on the rest of the interface.** The minimap and the other conveniences are settings:

```rust
map.settings.minimap.enabled = true;
```

[`examples/tutorial_themes.rs`](../examples/tutorial_themes.rs) is this tutorial with a theme picker; run it with `cargo run --example tutorial_themes`.

## Where to go from here

- [THEMES.md](../THEMES.md) shows every built-in palette.
- The README's [Region labels](../README.md#region-labels) section covers `LabelTemplate`, which takes over how they are drawn.

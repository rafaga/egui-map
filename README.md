# egui-map

An [`egui`](https://github.com/emilk/egui) widget that renders an interactive 2D map and displays information about it.

## Features

- Pan with click & drag, and zoom with the mouse wheel (hold `Ctrl` — or `Cmd` on macOS — to zoom faster) or the built-in slider.
- Spatial indexing via kd-tree: only the nodes inside the viewport are painted each frame.
- Node names with configurable visibility rules (always / on hover / hidden).
- Connection lines between nodes and free-floating text labels.
- Text is sized in **screen pixels** (`MapSettings::node_text_size`, `MapSettings::label_text_size`), so names stay readable at any zoom level instead of shrinking away as you zoom out.
- Animations attached per node through `map.node(id)`: one-off events that end on their own (`pulse`, `ripple`, `countdown`, `scale_in`, `crosshair`) and lasting state that runs until `clear()` (`halo`, `blink`, `orbit`), each with an optional `color()`. The effects live in `map::animation::Animation` and can be reused from your own `NodeTemplate`.
- The same idiom for segments through `map.segment(id)`: `flash` / `comet_once(at, direction)` / `wipe` (one-off) and `comet` / `dash` / `glow_band` / `chevrons` (lasting, until `clear()`) -- `comet_once` is a single dot pass with the direction you choose (`CometDirection::Forward`/`Reverse`), `wipe` draws the line in from one endpoint to the other, `dash` is a "marching ants" pattern and `chevrons` a row of sliding arrowheads, both painted as a repeating-texture mesh (two triangles per segment, one shared texture), `glow_band` a soft travelling highlight that fades out past each end instead of repeating, also with an optional `color()`.
- Custom node rendering and right-click context menus through the `NodeTemplate` and `ContextMenuManager` traits, and custom segment rendering through `SegmentTemplate`.
- [Fourteen built-in color themes](THEMES.md), each with a light and a dark variant, or install your own through the `MapTheme` trait.

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
use egui_map::map::objects::{MapPoint, RawLine, RawPoint};
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
let mut lines: HashMap<String, RawLine> = HashMap::new();
lines.insert("1-2".to_string(), RawLine::new(RawPoint::new(0.0, 0.0), RawPoint::new(10.0, 10.0)));
map.add_lines(lines);
```

A line is only drawn while the zoom level is above `MapSettings::line_visible_zoom` and its bounding box intersects the viewport. Segments are culled broad-phase with an R-tree, so long lines crossing the view are drawn even when both endpoints lie outside of it.

### Custom node rendering and animations

Implement `NodeTemplate` to take over how nodes, selection highlights, notifications and markers are drawn — including the name labels, which the widget no longer paints once a template is installed:

```rust
use egui_map::map::objects::{MapPoint, MarkerContext, NodeTemplate, NotificationContext};
use egui::{Color32, Pos2, Ui};

struct MyTemplate;

impl NodeTemplate for MyTemplate {
    fn node_ui(&self, ui: &mut Ui, position: Pos2, zoom: f32, point: &MapPoint) {
        // `point` is the node's screen position; scale every size by `zoom`.
        ui.painter().circle_filled(position, 6.0 * zoom, Color32::GOLD);
    }

    fn notification_ui(&self, ui: &mut Ui, ctx: NotificationContext) -> bool {
        // `ctx.kind` is which built-in event was requested (Pulse, Ripple, ...) and
        // `ctx.node_id` is which node -- dispatch on either, or reuse
        // `animation::Animation::*`. Draw a time-driven effect from `ctx.initial_time`.
        ui.ctx().request_repaint(); // keep the animation frames coming
        ctx.initial_time.elapsed().as_secs_f32() < 2.0 // returning false removes the notification
    }

    fn selection_ui(&self, _ui: &mut Ui, _position: Pos2, _zoom: f32) {}
    fn marker_ui(&self, _ui: &mut Ui, _ctx: MarkerContext) {
        // `ctx.kind` is Halo/Blink/Orbit for persistent node state, or the shared
        // `MapSettings::marker_animation` for a `Map::update_marker` marker.
    }
}

map.set_node_template(std::rc::Rc::new(MyTemplate));
```

See the `NodeTemplate` rustdoc for a complete example with a custom node shape and an animated notification.

### Custom segment rendering and animations

`SegmentTemplate` is the segment counterpart of `NodeTemplate`. Its methods take a bare `&Painter` rather than `&mut Ui`, since segments are visited in bulk after the R-tree viewport culling — use `painter.ctx()` to reach `request_repaint()`:

```rust
use egui_map::map::objects::{MapSegment, SegmentTemplate};
use egui::{Color32, Painter, Pos2, Stroke};
use std::time::Instant;

struct MySegments;

impl SegmentTemplate for MySegments {
    fn segment_ui(&self, painter: &Painter, a: Pos2, b: Pos2, zoom: f32, _segment: &MapSegment) {
        painter.line_segment([a, b], Stroke::new(1.5 * zoom, Color32::GRAY));
    }

    fn segment_notification_ui(&self, painter: &Painter, a: Pos2, b: Pos2, zoom: f32, start: Instant, color: Color32) -> bool {
        // ... draw a time-driven effect computed from `start.elapsed()` ...
        painter.ctx().request_repaint(); // keep the animation frames coming
        start.elapsed().as_secs_f32() < 1.0 // returning false removes the notification
    }

    fn segment_state_ui(&self, painter: &Painter, a: Pos2, b: Pos2, zoom: f32, time: f32, color: Color32) {
        // `time` is the frame time (`ui.input(|i| i.time)`), shared by every element animated this frame.
        painter.ctx().request_repaint();
    }
}

map.set_segment_template(std::rc::Rc::new(MySegments));
```

`examples/animations.rs` shows the built-in node and segment effects end to end, with no custom template at all. `examples/node_template_animations.rs` shows the opposite pairing: a custom `NodeTemplate` (its own node shape) that still reuses the built-in `Animation::*` functions from its `notification_ui`/`marker_ui` hooks instead of hand-rolling new ones, dispatching directly on the `kind`/`node_id` those hooks receive.

### Custom themes

The widget ships fourteen named [`Theme`](https://docs.rs/egui-map/latest/egui_map/map/theme/enum.Theme.html) palettes — `NebulaViolet` is the default — each with a light and a dark variant; see the `Theme` rustdoc for the full list. Switch between them, or install your own palette, with `Map::set_theme` and the `MapTheme` trait:

```rust
use egui_map::map::theme::{ColorMode, MapTheme, Theme, ThemeColors};

// A built-in theme:
map.set_theme(std::rc::Rc::new(Theme::ArticCyan));

// Or a custom palette:
struct HighContrast;

impl MapTheme for HighContrast {
    fn colors(&self, mode: ColorMode) -> ThemeColors {
        match mode {
            ColorMode::Light => ThemeColors {
                node: egui::Color32::BLACK,
                segment: egui::Color32::DARK_GRAY,
                selected: egui::Color32::RED,
                alert: egui::Color32::RED,
                text: egui::Color32::BLACK,
            },
            ColorMode::Dark => ThemeColors {
                node: egui::Color32::WHITE,
                segment: egui::Color32::LIGHT_GRAY,
                selected: egui::Color32::YELLOW,
                alert: egui::Color32::YELLOW,
                text: egui::Color32::WHITE,
            },
        }
    }
}

map.set_theme(std::rc::Rc::new(HighContrast));
```

`ColorMode` follows `egui`'s own light/dark mode, so the same map picks up the right palette automatically when the surrounding app's mode changes. Non-palette visual settings (stroke widths, font, background) stay on `MapSettings::styles` — see the `theme` module rustdoc.

## Crate features

- `debug_overlay`: adds a read-out of the widget's internal viewport state (bounds, current position, distance, zoom, node counts, pointer position). It stays out of the way: a dim `dbg` toggle in the map's top-left corner, collapsed by default and with no background of its own, that you click open when you need the numbers. egui remembers the open/closed state per widget, and the overlay never affects the map's layout.

## Profiling

The widget's hot paths (rendering, viewport culling, point/line loading) are instrumented with [`tracing`](https://docs.rs/tracing) spans. `tracing` is a normal, unconditional dependency of this crate, and the spans are cheap no-ops unless a subscriber is installed somewhere in your binary -- `egui-map` never installs one itself.

To see these spans in the [Tracy](https://github.com/wolfpld/tracy) profiler, install a `tracing_tracy::TracyLayer` in your own `main`, e.g.:

```rust
tracing_subscriber::registry()
    .with(tracing_tracy::TracyLayer::default())
    .init();
```

The `profile` feature pulls in `tracing-subscriber` and `tracing-tracy` so `examples/tracy_profile.rs` can demonstrate exactly this. Run it (with a Tracy capture window already listening) with:

```sh
cargo run --example tracy_profile --features profile
```

## License

MIT. See [LICENSE.md](LICENSE.md).

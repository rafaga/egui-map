# egui-map

An [`egui`](https://github.com/emilk/egui) widget that renders an interactive 2D map and displays information about it.

## Features

- Pan with click & drag, and zoom with the mouse wheel (hold `Ctrl` — or `Cmd` on macOS — to zoom faster) or the built-in slider.
- Spatial indexing via kd-tree: only the nodes inside the viewport are painted each frame.
- Node names with configurable visibility rules (always / on hover / hidden).
- Connection lines between nodes and free-floating text labels.
- Text is sized in **screen pixels** (`MapSettings::node_text_size`, `MapSettings::label_text_size`), so names stay readable at any zoom level instead of shrinking away as you zoom out.
- Region labels (`RegionLabel`, via `Map::add_region_labels`) for naming an *area* of the map rather than a node: unlike every other text the widget draws, their size scales *with* zoom, they are always painted first (behind everything else) and in a faded color, with the built-in renderer caching laid-out text for performance. Customizable through the `LabelTemplate` trait.
- Animations attached per node through `map.node(id)`: one-off events that end on their own (`pulse`, `ripple`, `countdown`, `scale_in`, `crosshair`) and lasting state that runs until `clear()` (`halo`, `blink`, `orbit`), each with an optional `color()`. The effects live in `map::animation::Animation` and can be reused from your own `NodeTemplate`.
- The same idiom for segments through `map.segment(id)`: `flash` / `comet_once(at, direction)` / `wipe` (one-off) and `comet` / `dash` / `glow_band` / `chevrons` (lasting, until `clear()`) -- `comet_once` is a single dot pass, and every effect but `flash` runs the way you choose with `segment.direction(CometDirection::Forward / Reverse)` before it, `wipe` draws the line in from one endpoint to the other and then wipes it out the same way, `dash` is a "marching ants" pattern and `chevrons` a row of sliding arrowheads, both painted as a repeating-texture mesh (two triangles per segment, one shared texture), `glow_band` a soft travelling highlight that fades out past each end instead of repeating, also with an optional `color()`.
- Custom node rendering and right-click context menus through the `NodeTemplate` and `ContextMenuManager` traits, custom segment rendering through `SegmentTemplate`, and custom region-label rendering through `LabelTemplate`.
- [Fifteen built-in color themes](THEMES.md), each with a light and a dark variant -- `SystemDefault`, matching plain egui's own colors, is the default -- or install your own through the `MapTheme` trait.

## Usage

Add the dependency:

```toml
[dependencies]
egui-map = "0.0"
```

The two tutorials below are complete programs that compile and run: `cargo run --example tutorial_map` and `cargo run --example tutorial_node_template`. The code in each step is the code of the example.

### Tutorial 1: your first map

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

Pan with click & drag, zoom with the mouse wheel or the slider. `examples/tutorial_map.rs` is this tutorial with five nodes, four lines and `eframe`.

### Configuring the map

Everything you can tune is a public field of `map.settings`. The defaults are a good start; these are the ones people reach for first.

```rust
use egui_map::map::objects::{LineFade, VisibilitySetting};
use egui_map::map::overlay::Corner;

// Zoom: the limits of the mouse wheel and the slider. 1.0 is 100%.
map.settings.min_zoom = 0.1;
map.settings.max_zoom = 4.0;

// Lines: when they show and how they fade in. By default they are hidden up
// to a zoom of 0.2 (20%) and fade in over the next 0.8, opaque from 100%.
map.settings.line_visible_zoom = 0.2;
map.settings.line_fade = LineFade::Zoom { range: 0.8 };

// A shorter ramp: opaque already at 50% (0.2 + 0.3).
map.settings.line_fade = LineFade::Zoom { range: 0.3 };

// No fade and no threshold: lines are opaque and drawn at every zoom.
map.settings.line_fade = LineFade::None;

// Lines: how thick. The width at the standard zoom (100%), in pixels; the
// line is drawn that wide times the zoom (4 px at 200%, 1 px at 50%).
// `None` turns the built-in line off, for a `SegmentTemplate` to draw them.
map.settings.style.line_width = Some(2.0);

// Names: `Always` (needs `label_visible_zoom`), `Hover` or `Hidden`.
map.settings.node_text_visibility = VisibilitySetting::Hover;
map.settings.label_visible_zoom = 0.58;
map.settings.node_text_size = 12.0; // screen pixels, whatever the zoom

// Minimap (off by default) and zoom slider (on by default), each pinned to
// the corner of its own `anchor`.
map.settings.minimap.enabled = true;
map.settings.minimap.scale = 0.3; // fraction of the map's width and height
map.settings.minimap.anchor = Corner::BottomRight;
map.settings.slider.anchor = Corner::TopRight;
map.settings.slider.ticks = true;
```

Colors are not in the settings: they come from the active theme (see *Custom themes* below).

A line is only drawn while its bounding box intersects the viewport, and -- unless `line_fade` is `LineFade::None` -- while the zoom is above `line_visible_zoom`. Segments are culled broad-phase with an R-tree, so long lines crossing the view are drawn even when both endpoints lie outside of it.

### Tutorial 2: a map with a `NodeTemplate`

The nodes of tutorial 1 are dots with the name next to them. A `NodeTemplate` lets you draw them yourself -- here, a rounded box with the name inside. The data is loaded exactly as before: a template changes how the nodes look, not how they are built.

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

`examples/tutorial_node_template.rs` is this tutorial with the same five nodes as the first one. `examples/custom_template.rs` and `examples/node_template_animations.rs` take it further, with notifications and markers.

### Custom node rendering and animations

Implement `NodeTemplate` to take over how nodes, selection highlights, notifications and markers are drawn — including the name labels, which the widget no longer paints once a template is installed:

```rust
use egui_map::map::objects::{
    MarkerContext, NodeContext, NodeTemplate, NotificationContext, SelectionContext,
};
use egui::Ui;

struct MyTemplate;

impl NodeTemplate for MyTemplate {
    fn node_ui(&self, ui: &mut Ui, ctx: NodeContext) {
        // `ctx.position` is the node's screen position; scale every size by `ctx.zoom`.
        // `ctx.color` is already resolved: the node's own color override, or the
        // active theme's node color if it doesn't have one.
        ui.painter().circle_filled(ctx.position, 6.0 * ctx.zoom, ctx.color);
    }

    fn notification_ui(&self, ui: &mut Ui, ctx: NotificationContext) -> bool {
        // `ctx.kind` is which built-in event was requested (Pulse, Ripple, ...) and
        // `ctx.node_id` is which node -- dispatch on either, or reuse
        // `animation::Animation::*`. Draw a time-driven effect from `ctx.initial_time`.
        ui.ctx().request_repaint(); // keep the animation frames coming
        ctx.initial_time.elapsed().as_secs_f32() < 2.0 // returning false removes the notification
    }

    fn selection_ui(&self, _ui: &mut Ui, _ctx: SelectionContext) {
        // `ctx.point` is which node the highlight belongs to; `ctx.color` is the
        // active theme's selection color.
    }
    fn marker_ui(&self, _ui: &mut Ui, _ctx: MarkerContext) {
        // `ctx.kind` is Halo/Blink/Orbit for persistent node state, or the shared
        // `MapSettings::marker_animation` for a `Map::update_marker` marker.
        // `ctx.color` is the color the built-in effect would paint with: the
        // node's own override, or the active theme's `marker` color -- its own
        // role, distinct from `selected`/`alert`, for "this is flagged"
        // indefinitely rather than a one-off event.
    }
}

map.set_node_template(std::rc::Rc::new(MyTemplate));
```

`NodeContext` also carries `marker: f32` — how present a `Map::update_marker` marker is on this node, from `0.0` (none) to `1.0` (fully in), fading in and out on its own instead of switching instantly (see `MARKER_FADE_SECS` in the `objects` module). Paired with `Animation::glow` (a tint that breathes in and out over a rounded rectangle, scaled by that strength), it lets a box-shaped `NodeTemplate` paint markers as part of the node itself — between its background and its label — instead of in `marker_ui`, which draws over every node and so over its label too:

```rust
# use egui_map::map::animation::Animation;
# use egui::{CornerRadius, Rect, Ui, Vec2};
# use egui_map::map::objects::NodeContext;
# fn node_ui(ui: &mut Ui, ctx: NodeContext) {
if ctx.marker > 0.0 {
    let time = ui.input(|i| i.time) as f32;
    let rect = Rect::from_center_size(ctx.position, Vec2::splat(30.0 * ctx.zoom));
    Animation::glow(
        ui.painter(),
        rect,
        CornerRadius::same((10.0 * ctx.zoom) as u8),
        time,
        ctx.theme.marker.gamma_multiply(0.6),
        ctx.marker,
    );
    ui.ctx().request_repaint();
}
# }
```

See the `NodeTemplate` rustdoc for a complete example with a custom node shape and an animated notification, and `examples/node_template_animations.rs` for this glow paired with the ring `marker_ui` already draws.

### Custom segment rendering and animations

`SegmentTemplate` is the segment counterpart of `NodeTemplate`. Its methods take a bare `&Painter` rather than `&mut Ui`, since segments are visited in bulk after the R-tree viewport culling — use `painter.ctx()` to reach `request_repaint()`. Like `NodeTemplate`, every method takes a `#[non_exhaustive]` context struct — `SegmentContext`, `SegmentNotificationContext`, `SegmentStateContext` — each carrying the segment's endpoints (`pos_a`/`pos_b`), `zoom`, the `segment` itself (id and coordinates) and a resolved `color`; the two effect contexts also carry `kind`, so you can match it and dispatch straight to the built-in `Animation::*` function instead of reimplementing the effect:

```rust
use egui_map::map::animation::Animation;
use egui_map::map::objects::{
    SegmentContext, SegmentNotificationContext, SegmentStateContext, SegmentTemplate,
    SteadySegmentAnimation,
};
use egui::{Color32, Painter, Stroke};

struct MySegments;

impl SegmentTemplate for MySegments {
    fn segment_ui(&self, painter: &Painter, ctx: SegmentContext) {
        painter.line_segment([ctx.pos_a, ctx.pos_b], Stroke::new(1.5 * ctx.zoom, Color32::GRAY));
    }

    fn segment_notification_ui(&self, painter: &Painter, ctx: SegmentNotificationContext) -> bool {
        // `ctx.kind` is which of flash/comet_once/wipe was requested -- dispatch
        // on it, or draw a time-driven effect computed from `ctx.initial_time.elapsed()`.
        painter.ctx().request_repaint(); // keep the animation frames coming
        ctx.initial_time.elapsed().as_secs_f32() < 1.0 // returning false removes the notification
    }

    fn segment_state_ui(&self, painter: &Painter, ctx: SegmentStateContext) {
        // `ctx.kind` is which persistent effect was requested -- reuse the
        // matching built-in one instead of reimplementing it:
        let effect = match ctx.kind {
            SteadySegmentAnimation::Comet => Animation::comet,
            SteadySegmentAnimation::Dash => Animation::dash,
            SteadySegmentAnimation::GlowBand => Animation::glow_band,
            SteadySegmentAnimation::Chevrons => Animation::chevrons,
        };
        effect(painter, ctx.pos_a, ctx.pos_b, ctx.zoom, ctx.time, ctx.color);
        painter.ctx().request_repaint();
    }
}

map.set_segment_template(std::rc::Rc::new(MySegments));
```

`examples/animations.rs` is a visual catalog of the built-in node and segment effects, with no custom template at all. `examples/custom_template.rs` is the shortest custom `NodeTemplate`: it paints the node and declares its outline, and the selection, notifications and marker are the defaults drawn along that outline. `examples/node_template_animations.rs` does the same with hexagonal nodes, a glow and a segment with marching ants.

### Region labels

`RegionLabel` names an area of the map rather than a single node — think "Domain" or "Nullsec", not a station name. It is deliberately the opposite of every other text the widget draws: its font size (see `Style::region_label_font` below) scales *with* the current zoom instead of staying a fixed screen size, it is always painted first so every node, line and free-floating `MapLabel` draws over it, and its default color is the active theme's text color faded by `MapSettings::region_label_alpha` (`0.50` by default) so it reads as a backdrop instead of competing for attention. The built-in renderer caches the laid-out text (`Arc<Galley>`, keyed by text and rounded size) and only re-applies color at paint time, so a region label that hasn't changed costs a cache lookup rather than a full relayout every frame:

```rust
use egui_map::map::Map;
use egui_map::map::objects::RegionLabel;

let mut map = Map::new();
map.add_region_labels(vec![RegionLabel {
    text: "Domain".to_string(),
    center: egui::pos2(300.0, 200.0), // map coordinates, like MapPoint::coords
    color: None, // default: the active theme's text color, faded by `region_label_alpha`
}]);
```

`examples/svg_template.rs` shows one in a small running example.

Install a `LabelTemplate` with `Map::set_label_template` to take over the drawing entirely — see its rustdoc for the `LabelContext` fields (`position`, `zoom`, `label`, `size` already scaled by zoom, `color` already faded, and the full theme `ThemeColors` palette).

The built-in renderer's font comes from `Style::region_label_font` — a `FontId`, for symmetry with `Style::font` — set on the single `Style` shared by both light and dark mode, `MapSettings::style`. Unlike `Style::font`, it is mandatory rather than optional: every `Style` must pick an explicit typeface and base size, there is no built-in fallback. `family` picks the typeface and `size` is the base size, still scaled by the current zoom:

```rust
use egui::{FontFamily, FontId};

map.settings.style.region_label_font = FontId::new(30.0, FontFamily::Monospace);
```

### Custom themes

The widget ships fifteen named [`Theme`](https://docs.rs/egui-map/latest/egui_map/map/theme/enum.Theme.html) palettes — `SystemDefault`, which carries over egui's own default colors so an unthemed map looks like plain egui, is the default — each with a light and a dark variant; see the `Theme` rustdoc for the full list. `examples/basic.rs` lists them in a combo box and switches the map live, next to egui's own light/dark/system buttons. Switch between them, or install your own palette, with `Map::set_theme` and the `MapTheme` trait:

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
                marker: egui::Color32::BLUE,
                text: egui::Color32::BLACK,
                background: egui::Color32::WHITE,
            },
            ColorMode::Dark => ThemeColors {
                node: egui::Color32::WHITE,
                segment: egui::Color32::LIGHT_GRAY,
                selected: egui::Color32::YELLOW,
                alert: egui::Color32::YELLOW,
                marker: egui::Color32::LIGHT_BLUE,
                text: egui::Color32::WHITE,
                background: egui::Color32::BLACK,
            },
        }
    }
}

map.set_theme(std::rc::Rc::new(HighContrast));
```

`ColorMode` is `egui::Theme` re-exported under this crate's name, so the same map picks up the right palette automatically when the surrounding app's mode changes. Non-palette visual settings (stroke width, font) stay on `MapSettings::style` — see the `theme` module rustdoc.

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

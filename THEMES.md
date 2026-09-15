# egui-map -- built-in themes

`egui-map` ships 15 named color palettes (`map::theme::Theme`), each with a `Light` and a `Dark` variant (`map::theme::ColorMode`, a re-export of `egui::Theme`). `Theme::colors(mode)` resolves a theme to the six colors the widget actually paints with (`map::theme::ThemeColors`): the node fill, connection lines (`segment`), the selection ring around the nearest node (`selected`), one-off notification/alert animations (`alert`), a lasting "this is marked" indicator -- a node's persistent state or a plain `update_marker` marker (`marker`) -- and node names/labels (`text`).

`EguiDefault` is the odd one out and the default theme: instead of a hand-picked palette, it carries over egui's own default `Visuals` colors (`hyperlink_color`, the separator-line color, `selection.stroke`, `warn_fg_color`, `error_fg_color`, and the active-widget text color, `strong_text_color()`), so a map with no theme installed looks like plain egui rather than an arbitrary house style.

Install a built-in theme, or your own palette, with `Map::set_theme` and the `MapTheme` trait -- see the README's "Custom themes" section and the `MapTheme` rustdoc for the full API.

![Preview of every built-in theme, light and dark](theme_gallery.png)

*Preview generated from the exact `Theme::colors` values below -- each card mocks the shapes the widget paints (nodes, connection lines, a selection ring, an alert ring, a marker ring) plus a node name label in the theme's actual `text` color, rather than being a captured screenshot of a running app.*

## `EguiDefault`

egui's own default `Visuals` colors (light and dark), carried over as a `Theme` instead of invented -- so a map with no theme installed looks like plain egui, not like an arbitrary house palette. The default theme.

```rust
map.set_theme(std::rc::Rc::new(egui_map::map::theme::Theme::EguiDefault));
```

| Mode | node | segment | selected | alert | marker | text |
|---|---|---|---|---|---|---|
| Light | `#009BFF` | `#BEBEBE` | `#00537D` | `#FF6400` | `#FF0000` | `#000000` |
| Dark | `#5AAAFF` | `#3C3C3C` | `#C0DEFF` | `#FF8F00` | `#FF0000` | `#FFFFFF` |

## `SlateOcean`

Muted blues over slate grays.

```rust
map.set_theme(std::rc::Rc::new(egui_map::map::theme::Theme::SlateOcean));
```

| Mode | node | segment | selected | alert | marker | text |
|---|---|---|---|---|---|---|
| Light | `#2E4C6D` | `#8A99A8` | `#0EA5E9` | `#E0592A` | `#40BF1D` | `#2B333D` |
| Dark | `#6E93BF` | `#4A5A6E` | `#38BDF8` | `#FF8A4C` | `#D43DF2` | `#D7DEE6` |

## `NebulaViolet`

Violet and teal on a soft neutral backdrop.

```rust
map.set_theme(std::rc::Rc::new(egui_map::map::theme::Theme::NebulaViolet));
```

| Mode | node | segment | selected | alert | marker | text |
|---|---|---|---|---|---|---|
| Light | `#5B3E96` | `#B3A4D6` | `#16B8A6` | `#E6337A` | `#97BF1D` | `#372F45` |
| Dark | `#A78BFA` | `#5C4A80` | `#2DD4BF` | `#FF5FA3` | `#CBF23D` | `#E4DCF2` |

## `TerminalGreen`

Greens and blues reminiscent of a terminal color scheme.

```rust
map.set_theme(std::rc::Rc::new(egui_map::map::theme::Theme::TerminalGreen));
```

| Mode | node | segment | selected | alert | marker | text |
|---|---|---|---|---|---|---|
| Light | `#1F7A3D` | `#9BB89E` | `#2563EB` | `#D6A429` | `#BF1D9F` | `#2A332C` |
| Dark | `#4ADE80` | `#3A5240` | `#60A5FA` | `#FFC94D` | `#F23DDD` | `#D7E6DA` |

## `EmberForge`

Warm oranges and pinks over charcoal/cream.

```rust
map.set_theme(std::rc::Rc::new(egui_map::map::theme::Theme::EmberForge));
```

| Mode | node | segment | selected | alert | marker | text |
|---|---|---|---|---|---|---|
| Light | `#8C4A1F` | `#C9A98C` | `#C4258C` | `#2E86AB` | `#8CBF1D` | `#362E28` |
| Dark | `#D97F3D` | `#5A4632` | `#F472B6` | `#4FC3E8` | `#B0F23D` | `#EDE0D3` |

## `SolarAmber`

Amber and teal on a warm neutral backdrop.

```rust
map.set_theme(std::rc::Rc::new(egui_map::map::theme::Theme::SolarAmber));
```

| Mode | node | segment | selected | alert | marker | text |
|---|---|---|---|---|---|---|
| Light | `#8A6D1E` | `#D8C48F` | `#2D6E5E` | `#C1442A` | `#631DBF` | `#362E1C` |
| Dark | `#F0C24C` | `#5A4E2E` | `#4FBF9E` | `#E8654A` | `#913DF2` | `#EFE4C4` |

## `ArticCyan`

Cyan and gold over deep blue-gray.

```rust
map.set_theme(std::rc::Rc::new(egui_map::map::theme::Theme::ArticCyan));
```

| Mode | node | segment | selected | alert | marker | text |
|---|---|---|---|---|---|---|
| Light | `#12708A` | `#9AC6D1` | `#F5A524` | `#E0527A` | `#1D9CBF` | `#1F2E30` |
| Dark | `#4FE0FF` | `#375E68` | `#FBBF24` | `#FF6B95` | `#3DD4F2` | `#D3EAEF` |

## `CrimsonSignal`

Signal red and steel blue over near-black.

```rust
map.set_theme(std::rc::Rc::new(egui_map::map::theme::Theme::CrimsonSignal));
```

| Mode | node | segment | selected | alert | marker | text |
|---|---|---|---|---|---|---|
| Light | `#7A1F2B` | `#B9A8A8` | `#1F6FEB` | `#E8A628` | `#BF1DAA` | `#332628` |
| Dark | `#E35B6B` | `#5C4548` | `#58A6FF` | `#FFC459` | `#F23DE3` | `#E8D6D8` |

## `MidnightIndigo`

Indigo and teal on deep midnight blue.

```rust
map.set_theme(std::rc::Rc::new(egui_map::map::theme::Theme::MidnightIndigo));
```

| Mode | node | segment | selected | alert | marker | text |
|---|---|---|---|---|---|---|
| Light | `#2B2F77` | `#A6A9C9` | `#00B8A9` | `#E0A400` | `#A41DBF` | `#262940` |
| Dark | `#7B82E0` | `#3A3D66` | `#2DD4C8` | `#FFD166` | `#D13DF2` | `#D8DAF0` |

## `CopperRose`

Copper and teal over warm taupe.

```rust
map.set_theme(std::rc::Rc::new(egui_map::map::theme::Theme::CopperRose));
```

| Mode | node | segment | selected | alert | marker | text |
|---|---|---|---|---|---|---|
| Light | `#9C4A3C` | `#D9B8AE` | `#2F7A6B` | `#E0A23C` | `#911DBF` | `#3D2E29` |
| Dark | `#E08B6F` | `#5E453D` | `#4FC3AE` | `#FFC65C` | `#C23DF2` | `#EFDAD0` |

## `LimeCircuit`

Lime green and violet over dark olive.

```rust
map.set_theme(std::rc::Rc::new(egui_map::map::theme::Theme::LimeCircuit));
```

| Mode | node | segment | selected | alert | marker | text |
|---|---|---|---|---|---|---|
| Light | `#4D7A1F` | `#B9C79A` | `#7B3FE4` | `#E85D2E` | `#1DBF4D` | `#2E3320` |
| Dark | `#A8E05F` | `#445230` | `#A78BFA` | `#FF8552` | `#3DF26D` | `#DCEAC0` |

## `CoralReef`

Coral and ocean blue over sea-glass teal.

```rust
map.set_theme(std::rc::Rc::new(egui_map::map::theme::Theme::CoralReef));
```

| Mode | node | segment | selected | alert | marker | text |
|---|---|---|---|---|---|---|
| Light | `#D65A45` | `#A8D4CE` | `#1D5C9E` | `#F2A93C` | `#BF1DB7` | `#33403E` |
| Dark | `#FF8B73` | `#386560` | `#5CA8E0` | `#FFC15E` | `#F23DEF` | `#D6EDE8` |

## `GraphiteMono`

Grayscale with a cool blue accent.

```rust
map.set_theme(std::rc::Rc::new(egui_map::map::theme::Theme::GraphiteMono));
```

| Mode | node | segment | selected | alert | marker | text |
|---|---|---|---|---|---|---|
| Light | `#3A3A3A` | `#B8B8B4` | `#1F8FE0` | `#E0483A` | `#45BF1D` | `#232323` |
| Dark | `#D6D6D2` | `#4A4A46` | `#4FB3F5` | `#FF6B5C` | `#6AF23D` | `#E8E8E4` |

## `PlumStatic`

Plum and jade over muted mauve.

```rust
map.set_theme(std::rc::Rc::new(egui_map::map::theme::Theme::PlumStatic));
```

| Mode | node | segment | selected | alert | marker | text |
|---|---|---|---|---|---|---|
| Light | `#6B3B5E` | `#C7AEC0` | `#2E8B6E` | `#E0793D` | `#731DBF` | `#362B33` |
| Dark | `#C994BB` | `#4A3A45` | `#52C99A` | `#FFA05C` | `#9A3DF2` | `#EBD9E5` |

## `SandstoneTrail`

Sand and clay over warm khaki.

```rust
map.set_theme(std::rc::Rc::new(egui_map::map::theme::Theme::SandstoneTrail));
```

| Mode | node | segment | selected | alert | marker | text |
|---|---|---|---|---|---|---|
| Light | `#6B5A3A` | `#DCCBA0` | `#2A6E8C` | `#D1495B` | `#60BF1D` | `#3A3121` |
| Dark | `#C9AD72` | `#4E4530` | `#4FA8CC` | `#F0708A` | `#8EF23D` | `#E6D9B8` |

---

`EguiDefault` is the default theme (`Theme::default()`). The gallery image and the tables above are generated together, straight from `src/map/theme.rs`, by `scripts/generate_theme_gallery` (a standalone Rust tool -- run it with `cargo run --manifest-path scripts/generate_theme_gallery/Cargo.toml` from the repo root) -- if the palettes there ever change, rerun it rather than hand-editing this file or the PNG.

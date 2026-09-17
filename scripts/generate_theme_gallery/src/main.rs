//! Regenerates one standalone SVG preview per built-in theme (under
//! `theme_gallery/`) and the per-theme sections of `THEMES.md`, straight
//! from `src/map/theme.rs`'s actual `Theme::colors` values, so neither can
//! drift from the real palettes the way it could when the gallery was a
//! one-off, uncommitted script (and the way `THEMES.md` itself did before
//! this tool existed to keep it in sync -- see the ArticCyan/SolarAmber
//! text-contrast bug this tool's addition fixed alongside it).
//!
//! Run from the repo root with `cargo run --manifest-path
//! scripts/generate_theme_gallery/Cargo.toml`. Whenever a built-in theme's
//! colors change, or a new one is added, rerun this and commit the
//! resulting `theme_gallery/*.svg` files and `THEMES.md` alongside the code
//! change.
//!
//! Each theme's preview is a self-contained `.svg` (no external font or
//! image asset to keep in sync -- text is plain SVG `<text>`, rendered by
//! whatever opens the file), one per theme rather than one combined image,
//! so it can be embedded on its own inside that theme's own `THEMES.md`
//! section and viewed individually. Each card mocks the shapes the widget
//! actually paints: two nodes joined by a segment-colored line, a third
//! node wearing the selected+alert rings, a fourth wearing the marker ring,
//! a node name rendered in the theme's real `text` color, and the card
//! itself filled with the theme's own `background` -- so a text/background
//! contrast regression would show up here directly instead of only in a
//! hex table.

use regex::Regex;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

type Color = (u8, u8, u8);

const FIELDS: [&str; 7] = [
    "node",
    "segment",
    "selected",
    "alert",
    "marker",
    "text",
    "background",
];

#[derive(Clone)]
struct ThemeData {
    name: String,
    /// Full doc comment from the enum, used in THEMES.md.
    doc: String,
    light: BTreeMap<&'static str, Color>,
    dark: BTreeMap<&'static str, Color>,
}

fn repo_root() -> PathBuf {
    // This crate lives at <repo>/scripts/generate_theme_gallery.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root should exist")
}

fn parse_theme_rs(src: &str) -> Vec<ThemeData> {
    // Normalize line endings up front: the real working tree this is meant
    // to run against checks out `theme.rs` with CRLF (Windows), while every
    // regex below is written against plain `\n` -- matching `\r?\n`
    // everywhere instead would work too, but is easy to miss in a new regex
    // added later, so it's simpler to normalize once here.
    let src = src.replace("\r\n", "\n");
    let src = src.as_str();

    // 1. Enum variant order + doc comments, in declaration order.
    let enum_re = Regex::new(r"(?s)pub enum Theme \{(.*?)\n\}").unwrap();
    let enum_block = &enum_re.captures(src).expect("Theme enum not found")[1];

    let variant_re = Regex::new(r"(?s)/// (.*?)\n(?:\s*#\[default\]\n)?\s*(\w+),").unwrap();
    let mut order = Vec::new();
    let mut docs = BTreeMap::new();
    for caps in variant_re.captures_iter(enum_block) {
        let doc_lines = &caps[1];
        let name = caps[2].to_string();
        let doc = doc_lines
            .lines()
            .map(|l| l.trim().trim_start_matches('/').trim())
            .collect::<Vec<_>>()
            .join(" ");
        order.push(name.clone());
        docs.insert(name, doc);
    }

    // 2. Every `(Theme, Mode) => ThemeColors { ... }` arm.
    let arm_re =
        Regex::new(r"(?s)\((\w+),\s*(Dark|Light)\)\s*=>\s*ThemeColors\s*\{([^}]*)\}").unwrap();
    let field_re = |field: &str| {
        Regex::new(&format!(
            r"{field}:\s*Color32::from_rgb\(0x([0-9A-Fa-f]{{1,2}}),\s*0x([0-9A-Fa-f]{{1,2}}),\s*0x([0-9A-Fa-f]{{1,2}})\)"
        ))
        .unwrap()
    };
    let field_res: BTreeMap<&str, Regex> = FIELDS.iter().map(|f| (*f, field_re(f))).collect();

    let mut light: BTreeMap<String, BTreeMap<&'static str, Color>> = BTreeMap::new();
    let mut dark: BTreeMap<String, BTreeMap<&'static str, Color>> = BTreeMap::new();

    for caps in arm_re.captures_iter(src) {
        let theme = caps[1].to_string();
        let mode = &caps[2];
        let body = &caps[3];
        let mut colors = BTreeMap::new();
        for field in FIELDS {
            let re = &field_res[field];
            let c = re
                .captures(body)
                .unwrap_or_else(|| panic!("missing field `{field}` for {theme} {mode}"));
            let r = u8::from_str_radix(&c[1], 16).unwrap();
            let g = u8::from_str_radix(&c[2], 16).unwrap();
            let b = u8::from_str_radix(&c[3], 16).unwrap();
            colors.insert(field, (r, g, b));
        }
        if mode == "Light" {
            light.insert(theme, colors);
        } else {
            dark.insert(theme, colors);
        }
    }

    order
        .into_iter()
        .map(|name| ThemeData {
            doc: docs[&name].clone(),
            light: light[&name].clone(),
            dark: dark[&name].clone(),
            name,
        })
        .collect()
}

// ---------------------------------------------------------------------
// SVG card (shared by every per-theme preview)
// ---------------------------------------------------------------------

const CARD_H: f32 = 190.0;
const CARD_W: f32 = 411.0;
const ROW_H: f32 = 30.0;
const PAD: f32 = 24.0;

/// Total width of a two-card (Light + Dark) row: `PAD` on each outer edge
/// plus `PAD` in the gutter between the two cards.
fn svg_width() -> f32 {
    PAD * 3.0 + CARD_W * 2.0
}

fn hex(c: Color) -> String {
    format!("#{:02X}{:02X}{:02X}", c.0, c.1, c.2)
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn card_short_description(name: &str, doc: &str) -> String {
    if name == "SystemDefault" {
        "Plain egui's own default colors. The default theme.".to_string()
    } else {
        doc.chars().take(90).collect()
    }
}

/// Rough advance width of `text` set at `size` px in a plain sans-serif
/// face -- there is no font metrics library involved anymore (SVG text is
/// rendered by whatever opens the file, not by this tool), so this is only
/// precise enough to keep the legend chips in a row from overlapping.
fn approx_text_width(text: &str, size: f32) -> f32 {
    text.chars().count() as f32 * size * 0.56
}

/// Builds one `Light` or `Dark` card as an SVG `<g>` fragment, positioned at
/// `(x0, y0)` -- the same geometry `draw_card` used to rasterize, just
/// emitted as SVG elements instead of pixels. The card's own background and
/// border now come from the theme's `background`/`text` colors instead of a
/// fixed light/dark gray, closing the gap the PNG gallery's own footnote
/// used to call out (the preview's background used to come from egui's
/// `Visuals`, not from `ThemeColors`).
fn build_card_svg(x0: f32, y0: f32, mode: &str, colors: &BTreeMap<&'static str, Color>) -> String {
    let dark = mode == "Dark";
    let card_bg = hex(colors["background"]);
    let border = if dark { "#3C3C3C" } else { "#DEDEE2" };
    let fg_label = if dark { "#E6E6E8" } else { "#28282C" };

    let mut s = String::new();

    // Border + card background (border drawn first as a slightly larger
    // rect showing a 1px ring around the fill, same trick `draw_card` used
    // with two nested filled rects).
    let _ = write!(
        s,
        r#"<rect x="{x0}" y="{y0}" width="{w}" height="{h}" fill="{border}"/>"#,
        w = CARD_W,
        h = CARD_H
    );
    let _ = write!(
        s,
        r#"<rect x="{x1}" y="{y1}" width="{w}" height="{h}" fill="{card_bg}"/>"#,
        x1 = x0 + 1.0,
        y1 = y0 + 1.0,
        w = CARD_W - 2.0,
        h = CARD_H - 2.0
    );

    // `RegionLabel` backdrop: the widget paints these first, behind
    // everything else, in `ThemeColors::text` faded by
    // `MapSettings::region_label_alpha` (default 0.50). Shown here bold and
    // at a fixed 50% so it reads clearly in a small preview card -- not the
    // crate's own default alpha, just enough to make the layer visible.
    let text_color = hex(colors["text"]);
    let _ = write!(
        s,
        r#"<text x="{cx}" y="{cy}" text-anchor="middle" font-family="sans-serif" font-weight="700" font-size="36" fill="{text_color}" fill-opacity="0.5">Region</text>"#,
        cx = x0 + CARD_W / 2.0,
        cy = y0 + CARD_H / 2.0 + 12.0
    );

    // Mode label, top-left of the card.
    let _ = write!(
        s,
        r#"<text x="{x}" y="{y}" font-family="sans-serif" font-size="13" fill="{fg_label}">{mode}</text>"#,
        x = x0 + 12.0,
        y = y0 + 20.0
    );

    let top = (x0 + CARD_W * 0.34, y0 + 42.0);
    let left = (x0 + CARD_W * 0.16, y0 + 82.0);
    let right = (x0 + CARD_W * 0.40, y0 + 82.0);
    let marker_node = (x0 + CARD_W * 0.60, y0 + 60.0);

    let seg = hex(colors["segment"]);
    let _ = write!(
        s,
        r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{seg}" stroke-width="2"/>"#,
        x1 = top.0,
        y1 = top.1,
        x2 = left.0,
        y2 = left.1
    );
    let _ = write!(
        s,
        r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{seg}" stroke-width="2"/>"#,
        x1 = top.0,
        y1 = top.1,
        x2 = right.0,
        y2 = right.1
    );
    // The right and marker nodes used to sit unconnected -- wire them up
    // with a `dash` animation snapshot instead of a plain line, so the
    // gallery also shows what an in-flight segment animation looks like.
    // "Marching ants" in two colors: the `segment` line draws the 6-on/5-off
    // pattern as before, and a second line in `alert` fills exactly the
    // 5-unit gaps left by the first -- its own dasharray is the complement
    // ("5,6", on-length matching the first line's gap) offset by 5 so its
    // "on" phase lands precisely where the first line is "off". Together
    // they tile the full 11-unit period with no overlap and no true gap --
    // a naive same-dasharray offset by a full period (11) is a no-op and
    // was tried first, which just left the gaps empty instead of alert-
    // colored.
    let alert_stroke = hex(colors["alert"]);
    let _ = write!(
        s,
        r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{seg}" stroke-width="2" stroke-dasharray="6,5"/>"#,
        x1 = right.0,
        y1 = right.1,
        x2 = marker_node.0,
        y2 = marker_node.1
    );
    let _ = write!(
        s,
        r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{alert_stroke}" stroke-width="2" stroke-dasharray="5,6" stroke-dashoffset="5"/>"#,
        x1 = right.0,
        y1 = right.1,
        x2 = marker_node.0,
        y2 = marker_node.1
    );

    let r: f32 = 7.0;
    let rr: f32 = r + 7.0;
    let node = hex(colors["node"]);
    let _ = write!(
        s,
        r#"<circle cx="{cx}" cy="{cy}" r="{r}" fill="{node}"/>"#,
        cx = top.0,
        cy = top.1
    );
    let _ = write!(
        s,
        r#"<circle cx="{cx}" cy="{cy}" r="{r}" fill="{node}"/>"#,
        cx = left.0,
        cy = left.1
    );

    // Right node: selection ring (outer) + alert ring (inner), same node,
    // redrawn on top with a plain filled circle last (same z-order as
    // `draw_card`).
    let selected = hex(colors["selected"]);
    let alert = hex(colors["alert"]);
    let _ = write!(
        s,
        r#"<circle cx="{cx}" cy="{cy}" r="{rr}" fill="none" stroke="{selected}" stroke-width="2"/>"#,
        cx = right.0,
        cy = right.1
    );
    let _ = write!(
        s,
        r#"<circle cx="{cx}" cy="{cy}" r="{ri}" fill="none" stroke="{alert}" stroke-width="2"/>"#,
        cx = right.0,
        cy = right.1,
        ri = r + 3.0
    );
    let _ = write!(
        s,
        r#"<circle cx="{cx}" cy="{cy}" r="{r}" fill="{node}"/>"#,
        cx = right.0,
        cy = right.1
    );

    // Separate node with a marker ring -- the "flagged" role, drawn apart
    // from the selected/alert node so it doesn't read as a third ring on
    // the same target.
    let marker = hex(colors["marker"]);
    let _ = write!(
        s,
        r#"<circle cx="{cx}" cy="{cy}" r="{rr}" fill="none" stroke="{marker}" stroke-width="3"/>"#,
        cx = marker_node.0,
        cy = marker_node.1
    );
    let _ = write!(
        s,
        r#"<circle cx="{cx}" cy="{cy}" r="{r}" fill="{node}"/>"#,
        cx = marker_node.0,
        cy = marker_node.1
    );

    // Node name label in the theme's actual `text` color, anchored under
    // the left node -- the concrete thing the ArticCyan/SolarAmber
    // contrast bug broke. Reuses `text_color` from the `RegionLabel`
    // backdrop above.
    let _ = write!(
        s,
        r#"<text x="{x}" y="{y}" font-family="sans-serif" font-size="13" fill="{text_color}">Node name</text>"#,
        x = left.0 - 24.0,
        y = left.1 + 12.0 + 10.0
    );

    // Legend chips: node / seg / sel / alert / marker / text / bg.
    let legend_y = y0 + CARD_H - 22.0;
    let chip: f32 = 10.0;
    let mut lx = x0 + 12.0;
    let legend_size: f32 = 11.0;
    let entries = [
        ("node", colors["node"]),
        ("seg", colors["segment"]),
        ("sel", colors["selected"]),
        ("alert", colors["alert"]),
        ("marker", colors["marker"]),
        ("text", colors["text"]),
        ("bg", colors["background"]),
    ];
    for (name, col) in entries {
        let _ = write!(
            s,
            r#"<rect x="{x}" y="{y}" width="{sz}" height="{sz}" fill="{fg_label}"/>"#,
            x = lx,
            y = legend_y,
            sz = chip + 1.0
        );
        let _ = write!(
            s,
            r#"<rect x="{x}" y="{y}" width="{sz}" height="{sz}" fill="{col}"/>"#,
            x = lx + 1.0,
            y = legend_y + 1.0,
            sz = chip,
            col = hex(col)
        );
        let _ = write!(
            s,
            r#"<text x="{x}" y="{y}" font-family="sans-serif" font-size="{sz}" fill="{fg_label}">{name}</text>"#,
            x = lx + chip + 3.0,
            y = legend_y + chip,
            sz = legend_size
        );
        lx += chip + 6.0 + approx_text_width(name, legend_size) + 10.0;
    }

    s
}

/// Renders one theme's complete, self-contained preview: a title, its
/// tagline, and its `Light`/`Dark` cards side by side.
fn render_theme_svg(theme: &ThemeData) -> String {
    let w = svg_width();
    let h = 56.0 + ROW_H + 8.0 + CARD_H + 12.0;

    let mut svg = String::new();
    let _ = write!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" font-family="sans-serif">"#
    );
    let _ = write!(svg, r##"<rect width="{w}" height="{h}" fill="#F5F5F7"/>"##);
    let _ = write!(
        svg,
        r##"<text x="{x}" y="24" font-size="18" font-weight="700" fill="#141418">{name}</text>"##,
        x = PAD,
        name = xml_escape(&theme.name)
    );
    let desc = card_short_description(&theme.name, &theme.doc);
    let _ = write!(
        svg,
        r##"<text x="{x}" y="44" font-size="12" fill="#6E6E74">{desc}</text>"##,
        x = PAD,
        desc = xml_escape(&desc)
    );

    let card_y = 56.0;
    svg.push_str(&build_card_svg(PAD, card_y, "Light", &theme.light));
    svg.push_str(&build_card_svg(
        PAD * 2.0 + CARD_W,
        card_y,
        "Dark",
        &theme.dark,
    ));

    svg.push_str("</svg>\n");
    svg
}

/// SVG file name for a theme's preview, relative to `THEMES.md` -- used both
/// to write the file and to embed it.
fn svg_relative_path(theme_name: &str) -> String {
    format!("theme_gallery/{theme_name}.svg")
}

fn write_theme_svgs(themes: &[ThemeData], root: &Path) {
    let out_dir = root.join("theme_gallery");
    fs::create_dir_all(&out_dir).expect("failed to create theme_gallery/");
    for theme in themes {
        let svg = render_theme_svg(theme);
        let out = out_dir.join(format!("{}.svg", theme.name));
        fs::write(&out, svg).unwrap_or_else(|e| panic!("failed to write {}: {e}", out.display()));
        println!("wrote {}", out.display());
    }
}

// ---------------------------------------------------------------------
// THEMES.md
// ---------------------------------------------------------------------

fn render_themes_md(themes: &[ThemeData], out: &Path) {
    let mut md = String::new();
    md.push_str("# egui-map -- built-in themes\n\n");
    md.push_str(&format!(
        "`egui-map` ships {} named color palettes (`map::theme::Theme`), each with a `Light` and a `Dark` variant (`map::theme::ColorMode`, a re-export of `egui::Theme`). `Theme::colors(mode)` resolves a theme to the seven colors the widget actually paints with (`map::theme::ThemeColors`): the node fill, connection lines (`segment`), the selection ring around the nearest node (`selected`), one-off notification/alert animations (`alert`), a lasting \"this is marked\" indicator -- a node's persistent state or a plain `update_marker` marker (`marker`) -- node names/labels (`text`), and the map canvas itself (`background`).\n\n",
        themes.len()
    ));
    md.push_str("`SystemDefault` is the odd one out and the default theme: instead of a hand-picked palette, it carries over egui's own default `Visuals` colors (`hyperlink_color`, the separator-line color, `selection.stroke`, `warn_fg_color`, `error_fg_color`, and the active-widget text color, `strong_text_color()`), so a map with no theme installed looks like plain egui rather than an arbitrary house style.\n\n");
    md.push_str("Install a built-in theme, or your own palette, with `Map::set_theme` and the `MapTheme` trait -- see the README's \"Custom themes\" section and the `MapTheme` rustdoc for the full API.\n\n");
    md.push_str("Each theme below has its own preview, generated straight from the `Theme::colors` values in the table under it -- each card mocks the shapes the widget paints (nodes, connection lines, a selection ring, an alert ring, a marker ring), a node name label in the theme's actual `text` color, and the card itself filled with the theme's own `background`, rather than being a captured screenshot of a running app.\n\n");

    for theme in themes {
        md.push_str(&format!("## `{}`\n\n", theme.name));
        md.push_str(&format!("{}\n\n", theme.doc));
        md.push_str(&format!(
            "![Preview of {name}, light and dark]({path})\n\n",
            name = theme.name,
            path = svg_relative_path(&theme.name)
        ));
        md.push_str("```rust\n");
        md.push_str(&format!(
            "map.set_theme(std::rc::Rc::new(egui_map::map::theme::Theme::{}));\n",
            theme.name
        ));
        md.push_str("```\n\n");
        md.push_str("| Mode | node | segment | selected | alert | marker | text | background |\n");
        md.push_str("|---|---|---|---|---|---|---|---|\n");
        md.push_str(&format!(
            "| Light | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |\n",
            hex(theme.light["node"]),
            hex(theme.light["segment"]),
            hex(theme.light["selected"]),
            hex(theme.light["alert"]),
            hex(theme.light["marker"]),
            hex(theme.light["text"]),
            hex(theme.light["background"]),
        ));
        md.push_str(&format!(
            "| Dark | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |\n\n",
            hex(theme.dark["node"]),
            hex(theme.dark["segment"]),
            hex(theme.dark["selected"]),
            hex(theme.dark["alert"]),
            hex(theme.dark["marker"]),
            hex(theme.dark["text"]),
            hex(theme.dark["background"]),
        ));
    }

    md.push_str("---\n\n");
    md.push_str("`SystemDefault` is the default theme (`Theme::default()`). Every preview above and the tables alongside them are generated together, straight from `src/map/theme.rs`, by `scripts/generate_theme_gallery` (a standalone Rust tool -- run it with `cargo run --manifest-path scripts/generate_theme_gallery/Cargo.toml` from the repo root) -- if the palettes there ever change, rerun it rather than hand-editing this file or the SVGs under `theme_gallery/`.\n");

    fs::write(out, md).expect("failed to write THEMES.md");
    println!("wrote {}", out.display());
}

fn main() {
    let root = repo_root();
    let theme_rs = fs::read_to_string(root.join("src/map/theme.rs")).expect("read theme.rs");
    let themes = parse_theme_rs(&theme_rs);
    assert!(!themes.is_empty(), "no themes parsed out of theme.rs");

    write_theme_svgs(&themes, &root);
    render_themes_md(&themes, &root.join("THEMES.md"));
}

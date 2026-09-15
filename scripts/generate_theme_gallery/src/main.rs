//! Regenerates `theme_gallery.png` and the per-theme sections of
//! `THEMES.md` straight from `src/map/theme.rs`'s actual `Theme::colors`
//! values, so neither can drift from the real palettes the way it could
//! when the gallery was a one-off, uncommitted script (and the way
//! `THEMES.md` itself did before this tool existed to keep it in sync --
//! see the ArticCyan/SolarAmber text-contrast bug this tool's addition
//! fixed alongside it).
//!
//! Run from the repo root with `cargo run --manifest-path
//! scripts/generate_theme_gallery/Cargo.toml`. Whenever a built-in theme's
//! colors change, or a new one is added, rerun this and commit the
//! resulting `theme_gallery.png` and `THEMES.md` alongside the code change.
//!
//! Needs a DejaVu Sans (regular + bold) TTF on disk -- not vendored here on
//! purpose, to keep this dev-only tool out of the repo's binary history.
//! Looked for at the usual Linux package paths; point `DEJAVU_SANS_TTF` /
//! `DEJAVU_SANS_BOLD_TTF` at your own copies if those aren't present (e.g.
//! on macOS/Windows, or a `fonts-dejavu-core` package installed elsewhere).
//!
//! Each gallery card mocks the shapes the widget actually paints: two nodes
//! joined by a segment-colored line, a third node wearing the
//! selected+alert rings, a fourth wearing the marker ring, and a node name
//! rendered in the theme's real `text` color -- so a text-contrast
//! regression would show up here directly instead of only in a hex table.

use ab_glyph::{FontRef, PxScale};
use image::{Rgb, RgbImage};
use imageproc::drawing::{
    draw_filled_circle_mut, draw_filled_rect_mut, draw_hollow_circle_mut, draw_line_segment_mut,
    draw_text_mut, text_size,
};
use imageproc::rect::Rect;
use regex::Regex;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

type Color = (u8, u8, u8);

const FIELDS: [&str; 6] = ["node", "segment", "selected", "alert", "marker", "text"];

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
// PNG gallery
// ---------------------------------------------------------------------

const W: i32 = 894;
const PAD: i32 = 24;
const CARD_H: i32 = 190;
const ROW_H: i32 = 30;
const GAP: i32 = 14;

fn card_w() -> i32 {
    (W - PAD * 3) / 2
}

fn rgb(c: Color) -> Rgb<u8> {
    Rgb([c.0, c.1, c.2])
}

fn card_short_description(name: &str, doc: &str) -> String {
    if name == "EguiDefault" {
        "Plain egui's own default colors. The default theme.".to_string()
    } else {
        doc.chars().take(90).collect()
    }
}

fn thick_hollow_circle(
    img: &mut RgbImage,
    center: (i32, i32),
    radius: i32,
    width: i32,
    color: Rgb<u8>,
) {
    for w in 0..width {
        draw_hollow_circle_mut(img, center, radius + w, color);
    }
}

fn thick_line(img: &mut RgbImage, a: (f32, f32), b: (f32, f32), width: i32, color: Rgb<u8>) {
    for w in 0..width {
        let off = w as f32;
        draw_line_segment_mut(img, (a.0, a.1 + off), (b.0, b.1 + off), color);
        draw_line_segment_mut(img, (a.0 + off, a.1), (b.0 + off, b.1), color);
    }
}

#[derive(Clone, Copy)]
struct CardArea {
    x0: i32,
    y0: i32,
    w: i32,
    h: i32,
}

fn draw_card(
    img: &mut RgbImage,
    font_reg: &FontRef,
    area: CardArea,
    mode: &str,
    colors: &BTreeMap<&'static str, Color>,
) {
    let CardArea { x0, y0, w, h } = area;
    let dark = mode == "Dark";
    let card_bg = if dark {
        (26u8, 26u8, 30u8)
    } else {
        (255, 255, 255)
    };
    let border = if dark {
        (50u8, 50u8, 55u8)
    } else {
        (222, 222, 226)
    };
    let fg_label = if dark {
        (230u8, 230u8, 232u8)
    } else {
        (40, 40, 44)
    };

    draw_filled_rect_mut(
        img,
        Rect::at(x0, y0).of_size(w as u32, h as u32),
        rgb(border),
    );
    draw_filled_rect_mut(
        img,
        Rect::at(x0 + 1, y0 + 1).of_size((w - 2) as u32, (h - 2) as u32),
        rgb(card_bg),
    );

    draw_text_mut(
        img,
        rgb(fg_label),
        x0 + 12,
        y0 + 8,
        PxScale::from(13.0),
        font_reg,
        mode,
    );

    let top = (x0 as f32 + w as f32 * 0.34, y0 as f32 + 42.0);
    let left = (x0 as f32 + w as f32 * 0.16, y0 as f32 + 82.0);
    let right = (x0 as f32 + w as f32 * 0.40, y0 as f32 + 82.0);
    let marker_node = (x0 as f32 + w as f32 * 0.60, y0 as f32 + 60.0);

    let seg = rgb(colors["segment"]);
    thick_line(img, top, left, 2, seg);
    thick_line(img, top, right, 2, seg);

    let r: i32 = 7;
    let node = rgb(colors["node"]);
    draw_filled_circle_mut(img, (top.0 as i32, top.1 as i32), r, node);
    draw_filled_circle_mut(img, (left.0 as i32, left.1 as i32), r, node);

    // Right node: selection ring (outer) + alert ring (inner), same node.
    let rr = r + 7;
    thick_hollow_circle(
        img,
        (right.0 as i32, right.1 as i32),
        rr,
        2,
        rgb(colors["selected"]),
    );
    thick_hollow_circle(
        img,
        (right.0 as i32, right.1 as i32),
        r + 3,
        2,
        rgb(colors["alert"]),
    );
    draw_filled_circle_mut(img, (right.0 as i32, right.1 as i32), r, node);

    // Separate node with a marker ring -- the "flagged" role, drawn apart
    // from the selected/alert node so it doesn't read as a third ring on
    // the same target.
    thick_hollow_circle(
        img,
        (marker_node.0 as i32, marker_node.1 as i32),
        rr,
        3,
        rgb(colors["marker"]),
    );
    draw_filled_circle_mut(img, (marker_node.0 as i32, marker_node.1 as i32), r, node);

    // Node name label in the theme's actual `text` color, anchored under
    // the left node -- the concrete thing the ArticCyan/SolarAmber
    // contrast bug broke.
    let label = "Node name";
    let tx = (left.0 - 24.0) as i32;
    let ty = (left.1 + 12.0) as i32;
    draw_text_mut(
        img,
        rgb(colors["text"]),
        tx,
        ty,
        PxScale::from(13.0),
        font_reg,
        label,
    );

    // Legend chips: node / seg / sel / alert / marker / text
    let legend_y = y0 + h - 22;
    let chip = 10;
    let mut lx = x0 + 12;
    let entries = [
        ("node", colors["node"]),
        ("seg", colors["segment"]),
        ("sel", colors["selected"]),
        ("alert", colors["alert"]),
        ("marker", colors["marker"]),
        ("text", colors["text"]),
    ];
    let legend_scale = PxScale::from(11.0);
    for (name, col) in entries {
        draw_filled_rect_mut(
            img,
            Rect::at(lx, legend_y).of_size((chip + 1) as u32, (chip + 1) as u32),
            rgb(fg_label),
        );
        draw_filled_rect_mut(
            img,
            Rect::at(lx + 1, legend_y + 1).of_size(chip as u32, chip as u32),
            rgb(col),
        );
        let (tw, _) = text_size(legend_scale, font_reg, name);
        draw_text_mut(
            img,
            rgb(fg_label),
            lx + chip + 3,
            legend_y - 1,
            legend_scale,
            font_reg,
            name,
        );
        lx += chip + 6 + tw as i32 + 10;
    }
}

fn load_font_bytes(env_var: &str, candidates: &[&str]) -> Vec<u8> {
    if let Ok(path) = std::env::var(env_var) {
        return fs::read(&path)
            .unwrap_or_else(|e| panic!("failed to read font from ${env_var}={path}: {e}"));
    }
    for path in candidates {
        if let Ok(bytes) = fs::read(path) {
            return bytes;
        }
    }
    panic!(
        "could not find a DejaVu Sans font in any of {candidates:?} -- install \
         fonts-dejavu-core, or point {env_var} at your own copy"
    );
}

fn render_gallery(themes: &[ThemeData], out: &Path) {
    let font_bold_bytes = load_font_bytes(
        "DEJAVU_SANS_BOLD_TTF",
        &[
            "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
            "/usr/share/fonts/dejavu/DejaVuSans-Bold.ttf",
        ],
    );
    let font_reg_bytes = load_font_bytes(
        "DEJAVU_SANS_TTF",
        &[
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/dejavu/DejaVuSans.ttf",
        ],
    );
    let font_bold = FontRef::try_from_slice(&font_bold_bytes).expect("bold font");
    let font_reg = FontRef::try_from_slice(&font_reg_bytes).expect("regular font");

    let n = themes.len() as i32;
    let h = 120 + n * (ROW_H + 10 + CARD_H + GAP);
    let bg = Rgb([245u8, 245, 247]);
    let mut img = RgbImage::from_pixel(W as u32, h as u32, bg);

    draw_text_mut(
        &mut img,
        Rgb([20, 20, 24]),
        PAD,
        18,
        PxScale::from(24.0),
        &font_bold,
        "egui-map -- built-in Theme gallery",
    );
    draw_text_mut(
        &mut img,
        Rgb([90, 90, 96]),
        PAD,
        46,
        PxScale::from(14.0),
        &font_reg,
        "Each palette resolves via Theme::colors(ColorMode) -- node / segment / selected / alert / marker / text.",
    );

    let mut y = 78;
    let cw = card_w();
    for theme in themes {
        draw_text_mut(
            &mut img,
            Rgb([20, 20, 24]),
            PAD,
            y,
            PxScale::from(18.0),
            &font_bold,
            &theme.name,
        );
        let desc = card_short_description(&theme.name, &theme.doc);
        draw_text_mut(
            &mut img,
            Rgb([110, 110, 116]),
            PAD,
            y + 20,
            PxScale::from(12.0),
            &font_reg,
            &desc,
        );
        let card_y = y + ROW_H + 8;
        draw_card(
            &mut img,
            &font_reg,
            CardArea {
                x0: PAD,
                y0: card_y,
                w: cw,
                h: CARD_H,
            },
            "Light",
            &theme.light,
        );
        draw_card(
            &mut img,
            &font_reg,
            CardArea {
                x0: PAD * 2 + cw,
                y0: card_y,
                w: cw,
                h: CARD_H,
            },
            "Dark",
            &theme.dark,
        );
        y = card_y + CARD_H + GAP;
    }

    let cropped = image::imageops::crop_imm(&img, 0, 0, W as u32, (y + 8) as u32).to_image();
    cropped.save(out).expect("failed to save theme_gallery.png");
    println!(
        "saved {} ({}x{})",
        out.display(),
        cropped.width(),
        cropped.height()
    );
}

// ---------------------------------------------------------------------
// THEMES.md
// ---------------------------------------------------------------------

fn hex(c: Color) -> String {
    format!("#{:02X}{:02X}{:02X}", c.0, c.1, c.2)
}

fn render_themes_md(themes: &[ThemeData], out: &Path) {
    let mut md = String::new();
    md.push_str("# egui-map -- built-in themes\n\n");
    md.push_str(&format!(
        "`egui-map` ships {} named color palettes (`map::theme::Theme`), each with a `Light` and a `Dark` variant (`map::theme::ColorMode`, a re-export of `egui::Theme`). `Theme::colors(mode)` resolves a theme to the six colors the widget actually paints with (`map::theme::ThemeColors`): the node fill, connection lines (`segment`), the selection ring around the nearest node (`selected`), one-off notification/alert animations (`alert`), a lasting \"this is marked\" indicator -- a node's persistent state or a plain `update_marker` marker (`marker`) -- and node names/labels (`text`).\n\n",
        themes.len()
    ));
    md.push_str("`EguiDefault` is the odd one out and the default theme: instead of a hand-picked palette, it carries over egui's own default `Visuals` colors (`hyperlink_color`, the separator-line color, `selection.stroke`, `warn_fg_color`, `error_fg_color`, and the active-widget text color, `strong_text_color()`), so a map with no theme installed looks like plain egui rather than an arbitrary house style.\n\n");
    md.push_str("Install a built-in theme, or your own palette, with `Map::set_theme` and the `MapTheme` trait -- see the README's \"Custom themes\" section and the `MapTheme` rustdoc for the full API.\n\n");
    md.push_str("![Preview of every built-in theme, light and dark](theme_gallery.png)\n\n");
    md.push_str("*Preview generated from the exact `Theme::colors` values below -- each card mocks the shapes the widget paints (nodes, connection lines, a selection ring, an alert ring, a marker ring) plus a node name label in the theme's actual `text` color, rather than being a captured screenshot of a running app.*\n\n");

    for theme in themes {
        md.push_str(&format!("## `{}`\n\n", theme.name));
        md.push_str(&format!("{}\n\n", theme.doc));
        md.push_str("```rust\n");
        md.push_str(&format!(
            "map.set_theme(std::rc::Rc::new(egui_map::map::theme::Theme::{}));\n",
            theme.name
        ));
        md.push_str("```\n\n");
        md.push_str("| Mode | node | segment | selected | alert | marker | text |\n");
        md.push_str("|---|---|---|---|---|---|---|\n");
        md.push_str(&format!(
            "| Light | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |\n",
            hex(theme.light["node"]),
            hex(theme.light["segment"]),
            hex(theme.light["selected"]),
            hex(theme.light["alert"]),
            hex(theme.light["marker"]),
            hex(theme.light["text"]),
        ));
        md.push_str(&format!(
            "| Dark | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |\n\n",
            hex(theme.dark["node"]),
            hex(theme.dark["segment"]),
            hex(theme.dark["selected"]),
            hex(theme.dark["alert"]),
            hex(theme.dark["marker"]),
            hex(theme.dark["text"]),
        ));
    }

    md.push_str("---\n\n");
    md.push_str("`EguiDefault` is the default theme (`Theme::default()`). The gallery image and the tables above are generated together, straight from `src/map/theme.rs`, by `scripts/generate_theme_gallery` (a standalone Rust tool -- run it with `cargo run --manifest-path scripts/generate_theme_gallery/Cargo.toml` from the repo root) -- if the palettes there ever change, rerun it rather than hand-editing this file or the PNG.\n");

    fs::write(out, md).expect("failed to write THEMES.md");
    println!("wrote {}", out.display());
}

fn main() {
    let root = repo_root();
    let theme_rs = fs::read_to_string(root.join("src/map/theme.rs")).expect("read theme.rs");
    let themes = parse_theme_rs(&theme_rs);
    assert!(!themes.is_empty(), "no themes parsed out of theme.rs");

    render_gallery(&themes, &root.join("theme_gallery.png"));
    render_themes_md(&themes, &root.join("THEMES.md"));
}

//! Color palettes and visual styling for the [`Map`](super::Map) widget.
//!
//! [`Theme`] is a built-in set of named color palettes, each with a light and
//! a dark variant ([`ColorMode`]); [`Theme::colors`] resolves a `(Theme,
//! ColorMode)` pair to the actual [`ThemeColors`] palette. [`MapTheme`] is
//! the customization point: implement it to install your own palette instead
//! of one of the built-ins, the same way [`NodeTemplate`](super::objects::NodeTemplate)
//! and [`SegmentTemplate`](super::objects::SegmentTemplate) let you replace
//! the built-in node/segment rendering. [`Style`] is the non-palette visual
//! configuration (stroke widths, font) the widget paints with; it holds no
//! *palette* color of its own -- every node/segment/selection/alert/marker/
//! text/background color comes live from the active [`MapTheme`], resolved
//! fresh each frame, so nothing in `Style` can drift out of sync with the
//! installed theme.

use crate::map::theme::{
    ColorMode::{Dark, Light},
    Theme::*,
};
use egui::FontFamily;
use egui::FontId;
use egui::ecolor::Color32;
use std::ops::{Div, Mul};

/// A built-in, named color palette, in both a light and a dark variant.
///
/// Resolve it to actual colors with [`Theme::colors`]. Install one on a
/// [`Map`](super::Map) as its active [`MapTheme`] with
/// [`Map::set_theme`](super::Map::set_theme) -- `Theme` implements
/// [`MapTheme`] directly, so any variant can be passed straight in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Theme {
    /// egui's own default `Visuals` colors (light and dark), carried over
    /// as a `Theme` instead of invented -- so a map with no theme installed
    /// looks like plain egui, not like an arbitrary house palette. The
    /// default theme.
    #[default]
    EguiDefault,
    /// Muted blues over slate grays.
    SlateOcean,
    /// Violet and teal on a soft neutral backdrop.
    NebulaViolet,
    /// Greens and blues reminiscent of a terminal color scheme.
    TerminalGreen,
    /// Warm oranges and pinks over charcoal/cream.
    EmberForge,
    /// Amber and teal on a warm neutral backdrop.
    SolarAmber,
    /// Cyan and gold over deep blue-gray.
    ArticCyan,
    /// Signal red and steel blue over near-black.
    CrimsonSignal,
    /// Indigo and teal on deep midnight blue.
    MidnightIndigo,
    /// Copper and teal over warm taupe.
    CopperRose,
    /// Lime green and violet over dark olive.
    LimeCircuit,
    /// Coral and ocean blue over sea-glass teal.
    CoralReef,
    /// Grayscale with a cool blue accent.
    GraphiteMono,
    /// Plum and jade over muted mauve.
    PlumStatic,
    /// Sand and clay over warm khaki.
    SandstoneTrail,
}

impl Theme {
    /// Resolves this theme to its actual [`ThemeColors`] for the given
    /// [`ColorMode`].
    pub const fn colors(self, mode: ColorMode) -> ThemeColors {
        match (self, mode) {
            (SlateOcean, Dark) => ThemeColors {
                node: Color32::from_rgb(0x80, 0xAD, 0xE0),
                segment: Color32::from_rgb(0x74, 0x83, 0x94),
                selected: Color32::from_rgb(0x61, 0xEB, 0xEB),
                alert: Color32::from_rgb(0xF2, 0xB0, 0x6E),
                marker: Color32::from_rgb(0xF2, 0xD1, 0xB0),
                text: Color32::from_rgb(0xDD, 0xE6, 0xF0),
                background: Color32::from_rgb(0x0B, 0x0C, 0x0D),
            },
            (SlateOcean, Light) => ThemeColors {
                node: Color32::from_rgb(0x44, 0x62, 0x85),
                segment: Color32::from_rgb(0x43, 0x48, 0x4C),
                selected: Color32::from_rgb(0x3A, 0xAD, 0xAD),
                alert: Color32::from_rgb(0xBF, 0x84, 0x48),
                marker: Color32::from_rgb(0xBF, 0xA2, 0x84),
                text: Color32::from_rgb(0x2B, 0x2F, 0x33),
                background: Color32::from_rgb(0xF1, 0xF6, 0xFB),
            },

            (NebulaViolet, Dark) => ThemeColors {
                node: Color32::from_rgb(0x9F, 0x65, 0xE0),
                segment: Color32::from_rgb(0x54, 0x5E, 0x8C),
                selected: Color32::from_rgb(0x3B, 0xEB, 0xEB),
                alert: Color32::from_rgb(0xF2, 0x49, 0x9D),
                marker: Color32::from_rgb(0xF2, 0x9E, 0xC8),
                text: Color32::from_rgb(0xE3, 0xD8, 0xF0),
                background: Color32::from_rgb(0x0C, 0x0B, 0x0D),
            },
            (NebulaViolet, Light) => ThemeColors {
                node: Color32::from_rgb(0x59, 0x32, 0x85),
                segment: Color32::from_rgb(0xA3, 0x96, 0xB2),
                selected: Color32::from_rgb(0x1A, 0xAD, 0xAD),
                alert: Color32::from_rgb(0xBF, 0x26, 0x73),
                marker: Color32::from_rgb(0xBF, 0x72, 0x99),
                text: Color32::from_rgb(0x2E, 0x29, 0x33),
                background: Color32::from_rgb(0xF6, 0xF1, 0xFB),
            },

            (TerminalGreen, Dark) => ThemeColors {
                node: Color32::from_rgb(0x65, 0xE0, 0x84),
                segment: Color32::from_rgb(0x5C, 0x80, 0x65),
                selected: Color32::from_rgb(0x3B, 0xBF, 0xEB),
                alert: Color32::from_rgb(0xF2, 0x73, 0x49),
                marker: Color32::from_rgb(0xF2, 0xB2, 0x9E),
                text: Color32::from_rgb(0xD8, 0xF0, 0xDE),
                background: Color32::from_rgb(0x0B, 0x0D, 0x0C),
            },
            (TerminalGreen, Light) => ThemeColors {
                node: Color32::from_rgb(0x32, 0x85, 0x47),
                segment: Color32::from_rgb(0x96, 0xB2, 0x9D),
                selected: Color32::from_rgb(0x1A, 0x89, 0xAD),
                alert: Color32::from_rgb(0xBF, 0x4C, 0x26),
                marker: Color32::from_rgb(0xBF, 0x86, 0x72),
                text: Color32::from_rgb(0x29, 0x33, 0x2B),
                background: Color32::from_rgb(0xF1, 0xFB, 0xF4),
            },

            (EmberForge, Dark) => ThemeColors {
                node: Color32::from_rgb(0xE0, 0xB3, 0x65),
                segment: Color32::from_rgb(0xD9, 0xC1, 0x98),
                selected: Color32::from_rgb(0x3B, 0xEB, 0xB0),
                alert: Color32::from_rgb(0x49, 0x87, 0xF2),
                marker: Color32::from_rgb(0x9E, 0xBC, 0xF2),
                text: Color32::from_rgb(0xF0, 0xE7, 0xD8),
                background: Color32::from_rgb(0x0D, 0x0C, 0x0B),
            },
            (EmberForge, Light) => ThemeColors {
                node: Color32::from_rgb(0x85, 0x66, 0x32),
                segment: Color32::from_rgb(0xB2, 0xA8, 0x96),
                selected: Color32::from_rgb(0x1A, 0xAD, 0x7C),
                alert: Color32::from_rgb(0x26, 0x5E, 0xBF),
                marker: Color32::from_rgb(0x72, 0x8F, 0xBF),
                text: Color32::from_rgb(0x33, 0x2F, 0x29),
                background: Color32::from_rgb(0xFB, 0xF7, 0xF1),
            },

            (SolarAmber, Dark) => ThemeColors {
                node: Color32::from_rgb(0xE0, 0xC6, 0x5F),
                segment: Color32::from_rgb(0x94, 0x8B, 0x68),
                selected: Color32::from_rgb(0x32, 0x7F, 0xEB),
                alert: Color32::from_rgb(0xF2, 0x40, 0x4F),
                marker: Color32::from_rgb(0xF2, 0x99, 0xA0),
                text: Color32::from_rgb(0xF0, 0xEB, 0xD7),
                background: Color32::from_rgb(0x0D, 0x0C, 0x0B),
            },
            (SolarAmber, Light) => ThemeColors {
                node: Color32::from_rgb(0x85, 0x73, 0x2E),
                segment: Color32::from_rgb(0xB2, 0xAD, 0x95),
                selected: Color32::from_rgb(0x13, 0x53, 0xAD),
                alert: Color32::from_rgb(0xBF, 0x1F, 0x2C),
                marker: Color32::from_rgb(0xBF, 0x6F, 0x76),
                text: Color32::from_rgb(0x33, 0x31, 0x28),
                background: Color32::from_rgb(0xFB, 0xF9, 0xF1),
            },

            (ArticCyan, Dark) => ThemeColors {
                node: Color32::from_rgb(0x65, 0xCC, 0xE0),
                segment: Color32::from_rgb(0x65, 0x86, 0x8C),
                selected: Color32::from_rgb(0xEB, 0x3B, 0xCD),
                alert: Color32::from_rgb(0xF2, 0x65, 0x49),
                marker: Color32::from_rgb(0xF2, 0xAB, 0x9E),
                text: Color32::from_rgb(0xD8, 0xEC, 0xF0),
                background: Color32::from_rgb(0x0B, 0x0D, 0x0D),
            },
            (ArticCyan, Light) => ThemeColors {
                node: Color32::from_rgb(0x32, 0x77, 0x85),
                segment: Color32::from_rgb(0x96, 0xAE, 0xB2),
                selected: Color32::from_rgb(0xAD, 0x1A, 0x95),
                alert: Color32::from_rgb(0xBF, 0x40, 0x26),
                marker: Color32::from_rgb(0xBF, 0x7F, 0x72),
                text: Color32::from_rgb(0x29, 0x31, 0x33),
                background: Color32::from_rgb(0xF1, 0xFA, 0xFB),
            },

            (CrimsonSignal, Dark) => ThemeColors {
                node: Color32::from_rgb(0xE0, 0x59, 0x6F),
                segment: Color32::from_rgb(0x85, 0x5C, 0x63),
                selected: Color32::from_rgb(0x29, 0xAA, 0xEB),
                alert: Color32::from_rgb(0xF2, 0x66, 0x38),
                marker: Color32::from_rgb(0xF2, 0xAC, 0x95),
                text: Color32::from_rgb(0xF0, 0xD5, 0xDA),
                background: Color32::from_rgb(0x0D, 0x0B, 0x0C),
            },
            (CrimsonSignal, Light) => ThemeColors {
                node: Color32::from_rgb(0x85, 0x2A, 0x39),
                segment: Color32::from_rgb(0x33, 0x2A, 0x2C),
                selected: Color32::from_rgb(0x0B, 0x77, 0xAD),
                alert: Color32::from_rgb(0xBF, 0x41, 0x17),
                marker: Color32::from_rgb(0xBF, 0x80, 0x6B),
                text: Color32::from_rgb(0x33, 0x28, 0x2A),
                background: Color32::from_rgb(0xFB, 0xF1, 0xF3),
            },

            (MidnightIndigo, Dark) => ThemeColors {
                node: Color32::from_rgb(0x6F, 0x65, 0xE0),
                segment: Color32::from_rgb(0x6E, 0x6A, 0x94),
                selected: Color32::from_rgb(0xEB, 0x67, 0x3B),
                alert: Color32::from_rgb(0xF2, 0xD6, 0x49),
                marker: Color32::from_rgb(0xF2, 0xE4, 0x9E),
                text: Color32::from_rgb(0xDA, 0xD8, 0xF0),
                background: Color32::from_rgb(0x0C, 0x0B, 0x0D),
            },
            (MidnightIndigo, Light) => ThemeColors {
                node: Color32::from_rgb(0x39, 0x32, 0x85),
                segment: Color32::from_rgb(0x46, 0x45, 0x52),
                selected: Color32::from_rgb(0xAD, 0x3F, 0x1A),
                alert: Color32::from_rgb(0xBF, 0xA6, 0x26),
                marker: Color32::from_rgb(0xBF, 0xB3, 0x72),
                text: Color32::from_rgb(0x2A, 0x29, 0x33),
                background: Color32::from_rgb(0xF2, 0xF1, 0xFB),
            },

            (CopperRose, Dark) => ThemeColors {
                node: Color32::from_rgb(0xE0, 0xA2, 0x7E),
                segment: Color32::from_rgb(0x8C, 0x78, 0x6D),
                selected: Color32::from_rgb(0xEB, 0x5E, 0xA4),
                alert: Color32::from_rgb(0x6B, 0xE7, 0xF2),
                marker: Color32::from_rgb(0xAE, 0xEC, 0xF2),
                text: Color32::from_rgb(0xF0, 0xE4, 0xDD),
                background: Color32::from_rgb(0x0D, 0x0C, 0x0B),
            },
            (CopperRose, Light) => ThemeColors {
                node: Color32::from_rgb(0x85, 0x5B, 0x43),
                segment: Color32::from_rgb(0x4C, 0x46, 0x43),
                selected: Color32::from_rgb(0xAD, 0x37, 0x72),
                alert: Color32::from_rgb(0x45, 0xB5, 0xBF),
                marker: Color32::from_rgb(0x82, 0xBA, 0xBF),
                text: Color32::from_rgb(0x33, 0x2E, 0x2B),
                background: Color32::from_rgb(0xFB, 0xF5, 0xF1),
            },

            (LimeCircuit, Dark) => ThemeColors {
                node: Color32::from_rgb(0x90, 0xE0, 0x56),
                segment: Color32::from_rgb(0xA8, 0xCC, 0x8F),
                selected: Color32::from_rgb(0x26, 0x78, 0xEB),
                alert: Color32::from_rgb(0xA3, 0x34, 0xF2),
                marker: Color32::from_rgb(0xCA, 0x93, 0xF2),
                text: Color32::from_rgb(0xE0, 0xF0, 0xD5),
                background: Color32::from_rgb(0x0C, 0x0D, 0x0B),
            },
            (LimeCircuit, Light) => ThemeColors {
                node: Color32::from_rgb(0x4F, 0x85, 0x29),
                segment: Color32::from_rgb(0xA0, 0xB2, 0x93),
                selected: Color32::from_rgb(0x08, 0x4D, 0xAD),
                alert: Color32::from_rgb(0x78, 0x14, 0xBF),
                marker: Color32::from_rgb(0x9B, 0x6A, 0xBF),
                text: Color32::from_rgb(0x2C, 0x33, 0x28),
                background: Color32::from_rgb(0xF5, 0xFB, 0xF1),
            },

            (CoralReef, Dark) => ThemeColors {
                node: Color32::from_rgb(0xE0, 0x75, 0x65),
                segment: Color32::from_rgb(0x8C, 0x6A, 0x65),
                selected: Color32::from_rgb(0x3B, 0xEB, 0x52),
                alert: Color32::from_rgb(0x49, 0xDC, 0xF2),
                marker: Color32::from_rgb(0x9E, 0xE7, 0xF2),
                text: Color32::from_rgb(0xF0, 0xDB, 0xD8),
                background: Color32::from_rgb(0x0D, 0x0C, 0x0B),
            },
            (CoralReef, Light) => ThemeColors {
                node: Color32::from_rgb(0x85, 0x3D, 0x32),
                segment: Color32::from_rgb(0x4C, 0x42, 0x40),
                selected: Color32::from_rgb(0x1A, 0xAD, 0x2E),
                alert: Color32::from_rgb(0x26, 0xAB, 0xBF),
                marker: Color32::from_rgb(0x72, 0xB5, 0xBF),
                text: Color32::from_rgb(0x33, 0x2A, 0x29),
                background: Color32::from_rgb(0xFB, 0xF2, 0xF1),
            },

            (GraphiteMono, Dark) => ThemeColors {
                node: Color32::from_rgb(0xCC, 0xCC, 0xCC),
                segment: Color32::from_rgb(0x4C, 0x4C, 0x4C),
                selected: Color32::from_rgb(0x3B, 0xA1, 0xEB),
                alert: Color32::from_rgb(0xF2, 0x6A, 0x3D),
                marker: Color32::from_rgb(0xF2, 0xAE, 0x98),
                text: Color32::from_rgb(0xEB, 0xEB, 0xEB),
                background: Color32::from_rgb(0x0D, 0x0D, 0x0D),
            },
            (GraphiteMono, Light) => ThemeColors {
                node: Color32::from_rgb(0x59, 0x59, 0x59),
                segment: Color32::from_rgb(0xBF, 0xBF, 0xBF),
                selected: Color32::from_rgb(0x17, 0x63, 0x99),
                alert: Color32::from_rgb(0xBF, 0x4C, 0x26),
                marker: Color32::from_rgb(0xBF, 0x86, 0x72),
                text: Color32::from_rgb(0x26, 0x26, 0x26),
                background: Color32::from_rgb(0xFB, 0xFB, 0xFB),
            },

            (PlumStatic, Dark) => ThemeColors {
                node: Color32::from_rgb(0xE0, 0x7B, 0xC7),
                segment: Color32::from_rgb(0x8C, 0x6C, 0x84),
                selected: Color32::from_rgb(0xEB, 0x5A, 0x66),
                alert: Color32::from_rgb(0xF2, 0xB8, 0x67),
                marker: Color32::from_rgb(0xF2, 0xD5, 0xAC),
                text: Color32::from_rgb(0xF0, 0xDC, 0xEB),
                background: Color32::from_rgb(0x0D, 0x0B, 0x0C),
            },
            (PlumStatic, Light) => ThemeColors {
                node: Color32::from_rgb(0x85, 0x41, 0x74),
                segment: Color32::from_rgb(0x47, 0x3E, 0x45),
                selected: Color32::from_rgb(0xAD, 0x35, 0x3F),
                alert: Color32::from_rgb(0xBF, 0x8B, 0x42),
                marker: Color32::from_rgb(0xBF, 0xA5, 0x80),
                text: Color32::from_rgb(0x33, 0x2B, 0x31),
                background: Color32::from_rgb(0xFB, 0xF1, 0xF9),
            },

            (SandstoneTrail, Dark) => ThemeColors {
                node: Color32::from_rgb(0xDD, 0xE0, 0x88),
                segment: Color32::from_rgb(0xD7, 0xD9, 0x9C),
                selected: Color32::from_rgb(0x6C, 0xE6, 0xEB),
                alert: Color32::from_rgb(0x7C, 0x78, 0xF2),
                marker: Color32::from_rgb(0xB7, 0xB5, 0xF2),
                text: Color32::from_rgb(0xEF, 0xF0, 0xDE),
                background: Color32::from_rgb(0x0D, 0x0D, 0x0B),
            },
            (SandstoneTrail, Light) => ThemeColors {
                node: Color32::from_rgb(0x83, 0x85, 0x49),
                segment: Color32::from_rgb(0xB2, 0xB2, 0x9E),
                selected: Color32::from_rgb(0x43, 0xAA, 0xAD),
                alert: Color32::from_rgb(0x55, 0x51, 0xBF),
                marker: Color32::from_rgb(0x8A, 0x88, 0xBF),
                text: Color32::from_rgb(0x33, 0x33, 0x2C),
                background: Color32::from_rgb(0xFB, 0xFB, 0xF1),
            },

            (EguiDefault, Dark) => ThemeColors {
                node: Color32::from_rgb(0x5A, 0xAA, 0xFF),
                segment: Color32::from_rgb(0x8C, 0x8C, 0x8C),
                selected: Color32::from_rgb(0xC0, 0xDE, 0xFF),
                alert: Color32::from_rgb(0xFF, 0x00, 0x00),
                marker: Color32::from_rgb(0xF2, 0x79, 0x79),
                text: Color32::from_rgb(0xFF, 0xFF, 0xFF),
                background: Color32::from_rgb(0x0A, 0x0A, 0x0A),
            },
            (EguiDefault, Light) => ThemeColors {
                node: Color32::from_rgb(0x00, 0x9B, 0xFF),
                segment: Color32::from_rgb(0xBE, 0xBE, 0xBE),
                selected: Color32::from_rgb(0x00, 0x53, 0x7D),
                alert: Color32::from_rgb(0xFF, 0x00, 0x00),
                marker: Color32::from_rgb(0xF2, 0x79, 0x79),
                text: Color32::from_rgb(0x00, 0x00, 0x00),
                background: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            },
        }
    }
}

/// A resolved color palette: the actual colors a [`Theme`] (or any other
/// [`MapTheme`] implementation) uses to paint the map under one
/// [`ColorMode`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemeColors {
    /// Color used to fill node shapes.
    pub node: Color32,
    /// Color used for connection lines between nodes.
    pub segment: Color32,
    /// Color used for the selection highlight over the nearest node.
    pub selected: Color32,
    /// Color used for one-off notification/alert animations (`pulse`,
    /// `ripple`, `flash`, `comet_once`, `wipe`, ...) -- transient, event-
    /// driven effects.
    pub alert: Color32,
    /// Color used for a lasting "this is marked" indicator: a node's
    /// persistent state (`halo`/`blink`/`orbit`, started through
    /// [`Map::node`](super::Map::node)) and a plain marker registered with
    /// [`Map::update_marker`](super::Map::update_marker). Deliberately its
    /// own role, distinct from [`selected`](Self::selected) (which node is
    /// nearest the pointer, right now) and [`alert`](Self::alert) (a
    /// one-off event playing out) -- a marker means "flagged", indefinitely,
    /// independent of both.
    pub marker: Color32,
    /// Color used for node names and map labels.
    pub text: Color32,
    /// Background color of the map canvas.
    ///
    /// Resolved live from the active [`MapTheme`] for the current
    /// [`ColorMode`], exactly like every other role on this struct -- unlike
    /// [`NodeContext::background_color`](super::objects::NodeContext::background_color),
    /// which deliberately follows the surrounding egui [`Visuals`](egui::Visuals)
    /// instead, so a template's chip/panel backgrounds still blend into the
    /// host application even when this role paints something else.
    pub background: Color32,
}

/// Which of a theme's two color variants to use.
///
/// A re-export of [`egui::Theme`] under this crate's existing name -- egui
/// already models exactly this dark/light distinction (down to a
/// [`ColorMode::from_dark_mode`] conversion from `ui.visuals().dark_mode`),
/// so a second, identical `Light`/`Dark` enum here would just be another
/// copy of the same two variants to keep in sync. Not to be confused with
/// this crate's own [`Theme`], a named color palette.
pub use egui::Theme as ColorMode;

/// Customization point for the color palette used to paint the map.
///
/// Implement this to install your own palette instead of one of the
/// built-in [`Theme`] variants -- install it with
/// [`Map::set_theme`](super::Map::set_theme). `Theme` itself implements
/// `MapTheme` by delegating to [`Theme::colors`], so switching between a
/// built-in palette and a custom one is just a different argument to
/// `set_theme`.
///
/// ```
/// use egui_map::map::theme::{ColorMode, MapTheme, ThemeColors};
/// use egui::Color32;
///
/// struct HighContrast;
///
/// impl MapTheme for HighContrast {
///     fn colors(&self, mode: ColorMode) -> ThemeColors {
///         match mode {
///             ColorMode::Light => ThemeColors {
///                 node: Color32::BLACK,
///                 segment: Color32::DARK_GRAY,
///                 selected: Color32::RED,
///                 alert: Color32::RED,
///                 marker: Color32::BLUE,
///                 text: Color32::BLACK,
///                 background: Color32::WHITE,
///             },
///             ColorMode::Dark => ThemeColors {
///                 node: Color32::WHITE,
///                 segment: Color32::LIGHT_GRAY,
///                 selected: Color32::YELLOW,
///                 alert: Color32::YELLOW,
///                 marker: Color32::LIGHT_BLUE,
///                 text: Color32::WHITE,
///                 background: Color32::BLACK,
///             },
///         }
///     }
/// }
///
/// let mut map = egui_map::map::Map::new();
/// map.set_theme(std::rc::Rc::new(HighContrast));
/// ```
pub trait MapTheme {
    /// Returns the color palette to use for the given [`ColorMode`].
    fn colors(&self, mode: ColorMode) -> ThemeColors;
}

impl MapTheme for Theme {
    fn colors(&self, mode: ColorMode) -> ThemeColors {
        Theme::colors(*self, mode)
    }
}

/// Visual style used to paint the map under a given theme.
///
/// Holds the non-palette visual configuration -- stroke width, font -- that
/// combines with the active [`MapTheme`]'s [`ThemeColors`] when the widget
/// paints. `Style` itself carries no *palette* color: every
/// node/segment/selection/alert/marker/text/background color the widget
/// paints with is resolved live from the installed [`MapTheme`] for the
/// current [`ColorMode`] -- see [`Map::set_theme`](super::Map::set_theme) --
/// instead of living here as a copy that would need to be kept in sync.
///
/// Multiplying or dividing a `Style` by a number scales
/// [`line_width`](Self::line_width) and [`font`](Self::font)'s size; the
/// widget uses this to scale the active style with the current zoom factor.
/// Fields that are `None` are left untouched by those operators, and so is
/// [`region_label_font`](Self::region_label_font) -- it has its own,
/// separate zoom scaling (see that field's own doc), unrelated to this
/// operator.
#[derive(Clone, Debug)]
pub struct Style {
    /// Width of the connection lines between nodes; their color comes from
    /// the active [`MapTheme`]'s [`ThemeColors::segment`]. `None` disables
    /// the default stroke -- a
    /// [`SegmentTemplate`](super::objects::SegmentTemplate) or a segment
    /// effect installed through [`Map::segment`](super::Map::segment) still
    /// runs either way.
    pub line_width: Option<f32>,
    /// Font used for map labels.
    pub font: Option<FontId>,
    /// Font used for [`RegionLabel`](super::objects::RegionLabel)s.
    ///
    /// Same type as [`font`](Self::font), for structural symmetry, but
    /// mandatory rather than optional: every `Style` must pick an explicit
    /// typeface and size for region labels, there is no built-in fallback
    /// to fall back to. Both [`FontId::family`] (the typeface) and
    /// [`FontId::size`] (the *base* size -- still multiplied by the
    /// current zoom) are read directly by the built-in renderer.
    pub region_label_font: FontId,
}

impl Style {
    /// Creates a fully transparent style with no line or font, and a
    /// zero-size proportional [`region_label_font`](Self::region_label_font)
    /// -- that field can't be absent, so this is its neutral placeholder,
    /// same spirit as the zeroed-out fields on
    /// [`MapSettings::new()`](super::objects::MapSettings::new()).
    pub fn new() -> Self {
        Style {
            line_width: None,
            font: None,
            region_label_font: FontId::new(0.0, FontFamily::Proportional),
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Style::new()
    }
}

impl Style {
    /// Returns a copy with the stroke widths and font size scaled by `factor`.
    /// Fields that are `None` are left untouched.
    fn scaled(mut self, factor: f32) -> Self {
        if let Some(width) = self.line_width.as_mut() {
            *width *= factor;
        }
        if let Some(font) = self.font.as_mut() {
            font.size *= factor;
        }
        self
    }
}

impl Mul<i64> for Style {
    type Output = Self;

    fn mul(self, rhs: i64) -> Self::Output {
        self.scaled(rhs as f32)
    }
}

impl Mul<i32> for Style {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        self.scaled(rhs as f32)
    }
}

impl Mul<f32> for Style {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        self.scaled(rhs)
    }
}

impl Mul<f64> for Style {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        self.scaled(rhs as f32)
    }
}

impl Div<i64> for Style {
    type Output = Self;

    fn div(self, rhs: i64) -> Self::Output {
        self.scaled(1.0 / rhs as f32)
    }
}

impl Div<i32> for Style {
    type Output = Self;

    fn div(self, rhs: i32) -> Self::Output {
        self.scaled(1.0 / rhs as f32)
    }
}

impl Div<f32> for Style {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        self.scaled(1.0 / rhs)
    }
}

impl Div<f64> for Style {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        self.scaled(1.0 / rhs as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_THEMES: [Theme; 15] = [
        Theme::EguiDefault,
        Theme::SlateOcean,
        Theme::NebulaViolet,
        Theme::TerminalGreen,
        Theme::EmberForge,
        Theme::SolarAmber,
        Theme::ArticCyan,
        Theme::CrimsonSignal,
        Theme::MidnightIndigo,
        Theme::CopperRose,
        Theme::LimeCircuit,
        Theme::CoralReef,
        Theme::GraphiteMono,
        Theme::PlumStatic,
        Theme::SandstoneTrail,
    ];

    #[test]
    fn every_theme_has_distinct_light_and_dark_palettes() {
        // Regression test for a typo (`Ligth` instead of `Light`) that once
        // made an unqualified identifier bind irrefutably in a match arm,
        // silently turning `(SolarAmber, Light)` into a catch-all -- it only
        // still looked right because `(SolarAmber, Dark)`, listed first,
        // matched exactly and won for that case, leaving the buggy arm to
        // fire (and return the *wrong*, `Dark`-shaped) colors for `Light`.
        for theme in ALL_THEMES {
            let light = theme.colors(ColorMode::Light);
            let dark = theme.colors(ColorMode::Dark);
            assert_ne!(
                light, dark,
                "{theme:?}'s Light and Dark palettes must not be identical"
            );
        }
    }

    #[test]
    fn solar_amber_light_matches_its_defined_palette() {
        // Pins the exact SolarAmber/Light values so a regression back to the
        // `Ligth` typo -- which made this arm silently reuse whatever
        // `(SolarAmber, Dark)` produced -- is caught even though `assert_ne`
        // alone only proves the two differ, not that `Light` is *this*
        // palette.
        //
        // These values also fix a second, distinct historical bug: the
        // entire (SolarAmber, Dark) and (SolarAmber, Light) blocks (all 5
        // fields, not just `text`) were swapped -- so `Light`'s `text` was
        // a near-white cream, nearly invisible against the map canvas's
        // white background in light mode (contrast ratio ~1.3:1, versus
        // ~13:1+ for every other built-in theme). Same bug, same fix, in
        // `ArticCyan`.
        assert_eq!(
            Theme::SolarAmber.colors(ColorMode::Light),
            ThemeColors {
                node: Color32::from_rgb(0x85, 0x73, 0x2E),
                segment: Color32::from_rgb(0xB2, 0xAD, 0x95),
                selected: Color32::from_rgb(0x13, 0x53, 0xAD),
                alert: Color32::from_rgb(0xBF, 0x1F, 0x2C),
                marker: Color32::from_rgb(0xBF, 0x6F, 0x76),
                text: Color32::from_rgb(0x33, 0x31, 0x28),
                background: Color32::from_rgb(0xFB, 0xF9, 0xF1),
            }
        );
    }

    #[test]
    fn built_in_theme_text_color_is_legible_on_the_map_canvas() {
        // Regression test for the ArticCyan/SolarAmber swapped-palette bug
        // (see `solar_amber_light_matches_its_defined_palette`): every
        // built-in theme's `text` must have real contrast against a
        // realistic map canvas background for its color mode -- not just
        // "not equal to it". These two constants stand in for a typical
        // host application's own light/dark background (what
        // `egui::Visuals::light()`/`dark()`'s `extreme_bg_color` looks
        // like) -- the map canvas has no background of its own, so this is
        // what `text` actually sits on in a default host app.
        const CANVAS_LIGHT: Color32 = Color32::from_rgb(255, 255, 255);
        const CANVAS_DARK: Color32 = Color32::from_rgb(10, 10, 10);
        // WCAG AA for normal text is 4.5:1; every hand-authored palette here
        // clears 10:1+, so this threshold only catches a genuinely broken
        // pairing, not a merely muted one.
        const MIN_CONTRAST: f32 = 4.5;

        fn relative_luminance(c: Color32) -> f32 {
            fn channel(v: u8) -> f32 {
                let v = v as f32 / 255.0;
                if v <= 0.03928 {
                    v / 12.92
                } else {
                    ((v + 0.055) / 1.055).powf(2.4)
                }
            }
            0.2126 * channel(c.r()) + 0.7152 * channel(c.g()) + 0.0722 * channel(c.b())
        }

        fn contrast_ratio(a: Color32, b: Color32) -> f32 {
            let (la, lb) = (relative_luminance(a), relative_luminance(b));
            let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
            (hi + 0.05) / (lo + 0.05)
        }

        for theme in ALL_THEMES {
            for (mode, canvas) in [
                (ColorMode::Light, CANVAS_LIGHT),
                (ColorMode::Dark, CANVAS_DARK),
            ] {
                let text = theme.colors(mode).text;
                let ratio = contrast_ratio(text, canvas);
                assert!(
                    ratio >= MIN_CONTRAST,
                    "{theme:?}/{mode:?}: text {text:?} has only {ratio:.2}:1 contrast \
                     against the {mode:?} canvas -- needs at least {MIN_CONTRAST}:1"
                );
            }
        }
    }

    #[test]
    fn built_in_theme_selected_is_at_least_as_saturated_as_marker() {
        // `selected` (the ring around the node nearest the pointer, live and
        // real-time) is meant to read as more vivid than `marker` (a lasting
        // "this is flagged" indicator) -- not a different hue, just clearly
        // punchier -- so the two roles don't compete for attention with the
        // wrong one winning. Saturation (not brightness) is the right axis
        // to compare on: it tracks "how colorful" independent of how bright
        // a color happens to be, so boosting it doesn't force `selected`
        // toward washed-out pastel or blown-out white the way chasing raw
        // brightness would.
        fn hsv_saturation(c: Color32) -> f32 {
            let (r, g, b) = (c.r() as f32, c.g() as f32, c.b() as f32);
            let max = r.max(g).max(b);
            let min = r.min(g).min(b);
            if max <= 0.0 { 0.0 } else { (max - min) / max }
        }

        for theme in ALL_THEMES {
            // `EguiDefault` is excluded: its `selected` is egui's own
            // `selection.stroke` light-blue (S=0.25 in Dark mode), which is
            // deliberately *less* saturated than the pale `marker` derived
            // from its `alert` (S=0.50) -- keeping `selected` at egui's own
            // literal value (rather than inventing a more saturated one just
            // to satisfy this invariant) was a deliberate choice for this
            // theme specifically, see `claude/diseno-revamp-temas-...md`.
            // The invariant still holds, unforced, for every other built-in
            // theme.
            if theme == Theme::EguiDefault {
                continue;
            }
            for mode in [ColorMode::Light, ColorMode::Dark] {
                let colors = theme.colors(mode);
                let (sel_s, marker_s) = (
                    hsv_saturation(colors.selected),
                    hsv_saturation(colors.marker),
                );
                assert!(
                    sel_s >= marker_s - 0.01,
                    "{theme:?}/{mode:?}: selected {:?} (S={sel_s:.2}) is less \
                     saturated than marker {:?} (S={marker_s:.2})",
                    colors.selected,
                    colors.marker,
                );
            }
        }
    }

    #[test]
    fn theme_implements_map_theme_by_delegating_to_colors() {
        let theme: &dyn MapTheme = &Theme::TerminalGreen;
        assert_eq!(
            theme.colors(ColorMode::Dark),
            Theme::TerminalGreen.colors(ColorMode::Dark)
        );
        assert_eq!(
            theme.colors(ColorMode::Light),
            Theme::TerminalGreen.colors(ColorMode::Light)
        );
    }
}

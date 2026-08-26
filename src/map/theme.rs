//! Color palettes and visual styling for the [`Map`](super::Map) widget.
//!
//! [`Theme`] is a built-in set of named color palettes, each with a light and
//! a dark variant ([`ColorMode`]); [`Theme::colors`] resolves a `(Theme,
//! ColorMode)` pair to the actual [`ThemeColors`] palette. [`MapTheme`] is
//! the customization point: implement it to install your own palette instead
//! of one of the built-ins, the same way [`NodeTemplate`](super::objects::NodeTemplate)
//! and [`SegmentTemplate`](super::objects::SegmentTemplate) let you replace
//! the built-in node/segment rendering. [`Style`] is the non-palette visual
//! configuration (stroke widths, font, background) the widget paints with; it
//! holds no color of its own -- every color comes live from the active
//! [`MapTheme`], resolved fresh each frame, so nothing in `Style` can drift
//! out of sync with the installed theme.

use crate::map::theme::{
    ColorMode::{Dark, Light},
    Theme::*,
};
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
    /// Muted blues over slate grays.
    SlateOcean,
    /// Violet and teal on a soft neutral backdrop. The default theme.
    #[default]
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
                node: Color32::from_rgb(0x6E, 0x93, 0xBF),
                segment: Color32::from_rgb(0x4A, 0x5A, 0x6E),
                selected: Color32::from_rgb(0x38, 0xBD, 0xF8),
                alert: Color32::from_rgb(0xFF, 0x8A, 0x4C),
                text: Color32::from_rgb(0xD7, 0xDE, 0xE6),
            },
            (SlateOcean, Light) => ThemeColors {
                node: Color32::from_rgb(0x2E, 0x4C, 0x6D),
                segment: Color32::from_rgb(0x8A, 0x99, 0xA8),
                selected: Color32::from_rgb(0x0E, 0xA5, 0xE9),
                alert: Color32::from_rgb(0xE0, 0x59, 0x2A),
                text: Color32::from_rgb(0x2B, 0x33, 0x3D),
            },
            (NebulaViolet, Light) => ThemeColors {
                node: Color32::from_rgb(0x5B, 0x3E, 0x96),
                segment: Color32::from_rgb(0xB3, 0xA4, 0xD6),
                selected: Color32::from_rgb(0x16, 0xB8, 0xA6),
                alert: Color32::from_rgb(0xE6, 0x33, 0x7A),
                text: Color32::from_rgb(0x37, 0x2F, 0x45),
            },
            (NebulaViolet, Dark) => ThemeColors {
                node: Color32::from_rgb(0xA7, 0x8B, 0xFA),
                segment: Color32::from_rgb(0x5C, 0x4A, 0x80),
                selected: Color32::from_rgb(0x2D, 0xD4, 0xBF),
                alert: Color32::from_rgb(0xFF, 0x5F, 0xA3),
                text: Color32::from_rgb(0xE4, 0xDC, 0xF2),
            },
            (TerminalGreen, Light) => ThemeColors {
                node: Color32::from_rgb(0x1F, 0x7A, 0x3D),
                segment: Color32::from_rgb(0x9B, 0xB8, 0x9E),
                selected: Color32::from_rgb(0x25, 0x63, 0xEB),
                alert: Color32::from_rgb(0xD6, 0xA4, 0x29),
                text: Color32::from_rgb(0x2A, 0x33, 0x2C),
            },
            (TerminalGreen, Dark) => ThemeColors {
                node: Color32::from_rgb(0x4A, 0xDE, 0x80),
                segment: Color32::from_rgb(0x3A, 0x52, 0x40),
                selected: Color32::from_rgb(0x60, 0xA5, 0xFA),
                alert: Color32::from_rgb(0xFF, 0xC9, 0x4D),
                text: Color32::from_rgb(0xD7, 0xE6, 0xDA),
            },
            (EmberForge, Light) => ThemeColors {
                node: Color32::from_rgb(0x8C, 0x4A, 0x1F),
                segment: Color32::from_rgb(0xC9, 0xA9, 0x8C),
                selected: Color32::from_rgb(0xC4, 0x25, 0x8C),
                alert: Color32::from_rgb(0x2E, 0x86, 0xAB),
                text: Color32::from_rgb(0x36, 0x2E, 0x28),
            },
            (EmberForge, Dark) => ThemeColors {
                node: Color32::from_rgb(0xD9, 0x7F, 0x3D),
                segment: Color32::from_rgb(0x5A, 0x46, 0x32),
                selected: Color32::from_rgb(0xF4, 0x72, 0xB6),
                alert: Color32::from_rgb(0x4F, 0xC3, 0xE8),
                text: Color32::from_rgb(0xED, 0xE0, 0xD3),
            },
            (SolarAmber, Dark) => ThemeColors {
                node: Color32::from_rgb(0x8A, 0x6D, 0x1E),
                segment: Color32::from_rgb(0xD8, 0xC4, 0x8F),
                selected: Color32::from_rgb(0x2D, 0x6E, 0x5E),
                alert: Color32::from_rgb(0xC1, 0x44, 0x2A),
                text: Color32::from_rgb(0x36, 0x2E, 0x1C),
            },
            (SolarAmber, Light) => ThemeColors {
                node: Color32::from_rgb(0xF0, 0xC2, 0x4C),
                segment: Color32::from_rgb(0x5A, 0x4E, 0x2E),
                selected: Color32::from_rgb(0x4F, 0xBF, 0x9E),
                alert: Color32::from_rgb(0xE8, 0x65, 0x4A),
                text: Color32::from_rgb(0xEF, 0xE4, 0xC4),
            },
            (ArticCyan, Dark) => ThemeColors {
                node: Color32::from_rgb(0x12, 0x70, 0x8A),
                segment: Color32::from_rgb(0x9A, 0xC6, 0xD1),
                selected: Color32::from_rgb(0xF5, 0xA5, 0x24),
                alert: Color32::from_rgb(0xE0, 0x52, 0x7A),
                text: Color32::from_rgb(0x1F, 0x2E, 0x30),
            },
            (ArticCyan, Light) => ThemeColors {
                node: Color32::from_rgb(0x4F, 0xE0, 0xFF),
                segment: Color32::from_rgb(0x37, 0x5E, 0x68),
                selected: Color32::from_rgb(0xFB, 0xBF, 0x24),
                alert: Color32::from_rgb(0xFF, 0x6B, 0x95),
                text: Color32::from_rgb(0xD3, 0xEA, 0xEF),
            },
            (CrimsonSignal, Dark) => ThemeColors {
                node: Color32::from_rgb(0xE3, 0x5B, 0x6B),
                segment: Color32::from_rgb(0x5C, 0x45, 0x48),
                selected: Color32::from_rgb(0x58, 0xA6, 0xFF),
                alert: Color32::from_rgb(0xFF, 0xC4, 0x59),
                text: Color32::from_rgb(0xE8, 0xD6, 0xD8),
            },
            (CrimsonSignal, Light) => ThemeColors {
                node: Color32::from_rgb(0x7A, 0x1F, 0x2B),
                segment: Color32::from_rgb(0xB9, 0xA8, 0xA8),
                selected: Color32::from_rgb(0x1F, 0x6F, 0xEB),
                alert: Color32::from_rgb(0xE8, 0xA6, 0x28),
                text: Color32::from_rgb(0x33, 0x26, 0x28),
            },

            (MidnightIndigo, Dark) => ThemeColors {
                node: Color32::from_rgb(0x7B, 0x82, 0xE0),
                segment: Color32::from_rgb(0x3A, 0x3D, 0x66),
                selected: Color32::from_rgb(0x2D, 0xD4, 0xC8),
                alert: Color32::from_rgb(0xFF, 0xD1, 0x66),
                text: Color32::from_rgb(0xD8, 0xDA, 0xF0),
            },
            (MidnightIndigo, Light) => ThemeColors {
                node: Color32::from_rgb(0x2B, 0x2F, 0x77),
                segment: Color32::from_rgb(0xA6, 0xA9, 0xC9),
                selected: Color32::from_rgb(0x00, 0xB8, 0xA9),
                alert: Color32::from_rgb(0xE0, 0xA4, 0x00),
                text: Color32::from_rgb(0x26, 0x29, 0x40),
            },

            (CopperRose, Dark) => ThemeColors {
                node: Color32::from_rgb(0xE0, 0x8B, 0x6F),
                segment: Color32::from_rgb(0x5E, 0x45, 0x3D),
                selected: Color32::from_rgb(0x4F, 0xC3, 0xAE),
                alert: Color32::from_rgb(0xFF, 0xC6, 0x5C),
                text: Color32::from_rgb(0xEF, 0xDA, 0xD0),
            },
            (CopperRose, Light) => ThemeColors {
                node: Color32::from_rgb(0x9C, 0x4A, 0x3C),
                segment: Color32::from_rgb(0xD9, 0xB8, 0xAE),
                selected: Color32::from_rgb(0x2F, 0x7A, 0x6B),
                alert: Color32::from_rgb(0xE0, 0xA2, 0x3C),
                text: Color32::from_rgb(0x3D, 0x2E, 0x29),
            },

            (LimeCircuit, Dark) => ThemeColors {
                node: Color32::from_rgb(0xA8, 0xE0, 0x5F),
                segment: Color32::from_rgb(0x44, 0x52, 0x30),
                selected: Color32::from_rgb(0xA7, 0x8B, 0xFA),
                alert: Color32::from_rgb(0xFF, 0x85, 0x52),
                text: Color32::from_rgb(0xDC, 0xEA, 0xC0),
            },
            (LimeCircuit, Light) => ThemeColors {
                node: Color32::from_rgb(0x4D, 0x7A, 0x1F),
                segment: Color32::from_rgb(0xB9, 0xC7, 0x9A),
                selected: Color32::from_rgb(0x7B, 0x3F, 0xE4),
                alert: Color32::from_rgb(0xE8, 0x5D, 0x2E),
                text: Color32::from_rgb(0x2E, 0x33, 0x20),
            },

            (CoralReef, Dark) => ThemeColors {
                node: Color32::from_rgb(0xFF, 0x8B, 0x73),
                segment: Color32::from_rgb(0x38, 0x65, 0x60),
                selected: Color32::from_rgb(0x5C, 0xA8, 0xE0),
                alert: Color32::from_rgb(0xFF, 0xC1, 0x5E),
                text: Color32::from_rgb(0xD6, 0xED, 0xE8),
            },
            (CoralReef, Light) => ThemeColors {
                node: Color32::from_rgb(0xD6, 0x5A, 0x45),
                segment: Color32::from_rgb(0xA8, 0xD4, 0xCE),
                selected: Color32::from_rgb(0x1D, 0x5C, 0x9E),
                alert: Color32::from_rgb(0xF2, 0xA9, 0x3C),
                text: Color32::from_rgb(0x33, 0x40, 0x3E),
            },

            (GraphiteMono, Dark) => ThemeColors {
                node: Color32::from_rgb(0xD6, 0xD6, 0xD2),
                segment: Color32::from_rgb(0x4A, 0x4A, 0x46),
                selected: Color32::from_rgb(0x4F, 0xB3, 0xF5),
                alert: Color32::from_rgb(0xFF, 0x6B, 0x5C),
                text: Color32::from_rgb(0xE8, 0xE8, 0xE4),
            },
            (GraphiteMono, Light) => ThemeColors {
                node: Color32::from_rgb(0x3A, 0x3A, 0x3A),
                segment: Color32::from_rgb(0xB8, 0xB8, 0xB4),
                selected: Color32::from_rgb(0x1F, 0x8F, 0xE0),
                alert: Color32::from_rgb(0xE0, 0x48, 0x3A),
                text: Color32::from_rgb(0x23, 0x23, 0x23),
            },

            (PlumStatic, Dark) => ThemeColors {
                node: Color32::from_rgb(0xC9, 0x94, 0xBB),
                segment: Color32::from_rgb(0x4A, 0x3A, 0x45),
                selected: Color32::from_rgb(0x52, 0xC9, 0x9A),
                alert: Color32::from_rgb(0xFF, 0xA0, 0x5C),
                text: Color32::from_rgb(0xEB, 0xD9, 0xE5),
            },
            (PlumStatic, Light) => ThemeColors {
                node: Color32::from_rgb(0x6B, 0x3B, 0x5E),
                segment: Color32::from_rgb(0xC7, 0xAE, 0xC0),
                selected: Color32::from_rgb(0x2E, 0x8B, 0x6E),
                alert: Color32::from_rgb(0xE0, 0x79, 0x3D),
                text: Color32::from_rgb(0x36, 0x2B, 0x33),
            },

            (SandstoneTrail, Dark) => ThemeColors {
                node: Color32::from_rgb(0xC9, 0xAD, 0x72),
                segment: Color32::from_rgb(0x4E, 0x45, 0x30),
                selected: Color32::from_rgb(0x4F, 0xA8, 0xCC),
                alert: Color32::from_rgb(0xF0, 0x70, 0x8A),
                text: Color32::from_rgb(0xE6, 0xD9, 0xB8),
            },
            (SandstoneTrail, Light) => ThemeColors {
                node: Color32::from_rgb(0x6B, 0x5A, 0x3A),
                segment: Color32::from_rgb(0xDC, 0xCB, 0xA0),
                selected: Color32::from_rgb(0x2A, 0x6E, 0x8C),
                alert: Color32::from_rgb(0xD1, 0x49, 0x5B),
                text: Color32::from_rgb(0x3A, 0x31, 0x21),
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
    /// Color used for notification and alert animations.
    pub alert: Color32,
    /// Color used for node names and map labels.
    pub text: Color32,
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
///                 text: Color32::BLACK,
///             },
///             ColorMode::Dark => ThemeColors {
///                 node: Color32::WHITE,
///                 segment: Color32::LIGHT_GRAY,
///                 selected: Color32::YELLOW,
///                 alert: Color32::YELLOW,
///                 text: Color32::WHITE,
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
/// Holds the non-palette visual configuration -- stroke width, font,
/// background -- that combines with the active [`MapTheme`]'s
/// [`ThemeColors`] when the widget paints. `Style` itself carries no color:
/// every color the widget paints with (node fill, connection lines, alerts,
/// selection, text) is resolved live from the installed [`MapTheme`] for the
/// current [`ColorMode`] -- see [`Map::set_theme`](super::Map::set_theme) --
/// instead of living here as a copy that would need to be kept in sync.
///
/// Multiplying or dividing a `Style` by a number scales
/// [`line_width`](Self::line_width) and the font size; the widget uses this
/// to scale the active style with the current zoom factor. Fields that are
/// `None` are left untouched by those operators.
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
    /// Background color of the map canvas.
    pub background_color: Color32,
}

impl Style {
    /// Creates a fully transparent style with no line or font.
    pub fn new() -> Self {
        Style {
            line_width: None,
            font: None,
            background_color: Color32::TRANSPARENT,
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

    const ALL_THEMES: [Theme; 14] = [
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
        assert_eq!(
            Theme::SolarAmber.colors(ColorMode::Light),
            ThemeColors {
                node: Color32::from_rgb(0xF0, 0xC2, 0x4C),
                segment: Color32::from_rgb(0x5A, 0x4E, 0x2E),
                selected: Color32::from_rgb(0x4F, 0xBF, 0x9E),
                alert: Color32::from_rgb(0xE8, 0x65, 0x4A),
                text: Color32::from_rgb(0xEF, 0xE4, 0xC4),
            }
        );
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

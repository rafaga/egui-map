//! The names the map gives to assistive technology.
//!
//! egui builds an [AccessKit](https://accesskit.dev) tree from the widgets of
//! every frame, and a screen reader walks it. The map is painted by hand, so
//! without help its controls would show up there as anonymous widgets. The
//! widget names four things:
//!
//! - the map itself, a `Canvas` named [`AccessibilityLabels::map`];
//! - the zoom slider, a `Slider` named [`AccessibilityLabels::zoom_slider`]
//!   whose value is the current zoom;
//! - the minimap, a `Canvas` named [`AccessibilityLabels::minimap`];
//! - every node on screen, a `Button` named after the node (or
//!   [`AccessibilityLabels::unnamed_node`] and its id), which Tab reaches and
//!   Enter, Space or a screen reader activates; see
//!   [`Map::activated_node`](super::Map::activated_node).
//!
//! The names are plain text in English by default. An application that is
//! translated sets its own through
//! [`MapSettings::accessibility`](super::objects::MapSettings::accessibility):
//!
//! ```
//! use egui_map::map::Map;
//!
//! let mut map = Map::new();
//! map.settings.accessibility.map = "Mapa de la galaxia".to_owned();
//! map.settings.accessibility.zoom_slider = "Zoom".to_owned();
//! map.settings.accessibility.minimap = "Minimapa".to_owned();
//! ```

/// The accessible names of the controls of the map; see the
/// [module](self) docs.
///
/// Reached as
/// [`MapSettings::accessibility`](super::objects::MapSettings::accessibility).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccessibilityLabels {
    /// Name of the map itself. `"Map"` by default.
    pub map: String,
    /// Name of the zoom slider. `"Zoom"` by default.
    pub zoom_slider: String,
    /// Name of the minimap. `"Minimap"` by default.
    pub minimap: String,
    /// What a node without a [`name`](super::objects::MapPoint::name) is
    /// called, followed by its id: `"Node 42"`. `"Node"` by default.
    pub unnamed_node: String,
}

impl Default for AccessibilityLabels {
    fn default() -> Self {
        Self {
            map: "Map".to_owned(),
            zoom_slider: "Zoom".to_owned(),
            minimap: "Minimap".to_owned(),
            unnamed_node: "Node".to_owned(),
        }
    }
}

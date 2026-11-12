//! Layers: custom drawing the map paints for you, at a place of your choice
//! in its paint order -- a heatmap, an ownership overlay, a range ring.
//!
//! A [`LayerTemplate`] is the fourth template of the family, after
//! [`NodeTemplate`](super::objects::NodeTemplate),
//! [`SegmentTemplate`](super::objects::SegmentTemplate) and
//! [`LabelTemplate`](super::objects::LabelTemplate). It is installed with
//! [`Map::add_layer`](super::Map::add_layer) in one [`LayerSlot`]. The map
//! knows nothing about what a layer draws or the data behind it: the layer
//! keeps its own data (e.g. in an `Rc<RefCell<..>>` shared with the
//! application) and the map only hands it what it needs to paint.

use crate::map::objects::MapPoint;
use crate::map::outline::NodeOutline;
use crate::map::theme::ThemeColors;
use egui::{Painter, Pos2, Rect};
use std::fmt;
use std::rc::Rc;

/// Where a layer is painted, from the back of the map to the front.
///
/// The order of a frame is: region labels, [`BelowLines`](Self::BelowLines),
/// free labels, connection lines, [`BelowNodes`](Self::BelowNodes), the
/// nodes (with [`OverNodeBody`](Self::OverNodeBody) inside each), the
/// markers and [`AboveNodes`](Self::AboveNodes). Layers of one slot are
/// painted by ascending `z`, and in the order they were added when the `z` is
/// equal.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LayerSlot {
    /// Over the region labels, under the connection lines.
    BelowLines,
    /// Over the connection lines, under the nodes. Nodes drawn with an opaque
    /// background hide what is painted here.
    #[default]
    BelowNodes,
    /// Inside each node, over its background and under its label.
    ///
    /// Painted by [`NodeLayers::paint`], which a [`NodeTemplate`] calls from
    /// its `node_ui` at the point it wants the layers (see
    /// [`NodeContext::layers`](super::objects::NodeContext::layers)). A map
    /// without a node template, or a template that never calls it, paints
    /// nothing for this slot. The layer draws through
    /// [`LayerTemplate::node_ui`].
    ///
    /// [`NodeTemplate`]: super::objects::NodeTemplate
    OverNodeBody,
    /// Over the nodes and the markers.
    AboveNodes,
}

/// A node as a layer sees it.
///
/// `#[non_exhaustive]`, like the contexts, so a field can be added without
/// breaking [`LayerTemplate`]. It is no longer `Copy`: it owns the node's
/// [`NodeOutline`].
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct LayerPoint<'a> {
    /// The node's screen position: already scaled by the zoom and translated
    /// to the viewport origin.
    pub position: Pos2,
    /// The node: its id, name, coordinates and connections.
    pub point: &'a MapPoint,
    /// The node's shape as the node template reports it
    /// ([`NodeTemplate::outline`](super::objects::NodeTemplate::outline)), or
    /// the circle of the built-in node (`4 * zoom`) when the map has none. A
    /// layer that rings, highlights or shades a node follows this shape
    /// (`outline.grown(SELECTION_GAP).stroke_shape(..)`) instead of drawing a
    /// circle that only fits round nodes.
    pub outline: NodeOutline,
}

/// The context passed to [`LayerTemplate::layer_ui`].
///
/// `#[non_exhaustive]`, like the contexts of the other templates, so a field
/// can be added without breaking [`LayerTemplate`].
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct LayerContext<'a> {
    /// The area the map paints into, in screen coordinates.
    pub rect: Rect,
    /// Multiply every size in map units by this.
    pub zoom: f32,
    /// The active theme's palette for the current color mode.
    pub theme: ThemeColors,
    /// Seconds since the app started (egui's clock), for animating.
    pub time: f32,
    /// The nodes inside the viewport (the ones the map paints this frame).
    pub points: &'a [LayerPoint<'a>],
}

/// The context passed to [`LayerTemplate::node_ui`].
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct LayerNodeContext<'a> {
    /// The node's box as the template draws it (the outline it reported).
    pub outline: &'a NodeOutline,
    /// The node being painted.
    pub point: &'a MapPoint,
    /// The node's screen position.
    pub position: Pos2,
    /// Multiply every size in map units by this.
    pub zoom: f32,
    /// The active theme's palette for the current color mode.
    pub theme: ThemeColors,
    /// Seconds since the app started (egui's clock), for animating.
    pub time: f32,
}

/// Draws a layer. Both methods do nothing by default: implement the one the
/// layer's [`LayerSlot`] calls.
pub trait LayerTemplate {
    /// Draws the layer once per frame. Called for the layers of
    /// [`LayerSlot::BelowLines`], [`BelowNodes`](LayerSlot::BelowNodes) and
    /// [`AboveNodes`](LayerSlot::AboveNodes). `painter` is clipped to the map.
    fn layer_ui(&self, _painter: &Painter, _ctx: LayerContext) {}

    /// Draws the layer inside one node. Called, for every node the map
    /// paints, by [`NodeLayers::paint`] for the layers of
    /// [`LayerSlot::OverNodeBody`].
    fn node_ui(&self, _painter: &Painter, _ctx: LayerNodeContext) {}
}

/// The [`LayerSlot::OverNodeBody`] layers of one node, for its template to
/// paint where it wants them.
#[derive(Clone, Copy)]
pub struct NodeLayers<'a> {
    layers: &'a [Rc<dyn LayerTemplate>],
    point: &'a MapPoint,
    position: Pos2,
    zoom: f32,
    theme: ThemeColors,
    time: f32,
}

impl fmt::Debug for NodeLayers<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NodeLayers")
            .field("layers", &self.layers.len())
            .field("node", &self.point.get_id())
            .finish()
    }
}

impl<'a> NodeLayers<'a> {
    pub(crate) fn new(
        layers: &'a [Rc<dyn LayerTemplate>],
        point: &'a MapPoint,
        position: Pos2,
        zoom: f32,
        theme: ThemeColors,
        time: f32,
    ) -> Self {
        Self {
            layers,
            point,
            position,
            zoom,
            theme,
            time,
        }
    }

    /// Whether no layer is installed in [`LayerSlot::OverNodeBody`], so a
    /// template can skip the work of computing the `outline`.
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// Paints the layers over the node's body, in the shape of `outline`.
    pub fn paint(&self, painter: &Painter, outline: &NodeOutline) {
        for layer in self.layers {
            layer.node_ui(
                painter,
                LayerNodeContext {
                    outline,
                    point: self.point,
                    position: self.position,
                    zoom: self.zoom,
                    theme: self.theme,
                    time: self.time,
                },
            );
        }
    }
}

/// A layer installed in a map.
#[derive(Clone)]
pub(crate) struct LayerEntry {
    pub(crate) id: String,
    pub(crate) slot: LayerSlot,
    pub(crate) z: i32,
    pub(crate) visible: bool,
    pub(crate) template: Rc<dyn LayerTemplate>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::theme::{ColorMode, Theme};
    use std::cell::Cell;

    struct Counter {
        frames: Cell<u32>,
        nodes: Cell<u32>,
    }

    impl LayerTemplate for Counter {
        fn layer_ui(&self, _painter: &Painter, ctx: LayerContext) {
            self.frames.set(self.frames.get() + 1);
            assert!(ctx.zoom > 0.0);
        }
        fn node_ui(&self, _painter: &Painter, _ctx: LayerNodeContext) {
            self.nodes.set(self.nodes.get() + 1);
        }
    }

    #[test]
    fn the_default_slot_is_below_the_nodes() {
        assert_eq!(LayerSlot::default(), LayerSlot::BelowNodes);
    }

    #[test]
    fn node_layers_without_layers_paint_nothing() {
        let point = MapPoint::new(1, [0.0, 0.0]);
        let none: Vec<Rc<dyn LayerTemplate>> = Vec::new();
        let layers = NodeLayers::new(
            &none,
            &point,
            Pos2::ZERO,
            1.0,
            Theme::default().colors(ColorMode::Dark),
            0.0,
        );
        assert!(layers.is_empty());
    }

    #[test]
    fn node_layers_call_every_layer_once_per_node() {
        let counter = Rc::new(Counter {
            frames: Cell::new(0),
            nodes: Cell::new(0),
        });
        let point = MapPoint::new(1, [0.0, 0.0]);
        let shared: Vec<Rc<dyn LayerTemplate>> = vec![counter.clone(), counter.clone()];
        let layers = NodeLayers::new(
            &shared,
            &point,
            Pos2::ZERO,
            1.0,
            Theme::default().colors(ColorMode::Dark),
            0.0,
        );
        let context = egui::Context::default();
        let painter = Painter::new(
            context,
            egui::LayerId::background(),
            Rect::from_min_size(Pos2::ZERO, egui::vec2(10.0, 10.0)),
        );
        let outline = NodeOutline::Circle {
            center: Pos2::ZERO,
            radius: 4.0,
        };
        layers.paint(&painter, &outline);
        assert!(!layers.is_empty());
        assert_eq!(counter.nodes.get(), 2);
        assert_eq!(counter.frames.get(), 0);
    }
}

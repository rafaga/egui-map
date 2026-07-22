//! # egui-map
//!
//! An [`egui`](https://docs.rs/egui) widget that renders an interactive 2D map
//! on screen.
//!
//! The [`map::Map`] widget displays a set of nodes connected by lines, with
//! support for:
//!
//! - Panning (click and drag) and zooming (mouse wheel; hold `Ctrl` — or `Cmd`
//!   on macOS — to zoom faster).
//! - Spatial indexing of nodes through a kd-tree, so only the nodes inside the
//!   current viewport are painted each frame.
//! - Node names and free-floating text labels with configurable visibility
//!   rules (see [`map::objects::VisibilitySetting`]).
//! - Pulsing notifications and blinking markers attached to nodes.
//! - Custom node rendering and right-click context menus through the
//!   [`map::objects::NodeTemplate`] and [`map::objects::ContextMenuManager`]
//!   traits.
//! - Independent light and dark themes (see [`map::objects::MapSettings`]).
//!
//! ## Quick start
//!
//! ```no_run
//! use egui_map::map::Map;
//! use egui_map::map::objects::{MapPoint, RawPoint};
//! use std::collections::HashMap;
//!
//! // Build the node set, keyed by node id.
//! let mut points: HashMap<usize, MapPoint> = HashMap::new();
//! points.insert(1, MapPoint::new(1, RawPoint::new(0.0, 0.0)));
//! points.insert(2, MapPoint::new(2, RawPoint::new(100.0, 50.0)));
//!
//! let mut map = Map::new();
//! map.add_hashmap_points(points);
//!
//! // Then, on every frame of your egui update loop:
//! // ui.add(&mut map);
//! ```
//!
//! ## Crate features
//!
//! - `puffin`: instruments the widget's hot paths with the
//!   [`puffin`](https://crates.io/crates/puffin) profiler.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod map;

//! Tutorial 3: changing the map while it is on screen.
//!
//! Nodes and lines are added and removed with `insert_point`, `remove_point`
//! and `insert_line` while the map is showing, and the view never jumps. Press
//! "Add node" to grow the network and click a node to remove it. Read it top
//! to bottom, the steps are numbered. `docs/tutorial-3-dynamic-nodes.md` has
//! the same tutorial in prose.
//!
//! Run with: cargo run --example tutorial_dynamic

use eframe::egui;
use egui_map::map::Map;
use egui_map::map::objects::{MapPoint, MapSegment, VisibilitySetting};
use std::collections::BTreeMap;

/// The map and a copy of its nodes, which is what lets us add a line to a
/// node that is already on the map (the map has no way to hand a node out).
struct Network {
    map: Map,
    nodes: BTreeMap<usize, MapPoint>,
    next_id: usize,
}

impl Network {
    fn new() -> Self {
        let mut map = Map::new();
        map.settings.node_text_visibility = VisibilitySetting::Hover;
        map.settings.style.line_width = Some(2.0);
        let mut network = Self {
            map,
            nodes: BTreeMap::new(),
            next_id: 1,
        };
        for _ in 0..8 {
            network.add_node();
        }
        network
    }

    /// 2. Add a node. It is placed on a spiral so it never lands on another
    ///    one, and joined to the node added before it.
    fn add_node(&mut self) {
        let id = self.next_id;
        self.next_id += 1;
        let angle = id as f32 * 2.399_963;
        let radius = 45.0 * (id as f32).sqrt();
        let mut point = MapPoint::new(id, [radius * angle.cos(), radius * angle.sin()]);
        point.set_name(format!("Node {id}"));

        // A line lives in the `connections` of BOTH of its nodes: that is what
        // lets `remove_point` take the line away together with either of them.
        if let Some(previous) = self.nodes.values().next_back().cloned() {
            let line_id = (previous.get_id(), id);
            let segment = MapSegment::new(line_id, previous.coords, point.coords);
            let mut previous = previous;
            previous.connections.push(line_id);
            point.connections.push(line_id);
            self.map.insert_point(previous.clone());
            self.nodes.insert(previous.get_id(), previous);
            self.map.insert_line(segment);
        }
        self.map.insert_point(point.clone());
        self.nodes.insert(id, point);
    }

    /// 3. Remove a node. The map removes its lines and forgets it, and we
    ///    forget the lines in the neighbours' copies.
    fn remove_node(&mut self, id: usize) {
        if let Some(removed) = self.map.remove_point(id) {
            self.nodes.remove(&id);
            for neighbour in self.nodes.values_mut() {
                neighbour
                    .connections
                    .retain(|line| !removed.connections.contains(line));
            }
        }
    }
}

fn main() -> eframe::Result<()> {
    // 1. A `Network` holds the map between frames, as in tutorial 1.
    let mut network = Network::new();

    eframe::run_ui_native(
        "egui-map: tutorial 3, dynamic nodes",
        eframe::NativeOptions::default(),
        move |ui, _frame| {
            egui::Panel::top("controls").show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Add node").clicked() {
                        network.add_node();
                    }
                    ui.label(format!(
                        "{} nodes. Click one to remove it.",
                        network.nodes.len()
                    ));
                });
            });
            egui::CentralPanel::default().show(ui, |ui| {
                // 4. `ui.add` returns the response of the map: ask it for
                //    clicks, and the map for the node under the pointer.
                let response = ui.add(&mut network.map);
                if response.clicked()
                    && let Some(id) = network.map.hovered_node()
                {
                    network.remove_node(id);
                }
            });
        },
    )
}

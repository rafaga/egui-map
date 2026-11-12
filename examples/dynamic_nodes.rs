//! Dynamic nodes example: a network of about 30 nodes that grows and shrinks
//! while it is on screen, without the view ever jumping.
//!
//! The network is not one piece: it has rings (circuits, some with a chord),
//! three of them joined by dashed bridges into a bigger circuit, a ring that is
//! on its own and two nodes that join nothing.
//!
//! - **Add node**: `Map::insert_point` and `Map::insert_line` put one node and
//!   its lines on the map. Unlike `add_points`/`add_lines` they do not replace
//!   what is there nor center the view again, so the map goes on where it was.
//!   The new node hangs from a node of the network, and when another is close
//!   it joins that one too, which closes a small circuit.
//! - **Add lone node**: a node that joins nothing.
//! - **Add ring**: five nodes and the five lines of a circuit, in a free place.
//! - **Remove** (click on a node, or the button for a random one):
//!   `Map::remove_point` takes the node out with the lines listed in its
//!   `connections`; the nodes at their other ends forget those lines too.
//! - **Auto** adds and removes by itself, about twice a second.
//!
//! The node keeps its `connections` in the example too, to push the id of each
//! new line into both of its ends: that list is what `remove_point` follows.
//! A node that is already on the map is replaced by inserting it again, which
//! costs nothing while its position is the same.
//!
//! Run with: cargo run --example dynamic_nodes

use eframe::egui;
use egui_map::map::Map;
use egui_map::map::objects::{MapPoint, MapSegment};
use std::collections::{BTreeMap, HashSet};
use std::f32::consts::TAU;

/// The most nodes the buttons and Auto make: past it the example only removes.
const MAX_NODES: usize = 60;

/// A small deterministic random generator (xorshift), so every run of the
/// example starts with the same network.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    /// A number from 0.0 up to 1.0.
    fn unit(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1_u64 << 24) as f32
    }

    /// A number from 0 up to `count` (`count` must not be 0).
    fn below(&mut self, count: usize) -> usize {
        (self.next() % count as u64) as usize
    }
}

/// The network and the map that shows it.
struct Network {
    map: Map,
    /// The nodes as the map has them. The map is given a copy each time a
    /// node changes (a new line in its `connections`).
    nodes: BTreeMap<usize, MapPoint>,
    /// The lines, by id.
    lines: HashSet<(usize, usize)>,
    rng: Rng,
    next_id: usize,
    auto: bool,
    last_step: f64,
}

impl Network {
    /// The least distance between two nodes, in map units.
    const SPACING: f32 = 58.0;

    fn new() -> Self {
        let mut network = Self {
            map: Map::new(),
            nodes: BTreeMap::new(),
            lines: HashSet::new(),
            rng: Rng(0x9E37_79B9_7F4A_7C15),
            next_id: 1,
            auto: false,
            last_step: 0.0,
        };
        let mut segments = Vec::new();
        // Five rings around the middle: three joined by bridges, one on its
        // own and a small one on its own.
        let sizes = [6, 6, 6, 6, 4];
        let mut rings: Vec<Vec<usize>> = Vec::new();
        for (index, size) in sizes.iter().enumerate() {
            let angle = TAU * index as f32 / sizes.len() as f32;
            let centre = [angle.cos() * 330.0, angle.sin() * 330.0];
            rings.push(network.ring(centre, *size, 75.0, &mut segments));
        }
        // A chord across two of the rings: a circuit inside a circuit.
        segments.extend(network.link(rings[0][0], rings[0][3], false));
        segments.extend(network.link(rings[2][1], rings[2][4], false));
        // The bridges close a circuit between the first three rings.
        segments.extend(network.link(rings[0][2], rings[1][5], true));
        segments.extend(network.link(rings[1][2], rings[2][5], true));
        segments.extend(network.link(rings[2][2], rings[0][5], true));
        // Two nodes that join nothing.
        for spot in [[-20.0, 20.0], [460.0, -420.0]] {
            network.place(spot);
        }
        network
            .map
            .add_points(network.nodes.values().cloned().collect());
        network.map.add_lines(segments);
        network
    }

    /// A node at `at`, on the map's list but not yet shown.
    fn place(&mut self, at: [f32; 2]) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let mut point = MapPoint::new(id, at);
        point.set_name(format!("N{id}"));
        self.nodes.insert(id, point);
        id
    }

    /// `count` nodes on a circle of `radius` around `centre`, each joined to
    /// the next: the lines are pushed to `segments`.
    fn ring(
        &mut self,
        centre: [f32; 2],
        count: usize,
        radius: f32,
        segments: &mut Vec<MapSegment>,
    ) -> Vec<usize> {
        let ids: Vec<usize> = (0..count)
            .map(|step| {
                let angle = TAU * step as f32 / count as f32;
                self.place([
                    centre[0] + angle.cos() * radius,
                    centre[1] + angle.sin() * radius,
                ])
            })
            .collect();
        for step in 0..count {
            segments.extend(self.link(ids[step], ids[(step + 1) % count], false));
        }
        ids
    }

    /// Joins two nodes: the line goes in the `connections` of both and is
    /// returned, not yet on the map. `None` if they were already joined.
    /// A bridge is dashed, curved and has a text.
    fn link(&mut self, from: usize, to: usize, bridge: bool) -> Option<MapSegment> {
        if from == to || self.lines.contains(&(from, to)) || self.lines.contains(&(to, from)) {
            return None;
        }
        let id = (from, to);
        self.lines.insert(id);
        let (start, end) = (self.nodes[&from].coords, self.nodes[&to].coords);
        for end in [from, to] {
            if let Some(node) = self.nodes.get_mut(&end) {
                node.connections.push(id);
            }
        }
        Some(if bridge {
            MapSegment::curved(id, start, end, None)
                .dashed()
                .with_label("bridge")
        } else {
            MapSegment::straight(id, start, end)
        })
    }

    /// Joins two nodes that are already on the map.
    fn connect(&mut self, from: usize, to: usize) {
        if let Some(segment) = self.link(from, to, false) {
            // Both ends changed their `connections`: insert them again.
            for end in [from, to] {
                self.map.insert_point(self.nodes[&end].clone());
            }
            self.map.insert_line(segment);
        }
    }

    /// Whether a node at `at` would be at least `spacing` from every node.
    fn is_free(&self, at: [f32; 2], spacing: f32) -> bool {
        self.nodes
            .values()
            .all(|node| (node.coords[0] - at[0]).hypot(node.coords[1] - at[1]) >= spacing)
    }

    /// A random node's id.
    fn any_node(&mut self) -> Option<usize> {
        if self.nodes.is_empty() {
            return None;
        }
        let index = self.rng.below(self.nodes.len());
        self.nodes.keys().nth(index).copied()
    }

    /// A free place near `around`, if a few tries find one.
    fn free_spot_near(&mut self, around: [f32; 2]) -> Option<[f32; 2]> {
        (0..24).find_map(|_| {
            let angle = self.rng.unit() * TAU;
            let distance = 70.0 + self.rng.unit() * 50.0;
            let at = [
                around[0] + angle.cos() * distance,
                around[1] + angle.sin() * distance,
            ];
            self.is_free(at, Self::SPACING).then_some(at)
        })
    }

    /// A node that hangs from a node of the network, and from a second one if
    /// there is one close: that closes a circuit.
    fn add_node(&mut self) {
        if self.nodes.len() >= MAX_NODES {
            return;
        }
        let Some(anchor) = self.any_node() else {
            return;
        };
        let Some(at) = self.free_spot_near(self.nodes[&anchor].coords) else {
            return;
        };
        let id = self.place(at);
        self.map.insert_point(self.nodes[&id].clone());
        self.connect(anchor, id);
        let near = self
            .nodes
            .values()
            .filter(|node| node.get_id() != id && node.get_id() != anchor)
            .map(|node| {
                let distance = (node.coords[0] - at[0]).hypot(node.coords[1] - at[1]);
                (node.get_id(), distance)
            })
            .filter(|(_, distance)| *distance < 140.0)
            .min_by(|a, b| a.1.total_cmp(&b.1));
        if let Some((other, _)) = near {
            self.connect(id, other);
        }
    }

    /// A random free place away from every node, in the area of the network.
    fn free_spot(&mut self, spacing: f32) -> Option<[f32; 2]> {
        (0..40).find_map(|_| {
            let at = [
                (self.rng.unit() - 0.5) * 1100.0,
                (self.rng.unit() - 0.5) * 900.0,
            ];
            self.is_free(at, spacing).then_some(at)
        })
    }

    /// A node that joins nothing.
    fn add_lone_node(&mut self) {
        if self.nodes.len() >= MAX_NODES {
            return;
        }
        if let Some(at) = self.free_spot(Self::SPACING * 1.5) {
            let id = self.place(at);
            self.map.insert_point(self.nodes[&id].clone());
        }
    }

    /// Five nodes joined in a circuit, in a free place.
    fn add_ring(&mut self) {
        if self.nodes.len() + 5 > MAX_NODES {
            return;
        }
        let Some(centre) = self.free_spot(200.0) else {
            return;
        };
        let mut segments = Vec::new();
        let ids = self.ring(centre, 5, 55.0, &mut segments);
        for id in ids {
            self.map.insert_point(self.nodes[&id].clone());
        }
        for segment in segments {
            self.map.insert_line(segment);
        }
    }

    /// Takes a node and its lines off the map.
    fn remove_node(&mut self, id: usize) {
        let Some(removed) = self.map.remove_point(id) else {
            return;
        };
        self.nodes.remove(&id);
        // The map forgot the lines of the node, and the other ends of them in
        // the nodes it has; the copies here do the same.
        for line in removed.connections {
            self.lines.remove(&line);
            for node in self.nodes.values_mut() {
                node.connections.retain(|known| *known != line);
            }
        }
    }

    fn remove_random(&mut self) {
        if let Some(id) = self.any_node() {
            self.remove_node(id);
        }
    }

    /// What Auto does each time: grow while the network is small, shrink while
    /// it is big, and in between do either.
    fn step(&mut self) {
        let grow = match self.nodes.len() {
            0..=24 => true,
            40.. => false,
            _ => self.rng.below(100) < 55,
        };
        if !grow {
            self.remove_random();
        } else {
            match self.rng.below(10) {
                0 => self.add_ring(),
                1 => self.add_lone_node(),
                _ => self.add_node(),
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("controls").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui.button("Add node").clicked() {
                    self.add_node();
                }
                if ui.button("Add lone node").clicked() {
                    self.add_lone_node();
                }
                if ui.button("Add ring").clicked() {
                    self.add_ring();
                }
                if ui.button("Remove random").clicked() {
                    self.remove_random();
                }
                ui.checkbox(&mut self.auto, "Auto");
                ui.separator();
                ui.label(format!(
                    "{} nodes, {} lines. Click a node to remove it.",
                    self.nodes.len(),
                    self.lines.len()
                ));
            });
        });
        let now = ui.input(|input| input.time);
        if self.auto {
            if now - self.last_step >= 0.5 {
                self.last_step = now;
                self.step();
            }
            ui.request_repaint_after(std::time::Duration::from_millis(100));
        }
        egui::CentralPanel::default().show(ui, |ui| {
            let response = ui.add(&mut self.map);
            if response.clicked()
                && let Some(id) = self.map.hovered_node()
            {
                self.remove_node(id);
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let mut network = Network::new();
    eframe::run_ui_native(
        "egui-map: dynamic nodes",
        eframe::NativeOptions::default(),
        move |ui, _frame| network.ui(ui),
    )
}

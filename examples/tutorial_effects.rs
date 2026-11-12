//! Tutorial 4: animations on nodes and lines.
//!
//! Every node and line can carry an effect: one that plays once and ends on
//! its own (`pulse`, `comet_once`) or one that lasts until it is cleared
//! (`halo`, `chevrons`). Click a node to pulse it; the controls change the
//! lasting ones. Read it top to bottom, the steps are numbered.
//! `docs/tutorial-4-effects.md` has the same tutorial in prose.
//!
//! Run with: cargo run --example tutorial_effects

use eframe::egui;
use egui_map::map::Map;
use egui_map::map::objects::{MapPoint, MapSegment, VisibilitySetting};
use std::time::Instant;

fn main() -> eframe::Result<()> {
    // A ring of six nodes, each joined to the next.
    let mut points = Vec::new();
    for id in 1..=6_usize {
        let angle = id as f32 * std::f32::consts::TAU / 6.0;
        let mut point = MapPoint::new(id, [150.0 * angle.cos(), 150.0 * angle.sin()]);
        point.set_name(format!("Node {id}"));
        points.push(point);
    }
    let mut segments = Vec::new();
    for from in 0..6 {
        let to = (from + 1) % 6;
        let line_id = (points[from].get_id(), points[to].get_id());
        segments.push(MapSegment::new(
            line_id,
            points[from].coords,
            points[to].coords,
        ));
        points[from].connections.push(line_id);
        points[to].connections.push(line_id);
    }
    let mut map = Map::new();
    map.add_points(points);
    map.add_lines(segments);
    map.settings.node_text_visibility = VisibilitySetting::Hover;
    map.settings.style.line_width = Some(2.0);

    let mut halo = false;
    let mut chevrons = false;

    eframe::run_ui_native(
        "egui-map: tutorial 4, effects",
        eframe::NativeOptions::default(),
        move |ui, _frame| {
            egui::Panel::top("controls").show(ui, |ui| {
                ui.horizontal(|ui| {
                    // 2. A lasting effect runs until `clear()`: turn it on and
                    //    off by what the checkbox says.
                    if ui.checkbox(&mut halo, "Halo on node 1").changed()
                        && let Some(node) = map.node(1)
                    {
                        if halo {
                            node.halo();
                        } else {
                            node.clear();
                        }
                    }
                    if ui.checkbox(&mut chevrons, "Chevrons on line 1-2").changed()
                        && let Some(line) = map.segment((1, 2))
                    {
                        if chevrons {
                            line.chevrons();
                        } else {
                            line.clear();
                        }
                    }
                    // 3. A one-off effect starts at the `Instant` you give
                    //    and stops by itself.
                    if ui.button("Send a comet along 1-2").clicked()
                        && let Some(line) = map.segment((1, 2))
                    {
                        line.comet_once(Instant::now());
                    }
                });
            });
            egui::CentralPanel::default().show(ui, |ui| {
                let response = ui.add(&mut map);
                // 1. A click on a node pulses it. `color()` is optional and
                //    comes first; the terminal method (`pulse`) attaches it.
                if response.clicked()
                    && let Some(id) = map.hovered_node()
                    && let Some(node) = map.node(id)
                {
                    node.pulse(Instant::now());
                }
            });
        },
    )
}

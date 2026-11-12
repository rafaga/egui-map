//! Tutorial 6: themes and region labels.
//!
//! The palette is a `Theme` you can swap at any time, and a `RegionLabel`
//! names an area of the map instead of a node. Read it top to bottom, the
//! steps are numbered. `docs/tutorial-6-themes-and-labels.md` has the same
//! tutorial in prose.
//!
//! Run with: cargo run --example tutorial_themes

use eframe::egui;
use egui_map::map::Map;
use egui_map::map::objects::{MapPoint, RegionLabel, VisibilitySetting};
use egui_map::map::theme::Theme;
use std::rc::Rc;

fn main() -> eframe::Result<()> {
    let mut map = Map::new();
    let mut points = Vec::new();
    for (id, name, x, y) in [
        (1, "Alpha", -200.0, -60.0),
        (2, "Beta", -120.0, 40.0),
        (3, "Gamma", 150.0, -80.0),
        (4, "Delta", 230.0, 30.0),
    ] {
        let mut point = MapPoint::new(id, [x, y]);
        point.set_name(name.to_string());
        points.push(point);
    }
    map.add_points(points);
    map.settings.node_text_visibility = VisibilitySetting::Hover;

    // 1. A region label sits at *map* coordinates, like a node, but unlike any
    //    other text it grows and shrinks with the zoom and is painted behind
    //    everything. `color: None` uses the theme's text color, faded.
    map.add_region_labels(vec![
        RegionLabel {
            text: "West".to_string(),
            center: egui::pos2(-160.0, 0.0),
            color: None,
        },
        RegionLabel {
            text: "East".to_string(),
            center: egui::pos2(190.0, -20.0),
            color: None,
        },
    ]);

    // 2. The built-in palettes are variants of `Theme`; `set_theme` takes
    //    effect on the next frame, in the light or dark mode egui is in.
    let themes = [
        Theme::SystemDefault,
        Theme::SlateOcean,
        Theme::NebulaViolet,
        Theme::TerminalGreen,
        Theme::EmberForge,
        Theme::ArticCyan,
    ];
    let mut chosen = 0;

    eframe::run_ui_native(
        "egui-map: tutorial 6, themes and region labels",
        eframe::NativeOptions::default(),
        move |ui, _frame| {
            egui::Panel::top("controls").show(ui, |ui| {
                ui.horizontal(|ui| {
                    let before = chosen;
                    egui::ComboBox::from_label("Theme")
                        .selected_text(format!("{:?}", themes[chosen]))
                        .show_ui(ui, |ui| {
                            for (index, theme) in themes.iter().enumerate() {
                                ui.selectable_value(&mut chosen, index, format!("{theme:?}"));
                            }
                        });
                    if chosen != before {
                        map.set_theme(Rc::new(themes[chosen]));
                    }
                    // 3. The minimap is a setting like the rest.
                    ui.checkbox(&mut map.settings.minimap.enabled, "Minimap");
                });
            });
            egui::CentralPanel::default().show(ui, |ui| {
                ui.add(&mut map);
            });
        },
    )
}

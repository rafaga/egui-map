use crate::map::objects::*;
use egui::{widgets::*, *};
use kdtree::distance::squared_euclidean;
use kdtree::KdTree;
use std::collections::hash_map::Entry;
//use rand::distributions::{Alphanumeric, Distribution};
//use rand::thread_rng;
use std::collections::HashMap;
use std::fmt::Error;
use std::time::Instant;
use crate::map::animation::AnimationPoint;

use self::animation::AnimationManager;

pub mod objects;
pub mod animation;
// This can by any object or point with its associated metadata
/// Struct that contains coordinates to help calculate nearest point in space

pub struct Map {
    zoom: f32,
    previous_zoom: f32,
    points: Option<HashMap<usize, MapPoint>>,
    lines: Vec<MapLine>,
    labels: Vec<MapLabel>,
    tree: Option<KdTree<f64, usize, [f64; 2]>>,
    visible_points: Vec<usize>,
    map_area: Rect,
    reference: MapBounds,
    current: MapBounds,
    style: egui::Style,
    current_index: usize,
    animon: AnimationManager,
    pub settings: MapSettings,
}

impl Default for Map {
    fn default() -> Self {
        Map::new()
    }
}

impl Widget for &mut Map {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        self.map_area = ui.available_rect_before_wrap();

        self.asign_visual_style(ui);

        let canvas = egui::Frame::canvas(ui.style());

        self.capture_mouse_events(ui);

        let inner_response = canvas.show(ui, |ui| {
            #[cfg(feature = "puffin")]
            puffin::profile_scope!("paint_map");

            //if ui_obj.is_rect_visible(self.map_area) {
            let (resp, paint) =
                ui.allocate_painter(self.map_area.size(), egui::Sense::click_and_drag());
            let vec = resp.drag_delta();
            if vec.length() != 0.0 {
                #[cfg(feature = "puffin")]
                puffin::profile_scope!("calculating_points_in_visible_area");

                let coords = (vec.to_pos2().x, vec.to_pos2().y);

                self.set_pos(self.current.pos.x - coords.0, self.current.pos.y - coords.1);
                self.calculate_visible_points();
            }
            let map_style = self.settings.styles[self.current_index].clone() * self.zoom;
            if self.zoom < self.settings.line_visible_zoom {
                // filling text settings
                let mut text_settings = TextSettings {
                    size: 12.00 * self.zoom * 2.00,
                    anchor: Align2::CENTER_CENTER,
                    family: FontFamily::Proportional,
                    text: String::new(),
                    position: Pos2::new(0.00, 0.00),
                    text_color: ui.visuals().text_color(),
                };
                for label in &self.labels {
                    text_settings.text = label.text.clone();
                    paint.text(
                        label.center,
                        Align2::CENTER_CENTER,
                        label.text.as_str(),
                        map_style.font.clone().unwrap(),
                        ui.visuals().text_color(),
                    );
                    self.paint_label(&paint, &text_settings);
                }
            }

            let vec_points = &self.visible_points;
            let hashm = &self.points;
            let factor = (
                self.map_area.center().x + self.map_area.min.x,
                self.map_area.center().y + self.map_area.min.y,
            );
            let min_point = Pos2::new(self.current.pos.x - factor.0, self.current.pos.y - factor.1);

            let _a = self.paint_map_lines(vec_points, hashm, &paint, &min_point);
            let _b = self.paint_map_points(vec_points, hashm, &paint, ui, &min_point, &resp);

            self.paint_sub_components(ui, self.map_area);

            if self.zoom != self.previous_zoom {
                #[cfg(feature = "puffin")]
                puffin::profile_scope!("calculating viewport with zoom");
                self.adjust_bounds();
                self.calculate_visible_points();
                self.previous_zoom = self.zoom;
            }

            self.hover_management(ui, &paint, &resp);

            // paint notification
            self.animon.animation_loop(ui);

            if cfg!(debug_assertions) {
                self.print_debug_info(paint, resp);
            }
            //}
        });
        inner_response.response
    }
}

impl Map {
    pub fn new() -> Self {
        let settings = MapSettings::default();
        Self {
            zoom: 1.0,
            previous_zoom: 1.0,
            map_area: Rect::NOTHING,
            tree: None,
            points: None,
            lines: Vec::new(),
            labels: Vec::new(),
            visible_points: Vec::new(),
            current: MapBounds::default(),
            reference: MapBounds::default(),
            animon: AnimationManager::new(),
            settings,
            current_index: 0,
            style: egui::Style::default(),
        }
    }

    fn calculate_visible_points(&mut self) {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("calculate_visible_points");
        if self.current.dist > 0.0 {
            if let Some(tree) = &self.tree {
                let center = [
                    (self.current.pos.x / self.zoom) as f64,
                    (self.current.pos.y / self.zoom) as f64,
                ];
                let radius = self.current.dist.powi(2);
                let vis_pos = tree.within(&center, radius, &squared_euclidean).unwrap();
                self.visible_points.clear();
                for point in vis_pos {
                    self.visible_points.push(*point.1);
                }
            }
        }
    }

    pub fn add_hashmap_points(&mut self, hash_map: HashMap<usize, MapPoint>) {
        let mut min = (f64::INFINITY, f64::INFINITY);
        let mut max = (f64::NEG_INFINITY, f64::NEG_INFINITY);
        let mut tree = KdTree::<f64, usize, [f64; 2]>::new(2);
        let mut h_map = hash_map.clone();
        for entry in h_map.iter_mut() {
            entry.1.coords[0] *= -1.0;
            entry.1.coords[1] *= -1.0;
            if entry.1.coords[0] < min.0 {
                min.0 = entry.1.coords[0];
            }
            if entry.1.coords[1] < min.1 {
                min.1 = entry.1.coords[1];
            }
            if entry.1.coords[0] > max.0 {
                max.0 = entry.1.coords[0];
            }
            if entry.1.coords[1] > max.1 {
                max.1 = entry.1.coords[1];
            }
            let _result = tree.add([entry.1.coords[0], entry.1.coords[1]], *entry.0);
            for line in &mut entry.1.lines {
                line[0] *= -1.0;
                line[1] *= -1.0;
                line[2] *= -1.0;
            }
        }
        self.reference.min = Pos2::new(min.0 as f32, min.1 as f32);
        self.reference.max = Pos2::new(max.0 as f32, max.1 as f32);
        self.points = Some(h_map);
        self.tree = Some(tree);
        let rect = Rect::from_min_max(self.reference.min, self.reference.max);
        self.reference.pos = rect.center();
        let dist_x =
            (self.map_area.right_bottom().x as f64 - self.map_area.left_top().x as f64) / 2.0;
        let dist_y =
            (self.map_area.right_bottom().y as f64 - self.map_area.left_top().y as f64) / 2.0;
        self.reference.dist = (dist_x.powi(2) + dist_y.powi(2) / 2.0).sqrt();
        self.current = self.reference.clone();
        self.calculate_visible_points();
    }

    pub fn set_pos(&mut self, x: f32, y: f32) {
        if x <= self.current.max.x
            && x >= self.current.min.x
            && y <= self.current.max.y
            && y >= self.current.min.y
        {
            self.current.pos = Pos2::new(x, y);
            self.reference.pos = Pos2::new(x / self.zoom, y / self.zoom);
        }
    }

    pub fn get_pos(self) -> Pos2 {
        self.current.pos
    }

    pub fn add_labels(&mut self, labels: Vec<MapLabel>) {
        let mut converted_labels = Vec::new();
        for mut label in labels {
            label.center[0] *= -1.00;
            label.center[1] *= -1.00;
            converted_labels.push(label);
        }
        self.labels = converted_labels;
    }

    pub fn add_lines(&mut self, lines: Vec<MapLine>) {
        let mut converted_lines = Vec::new();
        for mut line in lines {
            line.points[0] = line.points[0] * -1.00;
            line.points[1] = line.points[1] * -1.00;
            converted_lines.push(line);
        }
        self.lines = converted_lines;
    }

    fn adjust_bounds(&mut self) {
        self.current.max.x = self.reference.max.x * self.zoom;
        self.current.max.y = self.reference.max.y * self.zoom;
        self.current.min.x = self.reference.min.x * self.zoom;
        self.current.min.y = self.reference.min.y * self.zoom;
        self.current.dist = self.reference.dist / self.zoom as f64;
        self.set_pos(
            self.reference.pos.x * self.zoom,
            self.reference.pos.y * self.zoom,
        );
    }

    fn capture_mouse_events(&mut self, ui: &Ui) {
        // capture MouseWheel Event for Zoom control change
        ui.input(|x| {
            #[cfg(feature = "puffin")]
            puffin::profile_scope!("capture_mouse_events");

            if !x.events.is_empty() {
                for event in &x.events {
                    match event {
                        Event::MouseWheel {
                            unit: _,
                            delta,
                            modifiers,
                        } => {
                            #[cfg(target_os = "macos")]
                            let zoom_modifier = if modifiers.mac_cmd {
                                delta.y / 80.00
                            } else {
                                delta.y / 400.00
                            };

                            #[cfg(not(target_os = "macos"))]
                            let zoom_modifier = if modifiers.ctrl {
                                delta.y / 8.00
                            } else {
                                delta.y / 40.00
                            };

                            let mut pre_zoom = self.zoom + zoom_modifier;
                            if pre_zoom > self.settings.max_zoom {
                                pre_zoom = self.settings.max_zoom;
                            }
                            if pre_zoom < self.settings.min_zoom {
                                pre_zoom = self.settings.min_zoom;
                            }
                            self.zoom = pre_zoom;
                        }
                        _ => {
                            continue;
                        }
                    };
                }
            }
        });
    }

    pub fn set_zoom(mut self, value: f32) {
        if value >= self.settings.min_zoom && value <= self.settings.max_zoom {
            self.zoom = value;
        }
    }

    pub fn  get_zoom(self) -> f32 {
        self.zoom
    }

    fn asign_visual_style(&mut self, ui_obj: &mut Ui) {
        let style_index = ui_obj.visuals().dark_mode as usize;

        if self.current_index != style_index {
            #[cfg(feature = "puffin")]
            puffin::profile_scope!("asign_visual_style");

            self.current_index = style_index;
            self.style = ui_obj.style_mut().clone();
            self.style.visuals.extreme_bg_color =
                self.settings.styles[style_index].background_color;
            self.style.visuals.window_stroke = self.settings.styles[style_index].border.unwrap();
        }
    }

    fn print_debug_info(&mut self, paint: Painter, resp: Response) {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("printing debug data");

        let mut init_pos = Pos2::new(
            self.map_area.left_top().x + 10.00,
            self.map_area.left_top().y + 10.00,
        );
        let mut msg = "MIN:".to_string()
            + self.current.min.x.to_string().as_str()
            + ","
            + self.current.min.y.to_string().as_str();
        paint.debug_text(init_pos, Align2::LEFT_TOP, Color32::LIGHT_GREEN, msg);
        init_pos.y += 15.0;
        msg = "MAX:".to_string()
            + self.current.max.x.to_string().as_str()
            + ","
            + self.current.max.y.to_string().as_str();
        paint.debug_text(init_pos, Align2::LEFT_TOP, Color32::LIGHT_GREEN, msg);
        init_pos.y += 15.0;
        msg = "CUR:(".to_string()
            + self.current.pos.x.to_string().as_str()
            + ","
            + self.current.pos.y.to_string().as_str()
            + ")";
        paint.debug_text(init_pos, Align2::LEFT_TOP, Color32::LIGHT_GREEN, msg);
        init_pos.y += 15.0;
        msg = "DST:".to_string() + self.current.dist.to_string().as_str();
        paint.debug_text(init_pos, Align2::LEFT_TOP, Color32::LIGHT_GREEN, msg);
        init_pos.y += 15.0;
        msg = "ZOM:".to_string() + self.zoom.to_string().as_str();
        paint.debug_text(init_pos, Align2::LEFT_TOP, Color32::GREEN, msg);
        init_pos.y += 15.0;
        msg = "REC:(".to_string()
            + self.map_area.left_top().x.to_string().as_str()
            + ","
            + self.map_area.left_top().y.to_string().as_str()
            + "),("
            + self.map_area.right_bottom().x.to_string().as_str()
            + ","
            + self.map_area.right_bottom().y.to_string().as_str()
            + ")";
        paint.debug_text(init_pos, Align2::LEFT_TOP, Color32::LIGHT_GREEN, msg);
        if let Some(points) = &self.points {
            init_pos.y += 15.0;
            msg = "NUM:".to_string() + points.len().to_string().as_str();
            paint.debug_text(init_pos, Align2::LEFT_TOP, Color32::LIGHT_GREEN, msg);
        }
        if !self.visible_points.is_empty() {
            init_pos.y += 15.0;
            msg = "VIS:".to_string() + self.visible_points.len().to_string().as_str();
            paint.debug_text(init_pos, Align2::LEFT_TOP, Color32::LIGHT_GREEN, msg);
        }
        if let Some(pointer_pos) = resp.hover_pos() {
            init_pos.y += 15.0;
            msg = "HVR:".to_string()
                + pointer_pos.x.to_string().as_str()
                + ","
                + pointer_pos.y.to_string().as_str();
            paint.debug_text(init_pos, Align2::LEFT_TOP, Color32::LIGHT_BLUE, msg);
        }
        let vec = resp.drag_delta();
        if vec.length() != 0.0 {
            init_pos.y += 15.0;
            msg = "DRG:".to_string()
                + vec.to_pos2().x.to_string().as_str()
                + ","
                + vec.to_pos2().y.to_string().as_str();
            paint.debug_text(init_pos, Align2::LEFT_TOP, Color32::GOLD, msg);
        }
    }

    fn paint_sub_components(&mut self, ui_obj: &mut Ui, rect: Rect) {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("map_ui_paint_sub_components");
        let zoom_slider = egui::Slider::new(
            &mut self.zoom,
            self.settings.min_zoom..=self.settings.max_zoom,
        )
        .show_value(false)
        //.step_by(0.1)
        .orientation(SliderOrientation::Vertical);
        let mut pos1 = rect.right_top();
        let mut pos2 = rect.right_top();
        pos1.x -= 80.0;
        pos1.y += 120.0;
        pos2.x -= 50.0;
        pos2.y += 240.0;
        let sub_rect = egui::Rect::from_two_pos(pos1, pos2);
        ui_obj.allocate_ui_at_rect(sub_rect, |ui_obj| {
            ui_obj.add(zoom_slider);
        });
    }

    fn hover_management(&mut self, _ui_obj: &Ui, _paint: &Painter, resp: &Response) {
        if resp.secondary_clicked() {
            todo!();
        }
    }

    fn paint_map_points(
        &self,
        vec_points: &Vec<usize>,
        hashm: &Option<HashMap<usize, MapPoint>>,
        paint: &Painter,
        ui_obj: &Ui,
        min_point: &Pos2,
        resp: &Response,
    ) -> Result<(), ()> {
        let mut nearest_id = None;
        if hashm.is_none() {
            return Err(());
        }
        if vec_points.is_empty() {
            return Err(());
        }
        // detecting the nearest hover node
        if self.settings.node_text_visibility == VisibilitySetting::Hover && resp.hovered() {
            if let Some(point) = resp.hover_pos() {
                let hovered_map_point = Pos2::new(
                    (min_point.x + point.x) / self.zoom,
                    (min_point.y + point.y) / self.zoom,
                );
                if let Ok(nearest_node) = self.tree.as_ref().unwrap().nearest(
                    &[hovered_map_point.x as f64, hovered_map_point.y as f64],
                    1,
                    &squared_euclidean,
                ) {
                    nearest_id = Some(nearest_node.first().unwrap().1);
                }
            }
        }
        // filling text settings
        let mut text_settings = TextSettings {
            size: 12.00 * self.zoom,
            anchor: Align2::LEFT_BOTTOM,
            family: FontFamily::Proportional,
            text: String::new(),
            position: Pos2::new(0.00, 0.00),
            text_color: ui_obj.visuals().text_color(),
        };

        // Drawing Points
        for temp_point in vec_points {
            if let Some(system) = hashm.as_ref().unwrap().get(temp_point) {
                #[cfg(feature = "puffin")]
                puffin::profile_scope!("painting_points_m");
                let center = Pos2::new(
                    system.coords[0] as f32 * self.zoom,
                    system.coords[1] as f32 * self.zoom,
                );
                let viewport_point = Pos2::new(center.x - min_point.x, center.y - min_point.y);
                if self.zoom > self.settings.label_visible_zoom
                    && (self.settings.node_text_visibility == VisibilitySetting::Allways
                        || (self.settings.node_text_visibility == VisibilitySetting::Hover
                            && nearest_id.unwrap_or(&0usize) == &system.id))
                {
                    let mut viewport_text = viewport_point;
                    viewport_text.x += 3.0 * self.zoom;
                    viewport_text.y -= 3.0 * self.zoom;
                    text_settings.position = viewport_text;
                    text_settings.text = system.name.to_string();
                    self.paint_label(paint, &text_settings);
                }
                paint.circle(
                    viewport_point,
                    4.00 * self.zoom,
                    self.settings.styles[self.current_index].fill_color,
                    self.settings.styles[self.current_index].border.unwrap(),
                );
            }
        }
        Ok(())
    }

    fn paint_map_lines(
        &self,
        vec_points: &Vec<usize>,
        hashm: &Option<HashMap<usize, MapPoint>>,
        paint: &Painter,
        min_point: &Pos2,
    ) -> Result<(), ()> {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("paint_map_lines");

        if hashm.is_none() {
            return Err(());
        }
        if vec_points.is_empty() {
            return Err(());
        }

        // Drawing Lines
        if self.zoom > self.settings.line_visible_zoom {
            for temp_point in vec_points {
                let mut stroke = self.settings.styles[self.current_index].line.unwrap();
                if self.zoom - self.settings.line_visible_zoom <= 0.20 {
                    let mut tup_stroke = self.settings.styles[self.current_index].line.unwrap().color.to_tuple();
                    let transparency = (self.zoom - self.settings.line_visible_zoom) / 0.20;
                    tup_stroke.3 = (tup_stroke.3 as f32 / transparency).round() as u8;
                    let color = Color32::from_rgba_unmultiplied(tup_stroke.0,tup_stroke.1,tup_stroke.2,tup_stroke.3);
                    stroke = Stroke::new(self.settings.styles[self.current_index].line.unwrap().width,color);
                }
                if let Some(system) = hashm.as_ref().unwrap().get(temp_point) {
                    let a_point = Pos2::new(
                        (system.coords[0] as f32 * self.zoom) - min_point.x,
                        (system.coords[1] as f32 * self.zoom) - min_point.y,
                    );
                    for line in &system.lines {
                        let b_point = Pos2::new(
                            (line[0] as f32 * self.zoom) - min_point.x,
                            (line[1] as f32 * self.zoom) - min_point.y,
                        );
                        {
                            paint.line_segment(
                                [a_point, b_point],
                                stroke,
                            );
                        }
                    }
                }
            }
            // Drawing permanent lines
            for line in &self.lines {
                let a = Pos2::new(
                    (line.points[0].x * self.zoom) - min_point.x,
                    (line.points[0].y * self.zoom) - min_point.y,
                );
                let b = Pos2::new(
                    (line.points[1].x * self.zoom) - min_point.x,
                    (line.points[1].y * self.zoom) - min_point.y,
                );
                paint.line_segment(
                    [a, b],
                    self.settings.styles[self.current_index].line.unwrap(),
                );
            }
        }
        Ok(())
    }

    fn paint_label(&self, paint: &Painter, text_settings: &TextSettings) {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("paint_label");
        paint.text(
            text_settings.position,
            text_settings.anchor,
            text_settings.text.clone(),
            FontId::new(text_settings.size, text_settings.family.clone()),
            text_settings.text_color,
        );
    }

    pub fn notify(mut self,id_node: usize,center_map:bool) ->Result<bool,Error> {
        if let Entry::Occupied(system_entry) = self.points.clone().unwrap().entry(id_node){
            let system = system_entry.get();
            if center_map == true {       
                self.set_pos(system.coords[0] as f32, system.coords[1] as f32);
            }
            let node = AnimationPoint::new(system.coords[0] as f32, system.coords[1] as f32,Instant::now());
            self.animon.notifications.push(node);
        }
        else{
            //self.notifications.push((id_node,Utc::now()));
        }
        Ok(true)
    }

}

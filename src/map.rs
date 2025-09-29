use crate::map::animation::Animation;
use crate::map::objects::{
    ContextMenuManager, MapBounds, MapLabel, MapLine, MapPoint, MapSettings, RawLine, RawPoint,
    TextSettings, VisibilitySetting,
};
use chrono;
use egui::{epaint::CircleShape, widgets::*, *};
use kdtree::distance::squared_euclidean;
use kdtree::KdTree;
use std::collections::{HashMap, HashSet};
use std::fmt::Error;
use std::rc::Rc;
use std::time::Instant;

use self::objects::NodeTemplate;

pub mod animation;
pub mod objects;
// This can by any object or point with its associated metadata
/// Struct that contains coordinates to help calculate nearest point in space

#[derive(Clone)]
pub struct Map {
    zoom: f32,
    previous_zoom: f32,
    points: Option<HashMap<usize, MapPoint>>,
    lines: Option<HashMap<String, MapLine>>,
    labels: Vec<MapLabel>,
    tree: Option<KdTree<f32, usize, [f32; 2]>>,
    visible_points: Vec<usize>,
    map_area: Rect,
    reference: MapBounds,
    current: MapBounds,
    style: egui::Style,
    current_index: usize,
    entities: HashMap<usize, Instant>,
    min_size: (Option<f32>, Option<f32>),
    max_size: (Option<f32>, Option<f32>),
    pub settings: MapSettings,
    menu_manager: Option<Rc<dyn ContextMenuManager>>,
    node_template: Option<Rc<dyn NodeTemplate>>,
    visible_lines: HashSet<String>,
    markers: HashMap<usize, usize>,
}

impl Default for Map {
    fn default() -> Self {
        Map::new()
    }
}

impl Widget for &mut Map {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        let rect = self.calculate_widget_dimentions(ui);

        // we define the initial coordinate as the center of such rectangle
        self.reference.dist = rect.distance();

        self.assign_visual_style(ui);

        let canvas = egui::Frame::canvas(ui.style()).inner_margin(Margin::symmetric(3, 5));

        let inner_response = canvas.show(ui, |ui| {
            #[cfg(feature = "puffin")]
            puffin::profile_scope!("paint_map");

            if ui.is_rect_visible(self.map_area) {
                let (resp, paint) =
                    ui.allocate_painter(self.map_area.size(), egui::Sense::click_and_drag());
                let vec = resp.drag_delta();
                if vec.length() != 0.0 {
                    #[cfg(feature = "puffin")]
                    puffin::profile_scope!("calculating_points_in_visible_area");

                    let coords = RawPoint::from(vec.to_pos2());
                    let new_pos = self.reference.pos - (coords / self.zoom);
                    self.set_pos(new_pos.into());
                }
                let map_style = self.settings.styles[self.current_index].clone() * self.zoom;
                if self.zoom < self.settings.line_visible_zoom {
                    // filling text settings
                    let mut text_settings = TextSettings {
                        size: 12.00 * self.zoom * 2.00,
                        anchor: Align2::CENTER_CENTER,
                        family: FontFamily::Proportional,
                        text: String::new(),
                        position: RawPoint::default(),
                        text_color: ui.visuals().text_color(),
                    };
                    for label in &self.labels {
                        text_settings.text.clone_from(&label.text);
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

                // Here we determine the widget center to print all nodes
                // let min_point = self.current.pos - RawPoint::try_from([self.map_area.center().x,self.map_area.center().y]).unwrap();

                let rect_midpoint = RawPoint::from(self.map_area.center());
                let min_point = self.current.pos - rect_midpoint;
                let vec_points = &self.visible_points;
                let hashm = &self.points;
                self.paint_map_lines(&paint, &min_point);

                if let Ok(nodes_to_remove) =
                    self.paint_map_points(vec_points, hashm, &paint, ui, &min_point, &resp)
                {
                    for node in nodes_to_remove {
                        self.entities.remove(&node);
                    }
                }

                for marker in &self.markers {
                    if let Some(point) = self.points.as_ref().unwrap().get(marker.1) {
                        let adjusted_point = point.raw_point * self.zoom - min_point;
                        if let Some(template) = &self.node_template {
                            template.marker_ui(ui, adjusted_point.into(), self.zoom);
                        } else {
                            let mut shapes = Vec::new();
                            let color = if ui.visuals().dark_mode {
                                Color32::LIGHT_GREEN
                            } else {
                                Color32::GREEN
                            };
                            let mut transparency =
                                (chrono::Local::now().timestamp_millis() % 2550) / 5;
                            if transparency > 255 {
                                transparency = 255 - (transparency - 255)
                            }
                            let corrected_color = Color32::from_rgba_unmultiplied(
                                color.r(),
                                color.g(),
                                color.b(),
                                transparency as u8,
                            );
                            shapes.push(Shape::Circle(CircleShape::stroke(
                                adjusted_point.into(),
                                4.0 * self.zoom,
                                Stroke::new(9.0 * self.zoom, corrected_color),
                            )));
                            ui.ctx().request_repaint();
                            ui.painter().extend(shapes);
                        }
                    }
                }

                self.paint_sub_components(ui, self.map_area);

                self.capture_mouse_events(ui, &resp);

                if self.zoom != self.previous_zoom {
                    #[cfg(feature = "puffin")]
                    puffin::profile_scope!("calculating viewport with zoom");
                    self.adjust_bounds();
                    self.calculate_visible_points();
                    self.previous_zoom = self.zoom;
                }

                if let Some(ref mut menu_mon) = &mut self.menu_manager {
                    resp.context_menu(|ui| {
                        menu_mon.ui(ui);
                    });
                }

                if cfg!(debug_assertions) {
                    self.print_debug_info(paint, resp);
                }
            }
        });
        ui.allocate_space(self.map_area.size());
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
            lines: None,
            labels: Vec::new(),
            visible_points: Vec::new(),
            current: MapBounds::default(),
            reference: MapBounds::default(),
            settings,
            min_size: (None, None),
            max_size: (None, None),
            current_index: 0,
            entities: HashMap::new(),
            style: egui::Style::default(),
            menu_manager: None,
            node_template: None,
            visible_lines: HashSet::new(),
            markers: HashMap::new(),
        }
    }

    fn calculate_widget_dimentions(&mut self, ui: &mut Ui) -> RawLine {
        self.map_area = ui.available_rect_before_wrap();
        let mut left_top = RawPoint::from(self.map_area.left_top());
        let mut right_bottom = RawPoint::from(self.map_area.right_bottom());
        if self.max_size.0.is_some()
            && right_bottom.components[0] > self.max_size.0.unwrap_or(0.0f32)
        {
            right_bottom.components[0] = self.max_size.0.unwrap();
        }
        if self.max_size.1.is_some()
            && right_bottom.components[1] > self.max_size.1.unwrap_or(0.0f32)
        {
            right_bottom.components[1] = self.max_size.1.unwrap();
        }
        if self.min_size.0.is_some() && left_top.components[0] < self.min_size.0.unwrap_or(0.0f32) {
            left_top.components[0] = self.min_size.0.unwrap();
        }
        if self.min_size.1.is_some() && left_top.components[1] < self.min_size.1.unwrap_or(0.0f32) {
            left_top.components[1] = self.min_size.1.unwrap();
        }
        RawLine::new(left_top, right_bottom)
    }

    fn calculate_visible_points(&mut self) {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("calculate_visible_points");
        if self.current.dist > 0.0 && self.current.dist < f32::INFINITY {
            if let Some(tree) = &self.tree {
                let center = self.current.pos / self.zoom;
                let radius = self.current.dist.powi(2);
                let point: [f32; 2] = center.into();
                let vis_pos = tree.within(&point, radius, &squared_euclidean).unwrap();
                self.visible_points.clear();
                for point in vis_pos {
                    self.visible_points.push(*point.1);
                    let system = self.points.as_ref().unwrap().get(point.1);
                    for connection in &system.unwrap().connections {
                        if !self.visible_lines.contains(&connection.clone()) {
                            self.visible_lines.insert(connection.clone());
                        }
                    }
                }
            }
        }
    }

    pub fn add_hashmap_points(&mut self, hash_map: HashMap<usize, MapPoint>) {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("add_hashmap_points");
        let mut min = RawPoint::new(f32::INFINITY, f32::INFINITY);
        let mut max = RawPoint::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
        let mut tree = KdTree::<f32, usize, [f32; 2]>::new(2);
        let mut h_map = hash_map.clone();

        for entry in h_map.iter_mut() {
            for i in 0..min.components.len() {
                if entry.1.raw_point.components[i] < min.components[i] {
                    min.components[i] = entry.1.raw_point.components[i];
                }
                if entry.1.raw_point.components[i] > max.components[i] {
                    max.components[i] = entry.1.raw_point.components[i];
                }
            }
            let _result = tree.add(entry.1.raw_point.into(), *entry.0);
        }

        // We stablish the max and min coordinates in this map, this wont change until we change the point hash map
        self.reference.min = min;
        self.reference.max = max;
        self.points = Some(h_map);
        self.tree = Some(tree);
        self.reference.pos = RawLine::new(min, max).midpoint();
        // we create a rect that include every node in the map
        // Stupid fix because rect area could be infinite
        // I need to implement a more elegant fix
        if self.map_area.area() == 0.0 {
            self.reference.dist = 3000.00;
        } else {
            let rect = RawLine::new(
                RawPoint::from(self.map_area.left_top()),
                RawPoint::from(self.map_area.right_bottom()),
            );
            self.reference.dist = rect.distance();
        }
        self.current = self.reference.clone();
        self.calculate_visible_points();
    }

    pub fn set_pos_from_nodeid(&mut self, node_id: usize) {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("set_pos_from_nodeid");
        if let Some(hash_map) = &self.points {
            if let Some(map_point) = hash_map.get(&node_id) {
                self.reference.pos = map_point.raw_point;
                self.adjust_bounds();
                self.calculate_visible_points();
            }
        }
    }

    pub fn set_pos(&mut self, position: [f32; 2]) {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("set_pos");
        let point = RawPoint::from(position);
        self.reference.pos = point;
        self.adjust_bounds();
        self.calculate_visible_points();
    }

    pub fn get_pos(self) -> [f32; 2] {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("get_pos");
        self.reference.pos.into()
    }

    pub fn add_labels(&mut self, labels: Vec<MapLabel>) {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("add_labels");
        self.labels = labels;
    }

    pub fn add_lines(&mut self, lines: HashMap<String, MapLine>) {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("add_lines");
        self.lines = Some(lines);
    }

    fn adjust_bounds(&mut self) {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("adjust_bounds");
        self.current.max = self.reference.max * self.zoom;
        self.current.min = self.reference.min * self.zoom;
        self.current.dist = self.reference.dist / self.zoom;
        self.current.pos = self.reference.pos * self.zoom;
    }

    fn capture_mouse_events(&mut self, ui: &Ui, resp: &Response) {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("capture_mouse_events");
        // capture MouseWheel Event for Zoom control change
        if ui.rect_contains_pointer(self.map_area) {
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
        if resp.secondary_clicked() {}
    }

    pub fn set_zoom(mut self, value: f32) {
        if value >= self.settings.min_zoom && value <= self.settings.max_zoom {
            self.zoom = value;
        }
    }

    pub fn get_zoom(&mut self) -> f32 {
        self.zoom
    }

    fn assign_visual_style(&mut self, ui_obj: &mut Ui) {
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
            + self.current.min.components[0].to_string().as_str()
            + ","
            + self.current.min.components[1].to_string().as_str();
        paint.debug_text(init_pos, Align2::LEFT_TOP, Color32::LIGHT_GREEN, msg);
        init_pos.y += 15.0;
        msg = "MAX:".to_string()
            + self.current.max.components[0].to_string().as_str()
            + ","
            + self.current.max.components[1].to_string().as_str();
        paint.debug_text(init_pos, Align2::LEFT_TOP, Color32::LIGHT_GREEN, msg);
        init_pos.y += 15.0;
        msg = "CUR:(".to_string()
            + self.current.pos.components[0].to_string().as_str()
            + ","
            + self.current.pos.components[1].to_string().as_str()
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
        pos2.x -= 60.0;
        pos2.y += 240.0;

        // TODO: Verify if this implementation its correct migrated from allocate_ui_at_rect()
        let sub_rect = egui::Rect::from_two_pos(pos1, pos2);
        //ui_obj.allocate_ui_with_layout(sub_rect.size(), egui::Layout::right_to_left(Align::TOP), |ui_obj| {
        let ui_builder = egui::UiBuilder::new().clone().max_rect(sub_rect);
        //});
        ui_obj.scope_builder(ui_builder, |ui_obj| {
            ui_obj.add(zoom_slider);
        });
    }

    fn paint_map_points(
        &self,
        vec_points: &Vec<usize>,
        hashm: &Option<HashMap<usize, MapPoint>>,
        paint: &Painter,
        ui_obj: &mut Ui,
        min_point: &RawPoint,
        resp: &Response,
    ) -> Result<Vec<usize>, ()> {
        let mut nearest_id = None;
        let mut nodes_to_remove = Vec::new();
        let mut shape_vec = vec![];

        if hashm.is_none() {
            return Err(());
        }
        if vec_points.is_empty() {
            return Err(());
        }
        // detecting the nearest hover node
        if self.settings.node_text_visibility == VisibilitySetting::Hover && resp.hovered() {
            if let Some(point) = resp.hover_pos() {
                let raw_point = RawPoint::from(point);
                let hovered_map_point = (*min_point + raw_point) / self.zoom;
                if let Ok(nearest_node) = self.tree.as_ref().unwrap().nearest(
                    &hovered_map_point.components,
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
            position: RawPoint::default(),
            text_color: ui_obj.visuals().text_color(),
        };

        // Drawing Points
        for temp_point in vec_points {
            if let Some(system) = hashm.as_ref().unwrap().get(temp_point) {
                #[cfg(feature = "puffin")]
                puffin::profile_scope!("painting_points_m");
                let viewport_point = system.raw_point * self.zoom - min_point;
                if let Some(node_template) = &self.node_template {
                    if nearest_id.unwrap_or(&0usize) == &system.get_id() {
                        node_template.selection_ui(ui_obj, viewport_point.into(), self.zoom);
                    }
                } else if self.zoom > self.settings.label_visible_zoom
                    && self.settings.node_text_visibility == VisibilitySetting::Allways
                    || (self.settings.node_text_visibility == VisibilitySetting::Hover
                        && nearest_id.unwrap_or(&0usize) == &system.get_id())
                {
                    let mut viewport_text = viewport_point;
                    viewport_text.components[0] += 3.0 * self.zoom;
                    viewport_text.components[1] -= 3.0 * self.zoom;
                    text_settings.position = viewport_text;
                    text_settings.text = system.get_name();
                    self.paint_label(paint, &text_settings);
                }

                let system_id = system.get_id();
                if let Some(init_time) = self.entities.get(&system_id) {
                    if let Some(template) = &self.node_template {
                        template.notification_ui(
                            ui_obj,
                            viewport_point.into(),
                            self.zoom,
                            *init_time,
                            self.settings.styles[self.current_index].alert_color,
                        );
                    } else {
                        match Animation::pulse(
                            paint,
                            viewport_point,
                            self.zoom,
                            *init_time,
                            self.settings.styles[self.current_index].alert_color,
                        ) {
                            Ok(true) => {
                                ui_obj.ctx().request_repaint();
                            }
                            Ok(false) => nodes_to_remove.push(system_id),
                            Err(_) => (),
                        }
                    }
                }
                if let Some(node_template) = &self.node_template {
                    node_template.node_ui(ui_obj, viewport_point.into(), self.zoom, system);
                } else {
                    shape_vec.push(Shape::circle_filled(
                        viewport_point.into(),
                        4.00 * self.zoom,
                        self.settings.styles[self.current_index].fill_color,
                    ));
                }
            }
        }
        paint.extend(shape_vec);
        Ok(nodes_to_remove)
    }

    fn paint_map_lines(&self, painter: &Painter, min_point: &RawPoint) {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("paint_map_lines");

        // Drawing Lines
        if self.zoom > self.settings.line_visible_zoom {
            let mut shape_vec = vec![];
            let mut stroke = self.settings.styles[self.current_index].line.unwrap();
            let transparency_range = self.zoom - self.settings.line_visible_zoom;
            if (0.00..0.80).contains(&transparency_range) {
                let mut tup_stroke = self.settings.styles[self.current_index]
                    .line
                    .unwrap()
                    .color
                    .to_tuple();
                let transparency = (self.zoom - self.settings.line_visible_zoom) / 0.80;
                tup_stroke.3 = (255.0 * transparency).round() as u8;
                let color = Color32::from_rgba_unmultiplied(
                    tup_stroke.0,
                    tup_stroke.1,
                    tup_stroke.2,
                    tup_stroke.3,
                );
                stroke = Stroke::new(
                    self.settings.styles[self.current_index].line.unwrap().width,
                    color,
                );
            }
            //let stroke = Stroke::new(10.0,Color32::GREEN);
            for line in &self.visible_lines {
                if let Some(connection) = self.lines.as_ref().unwrap().get(line) {
                    let pos_a = connection.raw_line.points[0] * self.zoom - min_point;
                    let pos_b = connection.raw_line.points[1] * self.zoom - min_point;
                    //let pos_a = connection.raw_line.points[0] / self.zoom - min_point;
                    //let pos_b = connection.raw_line.points[1] / self.zoom - min_point;
                    shape_vec.push(Shape::line_segment([pos_a.into(), pos_b.into()], stroke));
                    //shape_vec.push(painter.line_segment([pos_a.into(),pos_b.into()], stroke));
                }
            }
            painter.extend(shape_vec);
        }
    }

    fn paint_label(&self, paint: &Painter, text_settings: &TextSettings) {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("paint_label");
        paint.text(
            text_settings.position.into(),
            text_settings.anchor,
            text_settings.text.clone(),
            FontId::new(text_settings.size, text_settings.family.clone()),
            text_settings.text_color,
        );
    }

    pub fn notify(&mut self, id_node: usize, time: Instant) -> Result<bool, Error> {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("notify");
        self.entities
            .entry(id_node)
            .and_modify(|value| *value = time)
            .or_insert(time);
        Ok(true)
    }

    pub fn set_context_manager(&mut self, manager: Rc<dyn ContextMenuManager>) {
        self.menu_manager = Some(manager);
    }

    pub fn set_node_template(&mut self, template: Rc<dyn NodeTemplate>) {
        self.node_template = Some(template);
    }

    pub fn update_marker(&mut self, id: usize, node_id: usize) {
        self.markers
            .entry(id)
            .and_modify(|value| *value = node_id)
            .or_insert(node_id);
    }

    pub fn allocate_at_least(&mut self, width: Option<f32>, height: Option<f32>) {
        self.min_size = (width, height);
    }

    pub fn allocate_at_most(&mut self, width: Option<f32>, height: Option<f32>) {
        self.max_size = (width, height);
    }
}

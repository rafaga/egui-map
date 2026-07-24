//! Interactive map widget and the data types it renders.
//!
//! [`Map`] is an [`egui::Widget`] that draws a 2D set of nodes
//! ([`objects::MapPoint`]), the connection lines between them
//! ([`objects::MapSegment`]) and free-floating text labels
//! ([`objects::MapLabel`]). Nodes are indexed in a kd-tree so that only the
//! ones inside the current viewport are painted each frame.
//!
//! ## Coordinate model
//!
//! The widget works with two coordinate spaces:
//!
//! - **Map coordinates**: the logical position of your nodes, as loaded through
//!   [`Map::add_hashmap_points`].
//! - **Screen coordinates**: positions inside the widget's rectangle on screen.
//!
//! Both are related by the current zoom factor and viewport origin:
//! `screen = map * zoom - origin`. Use [`Map::set_zoom`], [`Map::set_pos`] and
//! [`Map::set_pos_from_nodeid`] to control the visible region.
//!
//! ## Connecting nodes with lines
//!
//! Lines are wired up in three steps:
//!
//! 1. Create the nodes as a [`HashMap`] keyed by node id.
//! 2. For every connection, choose a unique string id and push it into
//!    [`MapPoint::connections`] of **both** endpoint nodes.
//! 3. Load the nodes with [`Map::add_hashmap_points`], then load a
//!    [`Vec`] of [`MapSegment`] keyed by those same connection ids and
//!    add it to the widget with [`Map::add_lines`].
//!
//! ```
//! use egui_map::map::Map;
//! use egui_map::map::objects::{MapPoint, MapSegment, RawPoint};
//! use std::collections::HashMap;
//! use std::rc::Rc;
//!
//! // 1. Create the nodes.
//! let mut points: HashMap<usize, MapPoint> = HashMap::new();
//! points.insert(1, MapPoint::new(1, RawPoint::new(0.0, 0.0)));
//! points.insert(2, MapPoint::new(2, RawPoint::new(10.0, 10.0)));
//!
//! // 2. Register the connection id on both endpoints.
//! for id in [1, 2] {
//!     points
//!         .get_mut(&id)
//!         .unwrap()
//!         .connections
//!         .push("1-2".to_string());
//! }
//!
//! let mut map = Map::new();
//! map.add_hashmap_points(points);
//!
//! // 3. Provide the line geometry keyed by the same connection id.
//! let mut lines: Vec<MapSegment> = Vec::new();
//! lines.push(
//!     MapSegment::new(Rc::from("1-2"), RawPoint::new(0.0, 0.0), RawPoint::new(10.0, 10.0))
//! );
//! map.add_lines(lines);
//! ```
//!
//! A line is only drawn while the zoom level is above
//! [`MapSettings::line_visible_zoom`] and its bounding box intersects the
//! viewport. Segments are culled broad-phase with an R-tree built by
//! [`Map::add_lines`], so long lines crossing the view are drawn even when
//! both endpoints lie outside of it.
//!
//! ## Custom node rendering
//!
//! Install a [`NodeTemplate`] implementation with [`Map::set_node_template`]
//! to take over the rendering of nodes, selection highlights, notification
//! animations and markers. Note that this replaces
//! *all* built-in node rendering, including the node name labels: draw them
//! yourself in [`NodeTemplate::node_ui`] if you need them.

use crate::map::animation::Animation;
use crate::map::objects::{
    ContextMenuManager, MapBounds, MapLabel, MapPoint, MapSegment, MapSettings, MapStyle, RawLine,
    RawPoint, TextSettings, VisibilitySetting,
};
use egui::{epaint::CircleShape, widgets::*, *};
use kdtree::KdTree;
use kdtree::distance::squared_euclidean;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

use self::objects::NodeTemplate;

pub mod animation;
pub mod objects;

/// An interactive 2D map widget.
///
/// `Map` renders a set of nodes ([`objects::MapPoint`]), connection lines
/// ([`objects::MapSegment`]) and text labels ([`objects::MapLabel`]). The user can
/// pan the view by dragging and zoom with the mouse wheel (hold `Ctrl` — or
/// `Cmd` on macOS — to zoom faster), or use the built-in zoom slider drawn at
/// the top-right corner of the widget.
///
/// The map is fed through [`Map::add_hashmap_points`], which also builds the
/// internal kd-tree used for viewport culling and nearest-node hover queries.
/// Behavior and appearance are configured through the public
/// [`settings`](Map::settings) field (see [`objects::MapSettings`]).
///
/// Rendering of nodes and their visual effects (selection highlight,
/// notifications and markers) can be fully customized by installing a
/// [`objects::NodeTemplate`] implementation with [`Map::set_node_template`];
/// likewise, a right-click context menu can be provided with
/// [`Map::set_context_manager`].
///
/// # Examples
///
/// ```no_run
/// # fn example(ui: &mut egui::Ui) {
/// use egui_map::map::Map;
/// use egui_map::map::objects::{MapPoint, RawPoint};
/// use std::collections::HashMap;
///
/// let mut points = HashMap::new();
/// points.insert(1, MapPoint::new(1, RawPoint::new(0.0, 0.0)));
///
/// let mut map = Map::new();
/// map.add_hashmap_points(points);
///
/// // Every frame, inside your egui update logic:
/// ui.add(&mut map);
/// # }
/// ```
#[derive(Clone)]
pub struct Map {
    zoom: f32,
    previous_zoom: f32,
    points: Option<HashMap<usize, MapPoint>>,
    segments: Option<rstar::RTree<MapSegment>>,
    labels: Vec<MapLabel>,
    tree: Option<KdTree<f32, usize, [f32; 2]>>,
    visible_points: Vec<isize>,
    map_area: Rect,
    reference: MapBounds,
    current: MapBounds,
    current_index: usize,
    entities: HashMap<usize, Instant>,
    min_size: (Option<f32>, Option<f32>),
    max_size: (Option<f32>, Option<f32>),
    /// Behavior and appearance configuration (zoom limits, visibility
    /// thresholds and per-theme styles). See [`objects::MapSettings`].
    pub settings: MapSettings,
    menu_manager: Option<Rc<dyn ContextMenuManager>>,
    node_template: Option<Rc<dyn NodeTemplate>>,
    markers: HashMap<usize, usize>,
}

impl Default for Map {
    /// Creates an empty map; equivalent to [`Map::new`].
    fn default() -> Self {
        Map::new()
    }
}

impl Widget for &mut Map {
    /// Renders the map, handling panning (drag), zooming (mouse wheel) and the
    /// right-click context menu if one was installed.
    fn ui(self, ui: &mut egui::Ui) -> Response {
        let rect = self.calculate_widget_dimensions(ui);

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
                        text_settings.position = RawPoint::from(label.center);
                        self.paint_label(&paint, &text_settings);
                    }
                }

                let rect_midpoint = RawPoint::from(self.map_area.center());
                let min_point = self.current.pos - rect_midpoint;
                let vec_points = &self.visible_points;
                let hashm = &self.points;

                // Safety net: drop stale notifications even if their node is
                // outside the viewport and never finishes its animation.
                let now = Instant::now();
                self.entities
                    .retain(|_, init| now.duration_since(*init).as_secs_f32() < 10.0);

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
                            let millis = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis();
                            let mut transparency = (millis % 2550 / 5) as i64;
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

                if let Some(menu_mon) = &mut self.menu_manager {
                    resp.context_menu(|ui| {
                        menu_mon.ui(ui);
                    });
                }

                #[cfg(feature = "debug_overlay")]
                self.print_debug_info(paint, resp);
            }
        });
        ui.allocate_space(self.map_area.size());
        inner_response.response
    }
}

impl Map {
    /// Creates an empty map widget with default [`MapSettings`].
    ///
    /// The widget displays nothing until nodes are loaded with
    /// [`Map::add_hashmap_points`].
    pub fn new() -> Self {
        let settings = MapSettings::default();
        Self {
            zoom: 1.0,
            previous_zoom: 1.0,
            map_area: Rect::NOTHING,
            tree: None,
            points: None,
            labels: Vec::new(),
            visible_points: Vec::new(),
            current: MapBounds::default(),
            reference: MapBounds::default(),
            settings,
            min_size: (None, None),
            max_size: (None, None),
            current_index: 0,
            entities: HashMap::new(),
            menu_manager: None,
            node_template: None,
            markers: HashMap::new(),
            segments: None,
        }
    }

    fn calculate_widget_dimensions(&mut self, ui: &mut Ui) -> RawLine {
        let available = ui.available_rect_before_wrap();
        let mut size = available.size();
        if let Some(max_width) = self.max_size.0 {
            size.x = size.x.min(max_width);
        }
        if let Some(max_height) = self.max_size.1 {
            size.y = size.y.min(max_height);
        }
        if let Some(min_width) = self.min_size.0 {
            size.x = size.x.max(min_width);
        }
        if let Some(min_height) = self.min_size.1 {
            size.y = size.y.max(min_height);
        }
        self.map_area = Rect::from_min_size(available.min, size);
        RawLine::new(
            RawPoint::from(self.map_area.left_top()),
            RawPoint::from(self.map_area.right_bottom()),
        )
    }

    fn calculate_visible_points(&mut self) {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("calculate_visible_points");
        if self.current.dist > 0.0
            && self.current.dist < f32::INFINITY
            && let Some(tree) = &self.tree
        {
            let center = self.current.pos / self.zoom;
            let radius = self.current.dist.powi(2);
            let point: [f32; 2] = center.into();
            let vis_pos = tree.within(&point, radius, &squared_euclidean).unwrap();
            self.visible_points.clear();
            for point in vis_pos {
                self.visible_points.push(point.1.cast_signed());
            }
        }
    }
    /// Loads the node set and (re)builds the spatial index.
    ///
    /// This replaces any previously loaded points, computes the bounding box of
    /// the whole set, centers the view on its midpoint and refreshes the list
    /// of visible nodes. It must be called at least once before the widget can
    /// display anything.
    ///
    /// The kd-tree built here is what enables viewport culling and
    /// nearest-neighbor hover lookups, so calling this method on every frame is
    /// discouraged; call it only when the node set changes.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_map::map::Map;
    /// use egui_map::map::objects::{MapPoint, RawPoint};
    /// use std::collections::HashMap;
    ///
    /// let mut points = HashMap::new();
    /// points.insert(1, MapPoint::new(1, RawPoint::new(0.0, 0.0)));
    /// points.insert(2, MapPoint::new(2, RawPoint::new(10.0, 10.0)));
    ///
    /// let mut map = Map::new();
    /// map.add_hashmap_points(points);
    ///
    /// // The view is centered on the midpoint of the loaded nodes.
    /// assert_eq!(map.get_pos(), [5.0, 5.0]);
    /// ```
    pub fn add_hashmap_points(&mut self, hash_map: HashMap<usize, MapPoint>) {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("add_hashmap_points");
        let mut min = RawPoint::new(f32::INFINITY, f32::INFINITY);
        let mut max = RawPoint::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
        let mut tree = KdTree::<f32, usize, [f32; 2]>::new(2);

        for entry in hash_map.iter() {
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
        self.points = Some(hash_map);
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

    /// Centers the view on the node with the given id.
    ///
    /// Does nothing if no points have been loaded yet or if `node_id` is
    /// unknown.
    pub fn set_pos_from_nodeid(&mut self, node_id: usize) {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("set_pos_from_nodeid");
        if let Some(hash_map) = &self.points
            && let Some(map_point) = hash_map.get(&node_id)
        {
            self.reference.pos = map_point.raw_point;
            self.adjust_bounds();
            self.calculate_visible_points();
        }
    }

    /// Centers the view on the given map coordinates.
    pub fn set_pos(&mut self, position: [f32; 2]) {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("set_pos");
        let point = RawPoint::from(position);
        self.reference.pos = point;
        self.adjust_bounds();
        self.calculate_visible_points();
    }

    /// Returns the map coordinates the view is currently centered on.
    pub fn get_pos(&self) -> [f32; 2] {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("get_pos");
        self.reference.pos.into()
    }

    /// Replaces the set of free-floating text labels drawn on the map.
    ///
    /// Labels are only rendered while the zoom level is below
    /// [`MapSettings::line_visible_zoom`].
    pub fn add_labels(&mut self, labels: Vec<MapLabel>) {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("add_labels");
        self.labels = labels;
    }

    /// Replaces the set of connection lines between nodes.
    ///
    /// Lines are keyed by a connection id that the endpoint nodes must
    /// reference through [`MapPoint::connections`] — push each line's key into
    /// the `connections` of the nodes it joins. The segments are stored in an
    /// R-tree keyed by bounding box: a line is drawn while its bounding box
    /// intersects the viewport and the zoom level is above
    /// [`MapSettings::line_visible_zoom`].
    ///
    /// See the [module-level example](self#connecting-nodes-with-lines) for
    /// the complete wiring.
    pub fn add_lines(&mut self, segments: Vec<MapSegment>) {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("add_lines");
        // Intern the keys as Rc<str> and build the broad-phase spatial index
        // over the line bounding boxes, so viewport culling and hit-testing
        // discard whole regions without touching every segment.

        self.segments = Some(rstar::RTree::bulk_load(segments));
    }

    fn adjust_bounds(&mut self) {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("adjust_bounds");
        self.current.max = self.reference.max * self.zoom;
        self.current.min = self.reference.min * self.zoom;
        self.current.dist = self.reference.dist / self.zoom;
        self.current.pos = self.reference.pos * self.zoom;
    }

    fn capture_mouse_events(&mut self, ui: &Ui, _resp: &Response) {
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
                                phase: _,
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
    }

    /// Sets the zoom factor.
    ///
    /// Values outside the [`MapSettings::min_zoom`]..=[`MapSettings::max_zoom`]
    /// range are ignored.
    pub fn set_zoom(&mut self, value: f32) {
        if value >= self.settings.min_zoom && value <= self.settings.max_zoom {
            self.zoom = value;
        }
    }

    /// Returns the current zoom factor.
    pub fn get_zoom(&mut self) -> f32 {
        self.zoom
    }

    /// Returns the style for the current theme, falling back to the first
    /// style if the current theme index has no entry.
    fn current_style(&self) -> &MapStyle {
        self.settings
            .styles
            .get(self.current_index)
            .or(self.settings.styles.first())
            .expect("MapSettings::styles must not be empty")
    }

    fn assign_visual_style(&mut self, ui_obj: &mut Ui) {
        let style_index = ui_obj.visuals().dark_mode as usize;

        if self.current_index != style_index {
            #[cfg(feature = "puffin")]
            puffin::profile_scope!("asign_visual_style");

            self.current_index = style_index;
            let map_style = self.settings.styles.get_mut(style_index).unwrap();
            let visuals = &ui_obj.style().visuals;
            map_style.background_color = visuals.extreme_bg_color;
            map_style.border = Some(visuals.window_stroke);
        }
    }

    #[cfg(feature = "debug_overlay")]
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
        .orientation(SliderOrientation::Vertical);
        let mut pos1 = rect.right_top();
        let mut pos2 = rect.right_top();
        pos1.x -= 80.0;
        pos1.y += 120.0;
        pos2.x -= 60.0;
        pos2.y += 240.0;

        let sub_rect = egui::Rect::from_two_pos(pos1, pos2);
        let ui_builder = egui::UiBuilder::new().clone().max_rect(sub_rect);
        ui_obj.scope_builder(ui_builder, |ui_obj| {
            ui_obj.add(zoom_slider);
        });
    }

    fn paint_map_points(
        &self,
        vec_points: &Vec<isize>,
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
        if self.settings.node_text_visibility == VisibilitySetting::Hover
            && resp.hovered()
            && let Some(point) = resp.hover_pos()
        {
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
            let parsed_point = temp_point.cast_unsigned();
            if let Some(system) = hashm.as_ref().unwrap().get(&parsed_point) {
                #[cfg(feature = "puffin")]
                puffin::profile_scope!("painting_points_m");
                let viewport_point = system.raw_point * self.zoom - min_point;
                if let Some(node_template) = &self.node_template {
                    if nearest_id.unwrap_or(&0usize) == &system.get_id() {
                        node_template.selection_ui(ui_obj, viewport_point.into(), self.zoom);
                    }
                } else if self.zoom > self.settings.label_visible_zoom
                    && self.settings.node_text_visibility == VisibilitySetting::Always
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
                            self.current_style().alert_color,
                        );
                    } else if Animation::pulse(
                        paint,
                        viewport_point,
                        self.zoom,
                        *init_time,
                        self.current_style().alert_color,
                    ) {
                        ui_obj.ctx().request_repaint();
                    } else {
                        nodes_to_remove.push(system_id);
                    }
                }
                if let Some(node_template) = &self.node_template {
                    node_template.node_ui(ui_obj, viewport_point.into(), self.zoom, system);
                } else {
                    shape_vec.push(Shape::circle_filled(
                        viewport_point.into(),
                        4.00 * self.zoom,
                        self.current_style().fill_color,
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
        if self.zoom > self.settings.line_visible_zoom
            && let Some(mut stroke) = self.current_style().line
            && let Some(segments) = &self.segments
        {
            let mut shape_vec = vec![];
            let transparency_range = self.zoom - self.settings.line_visible_zoom;
            if (0.00..0.80).contains(&transparency_range) {
                let mut tup_stroke = stroke.color.to_tuple();
                let transparency = (self.zoom - self.settings.line_visible_zoom) / 0.80;
                tup_stroke.3 = (255.0 * transparency).round() as u8;
                let color = Color32::from_rgba_unmultiplied(
                    tup_stroke.0,
                    tup_stroke.1,
                    tup_stroke.2,
                    tup_stroke.3,
                );
                stroke = Stroke::new(stroke.width, color);
            }
            // Broad-phase: query the segment R-tree with the viewport AABB
            // (in map coordinates), padded by the stroke width so lines at
            // the very edge are not clipped prematurely.
            let center = self.current.pos / self.zoom;
            let padding = stroke.width / self.zoom;
            let half = RawPoint::new(
                self.map_area.width() / 2.0 / self.zoom + padding,
                self.map_area.height() / 2.0 / self.zoom + padding,
            );
            let query = rstar::AABB::from_corners(center - half, center + half);
            for segment in segments.locate_in_envelope_intersecting(query) {
                let pos_a = segment.raw_line.points[0] * self.zoom - min_point;
                let pos_b = segment.raw_line.points[1] * self.zoom - min_point;
                shape_vec.push(Shape::line_segment([pos_a.into(), pos_b.into()], stroke));
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

    /// Triggers a notification highlight on the node `id_node`.
    ///
    /// By default the notification is rendered as a pulsing circle that starts
    /// at `time` and plays for about 3.5 seconds; calling `notify` again for
    /// the same node restarts the animation. The effect can be customized with
    /// [`objects::NodeTemplate::notification_ui`].
    pub fn notify(&mut self, id_node: usize, time: Instant) {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("notify");
        self.entities
            .entry(id_node)
            .and_modify(|value| *value = time)
            .or_insert(time);
    }

    /// Returns the id of the line closest to `point`, in map coordinates,
    /// when it lies within `tolerance` map units of the segment.
    ///
    /// Broad-phase candidates are taken from the segment R-tree built by
    /// [`Map::add_lines`]; the exact point-to-segment distance is then
    /// computed against the line geometry and the closest match wins. Returns
    /// `None` when no lines are loaded or every segment is farther than
    /// `tolerance`. A negative `tolerance` behaves like `0.0`.
    ///
    /// To hit-test a mouse click, convert the screen position to map
    /// coordinates first (`map = (screen + origin) / zoom`, see the
    /// [coordinate model](self#coordinate-model)) and pick a tolerance scaled
    /// by `1.0 / zoom` so it stays constant in screen pixels.
    pub fn line_at(&self, point: [f32; 2], tolerance: f32) -> Option<Rc<str>> {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("line_at");
        let segments = self.segments.as_ref()?;
        let tolerance = tolerance.max(0.0);

        let center = RawPoint::from(point);
        let padding = RawPoint::new(tolerance, tolerance);
        let query = rstar::AABB::from_corners(center - padding, center + padding);

        let mut closest: Option<(f32, Rc<str>)> = None;
        for segment in segments.locate_in_envelope_intersecting(query) {
            let distance = segment.raw_line.distance_to_point(center);
            if distance <= tolerance && closest.as_ref().is_none_or(|(best, _)| distance < *best) {
                closest = Some((distance, Rc::clone(&segment.id)));
            }
        }
        closest.map(|(_, id)| id)
    }

    /// Installs a right-click context menu whose contents are built by the
    /// given [`ContextMenuManager`] implementation.
    pub fn set_context_manager(&mut self, manager: Rc<dyn ContextMenuManager>) {
        self.menu_manager = Some(manager);
    }

    /// Replaces the built-in node rendering with a custom [`NodeTemplate`]
    /// implementation.
    ///
    /// The template takes over the drawing of nodes, selection highlights,
    /// notification animations and markers — including the node name labels,
    /// which the widget no longer draws once a template is installed. See the
    /// [`NodeTemplate`] examples for custom shapes and animations.
    pub fn set_node_template(&mut self, template: Rc<dyn NodeTemplate>) {
        self.node_template = Some(template);
    }

    /// Adds the marker `id`, or moves it, so it points to the node `node_id`.
    ///
    /// Markers are drawn as a blinking ring around the target node unless a
    /// custom [`objects::NodeTemplate::marker_ui`] is installed.
    pub fn update_marker(&mut self, id: usize, node_id: usize) {
        self.markers
            .entry(id)
            .and_modify(|value| *value = node_id)
            .or_insert(node_id);
    }

    /// Sets the minimum width and/or height the widget should occupy, in egui
    /// points. `None` leaves the corresponding dimension unconstrained.
    pub fn allocate_at_least(&mut self, width: Option<f32>, height: Option<f32>) {
        self.min_size = (width, height);
    }

    /// Sets the maximum width and/or height the widget should occupy, in egui
    /// points. `None` leaves the corresponding dimension unconstrained.
    pub fn allocate_at_most(&mut self, width: Option<f32>, height: Option<f32>) {
        self.max_size = (width, height);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn sample_points() -> HashMap<usize, MapPoint> {
        let mut map = HashMap::new();
        map.insert(1, MapPoint::new(1, RawPoint::new(0.0, 0.0)));
        map.insert(2, MapPoint::new(2, RawPoint::new(10.0, 10.0)));
        map.insert(3, MapPoint::new(3, RawPoint::new(-10.0, -10.0)));
        map
    }

    // ---------- construcción ----------

    #[test]
    fn map_new_initial_state() {
        let map = Map::new();
        assert_eq!(map.zoom, 1.0);
        assert_eq!(map.previous_zoom, 1.0);
        assert!(map.points.is_none());
        assert!(map.segments.is_none());
        assert!(map.tree.is_none());
        assert!(map.labels.is_empty());
        assert!(map.visible_points.is_empty());
        assert!(map.markers.is_empty());
        assert!(map.entities.is_empty());
        assert_eq!(map.min_size, (None, None));
        assert_eq!(map.max_size, (None, None));
        assert_eq!(map.current_index, 0);
    }

    #[test]
    fn map_default_equals_new() {
        let map = Map::default();
        assert_eq!(map.zoom, 1.0);
        assert!(map.points.is_none());
    }

    // ---------- zoom ----------

    #[test]
    fn set_zoom_within_range() {
        let mut map = Map::new();
        map.set_zoom(1.5);
        assert_eq!(map.get_zoom(), 1.5);
    }

    #[test]
    fn set_zoom_at_exact_limits() {
        let mut map = Map::new();
        map.set_zoom(map.settings.min_zoom);
        assert_eq!(map.get_zoom(), 0.1);
        map.set_zoom(map.settings.max_zoom);
        assert_eq!(map.get_zoom(), 2.0);
    }

    #[test]
    fn set_zoom_out_of_range_is_ignored() {
        let mut map = Map::new();
        let initial = map.get_zoom();
        map.set_zoom(0.05); // por debajo de min_zoom
        assert_eq!(map.get_zoom(), initial);
        map.set_zoom(2.5); // por encima de max_zoom
        assert_eq!(map.get_zoom(), initial);
    }

    // ---------- puntos ----------

    #[test]
    fn add_hashmap_points_computes_bounds() {
        let mut map = Map::new();
        map.add_hashmap_points(sample_points());

        assert_eq!(map.reference.min.components, [-10.0, -10.0]);
        assert_eq!(map.reference.max.components, [10.0, 10.0]);
        // pos es el punto medio del rectángulo que contiene todos los puntos
        assert_eq!(map.reference.pos.components, [0.0, 0.0]);
        // map_area tiene área 0 antes de renderizar, así que dist es el valor fijo
        assert_eq!(map.reference.dist, 3000.0);
        // current se inicializa como copia de reference
        assert_eq!(map.current.min.components, map.reference.min.components);
        assert_eq!(map.current.max.components, map.reference.max.components);
        assert_eq!(map.current.pos.components, map.reference.pos.components);
        assert_eq!(map.current.dist, map.reference.dist);
        assert!(map.points.is_some());
        assert!(map.tree.is_some());
        assert_eq!(map.points.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn add_hashmap_points_populates_visible_points() {
        let mut map = Map::new();
        map.add_hashmap_points(sample_points());
        // todos los puntos de muestra caen dentro del radio por defecto
        assert_eq!(map.visible_points.len(), 3);
    }

    /// Renders one frame of `map` in a 500x500 viewport and returns the
    /// painted line segments.
    fn render_line_segments(map: &mut Map) -> Vec<[egui::Pos2; 2]> {
        use egui::{Context, RawInput, Shape};
        let ctx = Context::default();
        let input = RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(500.0, 500.0),
            )),
            ..RawInput::default()
        };
        let output = ctx.run_ui(input, |ui| {
            ui.add(&mut *map);
        });
        output
            .shapes
            .iter()
            .filter_map(|cs| match cs.shape {
                Shape::LineSegment { points, .. } => Some(points),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn segment_crossing_viewport_is_painted_even_with_far_endpoints() {
        // With the old endpoint-based rule this line was culled: both
        // endpoints sit beyond the point-culling radius. With the R-tree the
        // segment AABB intersects the viewport, so it is painted — no points
        // needed at all.
        let mut map = Map::new();
        map.set_zoom(1.0);
        let mut lines = Vec::new();
        lines.push(MapSegment::new(
            Rc::from("long"),
            RawPoint::new(-4000.0, -1.0),
            RawPoint::new(4000.0, 1.0),
        ));
        map.add_lines(lines);
        map.set_pos([0.0, 0.0]);

        let segments = render_line_segments(&mut map);
        assert_eq!(segments.len(), 1);
    }

    #[test]
    fn segment_outside_viewport_is_not_painted() {
        let mut map = Map::new();
        map.set_zoom(1.0);
        let mut lines = Vec::new();
        lines.push(MapSegment::new(
            Rc::from("far"),
            RawPoint::new(10_000.0, 10_000.0),
            RawPoint::new(10_100.0, 10_100.0),
        ));
        map.add_lines(lines);
        map.set_pos([0.0, 0.0]);

        assert!(render_line_segments(&mut map).is_empty());
    }

    #[test]
    fn add_lines_builds_segment_tree() {
        let mut map = Map::new();
        map.add_hashmap_points(sample_points());
        let mut lines = Vec::new();
        lines.push(MapSegment::new(
            Rc::from("1-2"),
            RawPoint::new(0.0, 0.0),
            RawPoint::new(10.0, 10.0),
        ));
        map.add_lines(lines);

        let tree = map
            .segments
            .as_ref()
            .expect("add_lines must build the segment tree");
        assert_eq!(tree.size(), 1);

        // Broad-phase query: a viewport containing (0,0) must hit the segment;
        // a far-away viewport must not.
        let hit_query =
            rstar::AABB::from_corners(RawPoint::new(-1.0, -1.0), RawPoint::new(1.0, 1.0));
        let hits: Vec<_> = tree.locate_in_envelope_intersecting(hit_query).collect();
        assert_eq!(hits.len(), 1);
        assert_eq!(&*hits[0].id, "1-2");

        let miss_query =
            rstar::AABB::from_corners(RawPoint::new(100.0, 100.0), RawPoint::new(200.0, 200.0));
        assert_eq!(tree.locate_in_envelope_intersecting(miss_query).count(), 0);
    }

    #[test]
    fn map_check_line_is_painted_on_first_frame() {
        use egui::{Context, RawInput, Shape};

        // --- arrange ---
        let mut map = Map::new();
        map.set_zoom(1.0);

        let mut point_a = MapPoint::new(0, RawPoint::new(0.0, 0.0));
        point_a.connections.push("a0".to_string());
        let mut point_b = MapPoint::new(1, RawPoint::new(50.0, 50.0));
        point_b.connections.push("a0".to_string());

        let mut lines = Vec::new();
        lines.push(MapSegment::new(
            Rc::from("a0"),
            point_a.raw_point,
            point_b.raw_point,
        ));

        let mut points = HashMap::new();
        points.insert(0usize, point_a);
        points.insert(1usize, point_b);
        // Load points before lines — the natural order shown in the examples.
        map.add_hashmap_points(points);
        map.add_lines(lines);

        map.set_pos([25.0, 25.0]);

        // --- act: 1st frame (no CentralPanel — run_ui creates the root Ui) ---
        let ctx = Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(500.0, 500.0));
        let input = RawInput {
            screen_rect: Some(screen),
            ..RawInput::default()
        };

        let output1 = ctx.run_ui(input.clone(), |ui| {
            ui.add(&mut map);
        });

        let segments1: Vec<[egui::Pos2; 2]> = output1
            .shapes
            .iter()
            .filter_map(|cs| match cs.shape {
                Shape::LineSegment { points, .. } => Some(points),
                _ => None,
            })
            .collect();

        assert!(
            !segments1.is_empty(),
            "Frame 1: no LineSegment shapes painted (map lines did not draw)"
        );

        // Expected projection of (0,0)->(50,50) with zoom=1, center=(25,25),
        // viewport 500x500: pos_a = (225, 225), pos_b = (275, 275). Tolerance ±2 px.
        let expected_a = egui::pos2(225.0, 225.0);
        let expected_b = egui::pos2(275.0, 275.0);
        let tolerance = 2.0;
        let found_on_frame1 = segments1.iter().any(|[p1, p2]| {
            let d_a1 = p1.distance(expected_a);
            let d_b1 = p2.distance(expected_b);
            let d_a2 = p2.distance(expected_a);
            let d_b2 = p1.distance(expected_b);
            (d_a1 < tolerance && d_b1 < tolerance) || (d_a2 < tolerance && d_b2 < tolerance)
        });
        assert!(
            found_on_frame1,
            "Frame 1: no LineSegment matches expected endpoints (~225,225 -> ~275,275); got {:?}",
            segments1
        );

        // --- act: 2nd frame (unchanged) — detect duplicate-lines regression ---
        let output2 = ctx.run_ui(input, |ui| {
            ui.add(&mut map);
        });

        let segments2: Vec<[egui::Pos2; 2]> = output2
            .shapes
            .iter()
            .filter_map(|cs| match cs.shape {
                Shape::LineSegment { points, .. } => Some(points),
                _ => None,
            })
            .collect();

        assert_eq!(
            segments1.len(),
            segments2.len(),
            "Frame 2: expected {} line segments (no duplication across frames), got {}",
            segments1.len(),
            segments2.len()
        );
    }

    // ---------- posición ----------

    #[test]
    fn set_pos_and_get_pos_roundtrip() {
        let mut map = Map::new();
        map.set_pos([25.0, -35.0]);
        assert_eq!(map.get_pos(), [25.0, -35.0]);
    }

    #[test]
    fn set_pos_from_nodeid_with_valid_id() {
        let mut map = Map::new();
        map.add_hashmap_points(sample_points());
        map.set_pos_from_nodeid(2);
        assert_eq!(map.get_pos(), [10.0, 10.0]);
    }

    #[test]
    fn set_pos_from_nodeid_with_invalid_id_keeps_position() {
        let mut map = Map::new();
        map.add_hashmap_points(sample_points());
        let before = map.reference.pos.components;
        map.set_pos_from_nodeid(999);
        assert_eq!(map.reference.pos.components, before);
    }

    #[test]
    fn set_pos_from_nodeid_without_points_does_nothing() {
        let mut map = Map::new();
        map.set_pos_from_nodeid(1);
        assert_eq!(map.reference.pos.components, [0.0, 0.0]);
    }

    // ---------- etiquetas y líneas ----------

    #[test]
    fn add_labels_stores_labels() {
        let mut map = Map::new();
        let label = MapLabel {
            text: "Region".to_string(),
            center: Pos2::new(1.0, 2.0),
        };
        map.add_labels(vec![label]);
        assert_eq!(map.labels.len(), 1);
        assert_eq!(map.labels[0].text, "Region");
    }

    #[test]
    fn add_lines_stores_lines() {
        let mut map = Map::new();
        let mut lines = Vec::new();
        lines.push(MapSegment::new(
            Rc::from("a-b"),
            RawPoint::new(0.0, 0.0),
            RawPoint::new(1.0, 1.0),
        ));
        map.add_lines(lines);
        let tree = map.segments.as_ref().unwrap();
        assert_eq!(tree.size(), 1);
        assert_eq!(
            &*tree
                .locate_in_envelope_intersecting(rstar::AABB::from_corners(
                    RawPoint::new(-1.0, -1.0),
                    RawPoint::new(2.0, 2.0),
                ))
                .next()
                .unwrap()
                .id,
            "a-b"
        );
    }

    // ---------- notificaciones y marcadores ----------

    #[test]
    fn line_at_returns_closest_line_within_tolerance() {
        let mut map = Map::new();
        map.add_hashmap_points(sample_points());
        let mut lines = Vec::new();
        lines.push(MapSegment::new(
            Rc::from("horizontal"),
            RawPoint::new(0.0, 0.0),
            RawPoint::new(10.0, 0.0),
        ));
        lines.push(MapSegment::new(
            Rc::from("vertical"),
            RawPoint::new(20.0, -5.0),
            RawPoint::new(20.0, 5.0),
        ));
        map.add_lines(lines);

        // 1.5 units above the horizontal segment.
        let hit = map.line_at([5.0, 1.5], 2.0).expect("line must be hit");
        assert_eq!(&*hit, "horizontal");

        // Closest to the vertical segment.
        let hit = map.line_at([19.0, 0.0], 2.0).expect("line must be hit");
        assert_eq!(&*hit, "vertical");
    }

    #[test]
    fn line_at_returns_none_beyond_tolerance() {
        let mut map = Map::new();
        map.add_hashmap_points(sample_points());
        let mut lines = Vec::new();
        lines.push(MapSegment::new(
            Rc::from("1-2"),
            RawPoint::new(0.0, 0.0),
            RawPoint::new(10.0, 10.0),
        ));
        map.add_lines(lines);

        // Distance from (5,4) to the diagonal segment (0,0)-(10,10) is
        // |5-4|/sqrt(2) ~= 0.707.
        assert!(map.line_at([5.0, 4.0], 0.8).is_some());
        assert!(map.line_at([5.0, 4.0], 0.5).is_none());
        assert!(map.line_at([100.0, 100.0], 5.0).is_none());
    }

    #[test]
    fn line_at_returns_none_without_lines() {
        let map = Map::new();
        assert!(map.line_at([0.0, 0.0], 10.0).is_none());
    }

    #[test]
    fn line_at_negative_tolerance_behaves_like_zero() {
        let mut map = Map::new();
        map.add_hashmap_points(sample_points());
        let mut lines = Vec::new();
        lines.push(MapSegment::new(
            Rc::from("1-2"),
            RawPoint::new(0.0, 0.0),
            RawPoint::new(10.0, 10.0),
        ));
        map.add_lines(lines);

        // Exact point on the segment is hit even with tolerance clamped to 0.
        assert!(map.line_at([5.0, 5.0], -1.0).is_some());
        assert!(map.line_at([5.0, 5.1], -1.0).is_none());
    }

    #[test]
    fn notify_inserts_and_updates_entities() {
        let mut map = Map::new();
        let t1 = Instant::now();
        map.notify(5, t1);
        assert_eq!(map.entities.get(&5), Some(&t1));

        let t2 = t1 + Duration::from_secs(1);
        map.notify(5, t2);
        assert_eq!(map.entities.get(&5), Some(&t2));
        assert_eq!(map.entities.len(), 1);
    }

    #[test]
    fn update_marker_inserts_and_updates() {
        let mut map = Map::new();
        map.update_marker(1, 100);
        assert_eq!(map.markers.get(&1), Some(&100));
        map.update_marker(1, 200);
        assert_eq!(map.markers.get(&1), Some(&200));
        assert_eq!(map.markers.len(), 1);
    }

    // ---------- tamaño ----------

    #[test]
    fn allocate_at_least_sets_min_size() {
        let mut map = Map::new();
        map.allocate_at_least(Some(100.0), None);
        assert_eq!(map.min_size, (Some(100.0), None));
    }

    #[test]
    fn allocate_at_most_sets_max_size() {
        let mut map = Map::new();
        map.allocate_at_most(None, Some(200.0));
        assert_eq!(map.max_size, (None, Some(200.0)));
    }

    // ---------- bounds ----------

    #[test]
    fn adjust_bounds_scales_with_zoom() {
        let mut map = Map::new();
        map.reference.min = RawPoint::new(-10.0, -20.0);
        map.reference.max = RawPoint::new(10.0, 20.0);
        map.reference.pos = RawPoint::new(5.0, 5.0);
        map.reference.dist = 100.0;
        map.set_zoom(2.0);
        map.adjust_bounds();

        assert_eq!(map.current.max.components, [20.0, 40.0]);
        assert_eq!(map.current.min.components, [-20.0, -40.0]);
        assert_eq!(map.current.pos.components, [10.0, 10.0]);
        assert_eq!(map.current.dist, 50.0);
    }
}

use egui::{widgets::*, *};
use kdtree::KdTree;
use kdtree::distance::squared_euclidean;
use std::collections::HashMap;
use rand::thread_rng;
use rand::distributions::{Alphanumeric,Distribution};

// This can by any object or point with its associated metadata
/// Struct that contains coordinates to help calculate nearest point in space
#[derive(Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct MapPoint{
    dimension: usize,
    /// coordinates of the Solar System
    pub coords: [f64;3],
    /// coordinates for lines connecting this point
    pub lines: Vec<[f64;3]>,
    /// Object Identifier for search propurses
    pub id: usize,
    /// SolarSystem Name
    pub name: String,
}

impl MapPoint{
    /// Creates a new Spatial point with an Id (solarSystemId) and the system's 3D coordinates
    pub fn new(id: usize, coords: Vec<f64>) -> MapPoint {
        let mut point = [0.0f64;3];
        let size= coords.len();
        point[0] = coords[0];
        point[1] = coords[1];
        if size == 3 {
            point[2] = coords[2];
        }
        MapPoint {
            coords: point,
            dimension: size,
            id,
            lines: Vec::new(),
            name: String::new(),
        }
    }


    /// Get the number of dimensions used in this object
    pub fn get_dimension(self) -> usize {
        self.dimension
    }

}


#[derive(Clone)]
struct MapBounds{
    pub min: Pos2,
    pub max: Pos2,
    pub pos: Pos2,
    pub dist: f64,
}

impl MapBounds{
    pub fn new() -> Self {
        MapBounds{
            min: Pos2::new(0.0,0.0),
            max: Pos2::new(0.0,0.0),
            pos: Pos2::new(0.0,0.0),
            dist: 0.0,
        }
    }
}

impl Default for MapBounds {
    fn default() -> Self {
        MapBounds::new()
    }
}

// This can by any object or point with its associated metadata
/// Struct that contains coordinates to help calculate nearest point in space

pub struct Map {
    pub zoom: f32,
    previous_zoom: f32,
    points: Option<HashMap<usize,MapPoint>>,
    tree: Option<KdTree<f64,usize,[f64;2]>>,
    visible_points: Option<Vec<usize>>,
    map_area: Option<Rect>,
    initialized: bool,
    reference: MapBounds,
    current: MapBounds,
}

impl Default for Map {
    fn default() -> Self {
        Map::new()
    }
}

impl Widget for &mut Map {
    fn ui(self, ui_obj: &mut egui::Ui) -> Response {
        if self.initialized == false {
            let mut rng = thread_rng();
            let component_id: String = Alphanumeric
                .sample_iter(&mut rng)
                .take(15)
                .map(char::from)
                .collect();
            let idx = egui::Id::new(component_id);
            ui_obj.make_persistent_id(idx);
            self.map_area = Some(ui_obj.available_rect_before_wrap());
        } else {
            self.map_area = Some(ui_obj.ctx().used_rect());
        }
        if self.zoom != self.previous_zoom {
            self.adjust_bounds();
            self.calculate_visible_points();
            self.previous_zoom = self.zoom;
        }

        let style = egui::style::Style::default();
        let canvas = egui::Frame::canvas(&style)
            .stroke(egui::Stroke{width:2.0f32, color:Color32::DARK_GRAY});
        
        let inner_response = canvas.show(ui_obj, |ui_obj| {
            let (resp,paint) = ui_obj.allocate_painter(self.map_area.unwrap().size(), egui::Sense::click_and_drag());
            let vec = resp.drag_delta();
            if vec.length() != 0.0 {
                let coords = (vec.to_pos2().x,  vec.to_pos2().y);
                self.set_pos(self.current.pos.x - coords.0, self.current.pos.y -coords.1);
                self.calculate_visible_points();
            }
            let gate_stroke = egui::Stroke{ width: 2f32 * self.zoom, color: Color32::DARK_RED};
            let system_color = Color32::YELLOW;
            let system_stroke = egui::Stroke{ width: 2f32 -self.zoom, color: system_color};
            //let debug_stroke = egui::Stroke{ width: 2f32, color: Color32::GOLD};
            
            for temp_vec_point in &self.visible_points {
                if let Some(hashm) = self.points.as_mut() {
                    let factor = (self.map_area.unwrap().center().x  + self.map_area.unwrap().min.x,self.map_area.unwrap().center().y  + self.map_area.unwrap().min.y);
                    let min_point = Pos2::new(self.current.pos.x-factor.0, self.current.pos.y-factor.1);
                    if self.zoom > 0.2 {
                        for temp_point in temp_vec_point{
                            if let Some(system) = hashm.get(&temp_point) {
                                let center = Pos2::new(system.coords[0] as f32 * self.zoom,system.coords[1] as f32 * self.zoom);
                                let a_point = Pos2::new(center.x-min_point.x,center.y-min_point.y);
                                for line in &system.lines {
                                    let b_point = Pos2::new((line[0] as f32 * self.zoom)-min_point.x,(line[1] as f32 * self.zoom)-min_point.y);
                                    paint.line_segment([a_point, b_point], gate_stroke);
                                }
                            }
                        } 
                    }
                    for temp_point in temp_vec_point{
                        if let Some(system) = hashm.get(&temp_point) { 
                            let center = Pos2::new(system.coords[0] as f32 * self.zoom,system.coords[1] as f32 * self.zoom);
                            let viewport_point = Pos2::new(center.x-min_point.x,center.y-min_point.y);
                            let mut viewport_text = viewport_point.clone();
                            viewport_text.x += 3.0;
                            viewport_text.y -= 3.0;
                            if self.zoom > 0.58 {
                                paint.text(viewport_text,Align2::LEFT_BOTTOM,system.name.to_string(),FontId::new(12.00 * self.zoom,FontFamily::Proportional),Color32::LIGHT_GREEN);
                            }
                            paint.circle(viewport_point, 4.00 * self.zoom, system_color, system_stroke);
                        }
                    }
                }
            }
            if let Some(rect) = self.map_area{
                let zoom_slider = egui::Slider::new(&mut self.zoom, 0.1..=2.0)
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
                ui_obj.allocate_ui_at_rect(sub_rect,|ui_obj|{
                    ui_obj.add(zoom_slider);
                });
            }
            if cfg!(debug_assertions) {
                let mut init_pos = Pos2::new(self.map_area.unwrap().left_top().x + 10.00, self.map_area.unwrap().left_top().y + 10.00);
                let mut msg = String::from("MIN:".to_string() + self.current.min.x.to_string().as_str() + "," + self.current.min.y.to_string().as_str());
                paint.debug_text(init_pos, Align2::LEFT_TOP, Color32::LIGHT_GREEN, msg);
                init_pos.y += 15.0;
                msg = "MAX:".to_string() + self.current.max.x.to_string().as_str() + "," + self.current.max.y.to_string().as_str();
                paint.debug_text(init_pos, Align2::LEFT_TOP, Color32::LIGHT_GREEN, msg);
                init_pos.y += 15.0;
                msg = "CUR:(".to_string() + self.current.pos.x.to_string().as_str() + "," + self.current.pos.y.to_string().as_str() +")";
                paint.debug_text(init_pos, Align2::LEFT_TOP, Color32::LIGHT_GREEN, msg);
                init_pos.y += 15.0;
                msg = "DST:".to_string() + self.current.dist.to_string().as_str();
                paint.debug_text(init_pos, Align2::LEFT_TOP, Color32::LIGHT_GREEN, msg);
                init_pos.y += 15.0;
                msg = "ZOM:".to_string() + self.zoom.to_string().as_str();
                paint.debug_text(init_pos, Align2::LEFT_TOP, Color32::GREEN, msg);
                if let Some(rectz) = self.map_area {
                    init_pos.y += 15.0;
                    msg = "REC:(".to_string() + rectz.left_top().x.to_string().as_str() + "," + rectz.left_top().y.to_string().as_str() + "),(" + rectz.right_bottom().x.to_string().as_str() + "," + rectz.right_bottom().y.to_string().as_str() + ")";
                    paint.debug_text(init_pos, Align2::LEFT_TOP, Color32::LIGHT_GREEN, msg);
                }
                if let Some(points) = &self.points {
                    init_pos.y += 15.0;
                    msg = "NUM:".to_string() + points.len().to_string().as_str();
                    paint.debug_text(init_pos, Align2::LEFT_TOP, Color32::LIGHT_GREEN, msg);
                }
                if let Some(vec_k) = self.visible_points.as_ref(){
                    init_pos.y += 15.0;
                    msg = "VIS:".to_string() + vec_k.len().to_string().as_str();
                    paint.debug_text(init_pos, Align2::LEFT_TOP, Color32::LIGHT_GREEN, msg);
                }
                if let Some(pointer_pos) = resp.hover_pos() {
                    init_pos.y += 15.0;
                    msg = "HVR:".to_string() + pointer_pos.x.to_string().as_str() + "," + pointer_pos.y.to_string().as_str();
                    paint.debug_text(init_pos, Align2::LEFT_TOP, Color32::LIGHT_BLUE, msg);
                }
                let vec = resp.drag_delta();
                if vec.length() != 0.0 {
                    init_pos.y += 15.0;
                    msg = "DRG:".to_string() + vec.to_pos2().x.to_string().as_str() + "," + vec.to_pos2().y.to_string().as_str();
                    paint.debug_text(init_pos, Align2::LEFT_TOP, Color32::GOLD, msg);
                }
            }
        }); 
        inner_response.response
    }

}

impl Map {
    pub fn new() -> Self {
        Map {
            zoom: 1.0,
            previous_zoom: 1.0,
            map_area: None,
            tree: None,
            points: None,
            visible_points: None,
            initialized: false,
            current: MapBounds::default(),
            reference: MapBounds::default(),
        }
    }

    fn calculate_visible_points(&mut self) -> () {
        if self.current.dist > 0.0 {
            if let Some(tree) = &self.tree{
                let center = [(self.current.pos.x / self.zoom) as f64,(self.current.pos.y / self.zoom) as f64];
                let radius = self.current.dist.powi(2);
                let vis_pos = tree.within(&center, radius, &squared_euclidean).unwrap();
                let mut visible_points = vec![];
                for point in vis_pos {
                    visible_points.push(*point.1);
                }
                self.visible_points = Some(visible_points);
            }
        }
    }

    pub fn add_points(&mut self, points: Vec<MapPoint>) -> (){
        let mut hmap = HashMap::new();
        let mut min = (f64::INFINITY,f64::INFINITY);
        let mut max = (f64::NEG_INFINITY,f64::NEG_INFINITY);
        let mut tree = KdTree::<f64,usize,[f64;2]>::new(2);
        for mut point in points{
            point.coords[0] *= -1.0;
            point.coords[1] *= -1.0;
            if point.coords[0] < min.0 {
                min.0 = point.coords[0];
            }
            if point.coords[1] < min.1 {
                min.1 = point.coords[1];
            }
            if point.coords[0] > max.0 {
                max.0 = point.coords[0];
            }
            if point.coords[1] > max.1 {
                max.1 = point.coords[1];
            }
            let _result = tree.add([point.coords[0],point.coords[1]],point.id);
            for line in &mut point.lines {
                line[0] *= -1.0;
                line[1] *= -1.0;
                line[2] *= -1.0;
            }
            hmap.entry(point.id).or_insert(point);
        }
        self.reference.min = Pos2::new(min.0 as f32,min.1 as f32);
        self.reference.max = Pos2::new(max.0 as f32,max.1 as f32);
        self.points = Some(hmap);
        self.tree = Some(tree);
        let rect = Rect::from_min_max(self.reference.min, self.reference.max);
        self.reference.pos = rect.center();
        let dist_x = (self.map_area.unwrap().right_bottom().x as f64 - self.map_area.unwrap().left_top().x as f64)/2.0;
        let dist_y = (self.map_area.unwrap().right_bottom().y as f64 - self.map_area.unwrap().left_top().y as f64)/2.0;
        self.reference.dist = (dist_x.powi(2) + dist_y.powi(2)/2.0).sqrt() as f64;
        self.current = self.reference.clone();
        self.calculate_visible_points();
    } 

    pub fn set_pos(&mut self, x: f32, y:f32) -> () {
        if x <= self.current.max.x && x >= self.current.min.x && y <= self.current.max.y && y >= self.current.min.y{
            self.current.pos = Pos2::new(x ,y);
            self.reference.pos = Pos2::new(x/self.zoom,y/self.zoom);
        }
    }

    pub fn add_labels() -> () {

    }

    fn adjust_bounds(&mut self) -> () {
        self.current.max.x = self.reference.max.x * self.zoom;
        self.current.max.y = self.reference.max.y * self.zoom;
        self.current.min.x = self.reference.min.x * self.zoom;
        self.current.min.y = self.reference.min.y * self.zoom;
        self.current.dist = self.reference.dist / self.zoom as f64;
        self.set_pos(self.reference.pos.x * self.zoom, self.reference.pos.y * self.zoom);
    }
    
}
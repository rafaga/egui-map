//! A visual catalog of the built-in node and segment animations, reached
//! through `Map::node` and `Map::segment` -- no custom template involved, just
//! the effects the widget already knows how to draw.
//!
//! Everything is laid out on a grid so nothing overlaps, and every node and
//! every segment shows exactly one effect, named after it:
//!
//! - **Node effects** (left), one node each. Lasting ones are visible
//!   immediately and forever: `halo`, `blink`, `orbit`, and `ripple` with
//!   `NodeHandle::lasting`, which keeps a new ring coming with no break where
//!   it ends and starts over. Event ones are fired again on a timer of their
//!   own: `pulse`, `scale_in`, `crosshair`, and `countdown` with `lasting`,
//!   whose ring empties over exactly the time asked for. (`lasting` turns any
//!   of the event effects into one that repeats and fades out over the time
//!   asked for, as the `ripple` and the `countdown` show; it is the same
//!   effect, so it has no entry of its own.)
//! - **Segment effects** (right), one segment each, between two plain nodes.
//!   Lasting: `comet`, `dash`, `glow_band`, `chevrons`. Event: `flash`,
//!   `comet_once` and `wipe`. Every one but `flash` has a direction, set with
//!   `Map::segment(..).direction(..)` before the effect, and the *Direction*
//!   radio buttons in the window pick it for all of them: `Forward` runs from
//!   the first endpoint of the segment to the second, `Reverse` the other
//!   way. `flash` lights the whole line at once and has no direction.
//!
//! Every event timer fires independently and at a different period, so
//! several different animations are usually playing at once rather than
//! everything ticking in lockstep. The two titles are `RegionLabel`s
//! (`Map::add_region_labels`), painted behind the grid.
//!
//! The window has the same controls as `examples/basic.rs`: a combo box with
//! the built-in themes (`Map::set_theme`) next to egui's light/dark/system
//! buttons, and a tooltip on the node under the pointer (`Map::hovered_node`).
//!
//! Run with: cargo run --example animations

use eframe::egui::{self, FontId};
use egui_map::map::Map;
use egui_map::map::objects::{CometDirection, MapPoint, MapSegment, RegionLabel};
use egui_map::map::theme::Theme;
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Every built-in theme, in the order the combo box lists them.
const THEMES: [Theme; 15] = [
    Theme::SystemDefault,
    Theme::SlateOcean,
    Theme::NebulaViolet,
    Theme::TerminalGreen,
    Theme::EmberForge,
    Theme::SolarAmber,
    Theme::ArticCyan,
    Theme::CrimsonSignal,
    Theme::MidnightIndigo,
    Theme::CopperRose,
    Theme::LimeCircuit,
    Theme::CoralReef,
    Theme::GraphiteMono,
    Theme::PlumStatic,
    Theme::SandstoneTrail,
];

/// The node effects, in grid order (three per row): name, and the id the
/// node gets.
const NODE_EFFECTS: [&str; 8] = [
    "halo",
    "blink",
    "orbit",
    "pulse",
    "ripple",
    "countdown",
    "scale_in",
    "crosshair",
];

/// The segment effects: the lasting ones in the first column, the event ones
/// in the second. All of them follow the direction picked in the window
/// except `flash`, which lights the whole line at once.
const LASTING_SEGMENTS: [&str; 4] = ["comet", "dash", "glow_band", "chevrons"];
const EVENT_SEGMENTS: [&str; 3] = ["flash", "comet_once", "wipe"];

/// Distance between the nodes of the grid, and between the rows of segments.
const NODE_STEP: [f32; 2] = [135.0, 105.0];
const SEGMENT_STEP: f32 = 70.0;
/// Length of a segment.
const SEGMENT_LENGTH: f32 = 110.0;

/// Starts one effect on the map, at the given instant.
type Fire = Box<dyn Fn(&mut Map, Instant)>;

/// Fires an effect again every `period`, starting `first_delay` after the
/// start. Each repeater only ever plays the one effect it was built with.
struct Repeater {
    period: Duration,
    last_fired: Instant,
    fire: Fire,
}

impl Repeater {
    fn new(
        period_ms: u64,
        first_delay_ms: u64,
        fire: impl Fn(&mut Map, Instant) + 'static,
    ) -> Self {
        let period = Duration::from_millis(period_ms);
        Self {
            period,
            last_fired: Instant::now() - period + Duration::from_millis(first_delay_ms),
            fire: Box::new(fire),
        }
    }

    fn tick(&mut self, map: &mut Map, now: Instant) {
        if now.duration_since(self.last_fired) < self.period {
            return;
        }
        (self.fire)(map, now);
        self.last_fired = now;
    }
}

/// A repeater for a node effect.
fn on_node(
    id: usize,
    period_ms: u64,
    first_delay_ms: u64,
    effect: impl Fn(egui_map::map::NodeHandle<'_>, Instant) + 'static,
) -> Repeater {
    Repeater::new(period_ms, first_delay_ms, move |map, at| {
        if let Some(node) = map.node(id) {
            effect(node, at);
        }
    })
}

/// Starts the lasting segment effects, running the way `direction` says. Run
/// it again when the direction changes: it replaces the previous state.
fn start_lasting(map: &mut Map, ids: &HashMap<&str, (usize, usize)>, direction: CometDirection) {
    for name in LASTING_SEGMENTS {
        let segment = map
            .segment(ids[name])
            .expect("the segment is loaded")
            .direction(direction);
        match name {
            "comet" => segment.comet(),
            "dash" => segment.dash(),
            "glow_band" => segment.glow_band(),
            _ => segment.chevrons(),
        }
    }
}

/// A repeater for a segment effect.
fn on_segment(
    id: (usize, usize),
    period_ms: u64,
    first_delay_ms: u64,
    effect: impl Fn(egui_map::map::SegmentHandle<'_>, Instant) + 'static,
) -> Repeater {
    Repeater::new(period_ms, first_delay_ms, move |map, at| {
        if let Some(segment) = map.segment(id) {
            effect(segment, at);
        }
    })
}

fn main() -> eframe::Result<()> {
    // 1. The nodes, keyed by id. `names` is kept apart for the tooltip.
    let mut points: HashMap<usize, MapPoint> = HashMap::new();
    let mut names: HashMap<usize, String> = HashMap::new();
    let mut add_node = |id: usize, name: &str, tooltip: String, x: f32, y: f32| {
        let mut point = MapPoint::new(id, [x, y]);
        point.set_name(name.to_string());
        points.insert(id, point);
        names.insert(id, tooltip);
    };

    // The grid of node effects, three per row, on the left.
    for (index, name) in NODE_EFFECTS.iter().enumerate() {
        let (column, row) = ((index % 3) as f32, (index / 3) as f32);
        add_node(
            index + 1,
            name,
            format!("{name} (node {})", index + 1),
            -400.0 + NODE_STEP[0] * column,
            NODE_STEP[1] * row,
        );
    }

    // The segments, on the right: each one between a node named after the
    // effect and a plain one. Ids are `(left, right)`, and all of them run
    // from left to right, so `Forward` goes that way and `Reverse` the other.
    let mut segments = Vec::new();
    let mut segment_ids: HashMap<&str, (usize, usize)> = HashMap::new();
    let mut next_id = NODE_EFFECTS.len() + 1;
    for (column, effects) in [&LASTING_SEGMENTS[..], &EVENT_SEGMENTS[..]]
        .into_iter()
        .enumerate()
    {
        for (row, name) in effects.iter().enumerate() {
            let (left, right) = (next_id, next_id + 1);
            next_id += 2;
            let x = -10.0 + 200.0 * column as f32;
            let y = SEGMENT_STEP * row as f32;
            add_node(left, name, format!("{name}: start"), x, y);
            add_node(right, "", format!("{name}: end"), x + SEGMENT_LENGTH, y);
            segments.push(MapSegment::new(
                (left, right),
                [x, y],
                [x + SEGMENT_LENGTH, y],
            ));
            segment_ids.insert(name, (left, right));
        }
    }

    // 2. Register each connection id on both endpoint nodes.
    for segment in &segments {
        for endpoint in [segment.id.0, segment.id.1] {
            points
                .get_mut(&endpoint)
                .expect("the endpoint is one of the nodes above")
                .connections
                .push(segment.id);
        }
    }

    // 3. Load the nodes, the lines and the two titles.
    let mut map = Map::new();
    map.add_hashmap_points(points);
    map.add_lines(segments);
    map.settings.style.region_label_font = FontId::proportional(30.0);
    map.add_region_labels(vec![
        RegionLabel {
            text: "Node effects".to_string(),
            center: egui::pos2(-265.0, -70.0),
            color: None,
        },
        RegionLabel {
            text: "Segment effects".to_string(),
            center: egui::pos2(145.0, -65.0),
            color: None,
        },
    ]);

    // Lasting effects: set once, visible for as long as the app runs.
    map.node(1).expect("halo is loaded").halo();
    map.node(2).expect("blink is loaded").blink();
    map.node(3).expect("orbit is loaded").orbit();
    // The ripple is a lasting notification, started once: a new ring is born
    // every moment, with no break where the effect ends and starts over. It
    // fades out slowly over the duration it is given (an hour, here).
    map.node(5)
        .expect("ripple is loaded")
        .lasting(Duration::from_secs(3600))
        .ripple(Instant::now());
    // A segment effect runs from the first endpoint to the second unless
    // `direction` says otherwise. The window picks it, and the event effects
    // read it each time they fire.
    let direction = Rc::new(Cell::new(CometDirection::Forward));
    start_lasting(&mut map, &segment_ids, direction.get());

    // Event effects: each on its own independent, non-synchronized timer, so
    // several different animations are usually playing at once.
    let mut repeaters = vec![
        on_node(4, 2200, 0, |node, at| node.pulse(at)),
        // The ring empties over the time asked for with `lasting`: three
        // seconds, then a pause before it is fired again.
        on_node(6, 4000, 900, |node, at| {
            node.lasting(Duration::from_secs(3)).countdown(at)
        }),
        on_node(7, 1800, 200, |node, at| node.scale_in(at)),
        on_node(8, 2400, 700, |node, at| node.crosshair(at)),
        on_segment(segment_ids["flash"], 1800, 300, |segment, at| {
            segment.flash(at)
        }),
        on_segment(segment_ids["comet_once"], 2000, 0, {
            let direction = Rc::clone(&direction);
            move |segment, at| segment.direction(direction.get()).comet_once(at)
        }),
        on_segment(segment_ids["wipe"], 2200, 600, {
            let direction = Rc::clone(&direction);
            move |segment, at| segment.direction(direction.get()).wipe(at)
        }),
    ];

    let mut theme = Theme::default();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 640.0]),
        ..Default::default()
    };

    eframe::run_ui_native(
        "egui-map: node and segment animations",
        options,
        move |ui, _frame| {
            let now = Instant::now();
            for repeater in &mut repeaters {
                repeater.tick(&mut map, now);
            }
            egui::Panel::top("controls").show(ui, |ui| {
                ui.horizontal(|ui| {
                    egui::ComboBox::from_label("Theme")
                        .selected_text(format!("{theme:?}"))
                        .show_ui(ui, |ui| {
                            for choice in THEMES {
                                let label = format!("{choice:?}");
                                if ui.selectable_value(&mut theme, choice, label).changed() {
                                    map.set_theme(Rc::new(choice));
                                }
                            }
                        });
                    // Light, dark or the system's: the map resolves its colors
                    // for whichever mode egui is in.
                    egui::widgets::global_theme_preference_buttons(ui);
                    ui.separator();
                    // The direction of the segment effects. Exclusive, so
                    // radio buttons. The lasting effects are started again
                    // with the new one; the event ones read it when they fire.
                    ui.label("Direction");
                    let mut chosen = direction.get();
                    ui.radio_value(&mut chosen, CometDirection::Forward, "Forward");
                    ui.radio_value(&mut chosen, CometDirection::Reverse, "Reverse");
                    if chosen != direction.get() {
                        direction.set(chosen);
                        start_lasting(&mut map, &segment_ids, chosen);
                    }
                });
            });
            egui::CentralPanel::default().show(ui, |ui| {
                let response = ui.add(&mut map);
                // The node under the pointer: a tooltip with its name and id.
                if let Some(id) = map.hovered_node()
                    && let Some(text) = names.get(&id)
                {
                    response.on_hover_text_at_pointer(text);
                }
            });
        },
    )
}

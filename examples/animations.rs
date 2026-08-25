//! Demonstrates the built-in node and segment animations reached through
//! `Map::node` and `Map::segment` -- no custom template involved, just the
//! effects the widget already knows how to draw. Every node and every
//! segment demonstrates exactly one animation, named after it, so each one
//! is easy to tell apart on screen instead of a single node/segment cycling
//! through several effects over time.
//!
//! Node effects, one node each:
//!
//! - Lasting (visible immediately and forever): `Halo`, `Blink`, `Orbit`.
//! - Event (re-fired on its own repeating timer): `Pulse`, `Ripple`,
//!   `Countdown`, `ScaleIn`, `Crosshair`.
//!
//! Segment effects, one segment each:
//!
//! - Lasting: `Halo <-> Pulse` carries a `comet`, `Blink <-> Ripple` a `dash`,
//!   `Glow1 <-> Glow2` a `glow_band`, `Chevron1 <-> Chevron2` `chevrons`.
//! - Event: `Orbit <-> Countdown` is `flash`ed, `ScaleIn <-> Hub1` gets a
//!   `comet_once` travelling `Forward`, `Crosshair <-> Hub2` a `comet_once`
//!   travelling `Reverse` -- the two make the direction argument easy to see:
//!   both fire on the same timer, but the dot starts from opposite ends --
//!   and `Wipe1 <-> Wipe2` gets a `wipe`.
//!
//! `Hub1`, `Hub2`, `Wipe1`, `Wipe2`, `Glow1`, `Glow2`, `Chevron1` and
//! `Chevron2` carry no animation of their own -- pure segment endpoints,
//! named after the segment effect they anchor so it stays legible on screen
//! without a second node also blinking or pulsing on its own timer.
//!
//! Every event timer fires independently and at a different period, so
//! several different animations are usually playing at once rather than
//! everything ticking in lockstep.
//!
//! Run with: cargo run --example animations

use egui_map::map::Map;
use egui_map::map::objects::{CometDirection, MapPoint, MapSegment};
use std::collections::HashMap;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------- node fire

fn fire_pulse(map: &mut Map, id: usize, at: Instant) {
    if let Some(node) = map.node(id) {
        node.pulse(at);
    }
}
fn fire_ripple(map: &mut Map, id: usize, at: Instant) {
    if let Some(node) = map.node(id) {
        node.ripple(at);
    }
}
fn fire_countdown(map: &mut Map, id: usize, at: Instant) {
    if let Some(node) = map.node(id) {
        node.countdown(at);
    }
}
fn fire_scale_in(map: &mut Map, id: usize, at: Instant) {
    if let Some(node) = map.node(id) {
        node.scale_in(at);
    }
}
fn fire_crosshair(map: &mut Map, id: usize, at: Instant) {
    if let Some(node) = map.node(id) {
        node.crosshair(at);
    }
}

// ------------------------------------------------------------- segment fire

fn fire_flash(map: &mut Map, id: (usize, usize), at: Instant) {
    if let Some(segment) = map.segment(id) {
        segment.flash(at);
    }
}
fn fire_comet_once_forward(map: &mut Map, id: (usize, usize), at: Instant) {
    if let Some(segment) = map.segment(id) {
        segment.comet_once(at, CometDirection::Forward);
    }
}
fn fire_comet_once_reverse(map: &mut Map, id: (usize, usize), at: Instant) {
    if let Some(segment) = map.segment(id) {
        segment.comet_once(at, CometDirection::Reverse);
    }
}
fn fire_wipe(map: &mut Map, id: (usize, usize), at: Instant) {
    if let Some(segment) = map.segment(id) {
        segment.wipe(at);
    }
}

// ------------------------------------------------------------- repeaters

/// Re-fires the same node event effect on a timer of its own. Deliberately
/// *not* a cycle through several effects -- each repeater, and so each node,
/// only ever plays the one effect it was built with.
struct NodeRepeater {
    node_id: usize,
    period: Duration,
    last_fired: Instant,
    fire: fn(&mut Map, usize, Instant),
}

impl NodeRepeater {
    fn new(
        node_id: usize,
        period: Duration,
        first_delay: Duration,
        fire: fn(&mut Map, usize, Instant),
    ) -> Self {
        Self {
            node_id,
            period,
            last_fired: Instant::now() - period + first_delay,
            fire,
        }
    }

    fn tick(&mut self, map: &mut Map, now: Instant) {
        if now.duration_since(self.last_fired) < self.period {
            return;
        }
        (self.fire)(map, self.node_id, now);
        self.last_fired = now;
    }
}

/// The segment counterpart of [`NodeRepeater`] -- same one-effect-only rule.
struct SegmentRepeater {
    segment_id: (usize, usize),
    period: Duration,
    last_fired: Instant,
    fire: fn(&mut Map, (usize, usize), Instant),
}

impl SegmentRepeater {
    fn new(
        segment_id: (usize, usize),
        period: Duration,
        first_delay: Duration,
        fire: fn(&mut Map, (usize, usize), Instant),
    ) -> Self {
        Self {
            segment_id,
            period,
            last_fired: Instant::now() - period + first_delay,
            fire,
        }
    }

    fn tick(&mut self, map: &mut Map, now: Instant) {
        if now.duration_since(self.last_fired) < self.period {
            return;
        }
        (self.fire)(map, self.segment_id, now);
        self.last_fired = now;
    }
}

fn main() -> eframe::Result<()> {
    // 1. Nodes, keyed by id and named after the one effect each demonstrates.
    let mut points: HashMap<usize, MapPoint> = HashMap::new();
    for (id, name, x, y) in [
        (1, "Halo", 160.0, 0.0),
        (2, "Pulse", 120.0, 100.0),
        (3, "Blink", 20.0, 160.0),
        (4, "Ripple", -90.0, 120.0),
        (5, "Orbit", -160.0, 0.0),
        (6, "Countdown", -110.0, -110.0),
        (7, "ScaleIn", 0.0, -160.0),
        (8, "Crosshair", 110.0, -110.0),
        (9, "Hub1", 40.0, -230.0),
        (10, "Hub2", 200.0, -190.0),
        (11, "Wipe1", 300.0, 50.0),
        (12, "Wipe2", 380.0, 120.0),
        (13, "Glow1", 300.0, -260.0),
        (14, "Glow2", 380.0, -330.0),
        (15, "Chevron1", -260.0, 220.0),
        (16, "Chevron2", -340.0, 290.0),
    ] {
        let mut point = MapPoint::new(id, [x, y]);
        point.set_name(name.to_string());
        points.insert(id, point);
    }

    // 2. Register each connection id on both endpoint nodes.
    for (line_id, endpoints) in [
        ((1, 2), [1, 2]),
        ((3, 4), [3, 4]),
        ((5, 6), [5, 6]),
        ((7, 9), [7, 9]),
        ((8, 10), [8, 10]),
        ((11, 12), [11, 12]),
        ((13, 14), [13, 14]),
        ((15, 16), [15, 16]),
    ] {
        for id in endpoints {
            points.get_mut(&id).unwrap().connections.push(line_id);
        }
    }

    let mut map = Map::new();
    map.add_hashmap_points(points);
    map.add_lines(vec![
        MapSegment::new((1, 2), [160.0, 0.0], [120.0, 100.0]),
        MapSegment::new((3, 4), [20.0, 160.0], [-90.0, 120.0]),
        MapSegment::new((5, 6), [-160.0, 0.0], [-110.0, -110.0]),
        MapSegment::new((7, 9), [0.0, -160.0], [40.0, -230.0]),
        MapSegment::new((8, 10), [110.0, -110.0], [200.0, -190.0]),
        MapSegment::new((11, 12), [300.0, 50.0], [380.0, 120.0]),
        MapSegment::new((13, 14), [300.0, -260.0], [380.0, -330.0]),
        MapSegment::new((15, 16), [-260.0, 220.0], [-340.0, 290.0]),
    ]);

    // Lasting effects: set once, visible for as long as the app runs.
    map.node(1).expect("Halo is loaded").halo();
    map.node(3).expect("Blink is loaded").blink();
    map.node(5).expect("Orbit is loaded").orbit();
    map.segment((1, 2))
        .expect("Halo <-> Pulse is loaded")
        .comet();
    map.segment((3, 4))
        .expect("Blink <-> Ripple is loaded")
        .dash();
    map.segment((13, 14))
        .expect("Glow1 <-> Glow2 is loaded")
        .glow_band();
    map.segment((15, 16))
        .expect("Chevron1 <-> Chevron2 is loaded")
        .chevrons();

    // Event effects: each on its own independent, non-synchronized timer, so
    // several different animations are usually playing at once.
    let mut node_repeaters = vec![
        NodeRepeater::new(2, Duration::from_millis(2200), Duration::ZERO, fire_pulse),
        NodeRepeater::new(
            4,
            Duration::from_millis(2600),
            Duration::from_millis(400),
            fire_ripple,
        ),
        NodeRepeater::new(
            6,
            Duration::from_millis(3000),
            Duration::from_millis(900),
            fire_countdown,
        ),
        NodeRepeater::new(
            7,
            Duration::from_millis(1800),
            Duration::from_millis(200),
            fire_scale_in,
        ),
        NodeRepeater::new(
            8,
            Duration::from_millis(2400),
            Duration::from_millis(700),
            fire_crosshair,
        ),
    ];
    let mut segment_repeaters = vec![
        SegmentRepeater::new(
            (5, 6),
            Duration::from_millis(1800),
            Duration::from_millis(300),
            fire_flash,
        ),
        SegmentRepeater::new(
            (7, 9),
            Duration::from_millis(2000),
            Duration::ZERO,
            fire_comet_once_forward,
        ),
        SegmentRepeater::new(
            (8, 10),
            Duration::from_millis(2000),
            Duration::ZERO,
            fire_comet_once_reverse,
        ),
        SegmentRepeater::new(
            (11, 12),
            Duration::from_millis(2200),
            Duration::from_millis(600),
            fire_wipe,
        ),
    ];

    eframe::run_ui_native(
        "egui-map: node and segment animations",
        eframe::NativeOptions::default(),
        move |ui, _frame| {
            let now = Instant::now();
            for repeater in &mut node_repeaters {
                repeater.tick(&mut map, now);
            }
            for repeater in &mut segment_repeaters {
                repeater.tick(&mut map, now);
            }
            ui.add(&mut map);
        },
    )
}

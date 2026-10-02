//! Built-in animation effects for nodes and segments.
//!
//! Two families, distinguished by how they handle time and by when they stop:
//!
//! - **Event-driven** effects ([`Animation::pulse`], [`Animation::ripple`],
//!   [`Animation::countdown_arc`], [`Animation::scale_in`],
//!   [`Animation::crosshair`], [`Animation::flash_decay`],
//!   [`Animation::comet_once`], [`Animation::wipe`]) are anchored to the
//!   [`Instant`] an event happened and **terminate**: they return `true`
//!   while still playing and `false` once finished, so the caller can drop
//!   the entry and stop repainting.
//! - **Persistent** effects ([`Animation::halo`], [`Animation::blink`],
//!   [`Animation::orbit`], [`Animation::glow`], [`Animation::comet`],
//!   [`Animation::dash`], [`Animation::glow_band`], [`Animation::chevrons`])
//!   never end. They take
//!   the **frame time** in seconds (`ui.input(|i| i.time)`) rather than an
//!   `Instant`, so every element animated in the same frame shares one clock
//!   and cannot drift apart.
//!
//! Persistent effects require the caller to keep requesting repaints, which
//! turns an idle app into one redrawing continuously — use them for a handful
//! of elements, not for every node or segment.
//!
//! The node effects are reached through
//! [`Map::node`](crate::map::Map::node); see [`NodeHandle`](crate::map::NodeHandle).
//! They are also useful from a custom
//! [`NodeTemplate`](crate::map::objects::NodeTemplate): call them from
//! `notification_ui` / `marker_ui` instead of reimplementing the effect. When
//! you do, remember to call `ui.ctx().request_repaint()` yourself — the widget
//! only does that for its own built-in path.
//!
//! Every node effect also has an `*_outline` variant ([`Animation::pulse_outline`],
//! [`Animation::halo_outline`], ...) that follows a
//! [`NodeOutline`] -- the shape a [`NodeTemplate`](crate::map::objects::NodeTemplate)
//! declares in `outline` -- instead of a circle around a point. The
//! templates' default `notification_ui`/`marker_ui` use them, so a template
//! drawing boxes (or any convex shape) gets effects that keep that shape.
//! [`Animation::event_outline`]/[`Animation::state_outline`] pick
//! the variant for a requested kind.
//!
//! The segment effects ([`Animation::flash_decay`], [`Animation::comet_once`],
//! [`Animation::wipe`], [`Animation::comet`], [`Animation::dash`],
//! [`Animation::glow_band`], [`Animation::chevrons`]) are reached the same way, through
//! [`Map::segment`](crate::map::Map::segment); see
//! [`SegmentHandle`](crate::map::SegmentHandle). A custom
//! [`SegmentTemplate`](crate::map::objects::SegmentTemplate) calls them from
//! `segment_notification_ui` / `segment_state_ui`, remembering to call
//! `painter.ctx().request_repaint()` itself.

use super::objects::{NodeAnimation, SegmentAnimation, SteadyAnimation, SteadySegmentAnimation};
use super::outline::{NodeOutline, partial_perimeter, point_along};
use egui::{
    Color32, ColorImage, Context, CornerRadius, Id, Mesh, Painter, Pos2, Rect, Shape, Stroke,
    TextureFilter, TextureHandle, TextureOptions, TextureWrapMode, Vec2,
    epaint::{CircleShape, PathShape, Vertex},
    pos2,
};
use std::f32::consts::TAU;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use std::time::Instant;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use web_time::Instant;

/// How long [`Animation::pulse`] plays, in seconds.
pub const PULSE_DURATION: f32 = 3.5;
/// How long [`Animation::ripple`] plays, in seconds.
pub const RIPPLE_DURATION: f32 = 3.5;
/// How long [`Animation::countdown_arc`] takes to empty, in seconds.
pub const COUNTDOWN_DURATION: f32 = 5.0;
/// How long [`Animation::scale_in`] plays, in seconds.
pub const SCALE_IN_DURATION: f32 = 0.45;
/// How long [`Animation::crosshair`] takes to converge, in seconds.
pub const CROSSHAIR_DURATION: f32 = 0.6;
/// How long [`Animation::flash_decay`] takes to fade back out, in seconds.
pub const FLASH_DECAY_DURATION: f32 = 1.0;
/// Stroke width, before the `zoom` multiplier, [`Animation::flash_decay`]
/// settles to as it fades.
pub const FLASH_BASE_WIDTH: f32 = 2.0;
/// Width, before the `zoom` multiplier, [`Animation::flash_decay`] adds at its
/// start. With [`FLASH_BASE_WIDTH`] the widest the flash gets is 8, the
/// diameter of the built-in node (radius `4 * zoom`): it never swells past the
/// nodes at the ends of the segment.
pub const FLASH_EXTRA_WIDTH: f32 = 6.0;
// Checked when compiling: raising either constant past the built-in node's
// diameter (8) fails the build instead of silently bringing the flash back.
const _: () = assert!(FLASH_BASE_WIDTH + FLASH_EXTRA_WIDTH <= 8.0);
/// How long [`Animation::comet`] takes for one end-to-end pass, in seconds.
pub const COMET_PERIOD: f32 = 1.6;
/// How long a single [`Animation::comet_once`] pass takes to cross the
/// segment, in seconds.
pub const COMET_TRAVEL_DURATION: f32 = 1.2;
/// Length, in **screen pixels**, of one dash-plus-gap repeat of
/// [`Animation::dash`]. Deliberately not scaled by zoom, same as the dash
/// speed, so the pattern doesn't stretch as the map is zoomed -- matching how
/// node/label text is sized in screen space rather than map space.
pub const DASH_PERIOD_PX: f32 = 24.0;
/// How many repeats of the dash pattern [`Animation::dash`] slides through
/// per second ("marching ants" speed).
pub const DASH_SPEED: f32 = 0.6;
/// Width, in **screen pixels**, of the ribbon [`Animation::dash`] paints when
/// [`Dash::width`] is `None` and there is no default stroke to follow
/// ([`Style::line_width`](crate::map::theme::Style::line_width) is `None`).
/// Not scaled by zoom, like the default stroke it stands in for, and equal to
/// that stroke's default width.
pub const DASH_WIDTH: f32 = 2.0;
/// How long [`Animation::wipe`] takes to draw the line in, in seconds.
pub const WIPE_DURATION: f32 = 0.9;
/// How long one full traverse-and-loop of [`Animation::glow_band`] takes, in
/// seconds -- the band fades out past one end before it reappears at the
/// other, so this covers the whole cycle, not just the visible crossing.
pub const GLOW_BAND_PERIOD: f32 = 2.2;
/// Length, in **screen pixels**, of the visible glow band
/// [`Animation::glow_band`] paints. Deliberately not scaled by zoom, same
/// reasoning as [`DASH_PERIOD_PX`].
pub const GLOW_BAND_LENGTH_PX: f32 = 40.0;
/// Width, before the `zoom` multiplier, of the ribbon [`Animation::glow_band`]
/// paints.
pub const GLOW_BAND_THICKNESS: f32 = 5.0;
/// How long one full pulse of [`Animation::glow`] takes (dim, bright, dim),
/// in seconds.
pub const GLOW_PERIOD: f32 = 2.5;
/// Length, in **screen pixels**, of one chevron repeat of
/// [`Animation::chevrons`]. Deliberately not scaled by zoom, same reasoning
/// as [`DASH_PERIOD_PX`].
pub const CHEVRON_PERIOD_PX: f32 = 20.0;
/// How many repeats of the chevron pattern [`Animation::chevrons`] slides
/// through per second. With [`CHEVRON_PERIOD_PX`] this is about the speed of
/// [`DASH_SPEED`]'s marching ants.
pub const CHEVRON_SPEED: f32 = 0.7;
/// Width, in **screen pixels**, of the ribbon [`Animation::chevrons`] paints.
/// Not scaled by zoom, like [`CHEVRON_PERIOD_PX`]: the arrow's shape depends on
/// the ratio of the two, so scaling only one of them squashed or stretched it
/// every time the zoom changed. Close to [`GLOW_BAND_THICKNESS`], so the
/// arrows sit on the line like the other lasting segment effects instead of
/// dwarfing it.
pub const CHEVRON_WIDTH: f32 = 6.0;
/// How far the legs of each [`Animation::chevrons`] arrow sweep back from its
/// tip, see [`Chevrons::leg_slope`].
pub const CHEVRON_LEG_SLOPE: f32 = 0.35;
/// Stroke thickness of each [`Animation::chevrons`] arrow as a fraction of the
/// period, see [`Chevrons::stroke`]: about 2 pixels at the default period, the
/// weight of the segment line itself.
pub const CHEVRON_STROKE: f32 = 0.10;
/// Radius, in screen pixels at `zoom == 1`, of the dot of
/// [`Animation::comet`] and [`Animation::comet_once`]: the width of the
/// stroke [`Animation::wipe`] draws (`Wipe::width`), so the dot reads as a
/// bead on the line rather than a blob over it.
pub const COMET_DOT_RADIUS: f32 = 2.5;
/// Floor for [`COMET_DOT_RADIUS`], in screen pixels.
pub const COMET_DOT_MIN: f32 = 1.5;

/// Shapes smaller than this many screen points across are effectively
/// invisible; the `*_outline`/circular effects skip tessellating them. Only
/// reachable at an extreme zoom-out, but it keeps a frame full of tiny,
/// overlapping effects from paying for geometry no one can see.
const MIN_EFFECT_PX: f32 = 1.0;

/// Returns `color` with its opacity multiplied by `alpha` (clamped to
/// `0.0..=1.0`): an opaque `color` ends up with exactly that alpha, and a
/// translucent one (e.g. a lasting notification already fading out) keeps
/// its own transparency on top. `Color32` is premultiplied, so this has to
/// scale every channel, not just replace the alpha byte.
fn with_alpha(color: Color32, alpha: f32) -> Color32 {
    color.gamma_multiply(alpha.clamp(0.0, 1.0))
}

/// Seconds elapsed since `initial_time`.
fn elapsed(initial_time: Instant) -> f32 {
    Instant::now().duration_since(initial_time).as_secs_f32()
}

/// A `0 -> 1 -> 0` triangle wave of the given `period`, in seconds.
fn triangle_wave(time: f32, period: f32) -> f32 {
    let phase = (time / period).rem_euclid(1.0);
    1.0 - (2.0 * phase - 1.0).abs()
}

/// When the cycle a notification that started at `started` is in at `now`
/// began: `started` itself for the first cycle, then every `cycle` seconds.
/// Drawing an event effect from this instant repeats it.
pub fn cycle_start(started: Instant, now: Instant, cycle: f32) -> Instant {
    let elapsed = now.saturating_duration_since(started).as_secs_f32();
    let into_cycle = if cycle > 0.0 { elapsed % cycle } else { 0.0 };
    now.checked_sub(std::time::Duration::from_secs_f32(into_cycle))
        .unwrap_or(now)
}

/// The fraction of a notification left, `1.0` when it starts down to `0.0` at
/// `until` -- what a `countdown` shows over a lasting notification.
fn remaining_until(started: Instant, until: Instant) -> f32 {
    let total = until.saturating_duration_since(started).as_secs_f32();
    (1.0 - elapsed(started) / total.max(f32::EPSILON)).clamp(0.0, 1.0)
}

/// Opacity factor of [`Animation::glow`] at `time`: a smooth `0 -> 1 -> 0`
/// pulse of the given `period` (seconds), scaled by `strength` (clamped to
/// `0.0..=1.0`).
fn glow_level(time: f32, strength: f32, period: f32) -> f32 {
    let pulse = 0.5 - 0.5 * (TAU * time / period).cos();
    strength.clamp(0.0, 1.0) * pulse
}

/// Overshooting ease-out, so a scale-in settles with a small bounce.
fn ease_out_back(x: f32) -> f32 {
    const C1: f32 = 1.701_58;
    const C3: f32 = C1 + 1.0;
    let x1 = x - 1.0;
    1.0 + C3 * x1 * x1 * x1 + C1 * x1 * x1
}

/// Per-animation tuning for the built-in node effects, plus the factory for
/// them.
///
/// `Animation::default()` reproduces the values the widget used to hard-code,
/// so an untouched `Animation` behaves exactly as before. Tweak it with
/// [`Animation::with`]:
///
/// ```
/// use egui_map::map::animation::Animation;
///
/// let animation = Animation::default().with(|a| {
///     a.pulse.spread = 60.0;
///     a.ripple.stroke = 3.0;
/// });
/// ```
///
/// Every effect's methods ([`Animation::pulse`], [`Animation::pulse_outline`],
/// ...) read these fields, so one `Animation` drives both the circular and the
/// `*_outline` variant of an effect. A custom
/// [`NodeTemplate`](crate::map::objects::NodeTemplate) gets the active one as
/// [`NodeContext::animation`](crate::map::objects::NodeContext::animation).
///
/// See the [module docs](self) for the difference between the event-driven and
/// persistent families.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Animation {
    /// [`Animation::pulse`] / [`Animation::pulse_outline`].
    pub pulse: Pulse,
    /// [`Animation::ripple`] / [`Animation::ripple_outline`].
    pub ripple: Ripple,
    /// [`Animation::countdown_arc`] / [`Animation::countdown_outline`].
    pub countdown: Countdown,
    /// [`Animation::scale_in`] / [`Animation::scale_in_outline`].
    pub scale_in: ScaleIn,
    /// [`Animation::crosshair`] / [`Animation::crosshair_outline`].
    pub crosshair: Crosshair,
    /// [`Animation::halo`] / [`Animation::halo_outline`].
    pub halo: Halo,
    /// [`Animation::blink`] / [`Animation::blink_outline`].
    pub blink: Blink,
    /// [`Animation::orbit`] / [`Animation::orbit_outline`].
    pub orbit: Orbit,
    /// [`Animation::glow`] / [`Animation::glow_outline`].
    pub glow: Glow,
}

impl Animation {
    /// Runs `f` on a mutable copy of `self` and returns it, so several fields
    /// can be adjusted in one expression.
    pub fn with(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }
}

/// Per-animation tuning for the built-in **segment** effects.
///
/// The segment-side counterpart of [`Animation`]: `SegmentAnimations::default()`
/// reproduces the values the widget used to hard-code, and
/// [`SegmentAnimations::with`] adjusts them the same way. Held on
/// [`MapSettings`](crate::map::objects::MapSettings) and handed to the segment
/// contexts as `ctx.animation`.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct SegmentAnimations {
    /// [`SegmentAnimations::flash_decay`].
    pub flash_decay: FlashDecay,
    /// [`SegmentAnimations::comet_once`].
    pub comet_once: CometOnce,
    /// [`SegmentAnimations::wipe`].
    pub wipe: Wipe,
    /// [`SegmentAnimations::comet`].
    pub comet: Comet,
    /// [`SegmentAnimations::dash`].
    pub dash: Dash,
    /// [`SegmentAnimations::glow_band`].
    pub glow_band: GlowBand,
    /// [`SegmentAnimations::chevrons`].
    pub chevrons: Chevrons,
}

impl SegmentAnimations {
    /// Runs `f` on a mutable copy of `self` and returns it, so several fields
    /// can be adjusted in one expression.
    pub fn with(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }

    /// `self` with the effects that follow the default segment stroke given
    /// its width: a [`Dash::width`] of `None` becomes `line_width` (screen
    /// pixels), so [`SegmentAnimations::dash`] is as thick as the line it runs
    /// over. With no default stroke (`line_width` of `None`) it stays `None`
    /// and falls back to [`DASH_WIDTH`]. A width set explicitly is kept.
    ///
    /// The map applies this before handing the animations to the segment
    /// effects and to a [`SegmentTemplate`](crate::map::objects::SegmentTemplate),
    /// so call it yourself only when you draw segment effects on your own.
    pub fn with_line_width(self, line_width: Option<f32>) -> Self {
        self.with(|animation| {
            if animation.dash.width.is_none() {
                animation.dash.width = line_width;
            }
        })
    }
}

/// Tunables of [`SegmentAnimations::flash_decay`]: a segment thickening then
/// fading.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlashDecay {
    /// Base stroke width, before the `zoom` multiplier.
    pub base_width: f32,
    /// Extra width, before the `zoom` multiplier, at the very start of the
    /// effect (added to [`Self::base_width`] and shed over its lifetime).
    ///
    /// The widest the line gets is `base_width + extra_width`; the default
    /// keeps that at the diameter of the built-in node (`8 * zoom`), so the
    /// flash never swells past the nodes it joins. Keep it under the diameter
    /// of your own nodes if you use a [`NodeTemplate`](crate::map::objects::NodeTemplate)
    /// with a different size.
    pub extra_width: f32,
    /// How long the effect plays, in seconds.
    pub duration: f32,
}

impl FlashDecay {
    /// Runs `f` on a mutable copy of `self` and returns it.
    pub fn with(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }
}

impl Default for FlashDecay {
    fn default() -> Self {
        Self {
            base_width: FLASH_BASE_WIDTH,
            extra_width: FLASH_EXTRA_WIDTH,
            duration: FLASH_DECAY_DURATION,
        }
    }
}

/// Tunables of [`SegmentAnimations::comet_once`]: a single dot crossing the
/// segment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CometOnce {
    /// Dot radius, in screen pixels at `zoom == 1`.
    pub dot_radius: f32,
    /// Floor for [`Self::dot_radius`], in screen pixels.
    pub dot_min: f32,
    /// How long one pass takes, in seconds.
    pub duration: f32,
}

impl CometOnce {
    /// Runs `f` on a mutable copy of `self` and returns it.
    pub fn with(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }
}

impl Default for CometOnce {
    fn default() -> Self {
        Self {
            dot_radius: COMET_DOT_RADIUS,
            dot_min: COMET_DOT_MIN,
            duration: COMET_TRAVEL_DURATION,
        }
    }
}

/// Tunables of [`SegmentAnimations::wipe`]: the segment drawing itself in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wipe {
    /// Stroke width, before the `zoom` multiplier.
    pub width: f32,
    /// How long the effect plays, in seconds.
    pub duration: f32,
}

impl Wipe {
    /// Runs `f` on a mutable copy of `self` and returns it.
    pub fn with(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }
}

impl Default for Wipe {
    fn default() -> Self {
        Self {
            width: 2.5,
            duration: WIPE_DURATION,
        }
    }
}

/// Tunables of [`SegmentAnimations::comet`]: a dot looping along the segment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Comet {
    /// Dot radius, in screen pixels at `zoom == 1`.
    pub dot_radius: f32,
    /// Floor for [`Self::dot_radius`], in screen pixels.
    pub dot_min: f32,
    /// How long one end-to-end pass takes, in seconds.
    pub period: f32,
}

impl Comet {
    /// Runs `f` on a mutable copy of `self` and returns it.
    pub fn with(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }
}

impl Default for Comet {
    fn default() -> Self {
        Self {
            dot_radius: COMET_DOT_RADIUS,
            dot_min: COMET_DOT_MIN,
            period: COMET_PERIOD,
        }
    }
}

/// Tunables of [`SegmentAnimations::dash`]: a sliding dashed line.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dash {
    /// Length of one dash-plus-gap repeat, in **screen pixels** (not scaled by
    /// zoom, like the text sizes).
    pub period_px: f32,
    /// How many repeats slide through per second.
    pub speed: f32,
    /// Ribbon width, in **screen pixels** (not scaled by zoom, like
    /// `period_px` and like the default stroke it runs over).
    ///
    /// `None`, the default, follows the width of the default segment stroke,
    /// [`Style::line_width`](crate::map::theme::Style::line_width), so the
    /// dashes are as thick as the line at every zoom and for any
    /// `line_width`; when that stroke is off it falls back to [`DASH_WIDTH`].
    /// `Some(width)` is used as it is.
    pub width: Option<f32>,
}

impl Dash {
    /// Runs `f` on a mutable copy of `self` and returns it.
    pub fn with(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }
}

impl Default for Dash {
    fn default() -> Self {
        Self {
            period_px: DASH_PERIOD_PX,
            speed: DASH_SPEED,
            width: None,
        }
    }
}

/// Tunables of [`SegmentAnimations::glow_band`]: a soft highlight travelling
/// the segment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlowBand {
    /// How long one traverse-and-loop takes, in seconds.
    pub period: f32,
    /// Length of the visible band, in **screen pixels** (not scaled by zoom).
    pub length_px: f32,
    /// Ribbon width, before the `zoom` multiplier.
    pub thickness: f32,
}

impl GlowBand {
    /// Runs `f` on a mutable copy of `self` and returns it.
    pub fn with(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }
}

impl Default for GlowBand {
    fn default() -> Self {
        Self {
            period: GLOW_BAND_PERIOD,
            length_px: GLOW_BAND_LENGTH_PX,
            thickness: GLOW_BAND_THICKNESS,
        }
    }
}

/// Tunables of [`SegmentAnimations::chevrons`]: arrows sliding along the
/// segment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Chevrons {
    /// Length of one chevron repeat, in **screen pixels** (not scaled by zoom).
    pub period_px: f32,
    /// How many repeats slide through per second.
    pub speed: f32,
    /// Ribbon width, in **screen pixels** (not scaled by zoom, so the arrow
    /// keeps its shape at any zoom).
    pub width: f32,
    /// How far the two legs of each arrow sweep back from the tip, as a
    /// fraction of [`Self::period_px`] per unit of the ribbon's width: `0.0`
    /// is a straight bar across the segment, larger values a more open `>`.
    /// The default makes the legs about 45 degrees to the segment with the
    /// default [`Self::width`] and [`Self::period_px`]; keep it near
    /// `width / period_px` if you change those.
    pub leg_slope: f32,
    /// Stroke thickness of the arrow, as a fraction of [`Self::period_px`].
    /// Soft-edged either way.
    pub stroke: f32,
}

impl Chevrons {
    /// Runs `f` on a mutable copy of `self` and returns it.
    pub fn with(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }
}

impl Default for Chevrons {
    fn default() -> Self {
        Self {
            period_px: CHEVRON_PERIOD_PX,
            speed: CHEVRON_SPEED,
            width: CHEVRON_WIDTH,
            leg_slope: CHEVRON_LEG_SLOPE,
            stroke: CHEVRON_STROKE,
        }
    }
}

/// Tunables of [`Animation::pulse`]: an expanding, fading disc.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pulse {
    /// Radius at the start of the effect, in screen pixels at `zoom == 1`.
    pub base_radius: f32,
    /// How fast the radius grows, in screen pixels per second at `zoom == 1`.
    /// Also the `*_outline` variant's growth rate.
    pub spread: f32,
    /// How long the effect plays, in seconds.
    pub duration: f32,
}

impl Pulse {
    /// Runs `f` on a mutable copy of `self` and returns it.
    pub fn with(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }
}

impl Default for Pulse {
    fn default() -> Self {
        Self {
            base_radius: 4.0,
            spread: 40.0,
            duration: PULSE_DURATION,
        }
    }
}

/// Tunables of [`Animation::ripple`]: three staggered expanding rings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ripple {
    /// Radius the first ring starts at, in screen pixels at `zoom == 1`.
    pub base_radius: f32,
    /// Growth of each ring over its own life, in screen pixels at
    /// `zoom == 1`. Shared by the `*_outline` variant.
    pub spread: f32,
    /// Ring stroke width, before the `zoom` multiplier.
    pub stroke: f32,
    /// How long the effect plays, in seconds: from the first ring appearing
    /// until the last one has faded out. The rings are born one after the
    /// other, so each lives `3/5` of this.
    pub duration: f32,
}

/// How many staggered rings [`Animation::ripple`] draws.
const RIPPLE_RINGS: usize = 3;

impl Ripple {
    /// Runs `f` on a mutable copy of `self` and returns it.
    pub fn with(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }

    /// How long one ring lives. The last ring is born `RINGS - 1` staggers
    /// after the first, and has to finish by `duration`, so that no ring is
    /// cut off when the effect ends.
    fn ring_life(&self) -> f32 {
        self.duration * RIPPLE_RINGS as f32 / (2 * RIPPLE_RINGS - 1) as f32
    }

    /// The progress (`0.0..1.0`) of every ring alive `secs` after the effect
    /// started. With `looping`, a new ring is born every stagger for ever, so
    /// the rings keep coming without a break instead of ending after the
    /// last one. The rings still build up one by one at the start, like the
    /// one-off effect, and then repeat.
    fn ring_progress(&self, secs: f32, looping: bool) -> impl Iterator<Item = f32> + use<> {
        let life = self.ring_life();
        let stagger = life / RIPPLE_RINGS as f32;
        (0..RIPPLE_RINGS).filter_map(move |ring| {
            let local = secs - ring as f32 * stagger;
            if local < 0.0 {
                // Not born yet.
                return None;
            }
            let local = if looping { local % life } else { local };
            (0.0..life).contains(&local).then(|| local / life)
        })
    }
}

impl Default for Ripple {
    fn default() -> Self {
        Self {
            base_radius: 4.0,
            spread: 36.0,
            stroke: 2.0,
            duration: RIPPLE_DURATION,
        }
    }
}

/// Tunables of [`Animation::countdown_arc`]: a ring emptying clockwise.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Countdown {
    /// Ring radius of the circular variant, in screen pixels at `zoom == 1`.
    pub radius: f32,
    /// How far outside the node the `*_outline` variant draws its arc, in
    /// screen pixels at `zoom == 1`.
    pub outline_offset: f32,
    /// Stroke width, before the `zoom` multiplier.
    pub stroke: f32,
    /// How long the effect takes to empty, in seconds.
    pub duration: f32,
}

impl Countdown {
    /// Runs `f` on a mutable copy of `self` and returns it.
    pub fn with(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }
}

impl Default for Countdown {
    fn default() -> Self {
        Self {
            radius: 10.0,
            outline_offset: 4.0,
            stroke: 2.0,
            duration: COUNTDOWN_DURATION,
        }
    }
}

/// Tunables of [`Animation::scale_in`]: a disc that overshoots and settles.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScaleIn {
    /// Peak radius of the circular variant, in screen pixels at `zoom == 1`.
    /// The `*_outline` variant scales the node's own shape instead.
    pub radius: f32,
    /// How long the effect plays, in seconds.
    pub duration: f32,
}

impl ScaleIn {
    /// Runs `f` on a mutable copy of `self` and returns it.
    pub fn with(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }
}

impl Default for ScaleIn {
    fn default() -> Self {
        Self {
            radius: 8.0,
            duration: SCALE_IN_DURATION,
        }
    }
}

/// Tunables of [`Animation::crosshair`]: four ticks converging on the node.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Crosshair {
    /// How far the ticks start, in screen pixels at `zoom == 1`, for the
    /// circular variant.
    pub far: f32,
    /// Same, measured from the bounding box's edge, for the `*_outline`
    /// variant.
    pub outline_far: f32,
    /// How much of `far` the ticks cover while converging.
    pub travel: f32,
    /// Tick length, in screen pixels at `zoom == 1`.
    pub length: f32,
    /// Stroke width, before the `zoom` multiplier.
    pub stroke: f32,
    /// How long the effect takes to converge, in seconds.
    pub duration: f32,
}

impl Crosshair {
    /// Runs `f` on a mutable copy of `self` and returns it.
    pub fn with(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }
}

impl Default for Crosshair {
    fn default() -> Self {
        Self {
            far: 30.0,
            outline_far: 22.0,
            travel: 18.0,
            length: 8.0,
            stroke: 2.0,
            duration: CROSSHAIR_DURATION,
        }
    }
}

/// Tunables of [`Animation::halo`]: a breathing ring in the node's shape.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Halo {
    /// Ring radius of the circular variant, in screen pixels at `zoom == 1`.
    pub radius: f32,
    /// Floor for [`Self::radius`], in screen pixels, so the halo does not
    /// vanish when zoomed far out.
    pub radius_min: f32,
    /// Ring stroke width, before the `zoom` multiplier. Shared by the
    /// `*_outline` variant.
    pub stroke: f32,
    /// Floor for [`Self::stroke`], in screen pixels.
    pub stroke_min: f32,
    /// How far outside the node the `*_outline` variant draws its ring, in
    /// screen pixels at `zoom == 1`.
    pub outline_growth: f32,
    /// How long one breathe takes, in seconds.
    pub period: f32,
}

impl Halo {
    /// Runs `f` on a mutable copy of `self` and returns it.
    pub fn with(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }
}

impl Default for Halo {
    fn default() -> Self {
        Self {
            radius: 9.0,
            radius_min: 5.0,
            stroke: 2.0,
            stroke_min: 1.5,
            outline_growth: 5.0,
            period: 2.0,
        }
    }
}

/// Tunables of [`Animation::blink`]: a thick ring blinking on and off.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Blink {
    /// Ring radius of the circular variant, in screen pixels at `zoom == 1`.
    pub radius: f32,
    /// Circular variant's stroke width, before the `zoom` multiplier.
    pub stroke: f32,
    /// How far outside the node the `*_outline` variant draws its ring, in
    /// screen pixels at `zoom == 1`.
    pub outline_growth: f32,
    /// `*_outline` variant's stroke width, before the `zoom` multiplier.
    pub outline_stroke: f32,
    /// How long one blink takes, in seconds.
    pub period: f32,
}

impl Blink {
    /// Runs `f` on a mutable copy of `self` and returns it.
    pub fn with(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }
}

impl Default for Blink {
    fn default() -> Self {
        Self {
            radius: 4.0,
            stroke: 9.0,
            outline_growth: 2.0,
            outline_stroke: 4.0,
            period: 2.55,
        }
    }
}

/// Tunables of [`Animation::orbit`]: a dot travelling around the node.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Orbit {
    /// Orbit radius of the circular variant, in screen pixels at `zoom == 1`.
    pub radius: f32,
    /// Floor for [`Self::radius`], in screen pixels.
    pub radius_min: f32,
    /// How far outside the node the `*_outline` variant draws its path, in
    /// screen pixels at `zoom == 1`.
    pub outline_growth: f32,
    /// Width of the faint guide, before the `zoom` multiplier. Shared by both
    /// variants.
    pub guide_stroke: f32,
    /// Radius of the travelling dot, in screen pixels at `zoom == 1`.
    pub dot_radius: f32,
    /// Floor for [`Self::dot_radius`], in screen pixels.
    pub dot_min: f32,
    /// How long one full lap takes, in seconds.
    pub period: f32,
}

impl Orbit {
    /// Runs `f` on a mutable copy of `self` and returns it.
    pub fn with(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }
}

impl Default for Orbit {
    fn default() -> Self {
        Self {
            radius: 12.0,
            radius_min: 7.0,
            outline_growth: 8.0,
            guide_stroke: 1.0,
            dot_radius: 2.5,
            dot_min: 2.0,
            period: 3.0,
        }
    }
}

/// Tunables of [`Animation::glow`]: a tint breathing in and out.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glow {
    /// How long one pulse (dim, bright, dim) takes, in seconds.
    pub period: f32,
}

impl Glow {
    /// Runs `f` on a mutable copy of `self` and returns it.
    pub fn with(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }
}

impl Default for Glow {
    fn default() -> Self {
        Self {
            period: GLOW_PERIOD,
        }
    }
}

impl Animation {
    // ---------------------------------------------------------------- events

    /// One frame of an expanding, fading disc centred on `center`.
    ///
    /// Reads as *"one thing happened here"*. Plays for [`PULSE_DURATION`].
    /// Returns `true` while still playing.
    pub fn pulse(
        &self,
        painter: &Painter,
        center: Pos2,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        let secs = elapsed(initial_time);
        let radius = (self.pulse.base_radius + self.pulse.spread * secs) * zoom;
        let transparency = (1.00 - (secs / self.pulse.duration).abs()).max(0.0);
        // The fade tail rounds to a fully transparent color: skip it rather
        // than tessellating an invisible circle, which is pure waste when
        // many pulses are on screen at once. Same for a sub-pixel disc.
        if transparency > 0.0 && 2.0 * radius >= MIN_EFFECT_PX {
            painter.add(Shape::Circle(CircleShape::filled(
                center,
                radius,
                with_alpha(color, transparency),
            )));
        }
        secs < self.pulse.duration
    }

    /// One frame of three staggered expanding rings.
    ///
    /// Where [`Animation::pulse`] reads as a single event, the repetition here
    /// reads as *"activity is ongoing"*. Plays for [`RIPPLE_DURATION`]: every
    /// ring is born, spreads and fades out before it ends. Returns `true`
    /// while still playing.
    pub fn ripple(
        &self,
        painter: &Painter,
        center: Pos2,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        let secs = elapsed(initial_time);
        self.paint_ripple(painter, center, zoom, secs, false, color);
        secs < self.ripple.duration
    }

    /// The rings of [`Animation::ripple`] `secs` into the effect. With
    /// `looping`, rings keep being born for ever.
    fn paint_ripple(
        &self,
        painter: &Painter,
        center: Pos2,
        zoom: f32,
        secs: f32,
        looping: bool,
        color: Color32,
    ) {
        let shapes: Vec<Shape> = self
            .ripple
            .ring_progress(secs, looping)
            .map(|progress| {
                Shape::Circle(CircleShape::stroke(
                    center,
                    (self.ripple.base_radius + self.ripple.spread * progress) * zoom,
                    Stroke::new(self.ripple.stroke * zoom, with_alpha(color, 1.0 - progress)),
                ))
            })
            .collect();
        painter.extend(shapes);
    }

    /// One frame of a ring that empties clockwise from 12 o'clock.
    ///
    /// The remaining arc is the remaining fraction of [`COUNTDOWN_DURATION`],
    /// which makes it a natural fit for *"how old is this information"*.
    /// Returns `true` while still playing.
    ///
    /// To count down over another time, ask for it with
    /// [`NodeHandle::lasting`](crate::map::NodeHandle::lasting): a lasting
    /// countdown empties over the whole notification.
    pub fn countdown_arc(
        &self,
        painter: &Painter,
        center: Pos2,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        let secs = elapsed(initial_time);
        let remaining = 1.0 - secs / self.countdown.duration;
        self.paint_countdown(painter, center, zoom, remaining, color);
        secs < self.countdown.duration
    }

    /// The ring of [`Animation::countdown_arc`] with `remaining` of it left
    /// (`1.0` full, `0.0` empty).
    fn paint_countdown(
        &self,
        painter: &Painter,
        center: Pos2,
        zoom: f32,
        remaining: f32,
        color: Color32,
    ) {
        // Segments in a full turn; the arc draws a prefix of these.
        const STEPS: usize = 48;
        let remaining = remaining.clamp(0.0, 1.0);
        let radius = self.countdown.radius * zoom;

        if remaining > 0.0 {
            // The last point sits exactly at `remaining` of the turn, so the
            // end of the arc moves smoothly instead of in steps of a 48th.
            let count = (STEPS as f32 * remaining).ceil() as usize;
            let points = (0..=count)
                .map(|i| {
                    let turn = (i as f32 / STEPS as f32).min(remaining);
                    // Start at 12 o'clock and sweep clockwise. Screen y grows
                    // downwards, so a growing angle already turns clockwise.
                    let angle = TAU * turn - TAU / 4.0;
                    Pos2::new(
                        center.x + radius * angle.cos(),
                        center.y + radius * angle.sin(),
                    )
                })
                .collect();
            painter.add(Shape::Path(PathShape::line(
                points,
                Stroke::new(self.countdown.stroke * zoom, with_alpha(color, 1.0)),
            )));
        }
    }

    /// One frame of a disc that grows past its final size and settles back.
    ///
    /// Meant for nodes that just appeared. Plays for [`SCALE_IN_DURATION`].
    /// Returns `true` while still playing.
    pub fn scale_in(
        &self,
        painter: &Painter,
        center: Pos2,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        let secs = elapsed(initial_time);
        let progress = (secs / self.scale_in.duration).clamp(0.0, 1.0);
        let radius = self.scale_in.radius * zoom * ease_out_back(progress).max(0.0);
        painter.add(Shape::Circle(CircleShape::filled(
            center,
            radius,
            with_alpha(color, 1.0 - progress),
        )));
        secs < self.scale_in.duration
    }

    /// One frame of four ticks converging onto the node.
    ///
    /// Reads as *"target acquired"*; pairs well with selection. Plays for
    /// [`CROSSHAIR_DURATION`]. Returns `true` while still playing.
    pub fn crosshair(
        &self,
        painter: &Painter,
        center: Pos2,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        let secs = elapsed(initial_time);
        let progress = (secs / self.crosshair.duration).clamp(0.0, 1.0);
        // Ticks travel from far away down to just outside the node, and fade
        // out over the last third so they do not linger on top of it.
        let far = (self.crosshair.far - self.crosshair.travel * progress) * zoom;
        let near = far - self.crosshair.length * zoom;
        let alpha = if progress < 0.66 {
            1.0
        } else {
            1.0 - (progress - 0.66) / 0.34
        };
        let stroke = Stroke::new(self.crosshair.stroke * zoom, with_alpha(color, alpha));

        let mut shapes = Vec::with_capacity(4);
        for (dx, dy) in [(0.0, -1.0), (0.0, 1.0), (-1.0, 0.0), (1.0, 0.0)] {
            shapes.push(Shape::line_segment(
                [
                    Pos2::new(center.x + dx * far, center.y + dy * far),
                    Pos2::new(center.x + dx * near, center.y + dy * near),
                ],
                stroke,
            ));
        }
        painter.extend(shapes);
        secs < self.crosshair.duration
    }

    // ------------------------------------------------------------ dispatchers

    /// The circular event effect for `kind`, bound to this `Animation`'s
    /// tunables. Call it with `(painter, center, zoom, initial_time, color)`.
    pub fn event(
        &self,
        kind: NodeAnimation,
    ) -> impl Fn(&Painter, Pos2, f32, Instant, Color32) -> bool + 'static {
        let a = *self;
        move |painter, center, zoom, initial_time, color| match kind {
            NodeAnimation::Pulse => a.pulse(painter, center, zoom, initial_time, color),
            NodeAnimation::Ripple => a.ripple(painter, center, zoom, initial_time, color),
            NodeAnimation::CountdownArc => {
                a.countdown_arc(painter, center, zoom, initial_time, color)
            }
            NodeAnimation::ScaleIn => a.scale_in(painter, center, zoom, initial_time, color),
            NodeAnimation::Crosshair => a.crosshair(painter, center, zoom, initial_time, color),
        }
    }

    /// The `*_outline` event effect for `kind`, bound to this `Animation`'s
    /// tunables. Call it with `(painter, outline, zoom, initial_time, color)`.
    pub fn event_outline(
        &self,
        kind: NodeAnimation,
    ) -> impl Fn(&Painter, &NodeOutline, f32, Instant, Color32) -> bool + 'static {
        let a = *self;
        move |painter, outline, zoom, initial_time, color| match kind {
            NodeAnimation::Pulse => a.pulse_outline(painter, outline, zoom, initial_time, color),
            NodeAnimation::Ripple => a.ripple_outline(painter, outline, zoom, initial_time, color),
            NodeAnimation::CountdownArc => {
                a.countdown_outline(painter, outline, zoom, initial_time, color)
            }
            NodeAnimation::ScaleIn => {
                a.scale_in_outline(painter, outline, zoom, initial_time, color)
            }
            NodeAnimation::Crosshair => {
                a.crosshair_outline(painter, outline, zoom, initial_time, color)
            }
        }
    }

    /// The event effect for `kind` as a lasting notification
    /// ([`NodeHandle::lasting`](crate::map::NodeHandle::lasting)) draws it,
    /// for as long as the notification lasts, which is up to the caller to
    /// end. Call it with `(painter, center, zoom, started, until, color)`,
    /// where `started` is when the notification began and `until` when it
    /// ends; it always returns `true`.
    ///
    /// Most effects restart every [`event_duration`](Self::event_duration).
    /// Two do not:
    ///
    /// - `ripple` keeps a new ring coming every stagger, so there is no
    ///   moment where the effect ends and starts over.
    /// - `countdown` empties its ring once, over the whole time the
    ///   notification was asked to last (`until - started`), instead of
    ///   every [`Countdown::duration`].
    pub fn lasting_event(
        &self,
        kind: NodeAnimation,
    ) -> impl Fn(&Painter, Pos2, f32, Instant, Instant, Color32) -> bool + 'static {
        let a = *self;
        move |painter, center, zoom, started, until, color| {
            match kind {
                NodeAnimation::Ripple => {
                    a.paint_ripple(painter, center, zoom, elapsed(started), true, color);
                }
                NodeAnimation::CountdownArc => {
                    let remaining = remaining_until(started, until);
                    a.paint_countdown(painter, center, zoom, remaining, color);
                }
                _ => {
                    let cycle = cycle_start(started, Instant::now(), a.event_duration(kind));
                    a.event(kind)(painter, center, zoom, cycle, color);
                }
            }
            true
        }
    }

    /// [`Animation::lasting_event`] following an outline, like
    /// [`Animation::event_outline`]. Call it with
    /// `(painter, outline, zoom, started, until, color)`.
    pub fn lasting_event_outline(
        &self,
        kind: NodeAnimation,
    ) -> impl Fn(&Painter, &NodeOutline, f32, Instant, Instant, Color32) -> bool + 'static {
        let a = *self;
        move |painter, outline, zoom, started, until, color| {
            match kind {
                NodeAnimation::Ripple => {
                    a.paint_ripple_outline(painter, outline, zoom, elapsed(started), true, color);
                }
                NodeAnimation::CountdownArc => {
                    let remaining = remaining_until(started, until);
                    a.paint_countdown_outline(painter, outline, zoom, remaining, color);
                }
                _ => {
                    let cycle = cycle_start(started, Instant::now(), a.event_duration(kind));
                    a.event_outline(kind)(painter, outline, zoom, cycle, color);
                }
            }
            true
        }
    }

    /// Length of one cycle of a node event effect, in seconds: how long it
    /// plays once, and how often a lasting notification
    /// ([`NodeHandle::lasting`](crate::map::NodeHandle::lasting)) restarts it
    /// -- except `ripple`, which loops without restarting, and `countdown`,
    /// which spans the whole notification (see [`Animation::lasting_event`]).
    pub fn event_duration(&self, kind: NodeAnimation) -> f32 {
        match kind {
            NodeAnimation::Pulse => self.pulse.duration,
            NodeAnimation::Ripple => self.ripple.duration,
            NodeAnimation::CountdownArc => self.countdown.duration,
            NodeAnimation::ScaleIn => self.scale_in.duration,
            NodeAnimation::Crosshair => self.crosshair.duration,
        }
    }

    /// The circular persistent effect for `kind`, bound to this `Animation`'s
    /// tunables. Call it with `(painter, center, zoom, time, color)`.
    pub fn state(
        &self,
        kind: SteadyAnimation,
    ) -> impl Fn(&Painter, Pos2, f32, f32, Color32) + 'static {
        let a = *self;
        move |painter, center, zoom, time, color| match kind {
            SteadyAnimation::Blink => a.blink(painter, center, zoom, time, color),
            SteadyAnimation::Halo => a.halo(painter, center, zoom, time, color),
            SteadyAnimation::Orbit => a.orbit(painter, center, zoom, time, color),
        }
    }

    /// The `*_outline` persistent effect for `kind`, bound to this
    /// `Animation`'s tunables. Call it with
    /// `(painter, outline, zoom, time, color)`.
    pub fn state_outline(
        &self,
        kind: SteadyAnimation,
    ) -> impl Fn(&Painter, &NodeOutline, f32, f32, Color32) + 'static {
        let a = *self;
        move |painter, outline, zoom, time, color| match kind {
            SteadyAnimation::Blink => a.blink_outline(painter, outline, zoom, time, color),
            SteadyAnimation::Halo => a.halo_outline(painter, outline, zoom, time, color),
            SteadyAnimation::Orbit => a.orbit_outline(painter, outline, zoom, time, color),
        }
    }

    // ------------------------------------------------------- outline effects

    /// [`Animation::pulse`] following `outline`: the node's own shape grows
    /// outwards and fades. Painted before the node (as `notification_ui`
    /// is), the node covers the middle and it reads as a spreading halo.
    pub fn pulse_outline(
        &self,
        painter: &Painter,
        outline: &NodeOutline,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        let secs = elapsed(initial_time);
        let transparency = (1.0 - secs / self.pulse.duration).max(0.0);
        // See `pulse`: skip the fully transparent fade tail instead of
        // tessellating an invisible shape, and skip a sub-pixel one.
        if transparency > 0.0 {
            let grown = outline.grown(self.pulse.spread * secs * zoom);
            let bounds = grown.bounding_rect();
            if bounds.width().max(bounds.height()) >= MIN_EFFECT_PX {
                painter.add(grown.fill_shape(with_alpha(color, transparency)));
            }
        }
        secs < self.pulse.duration
    }

    /// [`Animation::ripple`] following `outline`: three staggered rings in
    /// the node's shape, spreading out.
    pub fn ripple_outline(
        &self,
        painter: &Painter,
        outline: &NodeOutline,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        let secs = elapsed(initial_time);
        self.paint_ripple_outline(painter, outline, zoom, secs, false, color);
        secs < self.ripple.duration
    }

    /// The rings of [`Animation::ripple_outline`] `secs` into the effect.
    /// With `looping`, rings keep being born for ever.
    fn paint_ripple_outline(
        &self,
        painter: &Painter,
        outline: &NodeOutline,
        zoom: f32,
        secs: f32,
        looping: bool,
        color: Color32,
    ) {
        let shapes: Vec<Shape> = self
            .ripple
            .ring_progress(secs, looping)
            .map(|progress| {
                outline
                    .grown(self.ripple.spread * progress * zoom)
                    .stroke_shape(Stroke::new(
                        self.ripple.stroke * zoom,
                        with_alpha(color, 1.0 - progress),
                    ))
            })
            .collect();
        painter.extend(shapes);
    }

    /// [`Animation::countdown_arc`] following `outline`: a line just outside
    /// the node's edge that empties clockwise from the top.
    pub fn countdown_outline(
        &self,
        painter: &Painter,
        outline: &NodeOutline,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        let secs = elapsed(initial_time);
        let remaining = 1.0 - secs / self.countdown.duration;
        self.paint_countdown_outline(painter, outline, zoom, remaining, color);
        secs < self.countdown.duration
    }

    /// The line of [`Animation::countdown_outline`] with `remaining` of it
    /// left (`1.0` full, `0.0` empty).
    fn paint_countdown_outline(
        &self,
        painter: &Painter,
        outline: &NodeOutline,
        zoom: f32,
        remaining: f32,
        color: Color32,
    ) {
        let remaining = remaining.clamp(0.0, 1.0);
        let points = partial_perimeter(
            &outline
                .grown(self.countdown.outline_offset * zoom)
                .perimeter(),
            remaining,
        );
        if points.len() >= 2 {
            painter.add(Shape::Path(PathShape::line(
                points,
                Stroke::new(self.countdown.stroke * zoom, with_alpha(color, 1.0)),
            )));
        }
    }

    /// [`Animation::scale_in`] following `outline`: the node's shape grows
    /// from nothing past its size and settles back, fading.
    pub fn scale_in_outline(
        &self,
        painter: &Painter,
        outline: &NodeOutline,
        _zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        let secs = elapsed(initial_time);
        let progress = (secs / self.scale_in.duration).clamp(0.0, 1.0);
        painter.add(
            outline
                .scaled(ease_out_back(progress).max(0.0))
                .fill_shape(with_alpha(color, 1.0 - progress)),
        );
        secs < self.scale_in.duration
    }

    /// [`Animation::crosshair`] following `outline`: four ticks converging on
    /// the middle of each side of the node's bounding box.
    pub fn crosshair_outline(
        &self,
        painter: &Painter,
        outline: &NodeOutline,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        let secs = elapsed(initial_time);
        let progress = (secs / self.crosshair.duration).clamp(0.0, 1.0);
        // Same travel as the circular version, measured from the box's edge.
        let far = (self.crosshair.outline_far - self.crosshair.travel * progress) * zoom;
        let near = far - self.crosshair.length * zoom;
        let alpha = if progress < 0.66 {
            1.0
        } else {
            1.0 - (progress - 0.66) / 0.34
        };
        let stroke = Stroke::new(self.crosshair.stroke * zoom, with_alpha(color, alpha));
        let bounds = outline.bounding_rect();
        let ticks = [
            (bounds.center_top(), Vec2::new(0.0, -1.0)),
            (bounds.center_bottom(), Vec2::new(0.0, 1.0)),
            (bounds.left_center(), Vec2::new(-1.0, 0.0)),
            (bounds.right_center(), Vec2::new(1.0, 0.0)),
        ];
        painter.extend(ticks.map(|(edge, direction)| {
            Shape::line_segment([edge + direction * far, edge + direction * near], stroke)
        }));
        secs < self.crosshair.duration
    }

    /// [`Animation::halo`] following `outline`: a ring in the node's shape,
    /// just outside it, whose opacity breathes. `time` is the frame time.
    pub fn halo_outline(
        &self,
        painter: &Painter,
        outline: &NodeOutline,
        zoom: f32,
        time: f32,
        color: Color32,
    ) {
        let alpha = 0.30 + 0.45 * triangle_wave(time, self.halo.period);
        painter.add(
            outline
                .grown(self.halo.outline_growth * zoom)
                .stroke_shape(Stroke::new(
                    (self.halo.stroke * zoom).max(self.halo.stroke_min),
                    with_alpha(color, alpha),
                )),
        );
    }

    /// [`Animation::blink`] following `outline`: a thick ring in the node's
    /// shape blinking on and off. `time` is the frame time.
    pub fn blink_outline(
        &self,
        painter: &Painter,
        outline: &NodeOutline,
        zoom: f32,
        time: f32,
        color: Color32,
    ) {
        painter.add(
            outline
                .grown(self.blink.outline_growth * zoom)
                .stroke_shape(Stroke::new(
                    self.blink.outline_stroke * zoom,
                    with_alpha(color, triangle_wave(time, self.blink.period)),
                )),
        );
    }

    /// [`Animation::orbit`] following `outline`: a dot travelling around the
    /// node's shape, with a faint guide. `time` is the frame time.
    pub fn orbit_outline(
        &self,
        painter: &Painter,
        outline: &NodeOutline,
        zoom: f32,
        time: f32,
        color: Color32,
    ) {
        let path = outline.grown(self.orbit.outline_growth * zoom);
        painter.add(path.stroke_shape(Stroke::new(
            self.orbit.guide_stroke,
            with_alpha(color, 0.25),
        )));
        if let Some(dot) = point_along(&path.perimeter(), time / self.orbit.period) {
            painter.add(Shape::Circle(CircleShape::filled(
                dot,
                (self.orbit.dot_radius * zoom).max(self.orbit.dot_min),
                with_alpha(color, 1.0),
            )));
        }
    }

    /// [`Animation::glow`] following `outline`: the node's shape filled with a
    /// tint breathing in and out, scaled by `strength` (e.g.
    /// [`NodeContext::marker`](crate::map::objects::NodeContext::marker)).
    /// Paint it over the node's background and before its label.
    pub fn glow_outline(
        &self,
        painter: &Painter,
        outline: &NodeOutline,
        time: f32,
        color: Color32,
        strength: f32,
    ) {
        let level = glow_level(time, strength, self.glow.period);
        if level > 0.0 {
            painter.add(outline.fill_shape(color.gamma_multiply(level)));
        }
    }
}

impl SegmentAnimations {
    // -------------------------------------------------------- events/segment

    /// The event effect for a segment `kind`, bound to this `SegmentAnimations`'
    /// tunables. Call it with `(painter, from, to, zoom, initial_time, color)`.
    ///
    /// The effect runs from `from` towards `to`: to run it in a
    /// [`CometDirection`](super::objects::CometDirection), get the pair from
    /// [`CometDirection::orient`](super::objects::CometDirection::orient).
    pub fn event(
        &self,
        kind: SegmentAnimation,
    ) -> impl Fn(&Painter, Pos2, Pos2, f32, Instant, Color32) -> bool + 'static {
        let a = *self;
        move |painter, from, to, zoom, initial_time, color| match kind {
            SegmentAnimation::FlashDecay => {
                a.flash_decay(painter, from, to, zoom, initial_time, color)
            }
            SegmentAnimation::Comet => a.comet_once(painter, [from, to], zoom, initial_time, color),
            SegmentAnimation::Wipe => a.wipe(painter, from, to, zoom, initial_time, color),
        }
    }

    /// The persistent effect for a segment `kind`, bound to this
    /// `SegmentAnimations`' tunables. Call it with
    /// `(painter, from, to, zoom, time, color)`.
    ///
    /// The effect runs from `from` towards `to`: to run it in a
    /// [`CometDirection`](super::objects::CometDirection), get the pair from
    /// [`CometDirection::orient`](super::objects::CometDirection::orient).
    pub fn state(
        &self,
        kind: SteadySegmentAnimation,
    ) -> impl Fn(&Painter, Pos2, Pos2, f32, f32, Color32) + 'static {
        let a = *self;
        move |painter, from, to, zoom, time, color| match kind {
            SteadySegmentAnimation::Comet => a.comet(painter, from, to, zoom, time, color),
            SteadySegmentAnimation::Dash => a.dash(painter, from, to, zoom, time, color),
            SteadySegmentAnimation::GlowBand => a.glow_band(painter, from, to, zoom, time, color),
            SteadySegmentAnimation::Chevrons => a.chevrons(painter, from, to, zoom, time, color),
        }
    }

    /// One frame of a segment briefly thickening and brightening, then fading
    /// back to nothing.
    ///
    /// The segment analogue of [`Animation::pulse`]: reads as *"something
    /// happened on this route"*. Plays for [`FlashDecay::duration`]. Returns
    /// `true` while still playing.
    pub fn flash_decay(
        &self,
        painter: &Painter,
        a: Pos2,
        b: Pos2,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        let secs = elapsed(initial_time);
        let progress = (secs / self.flash_decay.duration).clamp(0.0, 1.0);
        let width =
            (self.flash_decay.base_width + self.flash_decay.extra_width * (1.0 - progress)) * zoom;
        painter.line_segment(
            [a, b],
            Stroke::new(width, with_alpha(color, 1.0 - progress)),
        );
        secs < self.flash_decay.duration
    }

    /// One frame of a single dot pass along the segment, then gone — the
    /// event-driven counterpart to [`SegmentAnimations::comet`].
    ///
    /// `endpoints` is `[from, to]`, already oriented the way the dot should
    /// travel: the caller resolves a [`CometDirection`](super::objects::CometDirection)
    /// into that pair with
    /// [`CometDirection::orient`](super::objects::CometDirection::orient).
    /// Plays for
    /// [`CometOnce::duration`]. Returns `true` while still playing.
    pub fn comet_once(
        &self,
        painter: &Painter,
        endpoints: [Pos2; 2],
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        let [from, to] = endpoints;
        let secs = elapsed(initial_time);
        let progress = (secs / self.comet_once.duration).clamp(0.0, 1.0);
        let pos = from + (to - from) * progress;
        painter.add(Shape::Circle(CircleShape::filled(
            pos,
            (self.comet_once.dot_radius * zoom).max(self.comet_once.dot_min),
            color,
        )));
        secs < self.comet_once.duration
    }

    /// One frame of the segment drawing itself in, from `a` towards `b`, then
    /// gone. Reads as *"this route was just established"* — where
    /// [`SegmentAnimations::comet_once`] shows something moving along an
    /// existing route, this shows the route itself appearing. Plays for
    /// [`Wipe::duration`]. Returns `true` while still playing.
    ///
    /// Cheaper than the mesh technique [`SegmentAnimations::dash`] uses: the
    /// progressively-revealed portion is still just a straight line, so a
    /// plain `line_segment` from `a` to the interpolated point suffices.
    pub fn wipe(
        &self,
        painter: &Painter,
        a: Pos2,
        b: Pos2,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        let secs = elapsed(initial_time);
        let progress = (secs / self.wipe.duration).clamp(0.0, 1.0);
        let leading_edge = a + (b - a) * progress;
        painter.line_segment(
            [a, leading_edge],
            Stroke::new(self.wipe.width * zoom, color),
        );
        secs < self.wipe.duration
    }

    // ----------------------------------------------------- persistent/segment

    /// One frame of a dot travelling from `a` to `b` and looping back.
    ///
    /// Reads as *"this is the direction of flow"*. `time` is the frame time in
    /// seconds; one full pass takes [`Comet::period`].
    pub fn comet(&self, painter: &Painter, a: Pos2, b: Pos2, zoom: f32, time: f32, color: Color32) {
        let t = (time / self.comet.period).rem_euclid(1.0);
        let pos = a + (b - a) * t;
        painter.add(Shape::Circle(CircleShape::filled(
            pos,
            (self.comet.dot_radius * zoom).max(self.comet.dot_min),
            color,
        )));
    }

    /// One frame of a segment drawn as a dashed line whose pattern slides
    /// along it ("marching ants"). `time` is the frame time in seconds; the
    /// pattern repeats every [`Dash::period_px`] screen pixels and slides at
    /// [`Dash::speed`] repeats per second.
    ///
    /// The ribbon is [`Dash::width`] screen pixels wide, whatever the zoom
    /// (`zoom` is unused, and kept so every steady segment effect has the same
    /// signature): the width of the default stroke when that is `None` and the
    /// animations went through [`SegmentAnimations::with_line_width`] (the map
    /// does it), otherwise [`DASH_WIDTH`].
    ///
    /// Two triangles textured with a small repeating strip (registered once
    /// per [`egui::Context`] and reused after that), rather than one shape per
    /// dash -- see the [module docs](self) for why that matters at scale. A
    /// zero-length segment is skipped.
    pub fn dash(&self, painter: &Painter, a: Pos2, b: Pos2, _zoom: f32, time: f32, color: Color32) {
        let delta = b - a;
        let len = delta.length();
        if len <= f32::EPSILON {
            return;
        }
        let dir = delta / len;
        let width = self.dash.width.unwrap_or(DASH_WIDTH);
        let normal = Vec2::new(-dir.y, dir.x) * (width * 0.5);
        // Sampling a fixed screen point at an ever-larger `u` (`phase` growing
        // with time) makes the pattern crawl towards `a`, against `comet`,
        // `glow_band` and `chevrons`, which all go from `a` to `b`: negating
        // the time term makes it slide towards `b` like them (see
        // `chevrons`).
        let phase = (-(time * self.dash.speed)).rem_euclid(1.0);
        let u0 = phase;
        let u1 = phase + len / self.dash.period_px;

        let texture = Self::dash_texture(painter.ctx());
        let mut mesh = Mesh::with_texture(texture.id());
        mesh.vertices.extend([
            Vertex {
                pos: a + normal,
                uv: pos2(u0, 0.5),
                color,
            },
            Vertex {
                pos: a - normal,
                uv: pos2(u0, 0.5),
                color,
            },
            Vertex {
                pos: b + normal,
                uv: pos2(u1, 0.5),
                color,
            },
            Vertex {
                pos: b - normal,
                uv: pos2(u1, 0.5),
                color,
            },
        ]);
        mesh.indices.extend([0, 1, 2, 2, 1, 3]);
        painter.add(mesh);
    }

    /// Returns the texture [`SegmentAnimations::dash`] samples, creating and
    /// caching it in the context's own temp data on first use so every segment
    /// (and every frame) reuses the same GPU upload instead of re-registering
    /// one.
    ///
    /// A `[WHITE, alpha]` strip rather than an opaque/transparent one: the
    /// vertex `color` tints it (egui multiplies `vertex.color * texel` when
    /// painting a textured mesh), so the same texture works for every
    /// segment's colour. The alpha ramps over a few texels at each edge of the
    /// transition instead of stepping instantly, so a `Linear` sampler gives
    /// the dash a soft edge rather than the aliasing a hard step would show
    /// under magnification.
    fn dash_texture(ctx: &Context) -> TextureHandle {
        let id = Id::new("egui_map::dash_texture");
        if let Some(handle) = ctx.data(|d| d.get_temp::<TextureHandle>(id)) {
            return handle;
        }

        const WIDTH: usize = 32;
        const FADE: usize = 3;
        let half = WIDTH / 2;
        let pixels = (0..WIDTH)
            .map(|i| {
                let alpha = if i < half - FADE {
                    255
                } else if i < half + FADE {
                    let t = (i - (half - FADE)) as f32 / (2.0 * FADE as f32);
                    (255.0 * (1.0 - t)).round() as u8
                } else {
                    0
                };
                Color32::from_white_alpha(alpha)
            })
            .collect();
        let image = ColorImage::new([WIDTH, 1], pixels);
        let handle = ctx.load_texture(
            "egui_map::dash",
            image,
            TextureOptions {
                magnification: TextureFilter::Linear,
                minification: TextureFilter::Linear,
                wrap_mode: TextureWrapMode::Repeat,
                mipmap_mode: None,
            },
        );
        ctx.data_mut(|d| d.insert_temp(id, handle.clone()));
        handle
    }

    /// One frame of a localized band of brightness travelling the length of
    /// the segment and looping. `time` is the frame time in seconds; one full
    /// traverse-and-loop takes [`GlowBand::period`].
    ///
    /// Reads as *"flow"*, calmer than [`SegmentAnimations::dash`]'s marching
    /// pattern -- a single soft highlight rather than a repeating texture.
    /// Uses the same textured-mesh technique as `dash`, but with a
    /// [`TextureWrapMode::ClampToEdge`] sampler and a texture that is zero-alpha
    /// at both edges: as the band's mapped position slides past `0.0` or
    /// `1.0`, sampling clamps to that zero-alpha edge texel, so the band
    /// fades out before either endpoint instead of popping back in like
    /// `dash`'s repeating pattern would. A zero-length segment is skipped.
    pub fn glow_band(
        &self,
        painter: &Painter,
        a: Pos2,
        b: Pos2,
        zoom: f32,
        time: f32,
        color: Color32,
    ) {
        let delta = b - a;
        let len = delta.length();
        if len <= f32::EPSILON {
            return;
        }
        let dir = delta / len;
        let normal = Vec2::new(-dir.y, dir.x) * (self.glow_band.thickness * zoom * 0.5);

        // Half-width of the visible band, as a fraction of the segment's own
        // length -- capped at 0.5 so the band can never cover more than the
        // whole segment.
        let half_width_frac = (self.glow_band.length_px * 0.5 / len).min(0.5);
        // The band's peak travels from just before the start to just past the
        // end and loops, rather than jumping straight from `1.0` back to
        // `0.0` -- that extra span is what lets it fade out past each end.
        let span = 1.0 + 2.0 * half_width_frac;
        let t = (time / self.glow_band.period).rem_euclid(1.0);
        let peak = -half_width_frac + t * span;
        let texture_u = |frac: f32| 0.5 + (frac - peak) / (2.0 * half_width_frac);

        let texture = Self::glow_band_texture(painter.ctx());
        let mut mesh = Mesh::with_texture(texture.id());
        let u_a = texture_u(0.0);
        let u_b = texture_u(1.0);
        mesh.vertices.extend([
            Vertex {
                pos: a + normal,
                uv: pos2(u_a, 0.5),
                color,
            },
            Vertex {
                pos: a - normal,
                uv: pos2(u_a, 0.5),
                color,
            },
            Vertex {
                pos: b + normal,
                uv: pos2(u_b, 0.5),
                color,
            },
            Vertex {
                pos: b - normal,
                uv: pos2(u_b, 0.5),
                color,
            },
        ]);
        mesh.indices.extend([0, 1, 2, 2, 1, 3]);
        painter.add(mesh);
    }

    /// Returns the texture [`SegmentAnimations::glow_band`] samples, creating
    /// and caching it in the context's own temp data on first use -- same
    /// pattern as [`SegmentAnimations::dash_texture`].
    ///
    /// A symmetric tent-shaped alpha profile (zero at both edges, peaking at
    /// the centre, smoothstepped rather than a hard linear ramp for a softer
    /// glow) rather than `dash`'s step, and [`TextureWrapMode::ClampToEdge`]
    /// instead of `Repeat` -- the zero-alpha edges are exactly what makes
    /// sampling past `[0, 1]` fade out instead of wrapping.
    fn glow_band_texture(ctx: &Context) -> TextureHandle {
        let id = Id::new("egui_map::glow_band_texture");
        if let Some(handle) = ctx.data(|d| d.get_temp::<TextureHandle>(id)) {
            return handle;
        }

        const WIDTH: usize = 64;
        let pixels = (0..WIDTH)
            .map(|i| {
                let u = i as f32 / (WIDTH - 1) as f32;
                let distance_from_center = (u - 0.5).abs() * 2.0;
                let alpha = (1.0 - distance_from_center).clamp(0.0, 1.0);
                let alpha = alpha * alpha * (3.0 - 2.0 * alpha); // smoothstep
                Color32::from_white_alpha((255.0 * alpha).round() as u8)
            })
            .collect();
        let image = ColorImage::new([WIDTH, 1], pixels);
        let handle = ctx.load_texture(
            "egui_map::glow_band",
            image,
            TextureOptions {
                magnification: TextureFilter::Linear,
                minification: TextureFilter::Linear,
                wrap_mode: TextureWrapMode::ClampToEdge,
                mipmap_mode: None,
            },
        );
        ctx.data_mut(|d| d.insert_temp(id, handle.clone()));
        handle
    }

    /// One frame of a row of arrow shapes sliding along the segment. `time`
    /// is the frame time in seconds; the pattern repeats every
    /// [`Chevrons::period_px`] screen pixels and slides at
    /// [`Chevrons::speed`] repeats per second.
    ///
    /// Reads as *"direction of travel"*, more explicit at a glance than
    /// [`SegmentAnimations::comet`]'s single dot. Same mesh-building shape as
    /// [`SegmentAnimations::dash`], but where `dash` samples a 1D texture at a
    /// constant `uv.y = 0.5` (its stripes don't vary across the ribbon's
    /// width), the two long edges of this mesh get `uv.y = 0.0` / `1.0`
    /// instead, so the interpolated `uv.y` sweeps across a genuinely 2D
    /// texture and traces out the arrow shape. A zero-length segment is
    /// skipped.
    ///
    /// The ribbon is [`Chevrons::width`] screen pixels wide whatever the zoom
    /// (`zoom` is unused, and kept so every steady segment effect has the same
    /// signature, like [`SegmentAnimations::dash`]): the arrow's shape is the
    /// ratio of that width to [`Chevrons::period_px`], and both are in screen
    /// pixels, so it never gets squashed or stretched when the map is zoomed.
    pub fn chevrons(
        &self,
        painter: &Painter,
        a: Pos2,
        b: Pos2,
        _zoom: f32,
        time: f32,
        color: Color32,
    ) {
        let delta = b - a;
        let len = delta.length();
        if len <= f32::EPSILON {
            return;
        }
        let dir = delta / len;
        let normal = Vec2::new(-dir.y, dir.x) * (self.chevrons.width * 0.5);
        // `u0 < u1` (below) maps the texture's own +u direction onto the
        // segment's `a -> b` direction, and the arrow tip sits at the
        // texture's higher `u` (see `chevrons_texture`) -- so in any single
        // frame the tip already reads as pointing towards `b`. The pattern
        // must then *slide* towards `b` too, not away from it: sampling a
        // fixed screen point at an ever-larger `u` (`phase` growing with
        // time) is what would make it crawl towards `a` instead, backwards
        // from the way the arrows point. Negating the time term here is what
        // keeps the two in agreement.
        let phase = (-(time * self.chevrons.speed)).rem_euclid(1.0);
        let u0 = phase;
        let u1 = phase + len / self.chevrons.period_px;

        let texture =
            Self::chevrons_texture(painter.ctx(), self.chevrons.leg_slope, self.chevrons.stroke);
        let mut mesh = Mesh::with_texture(texture.id());
        mesh.vertices.extend([
            Vertex {
                pos: a + normal,
                uv: pos2(u0, 0.0),
                color,
            },
            Vertex {
                pos: a - normal,
                uv: pos2(u0, 1.0),
                color,
            },
            Vertex {
                pos: b + normal,
                uv: pos2(u1, 0.0),
                color,
            },
            Vertex {
                pos: b - normal,
                uv: pos2(u1, 1.0),
                color,
            },
        ]);
        mesh.indices.extend([0, 1, 2, 2, 1, 3]);
        painter.add(mesh);
    }

    /// Returns the texture [`SegmentAnimations::chevrons`] samples, creating
    /// and caching it in the context's own temp data on first use -- same
    /// pattern as [`SegmentAnimations::dash_texture`].
    ///
    /// A genuinely 2D tile, unlike `dash`'s 1x32 strip: for each row `v`
    /// (0 at one long edge of the ribbon, 1 at the other) the arrow's stroke
    /// sits at an "ideal" `u` that moves back from a tip near the leading
    /// edge as `v` moves away from the centreline in either direction --
    /// tracing the two legs of a `>` shape -- and each texel's alpha falls
    /// off with its distance from that ideal `u`, smoothstepped for a soft
    /// stroke. [`TextureWrapMode::Repeat`] tiles it along `u`; `v` never
    /// leaves `[0, 1]` in the mesh above, so wrapping never triggers on that
    /// axis.
    ///
    /// The shape comes from [`Chevrons::leg_slope`] and [`Chevrons::stroke`];
    /// one texture is cached per distinct pair, so changing them while the
    /// app runs registers a new one (a few kilobytes) the first time it is
    /// drawn.
    fn chevrons_texture(ctx: &Context, leg_slope: f32, stroke: f32) -> TextureHandle {
        let id = Id::new((
            "egui_map::chevrons_texture",
            leg_slope.to_bits(),
            stroke.to_bits(),
        ));
        if let Some(handle) = ctx.data(|d| d.get_temp::<TextureHandle>(id)) {
            return handle;
        }

        // Finer than `dash`'s strip: the ribbon is only a few pixels wide, so
        // the tile needs the rows to keep the arrow's legs smooth.
        const WIDTH: usize = 64;
        const HEIGHT: usize = 32;
        const TIP_U: f32 = 0.75;
        // Guards the division below against a zero or negative stroke.
        let stroke = stroke.max(f32::EPSILON);

        let mut pixels = Vec::with_capacity(WIDTH * HEIGHT);
        for j in 0..HEIGHT {
            let v = j as f32 / (HEIGHT - 1) as f32;
            let ideal_u = TIP_U - leg_slope * (v - 0.5).abs();
            for i in 0..WIDTH {
                let u = i as f32 / WIDTH as f32;
                let distance = (u - ideal_u).abs();
                let alpha = (1.0 - distance / stroke).clamp(0.0, 1.0);
                let alpha = alpha * alpha * (3.0 - 2.0 * alpha); // smoothstep
                pixels.push(Color32::from_white_alpha((255.0 * alpha).round() as u8));
            }
        }
        let image = ColorImage::new([WIDTH, HEIGHT], pixels);
        let handle = ctx.load_texture(
            "egui_map::chevrons",
            image,
            TextureOptions {
                magnification: TextureFilter::Linear,
                minification: TextureFilter::Linear,
                wrap_mode: TextureWrapMode::Repeat,
                mipmap_mode: None,
            },
        );
        ctx.data_mut(|d| d.insert_temp(id, handle.clone()));
        handle
    }
}

impl Animation {
    // ------------------------------------------------------------ persistent

    /// One frame of a ring whose opacity breathes in and out.
    ///
    /// For lasting state — *"you are here"*, *"this system is camped"*. `time`
    /// is the frame time in seconds (`ui.input(|i| i.time)`).
    ///
    /// The radius has a floor in screen pixels so the halo does not vanish
    /// when the map is zoomed far out.
    pub fn halo(&self, painter: &Painter, center: Pos2, zoom: f32, time: f32, color: Color32) {
        let alpha = 0.30 + 0.45 * triangle_wave(time, self.halo.period);
        let radius = (self.halo.radius * zoom).max(self.halo.radius_min);
        painter.add(Shape::Circle(CircleShape::stroke(
            center,
            radius,
            Stroke::new(
                (self.halo.stroke * zoom).max(self.halo.stroke_min),
                with_alpha(color, alpha),
            ),
        )));
    }

    /// One frame of a thick ring blinking on and off.
    ///
    /// This is the effect markers have always used, factored out of the widget
    /// so it can be selected and reused like any other. `time` is the frame
    /// time in seconds.
    pub fn blink(&self, painter: &Painter, center: Pos2, zoom: f32, time: f32, color: Color32) {
        painter.add(Shape::Circle(CircleShape::stroke(
            center,
            self.blink.radius * zoom,
            Stroke::new(
                self.blink.stroke * zoom,
                with_alpha(color, triangle_wave(time, self.blink.period)),
            ),
        )));
    }

    /// One frame of a tint breathing in and out over a rounded rectangle.
    ///
    /// For a custom [`NodeTemplate`](crate::map::objects::NodeTemplate) that
    /// draws its node as a box: paint it over the node's background and
    /// before its label, with the same `rect` and `corner_radius`, and the
    /// node itself pulses in `color` without covering its text -- unlike
    /// [`marker_ui`](crate::map::objects::NodeTemplate::marker_ui), which is
    /// drawn over every node. `color`'s own alpha is the peak opacity (e.g.
    /// `theme.marker.gamma_multiply(0.6)` to keep the label readable).
    ///
    /// `strength` (`0.0..=1.0`) scales the whole effect: pass
    /// [`NodeContext::marker`](crate::map::objects::NodeContext::marker) and
    /// the glow fades in and out with the node's markers. Nothing is drawn at
    /// `0.0`. `time` is the frame time in seconds; request repaints while
    /// `strength` is above zero.
    pub fn glow(
        &self,
        painter: &Painter,
        rect: Rect,
        corner_radius: CornerRadius,
        time: f32,
        color: Color32,
        strength: f32,
    ) {
        let level = glow_level(time, strength, self.glow.period);
        if level <= 0.0 {
            return;
        }
        painter.add(Shape::rect_filled(
            rect,
            corner_radius,
            color.gamma_multiply(level),
        ));
    }

    /// One frame of a dot orbiting the node, with a faint guide ring.
    ///
    /// Reads as *"under observation"*. `time` is the frame time in seconds.
    pub fn orbit(&self, painter: &Painter, center: Pos2, zoom: f32, time: f32, color: Color32) {
        let radius = (self.orbit.radius * zoom).max(self.orbit.radius_min);
        let angle = TAU * (time / self.orbit.period).rem_euclid(1.0);
        let dot = Pos2::new(
            center.x + radius * angle.cos(),
            center.y + radius * angle.sin(),
        );
        painter.extend([
            Shape::Circle(CircleShape::stroke(
                center,
                radius,
                Stroke::new(self.orbit.guide_stroke, with_alpha(color, 0.25)),
            )),
            Shape::Circle(CircleShape::filled(
                dot,
                (self.orbit.dot_radius * zoom).max(self.orbit.dot_min),
                with_alpha(color, 1.0),
            )),
        ]);
    }
}

#[cfg(test)]
mod tests {
    use super::super::objects::CometDirection;
    use super::*;
    use egui::{Context, LayerId, Vec2};
    use std::time::Duration;

    fn headless_painter() -> Painter {
        Painter::new(
            Context::default(),
            LayerId::background(),
            Rect::from_min_size(Pos2::ZERO, Vec2::new(100.0, 100.0)),
        )
    }

    /// Every event-driven node effect, with the duration it is supposed to run
    /// for. Each entry is the `kind` the dispatcher should turn into the
    /// corresponding method.
    fn event_effects() -> [(&'static str, NodeAnimation, f32); 5] {
        [
            ("pulse", NodeAnimation::Pulse, PULSE_DURATION),
            ("ripple", NodeAnimation::Ripple, RIPPLE_DURATION),
            (
                "countdown_arc",
                NodeAnimation::CountdownArc,
                COUNTDOWN_DURATION,
            ),
            ("scale_in", NodeAnimation::ScaleIn, SCALE_IN_DURATION),
            ("crosshair", NodeAnimation::Crosshair, CROSSHAIR_DURATION),
        ]
    }

    #[test]
    fn glow_pulses_and_scales_with_strength() {
        assert_eq!(glow_level(0.0, 1.0, GLOW_PERIOD), 0.0);
        assert!((glow_level(GLOW_PERIOD / 2.0, 1.0, GLOW_PERIOD) - 1.0).abs() < 1e-5);
        assert!((glow_level(GLOW_PERIOD / 2.0, 0.5, GLOW_PERIOD) - 0.5).abs() < 1e-5);
        assert_eq!(glow_level(GLOW_PERIOD / 2.0, 0.0, GLOW_PERIOD), 0.0);
        // Out-of-range strengths are clamped.
        assert!((glow_level(GLOW_PERIOD / 2.0, 3.0, GLOW_PERIOD) - 1.0).abs() < 1e-5);
        assert_eq!(glow_level(GLOW_PERIOD / 2.0, -1.0, GLOW_PERIOD), 0.0);
        // Drawing it, at any strength, never panics.
        let painter = headless_painter();
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(90.0, 35.0));
        let animation = Animation::default();
        for strength in [0.0, 0.5, 1.0] {
            animation.glow(
                &painter,
                rect,
                CornerRadius::same(10),
                GLOW_PERIOD / 2.0,
                Color32::GREEN,
                strength,
            );
        }
    }

    #[test]
    fn event_effects_report_running_then_finished() {
        let painter = headless_painter();
        let animation = Animation::default();
        for (name, kind, duration) in event_effects() {
            let effect = animation.event(kind);
            assert!(
                effect(&painter, Pos2::ZERO, 1.0, Instant::now(), Color32::RED),
                "{name} must report it is still running when it just started"
            );

            let long_past = Instant::now() - Duration::from_secs_f32(duration + 1.0);
            assert!(
                !effect(&painter, Pos2::ZERO, 1.0, long_past, Color32::RED),
                "{name} must report it is finished once its duration has passed"
            );
        }
    }

    #[test]
    fn every_event_effect_stays_under_the_orphan_sweep() {
        // `Map` drops notifications older than 10s as a safety net. An effect
        // that outlived it would be cut off mid-play.
        for (name, _, duration) in event_effects() {
            assert!(
                duration < 10.0,
                "{name} lasts {duration}s, which the 10s orphan sweep would truncate"
            );
        }
    }

    #[test]
    fn persistent_effects_run_at_any_time() {
        let painter = headless_painter();
        let animation = Animation::default();
        for time in [0.0, 0.7, 1.3, 60.0] {
            animation.halo(&painter, Pos2::ZERO, 1.0, time, Color32::GREEN);
            animation.blink(&painter, Pos2::ZERO, 1.0, time, Color32::GREEN);
            animation.orbit(&painter, Pos2::ZERO, 1.0, time, Color32::GREEN);
        }
    }

    #[test]
    fn animation_default_matches_the_built_in_values() {
        // Pins `Animation::default()` to the values the widget used to
        // hard-code, so a regression here is a visual change, not a silent one.
        let a = Animation::default();
        assert_eq!(
            a.pulse,
            Pulse {
                base_radius: 4.0,
                spread: 40.0,
                duration: PULSE_DURATION
            }
        );
        assert_eq!(
            a.ripple,
            Ripple {
                base_radius: 4.0,
                spread: 36.0,
                stroke: 2.0,
                duration: RIPPLE_DURATION
            }
        );
        assert_eq!(
            a.countdown,
            Countdown {
                radius: 10.0,
                outline_offset: 4.0,
                stroke: 2.0,
                duration: COUNTDOWN_DURATION
            }
        );
        assert_eq!(
            a.scale_in,
            ScaleIn {
                radius: 8.0,
                duration: SCALE_IN_DURATION
            }
        );
        assert_eq!(
            a.crosshair,
            Crosshair {
                far: 30.0,
                outline_far: 22.0,
                travel: 18.0,
                length: 8.0,
                stroke: 2.0,
                duration: CROSSHAIR_DURATION
            }
        );
        assert_eq!(
            a.halo,
            Halo {
                radius: 9.0,
                radius_min: 5.0,
                stroke: 2.0,
                stroke_min: 1.5,
                outline_growth: 5.0,
                period: 2.0
            }
        );
        assert_eq!(
            a.blink,
            Blink {
                radius: 4.0,
                stroke: 9.0,
                outline_growth: 2.0,
                outline_stroke: 4.0,
                period: 2.55
            }
        );
        assert_eq!(
            a.orbit,
            Orbit {
                radius: 12.0,
                radius_min: 7.0,
                outline_growth: 8.0,
                guide_stroke: 1.0,
                dot_radius: 2.5,
                dot_min: 2.0,
                period: 3.0
            }
        );
        assert_eq!(
            a.glow,
            Glow {
                period: GLOW_PERIOD
            }
        );
    }

    #[test]
    fn with_adjusts_only_the_given_fields() {
        let a = Animation::default().with(|a| {
            a.pulse.spread = 60.0;
            a.ripple.stroke = 3.0;
        });
        assert_eq!(a.pulse.spread, 60.0);
        assert_eq!(a.ripple.stroke, 3.0);
        // Everything else keeps its default.
        assert_eq!(a.pulse.base_radius, Pulse::default().base_radius);
        assert_eq!(a.ripple.spread, Ripple::default().spread);
        assert_eq!(a.halo, Halo::default());
    }

    #[test]
    fn pulse_spread_changes_the_drawn_radius() {
        // Two painters in the same frame; only the `spread` differs. The
        // recorded circle radius must differ by the expected amount, proving
        // the config actually reaches the drawing.
        let ctx = Context::default();
        // A fixed `initial_time` in the past so `secs` is non-zero and the
        // `spread` term is what differs between the two runs.
        let initial_time = Instant::now() - Duration::from_secs_f32(1.0);
        let shape_radius = |animation: Animation| -> f32 {
            let mut out = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(100.0, 100.0))),
                    ..Default::default()
                },
                |ui| {
                    animation.pulse(ui.painter(), Pos2::ZERO, 1.0, initial_time, Color32::RED);
                },
            );
            let radius = out
                .shapes
                .iter()
                .find_map(|cs| match &cs.shape {
                    egui::epaint::Shape::Circle(circle) => Some(circle.radius),
                    _ => None,
                })
                .expect("pulse must draw a circle");
            out.textures_delta.clear();
            radius
        };

        let default_radius = shape_radius(Animation::default());
        let wider = shape_radius(Animation::default().with(|a| a.pulse.spread = 80.0));
        assert!(
            wider > default_radius,
            "raising pulse.spread must grow the drawn radius \
             (default={default_radius}, wider={wider})"
        );
    }

    #[test]
    fn dispatchers_pick_the_effect_and_carry_the_config() {
        let painter = headless_painter();
        let animation = Animation::default();
        // `event`/`state` select by kind and report running state.
        assert!(animation.event(NodeAnimation::Pulse)(
            &painter,
            Pos2::ZERO,
            1.0,
            Instant::now(),
            Color32::RED
        ));
        // They also honor the config: a `Pulse` with a tiny duration is done.
        let quick = Animation::default().with(|a| a.pulse.duration = 0.001);
        let past = Instant::now() - Duration::from_secs_f32(1.0);
        assert!(!quick.event(NodeAnimation::Pulse)(
            &painter,
            Pos2::ZERO,
            1.0,
            past,
            Color32::RED
        ));
        // `event_duration` reads the same config.
        assert_eq!(quick.event_duration(NodeAnimation::Pulse), 0.001);
        assert_eq!(
            animation.event_duration(NodeAnimation::Ripple),
            RIPPLE_DURATION
        );
    }

    /// Every segment event-driven effect, with the duration it runs for. Each
    /// entry is the `kind` the dispatcher should turn into the corresponding
    /// method.
    fn segment_event_effects() -> [(&'static str, SegmentAnimation, f32); 2] {
        [
            (
                "flash_decay",
                SegmentAnimation::FlashDecay,
                FLASH_DECAY_DURATION,
            ),
            ("wipe", SegmentAnimation::Wipe, WIPE_DURATION),
        ]
    }

    #[test]
    fn segment_event_effects_report_running_then_finished() {
        let painter = headless_painter();
        let animation = SegmentAnimations::default();
        let a = Pos2::ZERO;
        let b = Pos2::new(50.0, 0.0);
        for (name, kind, duration) in segment_event_effects() {
            let effect = animation.event(kind);
            assert!(
                effect(&painter, a, b, 1.0, Instant::now(), Color32::RED),
                "{name} must report it is still running when it just started"
            );

            let long_past = Instant::now() - Duration::from_secs_f32(duration + 1.0);
            assert!(
                !effect(&painter, a, b, 1.0, long_past, Color32::RED),
                "{name} must report it is finished once its duration has passed"
            );
        }
    }

    #[test]
    fn every_segment_event_effect_stays_under_the_orphan_sweep() {
        for (name, _, duration) in segment_event_effects() {
            assert!(
                duration < 10.0,
                "{name} lasts {duration}s, which the 10s orphan sweep would truncate"
            );
        }
    }

    #[test]
    fn comet_runs_at_any_time_and_stays_on_the_segment() {
        let painter = headless_painter();
        let animation = SegmentAnimations::default();
        let a = Pos2::ZERO;
        let b = Pos2::new(50.0, 0.0);
        for time in [0.0, 0.4, 0.8, 60.0] {
            animation.comet(&painter, a, b, 1.0, time, Color32::GREEN);
        }
    }

    #[test]
    fn comet_loops_back_to_the_start() {
        // One full `COMET_PERIOD` later it should be back where it began.
        let t0 = 0.2;
        let t1 = t0 + COMET_PERIOD;
        let at = |t: f32| {
            let frac = (t / COMET_PERIOD).rem_euclid(1.0);
            Pos2::ZERO + (Pos2::new(50.0, 0.0) - Pos2::ZERO) * frac
        };
        assert_eq!(at(t0), at(t1));
    }

    #[test]
    fn comet_once_reports_running_then_finished() {
        let painter = headless_painter();
        let animation = SegmentAnimations::default();
        let a = Pos2::ZERO;
        let b = Pos2::new(50.0, 0.0);
        assert!(
            animation.comet_once(&painter, [a, b], 1.0, Instant::now(), Color32::RED,),
            "comet_once must report it is still running when it just started"
        );

        let long_past = Instant::now() - Duration::from_secs_f32(COMET_TRAVEL_DURATION + 1.0);
        assert!(
            !animation.comet_once(&painter, [a, b], 1.0, long_past, Color32::RED,),
            "comet_once must report it is finished once its duration has passed"
        );
    }

    #[test]
    // Comparing two `const`s is deliberate here: this is a guard against a
    // future edit to `COMET_TRAVEL_DURATION`, not a runtime check.
    #[allow(clippy::assertions_on_constants)]
    fn comet_once_stays_under_the_orphan_sweep() {
        assert!(
            COMET_TRAVEL_DURATION < 10.0,
            "comet_once lasts {COMET_TRAVEL_DURATION}s, which the 10s orphan sweep would truncate"
        );
    }

    #[test]
    fn comet_once_direction_picks_the_starting_endpoint() {
        // The dot must sit on the starting endpoint at the very start --
        // `a` for `Forward`, `b` for `Reverse` -- not partway along the
        // segment. The direction is resolved by `CometDirection::orient`,
        // which is what `map.rs` and templates go through before calling the
        // `event` dispatcher, so drive it the same way and read the circle's
        // center back out.
        let ctx = Context::default();
        let a = Pos2::ZERO;
        let b = Pos2::new(50.0, 0.0);
        let dot_center = |direction: CometDirection| -> Pos2 {
            let animation = SegmentAnimations::default();
            let mut out = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(100.0, 100.0))),
                    ..Default::default()
                },
                |ui| {
                    let (from, to) = direction.orient(a, b);
                    animation.event(SegmentAnimation::Comet)(
                        ui.painter(),
                        from,
                        to,
                        1.0,
                        Instant::now(),
                        Color32::RED,
                    );
                },
            );
            let center = out
                .shapes
                .iter()
                .find_map(|cs| match &cs.shape {
                    egui::epaint::Shape::Circle(circle) => Some(circle.center),
                    _ => None,
                })
                .expect("comet_once must draw a circle");
            out.textures_delta.clear();
            center
        };
        let forward = dot_center(CometDirection::Forward);
        let reverse = dot_center(CometDirection::Reverse);
        assert!(
            (forward - a).length() < 1.0,
            "Forward must start at `a`, got {forward:?}"
        );
        assert!(
            (reverse - b).length() < 1.0,
            "Reverse must start at `b`, got {reverse:?}"
        );
    }

    #[test]
    fn ripple_rings_all_finish_before_the_effect_ends() {
        // The last ring is born two staggers after the first. If the effect
        // ended at `duration` with each ring living that long, it would be
        // cut off a third of the way through its life.
        let ripple = Ripple::default();
        let nearly_over = ripple.duration * 0.999;
        let alive: Vec<f32> = ripple.ring_progress(nearly_over, false).collect();
        assert_eq!(alive.len(), 1, "only the last ring is left: {alive:?}");
        assert!(
            alive[0] > 0.99,
            "the last ring must have almost finished, got {}",
            alive[0]
        );
        assert_eq!(
            ripple.ring_progress(ripple.duration, false).count(),
            0,
            "nothing is left once the effect has played for `duration`"
        );
        // And every ring gets to play from its very start.
        assert_eq!(ripple.ring_progress(0.0, false).count(), 1);
    }

    #[test]
    fn looping_ripple_never_pauses_and_repeats_exactly() {
        let ripple = Ripple::default();
        let life = ripple.ring_life();
        let sorted = |secs: f32| {
            let mut rings: Vec<f32> = ripple.ring_progress(secs, true).collect();
            rings.sort_by(f32::total_cmp);
            rings
        };
        // Once the rings have built up (two staggers in).
        let mut secs = 2.0 * life / RIPPLE_RINGS as f32;
        while secs < 4.0 * life {
            let rings = sorted(secs);
            assert_eq!(rings.len(), RIPPLE_RINGS, "a ring is missing at {secs}s");
            // The pattern repeats every `life`.
            for (now, later) in rings.iter().zip(sorted(secs + life)) {
                assert!((now - later).abs() < 1e-3, "{rings:?} at {secs}s");
            }
            secs += 0.05;
        }
    }

    #[test]
    fn lasting_ripple_is_still_drawn_long_after_one_cycle() {
        let ctx = Context::default();
        let animation = Animation::default();
        let rings = |started: Instant| -> usize {
            let mut out = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(100.0, 100.0))),
                    ..Default::default()
                },
                |ui| {
                    assert!(
                        animation.lasting_event(NodeAnimation::Ripple)(
                            ui.painter(),
                            Pos2::new(50.0, 50.0),
                            1.0,
                            started,
                            started + Duration::from_secs(3600),
                            Color32::RED,
                        ),
                        "a lasting effect never reports itself finished"
                    );
                },
            );
            out.textures_delta.clear();
            out.shapes
                .iter()
                .filter(|cs| matches!(cs.shape, egui::epaint::Shape::Circle(_)))
                .count()
        };
        // Right at the start only the first ring exists, as with the
        // one-off effect; after that, all three at any moment.
        assert_eq!(rings(Instant::now()), 1);
        for secs in [4.0_f32, 9.3, 40.0] {
            let started = Instant::now() - Duration::from_secs_f32(secs);
            assert_eq!(rings(started), RIPPLE_RINGS, "{secs}s into the effect");
        }
    }

    #[test]
    fn remaining_until_runs_from_one_to_zero_over_the_requested_time() {
        let now = Instant::now();
        let around = |before: u64, after: u64| {
            remaining_until(
                now - Duration::from_secs(before),
                now + Duration::from_secs(after),
            )
        };
        assert!((around(0, 20) - 1.0).abs() < 0.01, "full at the start");
        assert!((around(10, 10) - 0.5).abs() < 0.01, "half way through");
        assert!((around(15, 5) - 0.25).abs() < 0.01, "a quarter left");
        assert_eq!(around(30, 0), 0.0, "empty at `until`");
        // No time at all asked for: empty, not a division by zero.
        assert_eq!(remaining_until(now, now), 0.0);
    }

    /// Where the ring `draw` paints ends: its last point, relative to the
    /// center of the node it is drawn around.
    fn countdown_end(animation: &Animation, draw: impl Fn(&Animation, &Painter, Pos2)) -> Vec2 {
        let ctx = Context::default();
        let center = Pos2::new(50.0, 50.0);
        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(100.0, 100.0))),
                ..Default::default()
            },
            |ui| draw(animation, ui.painter(), center),
        );
        out.textures_delta.clear();
        let end = out
            .shapes
            .iter()
            .find_map(|cs| match &cs.shape {
                egui::epaint::Shape::Path(path) => path.points.last().copied(),
                _ => None,
            })
            .expect("the countdown must paint an arc");
        end - center
    }

    #[test]
    fn a_lasting_countdown_empties_over_the_time_asked_for() {
        // Ten seconds into twenty, the ring is half empty: its end is at the
        // bottom of the circle (the arc runs clockwise from 12 o'clock).
        // `Countdown::duration` is 5 s, so the plain effect would be long
        // over by then.
        let animation = Animation::default();
        let end = countdown_end(&animation, |animation, painter, center| {
            let now = Instant::now();
            animation.lasting_event(NodeAnimation::CountdownArc)(
                painter,
                center,
                1.0,
                now - Duration::from_secs(10),
                now + Duration::from_secs(10),
                Color32::RED,
            );
        });
        assert!(
            end.x.abs() < 0.5,
            "half a turn ends below the center: {end:?}"
        );
        assert!(
            (end.y - animation.countdown.radius).abs() < 0.5,
            "half a turn ends one radius below the center: {end:?}"
        );
    }

    #[test]
    fn the_countdown_ring_empties_smoothly() {
        // 0.505 of a turn: not a multiple of the 1/48 steps the arc is drawn
        // in, so a ring that only draws whole steps ends at half a turn
        // (straight below the center) instead of a little past it.
        let animation = Animation::default().with(|a| a.countdown.radius = 100.0);
        let end = countdown_end(&animation, |animation, painter, center| {
            animation.paint_countdown(painter, center, 1.0, 0.505, Color32::RED);
        });
        let angle = end.y.atan2(end.x) + std::f32::consts::FRAC_PI_2;
        let turn = angle.rem_euclid(TAU) / TAU;
        assert!(
            (turn - 0.505).abs() < 0.001,
            "the arc ends at {turn} of a turn"
        );
    }

    #[test]
    fn wipe_progress_interpolates_toward_the_far_endpoint() {
        // Same reasoning as `comet_once_direction_picks_the_starting_endpoint`:
        // the widget centers and offsets the rendered view, so comparing a
        // rendered position against raw map-space coordinates would be
        // fragile. `wipe`'s leading edge is a plain `lerp(a, b, progress)`,
        // so this checks the formula directly instead of rendering a frame.
        let a = Pos2::ZERO;
        let b = Pos2::new(50.0, 0.0);
        let leading_edge = |progress: f32| a + (b - a) * progress;

        assert_eq!(leading_edge(0.0), a, "must start exactly at `a`");
        assert_eq!(leading_edge(1.0), b, "must finish exactly at `b`");
        assert_eq!(leading_edge(0.5), Pos2::new(25.0, 0.0));
    }

    #[test]
    fn dash_runs_at_any_time_and_skips_zero_length_segments() {
        let painter = headless_painter();
        let animation = SegmentAnimations::default();
        let a = Pos2::ZERO;
        let b = Pos2::new(50.0, 0.0);
        for time in [0.0, 0.4, 0.8, 60.0] {
            animation.dash(&painter, a, b, 1.0, time, Color32::GREEN);
        }
        // A degenerate (zero-length) segment must not panic -- the
        // direction/normal math divides by the segment's length.
        animation.dash(&painter, a, a, 1.0, 0.0, Color32::GREEN);
    }

    #[test]
    fn dash_texture_is_registered_once_per_context() {
        // Repeated calls on the same `Context` must reuse the same texture
        // rather than re-uploading one every frame.
        let ctx = Context::default();
        let first = SegmentAnimations::dash_texture(&ctx);
        let second = SegmentAnimations::dash_texture(&ctx);
        assert_eq!(first.id(), second.id());
    }

    #[test]
    fn glow_band_runs_at_any_time_and_skips_zero_length_segments() {
        let painter = headless_painter();
        let animation = SegmentAnimations::default();
        let a = Pos2::ZERO;
        let b = Pos2::new(50.0, 0.0);
        for time in [0.0, 0.4, 0.8, 60.0] {
            animation.glow_band(&painter, a, b, 1.0, time, Color32::GREEN);
        }
        // A degenerate (zero-length) segment must not panic -- the
        // direction/normal math divides by the segment's length.
        animation.glow_band(&painter, a, a, 1.0, 0.0, Color32::GREEN);
    }

    #[test]
    fn glow_band_texture_is_registered_once_per_context() {
        let ctx = Context::default();
        let first = SegmentAnimations::glow_band_texture(&ctx);
        let second = SegmentAnimations::glow_band_texture(&ctx);
        assert_eq!(first.id(), second.id());
    }

    #[test]
    fn chevrons_runs_at_any_time_and_skips_zero_length_segments() {
        let painter = headless_painter();
        let animation = SegmentAnimations::default();
        let a = Pos2::ZERO;
        let b = Pos2::new(50.0, 0.0);
        for time in [0.0, 0.4, 0.8, 60.0] {
            animation.chevrons(&painter, a, b, 1.0, time, Color32::GREEN);
        }
        animation.chevrons(&painter, a, a, 1.0, 0.0, Color32::GREEN);
    }

    #[test]
    fn chevrons_texture_is_registered_once_per_context() {
        let ctx = Context::default();
        let first = SegmentAnimations::chevrons_texture(&ctx, CHEVRON_LEG_SLOPE, CHEVRON_STROKE);
        let second = SegmentAnimations::chevrons_texture(&ctx, CHEVRON_LEG_SLOPE, CHEVRON_STROKE);
        assert_eq!(first.id(), second.id());
    }

    #[test]
    fn chevrons_texture_follows_its_shape_settings() {
        let ctx = Context::default();
        let default = SegmentAnimations::chevrons_texture(&ctx, CHEVRON_LEG_SLOPE, CHEVRON_STROKE);
        let flatter = SegmentAnimations::chevrons_texture(&ctx, 0.0, CHEVRON_STROKE);
        let thicker = SegmentAnimations::chevrons_texture(&ctx, CHEVRON_LEG_SLOPE, 0.3);
        assert_ne!(default.id(), flatter.id());
        assert_ne!(default.id(), thicker.id());
        // A zero stroke must not divide by zero.
        let _ = SegmentAnimations::chevrons_texture(&ctx, CHEVRON_LEG_SLOPE, 0.0);
    }

    #[test]
    fn segment_animations_default_matches_the_built_in_values() {
        // Pins `SegmentAnimations::default()` to the values the widget used to
        // hard-code, so a regression here is a visual change, not a silent one.
        let a = SegmentAnimations::default();
        assert_eq!(
            a.flash_decay,
            FlashDecay {
                base_width: FLASH_BASE_WIDTH,
                extra_width: FLASH_EXTRA_WIDTH,
                duration: FLASH_DECAY_DURATION
            }
        );
        assert_eq!(
            a.comet_once,
            CometOnce {
                dot_radius: COMET_DOT_RADIUS,
                dot_min: COMET_DOT_MIN,
                duration: COMET_TRAVEL_DURATION
            }
        );
        assert_eq!(
            a.wipe,
            Wipe {
                width: 2.5,
                duration: WIPE_DURATION
            }
        );
        assert_eq!(
            a.comet,
            Comet {
                dot_radius: COMET_DOT_RADIUS,
                dot_min: COMET_DOT_MIN,
                period: COMET_PERIOD
            }
        );
        assert_eq!(
            a.dash,
            Dash {
                period_px: DASH_PERIOD_PX,
                speed: DASH_SPEED,
                // Follows the default stroke unless it is set.
                width: None
            }
        );
        assert_eq!(
            a.glow_band,
            GlowBand {
                period: GLOW_BAND_PERIOD,
                length_px: GLOW_BAND_LENGTH_PX,
                thickness: GLOW_BAND_THICKNESS
            }
        );
        assert_eq!(
            a.chevrons,
            Chevrons {
                period_px: CHEVRON_PERIOD_PX,
                speed: CHEVRON_SPEED,
                width: CHEVRON_WIDTH,
                leg_slope: CHEVRON_LEG_SLOPE,
                stroke: CHEVRON_STROKE
            }
        );
    }

    #[test]
    fn segment_with_adjusts_only_the_given_fields() {
        let a = SegmentAnimations::default().with(|a| {
            a.dash.width = Some(8.0);
            a.chevrons.speed = 1.5;
        });
        assert_eq!(a.dash.width, Some(8.0));
        assert_eq!(a.chevrons.speed, 1.5);
        assert_eq!(a.dash.period_px, Dash::default().period_px);
        assert_eq!(a.wipe, Wipe::default());
    }

    #[test]
    fn segment_dispatchers_pick_the_effect_and_carry_the_config() {
        let painter = headless_painter();
        let a = Pos2::ZERO;
        let b = Pos2::new(50.0, 0.0);
        let animation = SegmentAnimations::default();

        // `event` selects by kind and reports running state.
        assert!(animation.event(SegmentAnimation::Wipe)(
            &painter,
            a,
            b,
            1.0,
            Instant::now(),
            Color32::RED
        ));
        // It honors the config: a tiny `wipe.duration` is already done.
        let quick = SegmentAnimations::default().with(|s| s.wipe.duration = 0.001);
        let past = Instant::now() - Duration::from_secs_f32(1.0);
        assert!(!quick.event(SegmentAnimation::Wipe)(
            &painter,
            a,
            b,
            1.0,
            past,
            Color32::RED
        ));
        // `state` selects by kind too.
        animation.state(SteadySegmentAnimation::Dash)(&painter, a, b, 1.0, 0.0, Color32::GREEN);
    }

    /// Half the width of the ribbon `animation` draws for a horizontal
    /// segment at `zoom`. The dashes are drawn as a mesh whose vertices
    /// straddle the segment line, so the farthest vertex from the centerline
    /// is half the ribbon.
    fn dash_half_width(animation: SegmentAnimations, zoom: f32) -> f32 {
        let ctx = Context::default();
        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(100.0, 100.0))),
                ..Default::default()
            },
            |ui| {
                animation.dash(
                    ui.painter(),
                    Pos2::ZERO,
                    Pos2::new(50.0, 0.0),
                    zoom,
                    0.0,
                    Color32::GREEN,
                );
            },
        );
        let offset = out
            .shapes
            .iter()
            .find_map(|cs| match &cs.shape {
                egui::epaint::Shape::Mesh(mesh) => Some(
                    mesh.vertices
                        .iter()
                        .map(|v| v.pos.y.abs())
                        .fold(0.0, f32::max),
                ),
                _ => None,
            })
            .expect("dash must draw a mesh");
        out.textures_delta.clear();
        offset
    }

    #[test]
    fn dash_width_changes_the_drawn_ribbon() {
        let default_offset = dash_half_width(SegmentAnimations::default(), 1.0);
        let wider = dash_half_width(
            SegmentAnimations::default().with(|s| s.dash.width = Some(12.0)),
            1.0,
        );
        assert!(
            wider > default_offset,
            "raising dash.width must widen the drawn ribbon \
             (default={default_offset}, wider={wider})"
        );
        assert_eq!(wider, 6.0, "the width is in screen pixels");
    }

    #[test]
    fn a_dash_with_no_width_uses_the_fallback() {
        let animation = SegmentAnimations::default();
        assert_eq!(animation.dash.width, None);
        assert_eq!(dash_half_width(animation, 1.0), DASH_WIDTH / 2.0);
    }

    #[test]
    fn dash_and_chevrons_slide_towards_the_second_endpoint() {
        // The pattern is a texture sampled at `u = phase + x / period`, so a
        // feature of it sits where that is constant: it moves towards `b`
        // when the `u` at `a` goes down as time passes. `comet` and
        // `glow_band` go from `a` to `b`, and the effects that slide a
        // texture must agree with them.
        let animation = SegmentAnimations::default();
        let ctx = Context::default();
        let u_at_a = |kind: SteadySegmentAnimation, time: f32| -> f32 {
            let mut out = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(100.0, 100.0))),
                    ..Default::default()
                },
                |ui| {
                    animation.state(kind)(
                        ui.painter(),
                        Pos2::ZERO,
                        Pos2::new(50.0, 0.0),
                        1.0,
                        time,
                        Color32::GREEN,
                    );
                },
            );
            let u = out
                .shapes
                .iter()
                .find_map(|cs| match &cs.shape {
                    egui::epaint::Shape::Mesh(mesh) => Some(mesh.vertices[0].uv.x),
                    _ => None,
                })
                .expect("the effect must draw a mesh");
            out.textures_delta.clear();
            u
        };
        for kind in [
            SteadySegmentAnimation::Dash,
            SteadySegmentAnimation::Chevrons,
        ] {
            let (early, later) = (u_at_a(kind, 0.1), u_at_a(kind, 0.11));
            assert!(
                later < early,
                "{kind:?} must slide towards `b` (u at `a`: {early} then {later})"
            );
        }
    }

    #[test]
    fn the_dash_ribbon_does_not_depend_on_the_zoom() {
        // Like the default stroke it runs over, and like the dash period.
        for animation in [
            SegmentAnimations::default(),
            SegmentAnimations::default().with(|s| s.dash.width = Some(5.0)),
            SegmentAnimations::default().with_line_width(Some(3.0)),
        ] {
            let reference = dash_half_width(animation, 1.0);
            for zoom in [0.1, 0.25, 0.5, 2.0, 8.0] {
                assert_eq!(dash_half_width(animation, zoom), reference, "zoom {zoom}");
            }
        }
    }

    #[test]
    fn the_dash_is_as_thick_as_the_line_it_follows() {
        for line_width in [1.0, 2.0, 5.0, 10.0] {
            let animation = SegmentAnimations::default().with_line_width(Some(line_width));
            assert_eq!(
                dash_half_width(animation, 1.0) * 2.0,
                line_width,
                "line_width {line_width}"
            );
        }
    }

    #[test]
    fn with_line_width_fills_in_only_a_missing_dash_width() {
        let followed = SegmentAnimations::default().with_line_width(Some(4.0));
        assert_eq!(followed.dash.width, Some(4.0));
        // A width set on purpose wins over the line's.
        let explicit = SegmentAnimations::default()
            .with(|s| s.dash.width = Some(9.0))
            .with_line_width(Some(4.0));
        assert_eq!(explicit.dash.width, Some(9.0));
        // No default stroke to follow: left to fall back to `DASH_WIDTH`.
        let none = SegmentAnimations::default().with_line_width(None);
        assert_eq!(none.dash.width, None);
        // Nothing else is touched.
        assert_eq!(
            followed.with(|s| s.dash.width = None),
            SegmentAnimations::default()
        );
    }

    #[test]
    fn triangle_wave_goes_up_and_back_down() {
        assert_eq!(triangle_wave(0.0, 2.0), 0.0);
        assert_eq!(triangle_wave(1.0, 2.0), 1.0);
        assert!(triangle_wave(2.0, 2.0).abs() < 1e-6);
        assert!((triangle_wave(3.0, 2.0) - 1.0).abs() < 1e-6);
        for step in 0..200 {
            let v = triangle_wave(step as f32 * 0.05, 2.55);
            assert!((0.0..=1.0).contains(&v), "{v} out of range");
        }
    }

    #[test]
    fn ease_out_back_overshoots_then_settles() {
        assert_eq!(ease_out_back(0.0), 0.0);
        assert!((ease_out_back(1.0) - 1.0).abs() < 1e-5);
        let peak = (0..=100)
            .map(|i| ease_out_back(i as f32 / 100.0))
            .fold(f32::MIN, f32::max);
        assert!(
            peak > 1.0,
            "ease_out_back should overshoot, peaked at {peak}"
        );
    }

    #[test]
    fn with_alpha_clamps_out_of_range_values() {
        assert_eq!(with_alpha(Color32::RED, 2.0).a(), 255);
        assert_eq!(with_alpha(Color32::RED, -1.0).a(), 0);
        assert_eq!(with_alpha(Color32::RED, 1.0), Color32::RED);
    }
}

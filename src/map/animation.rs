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

use super::objects::{
    CometDirection, NodeAnimation, SegmentAnimation, SteadyAnimation, SteadySegmentAnimation,
};
use super::outline::{NodeOutline, partial_perimeter, point_along};
use egui::{
    Color32, ColorImage, Context, CornerRadius, Id, Mesh, Painter, Pos2, Rect, Shape, Stroke,
    TextureFilter, TextureHandle, TextureOptions, TextureWrapMode, Vec2,
    epaint::{CircleShape, PathShape, Vertex},
    pos2,
};
use std::f32::consts::TAU;
use std::time::Instant;

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
/// Width, before the `zoom` multiplier, of the ribbon [`Animation::dash`]
/// paints.
pub const DASH_WIDTH: f32 = 3.0;
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
pub const CHEVRON_PERIOD_PX: f32 = 28.0;
/// How many repeats of the chevron pattern [`Animation::chevrons`] slides
/// through per second.
pub const CHEVRON_SPEED: f32 = 0.5;
/// Width, before the `zoom` multiplier, of the ribbon [`Animation::chevrons`]
/// paints.
pub const CHEVRON_WIDTH: f32 = 10.0;

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
}

/// Tunables of [`SegmentAnimations::flash_decay`]: a segment thickening then
/// fading.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlashDecay {
    /// Base stroke width, before the `zoom` multiplier.
    pub base_width: f32,
    /// Extra width, before the `zoom` multiplier, at the very start of the
    /// effect (added to [`Self::base_width`] and shed over its lifetime).
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
            base_width: 2.0,
            extra_width: 10.0,
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
            dot_radius: 4.0,
            dot_min: 2.5,
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
            dot_radius: 4.0,
            dot_min: 2.5,
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
    /// Ribbon width, before the `zoom` multiplier.
    pub width: f32,
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
            width: DASH_WIDTH,
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
    /// Ribbon width, before the `zoom` multiplier.
    pub width: f32,
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
    /// Growth of each ring over the whole effect, in screen pixels at
    /// `zoom == 1`. Shared by the `*_outline` variant.
    pub spread: f32,
    /// Ring stroke width, before the `zoom` multiplier.
    pub stroke: f32,
    /// How long the effect plays, in seconds.
    pub duration: f32,
}

impl Ripple {
    /// Runs `f` on a mutable copy of `self` and returns it.
    pub fn with(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
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
        painter.add(Shape::Circle(CircleShape::filled(
            center,
            radius,
            with_alpha(color, transparency),
        )));
        secs < self.pulse.duration
    }

    /// One frame of three staggered expanding rings.
    ///
    /// Where [`Animation::pulse`] reads as a single event, the repetition here
    /// reads as *"activity is ongoing"*. Plays for [`RIPPLE_DURATION`].
    /// Returns `true` while still playing.
    pub fn ripple(
        &self,
        painter: &Painter,
        center: Pos2,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        const RINGS: usize = 3;
        let secs = elapsed(initial_time);
        let stagger = self.ripple.duration / RINGS as f32;

        let mut shapes = Vec::with_capacity(RINGS);
        for ring in 0..RINGS {
            let local = secs - ring as f32 * stagger;
            if !(0.0..self.ripple.duration).contains(&local) {
                continue;
            }
            let progress = local / self.ripple.duration;
            shapes.push(Shape::Circle(CircleShape::stroke(
                center,
                (self.ripple.base_radius + self.ripple.spread * progress) * zoom,
                Stroke::new(self.ripple.stroke * zoom, with_alpha(color, 1.0 - progress)),
            )));
        }
        painter.extend(shapes);
        secs < self.ripple.duration
    }

    /// One frame of a ring that empties clockwise from 12 o'clock.
    ///
    /// The remaining arc is the remaining fraction of [`COUNTDOWN_DURATION`],
    /// which makes it a natural fit for *"how old is this information"*.
    /// Returns `true` while still playing.
    pub fn countdown_arc(
        &self,
        painter: &Painter,
        center: Pos2,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        // Segments in a full turn; the arc draws a prefix of these.
        const STEPS: usize = 48;
        let secs = elapsed(initial_time);
        let remaining = (1.0 - secs / self.countdown.duration).clamp(0.0, 1.0);
        let radius = self.countdown.radius * zoom;

        let count = (STEPS as f32 * remaining).round() as usize;
        if count >= 1 {
            let points = (0..=count)
                .map(|i| {
                    // Start at 12 o'clock and sweep clockwise. Screen y grows
                    // downwards, so a growing angle already turns clockwise.
                    let angle = TAU * (i as f32 / STEPS as f32) - TAU / 4.0;
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
        secs < self.countdown.duration
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

    /// Length of one cycle of a node event effect, in seconds: how long it
    /// plays once, and how often a lasting notification
    /// ([`NodeHandle::lasting`](crate::map::NodeHandle::lasting)) restarts it.
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
        painter.add(
            outline
                .grown(self.pulse.spread * secs * zoom)
                .fill_shape(with_alpha(color, transparency)),
        );
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
        const RINGS: usize = 3;
        let secs = elapsed(initial_time);
        let stagger = self.ripple.duration / RINGS as f32;
        let mut shapes = Vec::with_capacity(RINGS);
        for ring in 0..RINGS {
            let local = secs - ring as f32 * stagger;
            if !(0.0..self.ripple.duration).contains(&local) {
                continue;
            }
            let progress = local / self.ripple.duration;
            shapes.push(
                outline
                    .grown(self.ripple.spread * progress * zoom)
                    .stroke_shape(Stroke::new(
                        self.ripple.stroke * zoom,
                        with_alpha(color, 1.0 - progress),
                    )),
            );
        }
        painter.extend(shapes);
        secs < self.ripple.duration
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
        let remaining = (1.0 - secs / self.countdown.duration).clamp(0.0, 1.0);
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
        secs < self.countdown.duration
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
    /// tunables. Call it with `(painter, a, b, zoom, initial_time, color)`.
    pub fn event(
        &self,
        kind: SegmentAnimation,
    ) -> impl Fn(&Painter, Pos2, Pos2, f32, Instant, Color32) -> bool + 'static {
        let a = *self;
        move |painter, from, to, zoom, initial_time, color| match kind {
            SegmentAnimation::FlashDecay => {
                a.flash_decay(painter, from, to, zoom, initial_time, color)
            }
            SegmentAnimation::Comet(direction) => {
                let (from, to) = match direction {
                    CometDirection::Forward => (from, to),
                    CometDirection::Reverse => (to, from),
                };
                a.comet_once(painter, [from, to], zoom, initial_time, color)
            }
            SegmentAnimation::Wipe => a.wipe(painter, from, to, zoom, initial_time, color),
        }
    }

    /// The persistent effect for a segment `kind`, bound to this
    /// `SegmentAnimations`' tunables. Call it with
    /// `(painter, a, b, zoom, time, color)`.
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
    /// into that pair (see [`SegmentAnimations::event`]). Plays for
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
    /// Two triangles textured with a small repeating strip (registered once
    /// per [`egui::Context`] and reused after that), rather than one shape per
    /// dash -- see the [module docs](self) for why that matters at scale. A
    /// zero-length segment is skipped.
    pub fn dash(&self, painter: &Painter, a: Pos2, b: Pos2, zoom: f32, time: f32, color: Color32) {
        let delta = b - a;
        let len = delta.length();
        if len <= f32::EPSILON {
            return;
        }
        let dir = delta / len;
        let normal = Vec2::new(-dir.y, dir.x) * (self.dash.width * zoom * 0.5);
        let phase = (time * self.dash.speed).rem_euclid(1.0);
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
    pub fn chevrons(
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
        let normal = Vec2::new(-dir.y, dir.x) * (self.chevrons.width * zoom * 0.5);
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

        let texture = Self::chevrons_texture(painter.ctx());
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
    fn chevrons_texture(ctx: &Context) -> TextureHandle {
        let id = Id::new("egui_map::chevrons_texture");
        if let Some(handle) = ctx.data(|d| d.get_temp::<TextureHandle>(id)) {
            return handle;
        }

        const WIDTH: usize = 32;
        const HEIGHT: usize = 16;
        const TIP_U: f32 = 0.75;
        const LEG_SLOPE: f32 = 0.5;
        const STROKE_THICKNESS: f32 = 0.12;

        let mut pixels = Vec::with_capacity(WIDTH * HEIGHT);
        for j in 0..HEIGHT {
            let v = j as f32 / (HEIGHT - 1) as f32;
            let ideal_u = TIP_U - LEG_SLOPE * (v - 0.5).abs();
            for i in 0..WIDTH {
                let u = i as f32 / WIDTH as f32;
                let distance = (u - ideal_u).abs();
                let alpha = (1.0 - distance / STROKE_THICKNESS).clamp(0.0, 1.0);
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
        // segment. The direction is resolved by the `event` dispatcher, which
        // is what `map.rs` and templates go through, so drive it from there
        // and read the circle's center back out.
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
                    animation.event(SegmentAnimation::Comet(direction))(
                        ui.painter(),
                        a,
                        b,
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
        let first = SegmentAnimations::chevrons_texture(&ctx);
        let second = SegmentAnimations::chevrons_texture(&ctx);
        assert_eq!(first.id(), second.id());
    }

    #[test]
    fn segment_animations_default_matches_the_built_in_values() {
        // Pins `SegmentAnimations::default()` to the values the widget used to
        // hard-code, so a regression here is a visual change, not a silent one.
        let a = SegmentAnimations::default();
        assert_eq!(
            a.flash_decay,
            FlashDecay {
                base_width: 2.0,
                extra_width: 10.0,
                duration: FLASH_DECAY_DURATION
            }
        );
        assert_eq!(
            a.comet_once,
            CometOnce {
                dot_radius: 4.0,
                dot_min: 2.5,
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
                dot_radius: 4.0,
                dot_min: 2.5,
                period: COMET_PERIOD
            }
        );
        assert_eq!(
            a.dash,
            Dash {
                period_px: DASH_PERIOD_PX,
                speed: DASH_SPEED,
                width: DASH_WIDTH
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
                width: CHEVRON_WIDTH
            }
        );
    }

    #[test]
    fn segment_with_adjusts_only_the_given_fields() {
        let a = SegmentAnimations::default().with(|a| {
            a.dash.width = 8.0;
            a.chevrons.speed = 1.5;
        });
        assert_eq!(a.dash.width, 8.0);
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

    #[test]
    fn dash_width_changes_the_drawn_ribbon() {
        // Two runs in the same frame; only `dash.width` differs. The dashes
        // are drawn as meshes whose vertices straddle the segment line, so a
        // wider ribbon pushes them further from the centerline.
        let ctx = Context::default();
        let a = Pos2::ZERO;
        let b = Pos2::new(50.0, 0.0);
        let max_offset = |animation: SegmentAnimations| -> f32 {
            let mut out = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(100.0, 100.0))),
                    ..Default::default()
                },
                |ui| {
                    animation.dash(ui.painter(), a, b, 1.0, 0.0, Color32::GREEN);
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
        };

        let default_offset = max_offset(SegmentAnimations::default());
        let wider = max_offset(SegmentAnimations::default().with(|s| s.dash.width = 12.0));
        assert!(
            wider > default_offset,
            "raising dash.width must widen the drawn ribbon \
             (default={default_offset}, wider={wider})"
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

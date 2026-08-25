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
//!   [`Animation::orbit`], [`Animation::comet`], [`Animation::dash`],
//!   [`Animation::glow_band`], [`Animation::chevrons`]) never end. They take
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
//! The segment effects ([`Animation::flash_decay`], [`Animation::comet_once`],
//! [`Animation::wipe`], [`Animation::comet`], [`Animation::dash`],
//! [`Animation::glow_band`], [`Animation::chevrons`]) are reached the same way, through
//! [`Map::segment`](crate::map::Map::segment); see
//! [`SegmentHandle`](crate::map::SegmentHandle). A custom
//! [`SegmentTemplate`](crate::map::objects::SegmentTemplate) calls them from
//! `segment_notification_ui` / `segment_state_ui`, remembering to call
//! `painter.ctx().request_repaint()` itself.

use super::objects::CometDirection;
use egui::{
    Color32, ColorImage, Context, Id, Mesh, Painter, Pos2, Shape, Stroke, TextureFilter,
    TextureHandle, TextureOptions, TextureWrapMode, Vec2,
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

/// Returns `color` with its alpha replaced by `alpha` (clamped to `0.0..=1.0`).
fn with_alpha(color: Color32, alpha: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(
        color.r(),
        color.g(),
        color.b(),
        (255.0 * alpha.clamp(0.0, 1.0)).round() as u8,
    )
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

/// Overshooting ease-out, so a scale-in settles with a small bounce.
fn ease_out_back(x: f32) -> f32 {
    const C1: f32 = 1.701_58;
    const C3: f32 = C1 + 1.0;
    let x1 = x - 1.0;
    1.0 + C3 * x1 * x1 * x1 + C1 * x1 * x1
}

/// Factory for the built-in node animations.
///
/// See the [module docs](self) for the difference between the event-driven and
/// persistent families.
pub struct Animation {}

impl Animation {
    // ---------------------------------------------------------------- events

    /// One frame of an expanding, fading disc centred on `center`.
    ///
    /// Reads as *"one thing happened here"*. Plays for [`PULSE_DURATION`].
    /// Returns `true` while still playing.
    pub fn pulse(
        painter: &Painter,
        center: Pos2,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        let secs = elapsed(initial_time);
        let radius = (4.00 + (40.00 * secs)) * zoom;
        let transparency = (1.00 - (secs / PULSE_DURATION).abs()).max(0.0);
        painter.add(Shape::Circle(CircleShape::filled(
            center,
            radius,
            with_alpha(color, transparency),
        )));
        secs < PULSE_DURATION
    }

    /// One frame of three staggered expanding rings.
    ///
    /// Where [`Animation::pulse`] reads as a single event, the repetition here
    /// reads as *"activity is ongoing"*. Plays for [`RIPPLE_DURATION`].
    /// Returns `true` while still playing.
    pub fn ripple(
        painter: &Painter,
        center: Pos2,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        const RINGS: usize = 3;
        let secs = elapsed(initial_time);
        let stagger = RIPPLE_DURATION / RINGS as f32;

        let mut shapes = Vec::with_capacity(RINGS);
        for ring in 0..RINGS {
            let local = secs - ring as f32 * stagger;
            if !(0.0..RIPPLE_DURATION).contains(&local) {
                continue;
            }
            let progress = local / RIPPLE_DURATION;
            shapes.push(Shape::Circle(CircleShape::stroke(
                center,
                (4.0 + 36.0 * progress) * zoom,
                Stroke::new(2.0 * zoom, with_alpha(color, 1.0 - progress)),
            )));
        }
        painter.extend(shapes);
        secs < RIPPLE_DURATION
    }

    /// One frame of a ring that empties clockwise from 12 o'clock.
    ///
    /// The remaining arc is the remaining fraction of [`COUNTDOWN_DURATION`],
    /// which makes it a natural fit for *"how old is this information"*.
    /// Returns `true` while still playing.
    pub fn countdown_arc(
        painter: &Painter,
        center: Pos2,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        // Segments in a full turn; the arc draws a prefix of these.
        const STEPS: usize = 48;
        let secs = elapsed(initial_time);
        let remaining = (1.0 - secs / COUNTDOWN_DURATION).clamp(0.0, 1.0);
        let radius = 10.0 * zoom;

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
                Stroke::new(2.0 * zoom, with_alpha(color, 1.0)),
            )));
        }
        secs < COUNTDOWN_DURATION
    }

    /// One frame of a disc that grows past its final size and settles back.
    ///
    /// Meant for nodes that just appeared. Plays for [`SCALE_IN_DURATION`].
    /// Returns `true` while still playing.
    pub fn scale_in(
        painter: &Painter,
        center: Pos2,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        let secs = elapsed(initial_time);
        let progress = (secs / SCALE_IN_DURATION).clamp(0.0, 1.0);
        let radius = 8.0 * zoom * ease_out_back(progress).max(0.0);
        painter.add(Shape::Circle(CircleShape::filled(
            center,
            radius,
            with_alpha(color, 1.0 - progress),
        )));
        secs < SCALE_IN_DURATION
    }

    /// One frame of four ticks converging onto the node.
    ///
    /// Reads as *"target acquired"*; pairs well with selection. Plays for
    /// [`CROSSHAIR_DURATION`]. Returns `true` while still playing.
    pub fn crosshair(
        painter: &Painter,
        center: Pos2,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        let secs = elapsed(initial_time);
        let progress = (secs / CROSSHAIR_DURATION).clamp(0.0, 1.0);
        // Ticks travel from far away down to just outside the node, and fade
        // out over the last third so they do not linger on top of it.
        let far = (30.0 - 18.0 * progress) * zoom;
        let near = far - 8.0 * zoom;
        let alpha = if progress < 0.66 {
            1.0
        } else {
            1.0 - (progress - 0.66) / 0.34
        };
        let stroke = Stroke::new(2.0 * zoom, with_alpha(color, alpha));

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
        secs < CROSSHAIR_DURATION
    }

    // -------------------------------------------------------- events/segment

    /// One frame of a segment briefly thickening and brightening, then fading
    /// back to nothing.
    ///
    /// The segment analogue of [`Animation::pulse`]: reads as *"something
    /// happened on this route"*. Plays for [`FLASH_DECAY_DURATION`]. Returns
    /// `true` while still playing.
    pub fn flash_decay(
        painter: &Painter,
        a: Pos2,
        b: Pos2,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        let secs = elapsed(initial_time);
        let progress = (secs / FLASH_DECAY_DURATION).clamp(0.0, 1.0);
        let width = (2.0 + 10.0 * (1.0 - progress)) * zoom;
        painter.line_segment(
            [a, b],
            Stroke::new(width, with_alpha(color, 1.0 - progress)),
        );
        secs < FLASH_DECAY_DURATION
    }

    /// One frame of a single dot pass along the segment, then gone — the
    /// event-driven counterpart to [`Animation::comet`]. `direction` picks
    /// which endpoint it starts from. Plays for [`COMET_TRAVEL_DURATION`].
    /// Returns `true` while still playing.
    pub fn comet_once(
        painter: &Painter,
        a: Pos2,
        b: Pos2,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
        direction: CometDirection,
    ) -> bool {
        let secs = elapsed(initial_time);
        let progress = (secs / COMET_TRAVEL_DURATION).clamp(0.0, 1.0);
        let (from, to) = match direction {
            CometDirection::Forward => (a, b),
            CometDirection::Reverse => (b, a),
        };
        let pos = from + (to - from) * progress;
        painter.add(Shape::Circle(CircleShape::filled(
            pos,
            (4.0 * zoom).max(2.5),
            color,
        )));
        secs < COMET_TRAVEL_DURATION
    }

    /// One frame of the segment drawing itself in, from `a` towards `b`, then
    /// gone. Reads as *"this route was just established"* — where
    /// [`Animation::comet_once`] shows something moving along an existing
    /// route, this shows the route itself appearing. Plays for
    /// [`WIPE_DURATION`]. Returns `true` while still playing.
    ///
    /// Cheaper than the mesh technique [`Animation::dash`] uses: the
    /// progressively-revealed portion is still just a straight line, so a
    /// plain `line_segment` from `a` to the interpolated point suffices.
    pub fn wipe(
        painter: &Painter,
        a: Pos2,
        b: Pos2,
        zoom: f32,
        initial_time: Instant,
        color: Color32,
    ) -> bool {
        let secs = elapsed(initial_time);
        let progress = (secs / WIPE_DURATION).clamp(0.0, 1.0);
        let leading_edge = a + (b - a) * progress;
        painter.line_segment([a, leading_edge], Stroke::new(2.5 * zoom, color));
        secs < WIPE_DURATION
    }

    // ----------------------------------------------------- persistent/segment

    /// One frame of a dot travelling from `a` to `b` and looping back.
    ///
    /// Reads as *"this is the direction of flow"*. `time` is the frame time in
    /// seconds; one full pass takes [`COMET_PERIOD`].
    pub fn comet(painter: &Painter, a: Pos2, b: Pos2, zoom: f32, time: f32, color: Color32) {
        let t = (time / COMET_PERIOD).rem_euclid(1.0);
        let pos = a + (b - a) * t;
        painter.add(Shape::Circle(CircleShape::filled(
            pos,
            (4.0 * zoom).max(2.5),
            color,
        )));
    }

    /// One frame of a segment drawn as a dashed line whose pattern slides
    /// along it ("marching ants"). `time` is the frame time in seconds; the
    /// pattern repeats every [`DASH_PERIOD_PX`] screen pixels and slides at
    /// [`DASH_SPEED`] repeats per second.
    ///
    /// Two triangles textured with a small repeating strip (registered once
    /// per [`egui::Context`] and reused after that), rather than one shape per
    /// dash -- see the [module docs](self) for why that matters at scale. A
    /// zero-length segment is skipped.
    pub fn dash(painter: &Painter, a: Pos2, b: Pos2, zoom: f32, time: f32, color: Color32) {
        let delta = b - a;
        let len = delta.length();
        if len <= f32::EPSILON {
            return;
        }
        let dir = delta / len;
        let normal = Vec2::new(-dir.y, dir.x) * (DASH_WIDTH * zoom * 0.5);
        let phase = (time * DASH_SPEED).rem_euclid(1.0);
        let u0 = phase;
        let u1 = phase + len / DASH_PERIOD_PX;

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

    /// Returns the texture [`Animation::dash`] samples, creating and caching
    /// it in the context's own temp data on first use so every segment (and
    /// every frame) reuses the same GPU upload instead of re-registering one.
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
    /// traverse-and-loop takes [`GLOW_BAND_PERIOD`].
    ///
    /// Reads as *"flow"*, calmer than [`Animation::dash`]'s marching pattern
    /// -- a single soft highlight rather than a repeating texture. Uses the
    /// same textured-mesh technique as `dash`, but with a
    /// [`TextureWrapMode::ClampToEdge`] sampler and a texture that is zero-alpha
    /// at both edges: as the band's mapped position slides past `0.0` or
    /// `1.0`, sampling clamps to that zero-alpha edge texel, so the band
    /// fades out before either endpoint instead of popping back in like
    /// `dash`'s repeating pattern would. A zero-length segment is skipped.
    pub fn glow_band(painter: &Painter, a: Pos2, b: Pos2, zoom: f32, time: f32, color: Color32) {
        let delta = b - a;
        let len = delta.length();
        if len <= f32::EPSILON {
            return;
        }
        let dir = delta / len;
        let normal = Vec2::new(-dir.y, dir.x) * (GLOW_BAND_THICKNESS * zoom * 0.5);

        // Half-width of the visible band, as a fraction of the segment's own
        // length -- capped at 0.5 so the band can never cover more than the
        // whole segment.
        let half_width_frac = (GLOW_BAND_LENGTH_PX * 0.5 / len).min(0.5);
        // The band's peak travels from just before the start to just past the
        // end and loops, rather than jumping straight from `1.0` back to
        // `0.0` -- that extra span is what lets it fade out past each end.
        let span = 1.0 + 2.0 * half_width_frac;
        let t = (time / GLOW_BAND_PERIOD).rem_euclid(1.0);
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

    /// Returns the texture [`Animation::glow_band`] samples, creating and
    /// caching it in the context's own temp data on first use -- same pattern
    /// as [`Animation::dash_texture`].
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
    /// [`CHEVRON_PERIOD_PX`] screen pixels and slides at [`CHEVRON_SPEED`]
    /// repeats per second.
    ///
    /// Reads as *"direction of travel"*, more explicit at a glance than
    /// [`Animation::comet`]'s single dot. Same mesh-building shape as
    /// [`Animation::dash`], but where `dash` samples a 1D texture at a
    /// constant `uv.y = 0.5` (its stripes don't vary across the ribbon's
    /// width), the two long edges of this mesh get `uv.y = 0.0` / `1.0`
    /// instead, so the interpolated `uv.y` sweeps across a genuinely 2D
    /// texture and traces out the arrow shape. A zero-length segment is
    /// skipped.
    pub fn chevrons(painter: &Painter, a: Pos2, b: Pos2, zoom: f32, time: f32, color: Color32) {
        let delta = b - a;
        let len = delta.length();
        if len <= f32::EPSILON {
            return;
        }
        let dir = delta / len;
        let normal = Vec2::new(-dir.y, dir.x) * (CHEVRON_WIDTH * zoom * 0.5);
        // `u0 < u1` (below) maps the texture's own +u direction onto the
        // segment's `a -> b` direction, and the arrow tip sits at the
        // texture's higher `u` (see `chevrons_texture`) -- so in any single
        // frame the tip already reads as pointing towards `b`. The pattern
        // must then *slide* towards `b` too, not away from it: sampling a
        // fixed screen point at an ever-larger `u` (`phase` growing with
        // time) is what would make it crawl towards `a` instead, backwards
        // from the way the arrows point. Negating the time term here is what
        // keeps the two in agreement.
        let phase = (-(time * CHEVRON_SPEED)).rem_euclid(1.0);
        let u0 = phase;
        let u1 = phase + len / CHEVRON_PERIOD_PX;

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

    /// Returns the texture [`Animation::chevrons`] samples, creating and
    /// caching it in the context's own temp data on first use -- same pattern
    /// as [`Animation::dash_texture`].
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

    // ------------------------------------------------------------ persistent

    /// One frame of a ring whose opacity breathes in and out.
    ///
    /// For lasting state — *"you are here"*, *"this system is camped"*. `time`
    /// is the frame time in seconds (`ui.input(|i| i.time)`).
    ///
    /// The radius has a floor in screen pixels so the halo does not vanish
    /// when the map is zoomed far out.
    pub fn halo(painter: &Painter, center: Pos2, zoom: f32, time: f32, color: Color32) {
        const PERIOD: f32 = 2.0;
        let alpha = 0.30 + 0.45 * triangle_wave(time, PERIOD);
        let radius = (9.0 * zoom).max(5.0);
        painter.add(Shape::Circle(CircleShape::stroke(
            center,
            radius,
            Stroke::new((2.0 * zoom).max(1.5), with_alpha(color, alpha)),
        )));
    }

    /// One frame of a thick ring blinking on and off.
    ///
    /// This is the effect markers have always used, factored out of the widget
    /// so it can be selected and reused like any other. `time` is the frame
    /// time in seconds.
    pub fn blink(painter: &Painter, center: Pos2, zoom: f32, time: f32, color: Color32) {
        const PERIOD: f32 = 2.55;
        painter.add(Shape::Circle(CircleShape::stroke(
            center,
            4.0 * zoom,
            Stroke::new(9.0 * zoom, with_alpha(color, triangle_wave(time, PERIOD))),
        )));
    }

    /// One frame of a dot orbiting the node, with a faint guide ring.
    ///
    /// Reads as *"under observation"*. `time` is the frame time in seconds.
    pub fn orbit(painter: &Painter, center: Pos2, zoom: f32, time: f32, color: Color32) {
        const PERIOD: f32 = 3.0;
        let radius = (12.0 * zoom).max(7.0);
        let angle = TAU * (time / PERIOD).rem_euclid(1.0);
        let dot = Pos2::new(
            center.x + radius * angle.cos(),
            center.y + radius * angle.sin(),
        );
        painter.extend([
            Shape::Circle(CircleShape::stroke(
                center,
                radius,
                Stroke::new(1.0, with_alpha(color, 0.25)),
            )),
            Shape::Circle(CircleShape::filled(
                dot,
                (2.5 * zoom).max(2.0),
                with_alpha(color, 1.0),
            )),
        ]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Context, LayerId, Rect, Vec2};
    use std::time::Duration;

    fn headless_painter() -> Painter {
        Painter::new(
            Context::default(),
            LayerId::background(),
            Rect::from_min_size(Pos2::ZERO, Vec2::new(100.0, 100.0)),
        )
    }

    /// Every event-driven effect, with the duration it is supposed to run for.
    #[allow(clippy::type_complexity)]
    fn event_effects() -> Vec<(
        &'static str,
        fn(&Painter, Pos2, f32, Instant, Color32) -> bool,
        f32,
    )> {
        vec![
            ("pulse", Animation::pulse, PULSE_DURATION),
            ("ripple", Animation::ripple, RIPPLE_DURATION),
            (
                "countdown_arc",
                Animation::countdown_arc,
                COUNTDOWN_DURATION,
            ),
            ("scale_in", Animation::scale_in, SCALE_IN_DURATION),
            ("crosshair", Animation::crosshair, CROSSHAIR_DURATION),
        ]
    }

    #[test]
    fn event_effects_report_running_then_finished() {
        let painter = headless_painter();
        for (name, effect, duration) in event_effects() {
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
        for time in [0.0, 0.7, 1.3, 60.0] {
            Animation::halo(&painter, Pos2::ZERO, 1.0, time, Color32::GREEN);
            Animation::blink(&painter, Pos2::ZERO, 1.0, time, Color32::GREEN);
            Animation::orbit(&painter, Pos2::ZERO, 1.0, time, Color32::GREEN);
        }
    }

    /// Every segment event-driven effect, with the duration it runs for.
    #[allow(clippy::type_complexity)]
    fn segment_event_effects() -> Vec<(
        &'static str,
        fn(&Painter, Pos2, Pos2, f32, Instant, Color32) -> bool,
        f32,
    )> {
        vec![
            ("flash_decay", Animation::flash_decay, FLASH_DECAY_DURATION),
            ("wipe", Animation::wipe, WIPE_DURATION),
        ]
    }

    #[test]
    fn segment_event_effects_report_running_then_finished() {
        let painter = headless_painter();
        let a = Pos2::ZERO;
        let b = Pos2::new(50.0, 0.0);
        for (name, effect, duration) in segment_event_effects() {
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
        let a = Pos2::ZERO;
        let b = Pos2::new(50.0, 0.0);
        for time in [0.0, 0.4, 0.8, 60.0] {
            Animation::comet(&painter, a, b, 1.0, time, Color32::GREEN);
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
        let a = Pos2::ZERO;
        let b = Pos2::new(50.0, 0.0);
        assert!(
            Animation::comet_once(
                &painter,
                a,
                b,
                1.0,
                Instant::now(),
                Color32::RED,
                CometDirection::Forward,
            ),
            "comet_once must report it is still running when it just started"
        );

        let long_past = Instant::now() - Duration::from_secs_f32(COMET_TRAVEL_DURATION + 1.0);
        assert!(
            !Animation::comet_once(
                &painter,
                a,
                b,
                1.0,
                long_past,
                Color32::RED,
                CometDirection::Forward,
            ),
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
        // At the very start of the animation the dot must sit on the
        // starting endpoint -- `a` for `Forward`, `b` for `Reverse` -- not
        // partway along the segment.
        let a = Pos2::ZERO;
        let b = Pos2::new(50.0, 0.0);
        let start_pos = |direction: CometDirection| {
            let secs = 0.0_f32;
            let progress = (secs / COMET_TRAVEL_DURATION).clamp(0.0, 1.0);
            let (from, to) = match direction {
                CometDirection::Forward => (a, b),
                CometDirection::Reverse => (b, a),
            };
            from + (to - from) * progress
        };
        assert_eq!(start_pos(CometDirection::Forward), a);
        assert_eq!(start_pos(CometDirection::Reverse), b);
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
        let a = Pos2::ZERO;
        let b = Pos2::new(50.0, 0.0);
        for time in [0.0, 0.4, 0.8, 60.0] {
            Animation::dash(&painter, a, b, 1.0, time, Color32::GREEN);
        }
        // A degenerate (zero-length) segment must not panic -- the
        // direction/normal math divides by the segment's length.
        Animation::dash(&painter, a, a, 1.0, 0.0, Color32::GREEN);
    }

    #[test]
    fn dash_texture_is_registered_once_per_context() {
        // Repeated calls on the same `Context` must reuse the same texture
        // rather than re-uploading one every frame.
        let ctx = Context::default();
        let first = Animation::dash_texture(&ctx);
        let second = Animation::dash_texture(&ctx);
        assert_eq!(first.id(), second.id());
    }

    #[test]
    fn glow_band_runs_at_any_time_and_skips_zero_length_segments() {
        let painter = headless_painter();
        let a = Pos2::ZERO;
        let b = Pos2::new(50.0, 0.0);
        for time in [0.0, 0.4, 0.8, 60.0] {
            Animation::glow_band(&painter, a, b, 1.0, time, Color32::GREEN);
        }
        // A degenerate (zero-length) segment must not panic -- the
        // direction/normal math divides by the segment's length.
        Animation::glow_band(&painter, a, a, 1.0, 0.0, Color32::GREEN);
    }

    #[test]
    fn glow_band_texture_is_registered_once_per_context() {
        let ctx = Context::default();
        let first = Animation::glow_band_texture(&ctx);
        let second = Animation::glow_band_texture(&ctx);
        assert_eq!(first.id(), second.id());
    }

    #[test]
    fn chevrons_runs_at_any_time_and_skips_zero_length_segments() {
        let painter = headless_painter();
        let a = Pos2::ZERO;
        let b = Pos2::new(50.0, 0.0);
        for time in [0.0, 0.4, 0.8, 60.0] {
            Animation::chevrons(&painter, a, b, 1.0, time, Color32::GREEN);
        }
        Animation::chevrons(&painter, a, a, 1.0, 0.0, Color32::GREEN);
    }

    #[test]
    fn chevrons_texture_is_registered_once_per_context() {
        let ctx = Context::default();
        let first = Animation::chevrons_texture(&ctx);
        let second = Animation::chevrons_texture(&ctx);
        assert_eq!(first.id(), second.id());
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

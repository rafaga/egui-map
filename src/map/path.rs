//! The geometry of a connection line: a straight segment or a curve.
//!
//! A [`SegmentPath`] is what the widget draws for one connection and what the
//! built-in segment effects ([`SegmentAnimations`](super::animation::SegmentAnimations))
//! travel along. It is either a straight line between the two endpoints or a
//! quadratic Bézier curve bent to one side of it, see
//! [`MapSegment::shape`](super::objects::MapSegment::shape) and
//! [`MapSettings::default_curvature`](super::objects::MapSettings::default_curvature).
//!
//! The effects that need to walk along the path at a constant speed (the
//! travelling dot, the wipe, the ribbons of the dash, the glow band and the
//! chevrons) work on a [`Polyline`], the path flattened into short straight
//! pieces with the distance travelled recorded at each of them. A straight
//! path flattens to exactly its two endpoints, so a straight line costs and
//! looks the same as it did before curves existed.

use egui::epaint::{PathShape, QuadraticBezierShape, Vertex};
use egui::{Color32, Mesh, Pos2, Rect, Shape, Stroke, TextureId, Vec2};

/// A connection line, in whatever space its points are in: map coordinates
/// for the R-tree and the hit-testing, screen pixels for painting.
///
/// Straight when it has no control point, otherwise a quadratic Bézier curve
/// from [`start`](Self::start) to [`end`](Self::end) pulled towards the control
/// point. Build one with [`SegmentPath::line`] or [`SegmentPath::curved`].
///
/// # Examples
///
/// ```
/// use egui::pos2;
/// use egui_map::map::path::SegmentPath;
///
/// let line = SegmentPath::line(pos2(0.0, 0.0), pos2(100.0, 0.0));
/// assert!(!line.is_curved());
///
/// // A curvature of 0.5 bows the line by a quarter of its length.
/// let curve = SegmentPath::curved(pos2(0.0, 0.0), pos2(100.0, 0.0), 0.5);
/// assert!(curve.is_curved());
/// assert_eq!(curve.point_at(0.5), pos2(50.0, 25.0));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SegmentPath {
    start: Pos2,
    end: Pos2,
    control: Option<Pos2>,
}

impl SegmentPath {
    /// The most a line can be curved: [`curved`](Self::curved) clamps the
    /// curvature to `-MAX_CURVATURE..=MAX_CURVATURE`. At the limit the curve
    /// bows by half of the distance between its endpoints.
    pub const MAX_CURVATURE: f32 = 1.0;

    /// The target length, in the path's own units, of each straight piece a
    /// curve is flattened into.
    const FLATTEN_STEP: f32 = 6.0;
    /// The fewest pieces a curve is flattened into.
    const FLATTEN_MIN_STEPS: usize = 8;
    /// The most pieces a curve is flattened into.
    const FLATTEN_MAX_STEPS: usize = 64;

    /// A straight path from `start` to `end`.
    pub fn line(start: Pos2, end: Pos2) -> Self {
        Self {
            start,
            end,
            control: None,
        }
    }

    /// A path from `start` to `end` bowed by `curvature`.
    ///
    /// `curvature` is the offset of the Bézier control point from the middle
    /// of the line, as a fraction of the distance between the endpoints, so
    /// the peak of the curve bows by half of it. `0.0` is a straight line,
    /// positive values bend the line to the right of the way from `start` to
    /// `end` as seen on the screen and negative values to the left. It is
    /// clamped to [`MAX_CURVATURE`](Self::MAX_CURVATURE); a value that is not a
    /// number, or endpoints that coincide, give a straight line.
    ///
    /// Because the offset is relative to the line, the curve of a connection
    /// keeps its shape whatever the zoom: the same path scaled up.
    pub fn curved(start: Pos2, end: Pos2, curvature: f32) -> Self {
        let chord = end - start;
        let length = chord.length();
        if !curvature.is_finite() || curvature == 0.0 || length <= f32::EPSILON {
            return Self::line(start, end);
        }
        let curvature = curvature.clamp(-Self::MAX_CURVATURE, Self::MAX_CURVATURE);
        let normal = Vec2::new(-chord.y, chord.x) / length;
        Self {
            start,
            end,
            control: Some(start + chord * 0.5 + normal * (curvature * length)),
        }
    }

    /// Where the path begins.
    pub fn start(&self) -> Pos2 {
        self.start
    }

    /// Where the path ends.
    pub fn end(&self) -> Pos2 {
        self.end
    }

    /// The control point of the curve, `None` for a straight line.
    pub fn control(&self) -> Option<Pos2> {
        self.control
    }

    /// Whether the path is a curve rather than a straight line.
    pub fn is_curved(&self) -> bool {
        self.control.is_some()
    }

    /// The same path walked the other way: the endpoints swap, and the curve
    /// keeps its shape.
    pub fn reversed(&self) -> Self {
        Self {
            start: self.end,
            end: self.start,
            control: self.control,
        }
    }

    /// The point at parameter `t` (`0.0` at the start, `1.0` at the end).
    ///
    /// `t` is the parameter of the curve, not the distance along it: on a
    /// curve, equal steps of `t` are not equal distances. Use a
    /// [`Polyline`] to move at a constant speed.
    pub fn point_at(&self, t: f32) -> Pos2 {
        match self.control {
            None => self.start + (self.end - self.start) * t,
            Some(control) => {
                let u = 1.0 - t;
                Pos2::new(
                    u * u * self.start.x + 2.0 * u * t * control.x + t * t * self.end.x,
                    u * u * self.start.y + 2.0 * u * t * control.y + t * t * self.end.y,
                )
            }
        }
    }

    /// A rectangle that contains the whole path.
    ///
    /// A Bézier curve never leaves the hull of its three points, so this is
    /// the rectangle around the endpoints and the control point: not the
    /// tightest one, but never too small.
    pub fn bounds(&self) -> Rect {
        let mut rect = Rect::from_two_pos(self.start, self.end);
        if let Some(control) = self.control {
            rect.extend_with(control);
        }
        rect
    }

    /// The shape that strokes the whole path with `stroke`: a line segment for
    /// a straight path and a quadratic Bézier curve for a curved one.
    pub fn stroke_shape(&self, stroke: Stroke) -> Shape {
        match self.control {
            None => Shape::line_segment([self.start, self.end], stroke),
            Some(control) => Shape::QuadraticBezier(QuadraticBezierShape::from_points_stroke(
                [self.start, control, self.end],
                false,
                Color32::TRANSPARENT,
                stroke,
            )),
        }
    }

    /// The path flattened into short straight pieces. A straight path is just
    /// its two endpoints.
    pub fn polyline(&self) -> Polyline {
        let Some(control) = self.control else {
            return Polyline::from_points(vec![self.start, self.end]);
        };
        // The control polygon is never shorter than the curve, so it is a
        // cheap upper bound to size the number of pieces from.
        let reach = (control - self.start).length() + (self.end - control).length();
        let steps = ((reach / Self::FLATTEN_STEP).ceil() as usize)
            .clamp(Self::FLATTEN_MIN_STEPS, Self::FLATTEN_MAX_STEPS);
        let points = (0..=steps)
            .map(|i| self.point_at(i as f32 / steps as f32))
            .collect();
        Polyline::from_points(points)
    }

    /// The distance from `point` to the closest point of the path.
    ///
    /// Exact for a straight path; for a curve it is the distance to the
    /// flattened curve, which is as close as the pieces it is flattened into.
    pub fn distance_to_point(&self, point: Pos2) -> f32 {
        if self.is_curved() {
            self.polyline().distance_to_point(point)
        } else {
            Polyline::distance_to_piece(point, self.start, self.end)
        }
    }
}

/// A path flattened into straight pieces, with the distance travelled from
/// the start recorded at every point.
///
/// That lets an effect place things by distance along the path instead of by
/// the parameter of the curve: a dot moves at a constant speed, a wipe reveals
/// a fixed fraction of the length, and a ribbon gets its texture coordinate
/// from the distance. Built by [`SegmentPath::polyline`].
#[derive(Clone, Debug, PartialEq)]
pub struct Polyline {
    points: Vec<Pos2>,
    cumulative: Vec<f32>,
}

impl Polyline {
    /// The polyline through `points`, in order.
    fn from_points(points: Vec<Pos2>) -> Self {
        let mut cumulative = Vec::with_capacity(points.len());
        let mut travelled = 0.0;
        for (i, point) in points.iter().enumerate() {
            if i > 0 {
                travelled += (*point - points[i - 1]).length();
            }
            cumulative.push(travelled);
        }
        Self { points, cumulative }
    }

    /// The points the path was flattened to, from start to end.
    pub fn points(&self) -> &[Pos2] {
        &self.points
    }

    /// The length of the whole path.
    pub fn length(&self) -> f32 {
        self.cumulative.last().copied().unwrap_or(0.0)
    }

    /// The point `distance` along the path, clamped to its two ends.
    pub fn point_at_distance(&self, distance: f32) -> Pos2 {
        let length = self.length();
        if length <= f32::EPSILON {
            return self.points[0];
        }
        let distance = distance.clamp(0.0, length);
        // The first point at or past `distance`: the piece that holds it ends
        // there. At least the second point, so that a `distance` of `0.0` is
        // still inside the first piece.
        let end = self
            .cumulative
            .partition_point(|travelled| *travelled < distance)
            .clamp(1, self.points.len() - 1);
        let (from, to) = (self.cumulative[end - 1], self.cumulative[end]);
        let piece = to - from;
        if piece <= f32::EPSILON {
            return self.points[end];
        }
        let t = (distance - from) / piece;
        self.points[end - 1] + (self.points[end] - self.points[end - 1]) * t
    }

    /// The part of the path between the distances `from` and `to`, as points
    /// in order. Both are clamped to the path, and `to` to at least `from`.
    pub fn slice(&self, from: f32, to: f32) -> Vec<Pos2> {
        let length = self.length();
        let from = from.clamp(0.0, length);
        let to = to.clamp(from, length);
        let mut points = vec![self.point_at_distance(from)];
        for (point, travelled) in self.points.iter().zip(&self.cumulative) {
            if *travelled > from && *travelled < to {
                points.push(*point);
            }
        }
        points.push(self.point_at_distance(to));
        points
    }

    /// The shape that strokes the part of the path between the distances
    /// `from` and `to` with `stroke`: a line segment when that part is one
    /// straight piece, a path otherwise.
    pub fn slice_shape(&self, from: f32, to: f32, stroke: Stroke) -> Shape {
        let points = self.slice(from, to);
        match points.as_slice() {
            [a, b] => Shape::line_segment([*a, *b], stroke),
            _ => Shape::Path(PathShape::line(points, stroke)),
        }
    }

    /// The distance from `point` to the closest point of the polyline.
    pub fn distance_to_point(&self, point: Pos2) -> f32 {
        self.points
            .windows(2)
            .map(|piece| Self::distance_to_piece(point, piece[0], piece[1]))
            .fold(f32::INFINITY, f32::min)
    }

    /// The distance from `point` to the straight piece from `a` to `b`: to its
    /// closest point, which is an endpoint when the perpendicular from `point`
    /// falls outside it.
    fn distance_to_piece(point: Pos2, a: Pos2, b: Pos2) -> f32 {
        let ab = b - a;
        let ap = point - a;
        let length_sq = ab.length_sq();
        if length_sq == 0.0 {
            return ap.length();
        }
        let t = (ap.dot(ab) / length_sq).clamp(0.0, 1.0);
        (point - (a + ab * t)).length()
    }

    /// The direction the path is heading at its `index`-th point: along the
    /// pieces on both sides of it, so a ribbon bends smoothly through the
    /// joints instead of kinking.
    fn tangent_at(&self, index: usize) -> Vec2 {
        let last = self.points.len() - 1;
        let from = self.points[index.saturating_sub(1)];
        let to = self.points[(index + 1).min(last)];
        let tangent = to - from;
        let length = tangent.length();
        if length <= f32::EPSILON {
            Vec2::X
        } else {
            tangent / length
        }
    }

    /// A ribbon `2 * half_width` wide along the whole path, as a textured mesh.
    ///
    /// `uv` is asked for the texture coordinate of each vertex, with the
    /// distance travelled along the path and `0.0` for the vertex on the
    /// left-hand edge of the ribbon or `1.0` for the one on the right-hand
    /// edge. Two vertices per point of the polyline, so a straight path is
    /// the two triangles of a plain quad.
    pub fn ribbon(
        &self,
        texture_id: TextureId,
        half_width: f32,
        color: Color32,
        uv: impl Fn(f32, f32) -> Pos2,
    ) -> Mesh {
        let mut mesh = Mesh::with_texture(texture_id);
        for (index, point) in self.points.iter().enumerate() {
            let tangent = self.tangent_at(index);
            let normal = Vec2::new(-tangent.y, tangent.x) * half_width;
            let distance = self.cumulative[index];
            mesh.vertices.push(Vertex {
                pos: *point + normal,
                uv: uv(distance, 0.0),
                color,
            });
            mesh.vertices.push(Vertex {
                pos: *point - normal,
                uv: uv(distance, 1.0),
                color,
            });
            if index > 0 {
                let base = 2 * (index as u32 - 1);
                mesh.indices
                    .extend([base, base + 1, base + 2, base + 2, base + 1, base + 3]);
            }
        }
        mesh
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;

    fn assert_close(a: Pos2, b: Pos2) {
        assert!((a - b).length() < 1e-3, "{a:?} is not {b:?}");
    }

    #[test]
    fn a_line_has_no_control_point_and_flattens_to_its_endpoints() {
        let path = SegmentPath::line(pos2(0.0, 0.0), pos2(30.0, 40.0));
        assert!(!path.is_curved());
        assert_eq!(path.control(), None);
        let polyline = path.polyline();
        assert_eq!(polyline.points(), &[pos2(0.0, 0.0), pos2(30.0, 40.0)]);
        assert_eq!(polyline.length(), 50.0);
    }

    #[test]
    fn zero_curvature_is_a_line() {
        let path = SegmentPath::curved(pos2(0.0, 0.0), pos2(10.0, 0.0), 0.0);
        assert!(!path.is_curved());
    }

    #[test]
    fn a_curve_without_a_length_or_with_nan_is_a_line() {
        assert!(!SegmentPath::curved(pos2(3.0, 3.0), pos2(3.0, 3.0), 0.5).is_curved());
        assert!(!SegmentPath::curved(pos2(0.0, 0.0), pos2(1.0, 0.0), f32::NAN).is_curved());
        assert!(!SegmentPath::curved(pos2(0.0, 0.0), pos2(1.0, 0.0), f32::INFINITY).is_curved());
    }

    #[test]
    fn curvature_is_the_control_offset_over_the_length() {
        // Right of an eastward line, as seen on screen, is down: +y.
        let path = SegmentPath::curved(pos2(0.0, 0.0), pos2(100.0, 0.0), 0.5);
        assert_close(path.control().unwrap(), pos2(50.0, 50.0));
        // The peak of the curve bows by half the control offset.
        assert_close(path.point_at(0.5), pos2(50.0, 25.0));
        let left = SegmentPath::curved(pos2(0.0, 0.0), pos2(100.0, 0.0), -0.5);
        assert_close(left.point_at(0.5), pos2(50.0, -25.0));
    }

    #[test]
    fn curvature_is_clamped() {
        let huge = SegmentPath::curved(pos2(0.0, 0.0), pos2(100.0, 0.0), 40.0);
        let max = SegmentPath::curved(pos2(0.0, 0.0), pos2(100.0, 0.0), SegmentPath::MAX_CURVATURE);
        assert_eq!(huge, max);
        let tiny = SegmentPath::curved(pos2(0.0, 0.0), pos2(100.0, 0.0), -40.0);
        assert_close(tiny.control().unwrap(), pos2(50.0, -100.0));
    }

    #[test]
    fn a_curve_passes_through_both_endpoints() {
        let path = SegmentPath::curved(pos2(5.0, 7.0), pos2(80.0, -20.0), 0.3);
        assert_close(path.point_at(0.0), pos2(5.0, 7.0));
        assert_close(path.point_at(1.0), pos2(80.0, -20.0));
        let polyline = path.polyline();
        assert_close(polyline.points()[0], pos2(5.0, 7.0));
        assert_close(*polyline.points().last().unwrap(), pos2(80.0, -20.0));
    }

    #[test]
    fn a_curve_is_longer_than_its_chord_and_flattens_into_pieces() {
        let path = SegmentPath::curved(pos2(0.0, 0.0), pos2(100.0, 0.0), 0.5);
        let polyline = path.polyline();
        assert!(polyline.points().len() > SegmentPath::FLATTEN_MIN_STEPS);
        assert!(polyline.points().len() <= SegmentPath::FLATTEN_MAX_STEPS + 1);
        assert!(polyline.length() > 100.0);
        // ...and shorter than the control polygon that contains it.
        assert!(polyline.length() < 2.0 * (50.0f32 * 50.0 + 50.0 * 50.0).sqrt());
    }

    #[test]
    fn reversing_a_path_swaps_the_ends_and_keeps_the_curve() {
        let path = SegmentPath::curved(pos2(0.0, 0.0), pos2(100.0, 0.0), 0.4);
        let back = path.reversed();
        assert_eq!(back.start(), path.end());
        assert_eq!(back.end(), path.start());
        assert_eq!(back.control(), path.control());
        assert_close(back.point_at(0.25), path.point_at(0.75));
        assert_eq!(back.reversed(), path);
    }

    #[test]
    fn bounds_hold_the_endpoints_and_the_control_point() {
        let line = SegmentPath::line(pos2(10.0, 20.0), pos2(-5.0, 4.0));
        assert_eq!(
            line.bounds(),
            Rect::from_two_pos(pos2(-5.0, 4.0), pos2(10.0, 20.0))
        );
        let curve = SegmentPath::curved(pos2(0.0, 0.0), pos2(100.0, 0.0), 0.5);
        let bounds = curve.bounds();
        assert!(bounds.contains(pos2(0.0, 0.0)));
        assert!(bounds.contains(pos2(100.0, 0.0)));
        assert!(bounds.contains(curve.control().unwrap()));
        // Every point of the curve is inside.
        for i in 0..=20 {
            assert!(bounds.contains(curve.point_at(i as f32 / 20.0)));
        }
    }

    #[test]
    fn stroke_shape_is_a_segment_or_a_bezier() {
        let stroke = Stroke::new(2.0, Color32::WHITE);
        let line = SegmentPath::line(pos2(0.0, 0.0), pos2(10.0, 0.0));
        assert!(matches!(
            line.stroke_shape(stroke),
            Shape::LineSegment { .. }
        ));
        let curve = SegmentPath::curved(pos2(0.0, 0.0), pos2(10.0, 0.0), 0.5);
        assert!(matches!(
            curve.stroke_shape(stroke),
            Shape::QuadraticBezier(_)
        ));
    }

    #[test]
    fn point_at_distance_walks_the_path_at_a_constant_speed() {
        let polyline = SegmentPath::line(pos2(0.0, 0.0), pos2(100.0, 0.0)).polyline();
        assert_close(polyline.point_at_distance(0.0), pos2(0.0, 0.0));
        assert_close(polyline.point_at_distance(25.0), pos2(25.0, 0.0));
        assert_close(polyline.point_at_distance(100.0), pos2(100.0, 0.0));
        // Past the ends it is clamped.
        assert_close(polyline.point_at_distance(-5.0), pos2(0.0, 0.0));
        assert_close(polyline.point_at_distance(500.0), pos2(100.0, 0.0));

        // On a curve, the distance covered between two samples is the step.
        let polyline = SegmentPath::curved(pos2(0.0, 0.0), pos2(100.0, 0.0), 0.8).polyline();
        let step = polyline.length() / 40.0;
        for i in 0..40 {
            let a = polyline.point_at_distance(i as f32 * step);
            let b = polyline.point_at_distance((i + 1) as f32 * step);
            assert!(((b - a).length() - step).abs() < step * 0.1, "step {i}");
        }
    }

    #[test]
    fn point_at_distance_of_a_zero_length_path_is_its_point() {
        let polyline = SegmentPath::line(pos2(4.0, 4.0), pos2(4.0, 4.0)).polyline();
        assert_eq!(polyline.length(), 0.0);
        assert_eq!(polyline.point_at_distance(3.0), pos2(4.0, 4.0));
    }

    #[test]
    fn slice_keeps_the_part_between_two_distances() {
        let polyline = SegmentPath::line(pos2(0.0, 0.0), pos2(100.0, 0.0)).polyline();
        assert_eq!(
            polyline.slice(20.0, 70.0),
            vec![pos2(20.0, 0.0), pos2(70.0, 0.0)]
        );
        // `to` is raised to `from`, both are clamped.
        assert_eq!(
            polyline.slice(70.0, 20.0),
            vec![pos2(70.0, 0.0), pos2(70.0, 0.0)]
        );
        assert_eq!(
            polyline.slice(-10.0, 400.0),
            vec![pos2(0.0, 0.0), pos2(100.0, 0.0)]
        );

        // On a curve the slice has the joints in between, in order.
        let polyline = SegmentPath::curved(pos2(0.0, 0.0), pos2(100.0, 0.0), 0.5).polyline();
        let length = polyline.length();
        let slice = polyline.slice(length * 0.25, length * 0.75);
        assert!(slice.len() > 2);
        assert_close(slice[0], polyline.point_at_distance(length * 0.25));
        assert_close(
            *slice.last().unwrap(),
            polyline.point_at_distance(length * 0.75),
        );
    }

    #[test]
    fn slice_shape_is_a_segment_for_a_straight_piece_and_a_path_otherwise() {
        let stroke = Stroke::new(1.0, Color32::WHITE);
        let line = SegmentPath::line(pos2(0.0, 0.0), pos2(100.0, 0.0)).polyline();
        assert!(matches!(
            line.slice_shape(10.0, 60.0, stroke),
            Shape::LineSegment { .. }
        ));
        let curve = SegmentPath::curved(pos2(0.0, 0.0), pos2(100.0, 0.0), 0.5).polyline();
        assert!(matches!(
            curve.slice_shape(10.0, 60.0, stroke),
            Shape::Path(_)
        ));
    }

    #[test]
    fn distance_to_point_on_a_line_matches_the_perpendicular() {
        let line = SegmentPath::line(pos2(0.0, 0.0), pos2(10.0, 0.0));
        assert_eq!(line.distance_to_point(pos2(5.0, 3.0)), 3.0);
        // Beyond the end of the segment, the endpoint is the closest point.
        assert_eq!(line.distance_to_point(pos2(14.0, 0.0)), 4.0);
        // Degenerate: both endpoints coincide.
        let dot = SegmentPath::line(pos2(1.0, 1.0), pos2(1.0, 1.0));
        assert_eq!(dot.distance_to_point(pos2(4.0, 5.0)), 5.0);
    }

    #[test]
    fn distance_to_point_on_a_curve_follows_the_curve_not_the_chord() {
        let curve = SegmentPath::curved(pos2(0.0, 0.0), pos2(100.0, 0.0), 0.5);
        // The peak is at (50, 25): on the curve, but 25 away from the chord.
        assert!(curve.distance_to_point(pos2(50.0, 25.0)) < 0.5);
        let chord = SegmentPath::line(pos2(0.0, 0.0), pos2(100.0, 0.0));
        assert!(chord.distance_to_point(pos2(50.0, 25.0)) > 24.0);
        // And the chord itself, mid-way, is far from the curve.
        assert!(curve.distance_to_point(pos2(50.0, 0.0)) > 20.0);
    }

    #[test]
    fn a_ribbon_along_a_line_is_a_plain_quad() {
        let polyline = SegmentPath::line(pos2(0.0, 0.0), pos2(10.0, 0.0)).polyline();
        let mesh = polyline.ribbon(TextureId::default(), 2.0, Color32::WHITE, pos2);
        assert_eq!(mesh.indices, vec![0, 1, 2, 2, 1, 3]);
        let positions: Vec<Pos2> = mesh.vertices.iter().map(|v| v.pos).collect();
        // Left edge `a + normal`, right edge `a - normal`, as the dash always had.
        assert_eq!(
            positions,
            vec![
                pos2(0.0, 2.0),
                pos2(0.0, -2.0),
                pos2(10.0, 2.0),
                pos2(10.0, -2.0)
            ]
        );
        let uvs: Vec<Pos2> = mesh.vertices.iter().map(|v| v.uv).collect();
        assert_eq!(
            uvs,
            vec![
                pos2(0.0, 0.0),
                pos2(0.0, 1.0),
                pos2(10.0, 0.0),
                pos2(10.0, 1.0)
            ]
        );
    }

    #[test]
    fn a_ribbon_along_a_curve_has_two_vertices_per_point_and_stays_valid() {
        let polyline = SegmentPath::curved(pos2(0.0, 0.0), pos2(100.0, 0.0), 0.5).polyline();
        let points = polyline.points().len();
        let mesh = polyline.ribbon(TextureId::default(), 1.5, Color32::WHITE, pos2);
        assert_eq!(mesh.vertices.len(), 2 * points);
        assert_eq!(mesh.indices.len(), 6 * (points - 1));
        assert!(
            mesh.indices
                .iter()
                .all(|i| (*i as usize) < mesh.vertices.len())
        );
        // Each pair of vertices is the ribbon's width apart, across the path.
        for pair in mesh.vertices.chunks(2) {
            assert!(((pair[0].pos - pair[1].pos).length() - 3.0).abs() < 1e-3);
        }
        // The `u` the callback gets is the distance along the path.
        assert_eq!(mesh.vertices[0].uv.x, 0.0);
        assert!((mesh.vertices.last().unwrap().uv.x - polyline.length()).abs() < 1e-3);
    }
}

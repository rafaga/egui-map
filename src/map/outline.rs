//! The shape of a node, as a [`NodeTemplate`](super::objects::NodeTemplate)
//! draws it: [`NodeOutline`].
//!
//! A template describes its node's shape once, in
//! [`NodeTemplate::outline`](super::objects::NodeTemplate::outline), and the
//! widget derives the rest from it: the hit area behind
//! [`Map::hovered_node`](super::Map::hovered_node), the selection ring, and
//! the built-in effects ([`Animation`](super::animation::Animation)'s
//! `*_outline` functions), which follow the node's own shape instead of the
//! circle the default nodes are drawn with.
//!
//! Strokes built here ([`NodeOutline::stroke_shape`]) are drawn *outside* the
//! outline with a constant width, and radii that don't fit egui's
//! [`CornerRadius`] (a `u8`) fall back to a polygon, so an effect that grows
//! the outline keeps its shape at any size.

use egui::epaint::{CircleShape, PathShape, RectShape};
use egui::{Color32, CornerRadius, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

/// Points per quarter turn when a rounded corner or a circle is turned into a
/// polygon (perimeter walks, radii too large for [`CornerRadius`]).
const ARC_STEPS: usize = 12;

/// The shape of a node in screen coordinates.
///
/// `#[non_exhaustive]` so another shape can be added later without a breaking
/// change.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum NodeOutline {
    /// A circle, like the default nodes.
    Circle {
        /// Center, in screen coordinates.
        center: Pos2,
        /// Radius, in screen points.
        radius: f32,
    },
    /// A rectangle with the same radius on its four corners.
    RoundedRect {
        /// The rectangle, in screen coordinates.
        rect: Rect,
        /// Corner radius, in screen points.
        corner_radius: f32,
    },
    /// A **convex** polygon, vertices in order (either direction), in screen
    /// coordinates. For a concave shape, give its convex hull.
    Polygon(Vec<Pos2>),
}

impl NodeOutline {
    /// The center of the shape (for a polygon, the average of its vertices).
    pub fn center(&self) -> Pos2 {
        match self {
            Self::Circle { center, .. } => *center,
            Self::RoundedRect { rect, .. } => rect.center(),
            Self::Polygon(points) => centroid(points),
        }
    }

    /// The smallest rectangle that contains the shape.
    pub fn bounding_rect(&self) -> Rect {
        match self {
            Self::Circle { center, radius } => {
                Rect::from_center_size(*center, Vec2::splat(2.0 * radius.max(0.0)))
            }
            Self::RoundedRect { rect, .. } => *rect,
            Self::Polygon(points) => Rect::from_points(points),
        }
    }

    /// The same shape, `distance` screen points further out (or in, when
    /// negative): a circle's radius, or a rectangle and its corner radius,
    /// grow by `distance`, so the result stays concentric; a polygon's sides
    /// move out by `distance`.
    pub fn grown(&self, distance: f32) -> Self {
        match self {
            Self::Circle { center, radius } => Self::Circle {
                center: *center,
                radius: (radius + distance).max(0.0),
            },
            Self::RoundedRect {
                rect,
                corner_radius,
            } => {
                let half = rect.size() / 2.0;
                let grown = (half + Vec2::splat(distance)).max(Vec2::ZERO);
                Self::RoundedRect {
                    rect: Rect::from_center_size(rect.center(), grown * 2.0),
                    corner_radius: (corner_radius + distance).clamp(0.0, grown.min_elem()),
                }
            }
            Self::Polygon(points) => Self::Polygon(offset_convex(points, distance)),
        }
    }

    /// The same shape scaled by `factor` around its [`center`](Self::center).
    pub fn scaled(&self, factor: f32) -> Self {
        let factor = factor.max(0.0);
        match self {
            Self::Circle { center, radius } => Self::Circle {
                center: *center,
                radius: radius * factor,
            },
            Self::RoundedRect {
                rect,
                corner_radius,
            } => Self::RoundedRect {
                rect: Rect::from_center_size(rect.center(), rect.size() * factor),
                corner_radius: corner_radius * factor,
            },
            Self::Polygon(points) => {
                let center = centroid(points);
                Self::Polygon(
                    points
                        .iter()
                        .map(|point| center + (*point - center) * factor)
                        .collect(),
                )
            }
        }
    }

    /// Whether `point` is inside the shape (its edge included).
    pub fn contains(&self, point: Pos2) -> bool {
        match self {
            Self::Circle { center, radius } => center.distance(point) <= *radius,
            Self::RoundedRect {
                rect,
                corner_radius,
            } => {
                let half = rect.size() / 2.0;
                let radius = corner_radius.clamp(0.0, half.min_elem());
                let offset = (point - rect.center()).abs() - (half - Vec2::splat(radius));
                offset.max(Vec2::ZERO).length() <= radius && offset.max_elem() <= radius
            }
            Self::Polygon(points) => convex_contains(points, point),
        }
    }

    /// The farthest point of the shape from `origin`, in screen points: how
    /// far a hit test around `origin` must look to reach all of it.
    pub fn extent_from(&self, origin: Pos2) -> f32 {
        match self {
            Self::Circle { center, radius } => origin.distance(*center) + radius,
            Self::RoundedRect {
                rect,
                corner_radius,
            } => {
                let radius = corner_radius.clamp(0.0, rect.size().min_elem() / 2.0);
                let inner = rect.shrink(radius);
                [
                    inner.left_top(),
                    inner.right_top(),
                    inner.left_bottom(),
                    inner.right_bottom(),
                ]
                .iter()
                .map(|corner| origin.distance(*corner) + radius)
                .fold(0.0, f32::max)
            }
            Self::Polygon(points) => points
                .iter()
                .map(|point| origin.distance(*point))
                .fold(0.0, f32::max),
        }
    }

    /// The outline as a closed polygon, clockwise on screen, starting at the
    /// top center (for a polygon: at the vertex nearest it). Used to walk
    /// along the edge, e.g. a countdown or a dot orbiting the node.
    pub fn perimeter(&self) -> Vec<Pos2> {
        match self {
            Self::Circle { center, radius } => {
                let steps = 4 * ARC_STEPS;
                (0..steps)
                    .map(|i| {
                        let angle = -FRAC_PI_2 + TAU * i as f32 / steps as f32;
                        *center + Vec2::angled(angle) * *radius
                    })
                    .collect()
            }
            Self::RoundedRect {
                rect,
                corner_radius,
            } => rounded_rect_perimeter(*rect, *corner_radius),
            Self::Polygon(points) => {
                let mut points = clockwise(points);
                let top = self.bounding_rect().center_top();
                if let Some(start) = points
                    .iter()
                    .enumerate()
                    .min_by(|a, b| a.1.distance(top).total_cmp(&b.1.distance(top)))
                    .map(|(index, _)| index)
                {
                    points.rotate_left(start);
                }
                points
            }
        }
    }

    /// The shape filled with `color`.
    pub fn fill_shape(&self, color: Color32) -> Shape {
        match self {
            Self::Circle { center, radius } => {
                Shape::Circle(CircleShape::filled(*center, *radius, color))
            }
            Self::RoundedRect {
                rect,
                corner_radius,
            } => match corner_radius_u8(*corner_radius) {
                Some(radius) => Shape::Rect(RectShape::filled(*rect, radius, color)),
                None => Shape::Path(PathShape::convex_polygon(
                    self.perimeter(),
                    color,
                    Stroke::NONE,
                )),
            },
            Self::Polygon(points) => Shape::Path(PathShape::convex_polygon(
                points.clone(),
                color,
                Stroke::NONE,
            )),
        }
    }

    /// A line of `stroke` along the shape, drawn just **outside** it so the
    /// shape itself stays uncovered. Keep `stroke.width` modest: to make a
    /// ring expand, [`grow`](Self::grown) the outline rather than widening
    /// the stroke, so it keeps its shape.
    pub fn stroke_shape(&self, stroke: Stroke) -> Shape {
        match self {
            Self::Circle { center, radius } => Shape::Circle(CircleShape::stroke(
                *center,
                radius + stroke.width / 2.0,
                stroke,
            )),
            Self::RoundedRect {
                rect,
                corner_radius,
            } => match corner_radius_u8(corner_radius + stroke.width) {
                Some(_) => Shape::Rect(RectShape::stroke(
                    *rect,
                    corner_radius_u8(*corner_radius).unwrap_or(CornerRadius::ZERO),
                    stroke,
                    StrokeKind::Outside,
                )),
                None => Shape::Path(PathShape::closed_line(
                    self.grown(stroke.width / 2.0).perimeter(),
                    stroke,
                )),
            },
            Self::Polygon(_) => Shape::Path(PathShape::closed_line(
                self.grown(stroke.width / 2.0).perimeter(),
                stroke,
            )),
        }
    }
}

/// The first `fraction` (`0.0..=1.0`) of a closed polyline, measured along
/// its length, as an open polyline -- e.g. what is left of a countdown.
pub fn partial_perimeter(points: &[Pos2], fraction: f32) -> Vec<Pos2> {
    let fraction = fraction.clamp(0.0, 1.0);
    if points.len() < 2 || fraction <= 0.0 {
        return Vec::new();
    }
    let closed: Vec<Pos2> = points.iter().chain(points.first()).copied().collect();
    let total: f32 = closed
        .windows(2)
        .map(|pair| pair[0].distance(pair[1]))
        .sum();
    let mut left = total * fraction;
    let mut out = vec![closed[0]];
    for pair in closed.windows(2) {
        let length = pair[0].distance(pair[1]);
        if left >= length {
            out.push(pair[1]);
            left -= length;
        } else {
            if length > 0.0 && left > 0.0 {
                out.push(pair[0] + (pair[1] - pair[0]) * (left / length));
            }
            break;
        }
    }
    out
}

/// The point `fraction` (wrapping) of the way along a closed polyline.
pub fn point_along(points: &[Pos2], fraction: f32) -> Option<Pos2> {
    let fraction = fraction.rem_euclid(1.0);
    let partial = partial_perimeter(points, fraction.max(f32::EPSILON));
    partial.last().copied()
}

fn corner_radius_u8(radius: f32) -> Option<CornerRadius> {
    (0.0..=f32::from(u8::MAX))
        .contains(&radius)
        .then(|| CornerRadius::same(radius.round() as u8))
}

fn centroid(points: &[Pos2]) -> Pos2 {
    if points.is_empty() {
        return Pos2::ZERO;
    }
    let sum = points
        .iter()
        .fold(Vec2::ZERO, |sum, point| sum + point.to_vec2());
    (sum / points.len() as f32).to_pos2()
}

/// Twice the signed area: positive when the points turn clockwise on screen
/// (y grows downwards).
fn signed_area(points: &[Pos2]) -> f32 {
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .map(|(a, b)| a.x * b.y - b.x * a.y)
        .sum()
}

fn clockwise(points: &[Pos2]) -> Vec<Pos2> {
    let mut points = points.to_vec();
    if signed_area(&points) < 0.0 {
        points.reverse();
    }
    points
}

fn convex_contains(points: &[Pos2], point: Pos2) -> bool {
    if points.len() < 3 {
        return false;
    }
    let points = clockwise(points);
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .all(|(a, b)| (*b - *a).x * (point - *a).y - (*b - *a).y * (point - *a).x >= 0.0)
}

/// Moves every side of a convex polygon `distance` outwards (mitred
/// corners, the miter capped so a very sharp corner doesn't spike).
fn offset_convex(points: &[Pos2], distance: f32) -> Vec<Pos2> {
    if points.len() < 3 || distance == 0.0 {
        return points.to_vec();
    }
    let points = clockwise(points);
    let count = points.len();
    // Outward normal of the side from `a` to `b`, for a clockwise polygon.
    let normal = |a: Pos2, b: Pos2| {
        let side = (b - a).normalized();
        Vec2::new(side.y, -side.x)
    };
    (0..count)
        .map(|i| {
            let prev = points[(i + count - 1) % count];
            let here = points[i];
            let next = points[(i + 1) % count];
            let bisector = normal(prev, here) + normal(here, next);
            let length_sq = bisector.length_sq();
            if length_sq < 1e-6 {
                return here + normal(here, next) * distance;
            }
            // `bisector * 2 / |bisector|^2` has the miter length for a unit
            // offset; capped at 4x.
            let miter = bisector * (2.0 / length_sq);
            let miter = if miter.length() > 4.0 {
                miter.normalized() * 4.0
            } else {
                miter
            };
            here + miter * distance
        })
        .collect()
}

fn rounded_rect_perimeter(rect: Rect, corner_radius: f32) -> Vec<Pos2> {
    let radius = corner_radius.clamp(0.0, rect.size().min_elem() / 2.0);
    let inner = rect.shrink(radius);
    let mut points = vec![rect.center_top()];
    // Corners clockwise from the top right: arc center and starting angle.
    let corners = [
        (inner.right_top(), -FRAC_PI_2),
        (inner.right_bottom(), 0.0),
        (inner.left_bottom(), FRAC_PI_2),
        (inner.left_top(), PI),
    ];
    for (center, start) in corners {
        if radius <= 0.0 {
            points.push(center);
            continue;
        }
        for step in 0..=ARC_STEPS {
            let angle = start + FRAC_PI_2 * step as f32 / ARC_STEPS as f32;
            points.push(center + Vec2::angled(angle) * radius);
        }
    }
    points
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rounded() -> NodeOutline {
        NodeOutline::RoundedRect {
            rect: Rect::from_center_size(Pos2::new(100.0, 100.0), Vec2::new(90.0, 35.0)),
            corner_radius: 10.0,
        }
    }

    fn diamond() -> NodeOutline {
        NodeOutline::Polygon(vec![
            Pos2::new(0.0, -10.0),
            Pos2::new(10.0, 0.0),
            Pos2::new(0.0, 10.0),
            Pos2::new(-10.0, 0.0),
        ])
    }

    #[test]
    fn growing_keeps_the_shape_concentric() {
        let NodeOutline::RoundedRect {
            rect,
            corner_radius,
        } = rounded().grown(5.0)
        else {
            panic!("a rounded rect grows into a rounded rect");
        };
        assert_eq!(rect.size(), Vec2::new(100.0, 45.0));
        assert_eq!(rect.center(), Pos2::new(100.0, 100.0));
        assert_eq!(corner_radius, 15.0);

        let NodeOutline::Circle { radius, .. } = (NodeOutline::Circle {
            center: Pos2::ZERO,
            radius: 4.0,
        })
        .grown(-10.0) else {
            panic!("a circle grows into a circle");
        };
        assert_eq!(radius, 0.0);
    }

    #[test]
    fn contains_follows_the_shape() {
        let outline = rounded();
        assert!(outline.contains(Pos2::new(100.0, 100.0)));
        assert!(outline.contains(Pos2::new(140.0, 100.0)));
        // Just outside the rounded top-right corner, inside its bounding box.
        assert!(!outline.contains(Pos2::new(144.5, 82.8)));
        assert!(!outline.contains(Pos2::new(146.0, 100.0)));

        let diamond = diamond();
        assert!(diamond.contains(Pos2::new(0.0, 0.0)));
        assert!(!diamond.contains(Pos2::new(8.0, 8.0)));
        assert!(diamond.grown(4.0).contains(Pos2::new(7.0, 7.0)));
    }

    #[test]
    fn the_extent_reaches_the_whole_shape() {
        let outline = rounded();
        let center = outline.center();
        let extent = outline.extent_from(center);
        for point in outline.perimeter() {
            assert!(center.distance(point) <= extent + 1e-3);
        }
    }

    #[test]
    fn the_perimeter_starts_at_the_top_and_turns_clockwise() {
        for outline in [rounded(), diamond()] {
            let points = outline.perimeter();
            let top = outline.bounding_rect().center_top();
            assert!(points[0].distance(top) < 1e-3);
            assert!(signed_area(&points) > 0.0);
        }
    }

    #[test]
    fn partial_perimeter_measures_along_the_edge() {
        let square = [
            Pos2::new(0.0, 0.0),
            Pos2::new(10.0, 0.0),
            Pos2::new(10.0, 10.0),
            Pos2::new(0.0, 10.0),
        ];
        assert!(partial_perimeter(&square, 0.0).is_empty());
        let half = partial_perimeter(&square, 0.5);
        assert_eq!(half.last().copied(), Some(Pos2::new(10.0, 10.0)));
        let eighth = point_along(&square, 0.125).unwrap();
        assert!((eighth.x - 5.0).abs() < 1e-3 && eighth.y.abs() < 1e-3);
    }

    #[test]
    fn huge_radii_are_drawn_as_polygons() {
        let big = NodeOutline::RoundedRect {
            rect: Rect::from_center_size(Pos2::ZERO, Vec2::splat(1000.0)),
            corner_radius: 400.0,
        };
        assert!(matches!(big.fill_shape(Color32::RED), Shape::Path(_)));
        assert!(matches!(
            big.stroke_shape(Stroke::new(2.0, Color32::RED)),
            Shape::Path(_)
        ));
        assert!(matches!(rounded().fill_shape(Color32::RED), Shape::Rect(_)));
    }
}

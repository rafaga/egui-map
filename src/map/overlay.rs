//! Placement shared by the controls the map draws over its own content, the
//! [minimap](super::minimap) and the [zoom slider](super::slider).

use egui::{Rect, Vec2, pos2};

/// A corner of the map widget, where an overlay is pinned.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Corner {
    /// Top-left corner.
    TopLeft,
    /// Top-right corner.
    TopRight,
    /// Bottom-left corner.
    BottomLeft,
    /// Bottom-right corner.
    #[default]
    BottomRight,
}

impl Corner {
    /// Whether this is one of the two top corners.
    pub fn is_top(self) -> bool {
        matches!(self, Corner::TopLeft | Corner::TopRight)
    }

    /// Whether this is one of the two left corners.
    pub fn is_left(self) -> bool {
        matches!(self, Corner::TopLeft | Corner::BottomLeft)
    }

    /// The rectangle of `size` pinned to this corner of `area`, `margin` away
    /// from both of its edges.
    pub fn place(self, area: Rect, size: Vec2, margin: f32) -> Rect {
        let min = match self {
            Corner::TopLeft => pos2(area.left() + margin, area.top() + margin),
            Corner::TopRight => pos2(area.right() - margin - size.x, area.top() + margin),
            Corner::BottomLeft => pos2(area.left() + margin, area.bottom() - margin - size.y),
            Corner::BottomRight => pos2(
                area.right() - margin - size.x,
                area.bottom() - margin - size.y,
            ),
        };
        Rect::from_min_size(min, size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::vec2;

    #[test]
    fn each_corner_pins_the_rectangle_with_the_margin() {
        let area = Rect::from_min_size(pos2(100.0, 50.0), vec2(600.0, 400.0));
        let size = vec2(100.0, 80.0);
        let expected = [
            (Corner::TopLeft, pos2(110.0, 60.0)),
            (Corner::TopRight, pos2(590.0, 60.0)),
            (Corner::BottomLeft, pos2(110.0, 360.0)),
            (Corner::BottomRight, pos2(590.0, 360.0)),
        ];
        for (corner, min) in expected {
            let rect = corner.place(area, size, 10.0);
            assert_eq!(rect.min, min, "{corner:?}");
            assert_eq!(rect.size(), size, "{corner:?}");
        }
    }

    #[test]
    fn only_the_top_corners_are_top() {
        assert!(Corner::TopLeft.is_top() && Corner::TopRight.is_top());
        assert!(!Corner::BottomLeft.is_top() && !Corner::BottomRight.is_top());
    }

    #[test]
    fn only_the_left_corners_are_left() {
        assert!(Corner::TopLeft.is_left() && Corner::BottomLeft.is_left());
        assert!(!Corner::TopRight.is_left() && !Corner::BottomRight.is_left());
    }
}

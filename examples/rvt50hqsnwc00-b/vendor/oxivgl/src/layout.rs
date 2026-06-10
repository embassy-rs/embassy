// SPDX-License-Identifier: MIT OR Apache-2.0
//! Layout types: flex flow/alignment, grid alignment/cells, and layout engine
//! selection.

/// Type-safe wrapper for `lv_flex_flow_t`.
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum FlexFlow {
    /// Horizontal left to right.
    Row = 0,
    /// Vertical top to bottom.
    Column = 1,
    /// Row with wrapping.
    RowWrap = 4,
    /// Row, right to left.
    RowReverse = 8,
    /// Row with wrapping, reversed.
    RowWrapReverse = 12,
    /// Column with wrapping.
    ColumnWrap = 5,
    /// Column, bottom to top.
    ColumnReverse = 9,
    /// Column with wrapping, reversed.
    ColumnWrapReverse = 13,
}

/// Type-safe wrapper for `lv_flex_align_t`.
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum FlexAlign {
    /// Align to start.
    Start = 0,
    /// Align to end.
    End = 1,
    /// Center alignment.
    Center = 2,
    /// Equal space around all items.
    SpaceEvenly = 3,
    /// Equal space around each item.
    SpaceAround = 4,
    /// Equal space between items.
    SpaceBetween = 5,
}

/// Type-safe wrapper for `lv_grid_align_t`.
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum GridAlign {
    /// Align to start.
    Start = 0,
    /// Center alignment.
    Center = 1,
    /// Align to end.
    End = 2,
    /// Stretch to fill cell.
    Stretch = 3,
    /// Equal space around all items.
    SpaceEvenly = 4,
    /// Equal space around each item.
    SpaceAround = 5,
    /// Equal space between items.
    SpaceBetween = 6,
}

/// LVGL layout engine type.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    /// Flexbox layout.
    Flex = oxivgl_sys::lv_layout_t_LV_LAYOUT_FLEX,
    /// Grid layout.
    Grid = oxivgl_sys::lv_layout_t_LV_LAYOUT_GRID,
}

/// Grid cell placement (alignment + position + span).
/// Used with [`Obj::set_grid_cell`](crate::widgets::Obj::set_grid_cell) to avoid
/// positional argument confusion.
///
/// ```
/// use oxivgl::layout::{GridAlign, GridCell};
///
/// let col = GridCell::new(GridAlign::Stretch, 0, 1);
/// let row = GridCell::new(GridAlign::Center, 0, 1);
/// // Apply with: obj.set_grid_cell(col, row);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct GridCell {
    /// Cell alignment within the grid track.
    pub align: GridAlign,
    /// Column or row index (0-based).
    pub pos: i32,
    /// Number of columns or rows to span.
    pub span: i32,
}

impl GridCell {
    /// Create a grid cell placement.
    pub fn new(align: GridAlign, pos: i32, span: i32) -> Self {
        Self { align, pos, span }
    }

    /// Single-cell at given position with Start alignment and span 1.
    pub fn at(pos: i32) -> Self {
        Self {
            align: GridAlign::Start,
            pos,
            span: 1,
        }
    }
}

/// Sentinel value marking the end of a grid template descriptor array.
pub const GRID_TEMPLATE_LAST: i32 = oxivgl_sys::LV_COORD_MAX as i32;

/// Return a fractional grid unit. Equivalent to `LV_GRID_FR(x)`.
pub const fn grid_fr(x: i32) -> i32 {
    oxivgl_sys::LV_COORD_MAX as i32 - 100 + x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_defaults_to_start_span1() {
        let c = GridCell::at(2);
        assert_eq!(c.pos, 2);
        assert_eq!(c.span, 1);
        // GridAlign doesn't impl PartialEq, so check via debug
        assert!(format!("{:?}", c.align).contains("Start"));
    }

    #[test]
    fn new_preserves_fields() {
        let c = GridCell::new(GridAlign::Center, 3, 2);
        assert_eq!(c.pos, 3);
        assert_eq!(c.span, 2);
    }

    #[test]
    fn layout_discriminants() {
        assert_eq!(
            Layout::Flex as u32,
            oxivgl_sys::lv_layout_t_LV_LAYOUT_FLEX
        );
        assert_eq!(
            Layout::Grid as u32,
            oxivgl_sys::lv_layout_t_LV_LAYOUT_GRID
        );
    }
}

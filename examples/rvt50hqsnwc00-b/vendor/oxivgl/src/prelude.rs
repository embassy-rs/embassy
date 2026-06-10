// SPDX-License-Identifier: MIT OR Apache-2.0
//! Universal convenience re-exports for widget, style, and animation types.
//!
//! ```
//! use oxivgl::prelude::*;
//! ```

// View trait
// Animation system
pub use crate::anim::{
    ANIM_REPEAT_INFINITE, ANIM_TIMELINE_PROGRESS_MAX, Anim, AnimTimeline, anim_path_bounce, anim_path_ease_in,
    anim_path_ease_in_out, anim_path_ease_out, anim_path_linear, anim_path_overshoot, anim_set_arc_value,
    anim_set_bar_value, anim_set_height, anim_set_pad_column, anim_set_pad_row, anim_set_scale_rotation, anim_set_size, anim_set_slider_value,
    anim_set_width, anim_set_x, anim_set_y,
};
// Draw primitives (needed in DRAW_TASK_ADDED / DRAW_MAIN_END handlers)
pub use crate::draw::{
    DRAW_TASK_TYPE_FILL, DrawArcDsc, DrawBoxShadowDsc, DrawFillDsc, DrawImageDsc, DrawLabelDsc, DrawLabelDscOwned,
    DrawLetterDsc, DrawLineDsc, DrawRectDsc, DrawTask, DrawTriangleDsc, Layer, image_header_info,
};
// Canvas
pub use crate::draw_buf::{ColorFormat, DrawBuf, ImageDsc};
// Core LVGL enums
pub use crate::enums::{EventCode, Key, ObjFlag, ObjState, Opa, ScrollDir, ScrollSnap, ScrollbarMode};
// Event system
pub use crate::event::Event;
// Layout
pub use crate::layout::{FlexAlign, FlexFlow, GRID_TEMPLATE_LAST, GridAlign, GridCell, Layout, grid_fr};
// Math utilities
pub use crate::math::{BEZIER_VAL_MAX, TRIGO_SHIFT, atan2, bezier3, map, trigo_cos, trigo_sin};
// Style system
pub use crate::style::{
    BorderSide, ColorFilter, GradDir, GradDsc, GradExtend, LV_SIZE_CONTENT, Palette, Selector, Style, StyleBuilder,
    TextDecor, Theme, TransitionDsc, color_black, color_brightness, color_darken, color_hsv, color_make, color_mix,
    color_white, darken_filter_cb, lv_pct, palette_darken, palette_lighten, palette_main, props,
};
// Symbol icons
pub use crate::symbols::Symbol;
// Timer
pub use crate::timer::Timer;
// Widgets
pub use crate::widgets::{
    Align, AnimImg, Arc, ArcLabel, ArcLabelDir, ArcMode, AsLvHandle, Bar, BarMode, BaseDir, Button, Buttonmatrix, ButtonmatrixMap, Calendar,
    CalendarDate, Chart, ChartAxis,
    ChartSeries, ChartType, Checkbox, Child, DdDir, Dropdown, Image, ImageAlign, Imagebutton, ImagebuttonState,
    Keyboard, KeyboardMode, Label,
    LabelLongMode, Led, Line, List, Matrix, Menu, MenuHeaderMode, Msgbox, Obj, Part, RADIUS_MAX, Roller, RollerMode,
    SCALE_LABEL_ROTATE_KEEP_UPRIGHT, SCALE_LABEL_ROTATE_MATCH_TICKS, Scale, ScaleBuilder, ScaleLabels, ScaleMode,
    ScaleSection, Screen, ScreenAnim, ScreenAnimType, Slider, SliderMode, Span, SpanMode, SpanOverflow, Spangroup,
    Spinbox, Spinner, Switch,
    SwitchOrientation, Table, TableCellCtrl, Tabview, TextAlign, Textarea, Tileview, ValueLabel, WidgetError, Win,
};
pub use crate::{
    view::View,
    widgets::{Canvas, lv_color_t, lv_image_dsc_t, lv_point_precise_t},
};
// Group and gridnav
pub use crate::group::{Group, GroupRef, group_get_default, group_remove_obj};
pub use crate::gridnav::{GridnavCtrl, gridnav_add, gridnav_remove};

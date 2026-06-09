//! Styling for LVGL objects and widgets
//!
//! Objects in LVGL can have associated styling information. After a `Style` is
//! created and configured, it can be added to any object or widget:
//! ```
//! use lvgl::{Color, Widget};
//! use lvgl::style::Style;
//!
//! let mut my_style = Style::default();
//! my_style.set_text_color(Color::from_rgb((0, 0, 0)));
//!
//! //my_widget.add_style(Part::Main, &mut my_style).unwrap();
//! // ...
//! ```
//! All methods on the `Style` type directly lower to their C LVGL
//! counterparts.

use crate::{font::Font, Align, Box, Color, TextAlign};
use core::fmt;
use core::fmt::Debug;
use core::mem::{self, MaybeUninit};
use cty::c_uint;
use paste::paste;

pub enum Themes {
    Pretty,
}

/// An LVGL `lv_style_t`. Allows for styling objects. Once created, a `Style`
/// should be configured and then added to an object.
#[derive(Clone)]
pub struct Style {
    pub(crate) raw: Box<lvgl_sys::lv_style_t>,
}

impl Debug for Style {
    // TODO: Decode and dump style values
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Style")
            .field("raw", &"!! LVGL lv_style_t ptr !!")
            .finish()
    }
}

impl Default for Style {
    fn default() -> Self {
        let raw = unsafe {
            let mut style = mem::MaybeUninit::<lvgl_sys::lv_style_t>::uninit();
            lvgl_sys::lv_style_init(style.as_mut_ptr());
            Box::new(style.assume_init())
        };
        Self { raw }
    }
}

bitflags! {
    /// Represents possible opacities for use on `Style` objects.
    #[derive(Debug, Clone, Copy)]
    pub struct Opacity: u32 {
        const OPA_TRANSP = lvgl_sys::LV_OPA_TRANSP;
        const OPA_0 = lvgl_sys::LV_OPA_0;
        const OPA_10 = lvgl_sys::LV_OPA_10;
        const OPA_20 = lvgl_sys::LV_OPA_20;
        const OPA_30 = lvgl_sys::LV_OPA_30;
        const OPA_40 = lvgl_sys::LV_OPA_40;
        const OPA_50 = lvgl_sys::LV_OPA_50;
        const OPA_60 = lvgl_sys::LV_OPA_60;
        const OPA_70 = lvgl_sys::LV_OPA_70;
        const OPA_80 = lvgl_sys::LV_OPA_80;
        const OPA_90 = lvgl_sys::LV_OPA_90;
        const OPA_100 = lvgl_sys::LV_OPA_100;
        const OPA_COVER = lvgl_sys::LV_OPA_COVER;
    }
}

impl From<Opacity> for u8 {
    fn from(value: Opacity) -> u8 {
        value.bits() as u8
    }
}

bitflags! {
    pub struct GridAlign: c_uint {
        const START = lvgl_sys::lv_grid_align_t_LV_GRID_ALIGN_START;
        const CENTER = lvgl_sys::lv_grid_align_t_LV_GRID_ALIGN_CENTER;
        const END = lvgl_sys::lv_grid_align_t_LV_GRID_ALIGN_END;
        const STRETCH = lvgl_sys::lv_grid_align_t_LV_GRID_ALIGN_STRETCH;
        const SPACE_AROUND = lvgl_sys::lv_grid_align_t_LV_GRID_ALIGN_SPACE_AROUND;
        const SPACE_BETWEEN = lvgl_sys::lv_grid_align_t_LV_GRID_ALIGN_SPACE_BETWEEN;
        const SPACE_EVENLY = lvgl_sys::lv_grid_align_t_LV_GRID_ALIGN_SPACE_EVENLY;
    }
}

impl From<GridAlign> for c_uint {
    fn from(value: GridAlign) -> Self {
        value.bits() as c_uint
    }
}

impl From<GridAlign> for i16 {
    fn from(value: GridAlign) -> Self {
        value.bits() as i16
    }
}

bitflags! {
    pub struct FlexAlign: c_uint {
        const START = lvgl_sys::lv_flex_align_t_LV_FLEX_ALIGN_START;
        const CENTER = lvgl_sys::lv_flex_align_t_LV_FLEX_ALIGN_CENTER;
        const END = lvgl_sys::lv_flex_align_t_LV_FLEX_ALIGN_END;
        const SPACE_AROUND = lvgl_sys::lv_flex_align_t_LV_FLEX_ALIGN_SPACE_AROUND;
        const SPACE_BETWEEN = lvgl_sys::lv_flex_align_t_LV_FLEX_ALIGN_SPACE_BETWEEN;
        const SPACE_EVENLY = lvgl_sys::lv_flex_align_t_LV_FLEX_ALIGN_SPACE_EVENLY;
    }
}

impl From<FlexAlign> for c_uint {
    fn from(value: FlexAlign) -> Self {
        value.bits() as c_uint
    }
}

bitflags! {
    pub struct FlexFlow: c_uint {
        const COLUMN = lvgl_sys::lv_flex_flow_t_LV_FLEX_FLOW_COLUMN;
        const COLUMN_REVERSE = lvgl_sys::lv_flex_flow_t_LV_FLEX_FLOW_COLUMN_REVERSE;
        const COLUMN_WRAP = lvgl_sys::lv_flex_flow_t_LV_FLEX_FLOW_COLUMN_WRAP;
        const COLUMN_WRAP_REVERSE = lvgl_sys::lv_flex_flow_t_LV_FLEX_FLOW_COLUMN_WRAP_REVERSE;
        const ROW = lvgl_sys::lv_flex_flow_t_LV_FLEX_FLOW_ROW;
        const ROW_REVERSE = lvgl_sys::lv_flex_flow_t_LV_FLEX_FLOW_ROW_REVERSE;
        const ROW_WRAP = lvgl_sys::lv_flex_flow_t_LV_FLEX_FLOW_ROW_WRAP;
        const ROW_WRAP_REVERSE = lvgl_sys::lv_flex_flow_t_LV_FLEX_FLOW_ROW_WRAP_REVERSE;
    }
}

impl From<FlexFlow> for c_uint {
    fn from(value: FlexFlow) -> Self {
        value.bits() as c_uint
    }
}

/// Represents a `Layout`, to be used with the `set_layout()` method on `Style`
/// objects.
pub struct Layout {
    inner: u16,
}

impl Layout {
    /// Generates an `LV_LAYOUT_FLEX`
    pub fn flex() -> Self {
        Self {
            inner: unsafe { lvgl_sys::LV_LAYOUT_FLEX },
        }
    }

    /// Generates an `LV_LAYOUT_GRID`
    pub fn grid() -> Self {
        Self {
            inner: unsafe { lvgl_sys::LV_LAYOUT_GRID },
        }
    }
}

impl From<Layout> for u16 {
    fn from(value: Layout) -> Self {
        value.inner
    }
}

/// A coordinate array, for use with `set_grid_*_dsc_array()` methods on
/// `Style` objects.
#[derive(Clone)]
#[repr(C)]
pub struct CoordDesc<const N: usize> {
    inner: [i16; N],
    tail: i16,
}

impl<const N: usize> CoordDesc<N> {
    /// Generates a `CoordDesc` from values.
    ///
    /// # Safety
    ///
    /// `N` must be at least as long as LVGL expects. See the LVGL docs for
    /// details.
    pub unsafe fn from_values(values: [i16; N], is_grid: bool) -> Self {
        Self {
            inner: values,
            tail: if is_grid {
                lvgl_sys::LV_GRID_TEMPLATE_LAST.try_into().unwrap()
            } else {
                0b0
            },
        }
    }

    /// Returns the values contained.
    pub fn values(&self) -> [i16; N] {
        self.clone().inner
    }
}

impl<const N: usize> From<&CoordDesc<N>> for *const i16 {
    fn from(value: &CoordDesc<N>) -> Self {
        value as *const _ as *const i16
    }
}

#[derive(Clone)]
pub enum StyleValues {
    Num(i32),
    Color(Color),
    Opacity(Opacity),
    //Align(Align),
    None,
}

impl StyleValues {
    pub fn is_some(&self) -> bool {
        !matches!(self, StyleValues::None)
    }
}

/*impl StyleValues {
    pub fn num(&self) -> i32 {
        self.num
    }

    pub fn color(&self) -> Color {
        self.color
    }
}

impl From<lvgl_sys::lv_style_value_t> for StyleValues {
    fn from(value: lvgl_sys::lv_style_value_t) -> Self {
        #[cfg(debug_assertions)]
        assert!(!value.ptr.is_null());
        Self {
            num: value.num,
            color: Color::from_raw(value.color),
        }
    }
}*/

bitflags! {
    /// Various constants relevant for `Style` parameters
    #[derive(PartialEq, Eq)]
    pub struct StyleProp: u32 {
        //const PROP_INV = lvgl_sys::lv_style_prop_t_LV_STYLE_PROP_INV;

        /*Group 0*/
        const WIDTH = lvgl_sys::lv_style_prop_t_LV_STYLE_WIDTH;
        const MIN_WIDTH = lvgl_sys::lv_style_prop_t_LV_STYLE_MIN_WIDTH;
        const MAX_WIDTH = lvgl_sys::lv_style_prop_t_LV_STYLE_MAX_WIDTH;
        const HEIGHT = lvgl_sys::lv_style_prop_t_LV_STYLE_HEIGHT;
        const MIN_HEIGHT = lvgl_sys::lv_style_prop_t_LV_STYLE_MIN_HEIGHT;
        const MAX_HEIGHT = lvgl_sys::lv_style_prop_t_LV_STYLE_MAX_HEIGHT;
        const X = lvgl_sys::lv_style_prop_t_LV_STYLE_X;
        const Y = lvgl_sys::lv_style_prop_t_LV_STYLE_Y;
        const ALIGN = lvgl_sys::lv_style_prop_t_LV_STYLE_ALIGN;
        const TRANSFORM_WIDTH = lvgl_sys::lv_style_prop_t_LV_STYLE_TRANSFORM_WIDTH;
        const TRANSFORM_HEIGHT = lvgl_sys::lv_style_prop_t_LV_STYLE_TRANSFORM_HEIGHT;
        const TRANSLATE_X = lvgl_sys::lv_style_prop_t_LV_STYLE_TRANSLATE_X;
        const TRANSLATE_Y = lvgl_sys::lv_style_prop_t_LV_STYLE_TRANSLATE_Y;
        const TRANSFORM_ZOOM = lvgl_sys::lv_style_prop_t_LV_STYLE_TRANSFORM_ZOOM;
        const TRANSFORM_ANGLE = lvgl_sys::lv_style_prop_t_LV_STYLE_TRANSFORM_ANGLE;

        /*Group 1*/
        const PAD_TOP = lvgl_sys::lv_style_prop_t_LV_STYLE_PAD_TOP;
        const PAD_BOTTOM = lvgl_sys::lv_style_prop_t_LV_STYLE_PAD_BOTTOM;
        const PAD_LEFT = lvgl_sys::lv_style_prop_t_LV_STYLE_PAD_LEFT;
        const PAD_RIGHT = lvgl_sys::lv_style_prop_t_LV_STYLE_PAD_RIGHT;
        const PAD_ROW = lvgl_sys::lv_style_prop_t_LV_STYLE_PAD_ROW;
        const PAD_COLUMN = lvgl_sys::lv_style_prop_t_LV_STYLE_PAD_COLUMN;

        /*Group 2*/
        const BG_COLOR = lvgl_sys::lv_style_prop_t_LV_STYLE_BG_COLOR;
        //const BG_COLOR_FILTERED = lvgl_sys::lv_style_prop_t_LV_STYLE_BG_COLOR_FILTERED as u32;
        const BG_OPA = lvgl_sys::lv_style_prop_t_LV_STYLE_BG_OPA;
        const BG_GRAD_COLOR = lvgl_sys::lv_style_prop_t_LV_STYLE_BG_GRAD_COLOR;
        //const BG_GRAD_COLOR_FILTERED = lvgl_sys::lv_style_prop_t_LV_STYLE_BG_GRAD_COLOR_FILTERED as u32;
        const BG_GRAD_DIR = lvgl_sys::lv_style_prop_t_LV_STYLE_BG_GRAD_DIR;
        const BG_MAIN_STOP = lvgl_sys::lv_style_prop_t_LV_STYLE_BG_MAIN_STOP;
        const BG_GRAD_STOP = lvgl_sys::lv_style_prop_t_LV_STYLE_BG_GRAD_STOP;

        const BG_IMG_SRC = lvgl_sys::lv_style_prop_t_LV_STYLE_BG_IMG_SRC;
        const BG_IMG_OPA = lvgl_sys::lv_style_prop_t_LV_STYLE_BG_IMG_OPA;
        const BG_IMG_RECOLOR = lvgl_sys::lv_style_prop_t_LV_STYLE_BG_IMG_RECOLOR;
        //const BG_IMG_RECOLOR_FILTERED = lvgl_sys::lv_style_prop_t_LV_STYLE_BG_IMG_RECOLOR_FILTERED as u32;
        const BG_IMG_RECOLOR_OPA = lvgl_sys::lv_style_prop_t_LV_STYLE_BG_IMG_RECOLOR_OPA;
        const BG_IMG_TILED = lvgl_sys::lv_style_prop_t_LV_STYLE_BG_IMG_TILED;

        /*Group 3*/
        const BORDER_COLOR = lvgl_sys::lv_style_prop_t_LV_STYLE_BORDER_COLOR;
        //const BORDER_COLOR_FILTERED = lvgl_sys::lv_style_prop_t_LV_STYLE_BORDER_COLOR_FILTERED as u32;
        const BORDER_OPA = lvgl_sys::lv_style_prop_t_LV_STYLE_BORDER_OPA;
        const BORDER_WIDTH = lvgl_sys::lv_style_prop_t_LV_STYLE_BORDER_WIDTH;
        const BORDER_SIDE = lvgl_sys::lv_style_prop_t_LV_STYLE_BORDER_SIDE;
        const BORDER_POST = lvgl_sys::lv_style_prop_t_LV_STYLE_BORDER_POST;

        const OUTLINE_WIDTH = lvgl_sys::lv_style_prop_t_LV_STYLE_OUTLINE_WIDTH;
        const OUTLINE_COLOR = lvgl_sys::lv_style_prop_t_LV_STYLE_OUTLINE_COLOR;
        //const OUTLINE_COLOR_FILTERED = lvgl_sys::lv_style_prop_t_LV_STYLE_OUTLINE_COLOR_FILTERED as u32;
        const OUTLINE_OPA = lvgl_sys::lv_style_prop_t_LV_STYLE_OUTLINE_OPA;
        const OUTLINE_PAD = lvgl_sys::lv_style_prop_t_LV_STYLE_OUTLINE_PAD;

        /*Group 4*/
        const SHADOW_WIDTH = lvgl_sys::lv_style_prop_t_LV_STYLE_SHADOW_WIDTH;
        const SHADOW_OFS_X = lvgl_sys::lv_style_prop_t_LV_STYLE_SHADOW_OFS_X;
        const SHADOW_OFS_Y = lvgl_sys::lv_style_prop_t_LV_STYLE_SHADOW_OFS_Y;
        const SHADOW_SPREAD = lvgl_sys::lv_style_prop_t_LV_STYLE_SHADOW_SPREAD;
        const SHADOW_COLOR = lvgl_sys::lv_style_prop_t_LV_STYLE_SHADOW_COLOR;
        //const SHADOW_COLOR_FILTERED = lvgl_sys::lv_style_prop_t_LV_STYLE_SHADOW_COLOR_FILTERED as u32;
        const SHADOW_OPA = lvgl_sys::lv_style_prop_t_LV_STYLE_SHADOW_OPA;

        const IMG_OPA = lvgl_sys::lv_style_prop_t_LV_STYLE_IMG_OPA;
        const IMG_RECOLOR = lvgl_sys::lv_style_prop_t_LV_STYLE_IMG_RECOLOR;
        //const IMG_RECOLOR_FILTERED = lvgl_sys::lv_style_prop_t_LV_STYLE_IMG_RECOLOR_FILTERED as u32;
        const IMG_RECOLOR_OPA = lvgl_sys::lv_style_prop_t_LV_STYLE_IMG_RECOLOR_OPA;

        const LINE_WIDTH = lvgl_sys::lv_style_prop_t_LV_STYLE_LINE_WIDTH;
        const LINE_DASH_WIDTH = lvgl_sys::lv_style_prop_t_LV_STYLE_LINE_DASH_WIDTH;
        const LINE_DASH_GAP = lvgl_sys::lv_style_prop_t_LV_STYLE_LINE_DASH_GAP;
        const LINE_ROUNDED = lvgl_sys::lv_style_prop_t_LV_STYLE_LINE_ROUNDED;
        const LINE_COLOR = lvgl_sys::lv_style_prop_t_LV_STYLE_LINE_COLOR;
        //const LINE_COLOR_FILTERED = lvgl_sys::lv_style_prop_t_LV_STYLE_LINE_COLOR_FILTERED as u32;
        const LINE_OPA = lvgl_sys::lv_style_prop_t_LV_STYLE_LINE_OPA;

        /*Group 5*/
        const ARC_WIDTH = lvgl_sys::lv_style_prop_t_LV_STYLE_ARC_WIDTH;
        const ARC_ROUNDED = lvgl_sys::lv_style_prop_t_LV_STYLE_ARC_ROUNDED;
        const ARC_COLOR = lvgl_sys::lv_style_prop_t_LV_STYLE_ARC_COLOR;
        //const ARC_COLOR_FILTERED = lvgl_sys::lv_style_prop_t_LV_STYLE_ARC_COLOR_FILTERED as u32;
        const ARC_OPA = lvgl_sys::lv_style_prop_t_LV_STYLE_ARC_OPA;
        const ARC_IMG_SRC = lvgl_sys::lv_style_prop_t_LV_STYLE_ARC_IMG_SRC;

        const TEXT_COLOR = lvgl_sys::lv_style_prop_t_LV_STYLE_TEXT_COLOR;
        //const TEXT_COLOR_FILTERED = lvgl_sys::lv_style_prop_t_LV_STYLE_TEXT_COLOR_FILTERED as u32;
        const TEXT_OPA = lvgl_sys::lv_style_prop_t_LV_STYLE_TEXT_OPA;
        const TEXT_FONT = lvgl_sys::lv_style_prop_t_LV_STYLE_TEXT_FONT;
        const TEXT_LETTER_SPACE = lvgl_sys::lv_style_prop_t_LV_STYLE_TEXT_LETTER_SPACE;
        const TEXT_LINE_SPACE = lvgl_sys::lv_style_prop_t_LV_STYLE_TEXT_LINE_SPACE;
        const TEXT_DECOR = lvgl_sys::lv_style_prop_t_LV_STYLE_TEXT_DECOR;
        const TEXT_ALIGN = lvgl_sys::lv_style_prop_t_LV_STYLE_TEXT_ALIGN;

        /*Group 6*/
        const RADIUS = lvgl_sys::lv_style_prop_t_LV_STYLE_RADIUS;
        const CLIP_CORNER = lvgl_sys::lv_style_prop_t_LV_STYLE_CLIP_CORNER;
        const OPA = lvgl_sys::lv_style_prop_t_LV_STYLE_OPA;
        const COLOR_FILTER_DSC = lvgl_sys::lv_style_prop_t_LV_STYLE_COLOR_FILTER_DSC;
        const COLOR_FILTER_OPA = lvgl_sys::lv_style_prop_t_LV_STYLE_COLOR_FILTER_OPA;
        const ANIM_TIME = lvgl_sys::lv_style_prop_t_LV_STYLE_ANIM_TIME;
        const ANIM_SPEED = lvgl_sys::lv_style_prop_t_LV_STYLE_ANIM_SPEED;
        const TRANSITION = lvgl_sys::lv_style_prop_t_LV_STYLE_TRANSITION;
        const BLEND_MODE = lvgl_sys::lv_style_prop_t_LV_STYLE_BLEND_MODE;
        const LAYOUT = lvgl_sys::lv_style_prop_t_LV_STYLE_LAYOUT;
        const BASE_DIR = lvgl_sys::lv_style_prop_t_LV_STYLE_BASE_DIR;

        //const PROP_ANY = lvgl_sys::lv_style_prop_t_LV_STYLE_PROP_ANY;
    }
}

macro_rules! gen_lv_style {
    ($func_name:ident,$vty:ty) => {
        paste! {
            #[inline]
            pub fn $func_name(&mut self, value: $vty) {
                unsafe {
                    lvgl_sys::[<lv_style_ $func_name>](
                        self.raw.as_mut(),
                        value.into(),
                    );
                }
            }
        }
    };
}

macro_rules! gen_lv_style_generic {
    ($func_name:ident,$vty:ty) => {
        paste! {
            #[inline]
            pub fn $func_name<const N: usize>(&mut self, value: &$vty<N>) {
                unsafe {
                    lvgl_sys::[<lv_style_ $func_name>](
                        self.raw.as_mut(),
                        value.into(),
                    );
                }
            }
        }
    };
}

impl Style {
    pub fn get_prop(&self, prop: StyleProp) -> StyleValues {
        let mut raw_ret = MaybeUninit::<lvgl_sys::lv_style_value_t>::uninit();
        let mut ret = match prop {
            StyleProp::WIDTH
            | StyleProp::MIN_WIDTH
            | StyleProp::MAX_WIDTH
            | StyleProp::HEIGHT
            | StyleProp::MIN_HEIGHT
            | StyleProp::MAX_HEIGHT
            | StyleProp::X
            | StyleProp::Y
            | StyleProp::TRANSFORM_WIDTH
            | StyleProp::TRANSFORM_HEIGHT
            | StyleProp::TRANSFORM_ZOOM
            | StyleProp::TRANSFORM_ANGLE
            | StyleProp::TRANSLATE_X
            | StyleProp::TRANSLATE_Y
            | StyleProp::PAD_TOP
            | StyleProp::PAD_LEFT
            | StyleProp::PAD_BOTTOM
            | StyleProp::PAD_RIGHT
            | StyleProp::PAD_ROW
            | StyleProp::PAD_COLUMN
            | StyleProp::BORDER_WIDTH
            | StyleProp::OUTLINE_WIDTH
            | StyleProp::OUTLINE_PAD
            | StyleProp::SHADOW_WIDTH
            | StyleProp::SHADOW_SPREAD
            | StyleProp::SHADOW_OFS_X
            | StyleProp::SHADOW_OFS_Y
            | StyleProp::LINE_WIDTH
            | StyleProp::LINE_DASH_WIDTH
            | StyleProp::LINE_DASH_GAP
            | StyleProp::ARC_WIDTH
            | StyleProp::RADIUS => StyleValues::Num(0),

            StyleProp::BG_OPA
            | StyleProp::BG_IMG_OPA
            | StyleProp::BG_IMG_RECOLOR_OPA
            | StyleProp::BORDER_OPA
            | StyleProp::OUTLINE_OPA
            | StyleProp::SHADOW_OPA
            | StyleProp::IMG_OPA
            | StyleProp::IMG_RECOLOR_OPA
            | StyleProp::LINE_OPA
            | StyleProp::ARC_OPA
            | StyleProp::TEXT_OPA
            | StyleProp::OPA => StyleValues::Opacity(Opacity::OPA_0),

            StyleProp::BG_COLOR
            | StyleProp::BG_GRAD_COLOR
            | StyleProp::BORDER_COLOR
            | StyleProp::OUTLINE_COLOR
            | StyleProp::SHADOW_COLOR
            | StyleProp::LINE_COLOR
            | StyleProp::ARC_COLOR
            | StyleProp::TEXT_COLOR => StyleValues::Color(Color::default()),

            _ => StyleValues::None,
        };

        let ptr = raw_ret.as_mut_ptr() as *mut _;
        let result = unsafe {
            lvgl_sys::lv_style_get_prop(Box::into_raw(self.raw.clone()) as *const _, prop.bits(), ptr)
        };
        let raw_ret = unsafe { raw_ret.assume_init() };
        if <u8 as Into<u32>>::into(result) == lvgl_sys::LV_RES_OK {
            unsafe {
                ret = match ret {
                    StyleValues::Num(_) => StyleValues::Num(raw_ret.num),
                    StyleValues::Opacity(_) => StyleValues::Opacity(Opacity::from_bits_retain(
                        raw_ret.num.try_into().unwrap(),
                    )),
                    StyleValues::Color(_) => StyleValues::Color(Color::from_raw(raw_ret.color)),
                    _ => StyleValues::None,
                }
            }
            ret
        } else {
            StyleValues::None
        }
        /*unsafe {
            let ret = lvgl_sys::lv_style_value_t {
                num: 0,
                ptr: core::ptr::null(),
                color: Color::from_rgb((0, 0, 0)).raw,
            };
        }*/
    }

    gen_lv_style!(set_align, Align);
    //gen_lv_style!(set_anim, );
    //gen_lv_style!(set_anim_speed, );
    //gen_lv_style!(set_anim_time, );
    gen_lv_style!(set_arc_color, Color);
    //gen_lv_style!(set_arc_img_src, );
    gen_lv_style!(set_arc_opa, Opacity);
    gen_lv_style!(set_arc_rounded, bool);
    gen_lv_style!(set_arc_width, i16);
    //gen_lv_style!(set_base_dir, );
    gen_lv_style!(set_bg_color, Color);
    gen_lv_style!(set_bg_dither_mode, u8);
    //gen_lv_style!(set_bg_grad, );
    gen_lv_style!(set_bg_grad_color, Color);
    //gen_lv_style!(set_bg_grad_dir, );
    gen_lv_style!(set_bg_grad_stop, i16);
    gen_lv_style!(set_bg_img_opa, Opacity);
    gen_lv_style!(set_bg_img_recolor, Color);
    gen_lv_style!(set_bg_img_recolor_opa, Opacity);
    //gen_lv_style!(set_bg_img_src, );
    gen_lv_style!(set_bg_img_tiled, bool);
    gen_lv_style!(set_bg_main_stop, i16);
    gen_lv_style!(set_bg_opa, Opacity);
    gen_lv_style!(set_blend_mode, u8);
    gen_lv_style!(set_border_color, Color);
    gen_lv_style!(set_border_opa, Opacity);
    gen_lv_style!(set_border_post, bool);
    gen_lv_style!(set_border_side, u8);
    gen_lv_style!(set_border_width, i16);
    gen_lv_style!(set_clip_corner, bool);
    //gen_lv_style!(set_color_filter_dsc, );
    gen_lv_style!(set_color_filter_opa, Opacity);
    gen_lv_style!(set_flex_flow, FlexFlow);
    gen_lv_style!(set_flex_grow, u8);
    gen_lv_style!(set_flex_main_place, FlexAlign);
    gen_lv_style!(set_flex_cross_place, FlexAlign);
    gen_lv_style!(set_flex_track_place, FlexAlign);
    gen_lv_style!(set_grid_cell_column_pos, i16);
    gen_lv_style!(set_grid_cell_column_span, i16);
    gen_lv_style!(set_grid_cell_row_pos, i16);
    gen_lv_style!(set_grid_cell_row_span, i16);
    gen_lv_style!(set_grid_cell_x_align, GridAlign);
    gen_lv_style!(set_grid_cell_y_align, GridAlign);
    gen_lv_style!(set_grid_column_align, GridAlign);
    gen_lv_style_generic!(set_grid_column_dsc_array, CoordDesc);
    gen_lv_style!(set_grid_row_align, GridAlign);
    gen_lv_style_generic!(set_grid_row_dsc_array, CoordDesc);
    gen_lv_style!(set_height, i16);
    gen_lv_style!(set_img_opa, Opacity);
    gen_lv_style!(set_img_recolor, Color);
    gen_lv_style!(set_img_recolor_opa, Opacity);
    gen_lv_style!(set_layout, Layout);
    gen_lv_style!(set_line_color, Color);
    gen_lv_style!(set_line_dash_gap, i16);
    gen_lv_style!(set_line_dash_width, i16);
    gen_lv_style!(set_line_opa, Opacity);
    gen_lv_style!(set_line_rounded, bool);
    gen_lv_style!(set_line_width, i16);
    gen_lv_style!(set_max_height, i16);
    gen_lv_style!(set_max_width, i16);
    gen_lv_style!(set_min_height, i16);
    gen_lv_style!(set_min_width, i16);
    gen_lv_style!(set_opa, Opacity);
    gen_lv_style!(set_outline_color, Color);
    gen_lv_style!(set_outline_opa, Opacity);
    gen_lv_style!(set_outline_pad, i16);
    gen_lv_style!(set_outline_width, i16);
    gen_lv_style!(set_pad_bottom, i16);
    gen_lv_style!(set_pad_column, i16);
    gen_lv_style!(set_pad_left, i16);
    gen_lv_style!(set_pad_right, i16);
    gen_lv_style!(set_pad_row, i16);
    gen_lv_style!(set_pad_top, i16);
    //gen_lv_style!(set_prop, );
    //gen_lv_style!(set_prop_meta, );
    gen_lv_style!(set_radius, i16);
    gen_lv_style!(set_shadow_color, Color);
    gen_lv_style!(set_shadow_ofs_x, i16);
    gen_lv_style!(set_shadow_ofs_y, i16);
    gen_lv_style!(set_shadow_opa, Opacity);
    gen_lv_style!(set_shadow_spread, i16);
    gen_lv_style!(set_shadow_width, i16);
    gen_lv_style!(set_text_align, TextAlign);
    gen_lv_style!(set_text_color, Color);
    gen_lv_style!(set_text_decor, u8);
    gen_lv_style!(set_text_font, Font);
    gen_lv_style!(set_text_letter_space, i16);
    gen_lv_style!(set_text_line_space, i16);
    gen_lv_style!(set_text_opa, Opacity);
    gen_lv_style!(set_transform_angle, i16);
    gen_lv_style!(set_transform_height, i16);
    gen_lv_style!(set_transform_pivot_x, i16);
    gen_lv_style!(set_transform_pivot_y, i16);
    gen_lv_style!(set_transform_width, i16);
    gen_lv_style!(set_transform_zoom, i16);
    //gen_lv_style!(set_transition, );
    gen_lv_style!(set_translate_x, i16);
    gen_lv_style!(set_translate_y, i16);
    gen_lv_style!(set_width, i16);
    gen_lv_style!(set_x, i16);
    gen_lv_style!(set_y, i16);
}

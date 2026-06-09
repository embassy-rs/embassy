//! Native LVGL objects
//!
//! Objects are individual elements of a displayed surface, similar to widgets.
//! Specifically, an object can either be a widget or a screen. Screen objects
//! are special in that they do not have a parent object but do still implement
//! `NativeObject`.

use crate::lv_core::style::Style;
use crate::{Align, LvError, LvResult};
use core::fmt::{self, Debug};
use core::marker::PhantomData;
use core::ptr::{self, NonNull};

/// Represents a native LVGL object.
pub trait NativeObject {
    /// Provide common way to access to the underlying native object pointer.
    fn raw(&self) -> NonNull<lvgl_sys::lv_obj_t>;
}

/// Generic LVGL object.
///
/// This is the parent object of all widget types. It stores the native LVGL
/// raw pointer.
pub struct Obj<'a> {
    // We use a raw pointer here because we do not control this memory address,
    // it is controlled by LVGL's global state.
    raw: NonNull<lvgl_sys::lv_obj_t>,
    // This is to ensure safety for children memory; it has no runtime impact
    dependents: PhantomData<&'a isize>,
}

impl Debug for Obj<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("NativeObject")
            .field("raw", &"!! LVGL lv_obj_t ptr !!")
            .finish()
    }
}

// We need to manually impl methods on Obj since widget codegen is defined in
// terms of Obj
impl<'a> Obj<'a> {
    pub fn create(parent: &'a mut impl NativeObject) -> LvResult<Self> {
        unsafe {
            let ptr = lvgl_sys::lv_obj_create(parent.raw().as_mut());
            if let Some(nn_ptr) = ptr::NonNull::new(ptr) {
                //(*ptr).user_data = Box::new(UserDataObj::empty()).into_raw() as *mut _;
                Ok(Self {
                    raw: nn_ptr,
                    dependents: PhantomData::<&'a _>,
                })
            } else {
                Err(LvError::InvalidReference)
            }
        }
    }

    pub fn new() -> crate::LvResult<Self> {
        let mut parent = crate::display::get_scr_act()?;
        Self::create(unsafe { &mut *(&mut parent as *mut _) })
    }

    pub fn blank() -> LvResult<Self> {
        match NonNull::new(unsafe { lvgl_sys::lv_obj_create(ptr::null_mut()) }) {
            Some(raw) => Ok(Self {
                raw,
                dependents: PhantomData,
            }),
            None => Err(LvError::LvOOMemory),
        }
    }
}

impl NativeObject for Obj<'_> {
    fn raw(&self) -> ptr::NonNull<lvgl_sys::lv_obj_t> {
        self.raw
    }
}

/// A wrapper for all LVGL common operations on generic objects.
pub trait Widget<'a>: NativeObject + Sized + 'a {
    type SpecialEvent;
    type Part: Into<lvgl_sys::lv_part_t>;

    /// Construct an instance of the object from a raw pointer.
    ///
    /// # Safety
    ///
    /// If the pointer is derived from a Rust-instantiated `obj` such as via
    /// calling `.raw()`, only the `obj` that survives longest may be dropped
    /// and the caller is responsible for ensuring data races do not occur.
    unsafe fn from_raw(raw_pointer: ptr::NonNull<lvgl_sys::lv_obj_t>) -> Option<Self>;

    /// Adds a `Style` to a given widget.
    fn add_style(&mut self, part: Self::Part, style: &'a mut Style) {
        unsafe {
            lvgl_sys::lv_obj_add_style(
                self.raw().as_mut(),
                style.raw.as_mut() as *mut _,
                part.into(),
            );
        };
    }

    /// Sets a widget's position relative to its parent.
    fn set_pos(&mut self, x: i16, y: i16) {
        unsafe {
            lvgl_sys::lv_obj_set_pos(
                self.raw().as_mut(),
                x as lvgl_sys::lv_coord_t,
                y as lvgl_sys::lv_coord_t,
            );
        }
    }

    /// Sets a widget's size. Alternatively, use `set_width()` and `set_height()`.
    fn set_size(&mut self, w: i16, h: i16) {
        unsafe {
            lvgl_sys::lv_obj_set_size(
                self.raw().as_mut(),
                w as lvgl_sys::lv_coord_t,
                h as lvgl_sys::lv_coord_t,
            );
        }
    }

    /// Sets a widget's width. Alternatively, use `set_size()`.
    fn set_width(&mut self, w: u32) {
        unsafe {
            lvgl_sys::lv_obj_set_width(self.raw().as_mut(), w as lvgl_sys::lv_coord_t);
        }
    }

    /// Sets a widget's height. Alternatively, use `set_size()`.
    fn set_height(&mut self, h: u32) {
        unsafe {
            lvgl_sys::lv_obj_set_height(self.raw().as_mut(), h as lvgl_sys::lv_coord_t);
        }
    }

    /// Sets a widget's align relative to its parent along with an offset.
    fn set_align(&mut self, align: Align, x_mod: i32, y_mod: i32) {
        unsafe {
            lvgl_sys::lv_obj_align(
                self.raw().as_mut(),
                align.into(),
                x_mod as lvgl_sys::lv_coord_t,
                y_mod as lvgl_sys::lv_coord_t,
            );
        }
    }
}

impl<'a> Widget<'a> for Obj<'a> {
    type SpecialEvent = u32;
    type Part = Part;

    unsafe fn from_raw(raw: NonNull<lvgl_sys::lv_obj_t>) -> Option<Self> {
        Some(Self {
            raw,
            dependents: PhantomData,
        })
    }
}

macro_rules! define_object {
    ($item:ident) => {
        define_object!($item, event = (), part = $crate::Part);
    };
    ($item:ident, event = $event_type:ty) => {
        define_object!($item, event = $event_type, part = $crate::Part);
    };
    ($item:ident, part = $part_type:ty) => {
        define_object!($item, event = (), part = $part_type);
    };
    ($item:ident, part = $part_type:ty, event = $event_type:ty) => {
        define_object!($item, event = $event_type, part = $part_type);
    };
    ($item:ident, event = $event_type:ty, part = $part_type:ty) => {
        #[derive(Debug)]
        pub struct $item<'a> {
            core: $crate::Obj<'a>,
        }

        impl<'a> $item<'a> {
            pub fn on_event<F>(&mut self, f: F) -> $crate::LvResult<()>
            where
                F: FnMut(Self, $crate::support::Event<<Self as $crate::Widget<'a>>::SpecialEvent>),
            {
                use $crate::NativeObject;
                unsafe {
                    let obj = self.raw().as_mut();
                    obj.user_data = $crate::Box::into_raw($crate::Box::new(f)) as *mut _;
                    lvgl_sys::lv_obj_add_event_cb(
                        obj,
                        lvgl_sys::lv_event_cb_t::Some(
                            $crate::support::event_callback::<'a, Self, F>,
                        ),
                        lvgl_sys::lv_event_code_t_LV_EVENT_ALL,
                        obj.user_data,
                    );
                }
                Ok(())
            }
        }

        impl $crate::NativeObject for $item<'_> {
            fn raw(&self) -> core::ptr::NonNull<lvgl_sys::lv_obj_t> {
                self.core.raw()
            }
        }

        impl<'a> $crate::Widget<'a> for $item<'a> {
            type SpecialEvent = $event_type;
            type Part = $part_type;

            unsafe fn from_raw(
                raw_pointer: core::ptr::NonNull<lvgl_sys::lv_obj_t>,
            ) -> Option<Self> {
                Some(Self {
                    core: $crate::Obj::from_raw(raw_pointer).unwrap(),
                })
            }
        }
    };
}

// define_object!(Rafael);
//
// impl Rafael {
//     pub fn create(
//         parent: &mut impl crate::NativeObject,
//         copy: Option<&Rafael>,
//     ) -> crate::LvResult<Self> {
//         unsafe {
//             let ptr = lvgl_sys::lv_arc_create(
//                 parent.raw()?.as_mut(),
//                 copy.map(|c| c.raw().unwrap().as_mut() as *mut lvgl_sys::lv_obj_t)
//                     .unwrap_or(core::ptr::null_mut() as *mut lvgl_sys::lv_obj_t),
//             );
//             if let Some(raw) = core::ptr::NonNull::new(ptr) {
//                 let core = <crate::Obj as crate::Widget>::from_raw(raw);
//                 Ok(Self { core })
//             } else {
//                 Err(crate::LvError::InvalidReference)
//             }
//         }
//     }
//
//     pub fn create_at(parent: &mut impl crate::NativeObject) -> crate::LvResult<Self> {
//         Ok(Self::create(parent, None)?)
//     }
//
//     pub fn new() -> crate::LvResult<Self> {
//         let mut parent = crate::display::get_scr_act()?;
//         Ok(Self::create_at(&mut parent)?)
//     }
// }

pub enum Part {
    Main,
    Scrollbar,
    Indicator,
    Knob,
    Selected,
    Items,
    Ticks,
    Cursor,
    CustomFirst,
    Any,
}

impl Default for Part {
    fn default() -> Self {
        Self::Main
    }
}

impl From<Part> for lvgl_sys::lv_part_t {
    fn from(self_: Part) -> lvgl_sys::lv_part_t {
        match self_ {
            Part::Main => lvgl_sys::LV_PART_MAIN,
            Part::Scrollbar => lvgl_sys::LV_PART_SCROLLBAR,
            Part::Indicator => lvgl_sys::LV_PART_INDICATOR,
            Part::Knob => lvgl_sys::LV_PART_KNOB,
            Part::Selected => lvgl_sys::LV_PART_SELECTED,
            Part::Items => lvgl_sys::LV_PART_ITEMS,
            Part::Ticks => lvgl_sys::LV_PART_TICKS,
            Part::Cursor => lvgl_sys::LV_PART_CURSOR,
            Part::CustomFirst => lvgl_sys::LV_PART_CUSTOM_FIRST,
            Part::Any => lvgl_sys::LV_PART_ANY,
        }
    }
}

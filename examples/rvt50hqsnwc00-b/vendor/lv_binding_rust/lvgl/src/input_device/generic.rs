use super::encoder::*;
use super::pointer::*;
use crate::LvResult;

/// Generic data which can be associated with an input device driver. Varies
/// based on the concrete type of the input device driver
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Data {
    /// Pointer-specific data.
    Pointer(PointerInputData),
    /// Encoder-specific data.
    Encoder(EncoderInputData),
}

/// Boolean states for an input.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum InputState {
    /// Input device key is currently pressed down.
    Pressed(Data),
    /// Input device key is currently released.
    Released(Data),
}

impl InputState {
    /// Represents an input device with one entry in the buffer.
    pub fn once(self) -> BufferStatus {
        BufferStatus::Once(self)
    }
    /// Represents an input device with multiple entries in the buffer.
    pub fn and_continued(self) -> BufferStatus {
        BufferStatus::Buffered(self)
    }
}

/// Boolean buffering states for an input device driver.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum BufferStatus {
    /// One instance of `InputState` remains to be read.
    Once(InputState),
    /// Multiple instances of `InputState` remain to be read.
    Buffered(InputState),
}

/// A generic input driver trait.
pub trait InputDriver<D> {
    /// Creates an instance of a given input device, given a handler function.
    /// A `Display` must already have been created.
    fn register<F>(handler: F, display: &crate::Display) -> LvResult<D>
    where
        F: Fn() -> BufferStatus;

    /// Returns a pointer to the underlying raw driver.
    fn get_driver(&mut self) -> &mut lvgl_sys::lv_indev_drv_t;

    /// Returns a pointer to the descriptor.
    fn get_descriptor(&mut self) -> Option<&mut lvgl_sys::lv_indev_t>;

    /// Creates a new `InputDriver` from raw parts.
    ///
    /// # Safety
    ///
    /// The provided functions must not themselves cause undefined behavior
    /// when called by LVGL.
    unsafe fn new_raw(
        read_cb: Option<
            unsafe extern "C" fn(*mut lvgl_sys::lv_indev_drv_t, *mut lvgl_sys::lv_indev_data_t),
        >,
        feedback_cb: Option<unsafe extern "C" fn(*mut lvgl_sys::lv_indev_drv_t, u8)>,
        display: &crate::Display,
    ) -> LvResult<D>;

    /// Sets the descriptor for the input driver wrapper.
    ///
    /// # Safety
    ///
    /// `descriptor` must point to an initialized but unregistered and unused
    /// instance of an `lv_indev_t`, and must also be aligned.
    unsafe fn set_descriptor(&mut self, descriptor: *mut lvgl_sys::lv_indev_t) -> LvResult<()>;
}

use crate::display::{Display, DisplayDriver};
use crate::input_device::InputDriver;
use crate::{Event, LvError, LvResult, Obj, Widget};
use core::ptr::NonNull;
#[cfg(not(feature = "rust_timer"))]
use core::time::Duration;
use core::{ptr, result};

/// Internal LVGL error.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CoreError {
    ResourceNotAvailable,
    OperationFailed,
}

type Result<T> = result::Result<T, CoreError>;

/// Register own buffer
pub(crate) fn disp_drv_register<const N: usize>(
    disp_drv: &mut DisplayDriver<N>,
    drop: Option<unsafe extern "C" fn()>,
) -> Result<Display> {
    let disp_ptr = unsafe {
        let ptr = disp_drv.disp_drv.as_mut().get_mut() as *mut _;
        lvgl_sys::lv_disp_drv_register(ptr)
    };
    Ok(Display::from_raw(
        NonNull::new(disp_ptr).ok_or(CoreError::OperationFailed)?,
        drop,
    ))
}

pub(crate) fn disp_get_default() -> Result<Display> {
    let disp_ptr = unsafe { lvgl_sys::lv_disp_get_default() };
    Ok(Display::from_raw(
        NonNull::new(disp_ptr).ok_or(CoreError::OperationFailed)?,
        None,
    ))
}

pub(crate) fn get_str_act(disp: Option<&Display>) -> Result<Obj> {
    let scr_ptr = unsafe {
        lvgl_sys::lv_disp_get_scr_act(
            disp.map(|d| d.disp.as_ptr())
                .unwrap_or(ptr::null_mut() as *mut lvgl_sys::lv_disp_t),
        )
    };
    match unsafe { Obj::from_raw(NonNull::new(scr_ptr).ok_or(CoreError::ResourceNotAvailable)?) } {
        Some(o) => Ok(o),
        None => Err(CoreError::OperationFailed),
    }
}

/// Runs an LVGL tick lasting a given `core::time::Duration`. This function
/// should be called periodically.
#[inline]
#[cfg(not(any(feature = "rust_timer", feature = "custom_timer")))]
pub fn tick_inc(tick_period: Duration) {
    unsafe {
        lvgl_sys::lv_tick_inc(tick_period.as_millis() as u32);
    }
}

/// Calls the LVGL timer handler. This function should be called periodically.
#[inline]
pub fn task_handler() {
    unsafe { lvgl_sys::lv_timer_handler() };
}

/// Directly send an event to a specific widget.
#[inline]
pub fn event_send<W: for<'a> Widget<'a>>(
    obj: &mut W,
    event: Event<<W as Widget<'_>>::SpecialEvent>,
) {
    unsafe {
        lvgl_sys::lv_event_send(obj.raw().as_mut(), event.into(), ptr::null_mut());
    };
}

/// Register an input device driver to LVGL.
pub(crate) fn indev_drv_register<D>(input_device: &mut impl InputDriver<D>) -> LvResult<()> {
    unsafe {
        let descr = lvgl_sys::lv_indev_drv_register(input_device.get_driver() as *mut _);
        if descr.is_null() {
            return Err(LvError::LvOOMemory);
        }
        input_device.set_descriptor(descr)?;
    };
    Ok(())
}

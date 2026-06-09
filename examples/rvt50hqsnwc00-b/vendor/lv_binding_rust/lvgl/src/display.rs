use crate::functions::CoreError;
use crate::Screen;
use crate::{disp_drv_register, disp_get_default, get_str_act, NativeObject};
use crate::{Box, Color};
use core::convert::TryInto;
#[cfg(feature = "nightly")]
use core::error::Error;
use core::fmt;
use core::mem::{ManuallyDrop, MaybeUninit};
use core::pin::Pin;
use core::ptr::NonNull;
use core::{ptr, result};

/// Error in interacting with a `Display`.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DisplayError {
    NotAvailable,
    FailedToRegister,
    NotRegistered,
}

impl fmt::Display for DisplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Display {}",
            match self {
                DisplayError::NotAvailable => "not available",
                DisplayError::FailedToRegister => "failed to register",
                DisplayError::NotRegistered => "not registered",
            }
        )
    }
}

#[cfg(feature = "nightly")]
impl Error for DisplayError {}

type Result<T> = result::Result<T, DisplayError>;

/// An LVGL-registered display. Equivalent to an `lv_disp_t`.
pub struct Display {
    pub(crate) disp: NonNull<lvgl_sys::lv_disp_t>,
    drop: Option<unsafe extern "C" fn()>,
}

impl<'a> Display {
    pub(crate) fn from_raw(
        disp: NonNull<lvgl_sys::lv_disp_t>,
        drop: Option<unsafe extern "C" fn()>,
    ) -> Self {
        Self { disp, drop }
    }

    /// Registers a given `DrawBuffer` with an associated update function to
    /// LVGL. `display_update` takes a `&DisplayRefresh`.
    pub fn register<F, const N: usize>(
        draw_buffer: DrawBuffer<N>,
        hor_res: u32,
        ver_res: u32,
        display_update: F,
    ) -> Result<Self>
    where
        F: FnMut(&DisplayRefresh<N>) + 'a,
    {
        let mut display_diver = DisplayDriver::new(draw_buffer, display_update)?;
        let disp_p = &mut display_diver.disp_drv;
        disp_p.hor_res = hor_res.try_into().unwrap_or(240);
        disp_p.ver_res = ver_res.try_into().unwrap_or(240);
        Ok(disp_drv_register(&mut display_diver, None)?)
        //display_diver.disp_drv.leak();
    }

    /// Returns the current active screen.
    pub fn get_scr_act(&'a self) -> Result<Screen<'a>> {
        Ok(get_str_act(Some(self))?.try_into()?)
    }

    /// Sets a `Screen` as currently active.
    pub fn set_scr_act(&'a self, screen: &'a mut Screen) {
        let scr_ptr = unsafe { screen.raw().as_mut() };
        unsafe { lvgl_sys::lv_disp_load_scr(scr_ptr) }
    }

    /// Registers a display from raw functions and values.
    ///
    /// # Safety
    ///
    /// `hor_res` and `ver_res` must be nonzero, and the provided functions
    /// must not themselves cause undefined behavior.
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn register_raw<const N: usize>(
        draw_buffer: DrawBuffer<N>,
        hor_res: u32,
        ver_res: u32,
        flush_cb: Option<
            unsafe extern "C" fn(
                *mut lvgl_sys::lv_disp_drv_t,
                *const lvgl_sys::lv_area_t,
                *mut lvgl_sys::lv_color_t,
            ),
        >,
        rounder_cb: Option<
            unsafe extern "C" fn(*mut lvgl_sys::lv_disp_drv_t, *mut lvgl_sys::lv_area_t),
        >,
        set_px_cb: Option<
            unsafe extern "C" fn(
                *mut lvgl_sys::lv_disp_drv_t,
                *mut u8,
                lvgl_sys::lv_coord_t,
                lvgl_sys::lv_coord_t,
                lvgl_sys::lv_coord_t,
                lvgl_sys::lv_color_t,
                lvgl_sys::lv_opa_t,
            ),
        >,
        clear_cb: Option<unsafe extern "C" fn(*mut lvgl_sys::lv_disp_drv_t, *mut u8, u32)>,
        monitor_cb: Option<unsafe extern "C" fn(*mut lvgl_sys::lv_disp_drv_t, u32, u32)>,
        wait_cb: Option<unsafe extern "C" fn(*mut lvgl_sys::lv_disp_drv_t)>,
        clean_dcache_cb: Option<unsafe extern "C" fn(*mut lvgl_sys::lv_disp_drv_t)>,
        drv_update_cb: Option<unsafe extern "C" fn(*mut lvgl_sys::lv_disp_drv_t)>,
        render_start_cb: Option<unsafe extern "C" fn(*mut lvgl_sys::lv_disp_drv_t)>,
        drop: Option<unsafe extern "C" fn()>,
    ) -> Result<Self> {
        let mut display_driver = DisplayDriver::new_raw(
            draw_buffer,
            flush_cb,
            rounder_cb,
            set_px_cb,
            clear_cb,
            monitor_cb,
            wait_cb,
            clean_dcache_cb,
            drv_update_cb,
            render_start_cb,
        )?;
        let disp_p = &mut display_driver.disp_drv;
        disp_p.hor_res = hor_res.try_into().unwrap_or(240);
        disp_p.ver_res = ver_res.try_into().unwrap_or(240);
        Ok(disp_drv_register(&mut display_driver, drop)?)
    }
}

impl Default for Display {
    fn default() -> Self {
        disp_get_default().expect("LVGL must be INITIALIZED")
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        if let Some(drop) = self.drop {
            unsafe { drop() }
        }
    }
}

/// Gets the active screen of the default display.
pub(crate) fn get_scr_act() -> Result<Screen<'static>> {
    Ok(get_str_act(None)?.try_into()?)
}

/// A buffer of size `N` representing `N` pixels. `N` can be smaller than the
/// entire number of pixels on the screen, in which case the screen will be
/// drawn to multiple times per frame.
pub struct DrawBuffer<const N: usize> {
    draw_buf: Pin<Box<lvgl_sys::lv_disp_draw_buf_t>>,
    _refresh_buffer: Pin<Box<[MaybeUninit<lvgl_sys::lv_color_t>; N]>>,
}

impl<const N: usize> Default for DrawBuffer<N> {
    fn default() -> Self {
        let mut buf = Box::pin([MaybeUninit::uninit(); N]);
        Self {
            draw_buf: Box::pin(unsafe {
                let mut inner: MaybeUninit<lvgl_sys::lv_disp_draw_buf_t> = MaybeUninit::uninit();
                let raw_ptr = buf.as_mut_ptr() as *mut _;
                lvgl_sys::lv_disp_draw_buf_init(
                    inner.as_mut_ptr(),
                    raw_ptr,
                    ptr::null_mut(),
                    N as u32,
                );
                inner.assume_init()
            }),
            _refresh_buffer: buf,
        }
    }
}

impl<const N: usize> DrawBuffer<N> {
    fn get_ptr(&mut self) -> &mut lvgl_sys::lv_disp_draw_buf_t {
        &mut self.draw_buf
    }
}

#[repr(C)]
pub(crate) struct DisplayDriver<const N: usize> {
    pub(crate) disp_drv: Pin<Box<lvgl_sys::lv_disp_drv_t>>,
    _buffer: DrawBuffer<N>,
}

impl<'a, const N: usize> DisplayDriver<N> {
    pub fn new<F>(
        mut draw_buffer: DrawBuffer<N>,
        display_update_callback: F,
    ) -> Result<ManuallyDrop<Self>>
    where
        F: FnMut(&DisplayRefresh<N>) + 'a,
    {
        let mut disp_drv = Box::pin(unsafe {
            let mut inner = MaybeUninit::uninit();
            lvgl_sys::lv_disp_drv_init(inner.as_mut_ptr());
            inner.assume_init()
        });

        // Safety: The variable `draw_buffer` is statically allocated, no need to worry about this being dropped.
        disp_drv.draw_buf = draw_buffer.get_ptr() as *mut _;

        disp_drv.user_data = Box::<F>::into_raw(Box::new(display_update_callback)) as *mut _;

        // Sets trampoline pointer to the function implementation that uses the `F` type for a
        // refresh buffer of size N specifically.
        disp_drv.flush_cb = Some(disp_flush_trampoline::<F, N>);

        // We do not store any memory that can be accidentally deallocated by on the Rust side.
        Ok(ManuallyDrop::new(Self {
            disp_drv,
            _buffer: draw_buffer,
        }))
    }

    #[allow(clippy::too_many_arguments)]
    pub unsafe fn new_raw(
        mut draw_buffer: DrawBuffer<N>,
        flush_cb: Option<
            unsafe extern "C" fn(
                *mut lvgl_sys::_lv_disp_drv_t,
                *const lvgl_sys::lv_area_t,
                *mut lvgl_sys::lv_color_t,
            ),
        >,
        rounder_cb: Option<
            unsafe extern "C" fn(*mut lvgl_sys::_lv_disp_drv_t, *mut lvgl_sys::lv_area_t),
        >,
        set_px_cb: Option<
            unsafe extern "C" fn(
                *mut lvgl_sys::_lv_disp_drv_t,
                *mut u8,
                lvgl_sys::lv_coord_t,
                lvgl_sys::lv_coord_t,
                lvgl_sys::lv_coord_t,
                lvgl_sys::lv_color_t,
                lvgl_sys::lv_opa_t,
            ),
        >,
        clear_cb: Option<unsafe extern "C" fn(*mut lvgl_sys::_lv_disp_drv_t, *mut u8, u32)>,
        monitor_cb: Option<unsafe extern "C" fn(*mut lvgl_sys::_lv_disp_drv_t, u32, u32)>,
        wait_cb: Option<unsafe extern "C" fn(*mut lvgl_sys::_lv_disp_drv_t)>,
        clean_dcache_cb: Option<unsafe extern "C" fn(*mut lvgl_sys::_lv_disp_drv_t)>,
        drv_update_cb: Option<unsafe extern "C" fn(*mut lvgl_sys::_lv_disp_drv_t)>,
        render_start_cb: Option<unsafe extern "C" fn(*mut lvgl_sys::_lv_disp_drv_t)>,
    ) -> Result<ManuallyDrop<Self>> {
        let mut disp_drv = Box::pin(unsafe {
            let mut inner = MaybeUninit::uninit();
            lvgl_sys::lv_disp_drv_init(inner.as_mut_ptr());
            inner.assume_init()
        });

        disp_drv.draw_buf = draw_buffer.get_ptr() as *mut _;

        //disp_drv.user_data = Box::into_raw(Box::new(display_update_callback)) as *mut _;

        disp_drv.flush_cb = flush_cb;
        disp_drv.rounder_cb = rounder_cb;
        disp_drv.set_px_cb = set_px_cb;
        disp_drv.clear_cb = clear_cb;
        disp_drv.monitor_cb = monitor_cb;
        disp_drv.wait_cb = wait_cb;
        disp_drv.clean_dcache_cb = clean_dcache_cb;
        disp_drv.drv_update_cb = drv_update_cb;
        disp_drv.render_start_cb = render_start_cb;

        Ok(ManuallyDrop::new(Self {
            disp_drv,
            _buffer: draw_buffer,
        }))
    }
}

/// Represents a sub-area of the display that is being updated.
pub struct Area {
    pub x1: i16,
    pub x2: i16,
    pub y1: i16,
    pub y2: i16,
}

/// An update to the display information, contains the area that is being
/// updated and the color of the pixels that need to be updated. The colors
/// are represented in a contiguous array.
pub struct DisplayRefresh<const N: usize> {
    pub area: Area,
    pub colors: [Color; N],
}

#[cfg(feature = "embedded_graphics")]
mod embedded_graphics_impl {
    use crate::{Color, DisplayRefresh};
    use embedded_graphics::prelude::*;
    use embedded_graphics::Pixel;

    impl<const N: usize> DisplayRefresh<N> {
        pub fn as_pixels<C>(&self) -> impl IntoIterator<Item = Pixel<C>> + '_
        where
            C: PixelColor + From<Color>,
        {
            let area = &self.area;
            let x1 = area.x1;
            let x2 = area.x2;
            let y1 = area.y1;
            let y2 = area.y2;

            let ys = y1..=y2;
            let xs = (x1..=x2).enumerate();
            let x_len = (x2 - x1 + 1) as usize;

            // We use iterators here to ensure that the Rust compiler can apply all possible
            // optimizations at compile time.
            ys.enumerate().flat_map(move |(iy, y)| {
                xs.clone().map(move |(ix, x)| {
                    let color_len = x_len * iy + ix;
                    let raw_color = self.colors[color_len];
                    Pixel(Point::new(x as i32, y as i32), raw_color.into())
                })
            })
        }
    }
}

unsafe extern "C" fn disp_flush_trampoline<'a, F, const N: usize>(
    disp_drv: *mut lvgl_sys::lv_disp_drv_t,
    area: *const lvgl_sys::lv_area_t,
    color_p: *mut lvgl_sys::lv_color_t,
) where
    F: FnMut(&DisplayRefresh<N>) + 'a,
{
    let display_driver = *disp_drv;
    if !display_driver.user_data.is_null() {
        let callback = &mut *(display_driver.user_data as *mut F);

        let mut colors = [Color::default(); N];
        for (color_len, color) in colors.iter_mut().enumerate() {
            let lv_color = *color_p.add(color_len);
            *color = Color::from_raw(lv_color);
        }

        let update = DisplayRefresh {
            area: Area {
                x1: (*area).x1,
                x2: (*area).x2,
                y1: (*area).y1,
                y2: (*area).y2,
            },
            colors,
        };
        callback(&update);
    }
    // Not doing this causes a segfault in rust >= 1.69.0
    *disp_drv = display_driver;
    // Indicate to LVGL that we are ready with the flushing
    lvgl_sys::lv_disp_flush_ready(disp_drv);
}

impl From<CoreError> for DisplayError {
    fn from(err: CoreError) -> Self {
        use DisplayError::*;
        match err {
            CoreError::ResourceNotAvailable => NotAvailable,
            CoreError::OperationFailed => NotAvailable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests;

    #[test]
    fn get_scr_act_return_display() {
        tests::initialize_test(true);
        let _screen = get_str_act(None).expect("We can get the active screen");
    }

    #[test]
    fn get_default_display() {
        tests::initialize_test(true);
        let display = Display::default();
        let _screen_direct = display
            .get_scr_act()
            .expect("Return screen directly from the display instance");
        let _screen_default = get_scr_act().expect("Return screen from the default display");
    }

    #[test]
    fn register_display_directly() -> Result<()> {
        crate::tests::initialize_test(true);
        let display = Display::default();
        let _screen = display
            .get_scr_act()
            .expect("Return screen directly from the display instance");
        Ok(())
    }
}

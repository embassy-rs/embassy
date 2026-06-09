use super::{BufferStatus, Data, InputDriver, InputState};
use crate::Box;
use crate::Point;
use crate::{LvError, LvResult};
use core::mem::MaybeUninit;

/// Pointer-specific input data. Contains the point clicked and the key.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum PointerInputData {
    Touch(Point),
    Key(u32),
}

impl PointerInputData {
    pub fn pressed(self) -> InputState {
        InputState::Pressed(Data::Pointer(self))
    }

    pub fn released(self) -> InputState {
        InputState::Released(Data::Pointer(self))
    }
}

/// Represents a pointer-type input driver.
pub struct Pointer {
    pub(crate) driver: Box<lvgl_sys::lv_indev_drv_t>,
    pub(crate) descriptor: Option<*mut lvgl_sys::lv_indev_t>,
}

impl InputDriver<Pointer> for Pointer {
    fn register<F>(handler: F, _: &crate::Display) -> LvResult<Self>
    where
        F: Fn() -> BufferStatus,
    {
        let driver = unsafe {
            let mut indev_drv = MaybeUninit::uninit();
            lvgl_sys::lv_indev_drv_init(indev_drv.as_mut_ptr());
            let mut indev_drv = Box::new(indev_drv.assume_init());
            indev_drv.type_ = lvgl_sys::lv_indev_type_t_LV_INDEV_TYPE_POINTER;
            indev_drv.read_cb = Some(read_input::<F>);
            indev_drv.feedback_cb = Some(feedback);
            indev_drv.user_data = Box::into_raw(Box::new(handler)) as *mut _;
            indev_drv
        };

        let mut dev = Self {
            driver,
            descriptor: None,
        };

        match crate::indev_drv_register(&mut dev) {
            Ok(()) => Ok(dev),
            Err(e) => Err(e),
        }
    }

    fn get_driver(&mut self) -> &mut lvgl_sys::lv_indev_drv_t {
        self.driver.as_mut()
    }

    fn get_descriptor(&mut self) -> Option<&mut lvgl_sys::lv_indev_t> {
        match self.descriptor {
            Some(d) => unsafe { d.as_mut() },
            None => None,
        }
    }

    unsafe fn new_raw(
        read_cb: Option<
            unsafe extern "C" fn(*mut lvgl_sys::_lv_indev_drv_t, *mut lvgl_sys::lv_indev_data_t),
        >,
        feedback_cb: Option<unsafe extern "C" fn(*mut lvgl_sys::_lv_indev_drv_t, u8)>,
        _: &crate::Display,
    ) -> LvResult<Self> {
        let driver = unsafe {
            let mut indev_drv = MaybeUninit::uninit();
            lvgl_sys::lv_indev_drv_init(indev_drv.as_mut_ptr());
            let mut indev_drv = Box::new(indev_drv.assume_init());
            indev_drv.type_ = lvgl_sys::lv_indev_type_t_LV_INDEV_TYPE_POINTER;
            indev_drv.read_cb = read_cb;
            indev_drv.feedback_cb = feedback_cb;
            indev_drv
        };

        let mut dev = Self {
            driver,
            descriptor: None,
        };

        match crate::indev_drv_register(&mut dev) {
            Ok(()) => Ok(dev),
            Err(e) => Err(e),
        }
    }

    unsafe fn set_descriptor(&mut self, descriptor: *mut lvgl_sys::lv_indev_t) -> LvResult<()> {
        if self.descriptor.is_none() {
            self.descriptor = Some(descriptor);
        } else {
            return Err(LvError::AlreadyInUse);
        }
        Ok(())
    }
}

unsafe extern "C" fn read_input<F>(
    indev_drv: *mut lvgl_sys::lv_indev_drv_t,
    data: *mut lvgl_sys::lv_indev_data_t,
) where
    F: Fn() -> BufferStatus,
{
    // convert user data to function
    let user_closure = &mut *((*indev_drv).user_data as *mut F);
    // call user data
    let info = user_closure();
    unsafe {
        (*data).continue_reading = match info {
            BufferStatus::Once(b) => {
                (*data).state = match b {
                    InputState::Pressed(d) => {
                        match d {
                            Data::Pointer(PointerInputData::Touch(point)) => {
                                (*data).point.x = point.x as lvgl_sys::lv_coord_t;
                                (*data).point.y = point.y as lvgl_sys::lv_coord_t;
                            }
                            Data::Pointer(PointerInputData::Key(_)) => {}
                            _ => panic!("Non-pointer data returned from pointer device!"),
                        }
                        lvgl_sys::lv_indev_state_t_LV_INDEV_STATE_PRESSED
                    }
                    InputState::Released(d) => {
                        match d {
                            Data::Pointer(PointerInputData::Touch(point)) => {
                                (*data).point.x = point.x as lvgl_sys::lv_coord_t;
                                (*data).point.y = point.y as lvgl_sys::lv_coord_t;
                            }
                            Data::Pointer(PointerInputData::Key(_)) => {}
                            _ => panic!("Non-pointer data returned from pointer device!"),
                        }
                        lvgl_sys::lv_indev_state_t_LV_INDEV_STATE_RELEASED
                    }
                };
                false
            }
            BufferStatus::Buffered(b) => {
                (*data).state = match b {
                    InputState::Pressed(d) => {
                        match d {
                            Data::Pointer(PointerInputData::Touch(point)) => {
                                (*data).point.x = point.x as lvgl_sys::lv_coord_t;
                                (*data).point.y = point.y as lvgl_sys::lv_coord_t;
                            }
                            Data::Pointer(PointerInputData::Key(_)) => {}
                            _ => panic!("Non-pointer data returned from pointer device!"),
                        }
                        lvgl_sys::lv_indev_state_t_LV_INDEV_STATE_PRESSED
                    }
                    InputState::Released(d) => {
                        match d {
                            Data::Pointer(PointerInputData::Touch(point)) => {
                                (*data).point.x = point.x as lvgl_sys::lv_coord_t;
                                (*data).point.y = point.y as lvgl_sys::lv_coord_t;
                            }
                            Data::Pointer(PointerInputData::Key(_)) => {}
                            _ => panic!("Non-pointer data returned from pointer device!"),
                        }
                        lvgl_sys::lv_indev_state_t_LV_INDEV_STATE_RELEASED
                    }
                };
                true
            }
        }
    }
}

unsafe extern "C" fn feedback(_indev_drv: *mut lvgl_sys::lv_indev_drv_t, _code: u8) {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Display;
    use core::marker::PhantomData;
    use embedded_graphics::draw_target::DrawTarget;
    use embedded_graphics::geometry::Size;
    use embedded_graphics::pixelcolor::PixelColor;
    use embedded_graphics::prelude::OriginDimensions;
    use embedded_graphics::Pixel;

    struct FakeDisplay<C>
    where
        C: PixelColor,
    {
        p: PhantomData<C>,
    }

    impl<C> DrawTarget for FakeDisplay<C>
    where
        C: PixelColor,
    {
        type Color = C;
        type Error = ();

        fn draw_iter<I>(&mut self, _pixels: I) -> Result<(), Self::Error>
        where
            I: IntoIterator<Item = Pixel<Self::Color>>,
        {
            Ok(())
        }
    }

    impl<C> OriginDimensions for FakeDisplay<C>
    where
        C: PixelColor,
    {
        fn size(&self) -> Size {
            Size::new(240, 240)
        }
    }

    #[test]
    fn pointer_input_device() {
        crate::tests::initialize_test(true);
        let display = Display::default();

        fn read_touchpad_device() -> BufferStatus {
            PointerInputData::Touch(Point::new(120, 23))
                .pressed()
                .once()
        }

        let _touch_screen = Pointer::register(read_touchpad_device, &display).unwrap();
    }
}

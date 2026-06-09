use crate::{LvError, LvResult, NativeObject, Obj, Part, Widget};

/// An LVGL screen.
#[derive(Debug)]
pub struct Screen<'a> {
    raw: Obj<'a>,
}

impl Screen<'_> {
    pub fn blank() -> LvResult<Self> {
        Ok(Self { raw: Obj::blank()? })
    }
}

impl NativeObject for Screen<'_> {
    fn raw(&self) -> core::ptr::NonNull<lvgl_sys::lv_obj_t> {
        self.raw.raw()
    }
}

impl<'a> Widget<'a> for Screen<'a> {
    type SpecialEvent = u32;
    type Part = Part;

    unsafe fn from_raw(raw: core::ptr::NonNull<lvgl_sys::lv_obj_t>) -> Option<Self> {
        match Self::try_from(Obj::from_raw(raw)?) {
            Ok(s) => Some(s),
            Err(_) => None,
        }
    }
}

impl<'a> TryFrom<Obj<'a>> for Screen<'a> {
    type Error = LvError;

    fn try_from(value: Obj<'a>) -> Result<Self, Self::Error> {
        match unsafe { value.raw().as_mut().parent } as usize {
            0 => Ok(Self { raw: value }),
            _ => Err(LvError::InvalidReference),
        }
    }
}

#[allow(clippy::from_over_into)]
impl<'a> Into<Obj<'a>> for Screen<'a> {
    fn into(self) -> Obj<'a> {
        self.raw
    }
}

impl<'a> AsRef<Obj<'a>> for Screen<'a> {
    fn as_ref(&self) -> &Obj<'a> {
        &self.raw
    }
}

impl<'a> AsMut<Obj<'a>> for Screen<'a> {
    fn as_mut(&mut self) -> &mut Obj<'a> {
        &mut self.raw
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Display, DrawBuffer};

    #[test]
    fn screen_test() {
        const HOR_RES: u32 = 240;
        const VER_RES: u32 = 240;
        crate::tests::initialize_test(false);
        let buffer = DrawBuffer::<{ (HOR_RES * VER_RES) as usize }>::default();
        let display = Display::register(buffer, HOR_RES, VER_RES, |_| {}).unwrap();
        let mut screen_old = display.get_scr_act().unwrap();
        let mut screen_new = Screen::blank().unwrap();
        display.set_scr_act(&mut screen_new);
        display.set_scr_act(&mut screen_old);
    }
}

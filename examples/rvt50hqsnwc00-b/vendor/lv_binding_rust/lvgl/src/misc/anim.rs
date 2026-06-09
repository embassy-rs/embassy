use crate::{Box, LvResult, Obj, Widget};
use core::{
    mem::{self, MaybeUninit},
    num::TryFromIntError,
    ptr::NonNull,
    time::Duration,
};
use cty::c_void;

/// A repetition count for an animation, finite or infinite.
#[repr(u16)]
pub enum AnimRepeatCount {
    Finite(u16),
    Infinite,
}

/// An LVGL animation. Equivalent to an `lv_anim_t`.
pub struct Animation {
    pub(crate) raw: Box<lvgl_sys::lv_anim_t>,
}

impl Animation {
    /// Instantiates an `Animation` with the required attributes.
    pub fn new<'a, 'b, T, F>(
        target: &mut T,
        duration: Duration,
        start: i32,
        end: i32,
        animator: F,
    ) -> LvResult<Self>
    where
        T: Widget<'b>,
        F: FnMut(&mut Obj, i32) + 'a,
    {
        unsafe {
            let mut anim = Animation {
                raw: {
                    Box::new({
                        let mut inner: MaybeUninit<lvgl_sys::lv_anim_t> = MaybeUninit::uninit();
                        lvgl_sys::lv_anim_init(inner.as_mut_ptr());
                        inner.assume_init()
                    })
                },
            };

            anim.raw.time = duration.as_millis().try_into().unwrap_or(0);
            anim.raw.start_value = start;
            anim.raw.current_value = start;
            anim.raw.end_value = end;
            anim.raw.user_data = Box::<F>::into_raw(Box::new(animator)) as *mut _;
            anim.raw.var = target as *mut _ as *mut _;
            anim.raw.exec_cb = Some(animator_trampoline::<'a, 'b, T, F>);

            Ok(anim)
        }
    }

    /// Starts the animation.
    pub fn start(&mut self) {
        unsafe {
            self.raw = Box::from_raw(lvgl_sys::lv_anim_start(self.raw.as_mut()));
        }
    }

    /// Sets the delay before starting the animation.
    pub fn set_delay(&mut self, delay: Duration) -> Result<(), TryFromIntError> {
        self.raw.act_time = -(delay.as_millis().try_into()?);
        Ok(())
    }

    /// Sets the delay before playback.
    pub fn set_playback_delay(&mut self, delay: Duration) -> Result<(), TryFromIntError> {
        self.raw.playback_delay = delay.as_millis().try_into()?;
        Ok(())
    }

    /// Sets the total playback time.
    pub fn set_playback_time(&mut self, time: Duration) -> Result<(), TryFromIntError> {
        self.raw.playback_time = time.as_millis().try_into()?;
        Ok(())
    }

    /// Sets the delay before repeating the animation.
    pub fn set_repeat_delay(&mut self, delay: Duration) -> Result<(), TryFromIntError> {
        self.raw.repeat_delay = delay.as_millis().try_into()?;
        Ok(())
    }

    /// Sets how many times the animation repeats.
    pub fn set_repeat_count(&mut self, count: AnimRepeatCount) {
        unsafe {
            self.raw.repeat_cnt = match count {
                AnimRepeatCount::Finite(c) => c,
                AnimRepeatCount::Infinite => lvgl_sys::LV_ANIM_REPEAT_INFINITE
                    .try_into()
                    .unwrap_unchecked(),
            }
        }
    }

    /// Sets whether changes apply immediately or on the next cycle.
    pub fn set_early_apply(&mut self, apply: bool) {
        (*self.raw).set_early_apply(apply as u8);
    }
}

unsafe extern "C" fn animator_trampoline<'a, 'b, T, F>(obj: *mut c_void, val: i32)
where
    T: Widget<'b>,
    F: FnMut(&mut Obj, i32) + 'a,
{
    unsafe {
        let anim =
            NonNull::new(lvgl_sys::lv_anim_get(obj, None) as *mut lvgl_sys::lv_anim_t).unwrap();
        // yes, we have to do it this way. Casting `obj` directly to `&mut Obj` segfaults
        let obj = (*(obj as *mut T)).raw();
        if !anim.as_ref().user_data.is_null() {
            let callback = &mut *(obj.as_ref().user_data as *mut F);
            let mut obj_nondrop = Obj::from_raw(obj).unwrap();
            callback(&mut obj_nondrop, val);
            mem::forget(obj_nondrop)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::widgets::Btn;
    use crate::Display;

    #[test]
    fn anim_test() {
        crate::tests::initialize_test(true);
        let display = Display::default();
        let mut screen = display.get_scr_act().unwrap();
        let mut btn = Btn::create(&mut screen).unwrap();
        let mut anim =
            Animation::new(&mut btn, Duration::from_millis(10), 0, 100, |_, _| {}).unwrap();
        anim.start();
    }
}

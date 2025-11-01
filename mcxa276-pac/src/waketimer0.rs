#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    wake_timer_ctrl: WakeTimerCtrl,
    _reserved1: [u8; 0x08],
    wake_timer_cnt: WakeTimerCnt,
}
impl RegisterBlock {
    #[doc = "0x00 - Wake Timer Control"]
    #[inline(always)]
    pub const fn wake_timer_ctrl(&self) -> &WakeTimerCtrl {
        &self.wake_timer_ctrl
    }
    #[doc = "0x0c - Wake Timer Counter"]
    #[inline(always)]
    pub const fn wake_timer_cnt(&self) -> &WakeTimerCnt {
        &self.wake_timer_cnt
    }
}
#[doc = "WAKE_TIMER_CTRL (rw) register accessor: Wake Timer Control\n\nYou can [`read`](crate::Reg::read) this register and get [`wake_timer_ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`wake_timer_ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@wake_timer_ctrl`] module"]
#[doc(alias = "WAKE_TIMER_CTRL")]
pub type WakeTimerCtrl = crate::Reg<wake_timer_ctrl::WakeTimerCtrlSpec>;
#[doc = "Wake Timer Control"]
pub mod wake_timer_ctrl;
#[doc = "WAKE_TIMER_CNT (rw) register accessor: Wake Timer Counter\n\nYou can [`read`](crate::Reg::read) this register and get [`wake_timer_cnt::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`wake_timer_cnt::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@wake_timer_cnt`] module"]
#[doc(alias = "WAKE_TIMER_CNT")]
pub type WakeTimerCnt = crate::Reg<wake_timer_cnt::WakeTimerCntSpec>;
#[doc = "Wake Timer Counter"]
pub mod wake_timer_cnt;

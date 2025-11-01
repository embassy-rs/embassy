#[doc = "Register `KEEP_ALIVE_WKCTRL_RSVD` reader"]
pub type R = crate::R<KeepAliveWkctrlRsvdSpec>;
#[doc = "Register `KEEP_ALIVE_WKCTRL_RSVD` writer"]
pub type W = crate::W<KeepAliveWkctrlRsvdSpec>;
impl core::fmt::Debug for R {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.bits())
    }
}
impl W {}
#[doc = "Reserved\n\nYou can [`read`](crate::Reg::read) this register and get [`keep_alive_wkctrl_rsvd::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`keep_alive_wkctrl_rsvd::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct KeepAliveWkctrlRsvdSpec;
impl crate::RegisterSpec for KeepAliveWkctrlRsvdSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`keep_alive_wkctrl_rsvd::R`](R) reader structure"]
impl crate::Readable for KeepAliveWkctrlRsvdSpec {}
#[doc = "`write(|w| ..)` method takes [`keep_alive_wkctrl_rsvd::W`](W) writer structure"]
impl crate::Writable for KeepAliveWkctrlRsvdSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets KEEP_ALIVE_WKCTRL_RSVD to value 0x01"]
impl crate::Resettable for KeepAliveWkctrlRsvdSpec {
    const RESET_VALUE: u8 = 0x01;
}

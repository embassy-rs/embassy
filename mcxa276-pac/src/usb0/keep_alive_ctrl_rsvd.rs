#[doc = "Register `KEEP_ALIVE_CTRL_RSVD` reader"]
pub type R = crate::R<KeepAliveCtrlRsvdSpec>;
#[doc = "Register `KEEP_ALIVE_CTRL_RSVD` writer"]
pub type W = crate::W<KeepAliveCtrlRsvdSpec>;
impl core::fmt::Debug for R {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.bits())
    }
}
impl W {}
#[doc = "Reserved\n\nYou can [`read`](crate::Reg::read) this register and get [`keep_alive_ctrl_rsvd::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`keep_alive_ctrl_rsvd::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct KeepAliveCtrlRsvdSpec;
impl crate::RegisterSpec for KeepAliveCtrlRsvdSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`keep_alive_ctrl_rsvd::R`](R) reader structure"]
impl crate::Readable for KeepAliveCtrlRsvdSpec {}
#[doc = "`write(|w| ..)` method takes [`keep_alive_ctrl_rsvd::W`](W) writer structure"]
impl crate::Writable for KeepAliveCtrlRsvdSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets KEEP_ALIVE_CTRL_RSVD to value 0x08"]
impl crate::Resettable for KeepAliveCtrlRsvdSpec {
    const RESET_VALUE: u8 = 0x08;
}

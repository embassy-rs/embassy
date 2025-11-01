#[doc = "Register `WAKE_TIMER_CNT` reader"]
pub type R = crate::R<WakeTimerCntSpec>;
#[doc = "Register `WAKE_TIMER_CNT` writer"]
pub type W = crate::W<WakeTimerCntSpec>;
#[doc = "Field `WAKE_CNT` reader - Wake Counter"]
pub type WakeCntR = crate::FieldReader<u32>;
#[doc = "Field `WAKE_CNT` writer - Wake Counter"]
pub type WakeCntW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Wake Counter"]
    #[inline(always)]
    pub fn wake_cnt(&self) -> WakeCntR {
        WakeCntR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Wake Counter"]
    #[inline(always)]
    pub fn wake_cnt(&mut self) -> WakeCntW<WakeTimerCntSpec> {
        WakeCntW::new(self, 0)
    }
}
#[doc = "Wake Timer Counter\n\nYou can [`read`](crate::Reg::read) this register and get [`wake_timer_cnt::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`wake_timer_cnt::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct WakeTimerCntSpec;
impl crate::RegisterSpec for WakeTimerCntSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`wake_timer_cnt::R`](R) reader structure"]
impl crate::Readable for WakeTimerCntSpec {}
#[doc = "`write(|w| ..)` method takes [`wake_timer_cnt::W`](W) writer structure"]
impl crate::Writable for WakeTimerCntSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets WAKE_TIMER_CNT to value 0"]
impl crate::Resettable for WakeTimerCntSpec {}

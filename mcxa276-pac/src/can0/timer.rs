#[doc = "Register `TIMER` reader"]
pub type R = crate::R<TimerSpec>;
#[doc = "Register `TIMER` writer"]
pub type W = crate::W<TimerSpec>;
#[doc = "Field `TIMER` reader - Timer Value"]
pub type TimerR = crate::FieldReader<u16>;
#[doc = "Field `TIMER` writer - Timer Value"]
pub type TimerW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - Timer Value"]
    #[inline(always)]
    pub fn timer(&self) -> TimerR {
        TimerR::new((self.bits & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:15 - Timer Value"]
    #[inline(always)]
    pub fn timer(&mut self) -> TimerW<TimerSpec> {
        TimerW::new(self, 0)
    }
}
#[doc = "Free-Running Timer\n\nYou can [`read`](crate::Reg::read) this register and get [`timer::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`timer::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TimerSpec;
impl crate::RegisterSpec for TimerSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`timer::R`](R) reader structure"]
impl crate::Readable for TimerSpec {}
#[doc = "`write(|w| ..)` method takes [`timer::W`](W) writer structure"]
impl crate::Writable for TimerSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TIMER to value 0"]
impl crate::Resettable for TimerSpec {}

#[doc = "Register `TIMIEN` reader"]
pub type R = crate::R<TimienSpec>;
#[doc = "Register `TIMIEN` writer"]
pub type W = crate::W<TimienSpec>;
#[doc = "Field `TEIE` reader - Timer Status Interrupt Enable"]
pub type TeieR = crate::FieldReader;
#[doc = "Field `TEIE` writer - Timer Status Interrupt Enable"]
pub type TeieW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:3 - Timer Status Interrupt Enable"]
    #[inline(always)]
    pub fn teie(&self) -> TeieR {
        TeieR::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - Timer Status Interrupt Enable"]
    #[inline(always)]
    pub fn teie(&mut self) -> TeieW<TimienSpec> {
        TeieW::new(self, 0)
    }
}
#[doc = "Timer Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`timien::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`timien::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TimienSpec;
impl crate::RegisterSpec for TimienSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`timien::R`](R) reader structure"]
impl crate::Readable for TimienSpec {}
#[doc = "`write(|w| ..)` method takes [`timien::W`](W) writer structure"]
impl crate::Writable for TimienSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TIMIEN to value 0"]
impl crate::Resettable for TimienSpec {}

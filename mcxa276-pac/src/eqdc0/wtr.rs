#[doc = "Register `WTR` reader"]
pub type R = crate::R<WtrSpec>;
#[doc = "Register `WTR` writer"]
pub type W = crate::W<WtrSpec>;
#[doc = "Field `WDOG` reader - WDOG"]
pub type WdogR = crate::FieldReader<u16>;
#[doc = "Field `WDOG` writer - WDOG"]
pub type WdogW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - WDOG"]
    #[inline(always)]
    pub fn wdog(&self) -> WdogR {
        WdogR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:15 - WDOG"]
    #[inline(always)]
    pub fn wdog(&mut self) -> WdogW<WtrSpec> {
        WdogW::new(self, 0)
    }
}
#[doc = "Watchdog Timeout Register\n\nYou can [`read`](crate::Reg::read) this register and get [`wtr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`wtr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct WtrSpec;
impl crate::RegisterSpec for WtrSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`wtr::R`](R) reader structure"]
impl crate::Readable for WtrSpec {}
#[doc = "`write(|w| ..)` method takes [`wtr::W`](W) writer structure"]
impl crate::Writable for WtrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets WTR to value 0"]
impl crate::Resettable for WtrSpec {}

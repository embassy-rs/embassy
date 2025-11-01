#[doc = "Register `TSR` reader"]
pub type R = crate::R<TsrSpec>;
#[doc = "Register `TSR` writer"]
pub type W = crate::W<TsrSpec>;
#[doc = "Field `TSR` reader - Time Seconds Register"]
pub type TsrR = crate::FieldReader<u32>;
#[doc = "Field `TSR` writer - Time Seconds Register"]
pub type TsrW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Time Seconds Register"]
    #[inline(always)]
    pub fn tsr(&self) -> TsrR {
        TsrR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Time Seconds Register"]
    #[inline(always)]
    pub fn tsr(&mut self) -> TsrW<TsrSpec> {
        TsrW::new(self, 0)
    }
}
#[doc = "RTC Time Seconds\n\nYou can [`read`](crate::Reg::read) this register and get [`tsr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tsr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TsrSpec;
impl crate::RegisterSpec for TsrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`tsr::R`](R) reader structure"]
impl crate::Readable for TsrSpec {}
#[doc = "`write(|w| ..)` method takes [`tsr::W`](W) writer structure"]
impl crate::Writable for TsrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TSR to value 0"]
impl crate::Resettable for TsrSpec {}

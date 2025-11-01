#[doc = "Register `PCR` reader"]
pub type R = crate::R<PcrSpec>;
#[doc = "Register `PCR` writer"]
pub type W = crate::W<PcrSpec>;
#[doc = "Field `PTG_NUM` reader - Periodic Trigger Number"]
pub type PtgNumR = crate::FieldReader<u16>;
#[doc = "Field `PTG_NUM` writer - Periodic Trigger Number"]
pub type PtgNumW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
#[doc = "Field `PTG_PERIOD` reader - Periodic Trigger Period Width"]
pub type PtgPeriodR = crate::FieldReader<u16>;
#[doc = "Field `PTG_PERIOD` writer - Periodic Trigger Period Width"]
pub type PtgPeriodW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - Periodic Trigger Number"]
    #[inline(always)]
    pub fn ptg_num(&self) -> PtgNumR {
        PtgNumR::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bits 16:31 - Periodic Trigger Period Width"]
    #[inline(always)]
    pub fn ptg_period(&self) -> PtgPeriodR {
        PtgPeriodR::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:15 - Periodic Trigger Number"]
    #[inline(always)]
    pub fn ptg_num(&mut self) -> PtgNumW<PcrSpec> {
        PtgNumW::new(self, 0)
    }
    #[doc = "Bits 16:31 - Periodic Trigger Period Width"]
    #[inline(always)]
    pub fn ptg_period(&mut self) -> PtgPeriodW<PcrSpec> {
        PtgPeriodW::new(self, 16)
    }
}
#[doc = "Periodic Trigger Control\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PcrSpec;
impl crate::RegisterSpec for PcrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pcr::R`](R) reader structure"]
impl crate::Readable for PcrSpec {}
#[doc = "`write(|w| ..)` method takes [`pcr::W`](W) writer structure"]
impl crate::Writable for PcrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PCR to value 0"]
impl crate::Resettable for PcrSpec {}

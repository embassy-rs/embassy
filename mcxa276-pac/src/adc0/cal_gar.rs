#[doc = "Register `CAL_GAR[%s]` reader"]
pub type R = crate::R<CalGarSpec>;
#[doc = "Register `CAL_GAR[%s]` writer"]
pub type W = crate::W<CalGarSpec>;
#[doc = "Field `CAL_GAR_VAL` reader - Calibration General A Side Register Element"]
pub type CalGarValR = crate::FieldReader<u16>;
#[doc = "Field `CAL_GAR_VAL` writer - Calibration General A Side Register Element"]
pub type CalGarValW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - Calibration General A Side Register Element"]
    #[inline(always)]
    pub fn cal_gar_val(&self) -> CalGarValR {
        CalGarValR::new((self.bits & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:15 - Calibration General A Side Register Element"]
    #[inline(always)]
    pub fn cal_gar_val(&mut self) -> CalGarValW<CalGarSpec> {
        CalGarValW::new(self, 0)
    }
}
#[doc = "Calibration General A-Side Registers\n\nYou can [`read`](crate::Reg::read) this register and get [`cal_gar::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cal_gar::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CalGarSpec;
impl crate::RegisterSpec for CalGarSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`cal_gar::R`](R) reader structure"]
impl crate::Readable for CalGarSpec {}
#[doc = "`write(|w| ..)` method takes [`cal_gar::W`](W) writer structure"]
impl crate::Writable for CalGarSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CAL_GAR[%s] to value 0"]
impl crate::Resettable for CalGarSpec {}

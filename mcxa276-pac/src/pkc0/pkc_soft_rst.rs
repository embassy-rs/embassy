#[doc = "Register `PKC_SOFT_RST` writer"]
pub type W = crate::W<PkcSoftRstSpec>;
#[doc = "Field `SOFT_RST` writer - Write 1 to reset module (0 has no effect)"]
pub type SoftRstW<'a, REG> = crate::BitWriter<'a, REG>;
impl W {
    #[doc = "Bit 0 - Write 1 to reset module (0 has no effect)"]
    #[inline(always)]
    pub fn soft_rst(&mut self) -> SoftRstW<PkcSoftRstSpec> {
        SoftRstW::new(self, 0)
    }
}
#[doc = "Software reset\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_soft_rst::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkcSoftRstSpec;
impl crate::RegisterSpec for PkcSoftRstSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`pkc_soft_rst::W`](W) writer structure"]
impl crate::Writable for PkcSoftRstSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PKC_SOFT_RST to value 0"]
impl crate::Resettable for PkcSoftRstSpec {}

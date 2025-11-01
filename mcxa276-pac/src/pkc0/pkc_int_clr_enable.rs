#[doc = "Register `PKC_INT_CLR_ENABLE` writer"]
pub type W = crate::W<PkcIntClrEnableSpec>;
#[doc = "Field `EN_PDONE` writer - Write to clear PDONE interrupt enable flag (PKC_INT_ENABLE\\[EN_PDONE\\]=0)."]
pub type EnPdoneW<'a, REG> = crate::BitWriter<'a, REG>;
impl W {
    #[doc = "Bit 0 - Write to clear PDONE interrupt enable flag (PKC_INT_ENABLE\\[EN_PDONE\\]=0)."]
    #[inline(always)]
    pub fn en_pdone(&mut self) -> EnPdoneW<PkcIntClrEnableSpec> {
        EnPdoneW::new(self, 0)
    }
}
#[doc = "Interrupt enable clear\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_int_clr_enable::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkcIntClrEnableSpec;
impl crate::RegisterSpec for PkcIntClrEnableSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`pkc_int_clr_enable::W`](W) writer structure"]
impl crate::Writable for PkcIntClrEnableSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PKC_INT_CLR_ENABLE to value 0"]
impl crate::Resettable for PkcIntClrEnableSpec {}

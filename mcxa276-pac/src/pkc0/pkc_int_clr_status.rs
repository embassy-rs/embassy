#[doc = "Register `PKC_INT_CLR_STATUS` writer"]
pub type W = crate::W<PkcIntClrStatusSpec>;
#[doc = "Field `INT_PDONE` writer - Write to clear End-of-computation status flag (PKC_INT_STATUS\\[INT_PDONE\\]=0)."]
pub type IntPdoneW<'a, REG> = crate::BitWriter<'a, REG>;
impl W {
    #[doc = "Bit 0 - Write to clear End-of-computation status flag (PKC_INT_STATUS\\[INT_PDONE\\]=0)."]
    #[inline(always)]
    pub fn int_pdone(&mut self) -> IntPdoneW<PkcIntClrStatusSpec> {
        IntPdoneW::new(self, 0)
    }
}
#[doc = "Interrupt status clear\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_int_clr_status::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkcIntClrStatusSpec;
impl crate::RegisterSpec for PkcIntClrStatusSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`pkc_int_clr_status::W`](W) writer structure"]
impl crate::Writable for PkcIntClrStatusSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PKC_INT_CLR_STATUS to value 0"]
impl crate::Resettable for PkcIntClrStatusSpec {}

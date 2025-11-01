#[doc = "Register `PKC_INT_SET_STATUS` writer"]
pub type W = crate::W<PkcIntSetStatusSpec>;
#[doc = "Field `INT_PDONE` writer - Write to set End-of-computation status flag (PKC_INT_STATUS\\[INT_PDONE\\]=1) to trigger a PKC interrupt via software, e"]
pub type IntPdoneW<'a, REG> = crate::BitWriter<'a, REG>;
impl W {
    #[doc = "Bit 0 - Write to set End-of-computation status flag (PKC_INT_STATUS\\[INT_PDONE\\]=1) to trigger a PKC interrupt via software, e"]
    #[inline(always)]
    pub fn int_pdone(&mut self) -> IntPdoneW<PkcIntSetStatusSpec> {
        IntPdoneW::new(self, 0)
    }
}
#[doc = "Interrupt status set\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_int_set_status::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkcIntSetStatusSpec;
impl crate::RegisterSpec for PkcIntSetStatusSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`pkc_int_set_status::W`](W) writer structure"]
impl crate::Writable for PkcIntSetStatusSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PKC_INT_SET_STATUS to value 0"]
impl crate::Resettable for PkcIntSetStatusSpec {}

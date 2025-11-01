#[doc = "Register `PKC_INT_SET_ENABLE` writer"]
pub type W = crate::W<PkcIntSetEnableSpec>;
#[doc = "Field `EN_PDONE` writer - Write to set PDONE interrupt enable flag (PKC_INT_ENABLE\\[EN_PDONE\\]=1)."]
pub type EnPdoneW<'a, REG> = crate::BitWriter<'a, REG>;
impl W {
    #[doc = "Bit 0 - Write to set PDONE interrupt enable flag (PKC_INT_ENABLE\\[EN_PDONE\\]=1)."]
    #[inline(always)]
    pub fn en_pdone(&mut self) -> EnPdoneW<PkcIntSetEnableSpec> {
        EnPdoneW::new(self, 0)
    }
}
#[doc = "Interrupt enable set\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_int_set_enable::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkcIntSetEnableSpec;
impl crate::RegisterSpec for PkcIntSetEnableSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`pkc_int_set_enable::W`](W) writer structure"]
impl crate::Writable for PkcIntSetEnableSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PKC_INT_SET_ENABLE to value 0"]
impl crate::Resettable for PkcIntSetEnableSpec {}

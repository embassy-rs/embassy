#[doc = "Register `PKC_ACCESS_ERR_CLR` writer"]
pub type W = crate::W<PkcAccessErrClrSpec>;
#[doc = "Field `ERR_CLR` writer - Write 1 to reset PKC_ACCESS_ERR SFR."]
pub type ErrClrW<'a, REG> = crate::BitWriter<'a, REG>;
impl W {
    #[doc = "Bit 0 - Write 1 to reset PKC_ACCESS_ERR SFR."]
    #[inline(always)]
    pub fn err_clr(&mut self) -> ErrClrW<PkcAccessErrClrSpec> {
        ErrClrW::new(self, 0)
    }
}
#[doc = "Clear Access Error\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_access_err_clr::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkcAccessErrClrSpec;
impl crate::RegisterSpec for PkcAccessErrClrSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`pkc_access_err_clr::W`](W) writer structure"]
impl crate::Writable for PkcAccessErrClrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PKC_ACCESS_ERR_CLR to value 0"]
impl crate::Resettable for PkcAccessErrClrSpec {}

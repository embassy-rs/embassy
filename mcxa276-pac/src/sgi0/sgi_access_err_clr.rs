#[doc = "Register `sgi_access_err_clr` reader"]
pub type R = crate::R<SgiAccessErrClrSpec>;
#[doc = "Register `sgi_access_err_clr` writer"]
pub type W = crate::W<SgiAccessErrClrSpec>;
#[doc = "Field `err_clr` reader - Write to reset SGI_ACCESS_ERR SFR."]
pub type ErrClrR = crate::BitReader;
#[doc = "Field `err_clr` writer - Write to reset SGI_ACCESS_ERR SFR."]
pub type ErrClrW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `accerrc_rsvd` reader - reserved"]
pub type AccerrcRsvdR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bit 0 - Write to reset SGI_ACCESS_ERR SFR."]
    #[inline(always)]
    pub fn err_clr(&self) -> ErrClrR {
        ErrClrR::new((self.bits & 1) != 0)
    }
    #[doc = "Bits 1:31 - reserved"]
    #[inline(always)]
    pub fn accerrc_rsvd(&self) -> AccerrcRsvdR {
        AccerrcRsvdR::new((self.bits >> 1) & 0x7fff_ffff)
    }
}
impl W {
    #[doc = "Bit 0 - Write to reset SGI_ACCESS_ERR SFR."]
    #[inline(always)]
    pub fn err_clr(&mut self) -> ErrClrW<SgiAccessErrClrSpec> {
        ErrClrW::new(self, 0)
    }
}
#[doc = "Clear Access Error\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_access_err_clr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_access_err_clr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiAccessErrClrSpec;
impl crate::RegisterSpec for SgiAccessErrClrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_access_err_clr::R`](R) reader structure"]
impl crate::Readable for SgiAccessErrClrSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_access_err_clr::W`](W) writer structure"]
impl crate::Writable for SgiAccessErrClrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_access_err_clr to value 0"]
impl crate::Resettable for SgiAccessErrClrSpec {}

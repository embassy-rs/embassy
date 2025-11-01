#[doc = "Register `sgi_access_err` reader"]
pub type R = crate::R<SgiAccessErrSpec>;
#[doc = "Register `sgi_access_err` writer"]
pub type W = crate::W<SgiAccessErrSpec>;
#[doc = "Field `apb_notav` reader - APB Error: address not available"]
pub type ApbNotavR = crate::BitReader;
#[doc = "Field `apb_notav` writer - APB Error: address not available"]
pub type ApbNotavW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `apb_wrgmd` reader - APB Error: Wrong access mode"]
pub type ApbWrgmdR = crate::BitReader;
#[doc = "Field `apb_wrgmd` writer - APB Error: Wrong access mode"]
pub type ApbWrgmdW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `accerr_rsvd1` reader - reserved for future erors on SPB I/F"]
pub type AccerrRsvd1R = crate::FieldReader;
#[doc = "Field `apb_master` reader - APB Master that triggered first APB error"]
pub type ApbMasterR = crate::FieldReader;
#[doc = "Field `apb_master` writer - APB Master that triggered first APB error"]
pub type ApbMasterW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `accerr_rsvd2` reader - reserved for more block errors"]
pub type AccerrRsvd2R = crate::FieldReader<u32>;
impl R {
    #[doc = "Bit 0 - APB Error: address not available"]
    #[inline(always)]
    pub fn apb_notav(&self) -> ApbNotavR {
        ApbNotavR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - APB Error: Wrong access mode"]
    #[inline(always)]
    pub fn apb_wrgmd(&self) -> ApbWrgmdR {
        ApbWrgmdR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bits 2:3 - reserved for future erors on SPB I/F"]
    #[inline(always)]
    pub fn accerr_rsvd1(&self) -> AccerrRsvd1R {
        AccerrRsvd1R::new(((self.bits >> 2) & 3) as u8)
    }
    #[doc = "Bits 4:7 - APB Master that triggered first APB error"]
    #[inline(always)]
    pub fn apb_master(&self) -> ApbMasterR {
        ApbMasterR::new(((self.bits >> 4) & 0x0f) as u8)
    }
    #[doc = "Bits 8:31 - reserved for more block errors"]
    #[inline(always)]
    pub fn accerr_rsvd2(&self) -> AccerrRsvd2R {
        AccerrRsvd2R::new((self.bits >> 8) & 0x00ff_ffff)
    }
}
impl W {
    #[doc = "Bit 0 - APB Error: address not available"]
    #[inline(always)]
    pub fn apb_notav(&mut self) -> ApbNotavW<SgiAccessErrSpec> {
        ApbNotavW::new(self, 0)
    }
    #[doc = "Bit 1 - APB Error: Wrong access mode"]
    #[inline(always)]
    pub fn apb_wrgmd(&mut self) -> ApbWrgmdW<SgiAccessErrSpec> {
        ApbWrgmdW::new(self, 1)
    }
    #[doc = "Bits 4:7 - APB Master that triggered first APB error"]
    #[inline(always)]
    pub fn apb_master(&mut self) -> ApbMasterW<SgiAccessErrSpec> {
        ApbMasterW::new(self, 4)
    }
}
#[doc = "Access Error\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_access_err::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_access_err::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiAccessErrSpec;
impl crate::RegisterSpec for SgiAccessErrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_access_err::R`](R) reader structure"]
impl crate::Readable for SgiAccessErrSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_access_err::W`](W) writer structure"]
impl crate::Writable for SgiAccessErrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_access_err to value 0"]
impl crate::Resettable for SgiAccessErrSpec {}

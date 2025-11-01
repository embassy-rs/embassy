#[doc = "Register `sgi_sfr_sw_mask` reader"]
pub type R = crate::R<SgiSfrSwMaskSpec>;
#[doc = "Register `sgi_sfr_sw_mask` writer"]
pub type W = crate::W<SgiSfrSwMaskSpec>;
#[doc = "Field `sfr_mask_val` reader - Seed/mask used for sw level masking"]
pub type SfrMaskValR = crate::FieldReader<u32>;
#[doc = "Field `sfr_mask_val` writer - Seed/mask used for sw level masking"]
pub type SfrMaskValW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Seed/mask used for sw level masking"]
    #[inline(always)]
    pub fn sfr_mask_val(&self) -> SfrMaskValR {
        SfrMaskValR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Seed/mask used for sw level masking"]
    #[inline(always)]
    pub fn sfr_mask_val(&mut self) -> SfrMaskValW<SgiSfrSwMaskSpec> {
        SfrMaskValW::new(self, 0)
    }
}
#[doc = "Sofware Assisted Masking register .\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_sfr_sw_mask::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_sfr_sw_mask::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiSfrSwMaskSpec;
impl crate::RegisterSpec for SgiSfrSwMaskSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_sfr_sw_mask::R`](R) reader structure"]
impl crate::Readable for SgiSfrSwMaskSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_sfr_sw_mask::W`](W) writer structure"]
impl crate::Writable for SgiSfrSwMaskSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_sfr_sw_mask to value 0"]
impl crate::Resettable for SgiSfrSwMaskSpec {}

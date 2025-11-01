#[doc = "Register `sgi_sfrseed` reader"]
pub type R = crate::R<SgiSfrseedSpec>;
#[doc = "Register `sgi_sfrseed` writer"]
pub type W = crate::W<SgiSfrseedSpec>;
#[doc = "Field `sfrseed` reader - Seed/mask used for sw level masking"]
pub type SfrseedR = crate::FieldReader<u32>;
#[doc = "Field `sfrseed` writer - Seed/mask used for sw level masking"]
pub type SfrseedW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Seed/mask used for sw level masking"]
    #[inline(always)]
    pub fn sfrseed(&self) -> SfrseedR {
        SfrseedR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Seed/mask used for sw level masking"]
    #[inline(always)]
    pub fn sfrseed(&mut self) -> SfrseedW<SgiSfrseedSpec> {
        SfrseedW::new(self, 0)
    }
}
#[doc = "SFRSEED register for SFRMASK feature.\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_sfrseed::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_sfrseed::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiSfrseedSpec;
impl crate::RegisterSpec for SgiSfrseedSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_sfrseed::R`](R) reader structure"]
impl crate::Readable for SgiSfrseedSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_sfrseed::W`](W) writer structure"]
impl crate::Writable for SgiSfrseedSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_sfrseed to value 0"]
impl crate::Resettable for SgiSfrseedSpec {}

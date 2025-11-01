#[doc = "Register `sgi_datin2a` reader"]
pub type R = crate::R<SgiDatin2aSpec>;
#[doc = "Register `sgi_datin2a` writer"]
pub type W = crate::W<SgiDatin2aSpec>;
#[doc = "Field `datin2a` reader - Input Data register"]
pub type Datin2aR = crate::FieldReader<u32>;
#[doc = "Field `datin2a` writer - Input Data register"]
pub type Datin2aW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin2a(&self) -> Datin2aR {
        Datin2aR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin2a(&mut self) -> Datin2aW<SgiDatin2aSpec> {
        Datin2aW::new(self, 0)
    }
}
#[doc = "Input Data register 2 - Word-3\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin2a::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin2a::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiDatin2aSpec;
impl crate::RegisterSpec for SgiDatin2aSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_datin2a::R`](R) reader structure"]
impl crate::Readable for SgiDatin2aSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_datin2a::W`](W) writer structure"]
impl crate::Writable for SgiDatin2aSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_datin2a to value 0"]
impl crate::Resettable for SgiDatin2aSpec {}

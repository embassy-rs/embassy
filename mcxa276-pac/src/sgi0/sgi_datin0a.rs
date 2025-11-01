#[doc = "Register `sgi_datin0a` reader"]
pub type R = crate::R<SgiDatin0aSpec>;
#[doc = "Register `sgi_datin0a` writer"]
pub type W = crate::W<SgiDatin0aSpec>;
#[doc = "Field `datin0a` reader - Input Data register"]
pub type Datin0aR = crate::FieldReader<u32>;
#[doc = "Field `datin0a` writer - Input Data register"]
pub type Datin0aW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin0a(&self) -> Datin0aR {
        Datin0aR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin0a(&mut self) -> Datin0aW<SgiDatin0aSpec> {
        Datin0aW::new(self, 0)
    }
}
#[doc = "Input Data register 0 - Word-3\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin0a::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin0a::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiDatin0aSpec;
impl crate::RegisterSpec for SgiDatin0aSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_datin0a::R`](R) reader structure"]
impl crate::Readable for SgiDatin0aSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_datin0a::W`](W) writer structure"]
impl crate::Writable for SgiDatin0aSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_datin0a to value 0"]
impl crate::Resettable for SgiDatin0aSpec {}

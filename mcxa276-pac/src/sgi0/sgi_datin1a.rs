#[doc = "Register `sgi_datin1a` reader"]
pub type R = crate::R<SgiDatin1aSpec>;
#[doc = "Register `sgi_datin1a` writer"]
pub type W = crate::W<SgiDatin1aSpec>;
#[doc = "Field `datin1a` reader - Input Data register"]
pub type Datin1aR = crate::FieldReader<u32>;
#[doc = "Field `datin1a` writer - Input Data register"]
pub type Datin1aW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin1a(&self) -> Datin1aR {
        Datin1aR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin1a(&mut self) -> Datin1aW<SgiDatin1aSpec> {
        Datin1aW::new(self, 0)
    }
}
#[doc = "Input Data register 1 - Word-3\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin1a::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin1a::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiDatin1aSpec;
impl crate::RegisterSpec for SgiDatin1aSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_datin1a::R`](R) reader structure"]
impl crate::Readable for SgiDatin1aSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_datin1a::W`](W) writer structure"]
impl crate::Writable for SgiDatin1aSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_datin1a to value 0"]
impl crate::Resettable for SgiDatin1aSpec {}

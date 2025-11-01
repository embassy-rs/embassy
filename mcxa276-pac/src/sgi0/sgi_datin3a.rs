#[doc = "Register `sgi_datin3a` reader"]
pub type R = crate::R<SgiDatin3aSpec>;
#[doc = "Register `sgi_datin3a` writer"]
pub type W = crate::W<SgiDatin3aSpec>;
#[doc = "Field `datin3a` reader - Input Data register"]
pub type Datin3aR = crate::FieldReader<u32>;
#[doc = "Field `datin3a` writer - Input Data register"]
pub type Datin3aW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin3a(&self) -> Datin3aR {
        Datin3aR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin3a(&mut self) -> Datin3aW<SgiDatin3aSpec> {
        Datin3aW::new(self, 0)
    }
}
#[doc = "Input Data register 3 - Word-3\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin3a::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin3a::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiDatin3aSpec;
impl crate::RegisterSpec for SgiDatin3aSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_datin3a::R`](R) reader structure"]
impl crate::Readable for SgiDatin3aSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_datin3a::W`](W) writer structure"]
impl crate::Writable for SgiDatin3aSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_datin3a to value 0"]
impl crate::Resettable for SgiDatin3aSpec {}

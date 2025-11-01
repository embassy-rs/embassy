#[doc = "Register `sgi_datin3b` reader"]
pub type R = crate::R<SgiDatin3bSpec>;
#[doc = "Register `sgi_datin3b` writer"]
pub type W = crate::W<SgiDatin3bSpec>;
#[doc = "Field `datin3b` reader - Input Data register"]
pub type Datin3bR = crate::FieldReader<u32>;
#[doc = "Field `datin3b` writer - Input Data register"]
pub type Datin3bW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin3b(&self) -> Datin3bR {
        Datin3bR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin3b(&mut self) -> Datin3bW<SgiDatin3bSpec> {
        Datin3bW::new(self, 0)
    }
}
#[doc = "Input Data register 3 - Word-2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin3b::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin3b::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiDatin3bSpec;
impl crate::RegisterSpec for SgiDatin3bSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_datin3b::R`](R) reader structure"]
impl crate::Readable for SgiDatin3bSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_datin3b::W`](W) writer structure"]
impl crate::Writable for SgiDatin3bSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_datin3b to value 0"]
impl crate::Resettable for SgiDatin3bSpec {}

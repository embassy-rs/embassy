#[doc = "Register `sgi_datin2b` reader"]
pub type R = crate::R<SgiDatin2bSpec>;
#[doc = "Register `sgi_datin2b` writer"]
pub type W = crate::W<SgiDatin2bSpec>;
#[doc = "Field `datin2b` reader - Input Data register"]
pub type Datin2bR = crate::FieldReader<u32>;
#[doc = "Field `datin2b` writer - Input Data register"]
pub type Datin2bW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin2b(&self) -> Datin2bR {
        Datin2bR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin2b(&mut self) -> Datin2bW<SgiDatin2bSpec> {
        Datin2bW::new(self, 0)
    }
}
#[doc = "Input Data register 2 - Word-2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin2b::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin2b::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiDatin2bSpec;
impl crate::RegisterSpec for SgiDatin2bSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_datin2b::R`](R) reader structure"]
impl crate::Readable for SgiDatin2bSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_datin2b::W`](W) writer structure"]
impl crate::Writable for SgiDatin2bSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_datin2b to value 0"]
impl crate::Resettable for SgiDatin2bSpec {}

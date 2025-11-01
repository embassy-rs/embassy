#[doc = "Register `sgi_datin0b` reader"]
pub type R = crate::R<SgiDatin0bSpec>;
#[doc = "Register `sgi_datin0b` writer"]
pub type W = crate::W<SgiDatin0bSpec>;
#[doc = "Field `datin0b` reader - Input Data register"]
pub type Datin0bR = crate::FieldReader<u32>;
#[doc = "Field `datin0b` writer - Input Data register"]
pub type Datin0bW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin0b(&self) -> Datin0bR {
        Datin0bR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin0b(&mut self) -> Datin0bW<SgiDatin0bSpec> {
        Datin0bW::new(self, 0)
    }
}
#[doc = "Input Data register 0 - Word-2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin0b::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin0b::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiDatin0bSpec;
impl crate::RegisterSpec for SgiDatin0bSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_datin0b::R`](R) reader structure"]
impl crate::Readable for SgiDatin0bSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_datin0b::W`](W) writer structure"]
impl crate::Writable for SgiDatin0bSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_datin0b to value 0"]
impl crate::Resettable for SgiDatin0bSpec {}

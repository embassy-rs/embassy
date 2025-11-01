#[doc = "Register `sgi_datin1b` reader"]
pub type R = crate::R<SgiDatin1bSpec>;
#[doc = "Register `sgi_datin1b` writer"]
pub type W = crate::W<SgiDatin1bSpec>;
#[doc = "Field `datin1b` reader - Input Data register"]
pub type Datin1bR = crate::FieldReader<u32>;
#[doc = "Field `datin1b` writer - Input Data register"]
pub type Datin1bW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin1b(&self) -> Datin1bR {
        Datin1bR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin1b(&mut self) -> Datin1bW<SgiDatin1bSpec> {
        Datin1bW::new(self, 0)
    }
}
#[doc = "Input Data register 1 - Word-2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin1b::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin1b::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiDatin1bSpec;
impl crate::RegisterSpec for SgiDatin1bSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_datin1b::R`](R) reader structure"]
impl crate::Readable for SgiDatin1bSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_datin1b::W`](W) writer structure"]
impl crate::Writable for SgiDatin1bSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_datin1b to value 0"]
impl crate::Resettable for SgiDatin1bSpec {}

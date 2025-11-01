#[doc = "Register `sgi_datin3c` reader"]
pub type R = crate::R<SgiDatin3cSpec>;
#[doc = "Register `sgi_datin3c` writer"]
pub type W = crate::W<SgiDatin3cSpec>;
#[doc = "Field `datin3c` reader - Input Data register"]
pub type Datin3cR = crate::FieldReader<u32>;
#[doc = "Field `datin3c` writer - Input Data register"]
pub type Datin3cW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin3c(&self) -> Datin3cR {
        Datin3cR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin3c(&mut self) -> Datin3cW<SgiDatin3cSpec> {
        Datin3cW::new(self, 0)
    }
}
#[doc = "Input Data register 3 - Word-1\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin3c::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin3c::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiDatin3cSpec;
impl crate::RegisterSpec for SgiDatin3cSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_datin3c::R`](R) reader structure"]
impl crate::Readable for SgiDatin3cSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_datin3c::W`](W) writer structure"]
impl crate::Writable for SgiDatin3cSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_datin3c to value 0"]
impl crate::Resettable for SgiDatin3cSpec {}

#[doc = "Register `sgi_datin0c` reader"]
pub type R = crate::R<SgiDatin0cSpec>;
#[doc = "Register `sgi_datin0c` writer"]
pub type W = crate::W<SgiDatin0cSpec>;
#[doc = "Field `datin0c` reader - Input Data register"]
pub type Datin0cR = crate::FieldReader<u32>;
#[doc = "Field `datin0c` writer - Input Data register"]
pub type Datin0cW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin0c(&self) -> Datin0cR {
        Datin0cR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin0c(&mut self) -> Datin0cW<SgiDatin0cSpec> {
        Datin0cW::new(self, 0)
    }
}
#[doc = "Input Data register 0 - Word-1\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin0c::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin0c::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiDatin0cSpec;
impl crate::RegisterSpec for SgiDatin0cSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_datin0c::R`](R) reader structure"]
impl crate::Readable for SgiDatin0cSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_datin0c::W`](W) writer structure"]
impl crate::Writable for SgiDatin0cSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_datin0c to value 0"]
impl crate::Resettable for SgiDatin0cSpec {}

#[doc = "Register `sgi_datin0d` reader"]
pub type R = crate::R<SgiDatin0dSpec>;
#[doc = "Register `sgi_datin0d` writer"]
pub type W = crate::W<SgiDatin0dSpec>;
#[doc = "Field `datin0d` reader - Input Data register"]
pub type Datin0dR = crate::FieldReader<u32>;
#[doc = "Field `datin0d` writer - Input Data register"]
pub type Datin0dW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin0d(&self) -> Datin0dR {
        Datin0dR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin0d(&mut self) -> Datin0dW<SgiDatin0dSpec> {
        Datin0dW::new(self, 0)
    }
}
#[doc = "Input Data register 0 - Word-0\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin0d::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin0d::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiDatin0dSpec;
impl crate::RegisterSpec for SgiDatin0dSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_datin0d::R`](R) reader structure"]
impl crate::Readable for SgiDatin0dSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_datin0d::W`](W) writer structure"]
impl crate::Writable for SgiDatin0dSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_datin0d to value 0"]
impl crate::Resettable for SgiDatin0dSpec {}

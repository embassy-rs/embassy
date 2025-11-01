#[doc = "Register `sgi_datin2d` reader"]
pub type R = crate::R<SgiDatin2dSpec>;
#[doc = "Register `sgi_datin2d` writer"]
pub type W = crate::W<SgiDatin2dSpec>;
#[doc = "Field `datin2d` reader - Input Data register"]
pub type Datin2dR = crate::FieldReader<u32>;
#[doc = "Field `datin2d` writer - Input Data register"]
pub type Datin2dW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin2d(&self) -> Datin2dR {
        Datin2dR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin2d(&mut self) -> Datin2dW<SgiDatin2dSpec> {
        Datin2dW::new(self, 0)
    }
}
#[doc = "Input Data register 2 - Word-0\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin2d::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin2d::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiDatin2dSpec;
impl crate::RegisterSpec for SgiDatin2dSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_datin2d::R`](R) reader structure"]
impl crate::Readable for SgiDatin2dSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_datin2d::W`](W) writer structure"]
impl crate::Writable for SgiDatin2dSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_datin2d to value 0"]
impl crate::Resettable for SgiDatin2dSpec {}

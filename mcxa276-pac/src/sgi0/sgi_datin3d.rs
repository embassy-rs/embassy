#[doc = "Register `sgi_datin3d` reader"]
pub type R = crate::R<SgiDatin3dSpec>;
#[doc = "Register `sgi_datin3d` writer"]
pub type W = crate::W<SgiDatin3dSpec>;
#[doc = "Field `datin3d` reader - Input Data register"]
pub type Datin3dR = crate::FieldReader<u32>;
#[doc = "Field `datin3d` writer - Input Data register"]
pub type Datin3dW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin3d(&self) -> Datin3dR {
        Datin3dR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin3d(&mut self) -> Datin3dW<SgiDatin3dSpec> {
        Datin3dW::new(self, 0)
    }
}
#[doc = "Input Data register 3 - Word-0\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin3d::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin3d::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiDatin3dSpec;
impl crate::RegisterSpec for SgiDatin3dSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_datin3d::R`](R) reader structure"]
impl crate::Readable for SgiDatin3dSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_datin3d::W`](W) writer structure"]
impl crate::Writable for SgiDatin3dSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_datin3d to value 0"]
impl crate::Resettable for SgiDatin3dSpec {}

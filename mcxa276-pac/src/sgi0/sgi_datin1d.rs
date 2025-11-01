#[doc = "Register `sgi_datin1d` reader"]
pub type R = crate::R<SgiDatin1dSpec>;
#[doc = "Register `sgi_datin1d` writer"]
pub type W = crate::W<SgiDatin1dSpec>;
#[doc = "Field `datin1d` reader - Input Data register"]
pub type Datin1dR = crate::FieldReader<u32>;
#[doc = "Field `datin1d` writer - Input Data register"]
pub type Datin1dW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin1d(&self) -> Datin1dR {
        Datin1dR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin1d(&mut self) -> Datin1dW<SgiDatin1dSpec> {
        Datin1dW::new(self, 0)
    }
}
#[doc = "Input Data register 1 - Word-0\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin1d::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin1d::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiDatin1dSpec;
impl crate::RegisterSpec for SgiDatin1dSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_datin1d::R`](R) reader structure"]
impl crate::Readable for SgiDatin1dSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_datin1d::W`](W) writer structure"]
impl crate::Writable for SgiDatin1dSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_datin1d to value 0"]
impl crate::Resettable for SgiDatin1dSpec {}

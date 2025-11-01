#[doc = "Register `sgi_key1d` reader"]
pub type R = crate::R<SgiKey1dSpec>;
#[doc = "Register `sgi_key1d` writer"]
pub type W = crate::W<SgiKey1dSpec>;
#[doc = "Field `key1d` reader - Input Key register"]
pub type Key1dR = crate::FieldReader<u32>;
#[doc = "Field `key1d` writer - Input Key register"]
pub type Key1dW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key1d(&self) -> Key1dR {
        Key1dR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key1d(&mut self) -> Key1dW<SgiKey1dSpec> {
        Key1dW::new(self, 0)
    }
}
#[doc = "Input Key register 1 - Word-0\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key1d::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key1d::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey1dSpec;
impl crate::RegisterSpec for SgiKey1dSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key1d::R`](R) reader structure"]
impl crate::Readable for SgiKey1dSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key1d::W`](W) writer structure"]
impl crate::Writable for SgiKey1dSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key1d to value 0"]
impl crate::Resettable for SgiKey1dSpec {}

#[doc = "Register `sgi_key0d` reader"]
pub type R = crate::R<SgiKey0dSpec>;
#[doc = "Register `sgi_key0d` writer"]
pub type W = crate::W<SgiKey0dSpec>;
#[doc = "Field `key0d` reader - Input Key register"]
pub type Key0dR = crate::FieldReader<u32>;
#[doc = "Field `key0d` writer - Input Key register"]
pub type Key0dW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key0d(&self) -> Key0dR {
        Key0dR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key0d(&mut self) -> Key0dW<SgiKey0dSpec> {
        Key0dW::new(self, 0)
    }
}
#[doc = "Input Key register 0 - Word-0\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key0d::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key0d::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey0dSpec;
impl crate::RegisterSpec for SgiKey0dSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key0d::R`](R) reader structure"]
impl crate::Readable for SgiKey0dSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key0d::W`](W) writer structure"]
impl crate::Writable for SgiKey0dSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key0d to value 0"]
impl crate::Resettable for SgiKey0dSpec {}

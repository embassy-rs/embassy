#[doc = "Register `sgi_key2d` reader"]
pub type R = crate::R<SgiKey2dSpec>;
#[doc = "Register `sgi_key2d` writer"]
pub type W = crate::W<SgiKey2dSpec>;
#[doc = "Field `key2d` reader - Input Key register"]
pub type Key2dR = crate::FieldReader<u32>;
#[doc = "Field `key2d` writer - Input Key register"]
pub type Key2dW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key2d(&self) -> Key2dR {
        Key2dR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key2d(&mut self) -> Key2dW<SgiKey2dSpec> {
        Key2dW::new(self, 0)
    }
}
#[doc = "Input Key register 2 - Word-0\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key2d::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key2d::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey2dSpec;
impl crate::RegisterSpec for SgiKey2dSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key2d::R`](R) reader structure"]
impl crate::Readable for SgiKey2dSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key2d::W`](W) writer structure"]
impl crate::Writable for SgiKey2dSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key2d to value 0"]
impl crate::Resettable for SgiKey2dSpec {}

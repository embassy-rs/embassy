#[doc = "Register `sgi_key2a` reader"]
pub type R = crate::R<SgiKey2aSpec>;
#[doc = "Register `sgi_key2a` writer"]
pub type W = crate::W<SgiKey2aSpec>;
#[doc = "Field `key2a` reader - Input Key register"]
pub type Key2aR = crate::FieldReader<u32>;
#[doc = "Field `key2a` writer - Input Key register"]
pub type Key2aW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key2a(&self) -> Key2aR {
        Key2aR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key2a(&mut self) -> Key2aW<SgiKey2aSpec> {
        Key2aW::new(self, 0)
    }
}
#[doc = "Input Key register 2 - Word-3\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key2a::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key2a::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey2aSpec;
impl crate::RegisterSpec for SgiKey2aSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key2a::R`](R) reader structure"]
impl crate::Readable for SgiKey2aSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key2a::W`](W) writer structure"]
impl crate::Writable for SgiKey2aSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key2a to value 0"]
impl crate::Resettable for SgiKey2aSpec {}

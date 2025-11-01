#[doc = "Register `sgi_key0a` reader"]
pub type R = crate::R<SgiKey0aSpec>;
#[doc = "Register `sgi_key0a` writer"]
pub type W = crate::W<SgiKey0aSpec>;
#[doc = "Field `key0a` reader - Input Key register"]
pub type Key0aR = crate::FieldReader<u32>;
#[doc = "Field `key0a` writer - Input Key register"]
pub type Key0aW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key0a(&self) -> Key0aR {
        Key0aR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key0a(&mut self) -> Key0aW<SgiKey0aSpec> {
        Key0aW::new(self, 0)
    }
}
#[doc = "Input Key register 0 - Word-3\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key0a::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key0a::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey0aSpec;
impl crate::RegisterSpec for SgiKey0aSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key0a::R`](R) reader structure"]
impl crate::Readable for SgiKey0aSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key0a::W`](W) writer structure"]
impl crate::Writable for SgiKey0aSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key0a to value 0"]
impl crate::Resettable for SgiKey0aSpec {}

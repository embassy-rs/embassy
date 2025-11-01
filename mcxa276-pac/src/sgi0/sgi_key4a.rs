#[doc = "Register `sgi_key4a` reader"]
pub type R = crate::R<SgiKey4aSpec>;
#[doc = "Register `sgi_key4a` writer"]
pub type W = crate::W<SgiKey4aSpec>;
#[doc = "Field `key4a` reader - Input Key register"]
pub type Key4aR = crate::FieldReader<u32>;
#[doc = "Field `key4a` writer - Input Key register"]
pub type Key4aW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key4a(&self) -> Key4aR {
        Key4aR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key4a(&mut self) -> Key4aW<SgiKey4aSpec> {
        Key4aW::new(self, 0)
    }
}
#[doc = "Input Key register 4 - Word-3\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key4a::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key4a::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey4aSpec;
impl crate::RegisterSpec for SgiKey4aSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key4a::R`](R) reader structure"]
impl crate::Readable for SgiKey4aSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key4a::W`](W) writer structure"]
impl crate::Writable for SgiKey4aSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key4a to value 0"]
impl crate::Resettable for SgiKey4aSpec {}

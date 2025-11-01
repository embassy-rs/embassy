#[doc = "Register `sgi_key6a` reader"]
pub type R = crate::R<SgiKey6aSpec>;
#[doc = "Register `sgi_key6a` writer"]
pub type W = crate::W<SgiKey6aSpec>;
#[doc = "Field `key6a` reader - Input Key register"]
pub type Key6aR = crate::FieldReader<u32>;
#[doc = "Field `key6a` writer - Input Key register"]
pub type Key6aW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key6a(&self) -> Key6aR {
        Key6aR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key6a(&mut self) -> Key6aW<SgiKey6aSpec> {
        Key6aW::new(self, 0)
    }
}
#[doc = "Input Key register 6 - Word-3\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key6a::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key6a::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey6aSpec;
impl crate::RegisterSpec for SgiKey6aSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key6a::R`](R) reader structure"]
impl crate::Readable for SgiKey6aSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key6a::W`](W) writer structure"]
impl crate::Writable for SgiKey6aSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key6a to value 0"]
impl crate::Resettable for SgiKey6aSpec {}

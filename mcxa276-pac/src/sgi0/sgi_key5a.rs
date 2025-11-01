#[doc = "Register `sgi_key5a` reader"]
pub type R = crate::R<SgiKey5aSpec>;
#[doc = "Register `sgi_key5a` writer"]
pub type W = crate::W<SgiKey5aSpec>;
#[doc = "Field `key5a` reader - Input Key register"]
pub type Key5aR = crate::FieldReader<u32>;
#[doc = "Field `key5a` writer - Input Key register"]
pub type Key5aW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key5a(&self) -> Key5aR {
        Key5aR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key5a(&mut self) -> Key5aW<SgiKey5aSpec> {
        Key5aW::new(self, 0)
    }
}
#[doc = "Input Key register 5 - Word-3\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key5a::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key5a::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey5aSpec;
impl crate::RegisterSpec for SgiKey5aSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key5a::R`](R) reader structure"]
impl crate::Readable for SgiKey5aSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key5a::W`](W) writer structure"]
impl crate::Writable for SgiKey5aSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key5a to value 0"]
impl crate::Resettable for SgiKey5aSpec {}

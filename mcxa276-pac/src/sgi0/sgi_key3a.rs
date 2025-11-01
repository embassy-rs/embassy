#[doc = "Register `sgi_key3a` reader"]
pub type R = crate::R<SgiKey3aSpec>;
#[doc = "Register `sgi_key3a` writer"]
pub type W = crate::W<SgiKey3aSpec>;
#[doc = "Field `key3a` reader - Input Key register"]
pub type Key3aR = crate::FieldReader<u32>;
#[doc = "Field `key3a` writer - Input Key register"]
pub type Key3aW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key3a(&self) -> Key3aR {
        Key3aR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key3a(&mut self) -> Key3aW<SgiKey3aSpec> {
        Key3aW::new(self, 0)
    }
}
#[doc = "Input Key register 3 - Word-3\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key3a::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key3a::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey3aSpec;
impl crate::RegisterSpec for SgiKey3aSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key3a::R`](R) reader structure"]
impl crate::Readable for SgiKey3aSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key3a::W`](W) writer structure"]
impl crate::Writable for SgiKey3aSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key3a to value 0"]
impl crate::Resettable for SgiKey3aSpec {}

#[doc = "Register `sgi_key1c` reader"]
pub type R = crate::R<SgiKey1cSpec>;
#[doc = "Register `sgi_key1c` writer"]
pub type W = crate::W<SgiKey1cSpec>;
#[doc = "Field `key1c` reader - Input Key register"]
pub type Key1cR = crate::FieldReader<u32>;
#[doc = "Field `key1c` writer - Input Key register"]
pub type Key1cW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key1c(&self) -> Key1cR {
        Key1cR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key1c(&mut self) -> Key1cW<SgiKey1cSpec> {
        Key1cW::new(self, 0)
    }
}
#[doc = "Input Key register 1 - Word-1\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key1c::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key1c::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey1cSpec;
impl crate::RegisterSpec for SgiKey1cSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key1c::R`](R) reader structure"]
impl crate::Readable for SgiKey1cSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key1c::W`](W) writer structure"]
impl crate::Writable for SgiKey1cSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key1c to value 0"]
impl crate::Resettable for SgiKey1cSpec {}

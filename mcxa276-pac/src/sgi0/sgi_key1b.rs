#[doc = "Register `sgi_key1b` reader"]
pub type R = crate::R<SgiKey1bSpec>;
#[doc = "Register `sgi_key1b` writer"]
pub type W = crate::W<SgiKey1bSpec>;
#[doc = "Field `key1b` reader - Input Key register"]
pub type Key1bR = crate::FieldReader<u32>;
#[doc = "Field `key1b` writer - Input Key register"]
pub type Key1bW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key1b(&self) -> Key1bR {
        Key1bR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key1b(&mut self) -> Key1bW<SgiKey1bSpec> {
        Key1bW::new(self, 0)
    }
}
#[doc = "Input Key register 1 - Word-2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key1b::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key1b::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey1bSpec;
impl crate::RegisterSpec for SgiKey1bSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key1b::R`](R) reader structure"]
impl crate::Readable for SgiKey1bSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key1b::W`](W) writer structure"]
impl crate::Writable for SgiKey1bSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key1b to value 0"]
impl crate::Resettable for SgiKey1bSpec {}

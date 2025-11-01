#[doc = "Register `sgi_key0b` reader"]
pub type R = crate::R<SgiKey0bSpec>;
#[doc = "Register `sgi_key0b` writer"]
pub type W = crate::W<SgiKey0bSpec>;
#[doc = "Field `key0b` reader - Input Key register"]
pub type Key0bR = crate::FieldReader<u32>;
#[doc = "Field `key0b` writer - Input Key register"]
pub type Key0bW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key0b(&self) -> Key0bR {
        Key0bR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key0b(&mut self) -> Key0bW<SgiKey0bSpec> {
        Key0bW::new(self, 0)
    }
}
#[doc = "Input Key register 0 - Word-2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key0b::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key0b::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey0bSpec;
impl crate::RegisterSpec for SgiKey0bSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key0b::R`](R) reader structure"]
impl crate::Readable for SgiKey0bSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key0b::W`](W) writer structure"]
impl crate::Writable for SgiKey0bSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key0b to value 0"]
impl crate::Resettable for SgiKey0bSpec {}

#[doc = "Register `sgi_key3b` reader"]
pub type R = crate::R<SgiKey3bSpec>;
#[doc = "Register `sgi_key3b` writer"]
pub type W = crate::W<SgiKey3bSpec>;
#[doc = "Field `key3b` reader - Input Key register"]
pub type Key3bR = crate::FieldReader<u32>;
#[doc = "Field `key3b` writer - Input Key register"]
pub type Key3bW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key3b(&self) -> Key3bR {
        Key3bR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key3b(&mut self) -> Key3bW<SgiKey3bSpec> {
        Key3bW::new(self, 0)
    }
}
#[doc = "Input Key register 3 - Word-2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key3b::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key3b::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey3bSpec;
impl crate::RegisterSpec for SgiKey3bSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key3b::R`](R) reader structure"]
impl crate::Readable for SgiKey3bSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key3b::W`](W) writer structure"]
impl crate::Writable for SgiKey3bSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key3b to value 0"]
impl crate::Resettable for SgiKey3bSpec {}

#[doc = "Register `sgi_key2b` reader"]
pub type R = crate::R<SgiKey2bSpec>;
#[doc = "Register `sgi_key2b` writer"]
pub type W = crate::W<SgiKey2bSpec>;
#[doc = "Field `key2b` reader - Input Key register"]
pub type Key2bR = crate::FieldReader<u32>;
#[doc = "Field `key2b` writer - Input Key register"]
pub type Key2bW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key2b(&self) -> Key2bR {
        Key2bR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key2b(&mut self) -> Key2bW<SgiKey2bSpec> {
        Key2bW::new(self, 0)
    }
}
#[doc = "Input Key register 2 - Word-2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key2b::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key2b::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey2bSpec;
impl crate::RegisterSpec for SgiKey2bSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key2b::R`](R) reader structure"]
impl crate::Readable for SgiKey2bSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key2b::W`](W) writer structure"]
impl crate::Writable for SgiKey2bSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key2b to value 0"]
impl crate::Resettable for SgiKey2bSpec {}

#[doc = "Register `sgi_key4b` reader"]
pub type R = crate::R<SgiKey4bSpec>;
#[doc = "Register `sgi_key4b` writer"]
pub type W = crate::W<SgiKey4bSpec>;
#[doc = "Field `key4b` reader - Input Key register"]
pub type Key4bR = crate::FieldReader<u32>;
#[doc = "Field `key4b` writer - Input Key register"]
pub type Key4bW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key4b(&self) -> Key4bR {
        Key4bR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key4b(&mut self) -> Key4bW<SgiKey4bSpec> {
        Key4bW::new(self, 0)
    }
}
#[doc = "Input Key register 4 - Word-2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key4b::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key4b::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey4bSpec;
impl crate::RegisterSpec for SgiKey4bSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key4b::R`](R) reader structure"]
impl crate::Readable for SgiKey4bSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key4b::W`](W) writer structure"]
impl crate::Writable for SgiKey4bSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key4b to value 0"]
impl crate::Resettable for SgiKey4bSpec {}

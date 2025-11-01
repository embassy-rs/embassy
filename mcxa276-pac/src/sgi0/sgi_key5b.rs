#[doc = "Register `sgi_key5b` reader"]
pub type R = crate::R<SgiKey5bSpec>;
#[doc = "Register `sgi_key5b` writer"]
pub type W = crate::W<SgiKey5bSpec>;
#[doc = "Field `key5b` reader - Input Key register"]
pub type Key5bR = crate::FieldReader<u32>;
#[doc = "Field `key5b` writer - Input Key register"]
pub type Key5bW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key5b(&self) -> Key5bR {
        Key5bR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key5b(&mut self) -> Key5bW<SgiKey5bSpec> {
        Key5bW::new(self, 0)
    }
}
#[doc = "Input Key register 5 - Word-2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key5b::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key5b::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey5bSpec;
impl crate::RegisterSpec for SgiKey5bSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key5b::R`](R) reader structure"]
impl crate::Readable for SgiKey5bSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key5b::W`](W) writer structure"]
impl crate::Writable for SgiKey5bSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key5b to value 0"]
impl crate::Resettable for SgiKey5bSpec {}

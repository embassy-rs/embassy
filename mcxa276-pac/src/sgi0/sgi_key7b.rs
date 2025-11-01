#[doc = "Register `sgi_key7b` reader"]
pub type R = crate::R<SgiKey7bSpec>;
#[doc = "Register `sgi_key7b` writer"]
pub type W = crate::W<SgiKey7bSpec>;
#[doc = "Field `key7b` reader - Input Key register"]
pub type Key7bR = crate::FieldReader<u32>;
#[doc = "Field `key7b` writer - Input Key register"]
pub type Key7bW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key7b(&self) -> Key7bR {
        Key7bR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key7b(&mut self) -> Key7bW<SgiKey7bSpec> {
        Key7bW::new(self, 0)
    }
}
#[doc = "Input Key register 7 - Word-2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key7b::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key7b::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey7bSpec;
impl crate::RegisterSpec for SgiKey7bSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key7b::R`](R) reader structure"]
impl crate::Readable for SgiKey7bSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key7b::W`](W) writer structure"]
impl crate::Writable for SgiKey7bSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key7b to value 0"]
impl crate::Resettable for SgiKey7bSpec {}

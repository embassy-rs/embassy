#[doc = "Register `sgi_key6b` reader"]
pub type R = crate::R<SgiKey6bSpec>;
#[doc = "Register `sgi_key6b` writer"]
pub type W = crate::W<SgiKey6bSpec>;
#[doc = "Field `key6b` reader - Input Key register"]
pub type Key6bR = crate::FieldReader<u32>;
#[doc = "Field `key6b` writer - Input Key register"]
pub type Key6bW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key6b(&self) -> Key6bR {
        Key6bR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key6b(&mut self) -> Key6bW<SgiKey6bSpec> {
        Key6bW::new(self, 0)
    }
}
#[doc = "Input Key register 6 - Word-2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key6b::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key6b::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey6bSpec;
impl crate::RegisterSpec for SgiKey6bSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key6b::R`](R) reader structure"]
impl crate::Readable for SgiKey6bSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key6b::W`](W) writer structure"]
impl crate::Writable for SgiKey6bSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key6b to value 0"]
impl crate::Resettable for SgiKey6bSpec {}

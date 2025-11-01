#[doc = "Register `sgi_key6c` reader"]
pub type R = crate::R<SgiKey6cSpec>;
#[doc = "Register `sgi_key6c` writer"]
pub type W = crate::W<SgiKey6cSpec>;
#[doc = "Field `key6c` reader - Input Key register"]
pub type Key6cR = crate::FieldReader<u32>;
#[doc = "Field `key6c` writer - Input Key register"]
pub type Key6cW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key6c(&self) -> Key6cR {
        Key6cR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key6c(&mut self) -> Key6cW<SgiKey6cSpec> {
        Key6cW::new(self, 0)
    }
}
#[doc = "Input Key register 6 - Word-1\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key6c::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key6c::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey6cSpec;
impl crate::RegisterSpec for SgiKey6cSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key6c::R`](R) reader structure"]
impl crate::Readable for SgiKey6cSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key6c::W`](W) writer structure"]
impl crate::Writable for SgiKey6cSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key6c to value 0"]
impl crate::Resettable for SgiKey6cSpec {}

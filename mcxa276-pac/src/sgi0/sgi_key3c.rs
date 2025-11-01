#[doc = "Register `sgi_key3c` reader"]
pub type R = crate::R<SgiKey3cSpec>;
#[doc = "Register `sgi_key3c` writer"]
pub type W = crate::W<SgiKey3cSpec>;
#[doc = "Field `key3c` reader - Input Key register"]
pub type Key3cR = crate::FieldReader<u32>;
#[doc = "Field `key3c` writer - Input Key register"]
pub type Key3cW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key3c(&self) -> Key3cR {
        Key3cR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key3c(&mut self) -> Key3cW<SgiKey3cSpec> {
        Key3cW::new(self, 0)
    }
}
#[doc = "Input Key register 3 - Word-1\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key3c::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key3c::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey3cSpec;
impl crate::RegisterSpec for SgiKey3cSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key3c::R`](R) reader structure"]
impl crate::Readable for SgiKey3cSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key3c::W`](W) writer structure"]
impl crate::Writable for SgiKey3cSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key3c to value 0"]
impl crate::Resettable for SgiKey3cSpec {}

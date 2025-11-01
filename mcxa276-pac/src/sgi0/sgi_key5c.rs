#[doc = "Register `sgi_key5c` reader"]
pub type R = crate::R<SgiKey5cSpec>;
#[doc = "Register `sgi_key5c` writer"]
pub type W = crate::W<SgiKey5cSpec>;
#[doc = "Field `key5c` reader - Input Key register"]
pub type Key5cR = crate::FieldReader<u32>;
#[doc = "Field `key5c` writer - Input Key register"]
pub type Key5cW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key5c(&self) -> Key5cR {
        Key5cR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key5c(&mut self) -> Key5cW<SgiKey5cSpec> {
        Key5cW::new(self, 0)
    }
}
#[doc = "Input Key register 5 - Word-1\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key5c::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key5c::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey5cSpec;
impl crate::RegisterSpec for SgiKey5cSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key5c::R`](R) reader structure"]
impl crate::Readable for SgiKey5cSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key5c::W`](W) writer structure"]
impl crate::Writable for SgiKey5cSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key5c to value 0"]
impl crate::Resettable for SgiKey5cSpec {}

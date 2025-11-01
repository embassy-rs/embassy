#[doc = "Register `sgi_key0c` reader"]
pub type R = crate::R<SgiKey0cSpec>;
#[doc = "Register `sgi_key0c` writer"]
pub type W = crate::W<SgiKey0cSpec>;
#[doc = "Field `key0c` reader - Input Key register"]
pub type Key0cR = crate::FieldReader<u32>;
#[doc = "Field `key0c` writer - Input Key register"]
pub type Key0cW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key0c(&self) -> Key0cR {
        Key0cR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key0c(&mut self) -> Key0cW<SgiKey0cSpec> {
        Key0cW::new(self, 0)
    }
}
#[doc = "Input Key register 0 - Word-1\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key0c::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key0c::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey0cSpec;
impl crate::RegisterSpec for SgiKey0cSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key0c::R`](R) reader structure"]
impl crate::Readable for SgiKey0cSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key0c::W`](W) writer structure"]
impl crate::Writable for SgiKey0cSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key0c to value 0"]
impl crate::Resettable for SgiKey0cSpec {}

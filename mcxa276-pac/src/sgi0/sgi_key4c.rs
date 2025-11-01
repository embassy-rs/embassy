#[doc = "Register `sgi_key4c` reader"]
pub type R = crate::R<SgiKey4cSpec>;
#[doc = "Register `sgi_key4c` writer"]
pub type W = crate::W<SgiKey4cSpec>;
#[doc = "Field `key4c` reader - Input Key register"]
pub type Key4cR = crate::FieldReader<u32>;
#[doc = "Field `key4c` writer - Input Key register"]
pub type Key4cW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key4c(&self) -> Key4cR {
        Key4cR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key4c(&mut self) -> Key4cW<SgiKey4cSpec> {
        Key4cW::new(self, 0)
    }
}
#[doc = "Input Key register 4 - Word-1\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key4c::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key4c::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey4cSpec;
impl crate::RegisterSpec for SgiKey4cSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key4c::R`](R) reader structure"]
impl crate::Readable for SgiKey4cSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key4c::W`](W) writer structure"]
impl crate::Writable for SgiKey4cSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key4c to value 0"]
impl crate::Resettable for SgiKey4cSpec {}

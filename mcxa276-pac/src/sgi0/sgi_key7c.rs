#[doc = "Register `sgi_key7c` reader"]
pub type R = crate::R<SgiKey7cSpec>;
#[doc = "Register `sgi_key7c` writer"]
pub type W = crate::W<SgiKey7cSpec>;
#[doc = "Field `key7c` reader - Input Key register"]
pub type Key7cR = crate::FieldReader<u32>;
#[doc = "Field `key7c` writer - Input Key register"]
pub type Key7cW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key7c(&self) -> Key7cR {
        Key7cR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key7c(&mut self) -> Key7cW<SgiKey7cSpec> {
        Key7cW::new(self, 0)
    }
}
#[doc = "Input Key register 7 - Word-1\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key7c::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key7c::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey7cSpec;
impl crate::RegisterSpec for SgiKey7cSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key7c::R`](R) reader structure"]
impl crate::Readable for SgiKey7cSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key7c::W`](W) writer structure"]
impl crate::Writable for SgiKey7cSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key7c to value 0"]
impl crate::Resettable for SgiKey7cSpec {}

#[doc = "Register `SM0INIT` reader"]
pub type R = crate::R<Sm0initSpec>;
#[doc = "Register `SM0INIT` writer"]
pub type W = crate::W<Sm0initSpec>;
#[doc = "Field `INIT` reader - Initial Count Register Bits"]
pub type InitR = crate::FieldReader<u16>;
#[doc = "Field `INIT` writer - Initial Count Register Bits"]
pub type InitW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - Initial Count Register Bits"]
    #[inline(always)]
    pub fn init(&self) -> InitR {
        InitR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:15 - Initial Count Register Bits"]
    #[inline(always)]
    pub fn init(&mut self) -> InitW<Sm0initSpec> {
        InitW::new(self, 0)
    }
}
#[doc = "Initial Count Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0init::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm0init::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm0initSpec;
impl crate::RegisterSpec for Sm0initSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm0init::R`](R) reader structure"]
impl crate::Readable for Sm0initSpec {}
#[doc = "`write(|w| ..)` method takes [`sm0init::W`](W) writer structure"]
impl crate::Writable for Sm0initSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SM0INIT to value 0"]
impl crate::Resettable for Sm0initSpec {}

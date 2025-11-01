#[doc = "Register `SM2INIT` reader"]
pub type R = crate::R<Sm2initSpec>;
#[doc = "Register `SM2INIT` writer"]
pub type W = crate::W<Sm2initSpec>;
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
    pub fn init(&mut self) -> InitW<Sm2initSpec> {
        InitW::new(self, 0)
    }
}
#[doc = "Initial Count Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2init::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2init::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm2initSpec;
impl crate::RegisterSpec for Sm2initSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm2init::R`](R) reader structure"]
impl crate::Readable for Sm2initSpec {}
#[doc = "`write(|w| ..)` method takes [`sm2init::W`](W) writer structure"]
impl crate::Writable for Sm2initSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SM2INIT to value 0"]
impl crate::Resettable for Sm2initSpec {}

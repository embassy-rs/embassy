#[doc = "Register `SM3INIT` reader"]
pub type R = crate::R<Sm3initSpec>;
#[doc = "Register `SM3INIT` writer"]
pub type W = crate::W<Sm3initSpec>;
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
    pub fn init(&mut self) -> InitW<Sm3initSpec> {
        InitW::new(self, 0)
    }
}
#[doc = "Initial Count Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3init::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm3init::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm3initSpec;
impl crate::RegisterSpec for Sm3initSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm3init::R`](R) reader structure"]
impl crate::Readable for Sm3initSpec {}
#[doc = "`write(|w| ..)` method takes [`sm3init::W`](W) writer structure"]
impl crate::Writable for Sm3initSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SM3INIT to value 0"]
impl crate::Resettable for Sm3initSpec {}

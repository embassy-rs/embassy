#[doc = "Register `UINIT` reader"]
pub type R = crate::R<UinitSpec>;
#[doc = "Register `UINIT` writer"]
pub type W = crate::W<UinitSpec>;
#[doc = "Field `INIT` reader - INIT"]
pub type InitR = crate::FieldReader<u16>;
#[doc = "Field `INIT` writer - INIT"]
pub type InitW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - INIT"]
    #[inline(always)]
    pub fn init(&self) -> InitR {
        InitR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:15 - INIT"]
    #[inline(always)]
    pub fn init(&mut self) -> InitW<UinitSpec> {
        InitW::new(self, 0)
    }
}
#[doc = "Upper Initialization Register\n\nYou can [`read`](crate::Reg::read) this register and get [`uinit::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`uinit::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct UinitSpec;
impl crate::RegisterSpec for UinitSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`uinit::R`](R) reader structure"]
impl crate::Readable for UinitSpec {}
#[doc = "`write(|w| ..)` method takes [`uinit::W`](W) writer structure"]
impl crate::Writable for UinitSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets UINIT to value 0"]
impl crate::Resettable for UinitSpec {}

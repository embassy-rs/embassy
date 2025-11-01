#[doc = "Register `PKC_MODE1` reader"]
pub type R = crate::R<PkcMode1Spec>;
#[doc = "Register `PKC_MODE1` writer"]
pub type W = crate::W<PkcMode1Spec>;
#[doc = "Field `MODE` reader - Calculation Mode / MC Start address"]
pub type ModeR = crate::FieldReader;
#[doc = "Field `MODE` writer - Calculation Mode / MC Start address"]
pub type ModeW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Calculation Mode / MC Start address"]
    #[inline(always)]
    pub fn mode(&self) -> ModeR {
        ModeR::new((self.bits & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Calculation Mode / MC Start address"]
    #[inline(always)]
    pub fn mode(&mut self) -> ModeW<PkcMode1Spec> {
        ModeW::new(self, 0)
    }
}
#[doc = "Mode register, parameter set 1\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_mode1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_mode1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkcMode1Spec;
impl crate::RegisterSpec for PkcMode1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pkc_mode1::R`](R) reader structure"]
impl crate::Readable for PkcMode1Spec {}
#[doc = "`write(|w| ..)` method takes [`pkc_mode1::W`](W) writer structure"]
impl crate::Writable for PkcMode1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PKC_MODE1 to value 0"]
impl crate::Resettable for PkcMode1Spec {}

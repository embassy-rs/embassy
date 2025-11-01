#[doc = "Register `PKC_MODE2` reader"]
pub type R = crate::R<PkcMode2Spec>;
#[doc = "Register `PKC_MODE2` writer"]
pub type W = crate::W<PkcMode2Spec>;
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
    pub fn mode(&mut self) -> ModeW<PkcMode2Spec> {
        ModeW::new(self, 0)
    }
}
#[doc = "Mode register, parameter set 2\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_mode2::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_mode2::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkcMode2Spec;
impl crate::RegisterSpec for PkcMode2Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pkc_mode2::R`](R) reader structure"]
impl crate::Readable for PkcMode2Spec {}
#[doc = "`write(|w| ..)` method takes [`pkc_mode2::W`](W) writer structure"]
impl crate::Writable for PkcMode2Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PKC_MODE2 to value 0"]
impl crate::Resettable for PkcMode2Spec {}

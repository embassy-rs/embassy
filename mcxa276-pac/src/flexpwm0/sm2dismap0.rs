#[doc = "Register `SM2DISMAP0` reader"]
pub type R = crate::R<Sm2dismap0Spec>;
#[doc = "Register `SM2DISMAP0` writer"]
pub type W = crate::W<Sm2dismap0Spec>;
#[doc = "Field `DIS0A` reader - PWM_A Fault Disable Mask 0"]
pub type Dis0aR = crate::FieldReader;
#[doc = "Field `DIS0A` writer - PWM_A Fault Disable Mask 0"]
pub type Dis0aW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `DIS0B` reader - PWM_B Fault Disable Mask 0"]
pub type Dis0bR = crate::FieldReader;
#[doc = "Field `DIS0B` writer - PWM_B Fault Disable Mask 0"]
pub type Dis0bW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `DIS0X` reader - PWM_X Fault Disable Mask 0"]
pub type Dis0xR = crate::FieldReader;
#[doc = "Field `DIS0X` writer - PWM_X Fault Disable Mask 0"]
pub type Dis0xW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:3 - PWM_A Fault Disable Mask 0"]
    #[inline(always)]
    pub fn dis0a(&self) -> Dis0aR {
        Dis0aR::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bits 4:7 - PWM_B Fault Disable Mask 0"]
    #[inline(always)]
    pub fn dis0b(&self) -> Dis0bR {
        Dis0bR::new(((self.bits >> 4) & 0x0f) as u8)
    }
    #[doc = "Bits 8:11 - PWM_X Fault Disable Mask 0"]
    #[inline(always)]
    pub fn dis0x(&self) -> Dis0xR {
        Dis0xR::new(((self.bits >> 8) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - PWM_A Fault Disable Mask 0"]
    #[inline(always)]
    pub fn dis0a(&mut self) -> Dis0aW<Sm2dismap0Spec> {
        Dis0aW::new(self, 0)
    }
    #[doc = "Bits 4:7 - PWM_B Fault Disable Mask 0"]
    #[inline(always)]
    pub fn dis0b(&mut self) -> Dis0bW<Sm2dismap0Spec> {
        Dis0bW::new(self, 4)
    }
    #[doc = "Bits 8:11 - PWM_X Fault Disable Mask 0"]
    #[inline(always)]
    pub fn dis0x(&mut self) -> Dis0xW<Sm2dismap0Spec> {
        Dis0xW::new(self, 8)
    }
}
#[doc = "Fault Disable Mapping Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2dismap0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2dismap0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm2dismap0Spec;
impl crate::RegisterSpec for Sm2dismap0Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm2dismap0::R`](R) reader structure"]
impl crate::Readable for Sm2dismap0Spec {}
#[doc = "`write(|w| ..)` method takes [`sm2dismap0::W`](W) writer structure"]
impl crate::Writable for Sm2dismap0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SM2DISMAP0 to value 0xffff"]
impl crate::Resettable for Sm2dismap0Spec {
    const RESET_VALUE: u16 = 0xffff;
}

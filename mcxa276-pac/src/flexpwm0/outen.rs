#[doc = "Register `OUTEN` reader"]
pub type R = crate::R<OutenSpec>;
#[doc = "Register `OUTEN` writer"]
pub type W = crate::W<OutenSpec>;
#[doc = "Field `PWMX_EN` reader - PWM_X Output Enables"]
pub type PwmxEnR = crate::FieldReader;
#[doc = "Field `PWMX_EN` writer - PWM_X Output Enables"]
pub type PwmxEnW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `PWMB_EN` reader - PWM_B Output Enables"]
pub type PwmbEnR = crate::FieldReader;
#[doc = "Field `PWMB_EN` writer - PWM_B Output Enables"]
pub type PwmbEnW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `PWMA_EN` reader - PWM_A Output Enables"]
pub type PwmaEnR = crate::FieldReader;
#[doc = "Field `PWMA_EN` writer - PWM_A Output Enables"]
pub type PwmaEnW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:3 - PWM_X Output Enables"]
    #[inline(always)]
    pub fn pwmx_en(&self) -> PwmxEnR {
        PwmxEnR::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bits 4:7 - PWM_B Output Enables"]
    #[inline(always)]
    pub fn pwmb_en(&self) -> PwmbEnR {
        PwmbEnR::new(((self.bits >> 4) & 0x0f) as u8)
    }
    #[doc = "Bits 8:11 - PWM_A Output Enables"]
    #[inline(always)]
    pub fn pwma_en(&self) -> PwmaEnR {
        PwmaEnR::new(((self.bits >> 8) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - PWM_X Output Enables"]
    #[inline(always)]
    pub fn pwmx_en(&mut self) -> PwmxEnW<OutenSpec> {
        PwmxEnW::new(self, 0)
    }
    #[doc = "Bits 4:7 - PWM_B Output Enables"]
    #[inline(always)]
    pub fn pwmb_en(&mut self) -> PwmbEnW<OutenSpec> {
        PwmbEnW::new(self, 4)
    }
    #[doc = "Bits 8:11 - PWM_A Output Enables"]
    #[inline(always)]
    pub fn pwma_en(&mut self) -> PwmaEnW<OutenSpec> {
        PwmaEnW::new(self, 8)
    }
}
#[doc = "Output Enable Register\n\nYou can [`read`](crate::Reg::read) this register and get [`outen::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`outen::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct OutenSpec;
impl crate::RegisterSpec for OutenSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`outen::R`](R) reader structure"]
impl crate::Readable for OutenSpec {}
#[doc = "`write(|w| ..)` method takes [`outen::W`](W) writer structure"]
impl crate::Writable for OutenSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets OUTEN to value 0"]
impl crate::Resettable for OutenSpec {}

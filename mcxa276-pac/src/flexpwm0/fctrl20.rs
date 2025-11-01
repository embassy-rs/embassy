#[doc = "Register `FCTRL20` reader"]
pub type R = crate::R<Fctrl20Spec>;
#[doc = "Register `FCTRL20` writer"]
pub type W = crate::W<Fctrl20Spec>;
#[doc = "No Combinational Path From Fault Input To PWM Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Nocomb {
    #[doc = "0: There is a combinational link from the fault inputs to the PWM outputs. The fault inputs are combined with the filtered and latched fault signals to disable the PWM outputs."]
    Enabled = 0,
    #[doc = "1: The direct combinational path from the fault inputs to the PWM outputs is disabled and the filtered and latched fault signals are used to disable the PWM outputs."]
    Disabled = 1,
}
impl From<Nocomb> for u8 {
    #[inline(always)]
    fn from(variant: Nocomb) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Nocomb {
    type Ux = u8;
}
impl crate::IsEnum for Nocomb {}
#[doc = "Field `NOCOMB` reader - No Combinational Path From Fault Input To PWM Output"]
pub type NocombR = crate::FieldReader<Nocomb>;
impl NocombR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Nocomb> {
        match self.bits {
            0 => Some(Nocomb::Enabled),
            1 => Some(Nocomb::Disabled),
            _ => None,
        }
    }
    #[doc = "There is a combinational link from the fault inputs to the PWM outputs. The fault inputs are combined with the filtered and latched fault signals to disable the PWM outputs."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Nocomb::Enabled
    }
    #[doc = "The direct combinational path from the fault inputs to the PWM outputs is disabled and the filtered and latched fault signals are used to disable the PWM outputs."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Nocomb::Disabled
    }
}
#[doc = "Field `NOCOMB` writer - No Combinational Path From Fault Input To PWM Output"]
pub type NocombW<'a, REG> = crate::FieldWriter<'a, REG, 4, Nocomb>;
impl<'a, REG> NocombW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "There is a combinational link from the fault inputs to the PWM outputs. The fault inputs are combined with the filtered and latched fault signals to disable the PWM outputs."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Nocomb::Enabled)
    }
    #[doc = "The direct combinational path from the fault inputs to the PWM outputs is disabled and the filtered and latched fault signals are used to disable the PWM outputs."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Nocomb::Disabled)
    }
}
impl R {
    #[doc = "Bits 0:3 - No Combinational Path From Fault Input To PWM Output"]
    #[inline(always)]
    pub fn nocomb(&self) -> NocombR {
        NocombR::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - No Combinational Path From Fault Input To PWM Output"]
    #[inline(always)]
    pub fn nocomb(&mut self) -> NocombW<Fctrl20Spec> {
        NocombW::new(self, 0)
    }
}
#[doc = "Fault Control 2 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`fctrl20::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fctrl20::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Fctrl20Spec;
impl crate::RegisterSpec for Fctrl20Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`fctrl20::R`](R) reader structure"]
impl crate::Readable for Fctrl20Spec {}
#[doc = "`write(|w| ..)` method takes [`fctrl20::W`](W) writer structure"]
impl crate::Writable for Fctrl20Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FCTRL20 to value 0"]
impl crate::Resettable for Fctrl20Spec {}

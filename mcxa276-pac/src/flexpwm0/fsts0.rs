#[doc = "Register `FSTS0` reader"]
pub type R = crate::R<Fsts0Spec>;
#[doc = "Register `FSTS0` writer"]
pub type W = crate::W<Fsts0Spec>;
#[doc = "Fault Flags\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Fflag {
    #[doc = "0: No fault on the FAULTx pin."]
    NoFlag = 0,
    #[doc = "1: Fault on the FAULTx pin."]
    Flag = 1,
}
impl From<Fflag> for u8 {
    #[inline(always)]
    fn from(variant: Fflag) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Fflag {
    type Ux = u8;
}
impl crate::IsEnum for Fflag {}
#[doc = "Field `FFLAG` reader - Fault Flags"]
pub type FflagR = crate::FieldReader<Fflag>;
impl FflagR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Fflag> {
        match self.bits {
            0 => Some(Fflag::NoFlag),
            1 => Some(Fflag::Flag),
            _ => None,
        }
    }
    #[doc = "No fault on the FAULTx pin."]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Fflag::NoFlag
    }
    #[doc = "Fault on the FAULTx pin."]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Fflag::Flag
    }
}
#[doc = "Field `FFLAG` writer - Fault Flags"]
pub type FflagW<'a, REG> = crate::FieldWriter<'a, REG, 4, Fflag>;
impl<'a, REG> FflagW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "No fault on the FAULTx pin."]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Fflag::NoFlag)
    }
    #[doc = "Fault on the FAULTx pin."]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Fflag::Flag)
    }
}
#[doc = "Full Cycle\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Ffull {
    #[doc = "0: PWM outputs are not re-enabled at the start of a full cycle"]
    PwmOutputsNotReenabled = 0,
    #[doc = "1: PWM outputs are re-enabled at the start of a full cycle"]
    PwmOutputsReenabled = 1,
}
impl From<Ffull> for u8 {
    #[inline(always)]
    fn from(variant: Ffull) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Ffull {
    type Ux = u8;
}
impl crate::IsEnum for Ffull {}
#[doc = "Field `FFULL` reader - Full Cycle"]
pub type FfullR = crate::FieldReader<Ffull>;
impl FfullR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Ffull> {
        match self.bits {
            0 => Some(Ffull::PwmOutputsNotReenabled),
            1 => Some(Ffull::PwmOutputsReenabled),
            _ => None,
        }
    }
    #[doc = "PWM outputs are not re-enabled at the start of a full cycle"]
    #[inline(always)]
    pub fn is_pwm_outputs_not_reenabled(&self) -> bool {
        *self == Ffull::PwmOutputsNotReenabled
    }
    #[doc = "PWM outputs are re-enabled at the start of a full cycle"]
    #[inline(always)]
    pub fn is_pwm_outputs_reenabled(&self) -> bool {
        *self == Ffull::PwmOutputsReenabled
    }
}
#[doc = "Field `FFULL` writer - Full Cycle"]
pub type FfullW<'a, REG> = crate::FieldWriter<'a, REG, 4, Ffull>;
impl<'a, REG> FfullW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "PWM outputs are not re-enabled at the start of a full cycle"]
    #[inline(always)]
    pub fn pwm_outputs_not_reenabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ffull::PwmOutputsNotReenabled)
    }
    #[doc = "PWM outputs are re-enabled at the start of a full cycle"]
    #[inline(always)]
    pub fn pwm_outputs_reenabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ffull::PwmOutputsReenabled)
    }
}
#[doc = "Field `FFPIN` reader - Filtered Fault Pins"]
pub type FfpinR = crate::FieldReader;
#[doc = "Half Cycle Fault Recovery\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Fhalf {
    #[doc = "0: PWM outputs are not re-enabled at the start of a half cycle."]
    PwmOutputsNotReenabled = 0,
    #[doc = "1: PWM outputs are re-enabled at the start of a half cycle (as defined by VAL0)."]
    PwmOutputsReenabled = 1,
}
impl From<Fhalf> for u8 {
    #[inline(always)]
    fn from(variant: Fhalf) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Fhalf {
    type Ux = u8;
}
impl crate::IsEnum for Fhalf {}
#[doc = "Field `FHALF` reader - Half Cycle Fault Recovery"]
pub type FhalfR = crate::FieldReader<Fhalf>;
impl FhalfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Fhalf> {
        match self.bits {
            0 => Some(Fhalf::PwmOutputsNotReenabled),
            1 => Some(Fhalf::PwmOutputsReenabled),
            _ => None,
        }
    }
    #[doc = "PWM outputs are not re-enabled at the start of a half cycle."]
    #[inline(always)]
    pub fn is_pwm_outputs_not_reenabled(&self) -> bool {
        *self == Fhalf::PwmOutputsNotReenabled
    }
    #[doc = "PWM outputs are re-enabled at the start of a half cycle (as defined by VAL0)."]
    #[inline(always)]
    pub fn is_pwm_outputs_reenabled(&self) -> bool {
        *self == Fhalf::PwmOutputsReenabled
    }
}
#[doc = "Field `FHALF` writer - Half Cycle Fault Recovery"]
pub type FhalfW<'a, REG> = crate::FieldWriter<'a, REG, 4, Fhalf>;
impl<'a, REG> FhalfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "PWM outputs are not re-enabled at the start of a half cycle."]
    #[inline(always)]
    pub fn pwm_outputs_not_reenabled(self) -> &'a mut crate::W<REG> {
        self.variant(Fhalf::PwmOutputsNotReenabled)
    }
    #[doc = "PWM outputs are re-enabled at the start of a half cycle (as defined by VAL0)."]
    #[inline(always)]
    pub fn pwm_outputs_reenabled(self) -> &'a mut crate::W<REG> {
        self.variant(Fhalf::PwmOutputsReenabled)
    }
}
impl R {
    #[doc = "Bits 0:3 - Fault Flags"]
    #[inline(always)]
    pub fn fflag(&self) -> FflagR {
        FflagR::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bits 4:7 - Full Cycle"]
    #[inline(always)]
    pub fn ffull(&self) -> FfullR {
        FfullR::new(((self.bits >> 4) & 0x0f) as u8)
    }
    #[doc = "Bits 8:11 - Filtered Fault Pins"]
    #[inline(always)]
    pub fn ffpin(&self) -> FfpinR {
        FfpinR::new(((self.bits >> 8) & 0x0f) as u8)
    }
    #[doc = "Bits 12:15 - Half Cycle Fault Recovery"]
    #[inline(always)]
    pub fn fhalf(&self) -> FhalfR {
        FhalfR::new(((self.bits >> 12) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - Fault Flags"]
    #[inline(always)]
    pub fn fflag(&mut self) -> FflagW<Fsts0Spec> {
        FflagW::new(self, 0)
    }
    #[doc = "Bits 4:7 - Full Cycle"]
    #[inline(always)]
    pub fn ffull(&mut self) -> FfullW<Fsts0Spec> {
        FfullW::new(self, 4)
    }
    #[doc = "Bits 12:15 - Half Cycle Fault Recovery"]
    #[inline(always)]
    pub fn fhalf(&mut self) -> FhalfW<Fsts0Spec> {
        FhalfW::new(self, 12)
    }
}
#[doc = "Fault Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`fsts0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fsts0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Fsts0Spec;
impl crate::RegisterSpec for Fsts0Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`fsts0::R`](R) reader structure"]
impl crate::Readable for Fsts0Spec {}
#[doc = "`write(|w| ..)` method takes [`fsts0::W`](W) writer structure"]
impl crate::Writable for Fsts0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FSTS0 to value 0"]
impl crate::Resettable for Fsts0Spec {}

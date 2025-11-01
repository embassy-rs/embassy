#[doc = "Register `FCTRL0` reader"]
pub type R = crate::R<Fctrl0Spec>;
#[doc = "Register `FCTRL0` writer"]
pub type W = crate::W<Fctrl0Spec>;
#[doc = "Fault Interrupt Enables\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Fie {
    #[doc = "0: FAULTx CPU interrupt requests disabled."]
    Disabled = 0,
    #[doc = "1: FAULTx CPU interrupt requests enabled."]
    Enabled = 1,
}
impl From<Fie> for u8 {
    #[inline(always)]
    fn from(variant: Fie) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Fie {
    type Ux = u8;
}
impl crate::IsEnum for Fie {}
#[doc = "Field `FIE` reader - Fault Interrupt Enables"]
pub type FieR = crate::FieldReader<Fie>;
impl FieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Fie> {
        match self.bits {
            0 => Some(Fie::Disabled),
            1 => Some(Fie::Enabled),
            _ => None,
        }
    }
    #[doc = "FAULTx CPU interrupt requests disabled."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Fie::Disabled
    }
    #[doc = "FAULTx CPU interrupt requests enabled."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Fie::Enabled
    }
}
#[doc = "Field `FIE` writer - Fault Interrupt Enables"]
pub type FieW<'a, REG> = crate::FieldWriter<'a, REG, 4, Fie>;
impl<'a, REG> FieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "FAULTx CPU interrupt requests disabled."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Fie::Disabled)
    }
    #[doc = "FAULTx CPU interrupt requests enabled."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Fie::Enabled)
    }
}
#[doc = "Fault Safety Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Fsafe {
    #[doc = "0: Normal mode. PWM outputs disabled by this fault are not enabled until FSTS\\[FFLAGx\\] is clear at the start of a half cycle or full cycle depending on the states of FSTS\\[FHALF\\] and FSTS\\[FFULL\\] without regard to the state of FSTS\\[FFPINx\\]. If neither FHALF nor FFULL is set, then the fault condition cannot be cleared. The PWM outputs disabled by this fault input will not be re-enabled until the actual FAULTx input signal de-asserts since the fault input will combinationally disable the PWM outputs (as programmed in DISMAPn)."]
    Normal = 0,
    #[doc = "1: Safe mode. PWM outputs disabled by this fault are not enabled until FSTS\\[FFLAGx\\] is clear and FSTS\\[FFPINx\\] is clear at the start of a half cycle or full cycle depending on the states of FSTS\\[FHALF\\] and FSTS\\[FFULL\\]. If neither FHLAF nor FFULL is set, then the fault condition cannot be cleared."]
    Safe = 1,
}
impl From<Fsafe> for u8 {
    #[inline(always)]
    fn from(variant: Fsafe) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Fsafe {
    type Ux = u8;
}
impl crate::IsEnum for Fsafe {}
#[doc = "Field `FSAFE` reader - Fault Safety Mode"]
pub type FsafeR = crate::FieldReader<Fsafe>;
impl FsafeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Fsafe> {
        match self.bits {
            0 => Some(Fsafe::Normal),
            1 => Some(Fsafe::Safe),
            _ => None,
        }
    }
    #[doc = "Normal mode. PWM outputs disabled by this fault are not enabled until FSTS\\[FFLAGx\\] is clear at the start of a half cycle or full cycle depending on the states of FSTS\\[FHALF\\] and FSTS\\[FFULL\\] without regard to the state of FSTS\\[FFPINx\\]. If neither FHALF nor FFULL is set, then the fault condition cannot be cleared. The PWM outputs disabled by this fault input will not be re-enabled until the actual FAULTx input signal de-asserts since the fault input will combinationally disable the PWM outputs (as programmed in DISMAPn)."]
    #[inline(always)]
    pub fn is_normal(&self) -> bool {
        *self == Fsafe::Normal
    }
    #[doc = "Safe mode. PWM outputs disabled by this fault are not enabled until FSTS\\[FFLAGx\\] is clear and FSTS\\[FFPINx\\] is clear at the start of a half cycle or full cycle depending on the states of FSTS\\[FHALF\\] and FSTS\\[FFULL\\]. If neither FHLAF nor FFULL is set, then the fault condition cannot be cleared."]
    #[inline(always)]
    pub fn is_safe(&self) -> bool {
        *self == Fsafe::Safe
    }
}
#[doc = "Field `FSAFE` writer - Fault Safety Mode"]
pub type FsafeW<'a, REG> = crate::FieldWriter<'a, REG, 4, Fsafe>;
impl<'a, REG> FsafeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Normal mode. PWM outputs disabled by this fault are not enabled until FSTS\\[FFLAGx\\] is clear at the start of a half cycle or full cycle depending on the states of FSTS\\[FHALF\\] and FSTS\\[FFULL\\] without regard to the state of FSTS\\[FFPINx\\]. If neither FHALF nor FFULL is set, then the fault condition cannot be cleared. The PWM outputs disabled by this fault input will not be re-enabled until the actual FAULTx input signal de-asserts since the fault input will combinationally disable the PWM outputs (as programmed in DISMAPn)."]
    #[inline(always)]
    pub fn normal(self) -> &'a mut crate::W<REG> {
        self.variant(Fsafe::Normal)
    }
    #[doc = "Safe mode. PWM outputs disabled by this fault are not enabled until FSTS\\[FFLAGx\\] is clear and FSTS\\[FFPINx\\] is clear at the start of a half cycle or full cycle depending on the states of FSTS\\[FHALF\\] and FSTS\\[FFULL\\]. If neither FHLAF nor FFULL is set, then the fault condition cannot be cleared."]
    #[inline(always)]
    pub fn safe(self) -> &'a mut crate::W<REG> {
        self.variant(Fsafe::Safe)
    }
}
#[doc = "Automatic Fault Clearing\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Fauto {
    #[doc = "0: Manual fault clearing. PWM outputs disabled by this fault are not enabled until FSTS\\[FFLAGx\\] is clear at the start of a half cycle or full cycle depending on the states of FSTS\\[FHALF\\] and FSTS\\[FFULL\\]. If neither FFULL nor FHALF is set, then the fault condition cannot be cleared. This is further controlled by FCTRL\\[FSAFE\\]."]
    Manual = 0,
    #[doc = "1: Automatic fault clearing. PWM outputs disabled by this fault are enabled when FSTS\\[FFPINx\\] is clear at the start of a half cycle or full cycle depending on the states of FSTS\\[FHALF\\] and FSTS\\[FFULL\\] without regard to the state of FSTS\\[FFLAGx\\]. If neither FFULL nor FHALF is set, then the fault condition cannot be cleared."]
    Automatic = 1,
}
impl From<Fauto> for u8 {
    #[inline(always)]
    fn from(variant: Fauto) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Fauto {
    type Ux = u8;
}
impl crate::IsEnum for Fauto {}
#[doc = "Field `FAUTO` reader - Automatic Fault Clearing"]
pub type FautoR = crate::FieldReader<Fauto>;
impl FautoR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Fauto> {
        match self.bits {
            0 => Some(Fauto::Manual),
            1 => Some(Fauto::Automatic),
            _ => None,
        }
    }
    #[doc = "Manual fault clearing. PWM outputs disabled by this fault are not enabled until FSTS\\[FFLAGx\\] is clear at the start of a half cycle or full cycle depending on the states of FSTS\\[FHALF\\] and FSTS\\[FFULL\\]. If neither FFULL nor FHALF is set, then the fault condition cannot be cleared. This is further controlled by FCTRL\\[FSAFE\\]."]
    #[inline(always)]
    pub fn is_manual(&self) -> bool {
        *self == Fauto::Manual
    }
    #[doc = "Automatic fault clearing. PWM outputs disabled by this fault are enabled when FSTS\\[FFPINx\\] is clear at the start of a half cycle or full cycle depending on the states of FSTS\\[FHALF\\] and FSTS\\[FFULL\\] without regard to the state of FSTS\\[FFLAGx\\]. If neither FFULL nor FHALF is set, then the fault condition cannot be cleared."]
    #[inline(always)]
    pub fn is_automatic(&self) -> bool {
        *self == Fauto::Automatic
    }
}
#[doc = "Field `FAUTO` writer - Automatic Fault Clearing"]
pub type FautoW<'a, REG> = crate::FieldWriter<'a, REG, 4, Fauto>;
impl<'a, REG> FautoW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Manual fault clearing. PWM outputs disabled by this fault are not enabled until FSTS\\[FFLAGx\\] is clear at the start of a half cycle or full cycle depending on the states of FSTS\\[FHALF\\] and FSTS\\[FFULL\\]. If neither FFULL nor FHALF is set, then the fault condition cannot be cleared. This is further controlled by FCTRL\\[FSAFE\\]."]
    #[inline(always)]
    pub fn manual(self) -> &'a mut crate::W<REG> {
        self.variant(Fauto::Manual)
    }
    #[doc = "Automatic fault clearing. PWM outputs disabled by this fault are enabled when FSTS\\[FFPINx\\] is clear at the start of a half cycle or full cycle depending on the states of FSTS\\[FHALF\\] and FSTS\\[FFULL\\] without regard to the state of FSTS\\[FFLAGx\\]. If neither FFULL nor FHALF is set, then the fault condition cannot be cleared."]
    #[inline(always)]
    pub fn automatic(self) -> &'a mut crate::W<REG> {
        self.variant(Fauto::Automatic)
    }
}
#[doc = "Fault Level\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Flvl {
    #[doc = "0: A logic 0 on the fault input indicates a fault condition."]
    Logic0 = 0,
    #[doc = "1: A logic 1 on the fault input indicates a fault condition."]
    Logic1 = 1,
}
impl From<Flvl> for u8 {
    #[inline(always)]
    fn from(variant: Flvl) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Flvl {
    type Ux = u8;
}
impl crate::IsEnum for Flvl {}
#[doc = "Field `FLVL` reader - Fault Level"]
pub type FlvlR = crate::FieldReader<Flvl>;
impl FlvlR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Flvl> {
        match self.bits {
            0 => Some(Flvl::Logic0),
            1 => Some(Flvl::Logic1),
            _ => None,
        }
    }
    #[doc = "A logic 0 on the fault input indicates a fault condition."]
    #[inline(always)]
    pub fn is_logic_0(&self) -> bool {
        *self == Flvl::Logic0
    }
    #[doc = "A logic 1 on the fault input indicates a fault condition."]
    #[inline(always)]
    pub fn is_logic_1(&self) -> bool {
        *self == Flvl::Logic1
    }
}
#[doc = "Field `FLVL` writer - Fault Level"]
pub type FlvlW<'a, REG> = crate::FieldWriter<'a, REG, 4, Flvl>;
impl<'a, REG> FlvlW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "A logic 0 on the fault input indicates a fault condition."]
    #[inline(always)]
    pub fn logic_0(self) -> &'a mut crate::W<REG> {
        self.variant(Flvl::Logic0)
    }
    #[doc = "A logic 1 on the fault input indicates a fault condition."]
    #[inline(always)]
    pub fn logic_1(self) -> &'a mut crate::W<REG> {
        self.variant(Flvl::Logic1)
    }
}
impl R {
    #[doc = "Bits 0:3 - Fault Interrupt Enables"]
    #[inline(always)]
    pub fn fie(&self) -> FieR {
        FieR::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bits 4:7 - Fault Safety Mode"]
    #[inline(always)]
    pub fn fsafe(&self) -> FsafeR {
        FsafeR::new(((self.bits >> 4) & 0x0f) as u8)
    }
    #[doc = "Bits 8:11 - Automatic Fault Clearing"]
    #[inline(always)]
    pub fn fauto(&self) -> FautoR {
        FautoR::new(((self.bits >> 8) & 0x0f) as u8)
    }
    #[doc = "Bits 12:15 - Fault Level"]
    #[inline(always)]
    pub fn flvl(&self) -> FlvlR {
        FlvlR::new(((self.bits >> 12) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - Fault Interrupt Enables"]
    #[inline(always)]
    pub fn fie(&mut self) -> FieW<Fctrl0Spec> {
        FieW::new(self, 0)
    }
    #[doc = "Bits 4:7 - Fault Safety Mode"]
    #[inline(always)]
    pub fn fsafe(&mut self) -> FsafeW<Fctrl0Spec> {
        FsafeW::new(self, 4)
    }
    #[doc = "Bits 8:11 - Automatic Fault Clearing"]
    #[inline(always)]
    pub fn fauto(&mut self) -> FautoW<Fctrl0Spec> {
        FautoW::new(self, 8)
    }
    #[doc = "Bits 12:15 - Fault Level"]
    #[inline(always)]
    pub fn flvl(&mut self) -> FlvlW<Fctrl0Spec> {
        FlvlW::new(self, 12)
    }
}
#[doc = "Fault Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`fctrl0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fctrl0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Fctrl0Spec;
impl crate::RegisterSpec for Fctrl0Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`fctrl0::R`](R) reader structure"]
impl crate::Readable for Fctrl0Spec {}
#[doc = "`write(|w| ..)` method takes [`fctrl0::W`](W) writer structure"]
impl crate::Writable for Fctrl0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FCTRL0 to value 0"]
impl crate::Resettable for Fctrl0Spec {}

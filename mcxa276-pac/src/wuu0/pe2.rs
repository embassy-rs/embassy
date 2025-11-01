#[doc = "Register `PE2` reader"]
pub type R = crate::R<Pe2Spec>;
#[doc = "Register `PE2` writer"]
pub type W = crate::W<Pe2Spec>;
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe16 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe16> for u8 {
    #[inline(always)]
    fn from(variant: Wupe16) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe16 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe16 {}
#[doc = "Field `WUPE16` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe16R = crate::FieldReader<Wupe16>;
impl Wupe16R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe16 {
        match self.bits {
            0 => Wupe16::Disable,
            1 => Wupe16::EnRiseHi,
            2 => Wupe16::EnFallLo,
            3 => Wupe16::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe16::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe16::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe16::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe16::EnAny
    }
}
#[doc = "Field `WUPE16` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe16W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe16, crate::Safe>;
impl<'a, REG> Wupe16W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe16::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe16::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe16::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe16::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe17 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe17> for u8 {
    #[inline(always)]
    fn from(variant: Wupe17) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe17 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe17 {}
#[doc = "Field `WUPE17` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe17R = crate::FieldReader<Wupe17>;
impl Wupe17R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe17 {
        match self.bits {
            0 => Wupe17::Disable,
            1 => Wupe17::EnRiseHi,
            2 => Wupe17::EnFallLo,
            3 => Wupe17::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe17::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe17::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe17::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe17::EnAny
    }
}
#[doc = "Field `WUPE17` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe17W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe17, crate::Safe>;
impl<'a, REG> Wupe17W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe17::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe17::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe17::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe17::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe18 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe18> for u8 {
    #[inline(always)]
    fn from(variant: Wupe18) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe18 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe18 {}
#[doc = "Field `WUPE18` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe18R = crate::FieldReader<Wupe18>;
impl Wupe18R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe18 {
        match self.bits {
            0 => Wupe18::Disable,
            1 => Wupe18::EnRiseHi,
            2 => Wupe18::EnFallLo,
            3 => Wupe18::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe18::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe18::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe18::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe18::EnAny
    }
}
#[doc = "Field `WUPE18` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe18W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe18, crate::Safe>;
impl<'a, REG> Wupe18W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe18::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe18::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe18::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe18::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe19 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe19> for u8 {
    #[inline(always)]
    fn from(variant: Wupe19) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe19 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe19 {}
#[doc = "Field `WUPE19` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe19R = crate::FieldReader<Wupe19>;
impl Wupe19R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe19 {
        match self.bits {
            0 => Wupe19::Disable,
            1 => Wupe19::EnRiseHi,
            2 => Wupe19::EnFallLo,
            3 => Wupe19::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe19::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe19::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe19::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe19::EnAny
    }
}
#[doc = "Field `WUPE19` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe19W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe19, crate::Safe>;
impl<'a, REG> Wupe19W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe19::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe19::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe19::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe19::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe20 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe20> for u8 {
    #[inline(always)]
    fn from(variant: Wupe20) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe20 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe20 {}
#[doc = "Field `WUPE20` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe20R = crate::FieldReader<Wupe20>;
impl Wupe20R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe20 {
        match self.bits {
            0 => Wupe20::Disable,
            1 => Wupe20::EnRiseHi,
            2 => Wupe20::EnFallLo,
            3 => Wupe20::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe20::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe20::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe20::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe20::EnAny
    }
}
#[doc = "Field `WUPE20` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe20W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe20, crate::Safe>;
impl<'a, REG> Wupe20W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe20::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe20::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe20::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe20::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe21 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe21> for u8 {
    #[inline(always)]
    fn from(variant: Wupe21) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe21 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe21 {}
#[doc = "Field `WUPE21` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe21R = crate::FieldReader<Wupe21>;
impl Wupe21R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe21 {
        match self.bits {
            0 => Wupe21::Disable,
            1 => Wupe21::EnRiseHi,
            2 => Wupe21::EnFallLo,
            3 => Wupe21::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe21::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe21::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe21::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe21::EnAny
    }
}
#[doc = "Field `WUPE21` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe21W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe21, crate::Safe>;
impl<'a, REG> Wupe21W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe21::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe21::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe21::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe21::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe22 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe22> for u8 {
    #[inline(always)]
    fn from(variant: Wupe22) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe22 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe22 {}
#[doc = "Field `WUPE22` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe22R = crate::FieldReader<Wupe22>;
impl Wupe22R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe22 {
        match self.bits {
            0 => Wupe22::Disable,
            1 => Wupe22::EnRiseHi,
            2 => Wupe22::EnFallLo,
            3 => Wupe22::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe22::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe22::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe22::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe22::EnAny
    }
}
#[doc = "Field `WUPE22` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe22W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe22, crate::Safe>;
impl<'a, REG> Wupe22W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe22::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe22::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe22::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe22::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe23 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe23> for u8 {
    #[inline(always)]
    fn from(variant: Wupe23) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe23 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe23 {}
#[doc = "Field `WUPE23` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe23R = crate::FieldReader<Wupe23>;
impl Wupe23R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe23 {
        match self.bits {
            0 => Wupe23::Disable,
            1 => Wupe23::EnRiseHi,
            2 => Wupe23::EnFallLo,
            3 => Wupe23::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe23::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe23::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe23::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe23::EnAny
    }
}
#[doc = "Field `WUPE23` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe23W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe23, crate::Safe>;
impl<'a, REG> Wupe23W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe23::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe23::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe23::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe23::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe24 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe24> for u8 {
    #[inline(always)]
    fn from(variant: Wupe24) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe24 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe24 {}
#[doc = "Field `WUPE24` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe24R = crate::FieldReader<Wupe24>;
impl Wupe24R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe24 {
        match self.bits {
            0 => Wupe24::Disable,
            1 => Wupe24::EnRiseHi,
            2 => Wupe24::EnFallLo,
            3 => Wupe24::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe24::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe24::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe24::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe24::EnAny
    }
}
#[doc = "Field `WUPE24` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe24W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe24, crate::Safe>;
impl<'a, REG> Wupe24W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe24::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe24::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe24::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe24::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe25 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe25> for u8 {
    #[inline(always)]
    fn from(variant: Wupe25) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe25 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe25 {}
#[doc = "Field `WUPE25` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe25R = crate::FieldReader<Wupe25>;
impl Wupe25R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe25 {
        match self.bits {
            0 => Wupe25::Disable,
            1 => Wupe25::EnRiseHi,
            2 => Wupe25::EnFallLo,
            3 => Wupe25::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe25::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe25::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe25::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe25::EnAny
    }
}
#[doc = "Field `WUPE25` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe25W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe25, crate::Safe>;
impl<'a, REG> Wupe25W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe25::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe25::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe25::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe25::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe26 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe26> for u8 {
    #[inline(always)]
    fn from(variant: Wupe26) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe26 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe26 {}
#[doc = "Field `WUPE26` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe26R = crate::FieldReader<Wupe26>;
impl Wupe26R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe26 {
        match self.bits {
            0 => Wupe26::Disable,
            1 => Wupe26::EnRiseHi,
            2 => Wupe26::EnFallLo,
            3 => Wupe26::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe26::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe26::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe26::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe26::EnAny
    }
}
#[doc = "Field `WUPE26` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe26W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe26, crate::Safe>;
impl<'a, REG> Wupe26W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe26::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe26::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe26::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe26::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe27 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe27> for u8 {
    #[inline(always)]
    fn from(variant: Wupe27) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe27 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe27 {}
#[doc = "Field `WUPE27` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe27R = crate::FieldReader<Wupe27>;
impl Wupe27R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe27 {
        match self.bits {
            0 => Wupe27::Disable,
            1 => Wupe27::EnRiseHi,
            2 => Wupe27::EnFallLo,
            3 => Wupe27::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe27::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe27::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe27::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe27::EnAny
    }
}
#[doc = "Field `WUPE27` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe27W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe27, crate::Safe>;
impl<'a, REG> Wupe27W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe27::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe27::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe27::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe27::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe28 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe28> for u8 {
    #[inline(always)]
    fn from(variant: Wupe28) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe28 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe28 {}
#[doc = "Field `WUPE28` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe28R = crate::FieldReader<Wupe28>;
impl Wupe28R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe28 {
        match self.bits {
            0 => Wupe28::Disable,
            1 => Wupe28::EnRiseHi,
            2 => Wupe28::EnFallLo,
            3 => Wupe28::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe28::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe28::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe28::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe28::EnAny
    }
}
#[doc = "Field `WUPE28` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe28W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe28, crate::Safe>;
impl<'a, REG> Wupe28W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe28::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe28::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe28::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe28::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe29 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe29> for u8 {
    #[inline(always)]
    fn from(variant: Wupe29) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe29 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe29 {}
#[doc = "Field `WUPE29` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe29R = crate::FieldReader<Wupe29>;
impl Wupe29R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe29 {
        match self.bits {
            0 => Wupe29::Disable,
            1 => Wupe29::EnRiseHi,
            2 => Wupe29::EnFallLo,
            3 => Wupe29::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe29::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe29::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe29::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe29::EnAny
    }
}
#[doc = "Field `WUPE29` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe29W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe29, crate::Safe>;
impl<'a, REG> Wupe29W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe29::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe29::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe29::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe29::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe30 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe30> for u8 {
    #[inline(always)]
    fn from(variant: Wupe30) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe30 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe30 {}
#[doc = "Field `WUPE30` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe30R = crate::FieldReader<Wupe30>;
impl Wupe30R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe30 {
        match self.bits {
            0 => Wupe30::Disable,
            1 => Wupe30::EnRiseHi,
            2 => Wupe30::EnFallLo,
            3 => Wupe30::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe30::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe30::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe30::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe30::EnAny
    }
}
#[doc = "Field `WUPE30` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe30W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe30, crate::Safe>;
impl<'a, REG> Wupe30W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe30::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe30::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe30::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe30::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe31 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe31> for u8 {
    #[inline(always)]
    fn from(variant: Wupe31) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe31 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe31 {}
#[doc = "Field `WUPE31` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe31R = crate::FieldReader<Wupe31>;
impl Wupe31R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe31 {
        match self.bits {
            0 => Wupe31::Disable,
            1 => Wupe31::EnRiseHi,
            2 => Wupe31::EnFallLo,
            3 => Wupe31::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe31::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe31::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe31::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe31::EnAny
    }
}
#[doc = "Field `WUPE31` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe31W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe31, crate::Safe>;
impl<'a, REG> Wupe31W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe31::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe31::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe31::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe31::EnAny)
    }
}
impl R {
    #[doc = "Bits 0:1 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe16(&self) -> Wupe16R {
        Wupe16R::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 2:3 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe17(&self) -> Wupe17R {
        Wupe17R::new(((self.bits >> 2) & 3) as u8)
    }
    #[doc = "Bits 4:5 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe18(&self) -> Wupe18R {
        Wupe18R::new(((self.bits >> 4) & 3) as u8)
    }
    #[doc = "Bits 6:7 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe19(&self) -> Wupe19R {
        Wupe19R::new(((self.bits >> 6) & 3) as u8)
    }
    #[doc = "Bits 8:9 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe20(&self) -> Wupe20R {
        Wupe20R::new(((self.bits >> 8) & 3) as u8)
    }
    #[doc = "Bits 10:11 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe21(&self) -> Wupe21R {
        Wupe21R::new(((self.bits >> 10) & 3) as u8)
    }
    #[doc = "Bits 12:13 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe22(&self) -> Wupe22R {
        Wupe22R::new(((self.bits >> 12) & 3) as u8)
    }
    #[doc = "Bits 14:15 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe23(&self) -> Wupe23R {
        Wupe23R::new(((self.bits >> 14) & 3) as u8)
    }
    #[doc = "Bits 16:17 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe24(&self) -> Wupe24R {
        Wupe24R::new(((self.bits >> 16) & 3) as u8)
    }
    #[doc = "Bits 18:19 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe25(&self) -> Wupe25R {
        Wupe25R::new(((self.bits >> 18) & 3) as u8)
    }
    #[doc = "Bits 20:21 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe26(&self) -> Wupe26R {
        Wupe26R::new(((self.bits >> 20) & 3) as u8)
    }
    #[doc = "Bits 22:23 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe27(&self) -> Wupe27R {
        Wupe27R::new(((self.bits >> 22) & 3) as u8)
    }
    #[doc = "Bits 24:25 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe28(&self) -> Wupe28R {
        Wupe28R::new(((self.bits >> 24) & 3) as u8)
    }
    #[doc = "Bits 26:27 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe29(&self) -> Wupe29R {
        Wupe29R::new(((self.bits >> 26) & 3) as u8)
    }
    #[doc = "Bits 28:29 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe30(&self) -> Wupe30R {
        Wupe30R::new(((self.bits >> 28) & 3) as u8)
    }
    #[doc = "Bits 30:31 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe31(&self) -> Wupe31R {
        Wupe31R::new(((self.bits >> 30) & 3) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe16(&mut self) -> Wupe16W<Pe2Spec> {
        Wupe16W::new(self, 0)
    }
    #[doc = "Bits 2:3 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe17(&mut self) -> Wupe17W<Pe2Spec> {
        Wupe17W::new(self, 2)
    }
    #[doc = "Bits 4:5 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe18(&mut self) -> Wupe18W<Pe2Spec> {
        Wupe18W::new(self, 4)
    }
    #[doc = "Bits 6:7 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe19(&mut self) -> Wupe19W<Pe2Spec> {
        Wupe19W::new(self, 6)
    }
    #[doc = "Bits 8:9 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe20(&mut self) -> Wupe20W<Pe2Spec> {
        Wupe20W::new(self, 8)
    }
    #[doc = "Bits 10:11 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe21(&mut self) -> Wupe21W<Pe2Spec> {
        Wupe21W::new(self, 10)
    }
    #[doc = "Bits 12:13 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe22(&mut self) -> Wupe22W<Pe2Spec> {
        Wupe22W::new(self, 12)
    }
    #[doc = "Bits 14:15 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe23(&mut self) -> Wupe23W<Pe2Spec> {
        Wupe23W::new(self, 14)
    }
    #[doc = "Bits 16:17 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe24(&mut self) -> Wupe24W<Pe2Spec> {
        Wupe24W::new(self, 16)
    }
    #[doc = "Bits 18:19 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe25(&mut self) -> Wupe25W<Pe2Spec> {
        Wupe25W::new(self, 18)
    }
    #[doc = "Bits 20:21 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe26(&mut self) -> Wupe26W<Pe2Spec> {
        Wupe26W::new(self, 20)
    }
    #[doc = "Bits 22:23 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe27(&mut self) -> Wupe27W<Pe2Spec> {
        Wupe27W::new(self, 22)
    }
    #[doc = "Bits 24:25 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe28(&mut self) -> Wupe28W<Pe2Spec> {
        Wupe28W::new(self, 24)
    }
    #[doc = "Bits 26:27 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe29(&mut self) -> Wupe29W<Pe2Spec> {
        Wupe29W::new(self, 26)
    }
    #[doc = "Bits 28:29 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe30(&mut self) -> Wupe30W<Pe2Spec> {
        Wupe30W::new(self, 28)
    }
    #[doc = "Bits 30:31 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe31(&mut self) -> Wupe31W<Pe2Spec> {
        Wupe31W::new(self, 30)
    }
}
#[doc = "Pin Enable 2\n\nYou can [`read`](crate::Reg::read) this register and get [`pe2::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pe2::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Pe2Spec;
impl crate::RegisterSpec for Pe2Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pe2::R`](R) reader structure"]
impl crate::Readable for Pe2Spec {}
#[doc = "`write(|w| ..)` method takes [`pe2::W`](W) writer structure"]
impl crate::Writable for Pe2Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PE2 to value 0"]
impl crate::Resettable for Pe2Spec {}

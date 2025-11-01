#[doc = "Register `PE1` reader"]
pub type R = crate::R<Pe1Spec>;
#[doc = "Register `PE1` writer"]
pub type W = crate::W<Pe1Spec>;
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe0 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe0> for u8 {
    #[inline(always)]
    fn from(variant: Wupe0) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe0 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe0 {}
#[doc = "Field `WUPE0` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe0R = crate::FieldReader<Wupe0>;
impl Wupe0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe0 {
        match self.bits {
            0 => Wupe0::Disable,
            1 => Wupe0::EnRiseHi,
            2 => Wupe0::EnFallLo,
            3 => Wupe0::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe0::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe0::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe0::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe0::EnAny
    }
}
#[doc = "Field `WUPE0` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe0W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe0, crate::Safe>;
impl<'a, REG> Wupe0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe0::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe0::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe0::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe0::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe1 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe1> for u8 {
    #[inline(always)]
    fn from(variant: Wupe1) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe1 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe1 {}
#[doc = "Field `WUPE1` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe1R = crate::FieldReader<Wupe1>;
impl Wupe1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe1 {
        match self.bits {
            0 => Wupe1::Disable,
            1 => Wupe1::EnRiseHi,
            2 => Wupe1::EnFallLo,
            3 => Wupe1::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe1::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe1::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe1::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe1::EnAny
    }
}
#[doc = "Field `WUPE1` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe1W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe1, crate::Safe>;
impl<'a, REG> Wupe1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe1::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe1::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe1::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe1::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe2 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe2> for u8 {
    #[inline(always)]
    fn from(variant: Wupe2) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe2 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe2 {}
#[doc = "Field `WUPE2` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe2R = crate::FieldReader<Wupe2>;
impl Wupe2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe2 {
        match self.bits {
            0 => Wupe2::Disable,
            1 => Wupe2::EnRiseHi,
            2 => Wupe2::EnFallLo,
            3 => Wupe2::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe2::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe2::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe2::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe2::EnAny
    }
}
#[doc = "Field `WUPE2` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe2W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe2, crate::Safe>;
impl<'a, REG> Wupe2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe2::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe2::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe2::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe2::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe3 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe3> for u8 {
    #[inline(always)]
    fn from(variant: Wupe3) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe3 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe3 {}
#[doc = "Field `WUPE3` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe3R = crate::FieldReader<Wupe3>;
impl Wupe3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe3 {
        match self.bits {
            0 => Wupe3::Disable,
            1 => Wupe3::EnRiseHi,
            2 => Wupe3::EnFallLo,
            3 => Wupe3::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe3::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe3::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe3::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe3::EnAny
    }
}
#[doc = "Field `WUPE3` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe3W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe3, crate::Safe>;
impl<'a, REG> Wupe3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe3::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe3::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe3::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe3::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe4 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe4> for u8 {
    #[inline(always)]
    fn from(variant: Wupe4) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe4 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe4 {}
#[doc = "Field `WUPE4` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe4R = crate::FieldReader<Wupe4>;
impl Wupe4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe4 {
        match self.bits {
            0 => Wupe4::Disable,
            1 => Wupe4::EnRiseHi,
            2 => Wupe4::EnFallLo,
            3 => Wupe4::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe4::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe4::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe4::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe4::EnAny
    }
}
#[doc = "Field `WUPE4` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe4W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe4, crate::Safe>;
impl<'a, REG> Wupe4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe4::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe4::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe4::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe4::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe5 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe5> for u8 {
    #[inline(always)]
    fn from(variant: Wupe5) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe5 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe5 {}
#[doc = "Field `WUPE5` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe5R = crate::FieldReader<Wupe5>;
impl Wupe5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe5 {
        match self.bits {
            0 => Wupe5::Disable,
            1 => Wupe5::EnRiseHi,
            2 => Wupe5::EnFallLo,
            3 => Wupe5::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe5::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe5::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe5::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe5::EnAny
    }
}
#[doc = "Field `WUPE5` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe5W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe5, crate::Safe>;
impl<'a, REG> Wupe5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe5::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe5::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe5::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe5::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe6 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe6> for u8 {
    #[inline(always)]
    fn from(variant: Wupe6) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe6 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe6 {}
#[doc = "Field `WUPE6` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe6R = crate::FieldReader<Wupe6>;
impl Wupe6R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe6 {
        match self.bits {
            0 => Wupe6::Disable,
            1 => Wupe6::EnRiseHi,
            2 => Wupe6::EnFallLo,
            3 => Wupe6::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe6::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe6::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe6::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe6::EnAny
    }
}
#[doc = "Field `WUPE6` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe6W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe6, crate::Safe>;
impl<'a, REG> Wupe6W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe6::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe6::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe6::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe6::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe7 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe7> for u8 {
    #[inline(always)]
    fn from(variant: Wupe7) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe7 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe7 {}
#[doc = "Field `WUPE7` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe7R = crate::FieldReader<Wupe7>;
impl Wupe7R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe7 {
        match self.bits {
            0 => Wupe7::Disable,
            1 => Wupe7::EnRiseHi,
            2 => Wupe7::EnFallLo,
            3 => Wupe7::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe7::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe7::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe7::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe7::EnAny
    }
}
#[doc = "Field `WUPE7` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe7W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe7, crate::Safe>;
impl<'a, REG> Wupe7W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe7::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe7::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe7::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe7::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe8 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe8> for u8 {
    #[inline(always)]
    fn from(variant: Wupe8) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe8 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe8 {}
#[doc = "Field `WUPE8` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe8R = crate::FieldReader<Wupe8>;
impl Wupe8R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe8 {
        match self.bits {
            0 => Wupe8::Disable,
            1 => Wupe8::EnRiseHi,
            2 => Wupe8::EnFallLo,
            3 => Wupe8::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe8::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe8::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe8::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe8::EnAny
    }
}
#[doc = "Field `WUPE8` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe8W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe8, crate::Safe>;
impl<'a, REG> Wupe8W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe8::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe8::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe8::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe8::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe9 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe9> for u8 {
    #[inline(always)]
    fn from(variant: Wupe9) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe9 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe9 {}
#[doc = "Field `WUPE9` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe9R = crate::FieldReader<Wupe9>;
impl Wupe9R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe9 {
        match self.bits {
            0 => Wupe9::Disable,
            1 => Wupe9::EnRiseHi,
            2 => Wupe9::EnFallLo,
            3 => Wupe9::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe9::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe9::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe9::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe9::EnAny
    }
}
#[doc = "Field `WUPE9` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe9W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe9, crate::Safe>;
impl<'a, REG> Wupe9W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe9::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe9::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe9::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe9::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe10 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe10> for u8 {
    #[inline(always)]
    fn from(variant: Wupe10) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe10 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe10 {}
#[doc = "Field `WUPE10` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe10R = crate::FieldReader<Wupe10>;
impl Wupe10R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe10 {
        match self.bits {
            0 => Wupe10::Disable,
            1 => Wupe10::EnRiseHi,
            2 => Wupe10::EnFallLo,
            3 => Wupe10::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe10::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe10::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe10::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe10::EnAny
    }
}
#[doc = "Field `WUPE10` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe10W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe10, crate::Safe>;
impl<'a, REG> Wupe10W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe10::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe10::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe10::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe10::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe11 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe11> for u8 {
    #[inline(always)]
    fn from(variant: Wupe11) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe11 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe11 {}
#[doc = "Field `WUPE11` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe11R = crate::FieldReader<Wupe11>;
impl Wupe11R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe11 {
        match self.bits {
            0 => Wupe11::Disable,
            1 => Wupe11::EnRiseHi,
            2 => Wupe11::EnFallLo,
            3 => Wupe11::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe11::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe11::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe11::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe11::EnAny
    }
}
#[doc = "Field `WUPE11` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe11W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe11, crate::Safe>;
impl<'a, REG> Wupe11W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe11::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe11::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe11::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe11::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe12 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe12> for u8 {
    #[inline(always)]
    fn from(variant: Wupe12) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe12 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe12 {}
#[doc = "Field `WUPE12` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe12R = crate::FieldReader<Wupe12>;
impl Wupe12R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe12 {
        match self.bits {
            0 => Wupe12::Disable,
            1 => Wupe12::EnRiseHi,
            2 => Wupe12::EnFallLo,
            3 => Wupe12::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe12::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe12::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe12::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe12::EnAny
    }
}
#[doc = "Field `WUPE12` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe12W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe12, crate::Safe>;
impl<'a, REG> Wupe12W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe12::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe12::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe12::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe12::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe13 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe13> for u8 {
    #[inline(always)]
    fn from(variant: Wupe13) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe13 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe13 {}
#[doc = "Field `WUPE13` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe13R = crate::FieldReader<Wupe13>;
impl Wupe13R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe13 {
        match self.bits {
            0 => Wupe13::Disable,
            1 => Wupe13::EnRiseHi,
            2 => Wupe13::EnFallLo,
            3 => Wupe13::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe13::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe13::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe13::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe13::EnAny
    }
}
#[doc = "Field `WUPE13` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe13W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe13, crate::Safe>;
impl<'a, REG> Wupe13W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe13::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe13::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe13::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe13::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe14 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe14> for u8 {
    #[inline(always)]
    fn from(variant: Wupe14) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe14 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe14 {}
#[doc = "Field `WUPE14` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe14R = crate::FieldReader<Wupe14>;
impl Wupe14R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe14 {
        match self.bits {
            0 => Wupe14::Disable,
            1 => Wupe14::EnRiseHi,
            2 => Wupe14::EnFallLo,
            3 => Wupe14::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe14::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe14::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe14::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe14::EnAny
    }
}
#[doc = "Field `WUPE14` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe14W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe14, crate::Safe>;
impl<'a, REG> Wupe14W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe14::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe14::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe14::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe14::EnAny)
    }
}
#[doc = "Wake-up Pin Enable for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupe15 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (detect on any edge)"]
    EnAny = 3,
}
impl From<Wupe15> for u8 {
    #[inline(always)]
    fn from(variant: Wupe15) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupe15 {
    type Ux = u8;
}
impl crate::IsEnum for Wupe15 {}
#[doc = "Field `WUPE15` reader - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe15R = crate::FieldReader<Wupe15>;
impl Wupe15R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupe15 {
        match self.bits {
            0 => Wupe15::Disable,
            1 => Wupe15::EnRiseHi,
            2 => Wupe15::EnFallLo,
            3 => Wupe15::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wupe15::Disable
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Wupe15::EnRiseHi
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Wupe15::EnFallLo
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Wupe15::EnAny
    }
}
#[doc = "Field `WUPE15` writer - Wake-up Pin Enable for WUU_Pn"]
pub type Wupe15W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupe15, crate::Safe>;
impl<'a, REG> Wupe15W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe15::Disable)
    }
    #[doc = "Enable (detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe15::EnRiseHi)
    }
    #[doc = "Enable (detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe15::EnFallLo)
    }
    #[doc = "Enable (detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Wupe15::EnAny)
    }
}
impl R {
    #[doc = "Bits 0:1 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe0(&self) -> Wupe0R {
        Wupe0R::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 2:3 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe1(&self) -> Wupe1R {
        Wupe1R::new(((self.bits >> 2) & 3) as u8)
    }
    #[doc = "Bits 4:5 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe2(&self) -> Wupe2R {
        Wupe2R::new(((self.bits >> 4) & 3) as u8)
    }
    #[doc = "Bits 6:7 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe3(&self) -> Wupe3R {
        Wupe3R::new(((self.bits >> 6) & 3) as u8)
    }
    #[doc = "Bits 8:9 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe4(&self) -> Wupe4R {
        Wupe4R::new(((self.bits >> 8) & 3) as u8)
    }
    #[doc = "Bits 10:11 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe5(&self) -> Wupe5R {
        Wupe5R::new(((self.bits >> 10) & 3) as u8)
    }
    #[doc = "Bits 12:13 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe6(&self) -> Wupe6R {
        Wupe6R::new(((self.bits >> 12) & 3) as u8)
    }
    #[doc = "Bits 14:15 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe7(&self) -> Wupe7R {
        Wupe7R::new(((self.bits >> 14) & 3) as u8)
    }
    #[doc = "Bits 16:17 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe8(&self) -> Wupe8R {
        Wupe8R::new(((self.bits >> 16) & 3) as u8)
    }
    #[doc = "Bits 18:19 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe9(&self) -> Wupe9R {
        Wupe9R::new(((self.bits >> 18) & 3) as u8)
    }
    #[doc = "Bits 20:21 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe10(&self) -> Wupe10R {
        Wupe10R::new(((self.bits >> 20) & 3) as u8)
    }
    #[doc = "Bits 22:23 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe11(&self) -> Wupe11R {
        Wupe11R::new(((self.bits >> 22) & 3) as u8)
    }
    #[doc = "Bits 24:25 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe12(&self) -> Wupe12R {
        Wupe12R::new(((self.bits >> 24) & 3) as u8)
    }
    #[doc = "Bits 26:27 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe13(&self) -> Wupe13R {
        Wupe13R::new(((self.bits >> 26) & 3) as u8)
    }
    #[doc = "Bits 28:29 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe14(&self) -> Wupe14R {
        Wupe14R::new(((self.bits >> 28) & 3) as u8)
    }
    #[doc = "Bits 30:31 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe15(&self) -> Wupe15R {
        Wupe15R::new(((self.bits >> 30) & 3) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe0(&mut self) -> Wupe0W<Pe1Spec> {
        Wupe0W::new(self, 0)
    }
    #[doc = "Bits 2:3 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe1(&mut self) -> Wupe1W<Pe1Spec> {
        Wupe1W::new(self, 2)
    }
    #[doc = "Bits 4:5 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe2(&mut self) -> Wupe2W<Pe1Spec> {
        Wupe2W::new(self, 4)
    }
    #[doc = "Bits 6:7 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe3(&mut self) -> Wupe3W<Pe1Spec> {
        Wupe3W::new(self, 6)
    }
    #[doc = "Bits 8:9 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe4(&mut self) -> Wupe4W<Pe1Spec> {
        Wupe4W::new(self, 8)
    }
    #[doc = "Bits 10:11 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe5(&mut self) -> Wupe5W<Pe1Spec> {
        Wupe5W::new(self, 10)
    }
    #[doc = "Bits 12:13 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe6(&mut self) -> Wupe6W<Pe1Spec> {
        Wupe6W::new(self, 12)
    }
    #[doc = "Bits 14:15 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe7(&mut self) -> Wupe7W<Pe1Spec> {
        Wupe7W::new(self, 14)
    }
    #[doc = "Bits 16:17 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe8(&mut self) -> Wupe8W<Pe1Spec> {
        Wupe8W::new(self, 16)
    }
    #[doc = "Bits 18:19 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe9(&mut self) -> Wupe9W<Pe1Spec> {
        Wupe9W::new(self, 18)
    }
    #[doc = "Bits 20:21 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe10(&mut self) -> Wupe10W<Pe1Spec> {
        Wupe10W::new(self, 20)
    }
    #[doc = "Bits 22:23 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe11(&mut self) -> Wupe11W<Pe1Spec> {
        Wupe11W::new(self, 22)
    }
    #[doc = "Bits 24:25 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe12(&mut self) -> Wupe12W<Pe1Spec> {
        Wupe12W::new(self, 24)
    }
    #[doc = "Bits 26:27 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe13(&mut self) -> Wupe13W<Pe1Spec> {
        Wupe13W::new(self, 26)
    }
    #[doc = "Bits 28:29 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe14(&mut self) -> Wupe14W<Pe1Spec> {
        Wupe14W::new(self, 28)
    }
    #[doc = "Bits 30:31 - Wake-up Pin Enable for WUU_Pn"]
    #[inline(always)]
    pub fn wupe15(&mut self) -> Wupe15W<Pe1Spec> {
        Wupe15W::new(self, 30)
    }
}
#[doc = "Pin Enable 1\n\nYou can [`read`](crate::Reg::read) this register and get [`pe1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pe1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Pe1Spec;
impl crate::RegisterSpec for Pe1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pe1::R`](R) reader structure"]
impl crate::Readable for Pe1Spec {}
#[doc = "`write(|w| ..)` method takes [`pe1::W`](W) writer structure"]
impl crate::Writable for Pe1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PE1 to value 0"]
impl crate::Resettable for Pe1Spec {}

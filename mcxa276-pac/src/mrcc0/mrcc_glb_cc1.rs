#[doc = "Register `MRCC_GLB_CC1` reader"]
pub type R = crate::R<MrccGlbCc1Spec>;
#[doc = "Register `MRCC_GLB_CC1` writer"]
pub type W = crate::W<MrccGlbCc1Spec>;
#[doc = "FLEXPWM1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flexpwm1 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Flexpwm1> for bool {
    #[inline(always)]
    fn from(variant: Flexpwm1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FLEXPWM1` reader - FLEXPWM1"]
pub type Flexpwm1R = crate::BitReader<Flexpwm1>;
impl Flexpwm1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Flexpwm1 {
        match self.bits {
            false => Flexpwm1::Disabled,
            true => Flexpwm1::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Flexpwm1::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Flexpwm1::Enabled
    }
}
#[doc = "Field `FLEXPWM1` writer - FLEXPWM1"]
pub type Flexpwm1W<'a, REG> = crate::BitWriter<'a, REG, Flexpwm1>;
impl<'a, REG> Flexpwm1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Flexpwm1::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Flexpwm1::Enabled)
    }
}
#[doc = "OSTIMER0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ostimer0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Ostimer0> for bool {
    #[inline(always)]
    fn from(variant: Ostimer0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OSTIMER0` reader - OSTIMER0"]
pub type Ostimer0R = crate::BitReader<Ostimer0>;
impl Ostimer0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ostimer0 {
        match self.bits {
            false => Ostimer0::Disabled,
            true => Ostimer0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Ostimer0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Ostimer0::Enabled
    }
}
#[doc = "Field `OSTIMER0` writer - OSTIMER0"]
pub type Ostimer0W<'a, REG> = crate::BitWriter<'a, REG, Ostimer0>;
impl<'a, REG> Ostimer0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ostimer0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ostimer0::Enabled)
    }
}
#[doc = "ADC0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Adc0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Adc0> for bool {
    #[inline(always)]
    fn from(variant: Adc0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ADC0` reader - ADC0"]
pub type Adc0R = crate::BitReader<Adc0>;
impl Adc0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Adc0 {
        match self.bits {
            false => Adc0::Disabled,
            true => Adc0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Adc0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Adc0::Enabled
    }
}
#[doc = "Field `ADC0` writer - ADC0"]
pub type Adc0W<'a, REG> = crate::BitWriter<'a, REG, Adc0>;
impl<'a, REG> Adc0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Adc0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Adc0::Enabled)
    }
}
#[doc = "ADC1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Adc1 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Adc1> for bool {
    #[inline(always)]
    fn from(variant: Adc1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ADC1` reader - ADC1"]
pub type Adc1R = crate::BitReader<Adc1>;
impl Adc1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Adc1 {
        match self.bits {
            false => Adc1::Disabled,
            true => Adc1::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Adc1::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Adc1::Enabled
    }
}
#[doc = "Field `ADC1` writer - ADC1"]
pub type Adc1W<'a, REG> = crate::BitWriter<'a, REG, Adc1>;
impl<'a, REG> Adc1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Adc1::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Adc1::Enabled)
    }
}
#[doc = "CMP0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmp0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Cmp0> for bool {
    #[inline(always)]
    fn from(variant: Cmp0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CMP0` reader - CMP0"]
pub type Cmp0R = crate::BitReader<Cmp0>;
impl Cmp0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cmp0 {
        match self.bits {
            false => Cmp0::Disabled,
            true => Cmp0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Cmp0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Cmp0::Enabled
    }
}
#[doc = "Field `CMP0` writer - CMP0"]
pub type Cmp0W<'a, REG> = crate::BitWriter<'a, REG, Cmp0>;
impl<'a, REG> Cmp0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Cmp0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Cmp0::Enabled)
    }
}
#[doc = "CMP1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmp1 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Cmp1> for bool {
    #[inline(always)]
    fn from(variant: Cmp1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CMP1` reader - CMP1"]
pub type Cmp1R = crate::BitReader<Cmp1>;
impl Cmp1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cmp1 {
        match self.bits {
            false => Cmp1::Disabled,
            true => Cmp1::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Cmp1::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Cmp1::Enabled
    }
}
#[doc = "Field `CMP1` writer - CMP1"]
pub type Cmp1W<'a, REG> = crate::BitWriter<'a, REG, Cmp1>;
impl<'a, REG> Cmp1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Cmp1::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Cmp1::Enabled)
    }
}
#[doc = "CMP2\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmp2 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Cmp2> for bool {
    #[inline(always)]
    fn from(variant: Cmp2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CMP2` reader - CMP2"]
pub type Cmp2R = crate::BitReader<Cmp2>;
impl Cmp2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cmp2 {
        match self.bits {
            false => Cmp2::Disabled,
            true => Cmp2::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Cmp2::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Cmp2::Enabled
    }
}
#[doc = "Field `CMP2` writer - CMP2"]
pub type Cmp2W<'a, REG> = crate::BitWriter<'a, REG, Cmp2>;
impl<'a, REG> Cmp2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Cmp2::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Cmp2::Enabled)
    }
}
#[doc = "DAC0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dac0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Dac0> for bool {
    #[inline(always)]
    fn from(variant: Dac0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DAC0` reader - DAC0"]
pub type Dac0R = crate::BitReader<Dac0>;
impl Dac0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dac0 {
        match self.bits {
            false => Dac0::Disabled,
            true => Dac0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Dac0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Dac0::Enabled
    }
}
#[doc = "Field `DAC0` writer - DAC0"]
pub type Dac0W<'a, REG> = crate::BitWriter<'a, REG, Dac0>;
impl<'a, REG> Dac0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Dac0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Dac0::Enabled)
    }
}
#[doc = "OPAMP0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opamp0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Opamp0> for bool {
    #[inline(always)]
    fn from(variant: Opamp0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OPAMP0` reader - OPAMP0"]
pub type Opamp0R = crate::BitReader<Opamp0>;
impl Opamp0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Opamp0 {
        match self.bits {
            false => Opamp0::Disabled,
            true => Opamp0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Opamp0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Opamp0::Enabled
    }
}
#[doc = "Field `OPAMP0` writer - OPAMP0"]
pub type Opamp0W<'a, REG> = crate::BitWriter<'a, REG, Opamp0>;
impl<'a, REG> Opamp0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Opamp0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Opamp0::Enabled)
    }
}
#[doc = "OPAMP1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opamp1 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Opamp1> for bool {
    #[inline(always)]
    fn from(variant: Opamp1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OPAMP1` reader - OPAMP1"]
pub type Opamp1R = crate::BitReader<Opamp1>;
impl Opamp1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Opamp1 {
        match self.bits {
            false => Opamp1::Disabled,
            true => Opamp1::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Opamp1::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Opamp1::Enabled
    }
}
#[doc = "Field `OPAMP1` writer - OPAMP1"]
pub type Opamp1W<'a, REG> = crate::BitWriter<'a, REG, Opamp1>;
impl<'a, REG> Opamp1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Opamp1::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Opamp1::Enabled)
    }
}
#[doc = "OPAMP2\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opamp2 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Opamp2> for bool {
    #[inline(always)]
    fn from(variant: Opamp2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OPAMP2` reader - OPAMP2"]
pub type Opamp2R = crate::BitReader<Opamp2>;
impl Opamp2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Opamp2 {
        match self.bits {
            false => Opamp2::Disabled,
            true => Opamp2::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Opamp2::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Opamp2::Enabled
    }
}
#[doc = "Field `OPAMP2` writer - OPAMP2"]
pub type Opamp2W<'a, REG> = crate::BitWriter<'a, REG, Opamp2>;
impl<'a, REG> Opamp2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Opamp2::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Opamp2::Enabled)
    }
}
#[doc = "OPAMP3\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opamp3 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Opamp3> for bool {
    #[inline(always)]
    fn from(variant: Opamp3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OPAMP3` reader - OPAMP3"]
pub type Opamp3R = crate::BitReader<Opamp3>;
impl Opamp3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Opamp3 {
        match self.bits {
            false => Opamp3::Disabled,
            true => Opamp3::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Opamp3::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Opamp3::Enabled
    }
}
#[doc = "Field `OPAMP3` writer - OPAMP3"]
pub type Opamp3W<'a, REG> = crate::BitWriter<'a, REG, Opamp3>;
impl<'a, REG> Opamp3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Opamp3::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Opamp3::Enabled)
    }
}
#[doc = "PORT0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Port0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Port0> for bool {
    #[inline(always)]
    fn from(variant: Port0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PORT0` reader - PORT0"]
pub type Port0R = crate::BitReader<Port0>;
impl Port0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Port0 {
        match self.bits {
            false => Port0::Disabled,
            true => Port0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Port0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Port0::Enabled
    }
}
#[doc = "Field `PORT0` writer - PORT0"]
pub type Port0W<'a, REG> = crate::BitWriter<'a, REG, Port0>;
impl<'a, REG> Port0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Port0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Port0::Enabled)
    }
}
#[doc = "PORT1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Port1 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Port1> for bool {
    #[inline(always)]
    fn from(variant: Port1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PORT1` reader - PORT1"]
pub type Port1R = crate::BitReader<Port1>;
impl Port1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Port1 {
        match self.bits {
            false => Port1::Disabled,
            true => Port1::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Port1::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Port1::Enabled
    }
}
#[doc = "Field `PORT1` writer - PORT1"]
pub type Port1W<'a, REG> = crate::BitWriter<'a, REG, Port1>;
impl<'a, REG> Port1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Port1::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Port1::Enabled)
    }
}
#[doc = "PORT2\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Port2 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Port2> for bool {
    #[inline(always)]
    fn from(variant: Port2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PORT2` reader - PORT2"]
pub type Port2R = crate::BitReader<Port2>;
impl Port2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Port2 {
        match self.bits {
            false => Port2::Disabled,
            true => Port2::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Port2::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Port2::Enabled
    }
}
#[doc = "Field `PORT2` writer - PORT2"]
pub type Port2W<'a, REG> = crate::BitWriter<'a, REG, Port2>;
impl<'a, REG> Port2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Port2::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Port2::Enabled)
    }
}
#[doc = "PORT3\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Port3 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Port3> for bool {
    #[inline(always)]
    fn from(variant: Port3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PORT3` reader - PORT3"]
pub type Port3R = crate::BitReader<Port3>;
impl Port3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Port3 {
        match self.bits {
            false => Port3::Disabled,
            true => Port3::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Port3::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Port3::Enabled
    }
}
#[doc = "Field `PORT3` writer - PORT3"]
pub type Port3W<'a, REG> = crate::BitWriter<'a, REG, Port3>;
impl<'a, REG> Port3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Port3::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Port3::Enabled)
    }
}
#[doc = "PORT4\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Port4 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Port4> for bool {
    #[inline(always)]
    fn from(variant: Port4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PORT4` reader - PORT4"]
pub type Port4R = crate::BitReader<Port4>;
impl Port4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Port4 {
        match self.bits {
            false => Port4::Disabled,
            true => Port4::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Port4::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Port4::Enabled
    }
}
#[doc = "Field `PORT4` writer - PORT4"]
pub type Port4W<'a, REG> = crate::BitWriter<'a, REG, Port4>;
impl<'a, REG> Port4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Port4::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Port4::Enabled)
    }
}
#[doc = "SLCD0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slcd0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Slcd0> for bool {
    #[inline(always)]
    fn from(variant: Slcd0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SLCD0` reader - SLCD0"]
pub type Slcd0R = crate::BitReader<Slcd0>;
impl Slcd0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Slcd0 {
        match self.bits {
            false => Slcd0::Disabled,
            true => Slcd0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Slcd0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Slcd0::Enabled
    }
}
#[doc = "Field `SLCD0` writer - SLCD0"]
pub type Slcd0W<'a, REG> = crate::BitWriter<'a, REG, Slcd0>;
impl<'a, REG> Slcd0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Slcd0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Slcd0::Enabled)
    }
}
#[doc = "FLEXCAN0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flexcan0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Flexcan0> for bool {
    #[inline(always)]
    fn from(variant: Flexcan0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FLEXCAN0` reader - FLEXCAN0"]
pub type Flexcan0R = crate::BitReader<Flexcan0>;
impl Flexcan0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Flexcan0 {
        match self.bits {
            false => Flexcan0::Disabled,
            true => Flexcan0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Flexcan0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Flexcan0::Enabled
    }
}
#[doc = "Field `FLEXCAN0` writer - FLEXCAN0"]
pub type Flexcan0W<'a, REG> = crate::BitWriter<'a, REG, Flexcan0>;
impl<'a, REG> Flexcan0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Flexcan0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Flexcan0::Enabled)
    }
}
#[doc = "FLEXCAN1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flexcan1 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Flexcan1> for bool {
    #[inline(always)]
    fn from(variant: Flexcan1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FLEXCAN1` reader - FLEXCAN1"]
pub type Flexcan1R = crate::BitReader<Flexcan1>;
impl Flexcan1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Flexcan1 {
        match self.bits {
            false => Flexcan1::Disabled,
            true => Flexcan1::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Flexcan1::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Flexcan1::Enabled
    }
}
#[doc = "Field `FLEXCAN1` writer - FLEXCAN1"]
pub type Flexcan1W<'a, REG> = crate::BitWriter<'a, REG, Flexcan1>;
impl<'a, REG> Flexcan1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Flexcan1::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Flexcan1::Enabled)
    }
}
#[doc = "LPI2C2\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lpi2c2 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Lpi2c2> for bool {
    #[inline(always)]
    fn from(variant: Lpi2c2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LPI2C2` reader - LPI2C2"]
pub type Lpi2c2R = crate::BitReader<Lpi2c2>;
impl Lpi2c2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lpi2c2 {
        match self.bits {
            false => Lpi2c2::Disabled,
            true => Lpi2c2::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Lpi2c2::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Lpi2c2::Enabled
    }
}
#[doc = "Field `LPI2C2` writer - LPI2C2"]
pub type Lpi2c2W<'a, REG> = crate::BitWriter<'a, REG, Lpi2c2>;
impl<'a, REG> Lpi2c2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpi2c2::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpi2c2::Enabled)
    }
}
#[doc = "LPI2C3\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lpi2c3 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Lpi2c3> for bool {
    #[inline(always)]
    fn from(variant: Lpi2c3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LPI2C3` reader - LPI2C3"]
pub type Lpi2c3R = crate::BitReader<Lpi2c3>;
impl Lpi2c3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lpi2c3 {
        match self.bits {
            false => Lpi2c3::Disabled,
            true => Lpi2c3::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Lpi2c3::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Lpi2c3::Enabled
    }
}
#[doc = "Field `LPI2C3` writer - LPI2C3"]
pub type Lpi2c3W<'a, REG> = crate::BitWriter<'a, REG, Lpi2c3>;
impl<'a, REG> Lpi2c3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpi2c3::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpi2c3::Enabled)
    }
}
#[doc = "LPUART5\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lpuart5 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Lpuart5> for bool {
    #[inline(always)]
    fn from(variant: Lpuart5) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LPUART5` reader - LPUART5"]
pub type Lpuart5R = crate::BitReader<Lpuart5>;
impl Lpuart5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lpuart5 {
        match self.bits {
            false => Lpuart5::Disabled,
            true => Lpuart5::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Lpuart5::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Lpuart5::Enabled
    }
}
#[doc = "Field `LPUART5` writer - LPUART5"]
pub type Lpuart5W<'a, REG> = crate::BitWriter<'a, REG, Lpuart5>;
impl<'a, REG> Lpuart5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpuart5::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpuart5::Enabled)
    }
}
#[doc = "TDET0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tdet0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Tdet0> for bool {
    #[inline(always)]
    fn from(variant: Tdet0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TDET0` reader - TDET0"]
pub type Tdet0R = crate::BitReader<Tdet0>;
impl Tdet0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tdet0 {
        match self.bits {
            false => Tdet0::Disabled,
            true => Tdet0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Tdet0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Tdet0::Enabled
    }
}
#[doc = "Field `TDET0` writer - TDET0"]
pub type Tdet0W<'a, REG> = crate::BitWriter<'a, REG, Tdet0>;
impl<'a, REG> Tdet0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Tdet0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Tdet0::Enabled)
    }
}
#[doc = "PKC0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pkc0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Pkc0> for bool {
    #[inline(always)]
    fn from(variant: Pkc0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PKC0` reader - PKC0"]
pub type Pkc0R = crate::BitReader<Pkc0>;
impl Pkc0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pkc0 {
        match self.bits {
            false => Pkc0::Disabled,
            true => Pkc0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Pkc0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Pkc0::Enabled
    }
}
#[doc = "Field `PKC0` writer - PKC0"]
pub type Pkc0W<'a, REG> = crate::BitWriter<'a, REG, Pkc0>;
impl<'a, REG> Pkc0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Pkc0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Pkc0::Enabled)
    }
}
#[doc = "SGI0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sgi0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Sgi0> for bool {
    #[inline(always)]
    fn from(variant: Sgi0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SGI0` reader - SGI0"]
pub type Sgi0R = crate::BitReader<Sgi0>;
impl Sgi0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sgi0 {
        match self.bits {
            false => Sgi0::Disabled,
            true => Sgi0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Sgi0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Sgi0::Enabled
    }
}
#[doc = "Field `SGI0` writer - SGI0"]
pub type Sgi0W<'a, REG> = crate::BitWriter<'a, REG, Sgi0>;
impl<'a, REG> Sgi0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Sgi0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Sgi0::Enabled)
    }
}
#[doc = "TRNG0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trng0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Trng0> for bool {
    #[inline(always)]
    fn from(variant: Trng0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TRNG0` reader - TRNG0"]
pub type Trng0R = crate::BitReader<Trng0>;
impl Trng0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Trng0 {
        match self.bits {
            false => Trng0::Disabled,
            true => Trng0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Trng0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Trng0::Enabled
    }
}
#[doc = "Field `TRNG0` writer - TRNG0"]
pub type Trng0W<'a, REG> = crate::BitWriter<'a, REG, Trng0>;
impl<'a, REG> Trng0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Trng0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Trng0::Enabled)
    }
}
#[doc = "UDF0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Udf0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Udf0> for bool {
    #[inline(always)]
    fn from(variant: Udf0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `UDF0` reader - UDF0"]
pub type Udf0R = crate::BitReader<Udf0>;
impl Udf0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Udf0 {
        match self.bits {
            false => Udf0::Disabled,
            true => Udf0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Udf0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Udf0::Enabled
    }
}
#[doc = "Field `UDF0` writer - UDF0"]
pub type Udf0W<'a, REG> = crate::BitWriter<'a, REG, Udf0>;
impl<'a, REG> Udf0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Udf0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Udf0::Enabled)
    }
}
#[doc = "ADC2\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Adc2 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Adc2> for bool {
    #[inline(always)]
    fn from(variant: Adc2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ADC2` reader - ADC2"]
pub type Adc2R = crate::BitReader<Adc2>;
impl Adc2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Adc2 {
        match self.bits {
            false => Adc2::Disabled,
            true => Adc2::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Adc2::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Adc2::Enabled
    }
}
#[doc = "Field `ADC2` writer - ADC2"]
pub type Adc2W<'a, REG> = crate::BitWriter<'a, REG, Adc2>;
impl<'a, REG> Adc2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Adc2::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Adc2::Enabled)
    }
}
#[doc = "ADC3\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Adc3 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Adc3> for bool {
    #[inline(always)]
    fn from(variant: Adc3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ADC3` reader - ADC3"]
pub type Adc3R = crate::BitReader<Adc3>;
impl Adc3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Adc3 {
        match self.bits {
            false => Adc3::Disabled,
            true => Adc3::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Adc3::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Adc3::Enabled
    }
}
#[doc = "Field `ADC3` writer - ADC3"]
pub type Adc3W<'a, REG> = crate::BitWriter<'a, REG, Adc3>;
impl<'a, REG> Adc3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Adc3::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Adc3::Enabled)
    }
}
impl R {
    #[doc = "Bit 0 - FLEXPWM1"]
    #[inline(always)]
    pub fn flexpwm1(&self) -> Flexpwm1R {
        Flexpwm1R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - OSTIMER0"]
    #[inline(always)]
    pub fn ostimer0(&self) -> Ostimer0R {
        Ostimer0R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - ADC0"]
    #[inline(always)]
    pub fn adc0(&self) -> Adc0R {
        Adc0R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - ADC1"]
    #[inline(always)]
    pub fn adc1(&self) -> Adc1R {
        Adc1R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - CMP0"]
    #[inline(always)]
    pub fn cmp0(&self) -> Cmp0R {
        Cmp0R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - CMP1"]
    #[inline(always)]
    pub fn cmp1(&self) -> Cmp1R {
        Cmp1R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - CMP2"]
    #[inline(always)]
    pub fn cmp2(&self) -> Cmp2R {
        Cmp2R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - DAC0"]
    #[inline(always)]
    pub fn dac0(&self) -> Dac0R {
        Dac0R::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - OPAMP0"]
    #[inline(always)]
    pub fn opamp0(&self) -> Opamp0R {
        Opamp0R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - OPAMP1"]
    #[inline(always)]
    pub fn opamp1(&self) -> Opamp1R {
        Opamp1R::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - OPAMP2"]
    #[inline(always)]
    pub fn opamp2(&self) -> Opamp2R {
        Opamp2R::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - OPAMP3"]
    #[inline(always)]
    pub fn opamp3(&self) -> Opamp3R {
        Opamp3R::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - PORT0"]
    #[inline(always)]
    pub fn port0(&self) -> Port0R {
        Port0R::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - PORT1"]
    #[inline(always)]
    pub fn port1(&self) -> Port1R {
        Port1R::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - PORT2"]
    #[inline(always)]
    pub fn port2(&self) -> Port2R {
        Port2R::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - PORT3"]
    #[inline(always)]
    pub fn port3(&self) -> Port3R {
        Port3R::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - PORT4"]
    #[inline(always)]
    pub fn port4(&self) -> Port4R {
        Port4R::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - SLCD0"]
    #[inline(always)]
    pub fn slcd0(&self) -> Slcd0R {
        Slcd0R::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - FLEXCAN0"]
    #[inline(always)]
    pub fn flexcan0(&self) -> Flexcan0R {
        Flexcan0R::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - FLEXCAN1"]
    #[inline(always)]
    pub fn flexcan1(&self) -> Flexcan1R {
        Flexcan1R::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - LPI2C2"]
    #[inline(always)]
    pub fn lpi2c2(&self) -> Lpi2c2R {
        Lpi2c2R::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - LPI2C3"]
    #[inline(always)]
    pub fn lpi2c3(&self) -> Lpi2c3R {
        Lpi2c3R::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - LPUART5"]
    #[inline(always)]
    pub fn lpuart5(&self) -> Lpuart5R {
        Lpuart5R::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - TDET0"]
    #[inline(always)]
    pub fn tdet0(&self) -> Tdet0R {
        Tdet0R::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bit 24 - PKC0"]
    #[inline(always)]
    pub fn pkc0(&self) -> Pkc0R {
        Pkc0R::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - SGI0"]
    #[inline(always)]
    pub fn sgi0(&self) -> Sgi0R {
        Sgi0R::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - TRNG0"]
    #[inline(always)]
    pub fn trng0(&self) -> Trng0R {
        Trng0R::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - UDF0"]
    #[inline(always)]
    pub fn udf0(&self) -> Udf0R {
        Udf0R::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bit 28 - ADC2"]
    #[inline(always)]
    pub fn adc2(&self) -> Adc2R {
        Adc2R::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 29 - ADC3"]
    #[inline(always)]
    pub fn adc3(&self) -> Adc3R {
        Adc3R::new(((self.bits >> 29) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - FLEXPWM1"]
    #[inline(always)]
    pub fn flexpwm1(&mut self) -> Flexpwm1W<MrccGlbCc1Spec> {
        Flexpwm1W::new(self, 0)
    }
    #[doc = "Bit 1 - OSTIMER0"]
    #[inline(always)]
    pub fn ostimer0(&mut self) -> Ostimer0W<MrccGlbCc1Spec> {
        Ostimer0W::new(self, 1)
    }
    #[doc = "Bit 2 - ADC0"]
    #[inline(always)]
    pub fn adc0(&mut self) -> Adc0W<MrccGlbCc1Spec> {
        Adc0W::new(self, 2)
    }
    #[doc = "Bit 3 - ADC1"]
    #[inline(always)]
    pub fn adc1(&mut self) -> Adc1W<MrccGlbCc1Spec> {
        Adc1W::new(self, 3)
    }
    #[doc = "Bit 4 - CMP0"]
    #[inline(always)]
    pub fn cmp0(&mut self) -> Cmp0W<MrccGlbCc1Spec> {
        Cmp0W::new(self, 4)
    }
    #[doc = "Bit 5 - CMP1"]
    #[inline(always)]
    pub fn cmp1(&mut self) -> Cmp1W<MrccGlbCc1Spec> {
        Cmp1W::new(self, 5)
    }
    #[doc = "Bit 6 - CMP2"]
    #[inline(always)]
    pub fn cmp2(&mut self) -> Cmp2W<MrccGlbCc1Spec> {
        Cmp2W::new(self, 6)
    }
    #[doc = "Bit 7 - DAC0"]
    #[inline(always)]
    pub fn dac0(&mut self) -> Dac0W<MrccGlbCc1Spec> {
        Dac0W::new(self, 7)
    }
    #[doc = "Bit 8 - OPAMP0"]
    #[inline(always)]
    pub fn opamp0(&mut self) -> Opamp0W<MrccGlbCc1Spec> {
        Opamp0W::new(self, 8)
    }
    #[doc = "Bit 9 - OPAMP1"]
    #[inline(always)]
    pub fn opamp1(&mut self) -> Opamp1W<MrccGlbCc1Spec> {
        Opamp1W::new(self, 9)
    }
    #[doc = "Bit 10 - OPAMP2"]
    #[inline(always)]
    pub fn opamp2(&mut self) -> Opamp2W<MrccGlbCc1Spec> {
        Opamp2W::new(self, 10)
    }
    #[doc = "Bit 11 - OPAMP3"]
    #[inline(always)]
    pub fn opamp3(&mut self) -> Opamp3W<MrccGlbCc1Spec> {
        Opamp3W::new(self, 11)
    }
    #[doc = "Bit 12 - PORT0"]
    #[inline(always)]
    pub fn port0(&mut self) -> Port0W<MrccGlbCc1Spec> {
        Port0W::new(self, 12)
    }
    #[doc = "Bit 13 - PORT1"]
    #[inline(always)]
    pub fn port1(&mut self) -> Port1W<MrccGlbCc1Spec> {
        Port1W::new(self, 13)
    }
    #[doc = "Bit 14 - PORT2"]
    #[inline(always)]
    pub fn port2(&mut self) -> Port2W<MrccGlbCc1Spec> {
        Port2W::new(self, 14)
    }
    #[doc = "Bit 15 - PORT3"]
    #[inline(always)]
    pub fn port3(&mut self) -> Port3W<MrccGlbCc1Spec> {
        Port3W::new(self, 15)
    }
    #[doc = "Bit 16 - PORT4"]
    #[inline(always)]
    pub fn port4(&mut self) -> Port4W<MrccGlbCc1Spec> {
        Port4W::new(self, 16)
    }
    #[doc = "Bit 17 - SLCD0"]
    #[inline(always)]
    pub fn slcd0(&mut self) -> Slcd0W<MrccGlbCc1Spec> {
        Slcd0W::new(self, 17)
    }
    #[doc = "Bit 18 - FLEXCAN0"]
    #[inline(always)]
    pub fn flexcan0(&mut self) -> Flexcan0W<MrccGlbCc1Spec> {
        Flexcan0W::new(self, 18)
    }
    #[doc = "Bit 19 - FLEXCAN1"]
    #[inline(always)]
    pub fn flexcan1(&mut self) -> Flexcan1W<MrccGlbCc1Spec> {
        Flexcan1W::new(self, 19)
    }
    #[doc = "Bit 20 - LPI2C2"]
    #[inline(always)]
    pub fn lpi2c2(&mut self) -> Lpi2c2W<MrccGlbCc1Spec> {
        Lpi2c2W::new(self, 20)
    }
    #[doc = "Bit 21 - LPI2C3"]
    #[inline(always)]
    pub fn lpi2c3(&mut self) -> Lpi2c3W<MrccGlbCc1Spec> {
        Lpi2c3W::new(self, 21)
    }
    #[doc = "Bit 22 - LPUART5"]
    #[inline(always)]
    pub fn lpuart5(&mut self) -> Lpuart5W<MrccGlbCc1Spec> {
        Lpuart5W::new(self, 22)
    }
    #[doc = "Bit 23 - TDET0"]
    #[inline(always)]
    pub fn tdet0(&mut self) -> Tdet0W<MrccGlbCc1Spec> {
        Tdet0W::new(self, 23)
    }
    #[doc = "Bit 24 - PKC0"]
    #[inline(always)]
    pub fn pkc0(&mut self) -> Pkc0W<MrccGlbCc1Spec> {
        Pkc0W::new(self, 24)
    }
    #[doc = "Bit 25 - SGI0"]
    #[inline(always)]
    pub fn sgi0(&mut self) -> Sgi0W<MrccGlbCc1Spec> {
        Sgi0W::new(self, 25)
    }
    #[doc = "Bit 26 - TRNG0"]
    #[inline(always)]
    pub fn trng0(&mut self) -> Trng0W<MrccGlbCc1Spec> {
        Trng0W::new(self, 26)
    }
    #[doc = "Bit 27 - UDF0"]
    #[inline(always)]
    pub fn udf0(&mut self) -> Udf0W<MrccGlbCc1Spec> {
        Udf0W::new(self, 27)
    }
    #[doc = "Bit 28 - ADC2"]
    #[inline(always)]
    pub fn adc2(&mut self) -> Adc2W<MrccGlbCc1Spec> {
        Adc2W::new(self, 28)
    }
    #[doc = "Bit 29 - ADC3"]
    #[inline(always)]
    pub fn adc3(&mut self) -> Adc3W<MrccGlbCc1Spec> {
        Adc3W::new(self, 29)
    }
}
#[doc = "AHB Clock Control 1\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_glb_cc1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_cc1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MrccGlbCc1Spec;
impl crate::RegisterSpec for MrccGlbCc1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mrcc_glb_cc1::R`](R) reader structure"]
impl crate::Readable for MrccGlbCc1Spec {}
#[doc = "`write(|w| ..)` method takes [`mrcc_glb_cc1::W`](W) writer structure"]
impl crate::Writable for MrccGlbCc1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MRCC_GLB_CC1 to value 0"]
impl crate::Resettable for MrccGlbCc1Spec {}

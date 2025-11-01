#[doc = "Register `MCFGR1` reader"]
pub type R = crate::R<Mcfgr1Spec>;
#[doc = "Register `MCFGR1` writer"]
pub type W = crate::W<Mcfgr1Spec>;
#[doc = "Prescaler\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Prescale {
    #[doc = "0: Divide by 1"]
    DivideBy1 = 0,
    #[doc = "1: Divide by 2"]
    DivideBy2 = 1,
    #[doc = "2: Divide by 4"]
    DivideBy4 = 2,
    #[doc = "3: Divide by 8"]
    DivideBy8 = 3,
    #[doc = "4: Divide by 16"]
    DivideBy16 = 4,
    #[doc = "5: Divide by 32"]
    DivideBy32 = 5,
    #[doc = "6: Divide by 64"]
    DivideBy64 = 6,
    #[doc = "7: Divide by 128"]
    DivideBy128 = 7,
}
impl From<Prescale> for u8 {
    #[inline(always)]
    fn from(variant: Prescale) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Prescale {
    type Ux = u8;
}
impl crate::IsEnum for Prescale {}
#[doc = "Field `PRESCALE` reader - Prescaler"]
pub type PrescaleR = crate::FieldReader<Prescale>;
impl PrescaleR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Prescale {
        match self.bits {
            0 => Prescale::DivideBy1,
            1 => Prescale::DivideBy2,
            2 => Prescale::DivideBy4,
            3 => Prescale::DivideBy8,
            4 => Prescale::DivideBy16,
            5 => Prescale::DivideBy32,
            6 => Prescale::DivideBy64,
            7 => Prescale::DivideBy128,
            _ => unreachable!(),
        }
    }
    #[doc = "Divide by 1"]
    #[inline(always)]
    pub fn is_divide_by_1(&self) -> bool {
        *self == Prescale::DivideBy1
    }
    #[doc = "Divide by 2"]
    #[inline(always)]
    pub fn is_divide_by_2(&self) -> bool {
        *self == Prescale::DivideBy2
    }
    #[doc = "Divide by 4"]
    #[inline(always)]
    pub fn is_divide_by_4(&self) -> bool {
        *self == Prescale::DivideBy4
    }
    #[doc = "Divide by 8"]
    #[inline(always)]
    pub fn is_divide_by_8(&self) -> bool {
        *self == Prescale::DivideBy8
    }
    #[doc = "Divide by 16"]
    #[inline(always)]
    pub fn is_divide_by_16(&self) -> bool {
        *self == Prescale::DivideBy16
    }
    #[doc = "Divide by 32"]
    #[inline(always)]
    pub fn is_divide_by_32(&self) -> bool {
        *self == Prescale::DivideBy32
    }
    #[doc = "Divide by 64"]
    #[inline(always)]
    pub fn is_divide_by_64(&self) -> bool {
        *self == Prescale::DivideBy64
    }
    #[doc = "Divide by 128"]
    #[inline(always)]
    pub fn is_divide_by_128(&self) -> bool {
        *self == Prescale::DivideBy128
    }
}
#[doc = "Field `PRESCALE` writer - Prescaler"]
pub type PrescaleW<'a, REG> = crate::FieldWriter<'a, REG, 3, Prescale, crate::Safe>;
impl<'a, REG> PrescaleW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Divide by 1"]
    #[inline(always)]
    pub fn divide_by_1(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::DivideBy1)
    }
    #[doc = "Divide by 2"]
    #[inline(always)]
    pub fn divide_by_2(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::DivideBy2)
    }
    #[doc = "Divide by 4"]
    #[inline(always)]
    pub fn divide_by_4(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::DivideBy4)
    }
    #[doc = "Divide by 8"]
    #[inline(always)]
    pub fn divide_by_8(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::DivideBy8)
    }
    #[doc = "Divide by 16"]
    #[inline(always)]
    pub fn divide_by_16(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::DivideBy16)
    }
    #[doc = "Divide by 32"]
    #[inline(always)]
    pub fn divide_by_32(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::DivideBy32)
    }
    #[doc = "Divide by 64"]
    #[inline(always)]
    pub fn divide_by_64(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::DivideBy64)
    }
    #[doc = "Divide by 128"]
    #[inline(always)]
    pub fn divide_by_128(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::DivideBy128)
    }
}
#[doc = "Automatic Stop Generation\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Autostop {
    #[doc = "0: No effect"]
    Disabled = 0,
    #[doc = "1: Stop automatically generated"]
    Enabled = 1,
}
impl From<Autostop> for bool {
    #[inline(always)]
    fn from(variant: Autostop) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `AUTOSTOP` reader - Automatic Stop Generation"]
pub type AutostopR = crate::BitReader<Autostop>;
impl AutostopR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Autostop {
        match self.bits {
            false => Autostop::Disabled,
            true => Autostop::Enabled,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Autostop::Disabled
    }
    #[doc = "Stop automatically generated"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Autostop::Enabled
    }
}
#[doc = "Field `AUTOSTOP` writer - Automatic Stop Generation"]
pub type AutostopW<'a, REG> = crate::BitWriter<'a, REG, Autostop>;
impl<'a, REG> AutostopW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Autostop::Disabled)
    }
    #[doc = "Stop automatically generated"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Autostop::Enabled)
    }
}
#[doc = "Ignore NACK\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ignack {
    #[doc = "0: No effect"]
    Disabled = 0,
    #[doc = "1: Treat a received NACK as an ACK"]
    Enabled = 1,
}
impl From<Ignack> for bool {
    #[inline(always)]
    fn from(variant: Ignack) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `IGNACK` reader - Ignore NACK"]
pub type IgnackR = crate::BitReader<Ignack>;
impl IgnackR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ignack {
        match self.bits {
            false => Ignack::Disabled,
            true => Ignack::Enabled,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Ignack::Disabled
    }
    #[doc = "Treat a received NACK as an ACK"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Ignack::Enabled
    }
}
#[doc = "Field `IGNACK` writer - Ignore NACK"]
pub type IgnackW<'a, REG> = crate::BitWriter<'a, REG, Ignack>;
impl<'a, REG> IgnackW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ignack::Disabled)
    }
    #[doc = "Treat a received NACK as an ACK"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ignack::Enabled)
    }
}
#[doc = "Timeout Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Timecfg {
    #[doc = "0: SCL"]
    IfSclLow = 0,
    #[doc = "1: SCL or SDA"]
    IfSclOrSdaLow = 1,
}
impl From<Timecfg> for bool {
    #[inline(always)]
    fn from(variant: Timecfg) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIMECFG` reader - Timeout Configuration"]
pub type TimecfgR = crate::BitReader<Timecfg>;
impl TimecfgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Timecfg {
        match self.bits {
            false => Timecfg::IfSclLow,
            true => Timecfg::IfSclOrSdaLow,
        }
    }
    #[doc = "SCL"]
    #[inline(always)]
    pub fn is_if_scl_low(&self) -> bool {
        *self == Timecfg::IfSclLow
    }
    #[doc = "SCL or SDA"]
    #[inline(always)]
    pub fn is_if_scl_or_sda_low(&self) -> bool {
        *self == Timecfg::IfSclOrSdaLow
    }
}
#[doc = "Field `TIMECFG` writer - Timeout Configuration"]
pub type TimecfgW<'a, REG> = crate::BitWriter<'a, REG, Timecfg>;
impl<'a, REG> TimecfgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "SCL"]
    #[inline(always)]
    pub fn if_scl_low(self) -> &'a mut crate::W<REG> {
        self.variant(Timecfg::IfSclLow)
    }
    #[doc = "SCL or SDA"]
    #[inline(always)]
    pub fn if_scl_or_sda_low(self) -> &'a mut crate::W<REG> {
        self.variant(Timecfg::IfSclOrSdaLow)
    }
}
#[doc = "Stop Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stopcfg {
    #[doc = "0: Any Stop condition"]
    AnyStop = 0,
    #[doc = "1: Last Stop condition"]
    LastStop = 1,
}
impl From<Stopcfg> for bool {
    #[inline(always)]
    fn from(variant: Stopcfg) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STOPCFG` reader - Stop Configuration"]
pub type StopcfgR = crate::BitReader<Stopcfg>;
impl StopcfgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Stopcfg {
        match self.bits {
            false => Stopcfg::AnyStop,
            true => Stopcfg::LastStop,
        }
    }
    #[doc = "Any Stop condition"]
    #[inline(always)]
    pub fn is_any_stop(&self) -> bool {
        *self == Stopcfg::AnyStop
    }
    #[doc = "Last Stop condition"]
    #[inline(always)]
    pub fn is_last_stop(&self) -> bool {
        *self == Stopcfg::LastStop
    }
}
#[doc = "Field `STOPCFG` writer - Stop Configuration"]
pub type StopcfgW<'a, REG> = crate::BitWriter<'a, REG, Stopcfg>;
impl<'a, REG> StopcfgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Any Stop condition"]
    #[inline(always)]
    pub fn any_stop(self) -> &'a mut crate::W<REG> {
        self.variant(Stopcfg::AnyStop)
    }
    #[doc = "Last Stop condition"]
    #[inline(always)]
    pub fn last_stop(self) -> &'a mut crate::W<REG> {
        self.variant(Stopcfg::LastStop)
    }
}
#[doc = "Start Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Startcfg {
    #[doc = "0: Sets when both I2C bus and LPI2C controller are idle"]
    BothI2cAndLpi2cIdle = 0,
    #[doc = "1: Sets when I2C bus is idle"]
    I2cIdle = 1,
}
impl From<Startcfg> for bool {
    #[inline(always)]
    fn from(variant: Startcfg) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STARTCFG` reader - Start Configuration"]
pub type StartcfgR = crate::BitReader<Startcfg>;
impl StartcfgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Startcfg {
        match self.bits {
            false => Startcfg::BothI2cAndLpi2cIdle,
            true => Startcfg::I2cIdle,
        }
    }
    #[doc = "Sets when both I2C bus and LPI2C controller are idle"]
    #[inline(always)]
    pub fn is_both_i2c_and_lpi2c_idle(&self) -> bool {
        *self == Startcfg::BothI2cAndLpi2cIdle
    }
    #[doc = "Sets when I2C bus is idle"]
    #[inline(always)]
    pub fn is_i2c_idle(&self) -> bool {
        *self == Startcfg::I2cIdle
    }
}
#[doc = "Field `STARTCFG` writer - Start Configuration"]
pub type StartcfgW<'a, REG> = crate::BitWriter<'a, REG, Startcfg>;
impl<'a, REG> StartcfgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Sets when both I2C bus and LPI2C controller are idle"]
    #[inline(always)]
    pub fn both_i2c_and_lpi2c_idle(self) -> &'a mut crate::W<REG> {
        self.variant(Startcfg::BothI2cAndLpi2cIdle)
    }
    #[doc = "Sets when I2C bus is idle"]
    #[inline(always)]
    pub fn i2c_idle(self) -> &'a mut crate::W<REG> {
        self.variant(Startcfg::I2cIdle)
    }
}
#[doc = "Match Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Matcfg {
    #[doc = "0: Match is disabled"]
    Disabled = 0,
    #[doc = "2: Match is enabled: first data word equals MDMR\\[MATCH0\\] OR MDMR\\[MATCH1\\]"]
    FirstDataWordEqualsMatch0OrMatch1 = 2,
    #[doc = "3: Match is enabled: any data word equals MDMR\\[MATCH0\\] OR MDMR\\[MATCH1\\]"]
    AnyDataWordEqualsMatch0OrMatch1 = 3,
    #[doc = "4: Match is enabled: (first data word equals MDMR\\[MATCH0\\]) AND (second data word equals MDMR\\[MATCH1)"]
    FirstDataWordMatch0AndSecondDataWordMatch1 = 4,
    #[doc = "5: Match is enabled: (any data word equals MDMR\\[MATCH0\\]) AND (next data word equals MDMR\\[MATCH1)"]
    AnyDataWordMatch0NextDataWordMatch1 = 5,
    #[doc = "6: Match is enabled: (first data word AND MDMR\\[MATCH1\\]) equals (MDMR\\[MATCH0\\] AND MDMR\\[MATCH1\\])"]
    FirstDataWordAndMatch1EqualsMatch0AndMatch1 = 6,
    #[doc = "7: Match is enabled: (any data word AND MDMR\\[MATCH1\\]) equals (MDMR\\[MATCH0\\] AND MDMR\\[MATCH1\\])"]
    AnyDataWordAndMatch1EqualsMatch0AndMatch1 = 7,
}
impl From<Matcfg> for u8 {
    #[inline(always)]
    fn from(variant: Matcfg) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Matcfg {
    type Ux = u8;
}
impl crate::IsEnum for Matcfg {}
#[doc = "Field `MATCFG` reader - Match Configuration"]
pub type MatcfgR = crate::FieldReader<Matcfg>;
impl MatcfgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Matcfg> {
        match self.bits {
            0 => Some(Matcfg::Disabled),
            2 => Some(Matcfg::FirstDataWordEqualsMatch0OrMatch1),
            3 => Some(Matcfg::AnyDataWordEqualsMatch0OrMatch1),
            4 => Some(Matcfg::FirstDataWordMatch0AndSecondDataWordMatch1),
            5 => Some(Matcfg::AnyDataWordMatch0NextDataWordMatch1),
            6 => Some(Matcfg::FirstDataWordAndMatch1EqualsMatch0AndMatch1),
            7 => Some(Matcfg::AnyDataWordAndMatch1EqualsMatch0AndMatch1),
            _ => None,
        }
    }
    #[doc = "Match is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Matcfg::Disabled
    }
    #[doc = "Match is enabled: first data word equals MDMR\\[MATCH0\\] OR MDMR\\[MATCH1\\]"]
    #[inline(always)]
    pub fn is_first_data_word_equals_match0_or_match1(&self) -> bool {
        *self == Matcfg::FirstDataWordEqualsMatch0OrMatch1
    }
    #[doc = "Match is enabled: any data word equals MDMR\\[MATCH0\\] OR MDMR\\[MATCH1\\]"]
    #[inline(always)]
    pub fn is_any_data_word_equals_match0_or_match1(&self) -> bool {
        *self == Matcfg::AnyDataWordEqualsMatch0OrMatch1
    }
    #[doc = "Match is enabled: (first data word equals MDMR\\[MATCH0\\]) AND (second data word equals MDMR\\[MATCH1)"]
    #[inline(always)]
    pub fn is_first_data_word_match0_and_second_data_word_match1(&self) -> bool {
        *self == Matcfg::FirstDataWordMatch0AndSecondDataWordMatch1
    }
    #[doc = "Match is enabled: (any data word equals MDMR\\[MATCH0\\]) AND (next data word equals MDMR\\[MATCH1)"]
    #[inline(always)]
    pub fn is_any_data_word_match0_next_data_word_match1(&self) -> bool {
        *self == Matcfg::AnyDataWordMatch0NextDataWordMatch1
    }
    #[doc = "Match is enabled: (first data word AND MDMR\\[MATCH1\\]) equals (MDMR\\[MATCH0\\] AND MDMR\\[MATCH1\\])"]
    #[inline(always)]
    pub fn is_first_data_word_and_match1_equals_match0_and_match1(&self) -> bool {
        *self == Matcfg::FirstDataWordAndMatch1EqualsMatch0AndMatch1
    }
    #[doc = "Match is enabled: (any data word AND MDMR\\[MATCH1\\]) equals (MDMR\\[MATCH0\\] AND MDMR\\[MATCH1\\])"]
    #[inline(always)]
    pub fn is_any_data_word_and_match1_equals_match0_and_match1(&self) -> bool {
        *self == Matcfg::AnyDataWordAndMatch1EqualsMatch0AndMatch1
    }
}
#[doc = "Field `MATCFG` writer - Match Configuration"]
pub type MatcfgW<'a, REG> = crate::FieldWriter<'a, REG, 3, Matcfg>;
impl<'a, REG> MatcfgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Match is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Matcfg::Disabled)
    }
    #[doc = "Match is enabled: first data word equals MDMR\\[MATCH0\\] OR MDMR\\[MATCH1\\]"]
    #[inline(always)]
    pub fn first_data_word_equals_match0_or_match1(self) -> &'a mut crate::W<REG> {
        self.variant(Matcfg::FirstDataWordEqualsMatch0OrMatch1)
    }
    #[doc = "Match is enabled: any data word equals MDMR\\[MATCH0\\] OR MDMR\\[MATCH1\\]"]
    #[inline(always)]
    pub fn any_data_word_equals_match0_or_match1(self) -> &'a mut crate::W<REG> {
        self.variant(Matcfg::AnyDataWordEqualsMatch0OrMatch1)
    }
    #[doc = "Match is enabled: (first data word equals MDMR\\[MATCH0\\]) AND (second data word equals MDMR\\[MATCH1)"]
    #[inline(always)]
    pub fn first_data_word_match0_and_second_data_word_match1(self) -> &'a mut crate::W<REG> {
        self.variant(Matcfg::FirstDataWordMatch0AndSecondDataWordMatch1)
    }
    #[doc = "Match is enabled: (any data word equals MDMR\\[MATCH0\\]) AND (next data word equals MDMR\\[MATCH1)"]
    #[inline(always)]
    pub fn any_data_word_match0_next_data_word_match1(self) -> &'a mut crate::W<REG> {
        self.variant(Matcfg::AnyDataWordMatch0NextDataWordMatch1)
    }
    #[doc = "Match is enabled: (first data word AND MDMR\\[MATCH1\\]) equals (MDMR\\[MATCH0\\] AND MDMR\\[MATCH1\\])"]
    #[inline(always)]
    pub fn first_data_word_and_match1_equals_match0_and_match1(self) -> &'a mut crate::W<REG> {
        self.variant(Matcfg::FirstDataWordAndMatch1EqualsMatch0AndMatch1)
    }
    #[doc = "Match is enabled: (any data word AND MDMR\\[MATCH1\\]) equals (MDMR\\[MATCH0\\] AND MDMR\\[MATCH1\\])"]
    #[inline(always)]
    pub fn any_data_word_and_match1_equals_match0_and_match1(self) -> &'a mut crate::W<REG> {
        self.variant(Matcfg::AnyDataWordAndMatch1EqualsMatch0AndMatch1)
    }
}
#[doc = "Pin Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pincfg {
    #[doc = "0: Two-pin open drain mode"]
    OpenDrain2Pin = 0,
    #[doc = "1: Two-pin output only mode (Ultra-Fast mode)"]
    Output2PinOnly = 1,
    #[doc = "2: Two-pin push-pull mode"]
    PushPull2Pin = 2,
    #[doc = "3: Four-pin push-pull mode"]
    PushPull4Pin = 3,
    #[doc = "4: Two-pin open-drain mode with separate LPI2C target"]
    OpenDrain2PinWLpi2cSlave = 4,
    #[doc = "5: Two-pin output only mode (Ultra-Fast mode) with separate LPI2C target"]
    Output2PinOnlyWLpi2cSlave = 5,
    #[doc = "6: Two-pin push-pull mode with separate LPI2C target"]
    PushPull2PinWLpi2cSlave = 6,
    #[doc = "7: Four-pin push-pull mode (inverted outputs)"]
    PushPull4PinWLpi2cSlave = 7,
}
impl From<Pincfg> for u8 {
    #[inline(always)]
    fn from(variant: Pincfg) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pincfg {
    type Ux = u8;
}
impl crate::IsEnum for Pincfg {}
#[doc = "Field `PINCFG` reader - Pin Configuration"]
pub type PincfgR = crate::FieldReader<Pincfg>;
impl PincfgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pincfg {
        match self.bits {
            0 => Pincfg::OpenDrain2Pin,
            1 => Pincfg::Output2PinOnly,
            2 => Pincfg::PushPull2Pin,
            3 => Pincfg::PushPull4Pin,
            4 => Pincfg::OpenDrain2PinWLpi2cSlave,
            5 => Pincfg::Output2PinOnlyWLpi2cSlave,
            6 => Pincfg::PushPull2PinWLpi2cSlave,
            7 => Pincfg::PushPull4PinWLpi2cSlave,
            _ => unreachable!(),
        }
    }
    #[doc = "Two-pin open drain mode"]
    #[inline(always)]
    pub fn is_open_drain_2_pin(&self) -> bool {
        *self == Pincfg::OpenDrain2Pin
    }
    #[doc = "Two-pin output only mode (Ultra-Fast mode)"]
    #[inline(always)]
    pub fn is_output_2_pin_only(&self) -> bool {
        *self == Pincfg::Output2PinOnly
    }
    #[doc = "Two-pin push-pull mode"]
    #[inline(always)]
    pub fn is_push_pull_2_pin(&self) -> bool {
        *self == Pincfg::PushPull2Pin
    }
    #[doc = "Four-pin push-pull mode"]
    #[inline(always)]
    pub fn is_push_pull_4_pin(&self) -> bool {
        *self == Pincfg::PushPull4Pin
    }
    #[doc = "Two-pin open-drain mode with separate LPI2C target"]
    #[inline(always)]
    pub fn is_open_drain_2_pin_w_lpi2c_slave(&self) -> bool {
        *self == Pincfg::OpenDrain2PinWLpi2cSlave
    }
    #[doc = "Two-pin output only mode (Ultra-Fast mode) with separate LPI2C target"]
    #[inline(always)]
    pub fn is_output_2_pin_only_w_lpi2c_slave(&self) -> bool {
        *self == Pincfg::Output2PinOnlyWLpi2cSlave
    }
    #[doc = "Two-pin push-pull mode with separate LPI2C target"]
    #[inline(always)]
    pub fn is_push_pull_2_pin_w_lpi2c_slave(&self) -> bool {
        *self == Pincfg::PushPull2PinWLpi2cSlave
    }
    #[doc = "Four-pin push-pull mode (inverted outputs)"]
    #[inline(always)]
    pub fn is_push_pull_4_pin_w_lpi2c_slave(&self) -> bool {
        *self == Pincfg::PushPull4PinWLpi2cSlave
    }
}
#[doc = "Field `PINCFG` writer - Pin Configuration"]
pub type PincfgW<'a, REG> = crate::FieldWriter<'a, REG, 3, Pincfg, crate::Safe>;
impl<'a, REG> PincfgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Two-pin open drain mode"]
    #[inline(always)]
    pub fn open_drain_2_pin(self) -> &'a mut crate::W<REG> {
        self.variant(Pincfg::OpenDrain2Pin)
    }
    #[doc = "Two-pin output only mode (Ultra-Fast mode)"]
    #[inline(always)]
    pub fn output_2_pin_only(self) -> &'a mut crate::W<REG> {
        self.variant(Pincfg::Output2PinOnly)
    }
    #[doc = "Two-pin push-pull mode"]
    #[inline(always)]
    pub fn push_pull_2_pin(self) -> &'a mut crate::W<REG> {
        self.variant(Pincfg::PushPull2Pin)
    }
    #[doc = "Four-pin push-pull mode"]
    #[inline(always)]
    pub fn push_pull_4_pin(self) -> &'a mut crate::W<REG> {
        self.variant(Pincfg::PushPull4Pin)
    }
    #[doc = "Two-pin open-drain mode with separate LPI2C target"]
    #[inline(always)]
    pub fn open_drain_2_pin_w_lpi2c_slave(self) -> &'a mut crate::W<REG> {
        self.variant(Pincfg::OpenDrain2PinWLpi2cSlave)
    }
    #[doc = "Two-pin output only mode (Ultra-Fast mode) with separate LPI2C target"]
    #[inline(always)]
    pub fn output_2_pin_only_w_lpi2c_slave(self) -> &'a mut crate::W<REG> {
        self.variant(Pincfg::Output2PinOnlyWLpi2cSlave)
    }
    #[doc = "Two-pin push-pull mode with separate LPI2C target"]
    #[inline(always)]
    pub fn push_pull_2_pin_w_lpi2c_slave(self) -> &'a mut crate::W<REG> {
        self.variant(Pincfg::PushPull2PinWLpi2cSlave)
    }
    #[doc = "Four-pin push-pull mode (inverted outputs)"]
    #[inline(always)]
    pub fn push_pull_4_pin_w_lpi2c_slave(self) -> &'a mut crate::W<REG> {
        self.variant(Pincfg::PushPull4PinWLpi2cSlave)
    }
}
impl R {
    #[doc = "Bits 0:2 - Prescaler"]
    #[inline(always)]
    pub fn prescale(&self) -> PrescaleR {
        PrescaleR::new((self.bits & 7) as u8)
    }
    #[doc = "Bit 8 - Automatic Stop Generation"]
    #[inline(always)]
    pub fn autostop(&self) -> AutostopR {
        AutostopR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Ignore NACK"]
    #[inline(always)]
    pub fn ignack(&self) -> IgnackR {
        IgnackR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Timeout Configuration"]
    #[inline(always)]
    pub fn timecfg(&self) -> TimecfgR {
        TimecfgR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Stop Configuration"]
    #[inline(always)]
    pub fn stopcfg(&self) -> StopcfgR {
        StopcfgR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Start Configuration"]
    #[inline(always)]
    pub fn startcfg(&self) -> StartcfgR {
        StartcfgR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bits 16:18 - Match Configuration"]
    #[inline(always)]
    pub fn matcfg(&self) -> MatcfgR {
        MatcfgR::new(((self.bits >> 16) & 7) as u8)
    }
    #[doc = "Bits 24:26 - Pin Configuration"]
    #[inline(always)]
    pub fn pincfg(&self) -> PincfgR {
        PincfgR::new(((self.bits >> 24) & 7) as u8)
    }
}
impl W {
    #[doc = "Bits 0:2 - Prescaler"]
    #[inline(always)]
    pub fn prescale(&mut self) -> PrescaleW<Mcfgr1Spec> {
        PrescaleW::new(self, 0)
    }
    #[doc = "Bit 8 - Automatic Stop Generation"]
    #[inline(always)]
    pub fn autostop(&mut self) -> AutostopW<Mcfgr1Spec> {
        AutostopW::new(self, 8)
    }
    #[doc = "Bit 9 - Ignore NACK"]
    #[inline(always)]
    pub fn ignack(&mut self) -> IgnackW<Mcfgr1Spec> {
        IgnackW::new(self, 9)
    }
    #[doc = "Bit 10 - Timeout Configuration"]
    #[inline(always)]
    pub fn timecfg(&mut self) -> TimecfgW<Mcfgr1Spec> {
        TimecfgW::new(self, 10)
    }
    #[doc = "Bit 11 - Stop Configuration"]
    #[inline(always)]
    pub fn stopcfg(&mut self) -> StopcfgW<Mcfgr1Spec> {
        StopcfgW::new(self, 11)
    }
    #[doc = "Bit 12 - Start Configuration"]
    #[inline(always)]
    pub fn startcfg(&mut self) -> StartcfgW<Mcfgr1Spec> {
        StartcfgW::new(self, 12)
    }
    #[doc = "Bits 16:18 - Match Configuration"]
    #[inline(always)]
    pub fn matcfg(&mut self) -> MatcfgW<Mcfgr1Spec> {
        MatcfgW::new(self, 16)
    }
    #[doc = "Bits 24:26 - Pin Configuration"]
    #[inline(always)]
    pub fn pincfg(&mut self) -> PincfgW<Mcfgr1Spec> {
        PincfgW::new(self, 24)
    }
}
#[doc = "Controller Configuration 1\n\nYou can [`read`](crate::Reg::read) this register and get [`mcfgr1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mcfgr1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Mcfgr1Spec;
impl crate::RegisterSpec for Mcfgr1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mcfgr1::R`](R) reader structure"]
impl crate::Readable for Mcfgr1Spec {}
#[doc = "`write(|w| ..)` method takes [`mcfgr1::W`](W) writer structure"]
impl crate::Writable for Mcfgr1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MCFGR1 to value 0"]
impl crate::Resettable for Mcfgr1Spec {}

#[doc = "Register `FIRCCSR` reader"]
pub type R = crate::R<FirccsrSpec>;
#[doc = "Register `FIRCCSR` writer"]
pub type W = crate::W<FirccsrSpec>;
#[doc = "FIRC Enable\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fircen {
    #[doc = "0: FIRC is disabled"]
    Disabled = 0,
    #[doc = "1: FIRC is enabled"]
    Enabled = 1,
}
impl From<Fircen> for bool {
    #[inline(always)]
    fn from(variant: Fircen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FIRCEN` reader - FIRC Enable"]
pub type FircenR = crate::BitReader<Fircen>;
impl FircenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fircen {
        match self.bits {
            false => Fircen::Disabled,
            true => Fircen::Enabled,
        }
    }
    #[doc = "FIRC is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Fircen::Disabled
    }
    #[doc = "FIRC is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Fircen::Enabled
    }
}
#[doc = "Field `FIRCEN` writer - FIRC Enable"]
pub type FircenW<'a, REG> = crate::BitWriter<'a, REG, Fircen>;
impl<'a, REG> FircenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "FIRC is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Fircen::Disabled)
    }
    #[doc = "FIRC is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Fircen::Enabled)
    }
}
#[doc = "FIRC Stop Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fircsten {
    #[doc = "0: FIRC is disabled in Deep Sleep mode"]
    DisabledInStopModes = 0,
    #[doc = "1: FIRC is enabled in Deep Sleep mode"]
    EnabledInStopModes = 1,
}
impl From<Fircsten> for bool {
    #[inline(always)]
    fn from(variant: Fircsten) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FIRCSTEN` reader - FIRC Stop Enable"]
pub type FircstenR = crate::BitReader<Fircsten>;
impl FircstenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fircsten {
        match self.bits {
            false => Fircsten::DisabledInStopModes,
            true => Fircsten::EnabledInStopModes,
        }
    }
    #[doc = "FIRC is disabled in Deep Sleep mode"]
    #[inline(always)]
    pub fn is_disabled_in_stop_modes(&self) -> bool {
        *self == Fircsten::DisabledInStopModes
    }
    #[doc = "FIRC is enabled in Deep Sleep mode"]
    #[inline(always)]
    pub fn is_enabled_in_stop_modes(&self) -> bool {
        *self == Fircsten::EnabledInStopModes
    }
}
#[doc = "Field `FIRCSTEN` writer - FIRC Stop Enable"]
pub type FircstenW<'a, REG> = crate::BitWriter<'a, REG, Fircsten>;
impl<'a, REG> FircstenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "FIRC is disabled in Deep Sleep mode"]
    #[inline(always)]
    pub fn disabled_in_stop_modes(self) -> &'a mut crate::W<REG> {
        self.variant(Fircsten::DisabledInStopModes)
    }
    #[doc = "FIRC is enabled in Deep Sleep mode"]
    #[inline(always)]
    pub fn enabled_in_stop_modes(self) -> &'a mut crate::W<REG> {
        self.variant(Fircsten::EnabledInStopModes)
    }
}
#[doc = "FIRC 45 MHz Clock to peripherals Enable\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FircSclkPeriphEn {
    #[doc = "0: FIRC 45 MHz to peripherals is disabled"]
    Disabled = 0,
    #[doc = "1: FIRC 45 MHz to peripherals is enabled"]
    Enabled = 1,
}
impl From<FircSclkPeriphEn> for bool {
    #[inline(always)]
    fn from(variant: FircSclkPeriphEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FIRC_SCLK_PERIPH_EN` reader - FIRC 45 MHz Clock to peripherals Enable"]
pub type FircSclkPeriphEnR = crate::BitReader<FircSclkPeriphEn>;
impl FircSclkPeriphEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> FircSclkPeriphEn {
        match self.bits {
            false => FircSclkPeriphEn::Disabled,
            true => FircSclkPeriphEn::Enabled,
        }
    }
    #[doc = "FIRC 45 MHz to peripherals is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == FircSclkPeriphEn::Disabled
    }
    #[doc = "FIRC 45 MHz to peripherals is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == FircSclkPeriphEn::Enabled
    }
}
#[doc = "Field `FIRC_SCLK_PERIPH_EN` writer - FIRC 45 MHz Clock to peripherals Enable"]
pub type FircSclkPeriphEnW<'a, REG> = crate::BitWriter<'a, REG, FircSclkPeriphEn>;
impl<'a, REG> FircSclkPeriphEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "FIRC 45 MHz to peripherals is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(FircSclkPeriphEn::Disabled)
    }
    #[doc = "FIRC 45 MHz to peripherals is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(FircSclkPeriphEn::Enabled)
    }
}
#[doc = "FRO_HF Clock to peripherals Enable\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FircFclkPeriphEn {
    #[doc = "0: FRO_HF to peripherals is disabled"]
    Disabled = 0,
    #[doc = "1: FRO_HF to peripherals is enabled"]
    Enabled = 1,
}
impl From<FircFclkPeriphEn> for bool {
    #[inline(always)]
    fn from(variant: FircFclkPeriphEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FIRC_FCLK_PERIPH_EN` reader - FRO_HF Clock to peripherals Enable"]
pub type FircFclkPeriphEnR = crate::BitReader<FircFclkPeriphEn>;
impl FircFclkPeriphEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> FircFclkPeriphEn {
        match self.bits {
            false => FircFclkPeriphEn::Disabled,
            true => FircFclkPeriphEn::Enabled,
        }
    }
    #[doc = "FRO_HF to peripherals is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == FircFclkPeriphEn::Disabled
    }
    #[doc = "FRO_HF to peripherals is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == FircFclkPeriphEn::Enabled
    }
}
#[doc = "Field `FIRC_FCLK_PERIPH_EN` writer - FRO_HF Clock to peripherals Enable"]
pub type FircFclkPeriphEnW<'a, REG> = crate::BitWriter<'a, REG, FircFclkPeriphEn>;
impl<'a, REG> FircFclkPeriphEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "FRO_HF to peripherals is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(FircFclkPeriphEn::Disabled)
    }
    #[doc = "FRO_HF to peripherals is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(FircFclkPeriphEn::Enabled)
    }
}
#[doc = "Lock Register\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lk {
    #[doc = "0: Control Status Register can be written"]
    WriteEnabled = 0,
    #[doc = "1: Control Status Register cannot be written"]
    WriteDisabled = 1,
}
impl From<Lk> for bool {
    #[inline(always)]
    fn from(variant: Lk) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LK` reader - Lock Register"]
pub type LkR = crate::BitReader<Lk>;
impl LkR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lk {
        match self.bits {
            false => Lk::WriteEnabled,
            true => Lk::WriteDisabled,
        }
    }
    #[doc = "Control Status Register can be written"]
    #[inline(always)]
    pub fn is_write_enabled(&self) -> bool {
        *self == Lk::WriteEnabled
    }
    #[doc = "Control Status Register cannot be written"]
    #[inline(always)]
    pub fn is_write_disabled(&self) -> bool {
        *self == Lk::WriteDisabled
    }
}
#[doc = "Field `LK` writer - Lock Register"]
pub type LkW<'a, REG> = crate::BitWriter<'a, REG, Lk>;
impl<'a, REG> LkW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Control Status Register can be written"]
    #[inline(always)]
    pub fn write_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lk::WriteEnabled)
    }
    #[doc = "Control Status Register cannot be written"]
    #[inline(always)]
    pub fn write_disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lk::WriteDisabled)
    }
}
#[doc = "FIRC Valid status\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fircvld {
    #[doc = "0: FIRC is not enabled or clock is not valid."]
    NotEnabledOrNotValid = 0,
    #[doc = "1: FIRC is enabled and output clock is valid. The clock is valid after there is an output clock from the FIRC analog."]
    EnabledAndValid = 1,
}
impl From<Fircvld> for bool {
    #[inline(always)]
    fn from(variant: Fircvld) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FIRCVLD` reader - FIRC Valid status"]
pub type FircvldR = crate::BitReader<Fircvld>;
impl FircvldR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fircvld {
        match self.bits {
            false => Fircvld::NotEnabledOrNotValid,
            true => Fircvld::EnabledAndValid,
        }
    }
    #[doc = "FIRC is not enabled or clock is not valid."]
    #[inline(always)]
    pub fn is_not_enabled_or_not_valid(&self) -> bool {
        *self == Fircvld::NotEnabledOrNotValid
    }
    #[doc = "FIRC is enabled and output clock is valid. The clock is valid after there is an output clock from the FIRC analog."]
    #[inline(always)]
    pub fn is_enabled_and_valid(&self) -> bool {
        *self == Fircvld::EnabledAndValid
    }
}
#[doc = "FIRC Selected\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fircsel {
    #[doc = "0: FIRC is not the system clock source"]
    NotFirc = 0,
    #[doc = "1: FIRC is the system clock source"]
    Firc = 1,
}
impl From<Fircsel> for bool {
    #[inline(always)]
    fn from(variant: Fircsel) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FIRCSEL` reader - FIRC Selected"]
pub type FircselR = crate::BitReader<Fircsel>;
impl FircselR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fircsel {
        match self.bits {
            false => Fircsel::NotFirc,
            true => Fircsel::Firc,
        }
    }
    #[doc = "FIRC is not the system clock source"]
    #[inline(always)]
    pub fn is_not_firc(&self) -> bool {
        *self == Fircsel::NotFirc
    }
    #[doc = "FIRC is the system clock source"]
    #[inline(always)]
    pub fn is_firc(&self) -> bool {
        *self == Fircsel::Firc
    }
}
#[doc = "FIRC Clock Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fircerr {
    #[doc = "0: Error not detected with the FIRC trimming"]
    ErrorNotDetected = 0,
    #[doc = "1: Error detected with the FIRC trimming"]
    ErrorDetected = 1,
}
impl From<Fircerr> for bool {
    #[inline(always)]
    fn from(variant: Fircerr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FIRCERR` reader - FIRC Clock Error"]
pub type FircerrR = crate::BitReader<Fircerr>;
impl FircerrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fircerr {
        match self.bits {
            false => Fircerr::ErrorNotDetected,
            true => Fircerr::ErrorDetected,
        }
    }
    #[doc = "Error not detected with the FIRC trimming"]
    #[inline(always)]
    pub fn is_error_not_detected(&self) -> bool {
        *self == Fircerr::ErrorNotDetected
    }
    #[doc = "Error detected with the FIRC trimming"]
    #[inline(always)]
    pub fn is_error_detected(&self) -> bool {
        *self == Fircerr::ErrorDetected
    }
}
#[doc = "Field `FIRCERR` writer - FIRC Clock Error"]
pub type FircerrW<'a, REG> = crate::BitWriter1C<'a, REG, Fircerr>;
impl<'a, REG> FircerrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Error not detected with the FIRC trimming"]
    #[inline(always)]
    pub fn error_not_detected(self) -> &'a mut crate::W<REG> {
        self.variant(Fircerr::ErrorNotDetected)
    }
    #[doc = "Error detected with the FIRC trimming"]
    #[inline(always)]
    pub fn error_detected(self) -> &'a mut crate::W<REG> {
        self.variant(Fircerr::ErrorDetected)
    }
}
#[doc = "FIRC Clock Error Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FircerrIe {
    #[doc = "0: FIRCERR interrupt is not enabled"]
    ErrorNotDetected = 0,
    #[doc = "1: FIRCERR interrupt is enabled"]
    ErrorDetected = 1,
}
impl From<FircerrIe> for bool {
    #[inline(always)]
    fn from(variant: FircerrIe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FIRCERR_IE` reader - FIRC Clock Error Interrupt Enable"]
pub type FircerrIeR = crate::BitReader<FircerrIe>;
impl FircerrIeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> FircerrIe {
        match self.bits {
            false => FircerrIe::ErrorNotDetected,
            true => FircerrIe::ErrorDetected,
        }
    }
    #[doc = "FIRCERR interrupt is not enabled"]
    #[inline(always)]
    pub fn is_error_not_detected(&self) -> bool {
        *self == FircerrIe::ErrorNotDetected
    }
    #[doc = "FIRCERR interrupt is enabled"]
    #[inline(always)]
    pub fn is_error_detected(&self) -> bool {
        *self == FircerrIe::ErrorDetected
    }
}
#[doc = "Field `FIRCERR_IE` writer - FIRC Clock Error Interrupt Enable"]
pub type FircerrIeW<'a, REG> = crate::BitWriter<'a, REG, FircerrIe>;
impl<'a, REG> FircerrIeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "FIRCERR interrupt is not enabled"]
    #[inline(always)]
    pub fn error_not_detected(self) -> &'a mut crate::W<REG> {
        self.variant(FircerrIe::ErrorNotDetected)
    }
    #[doc = "FIRCERR interrupt is enabled"]
    #[inline(always)]
    pub fn error_detected(self) -> &'a mut crate::W<REG> {
        self.variant(FircerrIe::ErrorDetected)
    }
}
#[doc = "FIRC Accurate Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FircaccIe {
    #[doc = "0: FIRCACC interrupt is not enabled"]
    Fircaccnot = 0,
    #[doc = "1: FIRCACC interrupt is enabled"]
    Fircaccyes = 1,
}
impl From<FircaccIe> for bool {
    #[inline(always)]
    fn from(variant: FircaccIe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FIRCACC_IE` reader - FIRC Accurate Interrupt Enable"]
pub type FircaccIeR = crate::BitReader<FircaccIe>;
impl FircaccIeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> FircaccIe {
        match self.bits {
            false => FircaccIe::Fircaccnot,
            true => FircaccIe::Fircaccyes,
        }
    }
    #[doc = "FIRCACC interrupt is not enabled"]
    #[inline(always)]
    pub fn is_fircaccnot(&self) -> bool {
        *self == FircaccIe::Fircaccnot
    }
    #[doc = "FIRCACC interrupt is enabled"]
    #[inline(always)]
    pub fn is_fircaccyes(&self) -> bool {
        *self == FircaccIe::Fircaccyes
    }
}
#[doc = "Field `FIRCACC_IE` writer - FIRC Accurate Interrupt Enable"]
pub type FircaccIeW<'a, REG> = crate::BitWriter<'a, REG, FircaccIe>;
impl<'a, REG> FircaccIeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "FIRCACC interrupt is not enabled"]
    #[inline(always)]
    pub fn fircaccnot(self) -> &'a mut crate::W<REG> {
        self.variant(FircaccIe::Fircaccnot)
    }
    #[doc = "FIRCACC interrupt is enabled"]
    #[inline(always)]
    pub fn fircaccyes(self) -> &'a mut crate::W<REG> {
        self.variant(FircaccIe::Fircaccyes)
    }
}
#[doc = "FIRC Frequency Accurate\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fircacc {
    #[doc = "0: FIRC is not enabled or clock is not accurate."]
    NotEnabledOrNotValid = 0,
    #[doc = "1: FIRC is enabled and output clock is accurate after some preparation time which is obtained by counting FRO_HF clock."]
    EnabledAndValid = 1,
}
impl From<Fircacc> for bool {
    #[inline(always)]
    fn from(variant: Fircacc) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FIRCACC` reader - FIRC Frequency Accurate"]
pub type FircaccR = crate::BitReader<Fircacc>;
impl FircaccR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fircacc {
        match self.bits {
            false => Fircacc::NotEnabledOrNotValid,
            true => Fircacc::EnabledAndValid,
        }
    }
    #[doc = "FIRC is not enabled or clock is not accurate."]
    #[inline(always)]
    pub fn is_not_enabled_or_not_valid(&self) -> bool {
        *self == Fircacc::NotEnabledOrNotValid
    }
    #[doc = "FIRC is enabled and output clock is accurate after some preparation time which is obtained by counting FRO_HF clock."]
    #[inline(always)]
    pub fn is_enabled_and_valid(&self) -> bool {
        *self == Fircacc::EnabledAndValid
    }
}
impl R {
    #[doc = "Bit 0 - FIRC Enable"]
    #[inline(always)]
    pub fn fircen(&self) -> FircenR {
        FircenR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - FIRC Stop Enable"]
    #[inline(always)]
    pub fn fircsten(&self) -> FircstenR {
        FircstenR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 4 - FIRC 45 MHz Clock to peripherals Enable"]
    #[inline(always)]
    pub fn firc_sclk_periph_en(&self) -> FircSclkPeriphEnR {
        FircSclkPeriphEnR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - FRO_HF Clock to peripherals Enable"]
    #[inline(always)]
    pub fn firc_fclk_periph_en(&self) -> FircFclkPeriphEnR {
        FircFclkPeriphEnR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 23 - Lock Register"]
    #[inline(always)]
    pub fn lk(&self) -> LkR {
        LkR::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bit 24 - FIRC Valid status"]
    #[inline(always)]
    pub fn fircvld(&self) -> FircvldR {
        FircvldR::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - FIRC Selected"]
    #[inline(always)]
    pub fn fircsel(&self) -> FircselR {
        FircselR::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - FIRC Clock Error"]
    #[inline(always)]
    pub fn fircerr(&self) -> FircerrR {
        FircerrR::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - FIRC Clock Error Interrupt Enable"]
    #[inline(always)]
    pub fn fircerr_ie(&self) -> FircerrIeR {
        FircerrIeR::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bit 30 - FIRC Accurate Interrupt Enable"]
    #[inline(always)]
    pub fn fircacc_ie(&self) -> FircaccIeR {
        FircaccIeR::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - FIRC Frequency Accurate"]
    #[inline(always)]
    pub fn fircacc(&self) -> FircaccR {
        FircaccR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - FIRC Enable"]
    #[inline(always)]
    pub fn fircen(&mut self) -> FircenW<FirccsrSpec> {
        FircenW::new(self, 0)
    }
    #[doc = "Bit 1 - FIRC Stop Enable"]
    #[inline(always)]
    pub fn fircsten(&mut self) -> FircstenW<FirccsrSpec> {
        FircstenW::new(self, 1)
    }
    #[doc = "Bit 4 - FIRC 45 MHz Clock to peripherals Enable"]
    #[inline(always)]
    pub fn firc_sclk_periph_en(&mut self) -> FircSclkPeriphEnW<FirccsrSpec> {
        FircSclkPeriphEnW::new(self, 4)
    }
    #[doc = "Bit 5 - FRO_HF Clock to peripherals Enable"]
    #[inline(always)]
    pub fn firc_fclk_periph_en(&mut self) -> FircFclkPeriphEnW<FirccsrSpec> {
        FircFclkPeriphEnW::new(self, 5)
    }
    #[doc = "Bit 23 - Lock Register"]
    #[inline(always)]
    pub fn lk(&mut self) -> LkW<FirccsrSpec> {
        LkW::new(self, 23)
    }
    #[doc = "Bit 26 - FIRC Clock Error"]
    #[inline(always)]
    pub fn fircerr(&mut self) -> FircerrW<FirccsrSpec> {
        FircerrW::new(self, 26)
    }
    #[doc = "Bit 27 - FIRC Clock Error Interrupt Enable"]
    #[inline(always)]
    pub fn fircerr_ie(&mut self) -> FircerrIeW<FirccsrSpec> {
        FircerrIeW::new(self, 27)
    }
    #[doc = "Bit 30 - FIRC Accurate Interrupt Enable"]
    #[inline(always)]
    pub fn fircacc_ie(&mut self) -> FircaccIeW<FirccsrSpec> {
        FircaccIeW::new(self, 30)
    }
}
#[doc = "FIRC Control Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`firccsr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`firccsr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FirccsrSpec;
impl crate::RegisterSpec for FirccsrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`firccsr::R`](R) reader structure"]
impl crate::Readable for FirccsrSpec {}
#[doc = "`write(|w| ..)` method takes [`firccsr::W`](W) writer structure"]
impl crate::Writable for FirccsrSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x0400_0000;
}
#[doc = "`reset()` method sets FIRCCSR to value 0x0300_0031"]
impl crate::Resettable for FirccsrSpec {
    const RESET_VALUE: u32 = 0x0300_0031;
}

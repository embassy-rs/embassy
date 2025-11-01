#[doc = "Register `SIRCCSR` reader"]
pub type R = crate::R<SirccsrSpec>;
#[doc = "Register `SIRCCSR` writer"]
pub type W = crate::W<SirccsrSpec>;
#[doc = "SIRC Stop Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sircsten {
    #[doc = "0: SIRC is disabled in Deep Sleep mode"]
    Disabled = 0,
    #[doc = "1: SIRC is enabled in Deep Sleep mode"]
    Enabled = 1,
}
impl From<Sircsten> for bool {
    #[inline(always)]
    fn from(variant: Sircsten) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SIRCSTEN` reader - SIRC Stop Enable"]
pub type SircstenR = crate::BitReader<Sircsten>;
impl SircstenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sircsten {
        match self.bits {
            false => Sircsten::Disabled,
            true => Sircsten::Enabled,
        }
    }
    #[doc = "SIRC is disabled in Deep Sleep mode"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Sircsten::Disabled
    }
    #[doc = "SIRC is enabled in Deep Sleep mode"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Sircsten::Enabled
    }
}
#[doc = "Field `SIRCSTEN` writer - SIRC Stop Enable"]
pub type SircstenW<'a, REG> = crate::BitWriter<'a, REG, Sircsten>;
impl<'a, REG> SircstenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "SIRC is disabled in Deep Sleep mode"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Sircsten::Disabled)
    }
    #[doc = "SIRC is enabled in Deep Sleep mode"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Sircsten::Enabled)
    }
}
#[doc = "SIRC Clock to Peripherals Enable\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SircClkPeriphEn {
    #[doc = "0: SIRC clock to peripherals is disabled"]
    Disabled = 0,
    #[doc = "1: SIRC clock to peripherals is enabled"]
    Enabled = 1,
}
impl From<SircClkPeriphEn> for bool {
    #[inline(always)]
    fn from(variant: SircClkPeriphEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SIRC_CLK_PERIPH_EN` reader - SIRC Clock to Peripherals Enable"]
pub type SircClkPeriphEnR = crate::BitReader<SircClkPeriphEn>;
impl SircClkPeriphEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> SircClkPeriphEn {
        match self.bits {
            false => SircClkPeriphEn::Disabled,
            true => SircClkPeriphEn::Enabled,
        }
    }
    #[doc = "SIRC clock to peripherals is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == SircClkPeriphEn::Disabled
    }
    #[doc = "SIRC clock to peripherals is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == SircClkPeriphEn::Enabled
    }
}
#[doc = "Field `SIRC_CLK_PERIPH_EN` writer - SIRC Clock to Peripherals Enable"]
pub type SircClkPeriphEnW<'a, REG> = crate::BitWriter<'a, REG, SircClkPeriphEn>;
impl<'a, REG> SircClkPeriphEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "SIRC clock to peripherals is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(SircClkPeriphEn::Disabled)
    }
    #[doc = "SIRC clock to peripherals is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(SircClkPeriphEn::Enabled)
    }
}
#[doc = "SIRC 12 MHz Trim Enable (SIRCCFG\\[RANGE\\]=1)\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sirctren {
    #[doc = "0: Disables trimming SIRC to an external clock source"]
    Disabled = 0,
    #[doc = "1: Enables trimming SIRC to an external clock source"]
    Enabled = 1,
}
impl From<Sirctren> for bool {
    #[inline(always)]
    fn from(variant: Sirctren) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SIRCTREN` reader - SIRC 12 MHz Trim Enable (SIRCCFG\\[RANGE\\]=1)"]
pub type SirctrenR = crate::BitReader<Sirctren>;
impl SirctrenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sirctren {
        match self.bits {
            false => Sirctren::Disabled,
            true => Sirctren::Enabled,
        }
    }
    #[doc = "Disables trimming SIRC to an external clock source"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Sirctren::Disabled
    }
    #[doc = "Enables trimming SIRC to an external clock source"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Sirctren::Enabled
    }
}
#[doc = "Field `SIRCTREN` writer - SIRC 12 MHz Trim Enable (SIRCCFG\\[RANGE\\]=1)"]
pub type SirctrenW<'a, REG> = crate::BitWriter<'a, REG, Sirctren>;
impl<'a, REG> SirctrenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables trimming SIRC to an external clock source"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Sirctren::Disabled)
    }
    #[doc = "Enables trimming SIRC to an external clock source"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Sirctren::Enabled)
    }
}
#[doc = "SIRC Trim Update\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sirctrup {
    #[doc = "0: Disables SIRC trimming updates"]
    Disabled = 0,
    #[doc = "1: Enables SIRC trimming updates"]
    Enabled = 1,
}
impl From<Sirctrup> for bool {
    #[inline(always)]
    fn from(variant: Sirctrup) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SIRCTRUP` reader - SIRC Trim Update"]
pub type SirctrupR = crate::BitReader<Sirctrup>;
impl SirctrupR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sirctrup {
        match self.bits {
            false => Sirctrup::Disabled,
            true => Sirctrup::Enabled,
        }
    }
    #[doc = "Disables SIRC trimming updates"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Sirctrup::Disabled
    }
    #[doc = "Enables SIRC trimming updates"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Sirctrup::Enabled
    }
}
#[doc = "Field `SIRCTRUP` writer - SIRC Trim Update"]
pub type SirctrupW<'a, REG> = crate::BitWriter<'a, REG, Sirctrup>;
impl<'a, REG> SirctrupW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables SIRC trimming updates"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Sirctrup::Disabled)
    }
    #[doc = "Enables SIRC trimming updates"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Sirctrup::Enabled)
    }
}
#[doc = "SIRC TRIM LOCK\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrimLock {
    #[doc = "0: SIRC auto trim not locked to target frequency range"]
    SircNotLocked = 0,
    #[doc = "1: SIRC auto trim locked to target frequency range"]
    SircLocked = 1,
}
impl From<TrimLock> for bool {
    #[inline(always)]
    fn from(variant: TrimLock) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TRIM_LOCK` reader - SIRC TRIM LOCK"]
pub type TrimLockR = crate::BitReader<TrimLock>;
impl TrimLockR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> TrimLock {
        match self.bits {
            false => TrimLock::SircNotLocked,
            true => TrimLock::SircLocked,
        }
    }
    #[doc = "SIRC auto trim not locked to target frequency range"]
    #[inline(always)]
    pub fn is_sirc_not_locked(&self) -> bool {
        *self == TrimLock::SircNotLocked
    }
    #[doc = "SIRC auto trim locked to target frequency range"]
    #[inline(always)]
    pub fn is_sirc_locked(&self) -> bool {
        *self == TrimLock::SircLocked
    }
}
#[doc = "Coarse Auto Trim Bypass\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoarseTrimBypass {
    #[doc = "0: SIRC Coarse Auto Trim NOT Bypassed"]
    NotBypassed = 0,
    #[doc = "1: SIRC Coarse Auto Trim Bypassed"]
    Bypassed = 1,
}
impl From<CoarseTrimBypass> for bool {
    #[inline(always)]
    fn from(variant: CoarseTrimBypass) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `COARSE_TRIM_BYPASS` reader - Coarse Auto Trim Bypass"]
pub type CoarseTrimBypassR = crate::BitReader<CoarseTrimBypass>;
impl CoarseTrimBypassR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> CoarseTrimBypass {
        match self.bits {
            false => CoarseTrimBypass::NotBypassed,
            true => CoarseTrimBypass::Bypassed,
        }
    }
    #[doc = "SIRC Coarse Auto Trim NOT Bypassed"]
    #[inline(always)]
    pub fn is_not_bypassed(&self) -> bool {
        *self == CoarseTrimBypass::NotBypassed
    }
    #[doc = "SIRC Coarse Auto Trim Bypassed"]
    #[inline(always)]
    pub fn is_bypassed(&self) -> bool {
        *self == CoarseTrimBypass::Bypassed
    }
}
#[doc = "Field `COARSE_TRIM_BYPASS` writer - Coarse Auto Trim Bypass"]
pub type CoarseTrimBypassW<'a, REG> = crate::BitWriter<'a, REG, CoarseTrimBypass>;
impl<'a, REG> CoarseTrimBypassW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "SIRC Coarse Auto Trim NOT Bypassed"]
    #[inline(always)]
    pub fn not_bypassed(self) -> &'a mut crate::W<REG> {
        self.variant(CoarseTrimBypass::NotBypassed)
    }
    #[doc = "SIRC Coarse Auto Trim Bypassed"]
    #[inline(always)]
    pub fn bypassed(self) -> &'a mut crate::W<REG> {
        self.variant(CoarseTrimBypass::Bypassed)
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
#[doc = "SIRC Valid\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sircvld {
    #[doc = "0: SIRC is not enabled or clock is not valid"]
    DisabledOrNotValid = 0,
    #[doc = "1: SIRC is enabled and output clock is valid"]
    EnabledAndValid = 1,
}
impl From<Sircvld> for bool {
    #[inline(always)]
    fn from(variant: Sircvld) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SIRCVLD` reader - SIRC Valid"]
pub type SircvldR = crate::BitReader<Sircvld>;
impl SircvldR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sircvld {
        match self.bits {
            false => Sircvld::DisabledOrNotValid,
            true => Sircvld::EnabledAndValid,
        }
    }
    #[doc = "SIRC is not enabled or clock is not valid"]
    #[inline(always)]
    pub fn is_disabled_or_not_valid(&self) -> bool {
        *self == Sircvld::DisabledOrNotValid
    }
    #[doc = "SIRC is enabled and output clock is valid"]
    #[inline(always)]
    pub fn is_enabled_and_valid(&self) -> bool {
        *self == Sircvld::EnabledAndValid
    }
}
#[doc = "SIRC Selected\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sircsel {
    #[doc = "0: SIRC is not the system clock source"]
    NotSirc = 0,
    #[doc = "1: SIRC is the system clock source"]
    Sirc = 1,
}
impl From<Sircsel> for bool {
    #[inline(always)]
    fn from(variant: Sircsel) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SIRCSEL` reader - SIRC Selected"]
pub type SircselR = crate::BitReader<Sircsel>;
impl SircselR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sircsel {
        match self.bits {
            false => Sircsel::NotSirc,
            true => Sircsel::Sirc,
        }
    }
    #[doc = "SIRC is not the system clock source"]
    #[inline(always)]
    pub fn is_not_sirc(&self) -> bool {
        *self == Sircsel::NotSirc
    }
    #[doc = "SIRC is the system clock source"]
    #[inline(always)]
    pub fn is_sirc(&self) -> bool {
        *self == Sircsel::Sirc
    }
}
#[doc = "SIRC Clock Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sircerr {
    #[doc = "0: Error not detected with the SIRC trimming"]
    ErrorNotDetected = 0,
    #[doc = "1: Error detected with the SIRC trimming"]
    ErrorDetected = 1,
}
impl From<Sircerr> for bool {
    #[inline(always)]
    fn from(variant: Sircerr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SIRCERR` reader - SIRC Clock Error"]
pub type SircerrR = crate::BitReader<Sircerr>;
impl SircerrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sircerr {
        match self.bits {
            false => Sircerr::ErrorNotDetected,
            true => Sircerr::ErrorDetected,
        }
    }
    #[doc = "Error not detected with the SIRC trimming"]
    #[inline(always)]
    pub fn is_error_not_detected(&self) -> bool {
        *self == Sircerr::ErrorNotDetected
    }
    #[doc = "Error detected with the SIRC trimming"]
    #[inline(always)]
    pub fn is_error_detected(&self) -> bool {
        *self == Sircerr::ErrorDetected
    }
}
#[doc = "Field `SIRCERR` writer - SIRC Clock Error"]
pub type SircerrW<'a, REG> = crate::BitWriter1C<'a, REG, Sircerr>;
impl<'a, REG> SircerrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Error not detected with the SIRC trimming"]
    #[inline(always)]
    pub fn error_not_detected(self) -> &'a mut crate::W<REG> {
        self.variant(Sircerr::ErrorNotDetected)
    }
    #[doc = "Error detected with the SIRC trimming"]
    #[inline(always)]
    pub fn error_detected(self) -> &'a mut crate::W<REG> {
        self.variant(Sircerr::ErrorDetected)
    }
}
#[doc = "SIRC Clock Error Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SircerrIe {
    #[doc = "0: SIRCERR interrupt is not enabled"]
    ErrorNotDetected = 0,
    #[doc = "1: SIRCERR interrupt is enabled"]
    ErrorDetected = 1,
}
impl From<SircerrIe> for bool {
    #[inline(always)]
    fn from(variant: SircerrIe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SIRCERR_IE` reader - SIRC Clock Error Interrupt Enable"]
pub type SircerrIeR = crate::BitReader<SircerrIe>;
impl SircerrIeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> SircerrIe {
        match self.bits {
            false => SircerrIe::ErrorNotDetected,
            true => SircerrIe::ErrorDetected,
        }
    }
    #[doc = "SIRCERR interrupt is not enabled"]
    #[inline(always)]
    pub fn is_error_not_detected(&self) -> bool {
        *self == SircerrIe::ErrorNotDetected
    }
    #[doc = "SIRCERR interrupt is enabled"]
    #[inline(always)]
    pub fn is_error_detected(&self) -> bool {
        *self == SircerrIe::ErrorDetected
    }
}
#[doc = "Field `SIRCERR_IE` writer - SIRC Clock Error Interrupt Enable"]
pub type SircerrIeW<'a, REG> = crate::BitWriter<'a, REG, SircerrIe>;
impl<'a, REG> SircerrIeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "SIRCERR interrupt is not enabled"]
    #[inline(always)]
    pub fn error_not_detected(self) -> &'a mut crate::W<REG> {
        self.variant(SircerrIe::ErrorNotDetected)
    }
    #[doc = "SIRCERR interrupt is enabled"]
    #[inline(always)]
    pub fn error_detected(self) -> &'a mut crate::W<REG> {
        self.variant(SircerrIe::ErrorDetected)
    }
}
impl R {
    #[doc = "Bit 1 - SIRC Stop Enable"]
    #[inline(always)]
    pub fn sircsten(&self) -> SircstenR {
        SircstenR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 5 - SIRC Clock to Peripherals Enable"]
    #[inline(always)]
    pub fn sirc_clk_periph_en(&self) -> SircClkPeriphEnR {
        SircClkPeriphEnR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 8 - SIRC 12 MHz Trim Enable (SIRCCFG\\[RANGE\\]=1)"]
    #[inline(always)]
    pub fn sirctren(&self) -> SirctrenR {
        SirctrenR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - SIRC Trim Update"]
    #[inline(always)]
    pub fn sirctrup(&self) -> SirctrupR {
        SirctrupR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - SIRC TRIM LOCK"]
    #[inline(always)]
    pub fn trim_lock(&self) -> TrimLockR {
        TrimLockR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Coarse Auto Trim Bypass"]
    #[inline(always)]
    pub fn coarse_trim_bypass(&self) -> CoarseTrimBypassR {
        CoarseTrimBypassR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 23 - Lock Register"]
    #[inline(always)]
    pub fn lk(&self) -> LkR {
        LkR::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bit 24 - SIRC Valid"]
    #[inline(always)]
    pub fn sircvld(&self) -> SircvldR {
        SircvldR::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - SIRC Selected"]
    #[inline(always)]
    pub fn sircsel(&self) -> SircselR {
        SircselR::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - SIRC Clock Error"]
    #[inline(always)]
    pub fn sircerr(&self) -> SircerrR {
        SircerrR::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - SIRC Clock Error Interrupt Enable"]
    #[inline(always)]
    pub fn sircerr_ie(&self) -> SircerrIeR {
        SircerrIeR::new(((self.bits >> 27) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 1 - SIRC Stop Enable"]
    #[inline(always)]
    pub fn sircsten(&mut self) -> SircstenW<SirccsrSpec> {
        SircstenW::new(self, 1)
    }
    #[doc = "Bit 5 - SIRC Clock to Peripherals Enable"]
    #[inline(always)]
    pub fn sirc_clk_periph_en(&mut self) -> SircClkPeriphEnW<SirccsrSpec> {
        SircClkPeriphEnW::new(self, 5)
    }
    #[doc = "Bit 8 - SIRC 12 MHz Trim Enable (SIRCCFG\\[RANGE\\]=1)"]
    #[inline(always)]
    pub fn sirctren(&mut self) -> SirctrenW<SirccsrSpec> {
        SirctrenW::new(self, 8)
    }
    #[doc = "Bit 9 - SIRC Trim Update"]
    #[inline(always)]
    pub fn sirctrup(&mut self) -> SirctrupW<SirccsrSpec> {
        SirctrupW::new(self, 9)
    }
    #[doc = "Bit 11 - Coarse Auto Trim Bypass"]
    #[inline(always)]
    pub fn coarse_trim_bypass(&mut self) -> CoarseTrimBypassW<SirccsrSpec> {
        CoarseTrimBypassW::new(self, 11)
    }
    #[doc = "Bit 23 - Lock Register"]
    #[inline(always)]
    pub fn lk(&mut self) -> LkW<SirccsrSpec> {
        LkW::new(self, 23)
    }
    #[doc = "Bit 26 - SIRC Clock Error"]
    #[inline(always)]
    pub fn sircerr(&mut self) -> SircerrW<SirccsrSpec> {
        SircerrW::new(self, 26)
    }
    #[doc = "Bit 27 - SIRC Clock Error Interrupt Enable"]
    #[inline(always)]
    pub fn sircerr_ie(&mut self) -> SircerrIeW<SirccsrSpec> {
        SircerrIeW::new(self, 27)
    }
}
#[doc = "SIRC Control Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sirccsr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sirccsr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SirccsrSpec;
impl crate::RegisterSpec for SirccsrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sirccsr::R`](R) reader structure"]
impl crate::Readable for SirccsrSpec {}
#[doc = "`write(|w| ..)` method takes [`sirccsr::W`](W) writer structure"]
impl crate::Writable for SirccsrSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x0400_0000;
}
#[doc = "`reset()` method sets SIRCCSR to value 0x0100_0020"]
impl crate::Resettable for SirccsrSpec {
    const RESET_VALUE: u32 = 0x0100_0020;
}

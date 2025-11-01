#[doc = "Register `SPLLCSR` reader"]
pub type R = crate::R<SpllcsrSpec>;
#[doc = "Register `SPLLCSR` writer"]
pub type W = crate::W<SpllcsrSpec>;
#[doc = "SPLL Power Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spllpwren {
    #[doc = "0: SPLL clock is powered off"]
    Disabled = 0,
    #[doc = "1: SPLL clock is powered on"]
    Enabled = 1,
}
impl From<Spllpwren> for bool {
    #[inline(always)]
    fn from(variant: Spllpwren) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SPLLPWREN` reader - SPLL Power Enable"]
pub type SpllpwrenR = crate::BitReader<Spllpwren>;
impl SpllpwrenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Spllpwren {
        match self.bits {
            false => Spllpwren::Disabled,
            true => Spllpwren::Enabled,
        }
    }
    #[doc = "SPLL clock is powered off"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Spllpwren::Disabled
    }
    #[doc = "SPLL clock is powered on"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Spllpwren::Enabled
    }
}
#[doc = "Field `SPLLPWREN` writer - SPLL Power Enable"]
pub type SpllpwrenW<'a, REG> = crate::BitWriter<'a, REG, Spllpwren>;
impl<'a, REG> SpllpwrenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "SPLL clock is powered off"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Spllpwren::Disabled)
    }
    #[doc = "SPLL clock is powered on"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Spllpwren::Enabled)
    }
}
#[doc = "SPLL Clock Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spllclken {
    #[doc = "0: SPLL clock is disabled"]
    Disabled = 0,
    #[doc = "1: SPLL clock is enabled"]
    Enabled = 1,
}
impl From<Spllclken> for bool {
    #[inline(always)]
    fn from(variant: Spllclken) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SPLLCLKEN` reader - SPLL Clock Enable"]
pub type SpllclkenR = crate::BitReader<Spllclken>;
impl SpllclkenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Spllclken {
        match self.bits {
            false => Spllclken::Disabled,
            true => Spllclken::Enabled,
        }
    }
    #[doc = "SPLL clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Spllclken::Disabled
    }
    #[doc = "SPLL clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Spllclken::Enabled
    }
}
#[doc = "Field `SPLLCLKEN` writer - SPLL Clock Enable"]
pub type SpllclkenW<'a, REG> = crate::BitWriter<'a, REG, Spllclken>;
impl<'a, REG> SpllclkenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "SPLL clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Spllclken::Disabled)
    }
    #[doc = "SPLL clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Spllclken::Enabled)
    }
}
#[doc = "SPLL Stop Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spllsten {
    #[doc = "0: SPLL is disabled in Deep Sleep mode"]
    DisabledInStop = 0,
    #[doc = "1: SPLL is enabled in Deep Sleep mode"]
    EnabledInStop = 1,
}
impl From<Spllsten> for bool {
    #[inline(always)]
    fn from(variant: Spllsten) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SPLLSTEN` reader - SPLL Stop Enable"]
pub type SpllstenR = crate::BitReader<Spllsten>;
impl SpllstenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Spllsten {
        match self.bits {
            false => Spllsten::DisabledInStop,
            true => Spllsten::EnabledInStop,
        }
    }
    #[doc = "SPLL is disabled in Deep Sleep mode"]
    #[inline(always)]
    pub fn is_disabled_in_stop(&self) -> bool {
        *self == Spllsten::DisabledInStop
    }
    #[doc = "SPLL is enabled in Deep Sleep mode"]
    #[inline(always)]
    pub fn is_enabled_in_stop(&self) -> bool {
        *self == Spllsten::EnabledInStop
    }
}
#[doc = "Field `SPLLSTEN` writer - SPLL Stop Enable"]
pub type SpllstenW<'a, REG> = crate::BitWriter<'a, REG, Spllsten>;
impl<'a, REG> SpllstenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "SPLL is disabled in Deep Sleep mode"]
    #[inline(always)]
    pub fn disabled_in_stop(self) -> &'a mut crate::W<REG> {
        self.variant(Spllsten::DisabledInStop)
    }
    #[doc = "SPLL is enabled in Deep Sleep mode"]
    #[inline(always)]
    pub fn enabled_in_stop(self) -> &'a mut crate::W<REG> {
        self.variant(Spllsten::EnabledInStop)
    }
}
#[doc = "Free running mode clock stable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrmClockstable {
    #[doc = "0: Free running mode clock stable is disabled"]
    Disabled = 0,
    #[doc = "1: Free running mode clock stable is enabled"]
    Enabled = 1,
}
impl From<FrmClockstable> for bool {
    #[inline(always)]
    fn from(variant: FrmClockstable) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FRM_CLOCKSTABLE` reader - Free running mode clock stable"]
pub type FrmClockstableR = crate::BitReader<FrmClockstable>;
impl FrmClockstableR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> FrmClockstable {
        match self.bits {
            false => FrmClockstable::Disabled,
            true => FrmClockstable::Enabled,
        }
    }
    #[doc = "Free running mode clock stable is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == FrmClockstable::Disabled
    }
    #[doc = "Free running mode clock stable is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == FrmClockstable::Enabled
    }
}
#[doc = "Field `FRM_CLOCKSTABLE` writer - Free running mode clock stable"]
pub type FrmClockstableW<'a, REG> = crate::BitWriter<'a, REG, FrmClockstable>;
impl<'a, REG> FrmClockstableW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Free running mode clock stable is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(FrmClockstable::Disabled)
    }
    #[doc = "Free running mode clock stable is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(FrmClockstable::Enabled)
    }
}
#[doc = "SPLL Clock Monitor\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spllcm {
    #[doc = "0: SPLL Clock Monitor is disabled"]
    Disabled = 0,
    #[doc = "1: SPLL Clock Monitor is enabled"]
    Enabled = 1,
}
impl From<Spllcm> for bool {
    #[inline(always)]
    fn from(variant: Spllcm) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SPLLCM` reader - SPLL Clock Monitor"]
pub type SpllcmR = crate::BitReader<Spllcm>;
impl SpllcmR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Spllcm {
        match self.bits {
            false => Spllcm::Disabled,
            true => Spllcm::Enabled,
        }
    }
    #[doc = "SPLL Clock Monitor is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Spllcm::Disabled
    }
    #[doc = "SPLL Clock Monitor is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Spllcm::Enabled
    }
}
#[doc = "Field `SPLLCM` writer - SPLL Clock Monitor"]
pub type SpllcmW<'a, REG> = crate::BitWriter<'a, REG, Spllcm>;
impl<'a, REG> SpllcmW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "SPLL Clock Monitor is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Spllcm::Disabled)
    }
    #[doc = "SPLL Clock Monitor is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Spllcm::Enabled)
    }
}
#[doc = "SPLL Clock Monitor Reset Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spllcmre {
    #[doc = "0: Clock monitor generates an interrupt when an error is detected"]
    GenerateInterrupt = 0,
    #[doc = "1: Clock monitor generates a reset when an error is detected"]
    GenerateReset = 1,
}
impl From<Spllcmre> for bool {
    #[inline(always)]
    fn from(variant: Spllcmre) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SPLLCMRE` reader - SPLL Clock Monitor Reset Enable"]
pub type SpllcmreR = crate::BitReader<Spllcmre>;
impl SpllcmreR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Spllcmre {
        match self.bits {
            false => Spllcmre::GenerateInterrupt,
            true => Spllcmre::GenerateReset,
        }
    }
    #[doc = "Clock monitor generates an interrupt when an error is detected"]
    #[inline(always)]
    pub fn is_generate_interrupt(&self) -> bool {
        *self == Spllcmre::GenerateInterrupt
    }
    #[doc = "Clock monitor generates a reset when an error is detected"]
    #[inline(always)]
    pub fn is_generate_reset(&self) -> bool {
        *self == Spllcmre::GenerateReset
    }
}
#[doc = "Field `SPLLCMRE` writer - SPLL Clock Monitor Reset Enable"]
pub type SpllcmreW<'a, REG> = crate::BitWriter<'a, REG, Spllcmre>;
impl<'a, REG> SpllcmreW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Clock monitor generates an interrupt when an error is detected"]
    #[inline(always)]
    pub fn generate_interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Spllcmre::GenerateInterrupt)
    }
    #[doc = "Clock monitor generates a reset when an error is detected"]
    #[inline(always)]
    pub fn generate_reset(self) -> &'a mut crate::W<REG> {
        self.variant(Spllcmre::GenerateReset)
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
#[doc = "SPLL LOCK\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpllLock {
    #[doc = "0: SPLL is not powered on or not locked"]
    DisabledOrNotValid = 0,
    #[doc = "1: SPLL is locked"]
    EnabledAndValid = 1,
}
impl From<SpllLock> for bool {
    #[inline(always)]
    fn from(variant: SpllLock) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SPLL_LOCK` reader - SPLL LOCK"]
pub type SpllLockR = crate::BitReader<SpllLock>;
impl SpllLockR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> SpllLock {
        match self.bits {
            false => SpllLock::DisabledOrNotValid,
            true => SpllLock::EnabledAndValid,
        }
    }
    #[doc = "SPLL is not powered on or not locked"]
    #[inline(always)]
    pub fn is_disabled_or_not_valid(&self) -> bool {
        *self == SpllLock::DisabledOrNotValid
    }
    #[doc = "SPLL is locked"]
    #[inline(always)]
    pub fn is_enabled_and_valid(&self) -> bool {
        *self == SpllLock::EnabledAndValid
    }
}
#[doc = "SPLL Selected\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spllsel {
    #[doc = "0: SPLL is not the system clock source"]
    NotSpll = 0,
    #[doc = "1: SPLL is the system clock source"]
    Spll = 1,
}
impl From<Spllsel> for bool {
    #[inline(always)]
    fn from(variant: Spllsel) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SPLLSEL` reader - SPLL Selected"]
pub type SpllselR = crate::BitReader<Spllsel>;
impl SpllselR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Spllsel {
        match self.bits {
            false => Spllsel::NotSpll,
            true => Spllsel::Spll,
        }
    }
    #[doc = "SPLL is not the system clock source"]
    #[inline(always)]
    pub fn is_not_spll(&self) -> bool {
        *self == Spllsel::NotSpll
    }
    #[doc = "SPLL is the system clock source"]
    #[inline(always)]
    pub fn is_spll(&self) -> bool {
        *self == Spllsel::Spll
    }
}
#[doc = "SPLL Clock Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spllerr {
    #[doc = "0: SPLL Clock Monitor is disabled or has not detected an error"]
    DisabledOrNoError = 0,
    #[doc = "1: SPLL Clock Monitor is enabled and detected an error"]
    EnabledAndError = 1,
}
impl From<Spllerr> for bool {
    #[inline(always)]
    fn from(variant: Spllerr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SPLLERR` reader - SPLL Clock Error"]
pub type SpllerrR = crate::BitReader<Spllerr>;
impl SpllerrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Spllerr {
        match self.bits {
            false => Spllerr::DisabledOrNoError,
            true => Spllerr::EnabledAndError,
        }
    }
    #[doc = "SPLL Clock Monitor is disabled or has not detected an error"]
    #[inline(always)]
    pub fn is_disabled_or_no_error(&self) -> bool {
        *self == Spllerr::DisabledOrNoError
    }
    #[doc = "SPLL Clock Monitor is enabled and detected an error"]
    #[inline(always)]
    pub fn is_enabled_and_error(&self) -> bool {
        *self == Spllerr::EnabledAndError
    }
}
#[doc = "Field `SPLLERR` writer - SPLL Clock Error"]
pub type SpllerrW<'a, REG> = crate::BitWriter1C<'a, REG, Spllerr>;
impl<'a, REG> SpllerrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "SPLL Clock Monitor is disabled or has not detected an error"]
    #[inline(always)]
    pub fn disabled_or_no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Spllerr::DisabledOrNoError)
    }
    #[doc = "SPLL Clock Monitor is enabled and detected an error"]
    #[inline(always)]
    pub fn enabled_and_error(self) -> &'a mut crate::W<REG> {
        self.variant(Spllerr::EnabledAndError)
    }
}
#[doc = "SPLL LOCK Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpllLockIe {
    #[doc = "0: SPLL_LOCK interrupt is not enabled"]
    NotSpll = 0,
    #[doc = "1: SPLL_LOCK interrupt is enabled"]
    Spll = 1,
}
impl From<SpllLockIe> for bool {
    #[inline(always)]
    fn from(variant: SpllLockIe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SPLL_LOCK_IE` reader - SPLL LOCK Interrupt Enable"]
pub type SpllLockIeR = crate::BitReader<SpllLockIe>;
impl SpllLockIeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> SpllLockIe {
        match self.bits {
            false => SpllLockIe::NotSpll,
            true => SpllLockIe::Spll,
        }
    }
    #[doc = "SPLL_LOCK interrupt is not enabled"]
    #[inline(always)]
    pub fn is_not_spll(&self) -> bool {
        *self == SpllLockIe::NotSpll
    }
    #[doc = "SPLL_LOCK interrupt is enabled"]
    #[inline(always)]
    pub fn is_spll(&self) -> bool {
        *self == SpllLockIe::Spll
    }
}
#[doc = "Field `SPLL_LOCK_IE` writer - SPLL LOCK Interrupt Enable"]
pub type SpllLockIeW<'a, REG> = crate::BitWriter<'a, REG, SpllLockIe>;
impl<'a, REG> SpllLockIeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "SPLL_LOCK interrupt is not enabled"]
    #[inline(always)]
    pub fn not_spll(self) -> &'a mut crate::W<REG> {
        self.variant(SpllLockIe::NotSpll)
    }
    #[doc = "SPLL_LOCK interrupt is enabled"]
    #[inline(always)]
    pub fn spll(self) -> &'a mut crate::W<REG> {
        self.variant(SpllLockIe::Spll)
    }
}
impl R {
    #[doc = "Bit 0 - SPLL Power Enable"]
    #[inline(always)]
    pub fn spllpwren(&self) -> SpllpwrenR {
        SpllpwrenR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - SPLL Clock Enable"]
    #[inline(always)]
    pub fn spllclken(&self) -> SpllclkenR {
        SpllclkenR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - SPLL Stop Enable"]
    #[inline(always)]
    pub fn spllsten(&self) -> SpllstenR {
        SpllstenR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Free running mode clock stable"]
    #[inline(always)]
    pub fn frm_clockstable(&self) -> FrmClockstableR {
        FrmClockstableR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 16 - SPLL Clock Monitor"]
    #[inline(always)]
    pub fn spllcm(&self) -> SpllcmR {
        SpllcmR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - SPLL Clock Monitor Reset Enable"]
    #[inline(always)]
    pub fn spllcmre(&self) -> SpllcmreR {
        SpllcmreR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 23 - Lock Register"]
    #[inline(always)]
    pub fn lk(&self) -> LkR {
        LkR::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bit 24 - SPLL LOCK"]
    #[inline(always)]
    pub fn spll_lock(&self) -> SpllLockR {
        SpllLockR::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - SPLL Selected"]
    #[inline(always)]
    pub fn spllsel(&self) -> SpllselR {
        SpllselR::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - SPLL Clock Error"]
    #[inline(always)]
    pub fn spllerr(&self) -> SpllerrR {
        SpllerrR::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 30 - SPLL LOCK Interrupt Enable"]
    #[inline(always)]
    pub fn spll_lock_ie(&self) -> SpllLockIeR {
        SpllLockIeR::new(((self.bits >> 30) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - SPLL Power Enable"]
    #[inline(always)]
    pub fn spllpwren(&mut self) -> SpllpwrenW<SpllcsrSpec> {
        SpllpwrenW::new(self, 0)
    }
    #[doc = "Bit 1 - SPLL Clock Enable"]
    #[inline(always)]
    pub fn spllclken(&mut self) -> SpllclkenW<SpllcsrSpec> {
        SpllclkenW::new(self, 1)
    }
    #[doc = "Bit 2 - SPLL Stop Enable"]
    #[inline(always)]
    pub fn spllsten(&mut self) -> SpllstenW<SpllcsrSpec> {
        SpllstenW::new(self, 2)
    }
    #[doc = "Bit 3 - Free running mode clock stable"]
    #[inline(always)]
    pub fn frm_clockstable(&mut self) -> FrmClockstableW<SpllcsrSpec> {
        FrmClockstableW::new(self, 3)
    }
    #[doc = "Bit 16 - SPLL Clock Monitor"]
    #[inline(always)]
    pub fn spllcm(&mut self) -> SpllcmW<SpllcsrSpec> {
        SpllcmW::new(self, 16)
    }
    #[doc = "Bit 17 - SPLL Clock Monitor Reset Enable"]
    #[inline(always)]
    pub fn spllcmre(&mut self) -> SpllcmreW<SpllcsrSpec> {
        SpllcmreW::new(self, 17)
    }
    #[doc = "Bit 23 - Lock Register"]
    #[inline(always)]
    pub fn lk(&mut self) -> LkW<SpllcsrSpec> {
        LkW::new(self, 23)
    }
    #[doc = "Bit 26 - SPLL Clock Error"]
    #[inline(always)]
    pub fn spllerr(&mut self) -> SpllerrW<SpllcsrSpec> {
        SpllerrW::new(self, 26)
    }
    #[doc = "Bit 30 - SPLL LOCK Interrupt Enable"]
    #[inline(always)]
    pub fn spll_lock_ie(&mut self) -> SpllLockIeW<SpllcsrSpec> {
        SpllLockIeW::new(self, 30)
    }
}
#[doc = "SPLL Control Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`spllcsr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`spllcsr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SpllcsrSpec;
impl crate::RegisterSpec for SpllcsrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`spllcsr::R`](R) reader structure"]
impl crate::Readable for SpllcsrSpec {}
#[doc = "`write(|w| ..)` method takes [`spllcsr::W`](W) writer structure"]
impl crate::Writable for SpllcsrSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x0400_0000;
}
#[doc = "`reset()` method sets SPLLCSR to value 0"]
impl crate::Resettable for SpllcsrSpec {}

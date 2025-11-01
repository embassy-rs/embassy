#[doc = "Register `SOSCCSR` reader"]
pub type R = crate::R<SosccsrSpec>;
#[doc = "Register `SOSCCSR` writer"]
pub type W = crate::W<SosccsrSpec>;
#[doc = "SOSC Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Soscen {
    #[doc = "0: SOSC is disabled"]
    Disabled = 0,
    #[doc = "1: SOSC is enabled"]
    Enabled = 1,
}
impl From<Soscen> for bool {
    #[inline(always)]
    fn from(variant: Soscen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SOSCEN` reader - SOSC Enable"]
pub type SoscenR = crate::BitReader<Soscen>;
impl SoscenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Soscen {
        match self.bits {
            false => Soscen::Disabled,
            true => Soscen::Enabled,
        }
    }
    #[doc = "SOSC is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Soscen::Disabled
    }
    #[doc = "SOSC is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Soscen::Enabled
    }
}
#[doc = "Field `SOSCEN` writer - SOSC Enable"]
pub type SoscenW<'a, REG> = crate::BitWriter<'a, REG, Soscen>;
impl<'a, REG> SoscenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "SOSC is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Soscen::Disabled)
    }
    #[doc = "SOSC is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Soscen::Enabled)
    }
}
#[doc = "SOSC Stop Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Soscsten {
    #[doc = "0: SOSC is disabled in Deep Sleep mode"]
    Disabled = 0,
    #[doc = "1: SOSC is enabled in Deep Sleep mode only if SOSCEN is set"]
    Enabled = 1,
}
impl From<Soscsten> for bool {
    #[inline(always)]
    fn from(variant: Soscsten) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SOSCSTEN` reader - SOSC Stop Enable"]
pub type SoscstenR = crate::BitReader<Soscsten>;
impl SoscstenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Soscsten {
        match self.bits {
            false => Soscsten::Disabled,
            true => Soscsten::Enabled,
        }
    }
    #[doc = "SOSC is disabled in Deep Sleep mode"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Soscsten::Disabled
    }
    #[doc = "SOSC is enabled in Deep Sleep mode only if SOSCEN is set"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Soscsten::Enabled
    }
}
#[doc = "Field `SOSCSTEN` writer - SOSC Stop Enable"]
pub type SoscstenW<'a, REG> = crate::BitWriter<'a, REG, Soscsten>;
impl<'a, REG> SoscstenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "SOSC is disabled in Deep Sleep mode"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Soscsten::Disabled)
    }
    #[doc = "SOSC is enabled in Deep Sleep mode only if SOSCEN is set"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Soscsten::Enabled)
    }
}
#[doc = "SOSC Clock Monitor Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sosccm {
    #[doc = "0: SOSC Clock Monitor is disabled"]
    Disabled = 0,
    #[doc = "1: SOSC Clock Monitor is enabled"]
    Enabled = 1,
}
impl From<Sosccm> for bool {
    #[inline(always)]
    fn from(variant: Sosccm) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SOSCCM` reader - SOSC Clock Monitor Enable"]
pub type SosccmR = crate::BitReader<Sosccm>;
impl SosccmR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sosccm {
        match self.bits {
            false => Sosccm::Disabled,
            true => Sosccm::Enabled,
        }
    }
    #[doc = "SOSC Clock Monitor is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Sosccm::Disabled
    }
    #[doc = "SOSC Clock Monitor is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Sosccm::Enabled
    }
}
#[doc = "Field `SOSCCM` writer - SOSC Clock Monitor Enable"]
pub type SosccmW<'a, REG> = crate::BitWriter<'a, REG, Sosccm>;
impl<'a, REG> SosccmW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "SOSC Clock Monitor is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Sosccm::Disabled)
    }
    #[doc = "SOSC Clock Monitor is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Sosccm::Enabled)
    }
}
#[doc = "SOSC Clock Monitor Reset Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sosccmre {
    #[doc = "0: Clock monitor generates an interrupt when an error is detected"]
    GenerateInterrupt = 0,
    #[doc = "1: Clock monitor generates a reset when an error is detected"]
    GenerateReset = 1,
}
impl From<Sosccmre> for bool {
    #[inline(always)]
    fn from(variant: Sosccmre) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SOSCCMRE` reader - SOSC Clock Monitor Reset Enable"]
pub type SosccmreR = crate::BitReader<Sosccmre>;
impl SosccmreR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sosccmre {
        match self.bits {
            false => Sosccmre::GenerateInterrupt,
            true => Sosccmre::GenerateReset,
        }
    }
    #[doc = "Clock monitor generates an interrupt when an error is detected"]
    #[inline(always)]
    pub fn is_generate_interrupt(&self) -> bool {
        *self == Sosccmre::GenerateInterrupt
    }
    #[doc = "Clock monitor generates a reset when an error is detected"]
    #[inline(always)]
    pub fn is_generate_reset(&self) -> bool {
        *self == Sosccmre::GenerateReset
    }
}
#[doc = "Field `SOSCCMRE` writer - SOSC Clock Monitor Reset Enable"]
pub type SosccmreW<'a, REG> = crate::BitWriter<'a, REG, Sosccmre>;
impl<'a, REG> SosccmreW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Clock monitor generates an interrupt when an error is detected"]
    #[inline(always)]
    pub fn generate_interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Sosccmre::GenerateInterrupt)
    }
    #[doc = "Clock monitor generates a reset when an error is detected"]
    #[inline(always)]
    pub fn generate_reset(self) -> &'a mut crate::W<REG> {
        self.variant(Sosccmre::GenerateReset)
    }
}
#[doc = "Lock Register\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lk {
    #[doc = "0: This Control Status Register can be written"]
    WriteEnabled = 0,
    #[doc = "1: This Control Status Register cannot be written"]
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
    #[doc = "This Control Status Register can be written"]
    #[inline(always)]
    pub fn is_write_enabled(&self) -> bool {
        *self == Lk::WriteEnabled
    }
    #[doc = "This Control Status Register cannot be written"]
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
    #[doc = "This Control Status Register can be written"]
    #[inline(always)]
    pub fn write_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lk::WriteEnabled)
    }
    #[doc = "This Control Status Register cannot be written"]
    #[inline(always)]
    pub fn write_disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lk::WriteDisabled)
    }
}
#[doc = "SOSC Valid\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Soscvld {
    #[doc = "0: SOSC is not enabled or clock is not valid"]
    Disabled = 0,
    #[doc = "1: SOSC is enabled and output clock is valid"]
    Enabled = 1,
}
impl From<Soscvld> for bool {
    #[inline(always)]
    fn from(variant: Soscvld) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SOSCVLD` reader - SOSC Valid"]
pub type SoscvldR = crate::BitReader<Soscvld>;
impl SoscvldR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Soscvld {
        match self.bits {
            false => Soscvld::Disabled,
            true => Soscvld::Enabled,
        }
    }
    #[doc = "SOSC is not enabled or clock is not valid"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Soscvld::Disabled
    }
    #[doc = "SOSC is enabled and output clock is valid"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Soscvld::Enabled
    }
}
#[doc = "SOSC Selected\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Soscsel {
    #[doc = "0: SOSC is not the system clock source"]
    NotSosc = 0,
    #[doc = "1: SOSC is the system clock source"]
    Sosc = 1,
}
impl From<Soscsel> for bool {
    #[inline(always)]
    fn from(variant: Soscsel) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SOSCSEL` reader - SOSC Selected"]
pub type SoscselR = crate::BitReader<Soscsel>;
impl SoscselR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Soscsel {
        match self.bits {
            false => Soscsel::NotSosc,
            true => Soscsel::Sosc,
        }
    }
    #[doc = "SOSC is not the system clock source"]
    #[inline(always)]
    pub fn is_not_sosc(&self) -> bool {
        *self == Soscsel::NotSosc
    }
    #[doc = "SOSC is the system clock source"]
    #[inline(always)]
    pub fn is_sosc(&self) -> bool {
        *self == Soscsel::Sosc
    }
}
#[doc = "SOSC Clock Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Soscerr {
    #[doc = "0: SOSC Clock Monitor is disabled or has not detected an error"]
    DisabledOrNoError = 0,
    #[doc = "1: SOSC Clock Monitor is enabled and detected an error"]
    EnabledAndError = 1,
}
impl From<Soscerr> for bool {
    #[inline(always)]
    fn from(variant: Soscerr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SOSCERR` reader - SOSC Clock Error"]
pub type SoscerrR = crate::BitReader<Soscerr>;
impl SoscerrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Soscerr {
        match self.bits {
            false => Soscerr::DisabledOrNoError,
            true => Soscerr::EnabledAndError,
        }
    }
    #[doc = "SOSC Clock Monitor is disabled or has not detected an error"]
    #[inline(always)]
    pub fn is_disabled_or_no_error(&self) -> bool {
        *self == Soscerr::DisabledOrNoError
    }
    #[doc = "SOSC Clock Monitor is enabled and detected an error"]
    #[inline(always)]
    pub fn is_enabled_and_error(&self) -> bool {
        *self == Soscerr::EnabledAndError
    }
}
#[doc = "Field `SOSCERR` writer - SOSC Clock Error"]
pub type SoscerrW<'a, REG> = crate::BitWriter1C<'a, REG, Soscerr>;
impl<'a, REG> SoscerrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "SOSC Clock Monitor is disabled or has not detected an error"]
    #[inline(always)]
    pub fn disabled_or_no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Soscerr::DisabledOrNoError)
    }
    #[doc = "SOSC Clock Monitor is enabled and detected an error"]
    #[inline(always)]
    pub fn enabled_and_error(self) -> &'a mut crate::W<REG> {
        self.variant(Soscerr::EnabledAndError)
    }
}
#[doc = "SOSC Valid Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoscvldIe {
    #[doc = "0: SOSCVLD interrupt is not enabled"]
    NotSosc = 0,
    #[doc = "1: SOSCVLD interrupt is enabled"]
    Sosc = 1,
}
impl From<SoscvldIe> for bool {
    #[inline(always)]
    fn from(variant: SoscvldIe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SOSCVLD_IE` reader - SOSC Valid Interrupt Enable"]
pub type SoscvldIeR = crate::BitReader<SoscvldIe>;
impl SoscvldIeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> SoscvldIe {
        match self.bits {
            false => SoscvldIe::NotSosc,
            true => SoscvldIe::Sosc,
        }
    }
    #[doc = "SOSCVLD interrupt is not enabled"]
    #[inline(always)]
    pub fn is_not_sosc(&self) -> bool {
        *self == SoscvldIe::NotSosc
    }
    #[doc = "SOSCVLD interrupt is enabled"]
    #[inline(always)]
    pub fn is_sosc(&self) -> bool {
        *self == SoscvldIe::Sosc
    }
}
#[doc = "Field `SOSCVLD_IE` writer - SOSC Valid Interrupt Enable"]
pub type SoscvldIeW<'a, REG> = crate::BitWriter<'a, REG, SoscvldIe>;
impl<'a, REG> SoscvldIeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "SOSCVLD interrupt is not enabled"]
    #[inline(always)]
    pub fn not_sosc(self) -> &'a mut crate::W<REG> {
        self.variant(SoscvldIe::NotSosc)
    }
    #[doc = "SOSCVLD interrupt is enabled"]
    #[inline(always)]
    pub fn sosc(self) -> &'a mut crate::W<REG> {
        self.variant(SoscvldIe::Sosc)
    }
}
#[doc = "SOSC clock safety enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoscSafeEn {
    #[doc = "0: SOSC clock safety is disabled"]
    Disable = 0,
    #[doc = "1: SOSC clock safety is enabled"]
    Enable = 1,
}
impl From<SoscSafeEn> for bool {
    #[inline(always)]
    fn from(variant: SoscSafeEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SOSC_SAFE_EN` reader - SOSC clock safety enable"]
pub type SoscSafeEnR = crate::BitReader<SoscSafeEn>;
impl SoscSafeEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> SoscSafeEn {
        match self.bits {
            false => SoscSafeEn::Disable,
            true => SoscSafeEn::Enable,
        }
    }
    #[doc = "SOSC clock safety is disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == SoscSafeEn::Disable
    }
    #[doc = "SOSC clock safety is enabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == SoscSafeEn::Enable
    }
}
#[doc = "Field `SOSC_SAFE_EN` writer - SOSC clock safety enable"]
pub type SoscSafeEnW<'a, REG> = crate::BitWriter<'a, REG, SoscSafeEn>;
impl<'a, REG> SoscSafeEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "SOSC clock safety is disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(SoscSafeEn::Disable)
    }
    #[doc = "SOSC clock safety is enabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(SoscSafeEn::Enable)
    }
}
impl R {
    #[doc = "Bit 0 - SOSC Enable"]
    #[inline(always)]
    pub fn soscen(&self) -> SoscenR {
        SoscenR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - SOSC Stop Enable"]
    #[inline(always)]
    pub fn soscsten(&self) -> SoscstenR {
        SoscstenR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 16 - SOSC Clock Monitor Enable"]
    #[inline(always)]
    pub fn sosccm(&self) -> SosccmR {
        SosccmR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - SOSC Clock Monitor Reset Enable"]
    #[inline(always)]
    pub fn sosccmre(&self) -> SosccmreR {
        SosccmreR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 23 - Lock Register"]
    #[inline(always)]
    pub fn lk(&self) -> LkR {
        LkR::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bit 24 - SOSC Valid"]
    #[inline(always)]
    pub fn soscvld(&self) -> SoscvldR {
        SoscvldR::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - SOSC Selected"]
    #[inline(always)]
    pub fn soscsel(&self) -> SoscselR {
        SoscselR::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - SOSC Clock Error"]
    #[inline(always)]
    pub fn soscerr(&self) -> SoscerrR {
        SoscerrR::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 30 - SOSC Valid Interrupt Enable"]
    #[inline(always)]
    pub fn soscvld_ie(&self) -> SoscvldIeR {
        SoscvldIeR::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - SOSC clock safety enable"]
    #[inline(always)]
    pub fn sosc_safe_en(&self) -> SoscSafeEnR {
        SoscSafeEnR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - SOSC Enable"]
    #[inline(always)]
    pub fn soscen(&mut self) -> SoscenW<SosccsrSpec> {
        SoscenW::new(self, 0)
    }
    #[doc = "Bit 1 - SOSC Stop Enable"]
    #[inline(always)]
    pub fn soscsten(&mut self) -> SoscstenW<SosccsrSpec> {
        SoscstenW::new(self, 1)
    }
    #[doc = "Bit 16 - SOSC Clock Monitor Enable"]
    #[inline(always)]
    pub fn sosccm(&mut self) -> SosccmW<SosccsrSpec> {
        SosccmW::new(self, 16)
    }
    #[doc = "Bit 17 - SOSC Clock Monitor Reset Enable"]
    #[inline(always)]
    pub fn sosccmre(&mut self) -> SosccmreW<SosccsrSpec> {
        SosccmreW::new(self, 17)
    }
    #[doc = "Bit 23 - Lock Register"]
    #[inline(always)]
    pub fn lk(&mut self) -> LkW<SosccsrSpec> {
        LkW::new(self, 23)
    }
    #[doc = "Bit 26 - SOSC Clock Error"]
    #[inline(always)]
    pub fn soscerr(&mut self) -> SoscerrW<SosccsrSpec> {
        SoscerrW::new(self, 26)
    }
    #[doc = "Bit 30 - SOSC Valid Interrupt Enable"]
    #[inline(always)]
    pub fn soscvld_ie(&mut self) -> SoscvldIeW<SosccsrSpec> {
        SoscvldIeW::new(self, 30)
    }
    #[doc = "Bit 31 - SOSC clock safety enable"]
    #[inline(always)]
    pub fn sosc_safe_en(&mut self) -> SoscSafeEnW<SosccsrSpec> {
        SoscSafeEnW::new(self, 31)
    }
}
#[doc = "SOSC Control Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sosccsr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sosccsr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SosccsrSpec;
impl crate::RegisterSpec for SosccsrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sosccsr::R`](R) reader structure"]
impl crate::Readable for SosccsrSpec {}
#[doc = "`write(|w| ..)` method takes [`sosccsr::W`](W) writer structure"]
impl crate::Writable for SosccsrSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x0400_0000;
}
#[doc = "`reset()` method sets SOSCCSR to value 0"]
impl crate::Resettable for SosccsrSpec {}

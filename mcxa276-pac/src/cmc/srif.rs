#[doc = "Register `SRIF` reader"]
pub type R = crate::R<SrifSpec>;
#[doc = "Register `SRIF` writer"]
pub type W = crate::W<SrifSpec>;
#[doc = "Pin Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin {
    #[doc = "0: Reset source not pending"]
    Disabled = 0,
    #[doc = "1: Reset source pending"]
    Enabled = 1,
}
impl From<Pin> for bool {
    #[inline(always)]
    fn from(variant: Pin) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN` reader - Pin Reset"]
pub type PinR = crate::BitReader<Pin>;
impl PinR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin {
        match self.bits {
            false => Pin::Disabled,
            true => Pin::Enabled,
        }
    }
    #[doc = "Reset source not pending"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Pin::Disabled
    }
    #[doc = "Reset source pending"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Pin::Enabled
    }
}
#[doc = "Field `PIN` writer - Pin Reset"]
pub type PinW<'a, REG> = crate::BitWriter1C<'a, REG, Pin>;
impl<'a, REG> PinW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Reset source not pending"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Pin::Disabled)
    }
    #[doc = "Reset source pending"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Pin::Enabled)
    }
}
#[doc = "DAP Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dap {
    #[doc = "0: Reset source not pending"]
    Disabled = 0,
    #[doc = "1: Reset source pending"]
    Enabled = 1,
}
impl From<Dap> for bool {
    #[inline(always)]
    fn from(variant: Dap) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DAP` reader - DAP Reset"]
pub type DapR = crate::BitReader<Dap>;
impl DapR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dap {
        match self.bits {
            false => Dap::Disabled,
            true => Dap::Enabled,
        }
    }
    #[doc = "Reset source not pending"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Dap::Disabled
    }
    #[doc = "Reset source pending"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Dap::Enabled
    }
}
#[doc = "Field `DAP` writer - DAP Reset"]
pub type DapW<'a, REG> = crate::BitWriter1C<'a, REG, Dap>;
impl<'a, REG> DapW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Reset source not pending"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Dap::Disabled)
    }
    #[doc = "Reset source pending"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Dap::Enabled)
    }
}
#[doc = "Low Power Acknowledge Timeout Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lpack {
    #[doc = "0: Reset source not pending"]
    Disabled = 0,
    #[doc = "1: Reset source pending"]
    Enabled = 1,
}
impl From<Lpack> for bool {
    #[inline(always)]
    fn from(variant: Lpack) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LPACK` reader - Low Power Acknowledge Timeout Reset"]
pub type LpackR = crate::BitReader<Lpack>;
impl LpackR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lpack {
        match self.bits {
            false => Lpack::Disabled,
            true => Lpack::Enabled,
        }
    }
    #[doc = "Reset source not pending"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Lpack::Disabled
    }
    #[doc = "Reset source pending"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Lpack::Enabled
    }
}
#[doc = "Field `LPACK` writer - Low Power Acknowledge Timeout Reset"]
pub type LpackW<'a, REG> = crate::BitWriter1C<'a, REG, Lpack>;
impl<'a, REG> LpackW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Reset source not pending"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpack::Disabled)
    }
    #[doc = "Reset source pending"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpack::Enabled)
    }
}
#[doc = "Windowed Watchdog 0 Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wwdt0 {
    #[doc = "0: Reset source not pending"]
    Disabled = 0,
    #[doc = "1: Reset source pending"]
    Enabled = 1,
}
impl From<Wwdt0> for bool {
    #[inline(always)]
    fn from(variant: Wwdt0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WWDT0` reader - Windowed Watchdog 0 Reset"]
pub type Wwdt0R = crate::BitReader<Wwdt0>;
impl Wwdt0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wwdt0 {
        match self.bits {
            false => Wwdt0::Disabled,
            true => Wwdt0::Enabled,
        }
    }
    #[doc = "Reset source not pending"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Wwdt0::Disabled
    }
    #[doc = "Reset source pending"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Wwdt0::Enabled
    }
}
#[doc = "Field `WWDT0` writer - Windowed Watchdog 0 Reset"]
pub type Wwdt0W<'a, REG> = crate::BitWriter1C<'a, REG, Wwdt0>;
impl<'a, REG> Wwdt0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Reset source not pending"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Wwdt0::Disabled)
    }
    #[doc = "Reset source pending"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Wwdt0::Enabled)
    }
}
#[doc = "Software Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sw {
    #[doc = "0: Reset source not pending"]
    Disabled = 0,
    #[doc = "1: Reset source pending"]
    Enabled = 1,
}
impl From<Sw> for bool {
    #[inline(always)]
    fn from(variant: Sw) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SW` reader - Software Reset"]
pub type SwR = crate::BitReader<Sw>;
impl SwR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sw {
        match self.bits {
            false => Sw::Disabled,
            true => Sw::Enabled,
        }
    }
    #[doc = "Reset source not pending"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Sw::Disabled
    }
    #[doc = "Reset source pending"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Sw::Enabled
    }
}
#[doc = "Field `SW` writer - Software Reset"]
pub type SwW<'a, REG> = crate::BitWriter1C<'a, REG, Sw>;
impl<'a, REG> SwW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Reset source not pending"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Sw::Disabled)
    }
    #[doc = "Reset source pending"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Sw::Enabled)
    }
}
#[doc = "Lockup Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lockup {
    #[doc = "0: Reset source not pending"]
    Disabled = 0,
    #[doc = "1: Reset source pending"]
    Enabled = 1,
}
impl From<Lockup> for bool {
    #[inline(always)]
    fn from(variant: Lockup) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LOCKUP` reader - Lockup Reset"]
pub type LockupR = crate::BitReader<Lockup>;
impl LockupR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lockup {
        match self.bits {
            false => Lockup::Disabled,
            true => Lockup::Enabled,
        }
    }
    #[doc = "Reset source not pending"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Lockup::Disabled
    }
    #[doc = "Reset source pending"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Lockup::Enabled
    }
}
#[doc = "Field `LOCKUP` writer - Lockup Reset"]
pub type LockupW<'a, REG> = crate::BitWriter1C<'a, REG, Lockup>;
impl<'a, REG> LockupW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Reset source not pending"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lockup::Disabled)
    }
    #[doc = "Reset source pending"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lockup::Enabled)
    }
}
#[doc = "Code Watchdog 0 Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cdog0 {
    #[doc = "0: Reset source not pending"]
    Disabled = 0,
    #[doc = "1: Reset source pending"]
    Enabled = 1,
}
impl From<Cdog0> for bool {
    #[inline(always)]
    fn from(variant: Cdog0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CDOG0` reader - Code Watchdog 0 Reset"]
pub type Cdog0R = crate::BitReader<Cdog0>;
impl Cdog0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cdog0 {
        match self.bits {
            false => Cdog0::Disabled,
            true => Cdog0::Enabled,
        }
    }
    #[doc = "Reset source not pending"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Cdog0::Disabled
    }
    #[doc = "Reset source pending"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Cdog0::Enabled
    }
}
#[doc = "Field `CDOG0` writer - Code Watchdog 0 Reset"]
pub type Cdog0W<'a, REG> = crate::BitWriter1C<'a, REG, Cdog0>;
impl<'a, REG> Cdog0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Reset source not pending"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Cdog0::Disabled)
    }
    #[doc = "Reset source pending"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Cdog0::Enabled)
    }
}
#[doc = "Code Watchdog 1 Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cdog1 {
    #[doc = "0: Reset source not pending"]
    Disabled = 0,
    #[doc = "1: Reset source pending"]
    Enabled = 1,
}
impl From<Cdog1> for bool {
    #[inline(always)]
    fn from(variant: Cdog1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CDOG1` reader - Code Watchdog 1 Reset"]
pub type Cdog1R = crate::BitReader<Cdog1>;
impl Cdog1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cdog1 {
        match self.bits {
            false => Cdog1::Disabled,
            true => Cdog1::Enabled,
        }
    }
    #[doc = "Reset source not pending"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Cdog1::Disabled
    }
    #[doc = "Reset source pending"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Cdog1::Enabled
    }
}
#[doc = "Field `CDOG1` writer - Code Watchdog 1 Reset"]
pub type Cdog1W<'a, REG> = crate::BitWriter1C<'a, REG, Cdog1>;
impl<'a, REG> Cdog1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Reset source not pending"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Cdog1::Disabled)
    }
    #[doc = "Reset source pending"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Cdog1::Enabled)
    }
}
impl R {
    #[doc = "Bit 8 - Pin Reset"]
    #[inline(always)]
    pub fn pin(&self) -> PinR {
        PinR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - DAP Reset"]
    #[inline(always)]
    pub fn dap(&self) -> DapR {
        DapR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 11 - Low Power Acknowledge Timeout Reset"]
    #[inline(always)]
    pub fn lpack(&self) -> LpackR {
        LpackR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 13 - Windowed Watchdog 0 Reset"]
    #[inline(always)]
    pub fn wwdt0(&self) -> Wwdt0R {
        Wwdt0R::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Software Reset"]
    #[inline(always)]
    pub fn sw(&self) -> SwR {
        SwR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Lockup Reset"]
    #[inline(always)]
    pub fn lockup(&self) -> LockupR {
        LockupR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 26 - Code Watchdog 0 Reset"]
    #[inline(always)]
    pub fn cdog0(&self) -> Cdog0R {
        Cdog0R::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - Code Watchdog 1 Reset"]
    #[inline(always)]
    pub fn cdog1(&self) -> Cdog1R {
        Cdog1R::new(((self.bits >> 27) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 8 - Pin Reset"]
    #[inline(always)]
    pub fn pin(&mut self) -> PinW<SrifSpec> {
        PinW::new(self, 8)
    }
    #[doc = "Bit 9 - DAP Reset"]
    #[inline(always)]
    pub fn dap(&mut self) -> DapW<SrifSpec> {
        DapW::new(self, 9)
    }
    #[doc = "Bit 11 - Low Power Acknowledge Timeout Reset"]
    #[inline(always)]
    pub fn lpack(&mut self) -> LpackW<SrifSpec> {
        LpackW::new(self, 11)
    }
    #[doc = "Bit 13 - Windowed Watchdog 0 Reset"]
    #[inline(always)]
    pub fn wwdt0(&mut self) -> Wwdt0W<SrifSpec> {
        Wwdt0W::new(self, 13)
    }
    #[doc = "Bit 14 - Software Reset"]
    #[inline(always)]
    pub fn sw(&mut self) -> SwW<SrifSpec> {
        SwW::new(self, 14)
    }
    #[doc = "Bit 15 - Lockup Reset"]
    #[inline(always)]
    pub fn lockup(&mut self) -> LockupW<SrifSpec> {
        LockupW::new(self, 15)
    }
    #[doc = "Bit 26 - Code Watchdog 0 Reset"]
    #[inline(always)]
    pub fn cdog0(&mut self) -> Cdog0W<SrifSpec> {
        Cdog0W::new(self, 26)
    }
    #[doc = "Bit 27 - Code Watchdog 1 Reset"]
    #[inline(always)]
    pub fn cdog1(&mut self) -> Cdog1W<SrifSpec> {
        Cdog1W::new(self, 27)
    }
}
#[doc = "System Reset Interrupt Flag\n\nYou can [`read`](crate::Reg::read) this register and get [`srif::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`srif::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SrifSpec;
impl crate::RegisterSpec for SrifSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`srif::R`](R) reader structure"]
impl crate::Readable for SrifSpec {}
#[doc = "`write(|w| ..)` method takes [`srif::W`](W) writer structure"]
impl crate::Writable for SrifSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x0c00_eb00;
}
#[doc = "`reset()` method sets SRIF to value 0"]
impl crate::Resettable for SrifSpec {}

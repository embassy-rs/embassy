#[doc = "Register `SSRS` reader"]
pub type R = crate::R<SsrsSpec>;
#[doc = "Register `SSRS` writer"]
pub type W = crate::W<SsrsSpec>;
#[doc = "Wake-up Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wakeup {
    #[doc = "0: Reset not generated"]
    Disabled = 0,
    #[doc = "1: Reset generated"]
    Enabled = 1,
}
impl From<Wakeup> for bool {
    #[inline(always)]
    fn from(variant: Wakeup) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WAKEUP` reader - Wake-up Reset"]
pub type WakeupR = crate::BitReader<Wakeup>;
impl WakeupR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wakeup {
        match self.bits {
            false => Wakeup::Disabled,
            true => Wakeup::Enabled,
        }
    }
    #[doc = "Reset not generated"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Wakeup::Disabled
    }
    #[doc = "Reset generated"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Wakeup::Enabled
    }
}
#[doc = "Field `WAKEUP` writer - Wake-up Reset"]
pub type WakeupW<'a, REG> = crate::BitWriter1C<'a, REG, Wakeup>;
impl<'a, REG> WakeupW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Reset not generated"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Wakeup::Disabled)
    }
    #[doc = "Reset generated"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Wakeup::Enabled)
    }
}
#[doc = "Power-on Reset\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Por {
    #[doc = "0: Reset not generated"]
    Disabled = 0,
    #[doc = "1: Reset generated"]
    Enabled = 1,
}
impl From<Por> for bool {
    #[inline(always)]
    fn from(variant: Por) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `POR` reader - Power-on Reset"]
pub type PorR = crate::BitReader<Por>;
impl PorR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Por {
        match self.bits {
            false => Por::Disabled,
            true => Por::Enabled,
        }
    }
    #[doc = "Reset not generated"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Por::Disabled
    }
    #[doc = "Reset generated"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Por::Enabled
    }
}
#[doc = "Field `POR` writer - Power-on Reset"]
pub type PorW<'a, REG> = crate::BitWriter1C<'a, REG, Por>;
impl<'a, REG> PorW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Reset not generated"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Por::Disabled)
    }
    #[doc = "Reset generated"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Por::Enabled)
    }
}
#[doc = "Voltage Detect Reset\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vd {
    #[doc = "0: Reset not generated"]
    Disabled = 0,
    #[doc = "1: Reset generated"]
    Enabled = 1,
}
impl From<Vd> for bool {
    #[inline(always)]
    fn from(variant: Vd) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `VD` reader - Voltage Detect Reset"]
pub type VdR = crate::BitReader<Vd>;
impl VdR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Vd {
        match self.bits {
            false => Vd::Disabled,
            true => Vd::Enabled,
        }
    }
    #[doc = "Reset not generated"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Vd::Disabled
    }
    #[doc = "Reset generated"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Vd::Enabled
    }
}
#[doc = "Warm Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Warm {
    #[doc = "0: Reset not generated"]
    Disabled = 0,
    #[doc = "1: Reset generated"]
    Enabled = 1,
}
impl From<Warm> for bool {
    #[inline(always)]
    fn from(variant: Warm) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WARM` reader - Warm Reset"]
pub type WarmR = crate::BitReader<Warm>;
impl WarmR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Warm {
        match self.bits {
            false => Warm::Disabled,
            true => Warm::Enabled,
        }
    }
    #[doc = "Reset not generated"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Warm::Disabled
    }
    #[doc = "Reset generated"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Warm::Enabled
    }
}
#[doc = "Field `WARM` writer - Warm Reset"]
pub type WarmW<'a, REG> = crate::BitWriter1C<'a, REG, Warm>;
impl<'a, REG> WarmW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Reset not generated"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Warm::Disabled)
    }
    #[doc = "Reset generated"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Warm::Enabled)
    }
}
#[doc = "Fatal Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fatal {
    #[doc = "0: Reset was not generated"]
    Disabled = 0,
    #[doc = "1: Reset was generated"]
    Enabled = 1,
}
impl From<Fatal> for bool {
    #[inline(always)]
    fn from(variant: Fatal) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FATAL` reader - Fatal Reset"]
pub type FatalR = crate::BitReader<Fatal>;
impl FatalR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fatal {
        match self.bits {
            false => Fatal::Disabled,
            true => Fatal::Enabled,
        }
    }
    #[doc = "Reset was not generated"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Fatal::Disabled
    }
    #[doc = "Reset was generated"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Fatal::Enabled
    }
}
#[doc = "Field `FATAL` writer - Fatal Reset"]
pub type FatalW<'a, REG> = crate::BitWriter1C<'a, REG, Fatal>;
impl<'a, REG> FatalW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Reset was not generated"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Fatal::Disabled)
    }
    #[doc = "Reset was generated"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Fatal::Enabled)
    }
}
#[doc = "Pin Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin {
    #[doc = "0: Reset not generated"]
    Disabled = 0,
    #[doc = "1: Reset generated"]
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
    #[doc = "Reset not generated"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Pin::Disabled
    }
    #[doc = "Reset generated"]
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
    #[doc = "Reset not generated"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Pin::Disabled)
    }
    #[doc = "Reset generated"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Pin::Enabled)
    }
}
#[doc = "DAP Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dap {
    #[doc = "0: Reset not generated"]
    Disabled = 0,
    #[doc = "1: Reset generated"]
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
    #[doc = "Reset not generated"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Dap::Disabled
    }
    #[doc = "Reset generated"]
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
    #[doc = "Reset not generated"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Dap::Disabled)
    }
    #[doc = "Reset generated"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Dap::Enabled)
    }
}
#[doc = "Reset Timeout\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rstack {
    #[doc = "0: Reset not generated"]
    Disabled = 0,
    #[doc = "1: Reset generated"]
    Enabled = 1,
}
impl From<Rstack> for bool {
    #[inline(always)]
    fn from(variant: Rstack) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RSTACK` reader - Reset Timeout"]
pub type RstackR = crate::BitReader<Rstack>;
impl RstackR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rstack {
        match self.bits {
            false => Rstack::Disabled,
            true => Rstack::Enabled,
        }
    }
    #[doc = "Reset not generated"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Rstack::Disabled
    }
    #[doc = "Reset generated"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Rstack::Enabled
    }
}
#[doc = "Field `RSTACK` writer - Reset Timeout"]
pub type RstackW<'a, REG> = crate::BitWriter1C<'a, REG, Rstack>;
impl<'a, REG> RstackW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Reset not generated"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rstack::Disabled)
    }
    #[doc = "Reset generated"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rstack::Enabled)
    }
}
#[doc = "Low Power Acknowledge Timeout Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lpack {
    #[doc = "0: Reset not generated"]
    Disabled = 0,
    #[doc = "1: Reset generated"]
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
    #[doc = "Reset not generated"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Lpack::Disabled
    }
    #[doc = "Reset generated"]
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
    #[doc = "Reset not generated"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpack::Disabled)
    }
    #[doc = "Reset generated"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpack::Enabled)
    }
}
#[doc = "System Clock Generation Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scg {
    #[doc = "0: Reset is not generated"]
    Disabled = 0,
    #[doc = "1: Reset is generated"]
    Enabled = 1,
}
impl From<Scg> for bool {
    #[inline(always)]
    fn from(variant: Scg) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SCG` reader - System Clock Generation Reset"]
pub type ScgR = crate::BitReader<Scg>;
impl ScgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Scg {
        match self.bits {
            false => Scg::Disabled,
            true => Scg::Enabled,
        }
    }
    #[doc = "Reset is not generated"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Scg::Disabled
    }
    #[doc = "Reset is generated"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Scg::Enabled
    }
}
#[doc = "Field `SCG` writer - System Clock Generation Reset"]
pub type ScgW<'a, REG> = crate::BitWriter1C<'a, REG, Scg>;
impl<'a, REG> ScgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Reset is not generated"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Scg::Disabled)
    }
    #[doc = "Reset is generated"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Scg::Enabled)
    }
}
#[doc = "Windowed Watchdog 0 Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wwdt0 {
    #[doc = "0: Reset is not generated"]
    Disabled = 0,
    #[doc = "1: Reset is generated"]
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
    #[doc = "Reset is not generated"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Wwdt0::Disabled
    }
    #[doc = "Reset is generated"]
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
    #[doc = "Reset is not generated"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Wwdt0::Disabled)
    }
    #[doc = "Reset is generated"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Wwdt0::Enabled)
    }
}
#[doc = "Software Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sw {
    #[doc = "0: Reset not generated"]
    Disabled = 0,
    #[doc = "1: Reset generated"]
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
    #[doc = "Reset not generated"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Sw::Disabled
    }
    #[doc = "Reset generated"]
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
    #[doc = "Reset not generated"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Sw::Disabled)
    }
    #[doc = "Reset generated"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Sw::Enabled)
    }
}
#[doc = "Lockup Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lockup {
    #[doc = "0: Reset not generated"]
    Disabled = 0,
    #[doc = "1: Reset generated"]
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
    #[doc = "Reset not generated"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Lockup::Disabled
    }
    #[doc = "Reset generated"]
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
    #[doc = "Reset not generated"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lockup::Disabled)
    }
    #[doc = "Reset generated"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lockup::Enabled)
    }
}
#[doc = "Code Watchdog 0 Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cdog0 {
    #[doc = "0: Reset is not generated"]
    Disabled = 0,
    #[doc = "1: Reset is generated"]
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
    #[doc = "Reset is not generated"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Cdog0::Disabled
    }
    #[doc = "Reset is generated"]
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
    #[doc = "Reset is not generated"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Cdog0::Disabled)
    }
    #[doc = "Reset is generated"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Cdog0::Enabled)
    }
}
#[doc = "Code Watchdog 1 Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cdog1 {
    #[doc = "0: Reset is not generated"]
    Disabled = 0,
    #[doc = "1: Reset is generated"]
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
    #[doc = "Reset is not generated"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Cdog1::Disabled
    }
    #[doc = "Reset is generated"]
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
    #[doc = "Reset is not generated"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Cdog1::Disabled)
    }
    #[doc = "Reset is generated"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Cdog1::Enabled)
    }
}
#[doc = "JTAG System Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Jtag {
    #[doc = "0: Reset not generated"]
    Disabled = 0,
    #[doc = "1: Reset generated"]
    Enabled = 1,
}
impl From<Jtag> for bool {
    #[inline(always)]
    fn from(variant: Jtag) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `JTAG` reader - JTAG System Reset"]
pub type JtagR = crate::BitReader<Jtag>;
impl JtagR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Jtag {
        match self.bits {
            false => Jtag::Disabled,
            true => Jtag::Enabled,
        }
    }
    #[doc = "Reset not generated"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Jtag::Disabled
    }
    #[doc = "Reset generated"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Jtag::Enabled
    }
}
#[doc = "Field `JTAG` writer - JTAG System Reset"]
pub type JtagW<'a, REG> = crate::BitWriter1C<'a, REG, Jtag>;
impl<'a, REG> JtagW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Reset not generated"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Jtag::Disabled)
    }
    #[doc = "Reset generated"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Jtag::Enabled)
    }
}
#[doc = "Tamper Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tamper {
    #[doc = "0: Reset not generated"]
    Disabled = 0,
    #[doc = "1: Reset generated"]
    Enabled = 1,
}
impl From<Tamper> for bool {
    #[inline(always)]
    fn from(variant: Tamper) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TAMPER` reader - Tamper Reset"]
pub type TamperR = crate::BitReader<Tamper>;
impl TamperR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tamper {
        match self.bits {
            false => Tamper::Disabled,
            true => Tamper::Enabled,
        }
    }
    #[doc = "Reset not generated"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Tamper::Disabled
    }
    #[doc = "Reset generated"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Tamper::Enabled
    }
}
#[doc = "Field `TAMPER` writer - Tamper Reset"]
pub type TamperW<'a, REG> = crate::BitWriter1C<'a, REG, Tamper>;
impl<'a, REG> TamperW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Reset not generated"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Tamper::Disabled)
    }
    #[doc = "Reset generated"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Tamper::Enabled)
    }
}
impl R {
    #[doc = "Bit 0 - Wake-up Reset"]
    #[inline(always)]
    pub fn wakeup(&self) -> WakeupR {
        WakeupR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Power-on Reset"]
    #[inline(always)]
    pub fn por(&self) -> PorR {
        PorR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Voltage Detect Reset"]
    #[inline(always)]
    pub fn vd(&self) -> VdR {
        VdR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 4 - Warm Reset"]
    #[inline(always)]
    pub fn warm(&self) -> WarmR {
        WarmR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Fatal Reset"]
    #[inline(always)]
    pub fn fatal(&self) -> FatalR {
        FatalR::new(((self.bits >> 5) & 1) != 0)
    }
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
    #[doc = "Bit 10 - Reset Timeout"]
    #[inline(always)]
    pub fn rstack(&self) -> RstackR {
        RstackR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Low Power Acknowledge Timeout Reset"]
    #[inline(always)]
    pub fn lpack(&self) -> LpackR {
        LpackR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - System Clock Generation Reset"]
    #[inline(always)]
    pub fn scg(&self) -> ScgR {
        ScgR::new(((self.bits >> 12) & 1) != 0)
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
    #[doc = "Bit 28 - JTAG System Reset"]
    #[inline(always)]
    pub fn jtag(&self) -> JtagR {
        JtagR::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 31 - Tamper Reset"]
    #[inline(always)]
    pub fn tamper(&self) -> TamperR {
        TamperR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Wake-up Reset"]
    #[inline(always)]
    pub fn wakeup(&mut self) -> WakeupW<SsrsSpec> {
        WakeupW::new(self, 0)
    }
    #[doc = "Bit 1 - Power-on Reset"]
    #[inline(always)]
    pub fn por(&mut self) -> PorW<SsrsSpec> {
        PorW::new(self, 1)
    }
    #[doc = "Bit 4 - Warm Reset"]
    #[inline(always)]
    pub fn warm(&mut self) -> WarmW<SsrsSpec> {
        WarmW::new(self, 4)
    }
    #[doc = "Bit 5 - Fatal Reset"]
    #[inline(always)]
    pub fn fatal(&mut self) -> FatalW<SsrsSpec> {
        FatalW::new(self, 5)
    }
    #[doc = "Bit 8 - Pin Reset"]
    #[inline(always)]
    pub fn pin(&mut self) -> PinW<SsrsSpec> {
        PinW::new(self, 8)
    }
    #[doc = "Bit 9 - DAP Reset"]
    #[inline(always)]
    pub fn dap(&mut self) -> DapW<SsrsSpec> {
        DapW::new(self, 9)
    }
    #[doc = "Bit 10 - Reset Timeout"]
    #[inline(always)]
    pub fn rstack(&mut self) -> RstackW<SsrsSpec> {
        RstackW::new(self, 10)
    }
    #[doc = "Bit 11 - Low Power Acknowledge Timeout Reset"]
    #[inline(always)]
    pub fn lpack(&mut self) -> LpackW<SsrsSpec> {
        LpackW::new(self, 11)
    }
    #[doc = "Bit 12 - System Clock Generation Reset"]
    #[inline(always)]
    pub fn scg(&mut self) -> ScgW<SsrsSpec> {
        ScgW::new(self, 12)
    }
    #[doc = "Bit 13 - Windowed Watchdog 0 Reset"]
    #[inline(always)]
    pub fn wwdt0(&mut self) -> Wwdt0W<SsrsSpec> {
        Wwdt0W::new(self, 13)
    }
    #[doc = "Bit 14 - Software Reset"]
    #[inline(always)]
    pub fn sw(&mut self) -> SwW<SsrsSpec> {
        SwW::new(self, 14)
    }
    #[doc = "Bit 15 - Lockup Reset"]
    #[inline(always)]
    pub fn lockup(&mut self) -> LockupW<SsrsSpec> {
        LockupW::new(self, 15)
    }
    #[doc = "Bit 26 - Code Watchdog 0 Reset"]
    #[inline(always)]
    pub fn cdog0(&mut self) -> Cdog0W<SsrsSpec> {
        Cdog0W::new(self, 26)
    }
    #[doc = "Bit 27 - Code Watchdog 1 Reset"]
    #[inline(always)]
    pub fn cdog1(&mut self) -> Cdog1W<SsrsSpec> {
        Cdog1W::new(self, 27)
    }
    #[doc = "Bit 28 - JTAG System Reset"]
    #[inline(always)]
    pub fn jtag(&mut self) -> JtagW<SsrsSpec> {
        JtagW::new(self, 28)
    }
    #[doc = "Bit 31 - Tamper Reset"]
    #[inline(always)]
    pub fn tamper(&mut self) -> TamperW<SsrsSpec> {
        TamperW::new(self, 31)
    }
}
#[doc = "Sticky System Reset Status\n\nYou can [`read`](crate::Reg::read) this register and get [`ssrs::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ssrs::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SsrsSpec;
impl crate::RegisterSpec for SsrsSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ssrs::R`](R) reader structure"]
impl crate::Readable for SsrsSpec {}
#[doc = "`write(|w| ..)` method takes [`ssrs::W`](W) writer structure"]
impl crate::Writable for SsrsSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x9c00_ff33;
}
#[doc = "`reset()` method sets SSRS to value 0x06"]
impl crate::Resettable for SsrsSpec {
    const RESET_VALUE: u32 = 0x06;
}

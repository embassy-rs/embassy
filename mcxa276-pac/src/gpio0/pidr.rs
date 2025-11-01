#[doc = "Register `PIDR` reader"]
pub type R = crate::R<PidrSpec>;
#[doc = "Register `PIDR` writer"]
pub type W = crate::W<PidrSpec>;
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid0 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid0> for bool {
    #[inline(always)]
    fn from(variant: Pid0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID0` reader - Port Input Disable"]
pub type Pid0R = crate::BitReader<Pid0>;
impl Pid0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid0 {
        match self.bits {
            false => Pid0::Pid0,
            true => Pid0::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid0::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid0::Pid1
    }
}
#[doc = "Field `PID0` writer - Port Input Disable"]
pub type Pid0W<'a, REG> = crate::BitWriter<'a, REG, Pid0>;
impl<'a, REG> Pid0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid0::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid0::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid1 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid1> for bool {
    #[inline(always)]
    fn from(variant: Pid1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID1` reader - Port Input Disable"]
pub type Pid1R = crate::BitReader<Pid1>;
impl Pid1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid1 {
        match self.bits {
            false => Pid1::Pid0,
            true => Pid1::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid1::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid1::Pid1
    }
}
#[doc = "Field `PID1` writer - Port Input Disable"]
pub type Pid1W<'a, REG> = crate::BitWriter<'a, REG, Pid1>;
impl<'a, REG> Pid1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid1::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid1::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid2 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid2> for bool {
    #[inline(always)]
    fn from(variant: Pid2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID2` reader - Port Input Disable"]
pub type Pid2R = crate::BitReader<Pid2>;
impl Pid2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid2 {
        match self.bits {
            false => Pid2::Pid0,
            true => Pid2::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid2::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid2::Pid1
    }
}
#[doc = "Field `PID2` writer - Port Input Disable"]
pub type Pid2W<'a, REG> = crate::BitWriter<'a, REG, Pid2>;
impl<'a, REG> Pid2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid2::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid2::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid3 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid3> for bool {
    #[inline(always)]
    fn from(variant: Pid3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID3` reader - Port Input Disable"]
pub type Pid3R = crate::BitReader<Pid3>;
impl Pid3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid3 {
        match self.bits {
            false => Pid3::Pid0,
            true => Pid3::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid3::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid3::Pid1
    }
}
#[doc = "Field `PID3` writer - Port Input Disable"]
pub type Pid3W<'a, REG> = crate::BitWriter<'a, REG, Pid3>;
impl<'a, REG> Pid3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid3::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid3::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid4 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid4> for bool {
    #[inline(always)]
    fn from(variant: Pid4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID4` reader - Port Input Disable"]
pub type Pid4R = crate::BitReader<Pid4>;
impl Pid4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid4 {
        match self.bits {
            false => Pid4::Pid0,
            true => Pid4::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid4::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid4::Pid1
    }
}
#[doc = "Field `PID4` writer - Port Input Disable"]
pub type Pid4W<'a, REG> = crate::BitWriter<'a, REG, Pid4>;
impl<'a, REG> Pid4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid4::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid4::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid5 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid5> for bool {
    #[inline(always)]
    fn from(variant: Pid5) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID5` reader - Port Input Disable"]
pub type Pid5R = crate::BitReader<Pid5>;
impl Pid5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid5 {
        match self.bits {
            false => Pid5::Pid0,
            true => Pid5::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid5::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid5::Pid1
    }
}
#[doc = "Field `PID5` writer - Port Input Disable"]
pub type Pid5W<'a, REG> = crate::BitWriter<'a, REG, Pid5>;
impl<'a, REG> Pid5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid5::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid5::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid6 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid6> for bool {
    #[inline(always)]
    fn from(variant: Pid6) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID6` reader - Port Input Disable"]
pub type Pid6R = crate::BitReader<Pid6>;
impl Pid6R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid6 {
        match self.bits {
            false => Pid6::Pid0,
            true => Pid6::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid6::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid6::Pid1
    }
}
#[doc = "Field `PID6` writer - Port Input Disable"]
pub type Pid6W<'a, REG> = crate::BitWriter<'a, REG, Pid6>;
impl<'a, REG> Pid6W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid6::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid6::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid7 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid7> for bool {
    #[inline(always)]
    fn from(variant: Pid7) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID7` reader - Port Input Disable"]
pub type Pid7R = crate::BitReader<Pid7>;
impl Pid7R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid7 {
        match self.bits {
            false => Pid7::Pid0,
            true => Pid7::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid7::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid7::Pid1
    }
}
#[doc = "Field `PID7` writer - Port Input Disable"]
pub type Pid7W<'a, REG> = crate::BitWriter<'a, REG, Pid7>;
impl<'a, REG> Pid7W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid7::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid7::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid8 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid8> for bool {
    #[inline(always)]
    fn from(variant: Pid8) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID8` reader - Port Input Disable"]
pub type Pid8R = crate::BitReader<Pid8>;
impl Pid8R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid8 {
        match self.bits {
            false => Pid8::Pid0,
            true => Pid8::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid8::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid8::Pid1
    }
}
#[doc = "Field `PID8` writer - Port Input Disable"]
pub type Pid8W<'a, REG> = crate::BitWriter<'a, REG, Pid8>;
impl<'a, REG> Pid8W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid8::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid8::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid9 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid9> for bool {
    #[inline(always)]
    fn from(variant: Pid9) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID9` reader - Port Input Disable"]
pub type Pid9R = crate::BitReader<Pid9>;
impl Pid9R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid9 {
        match self.bits {
            false => Pid9::Pid0,
            true => Pid9::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid9::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid9::Pid1
    }
}
#[doc = "Field `PID9` writer - Port Input Disable"]
pub type Pid9W<'a, REG> = crate::BitWriter<'a, REG, Pid9>;
impl<'a, REG> Pid9W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid9::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid9::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid10 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid10> for bool {
    #[inline(always)]
    fn from(variant: Pid10) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID10` reader - Port Input Disable"]
pub type Pid10R = crate::BitReader<Pid10>;
impl Pid10R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid10 {
        match self.bits {
            false => Pid10::Pid0,
            true => Pid10::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid10::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid10::Pid1
    }
}
#[doc = "Field `PID10` writer - Port Input Disable"]
pub type Pid10W<'a, REG> = crate::BitWriter<'a, REG, Pid10>;
impl<'a, REG> Pid10W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid10::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid10::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid11 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid11> for bool {
    #[inline(always)]
    fn from(variant: Pid11) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID11` reader - Port Input Disable"]
pub type Pid11R = crate::BitReader<Pid11>;
impl Pid11R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid11 {
        match self.bits {
            false => Pid11::Pid0,
            true => Pid11::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid11::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid11::Pid1
    }
}
#[doc = "Field `PID11` writer - Port Input Disable"]
pub type Pid11W<'a, REG> = crate::BitWriter<'a, REG, Pid11>;
impl<'a, REG> Pid11W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid11::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid11::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid12 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid12> for bool {
    #[inline(always)]
    fn from(variant: Pid12) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID12` reader - Port Input Disable"]
pub type Pid12R = crate::BitReader<Pid12>;
impl Pid12R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid12 {
        match self.bits {
            false => Pid12::Pid0,
            true => Pid12::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid12::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid12::Pid1
    }
}
#[doc = "Field `PID12` writer - Port Input Disable"]
pub type Pid12W<'a, REG> = crate::BitWriter<'a, REG, Pid12>;
impl<'a, REG> Pid12W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid12::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid12::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid13 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid13> for bool {
    #[inline(always)]
    fn from(variant: Pid13) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID13` reader - Port Input Disable"]
pub type Pid13R = crate::BitReader<Pid13>;
impl Pid13R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid13 {
        match self.bits {
            false => Pid13::Pid0,
            true => Pid13::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid13::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid13::Pid1
    }
}
#[doc = "Field `PID13` writer - Port Input Disable"]
pub type Pid13W<'a, REG> = crate::BitWriter<'a, REG, Pid13>;
impl<'a, REG> Pid13W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid13::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid13::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid14 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid14> for bool {
    #[inline(always)]
    fn from(variant: Pid14) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID14` reader - Port Input Disable"]
pub type Pid14R = crate::BitReader<Pid14>;
impl Pid14R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid14 {
        match self.bits {
            false => Pid14::Pid0,
            true => Pid14::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid14::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid14::Pid1
    }
}
#[doc = "Field `PID14` writer - Port Input Disable"]
pub type Pid14W<'a, REG> = crate::BitWriter<'a, REG, Pid14>;
impl<'a, REG> Pid14W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid14::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid14::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid15 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid15> for bool {
    #[inline(always)]
    fn from(variant: Pid15) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID15` reader - Port Input Disable"]
pub type Pid15R = crate::BitReader<Pid15>;
impl Pid15R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid15 {
        match self.bits {
            false => Pid15::Pid0,
            true => Pid15::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid15::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid15::Pid1
    }
}
#[doc = "Field `PID15` writer - Port Input Disable"]
pub type Pid15W<'a, REG> = crate::BitWriter<'a, REG, Pid15>;
impl<'a, REG> Pid15W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid15::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid15::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid16 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid16> for bool {
    #[inline(always)]
    fn from(variant: Pid16) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID16` reader - Port Input Disable"]
pub type Pid16R = crate::BitReader<Pid16>;
impl Pid16R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid16 {
        match self.bits {
            false => Pid16::Pid0,
            true => Pid16::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid16::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid16::Pid1
    }
}
#[doc = "Field `PID16` writer - Port Input Disable"]
pub type Pid16W<'a, REG> = crate::BitWriter<'a, REG, Pid16>;
impl<'a, REG> Pid16W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid16::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid16::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid17 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid17> for bool {
    #[inline(always)]
    fn from(variant: Pid17) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID17` reader - Port Input Disable"]
pub type Pid17R = crate::BitReader<Pid17>;
impl Pid17R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid17 {
        match self.bits {
            false => Pid17::Pid0,
            true => Pid17::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid17::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid17::Pid1
    }
}
#[doc = "Field `PID17` writer - Port Input Disable"]
pub type Pid17W<'a, REG> = crate::BitWriter<'a, REG, Pid17>;
impl<'a, REG> Pid17W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid17::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid17::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid18 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid18> for bool {
    #[inline(always)]
    fn from(variant: Pid18) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID18` reader - Port Input Disable"]
pub type Pid18R = crate::BitReader<Pid18>;
impl Pid18R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid18 {
        match self.bits {
            false => Pid18::Pid0,
            true => Pid18::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid18::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid18::Pid1
    }
}
#[doc = "Field `PID18` writer - Port Input Disable"]
pub type Pid18W<'a, REG> = crate::BitWriter<'a, REG, Pid18>;
impl<'a, REG> Pid18W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid18::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid18::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid19 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid19> for bool {
    #[inline(always)]
    fn from(variant: Pid19) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID19` reader - Port Input Disable"]
pub type Pid19R = crate::BitReader<Pid19>;
impl Pid19R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid19 {
        match self.bits {
            false => Pid19::Pid0,
            true => Pid19::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid19::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid19::Pid1
    }
}
#[doc = "Field `PID19` writer - Port Input Disable"]
pub type Pid19W<'a, REG> = crate::BitWriter<'a, REG, Pid19>;
impl<'a, REG> Pid19W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid19::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid19::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid20 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid20> for bool {
    #[inline(always)]
    fn from(variant: Pid20) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID20` reader - Port Input Disable"]
pub type Pid20R = crate::BitReader<Pid20>;
impl Pid20R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid20 {
        match self.bits {
            false => Pid20::Pid0,
            true => Pid20::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid20::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid20::Pid1
    }
}
#[doc = "Field `PID20` writer - Port Input Disable"]
pub type Pid20W<'a, REG> = crate::BitWriter<'a, REG, Pid20>;
impl<'a, REG> Pid20W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid20::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid20::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid21 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid21> for bool {
    #[inline(always)]
    fn from(variant: Pid21) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID21` reader - Port Input Disable"]
pub type Pid21R = crate::BitReader<Pid21>;
impl Pid21R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid21 {
        match self.bits {
            false => Pid21::Pid0,
            true => Pid21::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid21::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid21::Pid1
    }
}
#[doc = "Field `PID21` writer - Port Input Disable"]
pub type Pid21W<'a, REG> = crate::BitWriter<'a, REG, Pid21>;
impl<'a, REG> Pid21W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid21::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid21::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid22 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid22> for bool {
    #[inline(always)]
    fn from(variant: Pid22) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID22` reader - Port Input Disable"]
pub type Pid22R = crate::BitReader<Pid22>;
impl Pid22R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid22 {
        match self.bits {
            false => Pid22::Pid0,
            true => Pid22::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid22::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid22::Pid1
    }
}
#[doc = "Field `PID22` writer - Port Input Disable"]
pub type Pid22W<'a, REG> = crate::BitWriter<'a, REG, Pid22>;
impl<'a, REG> Pid22W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid22::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid22::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid23 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid23> for bool {
    #[inline(always)]
    fn from(variant: Pid23) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID23` reader - Port Input Disable"]
pub type Pid23R = crate::BitReader<Pid23>;
impl Pid23R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid23 {
        match self.bits {
            false => Pid23::Pid0,
            true => Pid23::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid23::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid23::Pid1
    }
}
#[doc = "Field `PID23` writer - Port Input Disable"]
pub type Pid23W<'a, REG> = crate::BitWriter<'a, REG, Pid23>;
impl<'a, REG> Pid23W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid23::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid23::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid24 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid24> for bool {
    #[inline(always)]
    fn from(variant: Pid24) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID24` reader - Port Input Disable"]
pub type Pid24R = crate::BitReader<Pid24>;
impl Pid24R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid24 {
        match self.bits {
            false => Pid24::Pid0,
            true => Pid24::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid24::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid24::Pid1
    }
}
#[doc = "Field `PID24` writer - Port Input Disable"]
pub type Pid24W<'a, REG> = crate::BitWriter<'a, REG, Pid24>;
impl<'a, REG> Pid24W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid24::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid24::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid25 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid25> for bool {
    #[inline(always)]
    fn from(variant: Pid25) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID25` reader - Port Input Disable"]
pub type Pid25R = crate::BitReader<Pid25>;
impl Pid25R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid25 {
        match self.bits {
            false => Pid25::Pid0,
            true => Pid25::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid25::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid25::Pid1
    }
}
#[doc = "Field `PID25` writer - Port Input Disable"]
pub type Pid25W<'a, REG> = crate::BitWriter<'a, REG, Pid25>;
impl<'a, REG> Pid25W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid25::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid25::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid26 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid26> for bool {
    #[inline(always)]
    fn from(variant: Pid26) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID26` reader - Port Input Disable"]
pub type Pid26R = crate::BitReader<Pid26>;
impl Pid26R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid26 {
        match self.bits {
            false => Pid26::Pid0,
            true => Pid26::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid26::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid26::Pid1
    }
}
#[doc = "Field `PID26` writer - Port Input Disable"]
pub type Pid26W<'a, REG> = crate::BitWriter<'a, REG, Pid26>;
impl<'a, REG> Pid26W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid26::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid26::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid27 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid27> for bool {
    #[inline(always)]
    fn from(variant: Pid27) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID27` reader - Port Input Disable"]
pub type Pid27R = crate::BitReader<Pid27>;
impl Pid27R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid27 {
        match self.bits {
            false => Pid27::Pid0,
            true => Pid27::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid27::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid27::Pid1
    }
}
#[doc = "Field `PID27` writer - Port Input Disable"]
pub type Pid27W<'a, REG> = crate::BitWriter<'a, REG, Pid27>;
impl<'a, REG> Pid27W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid27::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid27::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid28 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid28> for bool {
    #[inline(always)]
    fn from(variant: Pid28) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID28` reader - Port Input Disable"]
pub type Pid28R = crate::BitReader<Pid28>;
impl Pid28R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid28 {
        match self.bits {
            false => Pid28::Pid0,
            true => Pid28::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid28::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid28::Pid1
    }
}
#[doc = "Field `PID28` writer - Port Input Disable"]
pub type Pid28W<'a, REG> = crate::BitWriter<'a, REG, Pid28>;
impl<'a, REG> Pid28W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid28::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid28::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid29 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid29> for bool {
    #[inline(always)]
    fn from(variant: Pid29) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID29` reader - Port Input Disable"]
pub type Pid29R = crate::BitReader<Pid29>;
impl Pid29R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid29 {
        match self.bits {
            false => Pid29::Pid0,
            true => Pid29::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid29::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid29::Pid1
    }
}
#[doc = "Field `PID29` writer - Port Input Disable"]
pub type Pid29W<'a, REG> = crate::BitWriter<'a, REG, Pid29>;
impl<'a, REG> Pid29W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid29::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid29::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid30 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid30> for bool {
    #[inline(always)]
    fn from(variant: Pid30) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID30` reader - Port Input Disable"]
pub type Pid30R = crate::BitReader<Pid30>;
impl Pid30R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid30 {
        match self.bits {
            false => Pid30::Pid0,
            true => Pid30::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid30::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid30::Pid1
    }
}
#[doc = "Field `PID30` writer - Port Input Disable"]
pub type Pid30W<'a, REG> = crate::BitWriter<'a, REG, Pid30>;
impl<'a, REG> Pid30W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid30::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid30::Pid1)
    }
}
#[doc = "Port Input Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pid31 {
    #[doc = "0: Configured for general-purpose input"]
    Pid0 = 0,
    #[doc = "1: Disabled for general-purpose input"]
    Pid1 = 1,
}
impl From<Pid31> for bool {
    #[inline(always)]
    fn from(variant: Pid31) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PID31` reader - Port Input Disable"]
pub type Pid31R = crate::BitReader<Pid31>;
impl Pid31R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pid31 {
        match self.bits {
            false => Pid31::Pid0,
            true => Pid31::Pid1,
        }
    }
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn is_pid0(&self) -> bool {
        *self == Pid31::Pid0
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn is_pid1(&self) -> bool {
        *self == Pid31::Pid1
    }
}
#[doc = "Field `PID31` writer - Port Input Disable"]
pub type Pid31W<'a, REG> = crate::BitWriter<'a, REG, Pid31>;
impl<'a, REG> Pid31W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configured for general-purpose input"]
    #[inline(always)]
    pub fn pid0(self) -> &'a mut crate::W<REG> {
        self.variant(Pid31::Pid0)
    }
    #[doc = "Disabled for general-purpose input"]
    #[inline(always)]
    pub fn pid1(self) -> &'a mut crate::W<REG> {
        self.variant(Pid31::Pid1)
    }
}
impl R {
    #[doc = "Bit 0 - Port Input Disable"]
    #[inline(always)]
    pub fn pid0(&self) -> Pid0R {
        Pid0R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Port Input Disable"]
    #[inline(always)]
    pub fn pid1(&self) -> Pid1R {
        Pid1R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Port Input Disable"]
    #[inline(always)]
    pub fn pid2(&self) -> Pid2R {
        Pid2R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Port Input Disable"]
    #[inline(always)]
    pub fn pid3(&self) -> Pid3R {
        Pid3R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Port Input Disable"]
    #[inline(always)]
    pub fn pid4(&self) -> Pid4R {
        Pid4R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Port Input Disable"]
    #[inline(always)]
    pub fn pid5(&self) -> Pid5R {
        Pid5R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Port Input Disable"]
    #[inline(always)]
    pub fn pid6(&self) -> Pid6R {
        Pid6R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Port Input Disable"]
    #[inline(always)]
    pub fn pid7(&self) -> Pid7R {
        Pid7R::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Port Input Disable"]
    #[inline(always)]
    pub fn pid8(&self) -> Pid8R {
        Pid8R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Port Input Disable"]
    #[inline(always)]
    pub fn pid9(&self) -> Pid9R {
        Pid9R::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Port Input Disable"]
    #[inline(always)]
    pub fn pid10(&self) -> Pid10R {
        Pid10R::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Port Input Disable"]
    #[inline(always)]
    pub fn pid11(&self) -> Pid11R {
        Pid11R::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Port Input Disable"]
    #[inline(always)]
    pub fn pid12(&self) -> Pid12R {
        Pid12R::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Port Input Disable"]
    #[inline(always)]
    pub fn pid13(&self) -> Pid13R {
        Pid13R::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Port Input Disable"]
    #[inline(always)]
    pub fn pid14(&self) -> Pid14R {
        Pid14R::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Port Input Disable"]
    #[inline(always)]
    pub fn pid15(&self) -> Pid15R {
        Pid15R::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - Port Input Disable"]
    #[inline(always)]
    pub fn pid16(&self) -> Pid16R {
        Pid16R::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Port Input Disable"]
    #[inline(always)]
    pub fn pid17(&self) -> Pid17R {
        Pid17R::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Port Input Disable"]
    #[inline(always)]
    pub fn pid18(&self) -> Pid18R {
        Pid18R::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - Port Input Disable"]
    #[inline(always)]
    pub fn pid19(&self) -> Pid19R {
        Pid19R::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - Port Input Disable"]
    #[inline(always)]
    pub fn pid20(&self) -> Pid20R {
        Pid20R::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - Port Input Disable"]
    #[inline(always)]
    pub fn pid21(&self) -> Pid21R {
        Pid21R::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - Port Input Disable"]
    #[inline(always)]
    pub fn pid22(&self) -> Pid22R {
        Pid22R::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - Port Input Disable"]
    #[inline(always)]
    pub fn pid23(&self) -> Pid23R {
        Pid23R::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bit 24 - Port Input Disable"]
    #[inline(always)]
    pub fn pid24(&self) -> Pid24R {
        Pid24R::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - Port Input Disable"]
    #[inline(always)]
    pub fn pid25(&self) -> Pid25R {
        Pid25R::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - Port Input Disable"]
    #[inline(always)]
    pub fn pid26(&self) -> Pid26R {
        Pid26R::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - Port Input Disable"]
    #[inline(always)]
    pub fn pid27(&self) -> Pid27R {
        Pid27R::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bit 28 - Port Input Disable"]
    #[inline(always)]
    pub fn pid28(&self) -> Pid28R {
        Pid28R::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 29 - Port Input Disable"]
    #[inline(always)]
    pub fn pid29(&self) -> Pid29R {
        Pid29R::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - Port Input Disable"]
    #[inline(always)]
    pub fn pid30(&self) -> Pid30R {
        Pid30R::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Port Input Disable"]
    #[inline(always)]
    pub fn pid31(&self) -> Pid31R {
        Pid31R::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Port Input Disable"]
    #[inline(always)]
    pub fn pid0(&mut self) -> Pid0W<PidrSpec> {
        Pid0W::new(self, 0)
    }
    #[doc = "Bit 1 - Port Input Disable"]
    #[inline(always)]
    pub fn pid1(&mut self) -> Pid1W<PidrSpec> {
        Pid1W::new(self, 1)
    }
    #[doc = "Bit 2 - Port Input Disable"]
    #[inline(always)]
    pub fn pid2(&mut self) -> Pid2W<PidrSpec> {
        Pid2W::new(self, 2)
    }
    #[doc = "Bit 3 - Port Input Disable"]
    #[inline(always)]
    pub fn pid3(&mut self) -> Pid3W<PidrSpec> {
        Pid3W::new(self, 3)
    }
    #[doc = "Bit 4 - Port Input Disable"]
    #[inline(always)]
    pub fn pid4(&mut self) -> Pid4W<PidrSpec> {
        Pid4W::new(self, 4)
    }
    #[doc = "Bit 5 - Port Input Disable"]
    #[inline(always)]
    pub fn pid5(&mut self) -> Pid5W<PidrSpec> {
        Pid5W::new(self, 5)
    }
    #[doc = "Bit 6 - Port Input Disable"]
    #[inline(always)]
    pub fn pid6(&mut self) -> Pid6W<PidrSpec> {
        Pid6W::new(self, 6)
    }
    #[doc = "Bit 7 - Port Input Disable"]
    #[inline(always)]
    pub fn pid7(&mut self) -> Pid7W<PidrSpec> {
        Pid7W::new(self, 7)
    }
    #[doc = "Bit 8 - Port Input Disable"]
    #[inline(always)]
    pub fn pid8(&mut self) -> Pid8W<PidrSpec> {
        Pid8W::new(self, 8)
    }
    #[doc = "Bit 9 - Port Input Disable"]
    #[inline(always)]
    pub fn pid9(&mut self) -> Pid9W<PidrSpec> {
        Pid9W::new(self, 9)
    }
    #[doc = "Bit 10 - Port Input Disable"]
    #[inline(always)]
    pub fn pid10(&mut self) -> Pid10W<PidrSpec> {
        Pid10W::new(self, 10)
    }
    #[doc = "Bit 11 - Port Input Disable"]
    #[inline(always)]
    pub fn pid11(&mut self) -> Pid11W<PidrSpec> {
        Pid11W::new(self, 11)
    }
    #[doc = "Bit 12 - Port Input Disable"]
    #[inline(always)]
    pub fn pid12(&mut self) -> Pid12W<PidrSpec> {
        Pid12W::new(self, 12)
    }
    #[doc = "Bit 13 - Port Input Disable"]
    #[inline(always)]
    pub fn pid13(&mut self) -> Pid13W<PidrSpec> {
        Pid13W::new(self, 13)
    }
    #[doc = "Bit 14 - Port Input Disable"]
    #[inline(always)]
    pub fn pid14(&mut self) -> Pid14W<PidrSpec> {
        Pid14W::new(self, 14)
    }
    #[doc = "Bit 15 - Port Input Disable"]
    #[inline(always)]
    pub fn pid15(&mut self) -> Pid15W<PidrSpec> {
        Pid15W::new(self, 15)
    }
    #[doc = "Bit 16 - Port Input Disable"]
    #[inline(always)]
    pub fn pid16(&mut self) -> Pid16W<PidrSpec> {
        Pid16W::new(self, 16)
    }
    #[doc = "Bit 17 - Port Input Disable"]
    #[inline(always)]
    pub fn pid17(&mut self) -> Pid17W<PidrSpec> {
        Pid17W::new(self, 17)
    }
    #[doc = "Bit 18 - Port Input Disable"]
    #[inline(always)]
    pub fn pid18(&mut self) -> Pid18W<PidrSpec> {
        Pid18W::new(self, 18)
    }
    #[doc = "Bit 19 - Port Input Disable"]
    #[inline(always)]
    pub fn pid19(&mut self) -> Pid19W<PidrSpec> {
        Pid19W::new(self, 19)
    }
    #[doc = "Bit 20 - Port Input Disable"]
    #[inline(always)]
    pub fn pid20(&mut self) -> Pid20W<PidrSpec> {
        Pid20W::new(self, 20)
    }
    #[doc = "Bit 21 - Port Input Disable"]
    #[inline(always)]
    pub fn pid21(&mut self) -> Pid21W<PidrSpec> {
        Pid21W::new(self, 21)
    }
    #[doc = "Bit 22 - Port Input Disable"]
    #[inline(always)]
    pub fn pid22(&mut self) -> Pid22W<PidrSpec> {
        Pid22W::new(self, 22)
    }
    #[doc = "Bit 23 - Port Input Disable"]
    #[inline(always)]
    pub fn pid23(&mut self) -> Pid23W<PidrSpec> {
        Pid23W::new(self, 23)
    }
    #[doc = "Bit 24 - Port Input Disable"]
    #[inline(always)]
    pub fn pid24(&mut self) -> Pid24W<PidrSpec> {
        Pid24W::new(self, 24)
    }
    #[doc = "Bit 25 - Port Input Disable"]
    #[inline(always)]
    pub fn pid25(&mut self) -> Pid25W<PidrSpec> {
        Pid25W::new(self, 25)
    }
    #[doc = "Bit 26 - Port Input Disable"]
    #[inline(always)]
    pub fn pid26(&mut self) -> Pid26W<PidrSpec> {
        Pid26W::new(self, 26)
    }
    #[doc = "Bit 27 - Port Input Disable"]
    #[inline(always)]
    pub fn pid27(&mut self) -> Pid27W<PidrSpec> {
        Pid27W::new(self, 27)
    }
    #[doc = "Bit 28 - Port Input Disable"]
    #[inline(always)]
    pub fn pid28(&mut self) -> Pid28W<PidrSpec> {
        Pid28W::new(self, 28)
    }
    #[doc = "Bit 29 - Port Input Disable"]
    #[inline(always)]
    pub fn pid29(&mut self) -> Pid29W<PidrSpec> {
        Pid29W::new(self, 29)
    }
    #[doc = "Bit 30 - Port Input Disable"]
    #[inline(always)]
    pub fn pid30(&mut self) -> Pid30W<PidrSpec> {
        Pid30W::new(self, 30)
    }
    #[doc = "Bit 31 - Port Input Disable"]
    #[inline(always)]
    pub fn pid31(&mut self) -> Pid31W<PidrSpec> {
        Pid31W::new(self, 31)
    }
}
#[doc = "Port Input Disable\n\nYou can [`read`](crate::Reg::read) this register and get [`pidr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pidr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PidrSpec;
impl crate::RegisterSpec for PidrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pidr::R`](R) reader structure"]
impl crate::Readable for PidrSpec {}
#[doc = "`write(|w| ..)` method takes [`pidr::W`](W) writer structure"]
impl crate::Writable for PidrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PIDR to value 0"]
impl crate::Resettable for PidrSpec {}

#[doc = "Register `PTOR` reader"]
pub type R = crate::R<PtorSpec>;
#[doc = "Register `PTOR` writer"]
pub type W = crate::W<PtorSpec>;
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto0 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto0> for bool {
    #[inline(always)]
    fn from(variant: Ptto0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO0` reader - Port Toggle Output"]
pub type Ptto0R = crate::BitReader<Ptto0>;
impl Ptto0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto0 {
        match self.bits {
            false => Ptto0::Ptto0,
            true => Ptto0::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto0::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto0::Ptto1
    }
}
#[doc = "Field `PTTO0` writer - Port Toggle Output"]
pub type Ptto0W<'a, REG> = crate::BitWriter<'a, REG, Ptto0>;
impl<'a, REG> Ptto0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto0::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto0::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto1 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto1> for bool {
    #[inline(always)]
    fn from(variant: Ptto1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO1` reader - Port Toggle Output"]
pub type Ptto1R = crate::BitReader<Ptto1>;
impl Ptto1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto1 {
        match self.bits {
            false => Ptto1::Ptto0,
            true => Ptto1::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto1::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto1::Ptto1
    }
}
#[doc = "Field `PTTO1` writer - Port Toggle Output"]
pub type Ptto1W<'a, REG> = crate::BitWriter<'a, REG, Ptto1>;
impl<'a, REG> Ptto1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto1::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto1::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto2 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto2> for bool {
    #[inline(always)]
    fn from(variant: Ptto2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO2` reader - Port Toggle Output"]
pub type Ptto2R = crate::BitReader<Ptto2>;
impl Ptto2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto2 {
        match self.bits {
            false => Ptto2::Ptto0,
            true => Ptto2::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto2::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto2::Ptto1
    }
}
#[doc = "Field `PTTO2` writer - Port Toggle Output"]
pub type Ptto2W<'a, REG> = crate::BitWriter<'a, REG, Ptto2>;
impl<'a, REG> Ptto2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto2::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto2::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto3 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto3> for bool {
    #[inline(always)]
    fn from(variant: Ptto3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO3` reader - Port Toggle Output"]
pub type Ptto3R = crate::BitReader<Ptto3>;
impl Ptto3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto3 {
        match self.bits {
            false => Ptto3::Ptto0,
            true => Ptto3::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto3::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto3::Ptto1
    }
}
#[doc = "Field `PTTO3` writer - Port Toggle Output"]
pub type Ptto3W<'a, REG> = crate::BitWriter<'a, REG, Ptto3>;
impl<'a, REG> Ptto3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto3::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto3::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto4 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto4> for bool {
    #[inline(always)]
    fn from(variant: Ptto4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO4` reader - Port Toggle Output"]
pub type Ptto4R = crate::BitReader<Ptto4>;
impl Ptto4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto4 {
        match self.bits {
            false => Ptto4::Ptto0,
            true => Ptto4::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto4::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto4::Ptto1
    }
}
#[doc = "Field `PTTO4` writer - Port Toggle Output"]
pub type Ptto4W<'a, REG> = crate::BitWriter<'a, REG, Ptto4>;
impl<'a, REG> Ptto4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto4::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto4::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto5 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto5> for bool {
    #[inline(always)]
    fn from(variant: Ptto5) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO5` reader - Port Toggle Output"]
pub type Ptto5R = crate::BitReader<Ptto5>;
impl Ptto5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto5 {
        match self.bits {
            false => Ptto5::Ptto0,
            true => Ptto5::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto5::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto5::Ptto1
    }
}
#[doc = "Field `PTTO5` writer - Port Toggle Output"]
pub type Ptto5W<'a, REG> = crate::BitWriter<'a, REG, Ptto5>;
impl<'a, REG> Ptto5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto5::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto5::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto6 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto6> for bool {
    #[inline(always)]
    fn from(variant: Ptto6) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO6` reader - Port Toggle Output"]
pub type Ptto6R = crate::BitReader<Ptto6>;
impl Ptto6R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto6 {
        match self.bits {
            false => Ptto6::Ptto0,
            true => Ptto6::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto6::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto6::Ptto1
    }
}
#[doc = "Field `PTTO6` writer - Port Toggle Output"]
pub type Ptto6W<'a, REG> = crate::BitWriter<'a, REG, Ptto6>;
impl<'a, REG> Ptto6W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto6::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto6::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto7 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto7> for bool {
    #[inline(always)]
    fn from(variant: Ptto7) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO7` reader - Port Toggle Output"]
pub type Ptto7R = crate::BitReader<Ptto7>;
impl Ptto7R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto7 {
        match self.bits {
            false => Ptto7::Ptto0,
            true => Ptto7::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto7::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto7::Ptto1
    }
}
#[doc = "Field `PTTO7` writer - Port Toggle Output"]
pub type Ptto7W<'a, REG> = crate::BitWriter<'a, REG, Ptto7>;
impl<'a, REG> Ptto7W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto7::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto7::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto8 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto8> for bool {
    #[inline(always)]
    fn from(variant: Ptto8) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO8` reader - Port Toggle Output"]
pub type Ptto8R = crate::BitReader<Ptto8>;
impl Ptto8R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto8 {
        match self.bits {
            false => Ptto8::Ptto0,
            true => Ptto8::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto8::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto8::Ptto1
    }
}
#[doc = "Field `PTTO8` writer - Port Toggle Output"]
pub type Ptto8W<'a, REG> = crate::BitWriter<'a, REG, Ptto8>;
impl<'a, REG> Ptto8W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto8::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto8::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto9 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto9> for bool {
    #[inline(always)]
    fn from(variant: Ptto9) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO9` reader - Port Toggle Output"]
pub type Ptto9R = crate::BitReader<Ptto9>;
impl Ptto9R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto9 {
        match self.bits {
            false => Ptto9::Ptto0,
            true => Ptto9::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto9::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto9::Ptto1
    }
}
#[doc = "Field `PTTO9` writer - Port Toggle Output"]
pub type Ptto9W<'a, REG> = crate::BitWriter<'a, REG, Ptto9>;
impl<'a, REG> Ptto9W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto9::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto9::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto10 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto10> for bool {
    #[inline(always)]
    fn from(variant: Ptto10) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO10` reader - Port Toggle Output"]
pub type Ptto10R = crate::BitReader<Ptto10>;
impl Ptto10R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto10 {
        match self.bits {
            false => Ptto10::Ptto0,
            true => Ptto10::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto10::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto10::Ptto1
    }
}
#[doc = "Field `PTTO10` writer - Port Toggle Output"]
pub type Ptto10W<'a, REG> = crate::BitWriter<'a, REG, Ptto10>;
impl<'a, REG> Ptto10W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto10::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto10::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto11 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto11> for bool {
    #[inline(always)]
    fn from(variant: Ptto11) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO11` reader - Port Toggle Output"]
pub type Ptto11R = crate::BitReader<Ptto11>;
impl Ptto11R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto11 {
        match self.bits {
            false => Ptto11::Ptto0,
            true => Ptto11::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto11::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto11::Ptto1
    }
}
#[doc = "Field `PTTO11` writer - Port Toggle Output"]
pub type Ptto11W<'a, REG> = crate::BitWriter<'a, REG, Ptto11>;
impl<'a, REG> Ptto11W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto11::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto11::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto12 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto12> for bool {
    #[inline(always)]
    fn from(variant: Ptto12) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO12` reader - Port Toggle Output"]
pub type Ptto12R = crate::BitReader<Ptto12>;
impl Ptto12R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto12 {
        match self.bits {
            false => Ptto12::Ptto0,
            true => Ptto12::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto12::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto12::Ptto1
    }
}
#[doc = "Field `PTTO12` writer - Port Toggle Output"]
pub type Ptto12W<'a, REG> = crate::BitWriter<'a, REG, Ptto12>;
impl<'a, REG> Ptto12W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto12::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto12::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto13 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto13> for bool {
    #[inline(always)]
    fn from(variant: Ptto13) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO13` reader - Port Toggle Output"]
pub type Ptto13R = crate::BitReader<Ptto13>;
impl Ptto13R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto13 {
        match self.bits {
            false => Ptto13::Ptto0,
            true => Ptto13::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto13::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto13::Ptto1
    }
}
#[doc = "Field `PTTO13` writer - Port Toggle Output"]
pub type Ptto13W<'a, REG> = crate::BitWriter<'a, REG, Ptto13>;
impl<'a, REG> Ptto13W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto13::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto13::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto14 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto14> for bool {
    #[inline(always)]
    fn from(variant: Ptto14) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO14` reader - Port Toggle Output"]
pub type Ptto14R = crate::BitReader<Ptto14>;
impl Ptto14R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto14 {
        match self.bits {
            false => Ptto14::Ptto0,
            true => Ptto14::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto14::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto14::Ptto1
    }
}
#[doc = "Field `PTTO14` writer - Port Toggle Output"]
pub type Ptto14W<'a, REG> = crate::BitWriter<'a, REG, Ptto14>;
impl<'a, REG> Ptto14W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto14::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto14::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto15 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto15> for bool {
    #[inline(always)]
    fn from(variant: Ptto15) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO15` reader - Port Toggle Output"]
pub type Ptto15R = crate::BitReader<Ptto15>;
impl Ptto15R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto15 {
        match self.bits {
            false => Ptto15::Ptto0,
            true => Ptto15::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto15::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto15::Ptto1
    }
}
#[doc = "Field `PTTO15` writer - Port Toggle Output"]
pub type Ptto15W<'a, REG> = crate::BitWriter<'a, REG, Ptto15>;
impl<'a, REG> Ptto15W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto15::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto15::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto16 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto16> for bool {
    #[inline(always)]
    fn from(variant: Ptto16) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO16` reader - Port Toggle Output"]
pub type Ptto16R = crate::BitReader<Ptto16>;
impl Ptto16R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto16 {
        match self.bits {
            false => Ptto16::Ptto0,
            true => Ptto16::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto16::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto16::Ptto1
    }
}
#[doc = "Field `PTTO16` writer - Port Toggle Output"]
pub type Ptto16W<'a, REG> = crate::BitWriter<'a, REG, Ptto16>;
impl<'a, REG> Ptto16W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto16::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto16::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto17 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto17> for bool {
    #[inline(always)]
    fn from(variant: Ptto17) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO17` reader - Port Toggle Output"]
pub type Ptto17R = crate::BitReader<Ptto17>;
impl Ptto17R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto17 {
        match self.bits {
            false => Ptto17::Ptto0,
            true => Ptto17::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto17::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto17::Ptto1
    }
}
#[doc = "Field `PTTO17` writer - Port Toggle Output"]
pub type Ptto17W<'a, REG> = crate::BitWriter<'a, REG, Ptto17>;
impl<'a, REG> Ptto17W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto17::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto17::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto18 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto18> for bool {
    #[inline(always)]
    fn from(variant: Ptto18) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO18` reader - Port Toggle Output"]
pub type Ptto18R = crate::BitReader<Ptto18>;
impl Ptto18R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto18 {
        match self.bits {
            false => Ptto18::Ptto0,
            true => Ptto18::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto18::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto18::Ptto1
    }
}
#[doc = "Field `PTTO18` writer - Port Toggle Output"]
pub type Ptto18W<'a, REG> = crate::BitWriter<'a, REG, Ptto18>;
impl<'a, REG> Ptto18W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto18::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto18::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto19 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto19> for bool {
    #[inline(always)]
    fn from(variant: Ptto19) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO19` reader - Port Toggle Output"]
pub type Ptto19R = crate::BitReader<Ptto19>;
impl Ptto19R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto19 {
        match self.bits {
            false => Ptto19::Ptto0,
            true => Ptto19::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto19::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto19::Ptto1
    }
}
#[doc = "Field `PTTO19` writer - Port Toggle Output"]
pub type Ptto19W<'a, REG> = crate::BitWriter<'a, REG, Ptto19>;
impl<'a, REG> Ptto19W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto19::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto19::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto20 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto20> for bool {
    #[inline(always)]
    fn from(variant: Ptto20) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO20` reader - Port Toggle Output"]
pub type Ptto20R = crate::BitReader<Ptto20>;
impl Ptto20R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto20 {
        match self.bits {
            false => Ptto20::Ptto0,
            true => Ptto20::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto20::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto20::Ptto1
    }
}
#[doc = "Field `PTTO20` writer - Port Toggle Output"]
pub type Ptto20W<'a, REG> = crate::BitWriter<'a, REG, Ptto20>;
impl<'a, REG> Ptto20W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto20::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto20::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto21 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto21> for bool {
    #[inline(always)]
    fn from(variant: Ptto21) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO21` reader - Port Toggle Output"]
pub type Ptto21R = crate::BitReader<Ptto21>;
impl Ptto21R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto21 {
        match self.bits {
            false => Ptto21::Ptto0,
            true => Ptto21::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto21::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto21::Ptto1
    }
}
#[doc = "Field `PTTO21` writer - Port Toggle Output"]
pub type Ptto21W<'a, REG> = crate::BitWriter<'a, REG, Ptto21>;
impl<'a, REG> Ptto21W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto21::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto21::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto22 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto22> for bool {
    #[inline(always)]
    fn from(variant: Ptto22) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO22` reader - Port Toggle Output"]
pub type Ptto22R = crate::BitReader<Ptto22>;
impl Ptto22R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto22 {
        match self.bits {
            false => Ptto22::Ptto0,
            true => Ptto22::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto22::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto22::Ptto1
    }
}
#[doc = "Field `PTTO22` writer - Port Toggle Output"]
pub type Ptto22W<'a, REG> = crate::BitWriter<'a, REG, Ptto22>;
impl<'a, REG> Ptto22W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto22::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto22::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto23 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto23> for bool {
    #[inline(always)]
    fn from(variant: Ptto23) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO23` reader - Port Toggle Output"]
pub type Ptto23R = crate::BitReader<Ptto23>;
impl Ptto23R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto23 {
        match self.bits {
            false => Ptto23::Ptto0,
            true => Ptto23::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto23::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto23::Ptto1
    }
}
#[doc = "Field `PTTO23` writer - Port Toggle Output"]
pub type Ptto23W<'a, REG> = crate::BitWriter<'a, REG, Ptto23>;
impl<'a, REG> Ptto23W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto23::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto23::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto24 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto24> for bool {
    #[inline(always)]
    fn from(variant: Ptto24) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO24` reader - Port Toggle Output"]
pub type Ptto24R = crate::BitReader<Ptto24>;
impl Ptto24R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto24 {
        match self.bits {
            false => Ptto24::Ptto0,
            true => Ptto24::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto24::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto24::Ptto1
    }
}
#[doc = "Field `PTTO24` writer - Port Toggle Output"]
pub type Ptto24W<'a, REG> = crate::BitWriter<'a, REG, Ptto24>;
impl<'a, REG> Ptto24W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto24::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto24::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto25 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto25> for bool {
    #[inline(always)]
    fn from(variant: Ptto25) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO25` reader - Port Toggle Output"]
pub type Ptto25R = crate::BitReader<Ptto25>;
impl Ptto25R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto25 {
        match self.bits {
            false => Ptto25::Ptto0,
            true => Ptto25::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto25::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto25::Ptto1
    }
}
#[doc = "Field `PTTO25` writer - Port Toggle Output"]
pub type Ptto25W<'a, REG> = crate::BitWriter<'a, REG, Ptto25>;
impl<'a, REG> Ptto25W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto25::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto25::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto26 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto26> for bool {
    #[inline(always)]
    fn from(variant: Ptto26) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO26` reader - Port Toggle Output"]
pub type Ptto26R = crate::BitReader<Ptto26>;
impl Ptto26R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto26 {
        match self.bits {
            false => Ptto26::Ptto0,
            true => Ptto26::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto26::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto26::Ptto1
    }
}
#[doc = "Field `PTTO26` writer - Port Toggle Output"]
pub type Ptto26W<'a, REG> = crate::BitWriter<'a, REG, Ptto26>;
impl<'a, REG> Ptto26W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto26::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto26::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto27 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto27> for bool {
    #[inline(always)]
    fn from(variant: Ptto27) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO27` reader - Port Toggle Output"]
pub type Ptto27R = crate::BitReader<Ptto27>;
impl Ptto27R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto27 {
        match self.bits {
            false => Ptto27::Ptto0,
            true => Ptto27::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto27::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto27::Ptto1
    }
}
#[doc = "Field `PTTO27` writer - Port Toggle Output"]
pub type Ptto27W<'a, REG> = crate::BitWriter<'a, REG, Ptto27>;
impl<'a, REG> Ptto27W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto27::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto27::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto28 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto28> for bool {
    #[inline(always)]
    fn from(variant: Ptto28) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO28` reader - Port Toggle Output"]
pub type Ptto28R = crate::BitReader<Ptto28>;
impl Ptto28R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto28 {
        match self.bits {
            false => Ptto28::Ptto0,
            true => Ptto28::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto28::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto28::Ptto1
    }
}
#[doc = "Field `PTTO28` writer - Port Toggle Output"]
pub type Ptto28W<'a, REG> = crate::BitWriter<'a, REG, Ptto28>;
impl<'a, REG> Ptto28W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto28::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto28::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto29 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto29> for bool {
    #[inline(always)]
    fn from(variant: Ptto29) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO29` reader - Port Toggle Output"]
pub type Ptto29R = crate::BitReader<Ptto29>;
impl Ptto29R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto29 {
        match self.bits {
            false => Ptto29::Ptto0,
            true => Ptto29::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto29::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto29::Ptto1
    }
}
#[doc = "Field `PTTO29` writer - Port Toggle Output"]
pub type Ptto29W<'a, REG> = crate::BitWriter<'a, REG, Ptto29>;
impl<'a, REG> Ptto29W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto29::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto29::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto30 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto30> for bool {
    #[inline(always)]
    fn from(variant: Ptto30) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO30` reader - Port Toggle Output"]
pub type Ptto30R = crate::BitReader<Ptto30>;
impl Ptto30R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto30 {
        match self.bits {
            false => Ptto30::Ptto0,
            true => Ptto30::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto30::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto30::Ptto1
    }
}
#[doc = "Field `PTTO30` writer - Port Toggle Output"]
pub type Ptto30W<'a, REG> = crate::BitWriter<'a, REG, Ptto30>;
impl<'a, REG> Ptto30W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto30::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto30::Ptto1)
    }
}
#[doc = "Port Toggle Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptto31 {
    #[doc = "0: No change"]
    Ptto0 = 0,
    #[doc = "1: Set to the inverse of its current logic state"]
    Ptto1 = 1,
}
impl From<Ptto31> for bool {
    #[inline(always)]
    fn from(variant: Ptto31) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTTO31` reader - Port Toggle Output"]
pub type Ptto31R = crate::BitReader<Ptto31>;
impl Ptto31R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptto31 {
        match self.bits {
            false => Ptto31::Ptto0,
            true => Ptto31::Ptto1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptto0(&self) -> bool {
        *self == Ptto31::Ptto0
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn is_ptto1(&self) -> bool {
        *self == Ptto31::Ptto1
    }
}
#[doc = "Field `PTTO31` writer - Port Toggle Output"]
pub type Ptto31W<'a, REG> = crate::BitWriter<'a, REG, Ptto31>;
impl<'a, REG> Ptto31W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptto0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto31::Ptto0)
    }
    #[doc = "Set to the inverse of its current logic state"]
    #[inline(always)]
    pub fn ptto1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptto31::Ptto1)
    }
}
impl R {
    #[doc = "Bit 0 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto0(&self) -> Ptto0R {
        Ptto0R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto1(&self) -> Ptto1R {
        Ptto1R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto2(&self) -> Ptto2R {
        Ptto2R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto3(&self) -> Ptto3R {
        Ptto3R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto4(&self) -> Ptto4R {
        Ptto4R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto5(&self) -> Ptto5R {
        Ptto5R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto6(&self) -> Ptto6R {
        Ptto6R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto7(&self) -> Ptto7R {
        Ptto7R::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto8(&self) -> Ptto8R {
        Ptto8R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto9(&self) -> Ptto9R {
        Ptto9R::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto10(&self) -> Ptto10R {
        Ptto10R::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto11(&self) -> Ptto11R {
        Ptto11R::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto12(&self) -> Ptto12R {
        Ptto12R::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto13(&self) -> Ptto13R {
        Ptto13R::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto14(&self) -> Ptto14R {
        Ptto14R::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto15(&self) -> Ptto15R {
        Ptto15R::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto16(&self) -> Ptto16R {
        Ptto16R::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto17(&self) -> Ptto17R {
        Ptto17R::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto18(&self) -> Ptto18R {
        Ptto18R::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto19(&self) -> Ptto19R {
        Ptto19R::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto20(&self) -> Ptto20R {
        Ptto20R::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto21(&self) -> Ptto21R {
        Ptto21R::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto22(&self) -> Ptto22R {
        Ptto22R::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto23(&self) -> Ptto23R {
        Ptto23R::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bit 24 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto24(&self) -> Ptto24R {
        Ptto24R::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto25(&self) -> Ptto25R {
        Ptto25R::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto26(&self) -> Ptto26R {
        Ptto26R::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto27(&self) -> Ptto27R {
        Ptto27R::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bit 28 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto28(&self) -> Ptto28R {
        Ptto28R::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 29 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto29(&self) -> Ptto29R {
        Ptto29R::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto30(&self) -> Ptto30R {
        Ptto30R::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto31(&self) -> Ptto31R {
        Ptto31R::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto0(&mut self) -> Ptto0W<PtorSpec> {
        Ptto0W::new(self, 0)
    }
    #[doc = "Bit 1 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto1(&mut self) -> Ptto1W<PtorSpec> {
        Ptto1W::new(self, 1)
    }
    #[doc = "Bit 2 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto2(&mut self) -> Ptto2W<PtorSpec> {
        Ptto2W::new(self, 2)
    }
    #[doc = "Bit 3 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto3(&mut self) -> Ptto3W<PtorSpec> {
        Ptto3W::new(self, 3)
    }
    #[doc = "Bit 4 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto4(&mut self) -> Ptto4W<PtorSpec> {
        Ptto4W::new(self, 4)
    }
    #[doc = "Bit 5 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto5(&mut self) -> Ptto5W<PtorSpec> {
        Ptto5W::new(self, 5)
    }
    #[doc = "Bit 6 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto6(&mut self) -> Ptto6W<PtorSpec> {
        Ptto6W::new(self, 6)
    }
    #[doc = "Bit 7 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto7(&mut self) -> Ptto7W<PtorSpec> {
        Ptto7W::new(self, 7)
    }
    #[doc = "Bit 8 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto8(&mut self) -> Ptto8W<PtorSpec> {
        Ptto8W::new(self, 8)
    }
    #[doc = "Bit 9 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto9(&mut self) -> Ptto9W<PtorSpec> {
        Ptto9W::new(self, 9)
    }
    #[doc = "Bit 10 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto10(&mut self) -> Ptto10W<PtorSpec> {
        Ptto10W::new(self, 10)
    }
    #[doc = "Bit 11 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto11(&mut self) -> Ptto11W<PtorSpec> {
        Ptto11W::new(self, 11)
    }
    #[doc = "Bit 12 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto12(&mut self) -> Ptto12W<PtorSpec> {
        Ptto12W::new(self, 12)
    }
    #[doc = "Bit 13 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto13(&mut self) -> Ptto13W<PtorSpec> {
        Ptto13W::new(self, 13)
    }
    #[doc = "Bit 14 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto14(&mut self) -> Ptto14W<PtorSpec> {
        Ptto14W::new(self, 14)
    }
    #[doc = "Bit 15 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto15(&mut self) -> Ptto15W<PtorSpec> {
        Ptto15W::new(self, 15)
    }
    #[doc = "Bit 16 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto16(&mut self) -> Ptto16W<PtorSpec> {
        Ptto16W::new(self, 16)
    }
    #[doc = "Bit 17 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto17(&mut self) -> Ptto17W<PtorSpec> {
        Ptto17W::new(self, 17)
    }
    #[doc = "Bit 18 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto18(&mut self) -> Ptto18W<PtorSpec> {
        Ptto18W::new(self, 18)
    }
    #[doc = "Bit 19 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto19(&mut self) -> Ptto19W<PtorSpec> {
        Ptto19W::new(self, 19)
    }
    #[doc = "Bit 20 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto20(&mut self) -> Ptto20W<PtorSpec> {
        Ptto20W::new(self, 20)
    }
    #[doc = "Bit 21 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto21(&mut self) -> Ptto21W<PtorSpec> {
        Ptto21W::new(self, 21)
    }
    #[doc = "Bit 22 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto22(&mut self) -> Ptto22W<PtorSpec> {
        Ptto22W::new(self, 22)
    }
    #[doc = "Bit 23 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto23(&mut self) -> Ptto23W<PtorSpec> {
        Ptto23W::new(self, 23)
    }
    #[doc = "Bit 24 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto24(&mut self) -> Ptto24W<PtorSpec> {
        Ptto24W::new(self, 24)
    }
    #[doc = "Bit 25 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto25(&mut self) -> Ptto25W<PtorSpec> {
        Ptto25W::new(self, 25)
    }
    #[doc = "Bit 26 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto26(&mut self) -> Ptto26W<PtorSpec> {
        Ptto26W::new(self, 26)
    }
    #[doc = "Bit 27 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto27(&mut self) -> Ptto27W<PtorSpec> {
        Ptto27W::new(self, 27)
    }
    #[doc = "Bit 28 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto28(&mut self) -> Ptto28W<PtorSpec> {
        Ptto28W::new(self, 28)
    }
    #[doc = "Bit 29 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto29(&mut self) -> Ptto29W<PtorSpec> {
        Ptto29W::new(self, 29)
    }
    #[doc = "Bit 30 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto30(&mut self) -> Ptto30W<PtorSpec> {
        Ptto30W::new(self, 30)
    }
    #[doc = "Bit 31 - Port Toggle Output"]
    #[inline(always)]
    pub fn ptto31(&mut self) -> Ptto31W<PtorSpec> {
        Ptto31W::new(self, 31)
    }
}
#[doc = "Port Toggle Output\n\nYou can [`read`](crate::Reg::read) this register and get [`ptor::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ptor::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PtorSpec;
impl crate::RegisterSpec for PtorSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ptor::R`](R) reader structure"]
impl crate::Readable for PtorSpec {}
#[doc = "`write(|w| ..)` method takes [`ptor::W`](W) writer structure"]
impl crate::Writable for PtorSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PTOR to value 0"]
impl crate::Resettable for PtorSpec {}

#[doc = "Register `PSOR` reader"]
pub type R = crate::R<PsorSpec>;
#[doc = "Register `PSOR` writer"]
pub type W = crate::W<PsorSpec>;
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso0 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso0> for bool {
    #[inline(always)]
    fn from(variant: Ptso0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO0` reader - Port Set Output"]
pub type Ptso0R = crate::BitReader<Ptso0>;
impl Ptso0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso0 {
        match self.bits {
            false => Ptso0::Ptso0,
            true => Ptso0::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso0::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso0::Ptso1
    }
}
#[doc = "Field `PTSO0` writer - Port Set Output"]
pub type Ptso0W<'a, REG> = crate::BitWriter<'a, REG, Ptso0>;
impl<'a, REG> Ptso0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso0::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso0::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso1 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso1> for bool {
    #[inline(always)]
    fn from(variant: Ptso1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO1` reader - Port Set Output"]
pub type Ptso1R = crate::BitReader<Ptso1>;
impl Ptso1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso1 {
        match self.bits {
            false => Ptso1::Ptso0,
            true => Ptso1::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso1::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso1::Ptso1
    }
}
#[doc = "Field `PTSO1` writer - Port Set Output"]
pub type Ptso1W<'a, REG> = crate::BitWriter<'a, REG, Ptso1>;
impl<'a, REG> Ptso1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso1::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso1::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso2 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso2> for bool {
    #[inline(always)]
    fn from(variant: Ptso2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO2` reader - Port Set Output"]
pub type Ptso2R = crate::BitReader<Ptso2>;
impl Ptso2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso2 {
        match self.bits {
            false => Ptso2::Ptso0,
            true => Ptso2::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso2::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso2::Ptso1
    }
}
#[doc = "Field `PTSO2` writer - Port Set Output"]
pub type Ptso2W<'a, REG> = crate::BitWriter<'a, REG, Ptso2>;
impl<'a, REG> Ptso2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso2::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso2::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso3 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso3> for bool {
    #[inline(always)]
    fn from(variant: Ptso3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO3` reader - Port Set Output"]
pub type Ptso3R = crate::BitReader<Ptso3>;
impl Ptso3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso3 {
        match self.bits {
            false => Ptso3::Ptso0,
            true => Ptso3::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso3::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso3::Ptso1
    }
}
#[doc = "Field `PTSO3` writer - Port Set Output"]
pub type Ptso3W<'a, REG> = crate::BitWriter<'a, REG, Ptso3>;
impl<'a, REG> Ptso3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso3::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso3::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso4 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso4> for bool {
    #[inline(always)]
    fn from(variant: Ptso4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO4` reader - Port Set Output"]
pub type Ptso4R = crate::BitReader<Ptso4>;
impl Ptso4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso4 {
        match self.bits {
            false => Ptso4::Ptso0,
            true => Ptso4::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso4::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso4::Ptso1
    }
}
#[doc = "Field `PTSO4` writer - Port Set Output"]
pub type Ptso4W<'a, REG> = crate::BitWriter<'a, REG, Ptso4>;
impl<'a, REG> Ptso4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso4::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso4::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso5 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso5> for bool {
    #[inline(always)]
    fn from(variant: Ptso5) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO5` reader - Port Set Output"]
pub type Ptso5R = crate::BitReader<Ptso5>;
impl Ptso5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso5 {
        match self.bits {
            false => Ptso5::Ptso0,
            true => Ptso5::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso5::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso5::Ptso1
    }
}
#[doc = "Field `PTSO5` writer - Port Set Output"]
pub type Ptso5W<'a, REG> = crate::BitWriter<'a, REG, Ptso5>;
impl<'a, REG> Ptso5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso5::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso5::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso6 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso6> for bool {
    #[inline(always)]
    fn from(variant: Ptso6) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO6` reader - Port Set Output"]
pub type Ptso6R = crate::BitReader<Ptso6>;
impl Ptso6R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso6 {
        match self.bits {
            false => Ptso6::Ptso0,
            true => Ptso6::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso6::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso6::Ptso1
    }
}
#[doc = "Field `PTSO6` writer - Port Set Output"]
pub type Ptso6W<'a, REG> = crate::BitWriter<'a, REG, Ptso6>;
impl<'a, REG> Ptso6W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso6::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso6::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso7 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso7> for bool {
    #[inline(always)]
    fn from(variant: Ptso7) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO7` reader - Port Set Output"]
pub type Ptso7R = crate::BitReader<Ptso7>;
impl Ptso7R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso7 {
        match self.bits {
            false => Ptso7::Ptso0,
            true => Ptso7::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso7::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso7::Ptso1
    }
}
#[doc = "Field `PTSO7` writer - Port Set Output"]
pub type Ptso7W<'a, REG> = crate::BitWriter<'a, REG, Ptso7>;
impl<'a, REG> Ptso7W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso7::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso7::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso8 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso8> for bool {
    #[inline(always)]
    fn from(variant: Ptso8) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO8` reader - Port Set Output"]
pub type Ptso8R = crate::BitReader<Ptso8>;
impl Ptso8R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso8 {
        match self.bits {
            false => Ptso8::Ptso0,
            true => Ptso8::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso8::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso8::Ptso1
    }
}
#[doc = "Field `PTSO8` writer - Port Set Output"]
pub type Ptso8W<'a, REG> = crate::BitWriter<'a, REG, Ptso8>;
impl<'a, REG> Ptso8W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso8::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso8::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso9 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso9> for bool {
    #[inline(always)]
    fn from(variant: Ptso9) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO9` reader - Port Set Output"]
pub type Ptso9R = crate::BitReader<Ptso9>;
impl Ptso9R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso9 {
        match self.bits {
            false => Ptso9::Ptso0,
            true => Ptso9::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso9::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso9::Ptso1
    }
}
#[doc = "Field `PTSO9` writer - Port Set Output"]
pub type Ptso9W<'a, REG> = crate::BitWriter<'a, REG, Ptso9>;
impl<'a, REG> Ptso9W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso9::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso9::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso10 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso10> for bool {
    #[inline(always)]
    fn from(variant: Ptso10) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO10` reader - Port Set Output"]
pub type Ptso10R = crate::BitReader<Ptso10>;
impl Ptso10R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso10 {
        match self.bits {
            false => Ptso10::Ptso0,
            true => Ptso10::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso10::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso10::Ptso1
    }
}
#[doc = "Field `PTSO10` writer - Port Set Output"]
pub type Ptso10W<'a, REG> = crate::BitWriter<'a, REG, Ptso10>;
impl<'a, REG> Ptso10W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso10::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso10::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso11 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso11> for bool {
    #[inline(always)]
    fn from(variant: Ptso11) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO11` reader - Port Set Output"]
pub type Ptso11R = crate::BitReader<Ptso11>;
impl Ptso11R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso11 {
        match self.bits {
            false => Ptso11::Ptso0,
            true => Ptso11::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso11::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso11::Ptso1
    }
}
#[doc = "Field `PTSO11` writer - Port Set Output"]
pub type Ptso11W<'a, REG> = crate::BitWriter<'a, REG, Ptso11>;
impl<'a, REG> Ptso11W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso11::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso11::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso12 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso12> for bool {
    #[inline(always)]
    fn from(variant: Ptso12) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO12` reader - Port Set Output"]
pub type Ptso12R = crate::BitReader<Ptso12>;
impl Ptso12R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso12 {
        match self.bits {
            false => Ptso12::Ptso0,
            true => Ptso12::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso12::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso12::Ptso1
    }
}
#[doc = "Field `PTSO12` writer - Port Set Output"]
pub type Ptso12W<'a, REG> = crate::BitWriter<'a, REG, Ptso12>;
impl<'a, REG> Ptso12W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso12::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso12::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso13 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso13> for bool {
    #[inline(always)]
    fn from(variant: Ptso13) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO13` reader - Port Set Output"]
pub type Ptso13R = crate::BitReader<Ptso13>;
impl Ptso13R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso13 {
        match self.bits {
            false => Ptso13::Ptso0,
            true => Ptso13::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso13::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso13::Ptso1
    }
}
#[doc = "Field `PTSO13` writer - Port Set Output"]
pub type Ptso13W<'a, REG> = crate::BitWriter<'a, REG, Ptso13>;
impl<'a, REG> Ptso13W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso13::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso13::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso14 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso14> for bool {
    #[inline(always)]
    fn from(variant: Ptso14) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO14` reader - Port Set Output"]
pub type Ptso14R = crate::BitReader<Ptso14>;
impl Ptso14R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso14 {
        match self.bits {
            false => Ptso14::Ptso0,
            true => Ptso14::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso14::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso14::Ptso1
    }
}
#[doc = "Field `PTSO14` writer - Port Set Output"]
pub type Ptso14W<'a, REG> = crate::BitWriter<'a, REG, Ptso14>;
impl<'a, REG> Ptso14W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso14::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso14::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso15 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso15> for bool {
    #[inline(always)]
    fn from(variant: Ptso15) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO15` reader - Port Set Output"]
pub type Ptso15R = crate::BitReader<Ptso15>;
impl Ptso15R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso15 {
        match self.bits {
            false => Ptso15::Ptso0,
            true => Ptso15::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso15::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso15::Ptso1
    }
}
#[doc = "Field `PTSO15` writer - Port Set Output"]
pub type Ptso15W<'a, REG> = crate::BitWriter<'a, REG, Ptso15>;
impl<'a, REG> Ptso15W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso15::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso15::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso16 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso16> for bool {
    #[inline(always)]
    fn from(variant: Ptso16) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO16` reader - Port Set Output"]
pub type Ptso16R = crate::BitReader<Ptso16>;
impl Ptso16R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso16 {
        match self.bits {
            false => Ptso16::Ptso0,
            true => Ptso16::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso16::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso16::Ptso1
    }
}
#[doc = "Field `PTSO16` writer - Port Set Output"]
pub type Ptso16W<'a, REG> = crate::BitWriter<'a, REG, Ptso16>;
impl<'a, REG> Ptso16W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso16::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso16::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso17 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso17> for bool {
    #[inline(always)]
    fn from(variant: Ptso17) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO17` reader - Port Set Output"]
pub type Ptso17R = crate::BitReader<Ptso17>;
impl Ptso17R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso17 {
        match self.bits {
            false => Ptso17::Ptso0,
            true => Ptso17::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso17::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso17::Ptso1
    }
}
#[doc = "Field `PTSO17` writer - Port Set Output"]
pub type Ptso17W<'a, REG> = crate::BitWriter<'a, REG, Ptso17>;
impl<'a, REG> Ptso17W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso17::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso17::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso18 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso18> for bool {
    #[inline(always)]
    fn from(variant: Ptso18) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO18` reader - Port Set Output"]
pub type Ptso18R = crate::BitReader<Ptso18>;
impl Ptso18R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso18 {
        match self.bits {
            false => Ptso18::Ptso0,
            true => Ptso18::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso18::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso18::Ptso1
    }
}
#[doc = "Field `PTSO18` writer - Port Set Output"]
pub type Ptso18W<'a, REG> = crate::BitWriter<'a, REG, Ptso18>;
impl<'a, REG> Ptso18W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso18::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso18::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso19 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso19> for bool {
    #[inline(always)]
    fn from(variant: Ptso19) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO19` reader - Port Set Output"]
pub type Ptso19R = crate::BitReader<Ptso19>;
impl Ptso19R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso19 {
        match self.bits {
            false => Ptso19::Ptso0,
            true => Ptso19::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso19::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso19::Ptso1
    }
}
#[doc = "Field `PTSO19` writer - Port Set Output"]
pub type Ptso19W<'a, REG> = crate::BitWriter<'a, REG, Ptso19>;
impl<'a, REG> Ptso19W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso19::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso19::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso20 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso20> for bool {
    #[inline(always)]
    fn from(variant: Ptso20) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO20` reader - Port Set Output"]
pub type Ptso20R = crate::BitReader<Ptso20>;
impl Ptso20R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso20 {
        match self.bits {
            false => Ptso20::Ptso0,
            true => Ptso20::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso20::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso20::Ptso1
    }
}
#[doc = "Field `PTSO20` writer - Port Set Output"]
pub type Ptso20W<'a, REG> = crate::BitWriter<'a, REG, Ptso20>;
impl<'a, REG> Ptso20W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso20::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso20::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso21 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso21> for bool {
    #[inline(always)]
    fn from(variant: Ptso21) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO21` reader - Port Set Output"]
pub type Ptso21R = crate::BitReader<Ptso21>;
impl Ptso21R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso21 {
        match self.bits {
            false => Ptso21::Ptso0,
            true => Ptso21::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso21::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso21::Ptso1
    }
}
#[doc = "Field `PTSO21` writer - Port Set Output"]
pub type Ptso21W<'a, REG> = crate::BitWriter<'a, REG, Ptso21>;
impl<'a, REG> Ptso21W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso21::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso21::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso22 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso22> for bool {
    #[inline(always)]
    fn from(variant: Ptso22) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO22` reader - Port Set Output"]
pub type Ptso22R = crate::BitReader<Ptso22>;
impl Ptso22R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso22 {
        match self.bits {
            false => Ptso22::Ptso0,
            true => Ptso22::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso22::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso22::Ptso1
    }
}
#[doc = "Field `PTSO22` writer - Port Set Output"]
pub type Ptso22W<'a, REG> = crate::BitWriter<'a, REG, Ptso22>;
impl<'a, REG> Ptso22W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso22::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso22::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso23 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso23> for bool {
    #[inline(always)]
    fn from(variant: Ptso23) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO23` reader - Port Set Output"]
pub type Ptso23R = crate::BitReader<Ptso23>;
impl Ptso23R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso23 {
        match self.bits {
            false => Ptso23::Ptso0,
            true => Ptso23::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso23::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso23::Ptso1
    }
}
#[doc = "Field `PTSO23` writer - Port Set Output"]
pub type Ptso23W<'a, REG> = crate::BitWriter<'a, REG, Ptso23>;
impl<'a, REG> Ptso23W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso23::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso23::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso24 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso24> for bool {
    #[inline(always)]
    fn from(variant: Ptso24) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO24` reader - Port Set Output"]
pub type Ptso24R = crate::BitReader<Ptso24>;
impl Ptso24R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso24 {
        match self.bits {
            false => Ptso24::Ptso0,
            true => Ptso24::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso24::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso24::Ptso1
    }
}
#[doc = "Field `PTSO24` writer - Port Set Output"]
pub type Ptso24W<'a, REG> = crate::BitWriter<'a, REG, Ptso24>;
impl<'a, REG> Ptso24W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso24::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso24::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso25 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso25> for bool {
    #[inline(always)]
    fn from(variant: Ptso25) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO25` reader - Port Set Output"]
pub type Ptso25R = crate::BitReader<Ptso25>;
impl Ptso25R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso25 {
        match self.bits {
            false => Ptso25::Ptso0,
            true => Ptso25::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso25::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso25::Ptso1
    }
}
#[doc = "Field `PTSO25` writer - Port Set Output"]
pub type Ptso25W<'a, REG> = crate::BitWriter<'a, REG, Ptso25>;
impl<'a, REG> Ptso25W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso25::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso25::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso26 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso26> for bool {
    #[inline(always)]
    fn from(variant: Ptso26) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO26` reader - Port Set Output"]
pub type Ptso26R = crate::BitReader<Ptso26>;
impl Ptso26R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso26 {
        match self.bits {
            false => Ptso26::Ptso0,
            true => Ptso26::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso26::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso26::Ptso1
    }
}
#[doc = "Field `PTSO26` writer - Port Set Output"]
pub type Ptso26W<'a, REG> = crate::BitWriter<'a, REG, Ptso26>;
impl<'a, REG> Ptso26W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso26::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso26::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso27 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso27> for bool {
    #[inline(always)]
    fn from(variant: Ptso27) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO27` reader - Port Set Output"]
pub type Ptso27R = crate::BitReader<Ptso27>;
impl Ptso27R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso27 {
        match self.bits {
            false => Ptso27::Ptso0,
            true => Ptso27::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso27::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso27::Ptso1
    }
}
#[doc = "Field `PTSO27` writer - Port Set Output"]
pub type Ptso27W<'a, REG> = crate::BitWriter<'a, REG, Ptso27>;
impl<'a, REG> Ptso27W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso27::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso27::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso28 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso28> for bool {
    #[inline(always)]
    fn from(variant: Ptso28) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO28` reader - Port Set Output"]
pub type Ptso28R = crate::BitReader<Ptso28>;
impl Ptso28R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso28 {
        match self.bits {
            false => Ptso28::Ptso0,
            true => Ptso28::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso28::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso28::Ptso1
    }
}
#[doc = "Field `PTSO28` writer - Port Set Output"]
pub type Ptso28W<'a, REG> = crate::BitWriter<'a, REG, Ptso28>;
impl<'a, REG> Ptso28W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso28::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso28::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso29 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso29> for bool {
    #[inline(always)]
    fn from(variant: Ptso29) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO29` reader - Port Set Output"]
pub type Ptso29R = crate::BitReader<Ptso29>;
impl Ptso29R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso29 {
        match self.bits {
            false => Ptso29::Ptso0,
            true => Ptso29::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso29::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso29::Ptso1
    }
}
#[doc = "Field `PTSO29` writer - Port Set Output"]
pub type Ptso29W<'a, REG> = crate::BitWriter<'a, REG, Ptso29>;
impl<'a, REG> Ptso29W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso29::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso29::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso30 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso30> for bool {
    #[inline(always)]
    fn from(variant: Ptso30) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO30` reader - Port Set Output"]
pub type Ptso30R = crate::BitReader<Ptso30>;
impl Ptso30R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso30 {
        match self.bits {
            false => Ptso30::Ptso0,
            true => Ptso30::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso30::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso30::Ptso1
    }
}
#[doc = "Field `PTSO30` writer - Port Set Output"]
pub type Ptso30W<'a, REG> = crate::BitWriter<'a, REG, Ptso30>;
impl<'a, REG> Ptso30W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso30::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso30::Ptso1)
    }
}
#[doc = "Port Set Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptso31 {
    #[doc = "0: No change"]
    Ptso0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 1"]
    Ptso1 = 1,
}
impl From<Ptso31> for bool {
    #[inline(always)]
    fn from(variant: Ptso31) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTSO31` reader - Port Set Output"]
pub type Ptso31R = crate::BitReader<Ptso31>;
impl Ptso31R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptso31 {
        match self.bits {
            false => Ptso31::Ptso0,
            true => Ptso31::Ptso1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptso0(&self) -> bool {
        *self == Ptso31::Ptso0
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn is_ptso1(&self) -> bool {
        *self == Ptso31::Ptso1
    }
}
#[doc = "Field `PTSO31` writer - Port Set Output"]
pub type Ptso31W<'a, REG> = crate::BitWriter<'a, REG, Ptso31>;
impl<'a, REG> Ptso31W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptso0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso31::Ptso0)
    }
    #[doc = "Corresponding field in PDOR becomes 1"]
    #[inline(always)]
    pub fn ptso1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptso31::Ptso1)
    }
}
impl R {
    #[doc = "Bit 0 - Port Set Output"]
    #[inline(always)]
    pub fn ptso0(&self) -> Ptso0R {
        Ptso0R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Port Set Output"]
    #[inline(always)]
    pub fn ptso1(&self) -> Ptso1R {
        Ptso1R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Port Set Output"]
    #[inline(always)]
    pub fn ptso2(&self) -> Ptso2R {
        Ptso2R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Port Set Output"]
    #[inline(always)]
    pub fn ptso3(&self) -> Ptso3R {
        Ptso3R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Port Set Output"]
    #[inline(always)]
    pub fn ptso4(&self) -> Ptso4R {
        Ptso4R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Port Set Output"]
    #[inline(always)]
    pub fn ptso5(&self) -> Ptso5R {
        Ptso5R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Port Set Output"]
    #[inline(always)]
    pub fn ptso6(&self) -> Ptso6R {
        Ptso6R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Port Set Output"]
    #[inline(always)]
    pub fn ptso7(&self) -> Ptso7R {
        Ptso7R::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Port Set Output"]
    #[inline(always)]
    pub fn ptso8(&self) -> Ptso8R {
        Ptso8R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Port Set Output"]
    #[inline(always)]
    pub fn ptso9(&self) -> Ptso9R {
        Ptso9R::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Port Set Output"]
    #[inline(always)]
    pub fn ptso10(&self) -> Ptso10R {
        Ptso10R::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Port Set Output"]
    #[inline(always)]
    pub fn ptso11(&self) -> Ptso11R {
        Ptso11R::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Port Set Output"]
    #[inline(always)]
    pub fn ptso12(&self) -> Ptso12R {
        Ptso12R::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Port Set Output"]
    #[inline(always)]
    pub fn ptso13(&self) -> Ptso13R {
        Ptso13R::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Port Set Output"]
    #[inline(always)]
    pub fn ptso14(&self) -> Ptso14R {
        Ptso14R::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Port Set Output"]
    #[inline(always)]
    pub fn ptso15(&self) -> Ptso15R {
        Ptso15R::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - Port Set Output"]
    #[inline(always)]
    pub fn ptso16(&self) -> Ptso16R {
        Ptso16R::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Port Set Output"]
    #[inline(always)]
    pub fn ptso17(&self) -> Ptso17R {
        Ptso17R::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Port Set Output"]
    #[inline(always)]
    pub fn ptso18(&self) -> Ptso18R {
        Ptso18R::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - Port Set Output"]
    #[inline(always)]
    pub fn ptso19(&self) -> Ptso19R {
        Ptso19R::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - Port Set Output"]
    #[inline(always)]
    pub fn ptso20(&self) -> Ptso20R {
        Ptso20R::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - Port Set Output"]
    #[inline(always)]
    pub fn ptso21(&self) -> Ptso21R {
        Ptso21R::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - Port Set Output"]
    #[inline(always)]
    pub fn ptso22(&self) -> Ptso22R {
        Ptso22R::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - Port Set Output"]
    #[inline(always)]
    pub fn ptso23(&self) -> Ptso23R {
        Ptso23R::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bit 24 - Port Set Output"]
    #[inline(always)]
    pub fn ptso24(&self) -> Ptso24R {
        Ptso24R::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - Port Set Output"]
    #[inline(always)]
    pub fn ptso25(&self) -> Ptso25R {
        Ptso25R::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - Port Set Output"]
    #[inline(always)]
    pub fn ptso26(&self) -> Ptso26R {
        Ptso26R::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - Port Set Output"]
    #[inline(always)]
    pub fn ptso27(&self) -> Ptso27R {
        Ptso27R::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bit 28 - Port Set Output"]
    #[inline(always)]
    pub fn ptso28(&self) -> Ptso28R {
        Ptso28R::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 29 - Port Set Output"]
    #[inline(always)]
    pub fn ptso29(&self) -> Ptso29R {
        Ptso29R::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - Port Set Output"]
    #[inline(always)]
    pub fn ptso30(&self) -> Ptso30R {
        Ptso30R::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Port Set Output"]
    #[inline(always)]
    pub fn ptso31(&self) -> Ptso31R {
        Ptso31R::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Port Set Output"]
    #[inline(always)]
    pub fn ptso0(&mut self) -> Ptso0W<PsorSpec> {
        Ptso0W::new(self, 0)
    }
    #[doc = "Bit 1 - Port Set Output"]
    #[inline(always)]
    pub fn ptso1(&mut self) -> Ptso1W<PsorSpec> {
        Ptso1W::new(self, 1)
    }
    #[doc = "Bit 2 - Port Set Output"]
    #[inline(always)]
    pub fn ptso2(&mut self) -> Ptso2W<PsorSpec> {
        Ptso2W::new(self, 2)
    }
    #[doc = "Bit 3 - Port Set Output"]
    #[inline(always)]
    pub fn ptso3(&mut self) -> Ptso3W<PsorSpec> {
        Ptso3W::new(self, 3)
    }
    #[doc = "Bit 4 - Port Set Output"]
    #[inline(always)]
    pub fn ptso4(&mut self) -> Ptso4W<PsorSpec> {
        Ptso4W::new(self, 4)
    }
    #[doc = "Bit 5 - Port Set Output"]
    #[inline(always)]
    pub fn ptso5(&mut self) -> Ptso5W<PsorSpec> {
        Ptso5W::new(self, 5)
    }
    #[doc = "Bit 6 - Port Set Output"]
    #[inline(always)]
    pub fn ptso6(&mut self) -> Ptso6W<PsorSpec> {
        Ptso6W::new(self, 6)
    }
    #[doc = "Bit 7 - Port Set Output"]
    #[inline(always)]
    pub fn ptso7(&mut self) -> Ptso7W<PsorSpec> {
        Ptso7W::new(self, 7)
    }
    #[doc = "Bit 8 - Port Set Output"]
    #[inline(always)]
    pub fn ptso8(&mut self) -> Ptso8W<PsorSpec> {
        Ptso8W::new(self, 8)
    }
    #[doc = "Bit 9 - Port Set Output"]
    #[inline(always)]
    pub fn ptso9(&mut self) -> Ptso9W<PsorSpec> {
        Ptso9W::new(self, 9)
    }
    #[doc = "Bit 10 - Port Set Output"]
    #[inline(always)]
    pub fn ptso10(&mut self) -> Ptso10W<PsorSpec> {
        Ptso10W::new(self, 10)
    }
    #[doc = "Bit 11 - Port Set Output"]
    #[inline(always)]
    pub fn ptso11(&mut self) -> Ptso11W<PsorSpec> {
        Ptso11W::new(self, 11)
    }
    #[doc = "Bit 12 - Port Set Output"]
    #[inline(always)]
    pub fn ptso12(&mut self) -> Ptso12W<PsorSpec> {
        Ptso12W::new(self, 12)
    }
    #[doc = "Bit 13 - Port Set Output"]
    #[inline(always)]
    pub fn ptso13(&mut self) -> Ptso13W<PsorSpec> {
        Ptso13W::new(self, 13)
    }
    #[doc = "Bit 14 - Port Set Output"]
    #[inline(always)]
    pub fn ptso14(&mut self) -> Ptso14W<PsorSpec> {
        Ptso14W::new(self, 14)
    }
    #[doc = "Bit 15 - Port Set Output"]
    #[inline(always)]
    pub fn ptso15(&mut self) -> Ptso15W<PsorSpec> {
        Ptso15W::new(self, 15)
    }
    #[doc = "Bit 16 - Port Set Output"]
    #[inline(always)]
    pub fn ptso16(&mut self) -> Ptso16W<PsorSpec> {
        Ptso16W::new(self, 16)
    }
    #[doc = "Bit 17 - Port Set Output"]
    #[inline(always)]
    pub fn ptso17(&mut self) -> Ptso17W<PsorSpec> {
        Ptso17W::new(self, 17)
    }
    #[doc = "Bit 18 - Port Set Output"]
    #[inline(always)]
    pub fn ptso18(&mut self) -> Ptso18W<PsorSpec> {
        Ptso18W::new(self, 18)
    }
    #[doc = "Bit 19 - Port Set Output"]
    #[inline(always)]
    pub fn ptso19(&mut self) -> Ptso19W<PsorSpec> {
        Ptso19W::new(self, 19)
    }
    #[doc = "Bit 20 - Port Set Output"]
    #[inline(always)]
    pub fn ptso20(&mut self) -> Ptso20W<PsorSpec> {
        Ptso20W::new(self, 20)
    }
    #[doc = "Bit 21 - Port Set Output"]
    #[inline(always)]
    pub fn ptso21(&mut self) -> Ptso21W<PsorSpec> {
        Ptso21W::new(self, 21)
    }
    #[doc = "Bit 22 - Port Set Output"]
    #[inline(always)]
    pub fn ptso22(&mut self) -> Ptso22W<PsorSpec> {
        Ptso22W::new(self, 22)
    }
    #[doc = "Bit 23 - Port Set Output"]
    #[inline(always)]
    pub fn ptso23(&mut self) -> Ptso23W<PsorSpec> {
        Ptso23W::new(self, 23)
    }
    #[doc = "Bit 24 - Port Set Output"]
    #[inline(always)]
    pub fn ptso24(&mut self) -> Ptso24W<PsorSpec> {
        Ptso24W::new(self, 24)
    }
    #[doc = "Bit 25 - Port Set Output"]
    #[inline(always)]
    pub fn ptso25(&mut self) -> Ptso25W<PsorSpec> {
        Ptso25W::new(self, 25)
    }
    #[doc = "Bit 26 - Port Set Output"]
    #[inline(always)]
    pub fn ptso26(&mut self) -> Ptso26W<PsorSpec> {
        Ptso26W::new(self, 26)
    }
    #[doc = "Bit 27 - Port Set Output"]
    #[inline(always)]
    pub fn ptso27(&mut self) -> Ptso27W<PsorSpec> {
        Ptso27W::new(self, 27)
    }
    #[doc = "Bit 28 - Port Set Output"]
    #[inline(always)]
    pub fn ptso28(&mut self) -> Ptso28W<PsorSpec> {
        Ptso28W::new(self, 28)
    }
    #[doc = "Bit 29 - Port Set Output"]
    #[inline(always)]
    pub fn ptso29(&mut self) -> Ptso29W<PsorSpec> {
        Ptso29W::new(self, 29)
    }
    #[doc = "Bit 30 - Port Set Output"]
    #[inline(always)]
    pub fn ptso30(&mut self) -> Ptso30W<PsorSpec> {
        Ptso30W::new(self, 30)
    }
    #[doc = "Bit 31 - Port Set Output"]
    #[inline(always)]
    pub fn ptso31(&mut self) -> Ptso31W<PsorSpec> {
        Ptso31W::new(self, 31)
    }
}
#[doc = "Port Set Output\n\nYou can [`read`](crate::Reg::read) this register and get [`psor::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`psor::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PsorSpec;
impl crate::RegisterSpec for PsorSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`psor::R`](R) reader structure"]
impl crate::Readable for PsorSpec {}
#[doc = "`write(|w| ..)` method takes [`psor::W`](W) writer structure"]
impl crate::Writable for PsorSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PSOR to value 0"]
impl crate::Resettable for PsorSpec {}

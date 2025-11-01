#[doc = "Register `PDOR` reader"]
pub type R = crate::R<PdorSpec>;
#[doc = "Register `PDOR` writer"]
pub type W = crate::W<PdorSpec>;
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo0 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo0> for bool {
    #[inline(always)]
    fn from(variant: Pdo0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO0` reader - Port Data Output"]
pub type Pdo0R = crate::BitReader<Pdo0>;
impl Pdo0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo0 {
        match self.bits {
            false => Pdo0::Pdo0,
            true => Pdo0::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo0::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo0::Pdo1
    }
}
#[doc = "Field `PDO0` writer - Port Data Output"]
pub type Pdo0W<'a, REG> = crate::BitWriter<'a, REG, Pdo0>;
impl<'a, REG> Pdo0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo0::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo0::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo1 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo1> for bool {
    #[inline(always)]
    fn from(variant: Pdo1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO1` reader - Port Data Output"]
pub type Pdo1R = crate::BitReader<Pdo1>;
impl Pdo1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo1 {
        match self.bits {
            false => Pdo1::Pdo0,
            true => Pdo1::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo1::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo1::Pdo1
    }
}
#[doc = "Field `PDO1` writer - Port Data Output"]
pub type Pdo1W<'a, REG> = crate::BitWriter<'a, REG, Pdo1>;
impl<'a, REG> Pdo1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo1::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo1::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo2 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo2> for bool {
    #[inline(always)]
    fn from(variant: Pdo2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO2` reader - Port Data Output"]
pub type Pdo2R = crate::BitReader<Pdo2>;
impl Pdo2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo2 {
        match self.bits {
            false => Pdo2::Pdo0,
            true => Pdo2::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo2::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo2::Pdo1
    }
}
#[doc = "Field `PDO2` writer - Port Data Output"]
pub type Pdo2W<'a, REG> = crate::BitWriter<'a, REG, Pdo2>;
impl<'a, REG> Pdo2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo2::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo2::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo3 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo3> for bool {
    #[inline(always)]
    fn from(variant: Pdo3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO3` reader - Port Data Output"]
pub type Pdo3R = crate::BitReader<Pdo3>;
impl Pdo3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo3 {
        match self.bits {
            false => Pdo3::Pdo0,
            true => Pdo3::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo3::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo3::Pdo1
    }
}
#[doc = "Field `PDO3` writer - Port Data Output"]
pub type Pdo3W<'a, REG> = crate::BitWriter<'a, REG, Pdo3>;
impl<'a, REG> Pdo3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo3::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo3::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo4 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo4> for bool {
    #[inline(always)]
    fn from(variant: Pdo4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO4` reader - Port Data Output"]
pub type Pdo4R = crate::BitReader<Pdo4>;
impl Pdo4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo4 {
        match self.bits {
            false => Pdo4::Pdo0,
            true => Pdo4::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo4::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo4::Pdo1
    }
}
#[doc = "Field `PDO4` writer - Port Data Output"]
pub type Pdo4W<'a, REG> = crate::BitWriter<'a, REG, Pdo4>;
impl<'a, REG> Pdo4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo4::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo4::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo5 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo5> for bool {
    #[inline(always)]
    fn from(variant: Pdo5) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO5` reader - Port Data Output"]
pub type Pdo5R = crate::BitReader<Pdo5>;
impl Pdo5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo5 {
        match self.bits {
            false => Pdo5::Pdo0,
            true => Pdo5::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo5::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo5::Pdo1
    }
}
#[doc = "Field `PDO5` writer - Port Data Output"]
pub type Pdo5W<'a, REG> = crate::BitWriter<'a, REG, Pdo5>;
impl<'a, REG> Pdo5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo5::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo5::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo6 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo6> for bool {
    #[inline(always)]
    fn from(variant: Pdo6) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO6` reader - Port Data Output"]
pub type Pdo6R = crate::BitReader<Pdo6>;
impl Pdo6R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo6 {
        match self.bits {
            false => Pdo6::Pdo0,
            true => Pdo6::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo6::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo6::Pdo1
    }
}
#[doc = "Field `PDO6` writer - Port Data Output"]
pub type Pdo6W<'a, REG> = crate::BitWriter<'a, REG, Pdo6>;
impl<'a, REG> Pdo6W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo6::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo6::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo7 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo7> for bool {
    #[inline(always)]
    fn from(variant: Pdo7) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO7` reader - Port Data Output"]
pub type Pdo7R = crate::BitReader<Pdo7>;
impl Pdo7R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo7 {
        match self.bits {
            false => Pdo7::Pdo0,
            true => Pdo7::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo7::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo7::Pdo1
    }
}
#[doc = "Field `PDO7` writer - Port Data Output"]
pub type Pdo7W<'a, REG> = crate::BitWriter<'a, REG, Pdo7>;
impl<'a, REG> Pdo7W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo7::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo7::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo8 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo8> for bool {
    #[inline(always)]
    fn from(variant: Pdo8) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO8` reader - Port Data Output"]
pub type Pdo8R = crate::BitReader<Pdo8>;
impl Pdo8R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo8 {
        match self.bits {
            false => Pdo8::Pdo0,
            true => Pdo8::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo8::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo8::Pdo1
    }
}
#[doc = "Field `PDO8` writer - Port Data Output"]
pub type Pdo8W<'a, REG> = crate::BitWriter<'a, REG, Pdo8>;
impl<'a, REG> Pdo8W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo8::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo8::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo9 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo9> for bool {
    #[inline(always)]
    fn from(variant: Pdo9) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO9` reader - Port Data Output"]
pub type Pdo9R = crate::BitReader<Pdo9>;
impl Pdo9R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo9 {
        match self.bits {
            false => Pdo9::Pdo0,
            true => Pdo9::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo9::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo9::Pdo1
    }
}
#[doc = "Field `PDO9` writer - Port Data Output"]
pub type Pdo9W<'a, REG> = crate::BitWriter<'a, REG, Pdo9>;
impl<'a, REG> Pdo9W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo9::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo9::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo10 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo10> for bool {
    #[inline(always)]
    fn from(variant: Pdo10) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO10` reader - Port Data Output"]
pub type Pdo10R = crate::BitReader<Pdo10>;
impl Pdo10R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo10 {
        match self.bits {
            false => Pdo10::Pdo0,
            true => Pdo10::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo10::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo10::Pdo1
    }
}
#[doc = "Field `PDO10` writer - Port Data Output"]
pub type Pdo10W<'a, REG> = crate::BitWriter<'a, REG, Pdo10>;
impl<'a, REG> Pdo10W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo10::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo10::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo11 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo11> for bool {
    #[inline(always)]
    fn from(variant: Pdo11) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO11` reader - Port Data Output"]
pub type Pdo11R = crate::BitReader<Pdo11>;
impl Pdo11R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo11 {
        match self.bits {
            false => Pdo11::Pdo0,
            true => Pdo11::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo11::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo11::Pdo1
    }
}
#[doc = "Field `PDO11` writer - Port Data Output"]
pub type Pdo11W<'a, REG> = crate::BitWriter<'a, REG, Pdo11>;
impl<'a, REG> Pdo11W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo11::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo11::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo12 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo12> for bool {
    #[inline(always)]
    fn from(variant: Pdo12) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO12` reader - Port Data Output"]
pub type Pdo12R = crate::BitReader<Pdo12>;
impl Pdo12R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo12 {
        match self.bits {
            false => Pdo12::Pdo0,
            true => Pdo12::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo12::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo12::Pdo1
    }
}
#[doc = "Field `PDO12` writer - Port Data Output"]
pub type Pdo12W<'a, REG> = crate::BitWriter<'a, REG, Pdo12>;
impl<'a, REG> Pdo12W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo12::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo12::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo13 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo13> for bool {
    #[inline(always)]
    fn from(variant: Pdo13) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO13` reader - Port Data Output"]
pub type Pdo13R = crate::BitReader<Pdo13>;
impl Pdo13R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo13 {
        match self.bits {
            false => Pdo13::Pdo0,
            true => Pdo13::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo13::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo13::Pdo1
    }
}
#[doc = "Field `PDO13` writer - Port Data Output"]
pub type Pdo13W<'a, REG> = crate::BitWriter<'a, REG, Pdo13>;
impl<'a, REG> Pdo13W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo13::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo13::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo14 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo14> for bool {
    #[inline(always)]
    fn from(variant: Pdo14) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO14` reader - Port Data Output"]
pub type Pdo14R = crate::BitReader<Pdo14>;
impl Pdo14R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo14 {
        match self.bits {
            false => Pdo14::Pdo0,
            true => Pdo14::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo14::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo14::Pdo1
    }
}
#[doc = "Field `PDO14` writer - Port Data Output"]
pub type Pdo14W<'a, REG> = crate::BitWriter<'a, REG, Pdo14>;
impl<'a, REG> Pdo14W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo14::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo14::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo15 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo15> for bool {
    #[inline(always)]
    fn from(variant: Pdo15) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO15` reader - Port Data Output"]
pub type Pdo15R = crate::BitReader<Pdo15>;
impl Pdo15R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo15 {
        match self.bits {
            false => Pdo15::Pdo0,
            true => Pdo15::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo15::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo15::Pdo1
    }
}
#[doc = "Field `PDO15` writer - Port Data Output"]
pub type Pdo15W<'a, REG> = crate::BitWriter<'a, REG, Pdo15>;
impl<'a, REG> Pdo15W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo15::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo15::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo16 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo16> for bool {
    #[inline(always)]
    fn from(variant: Pdo16) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO16` reader - Port Data Output"]
pub type Pdo16R = crate::BitReader<Pdo16>;
impl Pdo16R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo16 {
        match self.bits {
            false => Pdo16::Pdo0,
            true => Pdo16::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo16::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo16::Pdo1
    }
}
#[doc = "Field `PDO16` writer - Port Data Output"]
pub type Pdo16W<'a, REG> = crate::BitWriter<'a, REG, Pdo16>;
impl<'a, REG> Pdo16W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo16::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo16::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo17 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo17> for bool {
    #[inline(always)]
    fn from(variant: Pdo17) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO17` reader - Port Data Output"]
pub type Pdo17R = crate::BitReader<Pdo17>;
impl Pdo17R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo17 {
        match self.bits {
            false => Pdo17::Pdo0,
            true => Pdo17::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo17::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo17::Pdo1
    }
}
#[doc = "Field `PDO17` writer - Port Data Output"]
pub type Pdo17W<'a, REG> = crate::BitWriter<'a, REG, Pdo17>;
impl<'a, REG> Pdo17W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo17::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo17::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo18 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo18> for bool {
    #[inline(always)]
    fn from(variant: Pdo18) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO18` reader - Port Data Output"]
pub type Pdo18R = crate::BitReader<Pdo18>;
impl Pdo18R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo18 {
        match self.bits {
            false => Pdo18::Pdo0,
            true => Pdo18::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo18::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo18::Pdo1
    }
}
#[doc = "Field `PDO18` writer - Port Data Output"]
pub type Pdo18W<'a, REG> = crate::BitWriter<'a, REG, Pdo18>;
impl<'a, REG> Pdo18W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo18::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo18::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo19 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo19> for bool {
    #[inline(always)]
    fn from(variant: Pdo19) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO19` reader - Port Data Output"]
pub type Pdo19R = crate::BitReader<Pdo19>;
impl Pdo19R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo19 {
        match self.bits {
            false => Pdo19::Pdo0,
            true => Pdo19::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo19::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo19::Pdo1
    }
}
#[doc = "Field `PDO19` writer - Port Data Output"]
pub type Pdo19W<'a, REG> = crate::BitWriter<'a, REG, Pdo19>;
impl<'a, REG> Pdo19W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo19::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo19::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo20 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo20> for bool {
    #[inline(always)]
    fn from(variant: Pdo20) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO20` reader - Port Data Output"]
pub type Pdo20R = crate::BitReader<Pdo20>;
impl Pdo20R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo20 {
        match self.bits {
            false => Pdo20::Pdo0,
            true => Pdo20::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo20::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo20::Pdo1
    }
}
#[doc = "Field `PDO20` writer - Port Data Output"]
pub type Pdo20W<'a, REG> = crate::BitWriter<'a, REG, Pdo20>;
impl<'a, REG> Pdo20W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo20::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo20::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo21 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo21> for bool {
    #[inline(always)]
    fn from(variant: Pdo21) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO21` reader - Port Data Output"]
pub type Pdo21R = crate::BitReader<Pdo21>;
impl Pdo21R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo21 {
        match self.bits {
            false => Pdo21::Pdo0,
            true => Pdo21::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo21::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo21::Pdo1
    }
}
#[doc = "Field `PDO21` writer - Port Data Output"]
pub type Pdo21W<'a, REG> = crate::BitWriter<'a, REG, Pdo21>;
impl<'a, REG> Pdo21W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo21::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo21::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo22 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo22> for bool {
    #[inline(always)]
    fn from(variant: Pdo22) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO22` reader - Port Data Output"]
pub type Pdo22R = crate::BitReader<Pdo22>;
impl Pdo22R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo22 {
        match self.bits {
            false => Pdo22::Pdo0,
            true => Pdo22::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo22::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo22::Pdo1
    }
}
#[doc = "Field `PDO22` writer - Port Data Output"]
pub type Pdo22W<'a, REG> = crate::BitWriter<'a, REG, Pdo22>;
impl<'a, REG> Pdo22W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo22::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo22::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo23 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo23> for bool {
    #[inline(always)]
    fn from(variant: Pdo23) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO23` reader - Port Data Output"]
pub type Pdo23R = crate::BitReader<Pdo23>;
impl Pdo23R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo23 {
        match self.bits {
            false => Pdo23::Pdo0,
            true => Pdo23::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo23::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo23::Pdo1
    }
}
#[doc = "Field `PDO23` writer - Port Data Output"]
pub type Pdo23W<'a, REG> = crate::BitWriter<'a, REG, Pdo23>;
impl<'a, REG> Pdo23W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo23::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo23::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo24 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo24> for bool {
    #[inline(always)]
    fn from(variant: Pdo24) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO24` reader - Port Data Output"]
pub type Pdo24R = crate::BitReader<Pdo24>;
impl Pdo24R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo24 {
        match self.bits {
            false => Pdo24::Pdo0,
            true => Pdo24::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo24::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo24::Pdo1
    }
}
#[doc = "Field `PDO24` writer - Port Data Output"]
pub type Pdo24W<'a, REG> = crate::BitWriter<'a, REG, Pdo24>;
impl<'a, REG> Pdo24W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo24::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo24::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo25 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo25> for bool {
    #[inline(always)]
    fn from(variant: Pdo25) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO25` reader - Port Data Output"]
pub type Pdo25R = crate::BitReader<Pdo25>;
impl Pdo25R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo25 {
        match self.bits {
            false => Pdo25::Pdo0,
            true => Pdo25::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo25::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo25::Pdo1
    }
}
#[doc = "Field `PDO25` writer - Port Data Output"]
pub type Pdo25W<'a, REG> = crate::BitWriter<'a, REG, Pdo25>;
impl<'a, REG> Pdo25W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo25::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo25::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo26 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo26> for bool {
    #[inline(always)]
    fn from(variant: Pdo26) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO26` reader - Port Data Output"]
pub type Pdo26R = crate::BitReader<Pdo26>;
impl Pdo26R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo26 {
        match self.bits {
            false => Pdo26::Pdo0,
            true => Pdo26::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo26::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo26::Pdo1
    }
}
#[doc = "Field `PDO26` writer - Port Data Output"]
pub type Pdo26W<'a, REG> = crate::BitWriter<'a, REG, Pdo26>;
impl<'a, REG> Pdo26W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo26::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo26::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo27 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo27> for bool {
    #[inline(always)]
    fn from(variant: Pdo27) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO27` reader - Port Data Output"]
pub type Pdo27R = crate::BitReader<Pdo27>;
impl Pdo27R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo27 {
        match self.bits {
            false => Pdo27::Pdo0,
            true => Pdo27::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo27::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo27::Pdo1
    }
}
#[doc = "Field `PDO27` writer - Port Data Output"]
pub type Pdo27W<'a, REG> = crate::BitWriter<'a, REG, Pdo27>;
impl<'a, REG> Pdo27W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo27::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo27::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo28 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo28> for bool {
    #[inline(always)]
    fn from(variant: Pdo28) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO28` reader - Port Data Output"]
pub type Pdo28R = crate::BitReader<Pdo28>;
impl Pdo28R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo28 {
        match self.bits {
            false => Pdo28::Pdo0,
            true => Pdo28::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo28::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo28::Pdo1
    }
}
#[doc = "Field `PDO28` writer - Port Data Output"]
pub type Pdo28W<'a, REG> = crate::BitWriter<'a, REG, Pdo28>;
impl<'a, REG> Pdo28W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo28::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo28::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo29 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo29> for bool {
    #[inline(always)]
    fn from(variant: Pdo29) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO29` reader - Port Data Output"]
pub type Pdo29R = crate::BitReader<Pdo29>;
impl Pdo29R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo29 {
        match self.bits {
            false => Pdo29::Pdo0,
            true => Pdo29::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo29::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo29::Pdo1
    }
}
#[doc = "Field `PDO29` writer - Port Data Output"]
pub type Pdo29W<'a, REG> = crate::BitWriter<'a, REG, Pdo29>;
impl<'a, REG> Pdo29W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo29::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo29::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo30 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo30> for bool {
    #[inline(always)]
    fn from(variant: Pdo30) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO30` reader - Port Data Output"]
pub type Pdo30R = crate::BitReader<Pdo30>;
impl Pdo30R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo30 {
        match self.bits {
            false => Pdo30::Pdo0,
            true => Pdo30::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo30::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo30::Pdo1
    }
}
#[doc = "Field `PDO30` writer - Port Data Output"]
pub type Pdo30W<'a, REG> = crate::BitWriter<'a, REG, Pdo30>;
impl<'a, REG> Pdo30W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo30::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo30::Pdo1)
    }
}
#[doc = "Port Data Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdo31 {
    #[doc = "0: Logic level 0"]
    Pdo0 = 0,
    #[doc = "1: Logic level 1"]
    Pdo1 = 1,
}
impl From<Pdo31> for bool {
    #[inline(always)]
    fn from(variant: Pdo31) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDO31` reader - Port Data Output"]
pub type Pdo31R = crate::BitReader<Pdo31>;
impl Pdo31R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdo31 {
        match self.bits {
            false => Pdo31::Pdo0,
            true => Pdo31::Pdo1,
        }
    }
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn is_pdo0(&self) -> bool {
        *self == Pdo31::Pdo0
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn is_pdo1(&self) -> bool {
        *self == Pdo31::Pdo1
    }
}
#[doc = "Field `PDO31` writer - Port Data Output"]
pub type Pdo31W<'a, REG> = crate::BitWriter<'a, REG, Pdo31>;
impl<'a, REG> Pdo31W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Logic level 0"]
    #[inline(always)]
    pub fn pdo0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo31::Pdo0)
    }
    #[doc = "Logic level 1"]
    #[inline(always)]
    pub fn pdo1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdo31::Pdo1)
    }
}
impl R {
    #[doc = "Bit 0 - Port Data Output"]
    #[inline(always)]
    pub fn pdo0(&self) -> Pdo0R {
        Pdo0R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Port Data Output"]
    #[inline(always)]
    pub fn pdo1(&self) -> Pdo1R {
        Pdo1R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Port Data Output"]
    #[inline(always)]
    pub fn pdo2(&self) -> Pdo2R {
        Pdo2R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Port Data Output"]
    #[inline(always)]
    pub fn pdo3(&self) -> Pdo3R {
        Pdo3R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Port Data Output"]
    #[inline(always)]
    pub fn pdo4(&self) -> Pdo4R {
        Pdo4R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Port Data Output"]
    #[inline(always)]
    pub fn pdo5(&self) -> Pdo5R {
        Pdo5R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Port Data Output"]
    #[inline(always)]
    pub fn pdo6(&self) -> Pdo6R {
        Pdo6R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Port Data Output"]
    #[inline(always)]
    pub fn pdo7(&self) -> Pdo7R {
        Pdo7R::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Port Data Output"]
    #[inline(always)]
    pub fn pdo8(&self) -> Pdo8R {
        Pdo8R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Port Data Output"]
    #[inline(always)]
    pub fn pdo9(&self) -> Pdo9R {
        Pdo9R::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Port Data Output"]
    #[inline(always)]
    pub fn pdo10(&self) -> Pdo10R {
        Pdo10R::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Port Data Output"]
    #[inline(always)]
    pub fn pdo11(&self) -> Pdo11R {
        Pdo11R::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Port Data Output"]
    #[inline(always)]
    pub fn pdo12(&self) -> Pdo12R {
        Pdo12R::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Port Data Output"]
    #[inline(always)]
    pub fn pdo13(&self) -> Pdo13R {
        Pdo13R::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Port Data Output"]
    #[inline(always)]
    pub fn pdo14(&self) -> Pdo14R {
        Pdo14R::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Port Data Output"]
    #[inline(always)]
    pub fn pdo15(&self) -> Pdo15R {
        Pdo15R::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - Port Data Output"]
    #[inline(always)]
    pub fn pdo16(&self) -> Pdo16R {
        Pdo16R::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Port Data Output"]
    #[inline(always)]
    pub fn pdo17(&self) -> Pdo17R {
        Pdo17R::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Port Data Output"]
    #[inline(always)]
    pub fn pdo18(&self) -> Pdo18R {
        Pdo18R::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - Port Data Output"]
    #[inline(always)]
    pub fn pdo19(&self) -> Pdo19R {
        Pdo19R::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - Port Data Output"]
    #[inline(always)]
    pub fn pdo20(&self) -> Pdo20R {
        Pdo20R::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - Port Data Output"]
    #[inline(always)]
    pub fn pdo21(&self) -> Pdo21R {
        Pdo21R::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - Port Data Output"]
    #[inline(always)]
    pub fn pdo22(&self) -> Pdo22R {
        Pdo22R::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - Port Data Output"]
    #[inline(always)]
    pub fn pdo23(&self) -> Pdo23R {
        Pdo23R::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bit 24 - Port Data Output"]
    #[inline(always)]
    pub fn pdo24(&self) -> Pdo24R {
        Pdo24R::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - Port Data Output"]
    #[inline(always)]
    pub fn pdo25(&self) -> Pdo25R {
        Pdo25R::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - Port Data Output"]
    #[inline(always)]
    pub fn pdo26(&self) -> Pdo26R {
        Pdo26R::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - Port Data Output"]
    #[inline(always)]
    pub fn pdo27(&self) -> Pdo27R {
        Pdo27R::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bit 28 - Port Data Output"]
    #[inline(always)]
    pub fn pdo28(&self) -> Pdo28R {
        Pdo28R::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 29 - Port Data Output"]
    #[inline(always)]
    pub fn pdo29(&self) -> Pdo29R {
        Pdo29R::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - Port Data Output"]
    #[inline(always)]
    pub fn pdo30(&self) -> Pdo30R {
        Pdo30R::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Port Data Output"]
    #[inline(always)]
    pub fn pdo31(&self) -> Pdo31R {
        Pdo31R::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Port Data Output"]
    #[inline(always)]
    pub fn pdo0(&mut self) -> Pdo0W<PdorSpec> {
        Pdo0W::new(self, 0)
    }
    #[doc = "Bit 1 - Port Data Output"]
    #[inline(always)]
    pub fn pdo1(&mut self) -> Pdo1W<PdorSpec> {
        Pdo1W::new(self, 1)
    }
    #[doc = "Bit 2 - Port Data Output"]
    #[inline(always)]
    pub fn pdo2(&mut self) -> Pdo2W<PdorSpec> {
        Pdo2W::new(self, 2)
    }
    #[doc = "Bit 3 - Port Data Output"]
    #[inline(always)]
    pub fn pdo3(&mut self) -> Pdo3W<PdorSpec> {
        Pdo3W::new(self, 3)
    }
    #[doc = "Bit 4 - Port Data Output"]
    #[inline(always)]
    pub fn pdo4(&mut self) -> Pdo4W<PdorSpec> {
        Pdo4W::new(self, 4)
    }
    #[doc = "Bit 5 - Port Data Output"]
    #[inline(always)]
    pub fn pdo5(&mut self) -> Pdo5W<PdorSpec> {
        Pdo5W::new(self, 5)
    }
    #[doc = "Bit 6 - Port Data Output"]
    #[inline(always)]
    pub fn pdo6(&mut self) -> Pdo6W<PdorSpec> {
        Pdo6W::new(self, 6)
    }
    #[doc = "Bit 7 - Port Data Output"]
    #[inline(always)]
    pub fn pdo7(&mut self) -> Pdo7W<PdorSpec> {
        Pdo7W::new(self, 7)
    }
    #[doc = "Bit 8 - Port Data Output"]
    #[inline(always)]
    pub fn pdo8(&mut self) -> Pdo8W<PdorSpec> {
        Pdo8W::new(self, 8)
    }
    #[doc = "Bit 9 - Port Data Output"]
    #[inline(always)]
    pub fn pdo9(&mut self) -> Pdo9W<PdorSpec> {
        Pdo9W::new(self, 9)
    }
    #[doc = "Bit 10 - Port Data Output"]
    #[inline(always)]
    pub fn pdo10(&mut self) -> Pdo10W<PdorSpec> {
        Pdo10W::new(self, 10)
    }
    #[doc = "Bit 11 - Port Data Output"]
    #[inline(always)]
    pub fn pdo11(&mut self) -> Pdo11W<PdorSpec> {
        Pdo11W::new(self, 11)
    }
    #[doc = "Bit 12 - Port Data Output"]
    #[inline(always)]
    pub fn pdo12(&mut self) -> Pdo12W<PdorSpec> {
        Pdo12W::new(self, 12)
    }
    #[doc = "Bit 13 - Port Data Output"]
    #[inline(always)]
    pub fn pdo13(&mut self) -> Pdo13W<PdorSpec> {
        Pdo13W::new(self, 13)
    }
    #[doc = "Bit 14 - Port Data Output"]
    #[inline(always)]
    pub fn pdo14(&mut self) -> Pdo14W<PdorSpec> {
        Pdo14W::new(self, 14)
    }
    #[doc = "Bit 15 - Port Data Output"]
    #[inline(always)]
    pub fn pdo15(&mut self) -> Pdo15W<PdorSpec> {
        Pdo15W::new(self, 15)
    }
    #[doc = "Bit 16 - Port Data Output"]
    #[inline(always)]
    pub fn pdo16(&mut self) -> Pdo16W<PdorSpec> {
        Pdo16W::new(self, 16)
    }
    #[doc = "Bit 17 - Port Data Output"]
    #[inline(always)]
    pub fn pdo17(&mut self) -> Pdo17W<PdorSpec> {
        Pdo17W::new(self, 17)
    }
    #[doc = "Bit 18 - Port Data Output"]
    #[inline(always)]
    pub fn pdo18(&mut self) -> Pdo18W<PdorSpec> {
        Pdo18W::new(self, 18)
    }
    #[doc = "Bit 19 - Port Data Output"]
    #[inline(always)]
    pub fn pdo19(&mut self) -> Pdo19W<PdorSpec> {
        Pdo19W::new(self, 19)
    }
    #[doc = "Bit 20 - Port Data Output"]
    #[inline(always)]
    pub fn pdo20(&mut self) -> Pdo20W<PdorSpec> {
        Pdo20W::new(self, 20)
    }
    #[doc = "Bit 21 - Port Data Output"]
    #[inline(always)]
    pub fn pdo21(&mut self) -> Pdo21W<PdorSpec> {
        Pdo21W::new(self, 21)
    }
    #[doc = "Bit 22 - Port Data Output"]
    #[inline(always)]
    pub fn pdo22(&mut self) -> Pdo22W<PdorSpec> {
        Pdo22W::new(self, 22)
    }
    #[doc = "Bit 23 - Port Data Output"]
    #[inline(always)]
    pub fn pdo23(&mut self) -> Pdo23W<PdorSpec> {
        Pdo23W::new(self, 23)
    }
    #[doc = "Bit 24 - Port Data Output"]
    #[inline(always)]
    pub fn pdo24(&mut self) -> Pdo24W<PdorSpec> {
        Pdo24W::new(self, 24)
    }
    #[doc = "Bit 25 - Port Data Output"]
    #[inline(always)]
    pub fn pdo25(&mut self) -> Pdo25W<PdorSpec> {
        Pdo25W::new(self, 25)
    }
    #[doc = "Bit 26 - Port Data Output"]
    #[inline(always)]
    pub fn pdo26(&mut self) -> Pdo26W<PdorSpec> {
        Pdo26W::new(self, 26)
    }
    #[doc = "Bit 27 - Port Data Output"]
    #[inline(always)]
    pub fn pdo27(&mut self) -> Pdo27W<PdorSpec> {
        Pdo27W::new(self, 27)
    }
    #[doc = "Bit 28 - Port Data Output"]
    #[inline(always)]
    pub fn pdo28(&mut self) -> Pdo28W<PdorSpec> {
        Pdo28W::new(self, 28)
    }
    #[doc = "Bit 29 - Port Data Output"]
    #[inline(always)]
    pub fn pdo29(&mut self) -> Pdo29W<PdorSpec> {
        Pdo29W::new(self, 29)
    }
    #[doc = "Bit 30 - Port Data Output"]
    #[inline(always)]
    pub fn pdo30(&mut self) -> Pdo30W<PdorSpec> {
        Pdo30W::new(self, 30)
    }
    #[doc = "Bit 31 - Port Data Output"]
    #[inline(always)]
    pub fn pdo31(&mut self) -> Pdo31W<PdorSpec> {
        Pdo31W::new(self, 31)
    }
}
#[doc = "Port Data Output\n\nYou can [`read`](crate::Reg::read) this register and get [`pdor::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pdor::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PdorSpec;
impl crate::RegisterSpec for PdorSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pdor::R`](R) reader structure"]
impl crate::Readable for PdorSpec {}
#[doc = "`write(|w| ..)` method takes [`pdor::W`](W) writer structure"]
impl crate::Writable for PdorSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PDOR to value 0"]
impl crate::Resettable for PdorSpec {}

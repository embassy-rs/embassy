#[doc = "Register `GICLR` reader"]
pub type R = crate::R<GiclrSpec>;
#[doc = "Register `GICLR` writer"]
pub type W = crate::W<GiclrSpec>;
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe0 {
    #[doc = "0: Not updated"]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe0> for bool {
    #[inline(always)]
    fn from(variant: Giwe0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE0` reader - Global Interrupt Write Enable"]
pub type Giwe0R = crate::BitReader<Giwe0>;
impl Giwe0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe0 {
        match self.bits {
            false => Giwe0::Giwe0,
            true => Giwe0::Giwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe0::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe0::Giwe1
    }
}
#[doc = "Field `GIWE0` writer - Global Interrupt Write Enable"]
pub type Giwe0W<'a, REG> = crate::BitWriter<'a, REG, Giwe0>;
impl<'a, REG> Giwe0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe0::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe0::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe1 {
    #[doc = "0: Not updated"]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe1> for bool {
    #[inline(always)]
    fn from(variant: Giwe1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE1` reader - Global Interrupt Write Enable"]
pub type Giwe1R = crate::BitReader<Giwe1>;
impl Giwe1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe1 {
        match self.bits {
            false => Giwe1::Giwe0,
            true => Giwe1::Giwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe1::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe1::Giwe1
    }
}
#[doc = "Field `GIWE1` writer - Global Interrupt Write Enable"]
pub type Giwe1W<'a, REG> = crate::BitWriter<'a, REG, Giwe1>;
impl<'a, REG> Giwe1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe1::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe1::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe2 {
    #[doc = "0: Not updated"]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe2> for bool {
    #[inline(always)]
    fn from(variant: Giwe2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE2` reader - Global Interrupt Write Enable"]
pub type Giwe2R = crate::BitReader<Giwe2>;
impl Giwe2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe2 {
        match self.bits {
            false => Giwe2::Giwe0,
            true => Giwe2::Giwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe2::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe2::Giwe1
    }
}
#[doc = "Field `GIWE2` writer - Global Interrupt Write Enable"]
pub type Giwe2W<'a, REG> = crate::BitWriter<'a, REG, Giwe2>;
impl<'a, REG> Giwe2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe2::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe2::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe3 {
    #[doc = "0: Not updated"]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe3> for bool {
    #[inline(always)]
    fn from(variant: Giwe3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE3` reader - Global Interrupt Write Enable"]
pub type Giwe3R = crate::BitReader<Giwe3>;
impl Giwe3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe3 {
        match self.bits {
            false => Giwe3::Giwe0,
            true => Giwe3::Giwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe3::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe3::Giwe1
    }
}
#[doc = "Field `GIWE3` writer - Global Interrupt Write Enable"]
pub type Giwe3W<'a, REG> = crate::BitWriter<'a, REG, Giwe3>;
impl<'a, REG> Giwe3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe3::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe3::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe4 {
    #[doc = "0: Not updated"]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe4> for bool {
    #[inline(always)]
    fn from(variant: Giwe4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE4` reader - Global Interrupt Write Enable"]
pub type Giwe4R = crate::BitReader<Giwe4>;
impl Giwe4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe4 {
        match self.bits {
            false => Giwe4::Giwe0,
            true => Giwe4::Giwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe4::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe4::Giwe1
    }
}
#[doc = "Field `GIWE4` writer - Global Interrupt Write Enable"]
pub type Giwe4W<'a, REG> = crate::BitWriter<'a, REG, Giwe4>;
impl<'a, REG> Giwe4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe4::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe4::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe5 {
    #[doc = "0: Not updated"]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe5> for bool {
    #[inline(always)]
    fn from(variant: Giwe5) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE5` reader - Global Interrupt Write Enable"]
pub type Giwe5R = crate::BitReader<Giwe5>;
impl Giwe5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe5 {
        match self.bits {
            false => Giwe5::Giwe0,
            true => Giwe5::Giwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe5::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe5::Giwe1
    }
}
#[doc = "Field `GIWE5` writer - Global Interrupt Write Enable"]
pub type Giwe5W<'a, REG> = crate::BitWriter<'a, REG, Giwe5>;
impl<'a, REG> Giwe5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe5::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe5::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe6 {
    #[doc = "0: Not updated"]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe6> for bool {
    #[inline(always)]
    fn from(variant: Giwe6) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE6` reader - Global Interrupt Write Enable"]
pub type Giwe6R = crate::BitReader<Giwe6>;
impl Giwe6R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe6 {
        match self.bits {
            false => Giwe6::Giwe0,
            true => Giwe6::Giwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe6::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe6::Giwe1
    }
}
#[doc = "Field `GIWE6` writer - Global Interrupt Write Enable"]
pub type Giwe6W<'a, REG> = crate::BitWriter<'a, REG, Giwe6>;
impl<'a, REG> Giwe6W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe6::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe6::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe7 {
    #[doc = "0: Not updated"]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe7> for bool {
    #[inline(always)]
    fn from(variant: Giwe7) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE7` reader - Global Interrupt Write Enable"]
pub type Giwe7R = crate::BitReader<Giwe7>;
impl Giwe7R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe7 {
        match self.bits {
            false => Giwe7::Giwe0,
            true => Giwe7::Giwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe7::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe7::Giwe1
    }
}
#[doc = "Field `GIWE7` writer - Global Interrupt Write Enable"]
pub type Giwe7W<'a, REG> = crate::BitWriter<'a, REG, Giwe7>;
impl<'a, REG> Giwe7W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe7::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe7::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe8 {
    #[doc = "0: Not updated"]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe8> for bool {
    #[inline(always)]
    fn from(variant: Giwe8) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE8` reader - Global Interrupt Write Enable"]
pub type Giwe8R = crate::BitReader<Giwe8>;
impl Giwe8R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe8 {
        match self.bits {
            false => Giwe8::Giwe0,
            true => Giwe8::Giwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe8::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe8::Giwe1
    }
}
#[doc = "Field `GIWE8` writer - Global Interrupt Write Enable"]
pub type Giwe8W<'a, REG> = crate::BitWriter<'a, REG, Giwe8>;
impl<'a, REG> Giwe8W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe8::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe8::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe9 {
    #[doc = "0: Not updated"]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe9> for bool {
    #[inline(always)]
    fn from(variant: Giwe9) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE9` reader - Global Interrupt Write Enable"]
pub type Giwe9R = crate::BitReader<Giwe9>;
impl Giwe9R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe9 {
        match self.bits {
            false => Giwe9::Giwe0,
            true => Giwe9::Giwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe9::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe9::Giwe1
    }
}
#[doc = "Field `GIWE9` writer - Global Interrupt Write Enable"]
pub type Giwe9W<'a, REG> = crate::BitWriter<'a, REG, Giwe9>;
impl<'a, REG> Giwe9W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe9::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe9::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe10 {
    #[doc = "0: Not updated"]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe10> for bool {
    #[inline(always)]
    fn from(variant: Giwe10) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE10` reader - Global Interrupt Write Enable"]
pub type Giwe10R = crate::BitReader<Giwe10>;
impl Giwe10R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe10 {
        match self.bits {
            false => Giwe10::Giwe0,
            true => Giwe10::Giwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe10::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe10::Giwe1
    }
}
#[doc = "Field `GIWE10` writer - Global Interrupt Write Enable"]
pub type Giwe10W<'a, REG> = crate::BitWriter<'a, REG, Giwe10>;
impl<'a, REG> Giwe10W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe10::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe10::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe11 {
    #[doc = "0: Not updated"]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe11> for bool {
    #[inline(always)]
    fn from(variant: Giwe11) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE11` reader - Global Interrupt Write Enable"]
pub type Giwe11R = crate::BitReader<Giwe11>;
impl Giwe11R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe11 {
        match self.bits {
            false => Giwe11::Giwe0,
            true => Giwe11::Giwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe11::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe11::Giwe1
    }
}
#[doc = "Field `GIWE11` writer - Global Interrupt Write Enable"]
pub type Giwe11W<'a, REG> = crate::BitWriter<'a, REG, Giwe11>;
impl<'a, REG> Giwe11W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe11::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe11::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe12 {
    #[doc = "0: Not updated"]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe12> for bool {
    #[inline(always)]
    fn from(variant: Giwe12) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE12` reader - Global Interrupt Write Enable"]
pub type Giwe12R = crate::BitReader<Giwe12>;
impl Giwe12R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe12 {
        match self.bits {
            false => Giwe12::Giwe0,
            true => Giwe12::Giwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe12::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe12::Giwe1
    }
}
#[doc = "Field `GIWE12` writer - Global Interrupt Write Enable"]
pub type Giwe12W<'a, REG> = crate::BitWriter<'a, REG, Giwe12>;
impl<'a, REG> Giwe12W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe12::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe12::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe13 {
    #[doc = "0: Not updated"]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe13> for bool {
    #[inline(always)]
    fn from(variant: Giwe13) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE13` reader - Global Interrupt Write Enable"]
pub type Giwe13R = crate::BitReader<Giwe13>;
impl Giwe13R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe13 {
        match self.bits {
            false => Giwe13::Giwe0,
            true => Giwe13::Giwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe13::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe13::Giwe1
    }
}
#[doc = "Field `GIWE13` writer - Global Interrupt Write Enable"]
pub type Giwe13W<'a, REG> = crate::BitWriter<'a, REG, Giwe13>;
impl<'a, REG> Giwe13W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe13::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe13::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe14 {
    #[doc = "0: Not updated"]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe14> for bool {
    #[inline(always)]
    fn from(variant: Giwe14) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE14` reader - Global Interrupt Write Enable"]
pub type Giwe14R = crate::BitReader<Giwe14>;
impl Giwe14R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe14 {
        match self.bits {
            false => Giwe14::Giwe0,
            true => Giwe14::Giwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe14::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe14::Giwe1
    }
}
#[doc = "Field `GIWE14` writer - Global Interrupt Write Enable"]
pub type Giwe14W<'a, REG> = crate::BitWriter<'a, REG, Giwe14>;
impl<'a, REG> Giwe14W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe14::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe14::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe15 {
    #[doc = "0: Not updated"]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe15> for bool {
    #[inline(always)]
    fn from(variant: Giwe15) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE15` reader - Global Interrupt Write Enable"]
pub type Giwe15R = crate::BitReader<Giwe15>;
impl Giwe15R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe15 {
        match self.bits {
            false => Giwe15::Giwe0,
            true => Giwe15::Giwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe15::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe15::Giwe1
    }
}
#[doc = "Field `GIWE15` writer - Global Interrupt Write Enable"]
pub type Giwe15W<'a, REG> = crate::BitWriter<'a, REG, Giwe15>;
impl<'a, REG> Giwe15W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe15::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe15::Giwe1)
    }
}
#[doc = "Field `GIWD` reader - Global Interrupt Write Data"]
pub type GiwdR = crate::FieldReader<u16>;
#[doc = "Field `GIWD` writer - Global Interrupt Write Data"]
pub type GiwdW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bit 0 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe0(&self) -> Giwe0R {
        Giwe0R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe1(&self) -> Giwe1R {
        Giwe1R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe2(&self) -> Giwe2R {
        Giwe2R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe3(&self) -> Giwe3R {
        Giwe3R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe4(&self) -> Giwe4R {
        Giwe4R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe5(&self) -> Giwe5R {
        Giwe5R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe6(&self) -> Giwe6R {
        Giwe6R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe7(&self) -> Giwe7R {
        Giwe7R::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe8(&self) -> Giwe8R {
        Giwe8R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe9(&self) -> Giwe9R {
        Giwe9R::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe10(&self) -> Giwe10R {
        Giwe10R::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe11(&self) -> Giwe11R {
        Giwe11R::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe12(&self) -> Giwe12R {
        Giwe12R::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe13(&self) -> Giwe13R {
        Giwe13R::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe14(&self) -> Giwe14R {
        Giwe14R::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe15(&self) -> Giwe15R {
        Giwe15R::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bits 16:31 - Global Interrupt Write Data"]
    #[inline(always)]
    pub fn giwd(&self) -> GiwdR {
        GiwdR::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bit 0 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe0(&mut self) -> Giwe0W<GiclrSpec> {
        Giwe0W::new(self, 0)
    }
    #[doc = "Bit 1 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe1(&mut self) -> Giwe1W<GiclrSpec> {
        Giwe1W::new(self, 1)
    }
    #[doc = "Bit 2 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe2(&mut self) -> Giwe2W<GiclrSpec> {
        Giwe2W::new(self, 2)
    }
    #[doc = "Bit 3 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe3(&mut self) -> Giwe3W<GiclrSpec> {
        Giwe3W::new(self, 3)
    }
    #[doc = "Bit 4 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe4(&mut self) -> Giwe4W<GiclrSpec> {
        Giwe4W::new(self, 4)
    }
    #[doc = "Bit 5 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe5(&mut self) -> Giwe5W<GiclrSpec> {
        Giwe5W::new(self, 5)
    }
    #[doc = "Bit 6 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe6(&mut self) -> Giwe6W<GiclrSpec> {
        Giwe6W::new(self, 6)
    }
    #[doc = "Bit 7 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe7(&mut self) -> Giwe7W<GiclrSpec> {
        Giwe7W::new(self, 7)
    }
    #[doc = "Bit 8 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe8(&mut self) -> Giwe8W<GiclrSpec> {
        Giwe8W::new(self, 8)
    }
    #[doc = "Bit 9 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe9(&mut self) -> Giwe9W<GiclrSpec> {
        Giwe9W::new(self, 9)
    }
    #[doc = "Bit 10 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe10(&mut self) -> Giwe10W<GiclrSpec> {
        Giwe10W::new(self, 10)
    }
    #[doc = "Bit 11 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe11(&mut self) -> Giwe11W<GiclrSpec> {
        Giwe11W::new(self, 11)
    }
    #[doc = "Bit 12 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe12(&mut self) -> Giwe12W<GiclrSpec> {
        Giwe12W::new(self, 12)
    }
    #[doc = "Bit 13 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe13(&mut self) -> Giwe13W<GiclrSpec> {
        Giwe13W::new(self, 13)
    }
    #[doc = "Bit 14 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe14(&mut self) -> Giwe14W<GiclrSpec> {
        Giwe14W::new(self, 14)
    }
    #[doc = "Bit 15 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe15(&mut self) -> Giwe15W<GiclrSpec> {
        Giwe15W::new(self, 15)
    }
    #[doc = "Bits 16:31 - Global Interrupt Write Data"]
    #[inline(always)]
    pub fn giwd(&mut self) -> GiwdW<GiclrSpec> {
        GiwdW::new(self, 16)
    }
}
#[doc = "Global Interrupt Control Low\n\nYou can [`read`](crate::Reg::read) this register and get [`giclr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`giclr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct GiclrSpec;
impl crate::RegisterSpec for GiclrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`giclr::R`](R) reader structure"]
impl crate::Readable for GiclrSpec {}
#[doc = "`write(|w| ..)` method takes [`giclr::W`](W) writer structure"]
impl crate::Writable for GiclrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets GICLR to value 0"]
impl crate::Resettable for GiclrSpec {}

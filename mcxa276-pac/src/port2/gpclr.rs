#[doc = "Register `GPCLR` reader"]
pub type R = crate::R<GpclrSpec>;
#[doc = "Register `GPCLR` writer"]
pub type W = crate::W<GpclrSpec>;
#[doc = "Field `GPWD` reader - Global Pin Write Data"]
pub type GpwdR = crate::FieldReader<u16>;
#[doc = "Field `GPWD` writer - Global Pin Write Data"]
pub type GpwdW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe0 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe0> for bool {
    #[inline(always)]
    fn from(variant: Gpwe0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE0` reader - Global Pin Write Enable"]
pub type Gpwe0R = crate::BitReader<Gpwe0>;
impl Gpwe0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe0 {
        match self.bits {
            false => Gpwe0::Gpwe0,
            true => Gpwe0::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe0::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe0::Gpwe1
    }
}
#[doc = "Field `GPWE0` writer - Global Pin Write Enable"]
pub type Gpwe0W<'a, REG> = crate::BitWriter<'a, REG, Gpwe0>;
impl<'a, REG> Gpwe0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe0::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe0::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe1 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe1> for bool {
    #[inline(always)]
    fn from(variant: Gpwe1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE1` reader - Global Pin Write Enable"]
pub type Gpwe1R = crate::BitReader<Gpwe1>;
impl Gpwe1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe1 {
        match self.bits {
            false => Gpwe1::Gpwe0,
            true => Gpwe1::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe1::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe1::Gpwe1
    }
}
#[doc = "Field `GPWE1` writer - Global Pin Write Enable"]
pub type Gpwe1W<'a, REG> = crate::BitWriter<'a, REG, Gpwe1>;
impl<'a, REG> Gpwe1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe1::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe1::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe2 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe2> for bool {
    #[inline(always)]
    fn from(variant: Gpwe2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE2` reader - Global Pin Write Enable"]
pub type Gpwe2R = crate::BitReader<Gpwe2>;
impl Gpwe2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe2 {
        match self.bits {
            false => Gpwe2::Gpwe0,
            true => Gpwe2::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe2::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe2::Gpwe1
    }
}
#[doc = "Field `GPWE2` writer - Global Pin Write Enable"]
pub type Gpwe2W<'a, REG> = crate::BitWriter<'a, REG, Gpwe2>;
impl<'a, REG> Gpwe2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe2::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe2::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe3 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe3> for bool {
    #[inline(always)]
    fn from(variant: Gpwe3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE3` reader - Global Pin Write Enable"]
pub type Gpwe3R = crate::BitReader<Gpwe3>;
impl Gpwe3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe3 {
        match self.bits {
            false => Gpwe3::Gpwe0,
            true => Gpwe3::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe3::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe3::Gpwe1
    }
}
#[doc = "Field `GPWE3` writer - Global Pin Write Enable"]
pub type Gpwe3W<'a, REG> = crate::BitWriter<'a, REG, Gpwe3>;
impl<'a, REG> Gpwe3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe3::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe3::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe4 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe4> for bool {
    #[inline(always)]
    fn from(variant: Gpwe4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE4` reader - Global Pin Write Enable"]
pub type Gpwe4R = crate::BitReader<Gpwe4>;
impl Gpwe4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe4 {
        match self.bits {
            false => Gpwe4::Gpwe0,
            true => Gpwe4::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe4::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe4::Gpwe1
    }
}
#[doc = "Field `GPWE4` writer - Global Pin Write Enable"]
pub type Gpwe4W<'a, REG> = crate::BitWriter<'a, REG, Gpwe4>;
impl<'a, REG> Gpwe4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe4::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe4::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe5 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe5> for bool {
    #[inline(always)]
    fn from(variant: Gpwe5) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE5` reader - Global Pin Write Enable"]
pub type Gpwe5R = crate::BitReader<Gpwe5>;
impl Gpwe5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe5 {
        match self.bits {
            false => Gpwe5::Gpwe0,
            true => Gpwe5::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe5::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe5::Gpwe1
    }
}
#[doc = "Field `GPWE5` writer - Global Pin Write Enable"]
pub type Gpwe5W<'a, REG> = crate::BitWriter<'a, REG, Gpwe5>;
impl<'a, REG> Gpwe5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe5::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe5::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe6 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe6> for bool {
    #[inline(always)]
    fn from(variant: Gpwe6) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE6` reader - Global Pin Write Enable"]
pub type Gpwe6R = crate::BitReader<Gpwe6>;
impl Gpwe6R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe6 {
        match self.bits {
            false => Gpwe6::Gpwe0,
            true => Gpwe6::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe6::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe6::Gpwe1
    }
}
#[doc = "Field `GPWE6` writer - Global Pin Write Enable"]
pub type Gpwe6W<'a, REG> = crate::BitWriter<'a, REG, Gpwe6>;
impl<'a, REG> Gpwe6W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe6::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe6::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe7 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe7> for bool {
    #[inline(always)]
    fn from(variant: Gpwe7) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE7` reader - Global Pin Write Enable"]
pub type Gpwe7R = crate::BitReader<Gpwe7>;
impl Gpwe7R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe7 {
        match self.bits {
            false => Gpwe7::Gpwe0,
            true => Gpwe7::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe7::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe7::Gpwe1
    }
}
#[doc = "Field `GPWE7` writer - Global Pin Write Enable"]
pub type Gpwe7W<'a, REG> = crate::BitWriter<'a, REG, Gpwe7>;
impl<'a, REG> Gpwe7W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe7::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe7::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe8 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe8> for bool {
    #[inline(always)]
    fn from(variant: Gpwe8) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE8` reader - Global Pin Write Enable"]
pub type Gpwe8R = crate::BitReader<Gpwe8>;
impl Gpwe8R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe8 {
        match self.bits {
            false => Gpwe8::Gpwe0,
            true => Gpwe8::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe8::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe8::Gpwe1
    }
}
#[doc = "Field `GPWE8` writer - Global Pin Write Enable"]
pub type Gpwe8W<'a, REG> = crate::BitWriter<'a, REG, Gpwe8>;
impl<'a, REG> Gpwe8W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe8::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe8::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe9 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe9> for bool {
    #[inline(always)]
    fn from(variant: Gpwe9) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE9` reader - Global Pin Write Enable"]
pub type Gpwe9R = crate::BitReader<Gpwe9>;
impl Gpwe9R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe9 {
        match self.bits {
            false => Gpwe9::Gpwe0,
            true => Gpwe9::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe9::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe9::Gpwe1
    }
}
#[doc = "Field `GPWE9` writer - Global Pin Write Enable"]
pub type Gpwe9W<'a, REG> = crate::BitWriter<'a, REG, Gpwe9>;
impl<'a, REG> Gpwe9W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe9::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe9::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe10 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe10> for bool {
    #[inline(always)]
    fn from(variant: Gpwe10) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE10` reader - Global Pin Write Enable"]
pub type Gpwe10R = crate::BitReader<Gpwe10>;
impl Gpwe10R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe10 {
        match self.bits {
            false => Gpwe10::Gpwe0,
            true => Gpwe10::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe10::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe10::Gpwe1
    }
}
#[doc = "Field `GPWE10` writer - Global Pin Write Enable"]
pub type Gpwe10W<'a, REG> = crate::BitWriter<'a, REG, Gpwe10>;
impl<'a, REG> Gpwe10W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe10::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe10::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe11 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe11> for bool {
    #[inline(always)]
    fn from(variant: Gpwe11) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE11` reader - Global Pin Write Enable"]
pub type Gpwe11R = crate::BitReader<Gpwe11>;
impl Gpwe11R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe11 {
        match self.bits {
            false => Gpwe11::Gpwe0,
            true => Gpwe11::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe11::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe11::Gpwe1
    }
}
#[doc = "Field `GPWE11` writer - Global Pin Write Enable"]
pub type Gpwe11W<'a, REG> = crate::BitWriter<'a, REG, Gpwe11>;
impl<'a, REG> Gpwe11W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe11::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe11::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe12 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe12> for bool {
    #[inline(always)]
    fn from(variant: Gpwe12) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE12` reader - Global Pin Write Enable"]
pub type Gpwe12R = crate::BitReader<Gpwe12>;
impl Gpwe12R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe12 {
        match self.bits {
            false => Gpwe12::Gpwe0,
            true => Gpwe12::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe12::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe12::Gpwe1
    }
}
#[doc = "Field `GPWE12` writer - Global Pin Write Enable"]
pub type Gpwe12W<'a, REG> = crate::BitWriter<'a, REG, Gpwe12>;
impl<'a, REG> Gpwe12W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe12::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe12::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe13 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe13> for bool {
    #[inline(always)]
    fn from(variant: Gpwe13) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE13` reader - Global Pin Write Enable"]
pub type Gpwe13R = crate::BitReader<Gpwe13>;
impl Gpwe13R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe13 {
        match self.bits {
            false => Gpwe13::Gpwe0,
            true => Gpwe13::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe13::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe13::Gpwe1
    }
}
#[doc = "Field `GPWE13` writer - Global Pin Write Enable"]
pub type Gpwe13W<'a, REG> = crate::BitWriter<'a, REG, Gpwe13>;
impl<'a, REG> Gpwe13W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe13::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe13::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe14 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe14> for bool {
    #[inline(always)]
    fn from(variant: Gpwe14) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE14` reader - Global Pin Write Enable"]
pub type Gpwe14R = crate::BitReader<Gpwe14>;
impl Gpwe14R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe14 {
        match self.bits {
            false => Gpwe14::Gpwe0,
            true => Gpwe14::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe14::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe14::Gpwe1
    }
}
#[doc = "Field `GPWE14` writer - Global Pin Write Enable"]
pub type Gpwe14W<'a, REG> = crate::BitWriter<'a, REG, Gpwe14>;
impl<'a, REG> Gpwe14W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe14::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe14::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe15 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe15> for bool {
    #[inline(always)]
    fn from(variant: Gpwe15) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE15` reader - Global Pin Write Enable"]
pub type Gpwe15R = crate::BitReader<Gpwe15>;
impl Gpwe15R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe15 {
        match self.bits {
            false => Gpwe15::Gpwe0,
            true => Gpwe15::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe15::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe15::Gpwe1
    }
}
#[doc = "Field `GPWE15` writer - Global Pin Write Enable"]
pub type Gpwe15W<'a, REG> = crate::BitWriter<'a, REG, Gpwe15>;
impl<'a, REG> Gpwe15W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe15::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe15::Gpwe1)
    }
}
impl R {
    #[doc = "Bits 0:15 - Global Pin Write Data"]
    #[inline(always)]
    pub fn gpwd(&self) -> GpwdR {
        GpwdR::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bit 16 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe0(&self) -> Gpwe0R {
        Gpwe0R::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe1(&self) -> Gpwe1R {
        Gpwe1R::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe2(&self) -> Gpwe2R {
        Gpwe2R::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe3(&self) -> Gpwe3R {
        Gpwe3R::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe4(&self) -> Gpwe4R {
        Gpwe4R::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe5(&self) -> Gpwe5R {
        Gpwe5R::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe6(&self) -> Gpwe6R {
        Gpwe6R::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe7(&self) -> Gpwe7R {
        Gpwe7R::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bit 24 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe8(&self) -> Gpwe8R {
        Gpwe8R::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe9(&self) -> Gpwe9R {
        Gpwe9R::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe10(&self) -> Gpwe10R {
        Gpwe10R::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe11(&self) -> Gpwe11R {
        Gpwe11R::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bit 28 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe12(&self) -> Gpwe12R {
        Gpwe12R::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 29 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe13(&self) -> Gpwe13R {
        Gpwe13R::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe14(&self) -> Gpwe14R {
        Gpwe14R::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe15(&self) -> Gpwe15R {
        Gpwe15R::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:15 - Global Pin Write Data"]
    #[inline(always)]
    pub fn gpwd(&mut self) -> GpwdW<GpclrSpec> {
        GpwdW::new(self, 0)
    }
    #[doc = "Bit 16 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe0(&mut self) -> Gpwe0W<GpclrSpec> {
        Gpwe0W::new(self, 16)
    }
    #[doc = "Bit 17 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe1(&mut self) -> Gpwe1W<GpclrSpec> {
        Gpwe1W::new(self, 17)
    }
    #[doc = "Bit 18 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe2(&mut self) -> Gpwe2W<GpclrSpec> {
        Gpwe2W::new(self, 18)
    }
    #[doc = "Bit 19 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe3(&mut self) -> Gpwe3W<GpclrSpec> {
        Gpwe3W::new(self, 19)
    }
    #[doc = "Bit 20 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe4(&mut self) -> Gpwe4W<GpclrSpec> {
        Gpwe4W::new(self, 20)
    }
    #[doc = "Bit 21 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe5(&mut self) -> Gpwe5W<GpclrSpec> {
        Gpwe5W::new(self, 21)
    }
    #[doc = "Bit 22 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe6(&mut self) -> Gpwe6W<GpclrSpec> {
        Gpwe6W::new(self, 22)
    }
    #[doc = "Bit 23 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe7(&mut self) -> Gpwe7W<GpclrSpec> {
        Gpwe7W::new(self, 23)
    }
    #[doc = "Bit 24 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe8(&mut self) -> Gpwe8W<GpclrSpec> {
        Gpwe8W::new(self, 24)
    }
    #[doc = "Bit 25 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe9(&mut self) -> Gpwe9W<GpclrSpec> {
        Gpwe9W::new(self, 25)
    }
    #[doc = "Bit 26 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe10(&mut self) -> Gpwe10W<GpclrSpec> {
        Gpwe10W::new(self, 26)
    }
    #[doc = "Bit 27 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe11(&mut self) -> Gpwe11W<GpclrSpec> {
        Gpwe11W::new(self, 27)
    }
    #[doc = "Bit 28 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe12(&mut self) -> Gpwe12W<GpclrSpec> {
        Gpwe12W::new(self, 28)
    }
    #[doc = "Bit 29 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe13(&mut self) -> Gpwe13W<GpclrSpec> {
        Gpwe13W::new(self, 29)
    }
    #[doc = "Bit 30 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe14(&mut self) -> Gpwe14W<GpclrSpec> {
        Gpwe14W::new(self, 30)
    }
    #[doc = "Bit 31 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe15(&mut self) -> Gpwe15W<GpclrSpec> {
        Gpwe15W::new(self, 31)
    }
}
#[doc = "Global Pin Control Low\n\nYou can [`read`](crate::Reg::read) this register and get [`gpclr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`gpclr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct GpclrSpec;
impl crate::RegisterSpec for GpclrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`gpclr::R`](R) reader structure"]
impl crate::Readable for GpclrSpec {}
#[doc = "`write(|w| ..)` method takes [`gpclr::W`](W) writer structure"]
impl crate::Writable for GpclrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets GPCLR to value 0"]
impl crate::Resettable for GpclrSpec {}

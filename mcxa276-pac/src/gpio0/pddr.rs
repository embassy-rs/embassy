#[doc = "Register `PDDR` reader"]
pub type R = crate::R<PddrSpec>;
#[doc = "Register `PDDR` writer"]
pub type W = crate::W<PddrSpec>;
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd0 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd0> for bool {
    #[inline(always)]
    fn from(variant: Pdd0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD0` reader - Port Data Direction"]
pub type Pdd0R = crate::BitReader<Pdd0>;
impl Pdd0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd0 {
        match self.bits {
            false => Pdd0::Pdd0,
            true => Pdd0::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd0::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd0::Pdd1
    }
}
#[doc = "Field `PDD0` writer - Port Data Direction"]
pub type Pdd0W<'a, REG> = crate::BitWriter<'a, REG, Pdd0>;
impl<'a, REG> Pdd0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd0::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd0::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd1 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd1> for bool {
    #[inline(always)]
    fn from(variant: Pdd1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD1` reader - Port Data Direction"]
pub type Pdd1R = crate::BitReader<Pdd1>;
impl Pdd1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd1 {
        match self.bits {
            false => Pdd1::Pdd0,
            true => Pdd1::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd1::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd1::Pdd1
    }
}
#[doc = "Field `PDD1` writer - Port Data Direction"]
pub type Pdd1W<'a, REG> = crate::BitWriter<'a, REG, Pdd1>;
impl<'a, REG> Pdd1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd1::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd1::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd2 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd2> for bool {
    #[inline(always)]
    fn from(variant: Pdd2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD2` reader - Port Data Direction"]
pub type Pdd2R = crate::BitReader<Pdd2>;
impl Pdd2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd2 {
        match self.bits {
            false => Pdd2::Pdd0,
            true => Pdd2::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd2::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd2::Pdd1
    }
}
#[doc = "Field `PDD2` writer - Port Data Direction"]
pub type Pdd2W<'a, REG> = crate::BitWriter<'a, REG, Pdd2>;
impl<'a, REG> Pdd2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd2::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd2::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd3 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd3> for bool {
    #[inline(always)]
    fn from(variant: Pdd3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD3` reader - Port Data Direction"]
pub type Pdd3R = crate::BitReader<Pdd3>;
impl Pdd3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd3 {
        match self.bits {
            false => Pdd3::Pdd0,
            true => Pdd3::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd3::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd3::Pdd1
    }
}
#[doc = "Field `PDD3` writer - Port Data Direction"]
pub type Pdd3W<'a, REG> = crate::BitWriter<'a, REG, Pdd3>;
impl<'a, REG> Pdd3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd3::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd3::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd4 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd4> for bool {
    #[inline(always)]
    fn from(variant: Pdd4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD4` reader - Port Data Direction"]
pub type Pdd4R = crate::BitReader<Pdd4>;
impl Pdd4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd4 {
        match self.bits {
            false => Pdd4::Pdd0,
            true => Pdd4::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd4::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd4::Pdd1
    }
}
#[doc = "Field `PDD4` writer - Port Data Direction"]
pub type Pdd4W<'a, REG> = crate::BitWriter<'a, REG, Pdd4>;
impl<'a, REG> Pdd4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd4::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd4::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd5 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd5> for bool {
    #[inline(always)]
    fn from(variant: Pdd5) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD5` reader - Port Data Direction"]
pub type Pdd5R = crate::BitReader<Pdd5>;
impl Pdd5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd5 {
        match self.bits {
            false => Pdd5::Pdd0,
            true => Pdd5::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd5::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd5::Pdd1
    }
}
#[doc = "Field `PDD5` writer - Port Data Direction"]
pub type Pdd5W<'a, REG> = crate::BitWriter<'a, REG, Pdd5>;
impl<'a, REG> Pdd5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd5::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd5::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd6 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd6> for bool {
    #[inline(always)]
    fn from(variant: Pdd6) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD6` reader - Port Data Direction"]
pub type Pdd6R = crate::BitReader<Pdd6>;
impl Pdd6R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd6 {
        match self.bits {
            false => Pdd6::Pdd0,
            true => Pdd6::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd6::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd6::Pdd1
    }
}
#[doc = "Field `PDD6` writer - Port Data Direction"]
pub type Pdd6W<'a, REG> = crate::BitWriter<'a, REG, Pdd6>;
impl<'a, REG> Pdd6W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd6::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd6::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd7 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd7> for bool {
    #[inline(always)]
    fn from(variant: Pdd7) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD7` reader - Port Data Direction"]
pub type Pdd7R = crate::BitReader<Pdd7>;
impl Pdd7R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd7 {
        match self.bits {
            false => Pdd7::Pdd0,
            true => Pdd7::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd7::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd7::Pdd1
    }
}
#[doc = "Field `PDD7` writer - Port Data Direction"]
pub type Pdd7W<'a, REG> = crate::BitWriter<'a, REG, Pdd7>;
impl<'a, REG> Pdd7W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd7::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd7::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd8 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd8> for bool {
    #[inline(always)]
    fn from(variant: Pdd8) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD8` reader - Port Data Direction"]
pub type Pdd8R = crate::BitReader<Pdd8>;
impl Pdd8R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd8 {
        match self.bits {
            false => Pdd8::Pdd0,
            true => Pdd8::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd8::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd8::Pdd1
    }
}
#[doc = "Field `PDD8` writer - Port Data Direction"]
pub type Pdd8W<'a, REG> = crate::BitWriter<'a, REG, Pdd8>;
impl<'a, REG> Pdd8W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd8::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd8::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd9 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd9> for bool {
    #[inline(always)]
    fn from(variant: Pdd9) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD9` reader - Port Data Direction"]
pub type Pdd9R = crate::BitReader<Pdd9>;
impl Pdd9R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd9 {
        match self.bits {
            false => Pdd9::Pdd0,
            true => Pdd9::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd9::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd9::Pdd1
    }
}
#[doc = "Field `PDD9` writer - Port Data Direction"]
pub type Pdd9W<'a, REG> = crate::BitWriter<'a, REG, Pdd9>;
impl<'a, REG> Pdd9W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd9::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd9::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd10 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd10> for bool {
    #[inline(always)]
    fn from(variant: Pdd10) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD10` reader - Port Data Direction"]
pub type Pdd10R = crate::BitReader<Pdd10>;
impl Pdd10R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd10 {
        match self.bits {
            false => Pdd10::Pdd0,
            true => Pdd10::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd10::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd10::Pdd1
    }
}
#[doc = "Field `PDD10` writer - Port Data Direction"]
pub type Pdd10W<'a, REG> = crate::BitWriter<'a, REG, Pdd10>;
impl<'a, REG> Pdd10W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd10::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd10::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd11 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd11> for bool {
    #[inline(always)]
    fn from(variant: Pdd11) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD11` reader - Port Data Direction"]
pub type Pdd11R = crate::BitReader<Pdd11>;
impl Pdd11R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd11 {
        match self.bits {
            false => Pdd11::Pdd0,
            true => Pdd11::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd11::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd11::Pdd1
    }
}
#[doc = "Field `PDD11` writer - Port Data Direction"]
pub type Pdd11W<'a, REG> = crate::BitWriter<'a, REG, Pdd11>;
impl<'a, REG> Pdd11W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd11::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd11::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd12 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd12> for bool {
    #[inline(always)]
    fn from(variant: Pdd12) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD12` reader - Port Data Direction"]
pub type Pdd12R = crate::BitReader<Pdd12>;
impl Pdd12R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd12 {
        match self.bits {
            false => Pdd12::Pdd0,
            true => Pdd12::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd12::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd12::Pdd1
    }
}
#[doc = "Field `PDD12` writer - Port Data Direction"]
pub type Pdd12W<'a, REG> = crate::BitWriter<'a, REG, Pdd12>;
impl<'a, REG> Pdd12W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd12::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd12::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd13 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd13> for bool {
    #[inline(always)]
    fn from(variant: Pdd13) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD13` reader - Port Data Direction"]
pub type Pdd13R = crate::BitReader<Pdd13>;
impl Pdd13R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd13 {
        match self.bits {
            false => Pdd13::Pdd0,
            true => Pdd13::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd13::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd13::Pdd1
    }
}
#[doc = "Field `PDD13` writer - Port Data Direction"]
pub type Pdd13W<'a, REG> = crate::BitWriter<'a, REG, Pdd13>;
impl<'a, REG> Pdd13W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd13::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd13::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd14 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd14> for bool {
    #[inline(always)]
    fn from(variant: Pdd14) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD14` reader - Port Data Direction"]
pub type Pdd14R = crate::BitReader<Pdd14>;
impl Pdd14R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd14 {
        match self.bits {
            false => Pdd14::Pdd0,
            true => Pdd14::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd14::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd14::Pdd1
    }
}
#[doc = "Field `PDD14` writer - Port Data Direction"]
pub type Pdd14W<'a, REG> = crate::BitWriter<'a, REG, Pdd14>;
impl<'a, REG> Pdd14W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd14::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd14::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd15 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd15> for bool {
    #[inline(always)]
    fn from(variant: Pdd15) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD15` reader - Port Data Direction"]
pub type Pdd15R = crate::BitReader<Pdd15>;
impl Pdd15R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd15 {
        match self.bits {
            false => Pdd15::Pdd0,
            true => Pdd15::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd15::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd15::Pdd1
    }
}
#[doc = "Field `PDD15` writer - Port Data Direction"]
pub type Pdd15W<'a, REG> = crate::BitWriter<'a, REG, Pdd15>;
impl<'a, REG> Pdd15W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd15::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd15::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd16 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd16> for bool {
    #[inline(always)]
    fn from(variant: Pdd16) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD16` reader - Port Data Direction"]
pub type Pdd16R = crate::BitReader<Pdd16>;
impl Pdd16R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd16 {
        match self.bits {
            false => Pdd16::Pdd0,
            true => Pdd16::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd16::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd16::Pdd1
    }
}
#[doc = "Field `PDD16` writer - Port Data Direction"]
pub type Pdd16W<'a, REG> = crate::BitWriter<'a, REG, Pdd16>;
impl<'a, REG> Pdd16W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd16::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd16::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd17 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd17> for bool {
    #[inline(always)]
    fn from(variant: Pdd17) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD17` reader - Port Data Direction"]
pub type Pdd17R = crate::BitReader<Pdd17>;
impl Pdd17R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd17 {
        match self.bits {
            false => Pdd17::Pdd0,
            true => Pdd17::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd17::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd17::Pdd1
    }
}
#[doc = "Field `PDD17` writer - Port Data Direction"]
pub type Pdd17W<'a, REG> = crate::BitWriter<'a, REG, Pdd17>;
impl<'a, REG> Pdd17W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd17::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd17::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd18 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd18> for bool {
    #[inline(always)]
    fn from(variant: Pdd18) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD18` reader - Port Data Direction"]
pub type Pdd18R = crate::BitReader<Pdd18>;
impl Pdd18R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd18 {
        match self.bits {
            false => Pdd18::Pdd0,
            true => Pdd18::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd18::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd18::Pdd1
    }
}
#[doc = "Field `PDD18` writer - Port Data Direction"]
pub type Pdd18W<'a, REG> = crate::BitWriter<'a, REG, Pdd18>;
impl<'a, REG> Pdd18W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd18::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd18::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd19 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd19> for bool {
    #[inline(always)]
    fn from(variant: Pdd19) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD19` reader - Port Data Direction"]
pub type Pdd19R = crate::BitReader<Pdd19>;
impl Pdd19R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd19 {
        match self.bits {
            false => Pdd19::Pdd0,
            true => Pdd19::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd19::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd19::Pdd1
    }
}
#[doc = "Field `PDD19` writer - Port Data Direction"]
pub type Pdd19W<'a, REG> = crate::BitWriter<'a, REG, Pdd19>;
impl<'a, REG> Pdd19W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd19::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd19::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd20 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd20> for bool {
    #[inline(always)]
    fn from(variant: Pdd20) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD20` reader - Port Data Direction"]
pub type Pdd20R = crate::BitReader<Pdd20>;
impl Pdd20R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd20 {
        match self.bits {
            false => Pdd20::Pdd0,
            true => Pdd20::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd20::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd20::Pdd1
    }
}
#[doc = "Field `PDD20` writer - Port Data Direction"]
pub type Pdd20W<'a, REG> = crate::BitWriter<'a, REG, Pdd20>;
impl<'a, REG> Pdd20W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd20::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd20::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd21 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd21> for bool {
    #[inline(always)]
    fn from(variant: Pdd21) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD21` reader - Port Data Direction"]
pub type Pdd21R = crate::BitReader<Pdd21>;
impl Pdd21R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd21 {
        match self.bits {
            false => Pdd21::Pdd0,
            true => Pdd21::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd21::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd21::Pdd1
    }
}
#[doc = "Field `PDD21` writer - Port Data Direction"]
pub type Pdd21W<'a, REG> = crate::BitWriter<'a, REG, Pdd21>;
impl<'a, REG> Pdd21W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd21::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd21::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd22 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd22> for bool {
    #[inline(always)]
    fn from(variant: Pdd22) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD22` reader - Port Data Direction"]
pub type Pdd22R = crate::BitReader<Pdd22>;
impl Pdd22R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd22 {
        match self.bits {
            false => Pdd22::Pdd0,
            true => Pdd22::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd22::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd22::Pdd1
    }
}
#[doc = "Field `PDD22` writer - Port Data Direction"]
pub type Pdd22W<'a, REG> = crate::BitWriter<'a, REG, Pdd22>;
impl<'a, REG> Pdd22W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd22::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd22::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd23 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd23> for bool {
    #[inline(always)]
    fn from(variant: Pdd23) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD23` reader - Port Data Direction"]
pub type Pdd23R = crate::BitReader<Pdd23>;
impl Pdd23R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd23 {
        match self.bits {
            false => Pdd23::Pdd0,
            true => Pdd23::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd23::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd23::Pdd1
    }
}
#[doc = "Field `PDD23` writer - Port Data Direction"]
pub type Pdd23W<'a, REG> = crate::BitWriter<'a, REG, Pdd23>;
impl<'a, REG> Pdd23W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd23::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd23::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd24 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd24> for bool {
    #[inline(always)]
    fn from(variant: Pdd24) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD24` reader - Port Data Direction"]
pub type Pdd24R = crate::BitReader<Pdd24>;
impl Pdd24R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd24 {
        match self.bits {
            false => Pdd24::Pdd0,
            true => Pdd24::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd24::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd24::Pdd1
    }
}
#[doc = "Field `PDD24` writer - Port Data Direction"]
pub type Pdd24W<'a, REG> = crate::BitWriter<'a, REG, Pdd24>;
impl<'a, REG> Pdd24W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd24::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd24::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd25 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd25> for bool {
    #[inline(always)]
    fn from(variant: Pdd25) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD25` reader - Port Data Direction"]
pub type Pdd25R = crate::BitReader<Pdd25>;
impl Pdd25R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd25 {
        match self.bits {
            false => Pdd25::Pdd0,
            true => Pdd25::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd25::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd25::Pdd1
    }
}
#[doc = "Field `PDD25` writer - Port Data Direction"]
pub type Pdd25W<'a, REG> = crate::BitWriter<'a, REG, Pdd25>;
impl<'a, REG> Pdd25W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd25::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd25::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd26 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd26> for bool {
    #[inline(always)]
    fn from(variant: Pdd26) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD26` reader - Port Data Direction"]
pub type Pdd26R = crate::BitReader<Pdd26>;
impl Pdd26R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd26 {
        match self.bits {
            false => Pdd26::Pdd0,
            true => Pdd26::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd26::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd26::Pdd1
    }
}
#[doc = "Field `PDD26` writer - Port Data Direction"]
pub type Pdd26W<'a, REG> = crate::BitWriter<'a, REG, Pdd26>;
impl<'a, REG> Pdd26W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd26::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd26::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd27 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd27> for bool {
    #[inline(always)]
    fn from(variant: Pdd27) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD27` reader - Port Data Direction"]
pub type Pdd27R = crate::BitReader<Pdd27>;
impl Pdd27R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd27 {
        match self.bits {
            false => Pdd27::Pdd0,
            true => Pdd27::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd27::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd27::Pdd1
    }
}
#[doc = "Field `PDD27` writer - Port Data Direction"]
pub type Pdd27W<'a, REG> = crate::BitWriter<'a, REG, Pdd27>;
impl<'a, REG> Pdd27W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd27::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd27::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd28 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd28> for bool {
    #[inline(always)]
    fn from(variant: Pdd28) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD28` reader - Port Data Direction"]
pub type Pdd28R = crate::BitReader<Pdd28>;
impl Pdd28R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd28 {
        match self.bits {
            false => Pdd28::Pdd0,
            true => Pdd28::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd28::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd28::Pdd1
    }
}
#[doc = "Field `PDD28` writer - Port Data Direction"]
pub type Pdd28W<'a, REG> = crate::BitWriter<'a, REG, Pdd28>;
impl<'a, REG> Pdd28W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd28::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd28::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd29 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd29> for bool {
    #[inline(always)]
    fn from(variant: Pdd29) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD29` reader - Port Data Direction"]
pub type Pdd29R = crate::BitReader<Pdd29>;
impl Pdd29R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd29 {
        match self.bits {
            false => Pdd29::Pdd0,
            true => Pdd29::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd29::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd29::Pdd1
    }
}
#[doc = "Field `PDD29` writer - Port Data Direction"]
pub type Pdd29W<'a, REG> = crate::BitWriter<'a, REG, Pdd29>;
impl<'a, REG> Pdd29W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd29::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd29::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd30 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd30> for bool {
    #[inline(always)]
    fn from(variant: Pdd30) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD30` reader - Port Data Direction"]
pub type Pdd30R = crate::BitReader<Pdd30>;
impl Pdd30R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd30 {
        match self.bits {
            false => Pdd30::Pdd0,
            true => Pdd30::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd30::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd30::Pdd1
    }
}
#[doc = "Field `PDD30` writer - Port Data Direction"]
pub type Pdd30W<'a, REG> = crate::BitWriter<'a, REG, Pdd30>;
impl<'a, REG> Pdd30W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd30::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd30::Pdd1)
    }
}
#[doc = "Port Data Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdd31 {
    #[doc = "0: Input"]
    Pdd0 = 0,
    #[doc = "1: Output"]
    Pdd1 = 1,
}
impl From<Pdd31> for bool {
    #[inline(always)]
    fn from(variant: Pdd31) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDD31` reader - Port Data Direction"]
pub type Pdd31R = crate::BitReader<Pdd31>;
impl Pdd31R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdd31 {
        match self.bits {
            false => Pdd31::Pdd0,
            true => Pdd31::Pdd1,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_pdd0(&self) -> bool {
        *self == Pdd31::Pdd0
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_pdd1(&self) -> bool {
        *self == Pdd31::Pdd1
    }
}
#[doc = "Field `PDD31` writer - Port Data Direction"]
pub type Pdd31W<'a, REG> = crate::BitWriter<'a, REG, Pdd31>;
impl<'a, REG> Pdd31W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn pdd0(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd31::Pdd0)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn pdd1(self) -> &'a mut crate::W<REG> {
        self.variant(Pdd31::Pdd1)
    }
}
impl R {
    #[doc = "Bit 0 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd0(&self) -> Pdd0R {
        Pdd0R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd1(&self) -> Pdd1R {
        Pdd1R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd2(&self) -> Pdd2R {
        Pdd2R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd3(&self) -> Pdd3R {
        Pdd3R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd4(&self) -> Pdd4R {
        Pdd4R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd5(&self) -> Pdd5R {
        Pdd5R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd6(&self) -> Pdd6R {
        Pdd6R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd7(&self) -> Pdd7R {
        Pdd7R::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd8(&self) -> Pdd8R {
        Pdd8R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd9(&self) -> Pdd9R {
        Pdd9R::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd10(&self) -> Pdd10R {
        Pdd10R::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd11(&self) -> Pdd11R {
        Pdd11R::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd12(&self) -> Pdd12R {
        Pdd12R::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd13(&self) -> Pdd13R {
        Pdd13R::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd14(&self) -> Pdd14R {
        Pdd14R::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd15(&self) -> Pdd15R {
        Pdd15R::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd16(&self) -> Pdd16R {
        Pdd16R::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd17(&self) -> Pdd17R {
        Pdd17R::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd18(&self) -> Pdd18R {
        Pdd18R::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd19(&self) -> Pdd19R {
        Pdd19R::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd20(&self) -> Pdd20R {
        Pdd20R::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd21(&self) -> Pdd21R {
        Pdd21R::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd22(&self) -> Pdd22R {
        Pdd22R::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd23(&self) -> Pdd23R {
        Pdd23R::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bit 24 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd24(&self) -> Pdd24R {
        Pdd24R::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd25(&self) -> Pdd25R {
        Pdd25R::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd26(&self) -> Pdd26R {
        Pdd26R::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd27(&self) -> Pdd27R {
        Pdd27R::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bit 28 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd28(&self) -> Pdd28R {
        Pdd28R::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 29 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd29(&self) -> Pdd29R {
        Pdd29R::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd30(&self) -> Pdd30R {
        Pdd30R::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd31(&self) -> Pdd31R {
        Pdd31R::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd0(&mut self) -> Pdd0W<PddrSpec> {
        Pdd0W::new(self, 0)
    }
    #[doc = "Bit 1 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd1(&mut self) -> Pdd1W<PddrSpec> {
        Pdd1W::new(self, 1)
    }
    #[doc = "Bit 2 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd2(&mut self) -> Pdd2W<PddrSpec> {
        Pdd2W::new(self, 2)
    }
    #[doc = "Bit 3 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd3(&mut self) -> Pdd3W<PddrSpec> {
        Pdd3W::new(self, 3)
    }
    #[doc = "Bit 4 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd4(&mut self) -> Pdd4W<PddrSpec> {
        Pdd4W::new(self, 4)
    }
    #[doc = "Bit 5 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd5(&mut self) -> Pdd5W<PddrSpec> {
        Pdd5W::new(self, 5)
    }
    #[doc = "Bit 6 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd6(&mut self) -> Pdd6W<PddrSpec> {
        Pdd6W::new(self, 6)
    }
    #[doc = "Bit 7 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd7(&mut self) -> Pdd7W<PddrSpec> {
        Pdd7W::new(self, 7)
    }
    #[doc = "Bit 8 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd8(&mut self) -> Pdd8W<PddrSpec> {
        Pdd8W::new(self, 8)
    }
    #[doc = "Bit 9 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd9(&mut self) -> Pdd9W<PddrSpec> {
        Pdd9W::new(self, 9)
    }
    #[doc = "Bit 10 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd10(&mut self) -> Pdd10W<PddrSpec> {
        Pdd10W::new(self, 10)
    }
    #[doc = "Bit 11 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd11(&mut self) -> Pdd11W<PddrSpec> {
        Pdd11W::new(self, 11)
    }
    #[doc = "Bit 12 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd12(&mut self) -> Pdd12W<PddrSpec> {
        Pdd12W::new(self, 12)
    }
    #[doc = "Bit 13 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd13(&mut self) -> Pdd13W<PddrSpec> {
        Pdd13W::new(self, 13)
    }
    #[doc = "Bit 14 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd14(&mut self) -> Pdd14W<PddrSpec> {
        Pdd14W::new(self, 14)
    }
    #[doc = "Bit 15 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd15(&mut self) -> Pdd15W<PddrSpec> {
        Pdd15W::new(self, 15)
    }
    #[doc = "Bit 16 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd16(&mut self) -> Pdd16W<PddrSpec> {
        Pdd16W::new(self, 16)
    }
    #[doc = "Bit 17 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd17(&mut self) -> Pdd17W<PddrSpec> {
        Pdd17W::new(self, 17)
    }
    #[doc = "Bit 18 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd18(&mut self) -> Pdd18W<PddrSpec> {
        Pdd18W::new(self, 18)
    }
    #[doc = "Bit 19 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd19(&mut self) -> Pdd19W<PddrSpec> {
        Pdd19W::new(self, 19)
    }
    #[doc = "Bit 20 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd20(&mut self) -> Pdd20W<PddrSpec> {
        Pdd20W::new(self, 20)
    }
    #[doc = "Bit 21 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd21(&mut self) -> Pdd21W<PddrSpec> {
        Pdd21W::new(self, 21)
    }
    #[doc = "Bit 22 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd22(&mut self) -> Pdd22W<PddrSpec> {
        Pdd22W::new(self, 22)
    }
    #[doc = "Bit 23 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd23(&mut self) -> Pdd23W<PddrSpec> {
        Pdd23W::new(self, 23)
    }
    #[doc = "Bit 24 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd24(&mut self) -> Pdd24W<PddrSpec> {
        Pdd24W::new(self, 24)
    }
    #[doc = "Bit 25 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd25(&mut self) -> Pdd25W<PddrSpec> {
        Pdd25W::new(self, 25)
    }
    #[doc = "Bit 26 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd26(&mut self) -> Pdd26W<PddrSpec> {
        Pdd26W::new(self, 26)
    }
    #[doc = "Bit 27 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd27(&mut self) -> Pdd27W<PddrSpec> {
        Pdd27W::new(self, 27)
    }
    #[doc = "Bit 28 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd28(&mut self) -> Pdd28W<PddrSpec> {
        Pdd28W::new(self, 28)
    }
    #[doc = "Bit 29 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd29(&mut self) -> Pdd29W<PddrSpec> {
        Pdd29W::new(self, 29)
    }
    #[doc = "Bit 30 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd30(&mut self) -> Pdd30W<PddrSpec> {
        Pdd30W::new(self, 30)
    }
    #[doc = "Bit 31 - Port Data Direction"]
    #[inline(always)]
    pub fn pdd31(&mut self) -> Pdd31W<PddrSpec> {
        Pdd31W::new(self, 31)
    }
}
#[doc = "Port Data Direction\n\nYou can [`read`](crate::Reg::read) this register and get [`pddr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pddr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PddrSpec;
impl crate::RegisterSpec for PddrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pddr::R`](R) reader structure"]
impl crate::Readable for PddrSpec {}
#[doc = "`write(|w| ..)` method takes [`pddr::W`](W) writer structure"]
impl crate::Writable for PddrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PDDR to value 0"]
impl crate::Resettable for PddrSpec {}

#[doc = "Register `PF` reader"]
pub type R = crate::R<PfSpec>;
#[doc = "Register `PF` writer"]
pub type W = crate::W<PfSpec>;
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf0 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf0> for bool {
    #[inline(always)]
    fn from(variant: Wuf0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF0` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf0R = crate::BitReader<Wuf0>;
impl Wuf0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf0 {
        match self.bits {
            false => Wuf0::NoFlag,
            true => Wuf0::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf0::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf0::Flag
    }
}
#[doc = "Field `WUF0` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf0W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf0>;
impl<'a, REG> Wuf0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf0::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf0::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf1 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf1> for bool {
    #[inline(always)]
    fn from(variant: Wuf1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF1` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf1R = crate::BitReader<Wuf1>;
impl Wuf1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf1 {
        match self.bits {
            false => Wuf1::NoFlag,
            true => Wuf1::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf1::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf1::Flag
    }
}
#[doc = "Field `WUF1` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf1W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf1>;
impl<'a, REG> Wuf1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf1::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf1::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf2 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf2> for bool {
    #[inline(always)]
    fn from(variant: Wuf2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF2` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf2R = crate::BitReader<Wuf2>;
impl Wuf2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf2 {
        match self.bits {
            false => Wuf2::NoFlag,
            true => Wuf2::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf2::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf2::Flag
    }
}
#[doc = "Field `WUF2` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf2W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf2>;
impl<'a, REG> Wuf2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf2::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf2::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf3 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf3> for bool {
    #[inline(always)]
    fn from(variant: Wuf3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF3` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf3R = crate::BitReader<Wuf3>;
impl Wuf3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf3 {
        match self.bits {
            false => Wuf3::NoFlag,
            true => Wuf3::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf3::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf3::Flag
    }
}
#[doc = "Field `WUF3` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf3W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf3>;
impl<'a, REG> Wuf3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf3::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf3::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf4 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf4> for bool {
    #[inline(always)]
    fn from(variant: Wuf4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF4` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf4R = crate::BitReader<Wuf4>;
impl Wuf4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf4 {
        match self.bits {
            false => Wuf4::NoFlag,
            true => Wuf4::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf4::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf4::Flag
    }
}
#[doc = "Field `WUF4` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf4W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf4>;
impl<'a, REG> Wuf4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf4::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf4::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf5 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf5> for bool {
    #[inline(always)]
    fn from(variant: Wuf5) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF5` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf5R = crate::BitReader<Wuf5>;
impl Wuf5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf5 {
        match self.bits {
            false => Wuf5::NoFlag,
            true => Wuf5::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf5::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf5::Flag
    }
}
#[doc = "Field `WUF5` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf5W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf5>;
impl<'a, REG> Wuf5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf5::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf5::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf6 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf6> for bool {
    #[inline(always)]
    fn from(variant: Wuf6) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF6` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf6R = crate::BitReader<Wuf6>;
impl Wuf6R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf6 {
        match self.bits {
            false => Wuf6::NoFlag,
            true => Wuf6::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf6::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf6::Flag
    }
}
#[doc = "Field `WUF6` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf6W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf6>;
impl<'a, REG> Wuf6W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf6::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf6::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf7 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf7> for bool {
    #[inline(always)]
    fn from(variant: Wuf7) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF7` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf7R = crate::BitReader<Wuf7>;
impl Wuf7R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf7 {
        match self.bits {
            false => Wuf7::NoFlag,
            true => Wuf7::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf7::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf7::Flag
    }
}
#[doc = "Field `WUF7` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf7W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf7>;
impl<'a, REG> Wuf7W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf7::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf7::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf8 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf8> for bool {
    #[inline(always)]
    fn from(variant: Wuf8) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF8` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf8R = crate::BitReader<Wuf8>;
impl Wuf8R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf8 {
        match self.bits {
            false => Wuf8::NoFlag,
            true => Wuf8::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf8::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf8::Flag
    }
}
#[doc = "Field `WUF8` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf8W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf8>;
impl<'a, REG> Wuf8W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf8::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf8::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf9 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf9> for bool {
    #[inline(always)]
    fn from(variant: Wuf9) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF9` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf9R = crate::BitReader<Wuf9>;
impl Wuf9R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf9 {
        match self.bits {
            false => Wuf9::NoFlag,
            true => Wuf9::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf9::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf9::Flag
    }
}
#[doc = "Field `WUF9` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf9W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf9>;
impl<'a, REG> Wuf9W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf9::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf9::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf10 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf10> for bool {
    #[inline(always)]
    fn from(variant: Wuf10) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF10` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf10R = crate::BitReader<Wuf10>;
impl Wuf10R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf10 {
        match self.bits {
            false => Wuf10::NoFlag,
            true => Wuf10::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf10::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf10::Flag
    }
}
#[doc = "Field `WUF10` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf10W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf10>;
impl<'a, REG> Wuf10W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf10::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf10::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf11 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf11> for bool {
    #[inline(always)]
    fn from(variant: Wuf11) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF11` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf11R = crate::BitReader<Wuf11>;
impl Wuf11R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf11 {
        match self.bits {
            false => Wuf11::NoFlag,
            true => Wuf11::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf11::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf11::Flag
    }
}
#[doc = "Field `WUF11` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf11W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf11>;
impl<'a, REG> Wuf11W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf11::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf11::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf12 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf12> for bool {
    #[inline(always)]
    fn from(variant: Wuf12) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF12` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf12R = crate::BitReader<Wuf12>;
impl Wuf12R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf12 {
        match self.bits {
            false => Wuf12::NoFlag,
            true => Wuf12::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf12::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf12::Flag
    }
}
#[doc = "Field `WUF12` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf12W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf12>;
impl<'a, REG> Wuf12W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf12::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf12::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf13 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf13> for bool {
    #[inline(always)]
    fn from(variant: Wuf13) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF13` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf13R = crate::BitReader<Wuf13>;
impl Wuf13R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf13 {
        match self.bits {
            false => Wuf13::NoFlag,
            true => Wuf13::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf13::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf13::Flag
    }
}
#[doc = "Field `WUF13` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf13W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf13>;
impl<'a, REG> Wuf13W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf13::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf13::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf14 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf14> for bool {
    #[inline(always)]
    fn from(variant: Wuf14) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF14` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf14R = crate::BitReader<Wuf14>;
impl Wuf14R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf14 {
        match self.bits {
            false => Wuf14::NoFlag,
            true => Wuf14::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf14::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf14::Flag
    }
}
#[doc = "Field `WUF14` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf14W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf14>;
impl<'a, REG> Wuf14W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf14::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf14::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf15 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf15> for bool {
    #[inline(always)]
    fn from(variant: Wuf15) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF15` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf15R = crate::BitReader<Wuf15>;
impl Wuf15R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf15 {
        match self.bits {
            false => Wuf15::NoFlag,
            true => Wuf15::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf15::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf15::Flag
    }
}
#[doc = "Field `WUF15` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf15W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf15>;
impl<'a, REG> Wuf15W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf15::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf15::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf16 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf16> for bool {
    #[inline(always)]
    fn from(variant: Wuf16) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF16` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf16R = crate::BitReader<Wuf16>;
impl Wuf16R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf16 {
        match self.bits {
            false => Wuf16::NoFlag,
            true => Wuf16::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf16::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf16::Flag
    }
}
#[doc = "Field `WUF16` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf16W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf16>;
impl<'a, REG> Wuf16W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf16::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf16::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf17 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf17> for bool {
    #[inline(always)]
    fn from(variant: Wuf17) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF17` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf17R = crate::BitReader<Wuf17>;
impl Wuf17R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf17 {
        match self.bits {
            false => Wuf17::NoFlag,
            true => Wuf17::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf17::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf17::Flag
    }
}
#[doc = "Field `WUF17` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf17W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf17>;
impl<'a, REG> Wuf17W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf17::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf17::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf18 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf18> for bool {
    #[inline(always)]
    fn from(variant: Wuf18) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF18` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf18R = crate::BitReader<Wuf18>;
impl Wuf18R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf18 {
        match self.bits {
            false => Wuf18::NoFlag,
            true => Wuf18::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf18::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf18::Flag
    }
}
#[doc = "Field `WUF18` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf18W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf18>;
impl<'a, REG> Wuf18W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf18::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf18::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf19 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf19> for bool {
    #[inline(always)]
    fn from(variant: Wuf19) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF19` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf19R = crate::BitReader<Wuf19>;
impl Wuf19R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf19 {
        match self.bits {
            false => Wuf19::NoFlag,
            true => Wuf19::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf19::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf19::Flag
    }
}
#[doc = "Field `WUF19` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf19W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf19>;
impl<'a, REG> Wuf19W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf19::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf19::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf20 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf20> for bool {
    #[inline(always)]
    fn from(variant: Wuf20) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF20` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf20R = crate::BitReader<Wuf20>;
impl Wuf20R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf20 {
        match self.bits {
            false => Wuf20::NoFlag,
            true => Wuf20::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf20::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf20::Flag
    }
}
#[doc = "Field `WUF20` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf20W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf20>;
impl<'a, REG> Wuf20W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf20::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf20::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf21 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf21> for bool {
    #[inline(always)]
    fn from(variant: Wuf21) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF21` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf21R = crate::BitReader<Wuf21>;
impl Wuf21R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf21 {
        match self.bits {
            false => Wuf21::NoFlag,
            true => Wuf21::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf21::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf21::Flag
    }
}
#[doc = "Field `WUF21` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf21W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf21>;
impl<'a, REG> Wuf21W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf21::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf21::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf22 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf22> for bool {
    #[inline(always)]
    fn from(variant: Wuf22) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF22` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf22R = crate::BitReader<Wuf22>;
impl Wuf22R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf22 {
        match self.bits {
            false => Wuf22::NoFlag,
            true => Wuf22::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf22::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf22::Flag
    }
}
#[doc = "Field `WUF22` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf22W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf22>;
impl<'a, REG> Wuf22W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf22::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf22::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf23 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf23> for bool {
    #[inline(always)]
    fn from(variant: Wuf23) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF23` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf23R = crate::BitReader<Wuf23>;
impl Wuf23R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf23 {
        match self.bits {
            false => Wuf23::NoFlag,
            true => Wuf23::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf23::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf23::Flag
    }
}
#[doc = "Field `WUF23` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf23W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf23>;
impl<'a, REG> Wuf23W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf23::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf23::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf24 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf24> for bool {
    #[inline(always)]
    fn from(variant: Wuf24) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF24` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf24R = crate::BitReader<Wuf24>;
impl Wuf24R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf24 {
        match self.bits {
            false => Wuf24::NoFlag,
            true => Wuf24::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf24::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf24::Flag
    }
}
#[doc = "Field `WUF24` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf24W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf24>;
impl<'a, REG> Wuf24W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf24::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf24::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf25 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf25> for bool {
    #[inline(always)]
    fn from(variant: Wuf25) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF25` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf25R = crate::BitReader<Wuf25>;
impl Wuf25R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf25 {
        match self.bits {
            false => Wuf25::NoFlag,
            true => Wuf25::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf25::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf25::Flag
    }
}
#[doc = "Field `WUF25` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf25W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf25>;
impl<'a, REG> Wuf25W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf25::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf25::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf26 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf26> for bool {
    #[inline(always)]
    fn from(variant: Wuf26) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF26` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf26R = crate::BitReader<Wuf26>;
impl Wuf26R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf26 {
        match self.bits {
            false => Wuf26::NoFlag,
            true => Wuf26::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf26::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf26::Flag
    }
}
#[doc = "Field `WUF26` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf26W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf26>;
impl<'a, REG> Wuf26W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf26::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf26::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf27 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf27> for bool {
    #[inline(always)]
    fn from(variant: Wuf27) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF27` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf27R = crate::BitReader<Wuf27>;
impl Wuf27R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf27 {
        match self.bits {
            false => Wuf27::NoFlag,
            true => Wuf27::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf27::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf27::Flag
    }
}
#[doc = "Field `WUF27` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf27W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf27>;
impl<'a, REG> Wuf27W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf27::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf27::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf28 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf28> for bool {
    #[inline(always)]
    fn from(variant: Wuf28) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF28` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf28R = crate::BitReader<Wuf28>;
impl Wuf28R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf28 {
        match self.bits {
            false => Wuf28::NoFlag,
            true => Wuf28::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf28::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf28::Flag
    }
}
#[doc = "Field `WUF28` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf28W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf28>;
impl<'a, REG> Wuf28W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf28::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf28::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf29 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf29> for bool {
    #[inline(always)]
    fn from(variant: Wuf29) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF29` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf29R = crate::BitReader<Wuf29>;
impl Wuf29R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf29 {
        match self.bits {
            false => Wuf29::NoFlag,
            true => Wuf29::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf29::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf29::Flag
    }
}
#[doc = "Field `WUF29` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf29W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf29>;
impl<'a, REG> Wuf29W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf29::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf29::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf30 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf30> for bool {
    #[inline(always)]
    fn from(variant: Wuf30) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF30` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf30R = crate::BitReader<Wuf30>;
impl Wuf30R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf30 {
        match self.bits {
            false => Wuf30::NoFlag,
            true => Wuf30::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf30::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf30::Flag
    }
}
#[doc = "Field `WUF30` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf30W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf30>;
impl<'a, REG> Wuf30W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf30::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf30::Flag)
    }
}
#[doc = "Wake-up Flag for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wuf31 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Wuf31> for bool {
    #[inline(always)]
    fn from(variant: Wuf31) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUF31` reader - Wake-up Flag for WUU_Pn"]
pub type Wuf31R = crate::BitReader<Wuf31>;
impl Wuf31R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wuf31 {
        match self.bits {
            false => Wuf31::NoFlag,
            true => Wuf31::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wuf31::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wuf31::Flag
    }
}
#[doc = "Field `WUF31` writer - Wake-up Flag for WUU_Pn"]
pub type Wuf31W<'a, REG> = crate::BitWriter1C<'a, REG, Wuf31>;
impl<'a, REG> Wuf31W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf31::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wuf31::Flag)
    }
}
impl R {
    #[doc = "Bit 0 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf0(&self) -> Wuf0R {
        Wuf0R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf1(&self) -> Wuf1R {
        Wuf1R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf2(&self) -> Wuf2R {
        Wuf2R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf3(&self) -> Wuf3R {
        Wuf3R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf4(&self) -> Wuf4R {
        Wuf4R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf5(&self) -> Wuf5R {
        Wuf5R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf6(&self) -> Wuf6R {
        Wuf6R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf7(&self) -> Wuf7R {
        Wuf7R::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf8(&self) -> Wuf8R {
        Wuf8R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf9(&self) -> Wuf9R {
        Wuf9R::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf10(&self) -> Wuf10R {
        Wuf10R::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf11(&self) -> Wuf11R {
        Wuf11R::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf12(&self) -> Wuf12R {
        Wuf12R::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf13(&self) -> Wuf13R {
        Wuf13R::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf14(&self) -> Wuf14R {
        Wuf14R::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf15(&self) -> Wuf15R {
        Wuf15R::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf16(&self) -> Wuf16R {
        Wuf16R::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf17(&self) -> Wuf17R {
        Wuf17R::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf18(&self) -> Wuf18R {
        Wuf18R::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf19(&self) -> Wuf19R {
        Wuf19R::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf20(&self) -> Wuf20R {
        Wuf20R::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf21(&self) -> Wuf21R {
        Wuf21R::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf22(&self) -> Wuf22R {
        Wuf22R::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf23(&self) -> Wuf23R {
        Wuf23R::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bit 24 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf24(&self) -> Wuf24R {
        Wuf24R::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf25(&self) -> Wuf25R {
        Wuf25R::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf26(&self) -> Wuf26R {
        Wuf26R::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf27(&self) -> Wuf27R {
        Wuf27R::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bit 28 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf28(&self) -> Wuf28R {
        Wuf28R::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 29 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf29(&self) -> Wuf29R {
        Wuf29R::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf30(&self) -> Wuf30R {
        Wuf30R::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf31(&self) -> Wuf31R {
        Wuf31R::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf0(&mut self) -> Wuf0W<PfSpec> {
        Wuf0W::new(self, 0)
    }
    #[doc = "Bit 1 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf1(&mut self) -> Wuf1W<PfSpec> {
        Wuf1W::new(self, 1)
    }
    #[doc = "Bit 2 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf2(&mut self) -> Wuf2W<PfSpec> {
        Wuf2W::new(self, 2)
    }
    #[doc = "Bit 3 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf3(&mut self) -> Wuf3W<PfSpec> {
        Wuf3W::new(self, 3)
    }
    #[doc = "Bit 4 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf4(&mut self) -> Wuf4W<PfSpec> {
        Wuf4W::new(self, 4)
    }
    #[doc = "Bit 5 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf5(&mut self) -> Wuf5W<PfSpec> {
        Wuf5W::new(self, 5)
    }
    #[doc = "Bit 6 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf6(&mut self) -> Wuf6W<PfSpec> {
        Wuf6W::new(self, 6)
    }
    #[doc = "Bit 7 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf7(&mut self) -> Wuf7W<PfSpec> {
        Wuf7W::new(self, 7)
    }
    #[doc = "Bit 8 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf8(&mut self) -> Wuf8W<PfSpec> {
        Wuf8W::new(self, 8)
    }
    #[doc = "Bit 9 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf9(&mut self) -> Wuf9W<PfSpec> {
        Wuf9W::new(self, 9)
    }
    #[doc = "Bit 10 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf10(&mut self) -> Wuf10W<PfSpec> {
        Wuf10W::new(self, 10)
    }
    #[doc = "Bit 11 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf11(&mut self) -> Wuf11W<PfSpec> {
        Wuf11W::new(self, 11)
    }
    #[doc = "Bit 12 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf12(&mut self) -> Wuf12W<PfSpec> {
        Wuf12W::new(self, 12)
    }
    #[doc = "Bit 13 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf13(&mut self) -> Wuf13W<PfSpec> {
        Wuf13W::new(self, 13)
    }
    #[doc = "Bit 14 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf14(&mut self) -> Wuf14W<PfSpec> {
        Wuf14W::new(self, 14)
    }
    #[doc = "Bit 15 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf15(&mut self) -> Wuf15W<PfSpec> {
        Wuf15W::new(self, 15)
    }
    #[doc = "Bit 16 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf16(&mut self) -> Wuf16W<PfSpec> {
        Wuf16W::new(self, 16)
    }
    #[doc = "Bit 17 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf17(&mut self) -> Wuf17W<PfSpec> {
        Wuf17W::new(self, 17)
    }
    #[doc = "Bit 18 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf18(&mut self) -> Wuf18W<PfSpec> {
        Wuf18W::new(self, 18)
    }
    #[doc = "Bit 19 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf19(&mut self) -> Wuf19W<PfSpec> {
        Wuf19W::new(self, 19)
    }
    #[doc = "Bit 20 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf20(&mut self) -> Wuf20W<PfSpec> {
        Wuf20W::new(self, 20)
    }
    #[doc = "Bit 21 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf21(&mut self) -> Wuf21W<PfSpec> {
        Wuf21W::new(self, 21)
    }
    #[doc = "Bit 22 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf22(&mut self) -> Wuf22W<PfSpec> {
        Wuf22W::new(self, 22)
    }
    #[doc = "Bit 23 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf23(&mut self) -> Wuf23W<PfSpec> {
        Wuf23W::new(self, 23)
    }
    #[doc = "Bit 24 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf24(&mut self) -> Wuf24W<PfSpec> {
        Wuf24W::new(self, 24)
    }
    #[doc = "Bit 25 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf25(&mut self) -> Wuf25W<PfSpec> {
        Wuf25W::new(self, 25)
    }
    #[doc = "Bit 26 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf26(&mut self) -> Wuf26W<PfSpec> {
        Wuf26W::new(self, 26)
    }
    #[doc = "Bit 27 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf27(&mut self) -> Wuf27W<PfSpec> {
        Wuf27W::new(self, 27)
    }
    #[doc = "Bit 28 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf28(&mut self) -> Wuf28W<PfSpec> {
        Wuf28W::new(self, 28)
    }
    #[doc = "Bit 29 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf29(&mut self) -> Wuf29W<PfSpec> {
        Wuf29W::new(self, 29)
    }
    #[doc = "Bit 30 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf30(&mut self) -> Wuf30W<PfSpec> {
        Wuf30W::new(self, 30)
    }
    #[doc = "Bit 31 - Wake-up Flag for WUU_Pn"]
    #[inline(always)]
    pub fn wuf31(&mut self) -> Wuf31W<PfSpec> {
        Wuf31W::new(self, 31)
    }
}
#[doc = "Pin Flag\n\nYou can [`read`](crate::Reg::read) this register and get [`pf::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pf::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PfSpec;
impl crate::RegisterSpec for PfSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pf::R`](R) reader structure"]
impl crate::Readable for PfSpec {}
#[doc = "`write(|w| ..)` method takes [`pf::W`](W) writer structure"]
impl crate::Writable for PfSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0xffff_ffff;
}
#[doc = "`reset()` method sets PF to value 0"]
impl crate::Resettable for PfSpec {}

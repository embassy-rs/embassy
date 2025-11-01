#[doc = "Register `PDIR` reader"]
pub type R = crate::R<PdirSpec>;
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi0 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi0> for bool {
    #[inline(always)]
    fn from(variant: Pdi0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI0` reader - Port Data Input"]
pub type Pdi0R = crate::BitReader<Pdi0>;
impl Pdi0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi0 {
        match self.bits {
            false => Pdi0::Pdi0,
            true => Pdi0::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi0::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi0::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi1 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi1> for bool {
    #[inline(always)]
    fn from(variant: Pdi1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI1` reader - Port Data Input"]
pub type Pdi1R = crate::BitReader<Pdi1>;
impl Pdi1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi1 {
        match self.bits {
            false => Pdi1::Pdi0,
            true => Pdi1::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi1::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi1::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi2 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi2> for bool {
    #[inline(always)]
    fn from(variant: Pdi2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI2` reader - Port Data Input"]
pub type Pdi2R = crate::BitReader<Pdi2>;
impl Pdi2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi2 {
        match self.bits {
            false => Pdi2::Pdi0,
            true => Pdi2::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi2::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi2::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi3 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi3> for bool {
    #[inline(always)]
    fn from(variant: Pdi3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI3` reader - Port Data Input"]
pub type Pdi3R = crate::BitReader<Pdi3>;
impl Pdi3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi3 {
        match self.bits {
            false => Pdi3::Pdi0,
            true => Pdi3::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi3::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi3::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi4 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi4> for bool {
    #[inline(always)]
    fn from(variant: Pdi4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI4` reader - Port Data Input"]
pub type Pdi4R = crate::BitReader<Pdi4>;
impl Pdi4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi4 {
        match self.bits {
            false => Pdi4::Pdi0,
            true => Pdi4::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi4::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi4::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi5 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi5> for bool {
    #[inline(always)]
    fn from(variant: Pdi5) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI5` reader - Port Data Input"]
pub type Pdi5R = crate::BitReader<Pdi5>;
impl Pdi5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi5 {
        match self.bits {
            false => Pdi5::Pdi0,
            true => Pdi5::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi5::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi5::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi6 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi6> for bool {
    #[inline(always)]
    fn from(variant: Pdi6) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI6` reader - Port Data Input"]
pub type Pdi6R = crate::BitReader<Pdi6>;
impl Pdi6R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi6 {
        match self.bits {
            false => Pdi6::Pdi0,
            true => Pdi6::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi6::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi6::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi7 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi7> for bool {
    #[inline(always)]
    fn from(variant: Pdi7) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI7` reader - Port Data Input"]
pub type Pdi7R = crate::BitReader<Pdi7>;
impl Pdi7R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi7 {
        match self.bits {
            false => Pdi7::Pdi0,
            true => Pdi7::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi7::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi7::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi8 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi8> for bool {
    #[inline(always)]
    fn from(variant: Pdi8) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI8` reader - Port Data Input"]
pub type Pdi8R = crate::BitReader<Pdi8>;
impl Pdi8R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi8 {
        match self.bits {
            false => Pdi8::Pdi0,
            true => Pdi8::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi8::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi8::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi9 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi9> for bool {
    #[inline(always)]
    fn from(variant: Pdi9) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI9` reader - Port Data Input"]
pub type Pdi9R = crate::BitReader<Pdi9>;
impl Pdi9R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi9 {
        match self.bits {
            false => Pdi9::Pdi0,
            true => Pdi9::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi9::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi9::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi10 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi10> for bool {
    #[inline(always)]
    fn from(variant: Pdi10) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI10` reader - Port Data Input"]
pub type Pdi10R = crate::BitReader<Pdi10>;
impl Pdi10R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi10 {
        match self.bits {
            false => Pdi10::Pdi0,
            true => Pdi10::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi10::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi10::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi11 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi11> for bool {
    #[inline(always)]
    fn from(variant: Pdi11) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI11` reader - Port Data Input"]
pub type Pdi11R = crate::BitReader<Pdi11>;
impl Pdi11R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi11 {
        match self.bits {
            false => Pdi11::Pdi0,
            true => Pdi11::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi11::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi11::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi12 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi12> for bool {
    #[inline(always)]
    fn from(variant: Pdi12) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI12` reader - Port Data Input"]
pub type Pdi12R = crate::BitReader<Pdi12>;
impl Pdi12R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi12 {
        match self.bits {
            false => Pdi12::Pdi0,
            true => Pdi12::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi12::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi12::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi13 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi13> for bool {
    #[inline(always)]
    fn from(variant: Pdi13) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI13` reader - Port Data Input"]
pub type Pdi13R = crate::BitReader<Pdi13>;
impl Pdi13R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi13 {
        match self.bits {
            false => Pdi13::Pdi0,
            true => Pdi13::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi13::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi13::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi14 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi14> for bool {
    #[inline(always)]
    fn from(variant: Pdi14) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI14` reader - Port Data Input"]
pub type Pdi14R = crate::BitReader<Pdi14>;
impl Pdi14R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi14 {
        match self.bits {
            false => Pdi14::Pdi0,
            true => Pdi14::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi14::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi14::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi15 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi15> for bool {
    #[inline(always)]
    fn from(variant: Pdi15) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI15` reader - Port Data Input"]
pub type Pdi15R = crate::BitReader<Pdi15>;
impl Pdi15R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi15 {
        match self.bits {
            false => Pdi15::Pdi0,
            true => Pdi15::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi15::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi15::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi16 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi16> for bool {
    #[inline(always)]
    fn from(variant: Pdi16) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI16` reader - Port Data Input"]
pub type Pdi16R = crate::BitReader<Pdi16>;
impl Pdi16R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi16 {
        match self.bits {
            false => Pdi16::Pdi0,
            true => Pdi16::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi16::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi16::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi17 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi17> for bool {
    #[inline(always)]
    fn from(variant: Pdi17) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI17` reader - Port Data Input"]
pub type Pdi17R = crate::BitReader<Pdi17>;
impl Pdi17R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi17 {
        match self.bits {
            false => Pdi17::Pdi0,
            true => Pdi17::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi17::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi17::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi18 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi18> for bool {
    #[inline(always)]
    fn from(variant: Pdi18) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI18` reader - Port Data Input"]
pub type Pdi18R = crate::BitReader<Pdi18>;
impl Pdi18R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi18 {
        match self.bits {
            false => Pdi18::Pdi0,
            true => Pdi18::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi18::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi18::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi19 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi19> for bool {
    #[inline(always)]
    fn from(variant: Pdi19) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI19` reader - Port Data Input"]
pub type Pdi19R = crate::BitReader<Pdi19>;
impl Pdi19R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi19 {
        match self.bits {
            false => Pdi19::Pdi0,
            true => Pdi19::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi19::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi19::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi20 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi20> for bool {
    #[inline(always)]
    fn from(variant: Pdi20) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI20` reader - Port Data Input"]
pub type Pdi20R = crate::BitReader<Pdi20>;
impl Pdi20R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi20 {
        match self.bits {
            false => Pdi20::Pdi0,
            true => Pdi20::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi20::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi20::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi21 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi21> for bool {
    #[inline(always)]
    fn from(variant: Pdi21) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI21` reader - Port Data Input"]
pub type Pdi21R = crate::BitReader<Pdi21>;
impl Pdi21R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi21 {
        match self.bits {
            false => Pdi21::Pdi0,
            true => Pdi21::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi21::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi21::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi22 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi22> for bool {
    #[inline(always)]
    fn from(variant: Pdi22) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI22` reader - Port Data Input"]
pub type Pdi22R = crate::BitReader<Pdi22>;
impl Pdi22R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi22 {
        match self.bits {
            false => Pdi22::Pdi0,
            true => Pdi22::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi22::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi22::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi23 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi23> for bool {
    #[inline(always)]
    fn from(variant: Pdi23) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI23` reader - Port Data Input"]
pub type Pdi23R = crate::BitReader<Pdi23>;
impl Pdi23R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi23 {
        match self.bits {
            false => Pdi23::Pdi0,
            true => Pdi23::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi23::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi23::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi24 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi24> for bool {
    #[inline(always)]
    fn from(variant: Pdi24) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI24` reader - Port Data Input"]
pub type Pdi24R = crate::BitReader<Pdi24>;
impl Pdi24R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi24 {
        match self.bits {
            false => Pdi24::Pdi0,
            true => Pdi24::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi24::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi24::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi25 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi25> for bool {
    #[inline(always)]
    fn from(variant: Pdi25) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI25` reader - Port Data Input"]
pub type Pdi25R = crate::BitReader<Pdi25>;
impl Pdi25R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi25 {
        match self.bits {
            false => Pdi25::Pdi0,
            true => Pdi25::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi25::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi25::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi26 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi26> for bool {
    #[inline(always)]
    fn from(variant: Pdi26) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI26` reader - Port Data Input"]
pub type Pdi26R = crate::BitReader<Pdi26>;
impl Pdi26R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi26 {
        match self.bits {
            false => Pdi26::Pdi0,
            true => Pdi26::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi26::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi26::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi27 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi27> for bool {
    #[inline(always)]
    fn from(variant: Pdi27) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI27` reader - Port Data Input"]
pub type Pdi27R = crate::BitReader<Pdi27>;
impl Pdi27R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi27 {
        match self.bits {
            false => Pdi27::Pdi0,
            true => Pdi27::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi27::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi27::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi28 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi28> for bool {
    #[inline(always)]
    fn from(variant: Pdi28) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI28` reader - Port Data Input"]
pub type Pdi28R = crate::BitReader<Pdi28>;
impl Pdi28R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi28 {
        match self.bits {
            false => Pdi28::Pdi0,
            true => Pdi28::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi28::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi28::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi29 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi29> for bool {
    #[inline(always)]
    fn from(variant: Pdi29) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI29` reader - Port Data Input"]
pub type Pdi29R = crate::BitReader<Pdi29>;
impl Pdi29R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi29 {
        match self.bits {
            false => Pdi29::Pdi0,
            true => Pdi29::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi29::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi29::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi30 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi30> for bool {
    #[inline(always)]
    fn from(variant: Pdi30) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI30` reader - Port Data Input"]
pub type Pdi30R = crate::BitReader<Pdi30>;
impl Pdi30R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi30 {
        match self.bits {
            false => Pdi30::Pdi0,
            true => Pdi30::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi30::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi30::Pdi1
    }
}
#[doc = "Port Data Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdi31 {
    #[doc = "0: Logic 0"]
    Pdi0 = 0,
    #[doc = "1: Logic 1"]
    Pdi1 = 1,
}
impl From<Pdi31> for bool {
    #[inline(always)]
    fn from(variant: Pdi31) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDI31` reader - Port Data Input"]
pub type Pdi31R = crate::BitReader<Pdi31>;
impl Pdi31R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdi31 {
        match self.bits {
            false => Pdi31::Pdi0,
            true => Pdi31::Pdi1,
        }
    }
    #[doc = "Logic 0"]
    #[inline(always)]
    pub fn is_pdi0(&self) -> bool {
        *self == Pdi31::Pdi0
    }
    #[doc = "Logic 1"]
    #[inline(always)]
    pub fn is_pdi1(&self) -> bool {
        *self == Pdi31::Pdi1
    }
}
impl R {
    #[doc = "Bit 0 - Port Data Input"]
    #[inline(always)]
    pub fn pdi0(&self) -> Pdi0R {
        Pdi0R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Port Data Input"]
    #[inline(always)]
    pub fn pdi1(&self) -> Pdi1R {
        Pdi1R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Port Data Input"]
    #[inline(always)]
    pub fn pdi2(&self) -> Pdi2R {
        Pdi2R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Port Data Input"]
    #[inline(always)]
    pub fn pdi3(&self) -> Pdi3R {
        Pdi3R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Port Data Input"]
    #[inline(always)]
    pub fn pdi4(&self) -> Pdi4R {
        Pdi4R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Port Data Input"]
    #[inline(always)]
    pub fn pdi5(&self) -> Pdi5R {
        Pdi5R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Port Data Input"]
    #[inline(always)]
    pub fn pdi6(&self) -> Pdi6R {
        Pdi6R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Port Data Input"]
    #[inline(always)]
    pub fn pdi7(&self) -> Pdi7R {
        Pdi7R::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Port Data Input"]
    #[inline(always)]
    pub fn pdi8(&self) -> Pdi8R {
        Pdi8R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Port Data Input"]
    #[inline(always)]
    pub fn pdi9(&self) -> Pdi9R {
        Pdi9R::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Port Data Input"]
    #[inline(always)]
    pub fn pdi10(&self) -> Pdi10R {
        Pdi10R::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Port Data Input"]
    #[inline(always)]
    pub fn pdi11(&self) -> Pdi11R {
        Pdi11R::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Port Data Input"]
    #[inline(always)]
    pub fn pdi12(&self) -> Pdi12R {
        Pdi12R::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Port Data Input"]
    #[inline(always)]
    pub fn pdi13(&self) -> Pdi13R {
        Pdi13R::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Port Data Input"]
    #[inline(always)]
    pub fn pdi14(&self) -> Pdi14R {
        Pdi14R::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Port Data Input"]
    #[inline(always)]
    pub fn pdi15(&self) -> Pdi15R {
        Pdi15R::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - Port Data Input"]
    #[inline(always)]
    pub fn pdi16(&self) -> Pdi16R {
        Pdi16R::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Port Data Input"]
    #[inline(always)]
    pub fn pdi17(&self) -> Pdi17R {
        Pdi17R::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Port Data Input"]
    #[inline(always)]
    pub fn pdi18(&self) -> Pdi18R {
        Pdi18R::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - Port Data Input"]
    #[inline(always)]
    pub fn pdi19(&self) -> Pdi19R {
        Pdi19R::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - Port Data Input"]
    #[inline(always)]
    pub fn pdi20(&self) -> Pdi20R {
        Pdi20R::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - Port Data Input"]
    #[inline(always)]
    pub fn pdi21(&self) -> Pdi21R {
        Pdi21R::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - Port Data Input"]
    #[inline(always)]
    pub fn pdi22(&self) -> Pdi22R {
        Pdi22R::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - Port Data Input"]
    #[inline(always)]
    pub fn pdi23(&self) -> Pdi23R {
        Pdi23R::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bit 24 - Port Data Input"]
    #[inline(always)]
    pub fn pdi24(&self) -> Pdi24R {
        Pdi24R::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - Port Data Input"]
    #[inline(always)]
    pub fn pdi25(&self) -> Pdi25R {
        Pdi25R::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - Port Data Input"]
    #[inline(always)]
    pub fn pdi26(&self) -> Pdi26R {
        Pdi26R::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - Port Data Input"]
    #[inline(always)]
    pub fn pdi27(&self) -> Pdi27R {
        Pdi27R::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bit 28 - Port Data Input"]
    #[inline(always)]
    pub fn pdi28(&self) -> Pdi28R {
        Pdi28R::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 29 - Port Data Input"]
    #[inline(always)]
    pub fn pdi29(&self) -> Pdi29R {
        Pdi29R::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - Port Data Input"]
    #[inline(always)]
    pub fn pdi30(&self) -> Pdi30R {
        Pdi30R::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Port Data Input"]
    #[inline(always)]
    pub fn pdi31(&self) -> Pdi31R {
        Pdi31R::new(((self.bits >> 31) & 1) != 0)
    }
}
#[doc = "Port Data Input\n\nYou can [`read`](crate::Reg::read) this register and get [`pdir::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PdirSpec;
impl crate::RegisterSpec for PdirSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pdir::R`](R) reader structure"]
impl crate::Readable for PdirSpec {}
#[doc = "`reset()` method sets PDIR to value 0"]
impl crate::Resettable for PdirSpec {}

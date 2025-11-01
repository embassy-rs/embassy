#[doc = "Register `STATUS` reader"]
pub type R = crate::R<StatusSpec>;
#[doc = "Test Fail, 1-Bit Run, Sampling 0s.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tf1br0 {
    #[doc = "0: The 1-Bit Run, Sampling 0s Test has passed"]
    Disable = 0,
    #[doc = "1: The 1-Bit Run, Sampling 0s Test has failed"]
    Enable = 1,
}
impl From<Tf1br0> for bool {
    #[inline(always)]
    fn from(variant: Tf1br0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TF1BR0` reader - Test Fail, 1-Bit Run, Sampling 0s."]
pub type Tf1br0R = crate::BitReader<Tf1br0>;
impl Tf1br0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tf1br0 {
        match self.bits {
            false => Tf1br0::Disable,
            true => Tf1br0::Enable,
        }
    }
    #[doc = "The 1-Bit Run, Sampling 0s Test has passed"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tf1br0::Disable
    }
    #[doc = "The 1-Bit Run, Sampling 0s Test has failed"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tf1br0::Enable
    }
}
#[doc = "Test Fail, 1-Bit Run, Sampling 1s.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tf1br1 {
    #[doc = "0: The 1-Bit Run, Sampling 1s Test has passed"]
    Disable = 0,
    #[doc = "1: The 1-Bit Run, Sampling 1s Test has failed"]
    Enable = 1,
}
impl From<Tf1br1> for bool {
    #[inline(always)]
    fn from(variant: Tf1br1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TF1BR1` reader - Test Fail, 1-Bit Run, Sampling 1s."]
pub type Tf1br1R = crate::BitReader<Tf1br1>;
impl Tf1br1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tf1br1 {
        match self.bits {
            false => Tf1br1::Disable,
            true => Tf1br1::Enable,
        }
    }
    #[doc = "The 1-Bit Run, Sampling 1s Test has passed"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tf1br1::Disable
    }
    #[doc = "The 1-Bit Run, Sampling 1s Test has failed"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tf1br1::Enable
    }
}
#[doc = "Test Fail, 2-Bit Run, Sampling 0s.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tf2br0 {
    #[doc = "0: The 2-Bit Run, Sampling 0s Test has passed"]
    Disable = 0,
    #[doc = "1: The 2-Bit Run, Sampling 0s Test has failed"]
    Enable = 1,
}
impl From<Tf2br0> for bool {
    #[inline(always)]
    fn from(variant: Tf2br0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TF2BR0` reader - Test Fail, 2-Bit Run, Sampling 0s."]
pub type Tf2br0R = crate::BitReader<Tf2br0>;
impl Tf2br0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tf2br0 {
        match self.bits {
            false => Tf2br0::Disable,
            true => Tf2br0::Enable,
        }
    }
    #[doc = "The 2-Bit Run, Sampling 0s Test has passed"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tf2br0::Disable
    }
    #[doc = "The 2-Bit Run, Sampling 0s Test has failed"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tf2br0::Enable
    }
}
#[doc = "Test Fail, 2-Bit Run, Sampling 1s.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tf2br1 {
    #[doc = "0: The 2-Bit Run, Sampling 1s Test has passed"]
    Disable = 0,
    #[doc = "1: The 2-Bit Run, Sampling 1s Test has failed"]
    Enable = 1,
}
impl From<Tf2br1> for bool {
    #[inline(always)]
    fn from(variant: Tf2br1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TF2BR1` reader - Test Fail, 2-Bit Run, Sampling 1s."]
pub type Tf2br1R = crate::BitReader<Tf2br1>;
impl Tf2br1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tf2br1 {
        match self.bits {
            false => Tf2br1::Disable,
            true => Tf2br1::Enable,
        }
    }
    #[doc = "The 2-Bit Run, Sampling 1s Test has passed"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tf2br1::Disable
    }
    #[doc = "The 2-Bit Run, Sampling 1s Test has failed"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tf2br1::Enable
    }
}
#[doc = "Test Fail, 3-Bit Run, Sampling 0s.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tf3br0 {
    #[doc = "0: The 3-Bit Run, Sampling 0s Test has passed"]
    Disable = 0,
    #[doc = "1: The 3-Bit Run, Sampling 0s Test has failed"]
    Enable = 1,
}
impl From<Tf3br0> for bool {
    #[inline(always)]
    fn from(variant: Tf3br0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TF3BR0` reader - Test Fail, 3-Bit Run, Sampling 0s."]
pub type Tf3br0R = crate::BitReader<Tf3br0>;
impl Tf3br0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tf3br0 {
        match self.bits {
            false => Tf3br0::Disable,
            true => Tf3br0::Enable,
        }
    }
    #[doc = "The 3-Bit Run, Sampling 0s Test has passed"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tf3br0::Disable
    }
    #[doc = "The 3-Bit Run, Sampling 0s Test has failed"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tf3br0::Enable
    }
}
#[doc = "Test Fail\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tf3br1 {
    #[doc = "0: The 3-Bit Run, Sampling 1s Test has passed"]
    Disable = 0,
    #[doc = "1: The 3-Bit Run, Sampling 1s Test has failed"]
    Enable = 1,
}
impl From<Tf3br1> for bool {
    #[inline(always)]
    fn from(variant: Tf3br1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TF3BR1` reader - Test Fail"]
pub type Tf3br1R = crate::BitReader<Tf3br1>;
impl Tf3br1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tf3br1 {
        match self.bits {
            false => Tf3br1::Disable,
            true => Tf3br1::Enable,
        }
    }
    #[doc = "The 3-Bit Run, Sampling 1s Test has passed"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tf3br1::Disable
    }
    #[doc = "The 3-Bit Run, Sampling 1s Test has failed"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tf3br1::Enable
    }
}
#[doc = "Test Fail, 4-Bit Run, Sampling 0s\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tf4br0 {
    #[doc = "0: The 4-Bit Run, Sampling 0s Test has passed"]
    Disable = 0,
    #[doc = "1: The 4-Bit Run, Sampling 0s Test has failed"]
    Enable = 1,
}
impl From<Tf4br0> for bool {
    #[inline(always)]
    fn from(variant: Tf4br0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TF4BR0` reader - Test Fail, 4-Bit Run, Sampling 0s"]
pub type Tf4br0R = crate::BitReader<Tf4br0>;
impl Tf4br0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tf4br0 {
        match self.bits {
            false => Tf4br0::Disable,
            true => Tf4br0::Enable,
        }
    }
    #[doc = "The 4-Bit Run, Sampling 0s Test has passed"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tf4br0::Disable
    }
    #[doc = "The 4-Bit Run, Sampling 0s Test has failed"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tf4br0::Enable
    }
}
#[doc = "Test Fail, 4-Bit Run, Sampling 1s.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tf4br1 {
    #[doc = "0: The 4-Bit Run, Sampling 1s Test has passed"]
    Disable = 0,
    #[doc = "1: The 4-Bit Run, Sampling 1s Test has failed"]
    Enable = 1,
}
impl From<Tf4br1> for bool {
    #[inline(always)]
    fn from(variant: Tf4br1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TF4BR1` reader - Test Fail, 4-Bit Run, Sampling 1s."]
pub type Tf4br1R = crate::BitReader<Tf4br1>;
impl Tf4br1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tf4br1 {
        match self.bits {
            false => Tf4br1::Disable,
            true => Tf4br1::Enable,
        }
    }
    #[doc = "The 4-Bit Run, Sampling 1s Test has passed"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tf4br1::Disable
    }
    #[doc = "The 4-Bit Run, Sampling 1s Test has failed"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tf4br1::Enable
    }
}
#[doc = "Test Fail, 5-Bit Run, Sampling 0s.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tf5br0 {
    #[doc = "0: The 5-Bit Run, Sampling 0s Test has passed"]
    Disable = 0,
    #[doc = "1: The 5-Bit Run, Sampling 0s Test has failed"]
    Enable = 1,
}
impl From<Tf5br0> for bool {
    #[inline(always)]
    fn from(variant: Tf5br0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TF5BR0` reader - Test Fail, 5-Bit Run, Sampling 0s."]
pub type Tf5br0R = crate::BitReader<Tf5br0>;
impl Tf5br0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tf5br0 {
        match self.bits {
            false => Tf5br0::Disable,
            true => Tf5br0::Enable,
        }
    }
    #[doc = "The 5-Bit Run, Sampling 0s Test has passed"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tf5br0::Disable
    }
    #[doc = "The 5-Bit Run, Sampling 0s Test has failed"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tf5br0::Enable
    }
}
#[doc = "Test Fail, 5-Bit Run, Sampling 1s. If TF5BR1=1, the 5-Bit Run, Sampling 1s Test has failed.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tf5br1 {
    #[doc = "0: The 5-Bit Run, Sampling 1s Test has passed"]
    Disable = 0,
    #[doc = "1: The 5-Bit Run, Sampling 1s Test has failed"]
    Enable = 1,
}
impl From<Tf5br1> for bool {
    #[inline(always)]
    fn from(variant: Tf5br1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TF5BR1` reader - Test Fail, 5-Bit Run, Sampling 1s. If TF5BR1=1, the 5-Bit Run, Sampling 1s Test has failed."]
pub type Tf5br1R = crate::BitReader<Tf5br1>;
impl Tf5br1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tf5br1 {
        match self.bits {
            false => Tf5br1::Disable,
            true => Tf5br1::Enable,
        }
    }
    #[doc = "The 5-Bit Run, Sampling 1s Test has passed"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tf5br1::Disable
    }
    #[doc = "The 5-Bit Run, Sampling 1s Test has failed"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tf5br1::Enable
    }
}
#[doc = "Test Fail, 6 Plus Bit Run, Sampling 0s.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tf6pbr0 {
    #[doc = "0: The 6 Plus Bit Run, Sampling 0s Test has passed"]
    Disable = 0,
    #[doc = "1: the 6 Plus Bit Run, Sampling 0s Test has failed"]
    Enable = 1,
}
impl From<Tf6pbr0> for bool {
    #[inline(always)]
    fn from(variant: Tf6pbr0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TF6PBR0` reader - Test Fail, 6 Plus Bit Run, Sampling 0s."]
pub type Tf6pbr0R = crate::BitReader<Tf6pbr0>;
impl Tf6pbr0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tf6pbr0 {
        match self.bits {
            false => Tf6pbr0::Disable,
            true => Tf6pbr0::Enable,
        }
    }
    #[doc = "The 6 Plus Bit Run, Sampling 0s Test has passed"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tf6pbr0::Disable
    }
    #[doc = "the 6 Plus Bit Run, Sampling 0s Test has failed"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tf6pbr0::Enable
    }
}
#[doc = "Test Fail, 6 Plus Bit Run, Sampling 1s.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tf6pbr1 {
    #[doc = "0: The 6 Plus Bit Run, Sampling 1s Test has passed"]
    Disable = 0,
    #[doc = "1: The 6 Plus Bit Run, Sampling 1s Test has failed"]
    Enable = 1,
}
impl From<Tf6pbr1> for bool {
    #[inline(always)]
    fn from(variant: Tf6pbr1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TF6PBR1` reader - Test Fail, 6 Plus Bit Run, Sampling 1s."]
pub type Tf6pbr1R = crate::BitReader<Tf6pbr1>;
impl Tf6pbr1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tf6pbr1 {
        match self.bits {
            false => Tf6pbr1::Disable,
            true => Tf6pbr1::Enable,
        }
    }
    #[doc = "The 6 Plus Bit Run, Sampling 1s Test has passed"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tf6pbr1::Disable
    }
    #[doc = "The 6 Plus Bit Run, Sampling 1s Test has failed"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tf6pbr1::Enable
    }
}
#[doc = "Test Fail, Sparse Bit.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tfsb {
    #[doc = "0: The Sparse Bit Test has passed"]
    Disable = 0,
    #[doc = "1: The Sparse Bit Test has failed"]
    Enable = 1,
}
impl From<Tfsb> for bool {
    #[inline(always)]
    fn from(variant: Tfsb) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TFSB` reader - Test Fail, Sparse Bit."]
pub type TfsbR = crate::BitReader<Tfsb>;
impl TfsbR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tfsb {
        match self.bits {
            false => Tfsb::Disable,
            true => Tfsb::Enable,
        }
    }
    #[doc = "The Sparse Bit Test has passed"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tfsb::Disable
    }
    #[doc = "The Sparse Bit Test has failed"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tfsb::Enable
    }
}
#[doc = "Test Fail, Long Run.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tflr {
    #[doc = "0: The Long Run Test has passed"]
    Disable = 0,
    #[doc = "1: The Long Run Test has failed"]
    Enable = 1,
}
impl From<Tflr> for bool {
    #[inline(always)]
    fn from(variant: Tflr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TFLR` reader - Test Fail, Long Run."]
pub type TflrR = crate::BitReader<Tflr>;
impl TflrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tflr {
        match self.bits {
            false => Tflr::Disable,
            true => Tflr::Enable,
        }
    }
    #[doc = "The Long Run Test has passed"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tflr::Disable
    }
    #[doc = "The Long Run Test has failed"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tflr::Enable
    }
}
#[doc = "Test Fail, Poker.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tfp {
    #[doc = "0: The Poker Test has passed"]
    Disable = 0,
    #[doc = "1: The Poker Test has failed"]
    Enable = 1,
}
impl From<Tfp> for bool {
    #[inline(always)]
    fn from(variant: Tfp) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TFP` reader - Test Fail, Poker."]
pub type TfpR = crate::BitReader<Tfp>;
impl TfpR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tfp {
        match self.bits {
            false => Tfp::Disable,
            true => Tfp::Enable,
        }
    }
    #[doc = "The Poker Test has passed"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tfp::Disable
    }
    #[doc = "The Poker Test has failed"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tfp::Enable
    }
}
#[doc = "Test Fail, Mono Bit.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tfmb {
    #[doc = "0: The Mono Bit Test has passed"]
    Disable = 0,
    #[doc = "1: The Mono Bit Test has failed"]
    Enable = 1,
}
impl From<Tfmb> for bool {
    #[inline(always)]
    fn from(variant: Tfmb) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TFMB` reader - Test Fail, Mono Bit."]
pub type TfmbR = crate::BitReader<Tfmb>;
impl TfmbR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tfmb {
        match self.bits {
            false => Tfmb::Disable,
            true => Tfmb::Enable,
        }
    }
    #[doc = "The Mono Bit Test has passed"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tfmb::Disable
    }
    #[doc = "The Mono Bit Test has failed"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tfmb::Enable
    }
}
#[doc = "Field `RETRY_CT` reader - RETRY COUNT"]
pub type RetryCtR = crate::FieldReader;
impl R {
    #[doc = "Bit 0 - Test Fail, 1-Bit Run, Sampling 0s."]
    #[inline(always)]
    pub fn tf1br0(&self) -> Tf1br0R {
        Tf1br0R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Test Fail, 1-Bit Run, Sampling 1s."]
    #[inline(always)]
    pub fn tf1br1(&self) -> Tf1br1R {
        Tf1br1R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Test Fail, 2-Bit Run, Sampling 0s."]
    #[inline(always)]
    pub fn tf2br0(&self) -> Tf2br0R {
        Tf2br0R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Test Fail, 2-Bit Run, Sampling 1s."]
    #[inline(always)]
    pub fn tf2br1(&self) -> Tf2br1R {
        Tf2br1R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Test Fail, 3-Bit Run, Sampling 0s."]
    #[inline(always)]
    pub fn tf3br0(&self) -> Tf3br0R {
        Tf3br0R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Test Fail"]
    #[inline(always)]
    pub fn tf3br1(&self) -> Tf3br1R {
        Tf3br1R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Test Fail, 4-Bit Run, Sampling 0s"]
    #[inline(always)]
    pub fn tf4br0(&self) -> Tf4br0R {
        Tf4br0R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Test Fail, 4-Bit Run, Sampling 1s."]
    #[inline(always)]
    pub fn tf4br1(&self) -> Tf4br1R {
        Tf4br1R::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Test Fail, 5-Bit Run, Sampling 0s."]
    #[inline(always)]
    pub fn tf5br0(&self) -> Tf5br0R {
        Tf5br0R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Test Fail, 5-Bit Run, Sampling 1s. If TF5BR1=1, the 5-Bit Run, Sampling 1s Test has failed."]
    #[inline(always)]
    pub fn tf5br1(&self) -> Tf5br1R {
        Tf5br1R::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Test Fail, 6 Plus Bit Run, Sampling 0s."]
    #[inline(always)]
    pub fn tf6pbr0(&self) -> Tf6pbr0R {
        Tf6pbr0R::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Test Fail, 6 Plus Bit Run, Sampling 1s."]
    #[inline(always)]
    pub fn tf6pbr1(&self) -> Tf6pbr1R {
        Tf6pbr1R::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Test Fail, Sparse Bit."]
    #[inline(always)]
    pub fn tfsb(&self) -> TfsbR {
        TfsbR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Test Fail, Long Run."]
    #[inline(always)]
    pub fn tflr(&self) -> TflrR {
        TflrR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Test Fail, Poker."]
    #[inline(always)]
    pub fn tfp(&self) -> TfpR {
        TfpR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Test Fail, Mono Bit."]
    #[inline(always)]
    pub fn tfmb(&self) -> TfmbR {
        TfmbR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bits 16:19 - RETRY COUNT"]
    #[inline(always)]
    pub fn retry_ct(&self) -> RetryCtR {
        RetryCtR::new(((self.bits >> 16) & 0x0f) as u8)
    }
}
#[doc = "Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`status::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct StatusSpec;
impl crate::RegisterSpec for StatusSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`status::R`](R) reader structure"]
impl crate::Readable for StatusSpec {}
#[doc = "`reset()` method sets STATUS to value 0"]
impl crate::Resettable for StatusSpec {}

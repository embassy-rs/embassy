#[doc = "Register `TRIGFIL_STAT0` reader"]
pub type R = crate::R<TrigfilStat0Spec>;
#[doc = "TRIG_IN value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrigIn0Val {
    #[doc = "0: TRIG_IN0 is 0"]
    Val0 = 0,
    #[doc = "1: TRIG_IN0 is 1"]
    Val1 = 1,
}
impl From<TrigIn0Val> for bool {
    #[inline(always)]
    fn from(variant: TrigIn0Val) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TRIG_IN0_VAL` reader - TRIG_IN value"]
pub type TrigIn0ValR = crate::BitReader<TrigIn0Val>;
impl TrigIn0ValR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> TrigIn0Val {
        match self.bits {
            false => TrigIn0Val::Val0,
            true => TrigIn0Val::Val1,
        }
    }
    #[doc = "TRIG_IN0 is 0"]
    #[inline(always)]
    pub fn is_val0(&self) -> bool {
        *self == TrigIn0Val::Val0
    }
    #[doc = "TRIG_IN0 is 1"]
    #[inline(always)]
    pub fn is_val1(&self) -> bool {
        *self == TrigIn0Val::Val1
    }
}
#[doc = "TRIG_IN value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrigIn1Val {
    #[doc = "0: TRIG_IN1 is 0"]
    Val0 = 0,
    #[doc = "1: TRIG_IN1 is 1"]
    Val1 = 1,
}
impl From<TrigIn1Val> for bool {
    #[inline(always)]
    fn from(variant: TrigIn1Val) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TRIG_IN1_VAL` reader - TRIG_IN value"]
pub type TrigIn1ValR = crate::BitReader<TrigIn1Val>;
impl TrigIn1ValR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> TrigIn1Val {
        match self.bits {
            false => TrigIn1Val::Val0,
            true => TrigIn1Val::Val1,
        }
    }
    #[doc = "TRIG_IN1 is 0"]
    #[inline(always)]
    pub fn is_val0(&self) -> bool {
        *self == TrigIn1Val::Val0
    }
    #[doc = "TRIG_IN1 is 1"]
    #[inline(always)]
    pub fn is_val1(&self) -> bool {
        *self == TrigIn1Val::Val1
    }
}
#[doc = "TRIG_IN value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrigIn2Val {
    #[doc = "0: TRIG_IN2 is 0"]
    Val0 = 0,
    #[doc = "1: TRIG_IN2 is 1"]
    Val1 = 1,
}
impl From<TrigIn2Val> for bool {
    #[inline(always)]
    fn from(variant: TrigIn2Val) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TRIG_IN2_VAL` reader - TRIG_IN value"]
pub type TrigIn2ValR = crate::BitReader<TrigIn2Val>;
impl TrigIn2ValR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> TrigIn2Val {
        match self.bits {
            false => TrigIn2Val::Val0,
            true => TrigIn2Val::Val1,
        }
    }
    #[doc = "TRIG_IN2 is 0"]
    #[inline(always)]
    pub fn is_val0(&self) -> bool {
        *self == TrigIn2Val::Val0
    }
    #[doc = "TRIG_IN2 is 1"]
    #[inline(always)]
    pub fn is_val1(&self) -> bool {
        *self == TrigIn2Val::Val1
    }
}
#[doc = "TRIG_IN value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrigIn3Val {
    #[doc = "0: TRIG_IN3 is 0"]
    Val0 = 0,
    #[doc = "1: TRIG_IN3 is 1"]
    Val1 = 1,
}
impl From<TrigIn3Val> for bool {
    #[inline(always)]
    fn from(variant: TrigIn3Val) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TRIG_IN3_VAL` reader - TRIG_IN value"]
pub type TrigIn3ValR = crate::BitReader<TrigIn3Val>;
impl TrigIn3ValR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> TrigIn3Val {
        match self.bits {
            false => TrigIn3Val::Val0,
            true => TrigIn3Val::Val1,
        }
    }
    #[doc = "TRIG_IN3 is 0"]
    #[inline(always)]
    pub fn is_val0(&self) -> bool {
        *self == TrigIn3Val::Val0
    }
    #[doc = "TRIG_IN3 is 1"]
    #[inline(always)]
    pub fn is_val1(&self) -> bool {
        *self == TrigIn3Val::Val1
    }
}
#[doc = "TRIG_IN value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrigIn4Val {
    #[doc = "0: TRIG_IN4 is 0"]
    Val0 = 0,
    #[doc = "1: TRIG_IN4 is 1"]
    Val1 = 1,
}
impl From<TrigIn4Val> for bool {
    #[inline(always)]
    fn from(variant: TrigIn4Val) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TRIG_IN4_VAL` reader - TRIG_IN value"]
pub type TrigIn4ValR = crate::BitReader<TrigIn4Val>;
impl TrigIn4ValR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> TrigIn4Val {
        match self.bits {
            false => TrigIn4Val::Val0,
            true => TrigIn4Val::Val1,
        }
    }
    #[doc = "TRIG_IN4 is 0"]
    #[inline(always)]
    pub fn is_val0(&self) -> bool {
        *self == TrigIn4Val::Val0
    }
    #[doc = "TRIG_IN4 is 1"]
    #[inline(always)]
    pub fn is_val1(&self) -> bool {
        *self == TrigIn4Val::Val1
    }
}
#[doc = "TRIG_IN value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrigIn5Val {
    #[doc = "0: TRIG_IN5 is 0"]
    Val0 = 0,
    #[doc = "1: TRIG_IN5 is 1"]
    Val1 = 1,
}
impl From<TrigIn5Val> for bool {
    #[inline(always)]
    fn from(variant: TrigIn5Val) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TRIG_IN5_VAL` reader - TRIG_IN value"]
pub type TrigIn5ValR = crate::BitReader<TrigIn5Val>;
impl TrigIn5ValR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> TrigIn5Val {
        match self.bits {
            false => TrigIn5Val::Val0,
            true => TrigIn5Val::Val1,
        }
    }
    #[doc = "TRIG_IN5 is 0"]
    #[inline(always)]
    pub fn is_val0(&self) -> bool {
        *self == TrigIn5Val::Val0
    }
    #[doc = "TRIG_IN5 is 1"]
    #[inline(always)]
    pub fn is_val1(&self) -> bool {
        *self == TrigIn5Val::Val1
    }
}
#[doc = "TRIG_IN value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrigIn6Val {
    #[doc = "0: TRIG_IN6 is 0"]
    Val0 = 0,
    #[doc = "1: TRIG_IN6 is 1"]
    Val1 = 1,
}
impl From<TrigIn6Val> for bool {
    #[inline(always)]
    fn from(variant: TrigIn6Val) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TRIG_IN6_VAL` reader - TRIG_IN value"]
pub type TrigIn6ValR = crate::BitReader<TrigIn6Val>;
impl TrigIn6ValR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> TrigIn6Val {
        match self.bits {
            false => TrigIn6Val::Val0,
            true => TrigIn6Val::Val1,
        }
    }
    #[doc = "TRIG_IN6 is 0"]
    #[inline(always)]
    pub fn is_val0(&self) -> bool {
        *self == TrigIn6Val::Val0
    }
    #[doc = "TRIG_IN6 is 1"]
    #[inline(always)]
    pub fn is_val1(&self) -> bool {
        *self == TrigIn6Val::Val1
    }
}
#[doc = "TRIG_IN value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrigIn7Val {
    #[doc = "0: TRIG_IN7 is 0"]
    Val0 = 0,
    #[doc = "1: TRIG_IN7 is 1"]
    Val1 = 1,
}
impl From<TrigIn7Val> for bool {
    #[inline(always)]
    fn from(variant: TrigIn7Val) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TRIG_IN7_VAL` reader - TRIG_IN value"]
pub type TrigIn7ValR = crate::BitReader<TrigIn7Val>;
impl TrigIn7ValR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> TrigIn7Val {
        match self.bits {
            false => TrigIn7Val::Val0,
            true => TrigIn7Val::Val1,
        }
    }
    #[doc = "TRIG_IN7 is 0"]
    #[inline(always)]
    pub fn is_val0(&self) -> bool {
        *self == TrigIn7Val::Val0
    }
    #[doc = "TRIG_IN7 is 1"]
    #[inline(always)]
    pub fn is_val1(&self) -> bool {
        *self == TrigIn7Val::Val1
    }
}
#[doc = "TRIG_IN value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrigIn8Val {
    #[doc = "0: TRIG_IN8 is 0"]
    Val0 = 0,
    #[doc = "1: TRIG_IN8 is 1"]
    Val1 = 1,
}
impl From<TrigIn8Val> for bool {
    #[inline(always)]
    fn from(variant: TrigIn8Val) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TRIG_IN8_VAL` reader - TRIG_IN value"]
pub type TrigIn8ValR = crate::BitReader<TrigIn8Val>;
impl TrigIn8ValR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> TrigIn8Val {
        match self.bits {
            false => TrigIn8Val::Val0,
            true => TrigIn8Val::Val1,
        }
    }
    #[doc = "TRIG_IN8 is 0"]
    #[inline(always)]
    pub fn is_val0(&self) -> bool {
        *self == TrigIn8Val::Val0
    }
    #[doc = "TRIG_IN8 is 1"]
    #[inline(always)]
    pub fn is_val1(&self) -> bool {
        *self == TrigIn8Val::Val1
    }
}
#[doc = "TRIG_IN value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrigIn9Val {
    #[doc = "0: TRIG_IN9 is 0"]
    Val0 = 0,
    #[doc = "1: TRIG_IN9 is 1"]
    Val1 = 1,
}
impl From<TrigIn9Val> for bool {
    #[inline(always)]
    fn from(variant: TrigIn9Val) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TRIG_IN9_VAL` reader - TRIG_IN value"]
pub type TrigIn9ValR = crate::BitReader<TrigIn9Val>;
impl TrigIn9ValR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> TrigIn9Val {
        match self.bits {
            false => TrigIn9Val::Val0,
            true => TrigIn9Val::Val1,
        }
    }
    #[doc = "TRIG_IN9 is 0"]
    #[inline(always)]
    pub fn is_val0(&self) -> bool {
        *self == TrigIn9Val::Val0
    }
    #[doc = "TRIG_IN9 is 1"]
    #[inline(always)]
    pub fn is_val1(&self) -> bool {
        *self == TrigIn9Val::Val1
    }
}
#[doc = "TRIG_IN value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrigIn10Val {
    #[doc = "0: TRIG_IN10 is 0"]
    Val0 = 0,
    #[doc = "1: TRIG_IN10 is 1"]
    Val1 = 1,
}
impl From<TrigIn10Val> for bool {
    #[inline(always)]
    fn from(variant: TrigIn10Val) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TRIG_IN10_VAL` reader - TRIG_IN value"]
pub type TrigIn10ValR = crate::BitReader<TrigIn10Val>;
impl TrigIn10ValR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> TrigIn10Val {
        match self.bits {
            false => TrigIn10Val::Val0,
            true => TrigIn10Val::Val1,
        }
    }
    #[doc = "TRIG_IN10 is 0"]
    #[inline(always)]
    pub fn is_val0(&self) -> bool {
        *self == TrigIn10Val::Val0
    }
    #[doc = "TRIG_IN10 is 1"]
    #[inline(always)]
    pub fn is_val1(&self) -> bool {
        *self == TrigIn10Val::Val1
    }
}
#[doc = "TRIG_IN value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrigIn11Val {
    #[doc = "0: TRIG_IN11 is 0"]
    Val0 = 0,
    #[doc = "1: TRIG_IN11 is 1"]
    Val1 = 1,
}
impl From<TrigIn11Val> for bool {
    #[inline(always)]
    fn from(variant: TrigIn11Val) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TRIG_IN11_VAL` reader - TRIG_IN value"]
pub type TrigIn11ValR = crate::BitReader<TrigIn11Val>;
impl TrigIn11ValR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> TrigIn11Val {
        match self.bits {
            false => TrigIn11Val::Val0,
            true => TrigIn11Val::Val1,
        }
    }
    #[doc = "TRIG_IN11 is 0"]
    #[inline(always)]
    pub fn is_val0(&self) -> bool {
        *self == TrigIn11Val::Val0
    }
    #[doc = "TRIG_IN11 is 1"]
    #[inline(always)]
    pub fn is_val1(&self) -> bool {
        *self == TrigIn11Val::Val1
    }
}
impl R {
    #[doc = "Bit 0 - TRIG_IN value"]
    #[inline(always)]
    pub fn trig_in0_val(&self) -> TrigIn0ValR {
        TrigIn0ValR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - TRIG_IN value"]
    #[inline(always)]
    pub fn trig_in1_val(&self) -> TrigIn1ValR {
        TrigIn1ValR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - TRIG_IN value"]
    #[inline(always)]
    pub fn trig_in2_val(&self) -> TrigIn2ValR {
        TrigIn2ValR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - TRIG_IN value"]
    #[inline(always)]
    pub fn trig_in3_val(&self) -> TrigIn3ValR {
        TrigIn3ValR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - TRIG_IN value"]
    #[inline(always)]
    pub fn trig_in4_val(&self) -> TrigIn4ValR {
        TrigIn4ValR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - TRIG_IN value"]
    #[inline(always)]
    pub fn trig_in5_val(&self) -> TrigIn5ValR {
        TrigIn5ValR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - TRIG_IN value"]
    #[inline(always)]
    pub fn trig_in6_val(&self) -> TrigIn6ValR {
        TrigIn6ValR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - TRIG_IN value"]
    #[inline(always)]
    pub fn trig_in7_val(&self) -> TrigIn7ValR {
        TrigIn7ValR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - TRIG_IN value"]
    #[inline(always)]
    pub fn trig_in8_val(&self) -> TrigIn8ValR {
        TrigIn8ValR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - TRIG_IN value"]
    #[inline(always)]
    pub fn trig_in9_val(&self) -> TrigIn9ValR {
        TrigIn9ValR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - TRIG_IN value"]
    #[inline(always)]
    pub fn trig_in10_val(&self) -> TrigIn10ValR {
        TrigIn10ValR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - TRIG_IN value"]
    #[inline(always)]
    pub fn trig_in11_val(&self) -> TrigIn11ValR {
        TrigIn11ValR::new(((self.bits >> 11) & 1) != 0)
    }
}
#[doc = "Trigger filter stat\n\nYou can [`read`](crate::Reg::read) this register and get [`trigfil_stat0::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TrigfilStat0Spec;
impl crate::RegisterSpec for TrigfilStat0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`trigfil_stat0::R`](R) reader structure"]
impl crate::Readable for TrigfilStat0Spec {}
#[doc = "`reset()` method sets TRIGFIL_STAT0 to value 0"]
impl crate::Resettable for TrigfilStat0Spec {}

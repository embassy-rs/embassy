#[doc = "Register `MCR` reader"]
pub type R = crate::R<McrSpec>;
#[doc = "Register `MCR` writer"]
pub type W = crate::W<McrSpec>;
#[doc = "Interrupt on MR0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mr0i {
    #[doc = "0: Does not generate"]
    Mr0i0 = 0,
    #[doc = "1: Generates"]
    Mr0i1 = 1,
}
impl From<Mr0i> for bool {
    #[inline(always)]
    fn from(variant: Mr0i) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MR0I` reader - Interrupt on MR0"]
pub type Mr0iR = crate::BitReader<Mr0i>;
impl Mr0iR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mr0i {
        match self.bits {
            false => Mr0i::Mr0i0,
            true => Mr0i::Mr0i1,
        }
    }
    #[doc = "Does not generate"]
    #[inline(always)]
    pub fn is_mr0i_0(&self) -> bool {
        *self == Mr0i::Mr0i0
    }
    #[doc = "Generates"]
    #[inline(always)]
    pub fn is_mr0i_1(&self) -> bool {
        *self == Mr0i::Mr0i1
    }
}
#[doc = "Field `MR0I` writer - Interrupt on MR0"]
pub type Mr0iW<'a, REG> = crate::BitWriter<'a, REG, Mr0i>;
impl<'a, REG> Mr0iW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not generate"]
    #[inline(always)]
    pub fn mr0i_0(self) -> &'a mut crate::W<REG> {
        self.variant(Mr0i::Mr0i0)
    }
    #[doc = "Generates"]
    #[inline(always)]
    pub fn mr0i_1(self) -> &'a mut crate::W<REG> {
        self.variant(Mr0i::Mr0i1)
    }
}
#[doc = "Reset on MR0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mr0r {
    #[doc = "0: Does not reset"]
    Mr0r0 = 0,
    #[doc = "1: Resets"]
    Mr0r1 = 1,
}
impl From<Mr0r> for bool {
    #[inline(always)]
    fn from(variant: Mr0r) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MR0R` reader - Reset on MR0"]
pub type Mr0rR = crate::BitReader<Mr0r>;
impl Mr0rR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mr0r {
        match self.bits {
            false => Mr0r::Mr0r0,
            true => Mr0r::Mr0r1,
        }
    }
    #[doc = "Does not reset"]
    #[inline(always)]
    pub fn is_mr0r_0(&self) -> bool {
        *self == Mr0r::Mr0r0
    }
    #[doc = "Resets"]
    #[inline(always)]
    pub fn is_mr0r_1(&self) -> bool {
        *self == Mr0r::Mr0r1
    }
}
#[doc = "Field `MR0R` writer - Reset on MR0"]
pub type Mr0rW<'a, REG> = crate::BitWriter<'a, REG, Mr0r>;
impl<'a, REG> Mr0rW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not reset"]
    #[inline(always)]
    pub fn mr0r_0(self) -> &'a mut crate::W<REG> {
        self.variant(Mr0r::Mr0r0)
    }
    #[doc = "Resets"]
    #[inline(always)]
    pub fn mr0r_1(self) -> &'a mut crate::W<REG> {
        self.variant(Mr0r::Mr0r1)
    }
}
#[doc = "Stop on MR0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mr0s {
    #[doc = "0: Does not stop"]
    Mr0s0 = 0,
    #[doc = "1: Stops"]
    Mr0s1 = 1,
}
impl From<Mr0s> for bool {
    #[inline(always)]
    fn from(variant: Mr0s) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MR0S` reader - Stop on MR0"]
pub type Mr0sR = crate::BitReader<Mr0s>;
impl Mr0sR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mr0s {
        match self.bits {
            false => Mr0s::Mr0s0,
            true => Mr0s::Mr0s1,
        }
    }
    #[doc = "Does not stop"]
    #[inline(always)]
    pub fn is_mr0s_0(&self) -> bool {
        *self == Mr0s::Mr0s0
    }
    #[doc = "Stops"]
    #[inline(always)]
    pub fn is_mr0s_1(&self) -> bool {
        *self == Mr0s::Mr0s1
    }
}
#[doc = "Field `MR0S` writer - Stop on MR0"]
pub type Mr0sW<'a, REG> = crate::BitWriter<'a, REG, Mr0s>;
impl<'a, REG> Mr0sW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not stop"]
    #[inline(always)]
    pub fn mr0s_0(self) -> &'a mut crate::W<REG> {
        self.variant(Mr0s::Mr0s0)
    }
    #[doc = "Stops"]
    #[inline(always)]
    pub fn mr0s_1(self) -> &'a mut crate::W<REG> {
        self.variant(Mr0s::Mr0s1)
    }
}
#[doc = "Interrupt on MR1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mr1i {
    #[doc = "0: Does not generate"]
    Mr1i0 = 0,
    #[doc = "1: Generates"]
    Mr1i1 = 1,
}
impl From<Mr1i> for bool {
    #[inline(always)]
    fn from(variant: Mr1i) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MR1I` reader - Interrupt on MR1"]
pub type Mr1iR = crate::BitReader<Mr1i>;
impl Mr1iR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mr1i {
        match self.bits {
            false => Mr1i::Mr1i0,
            true => Mr1i::Mr1i1,
        }
    }
    #[doc = "Does not generate"]
    #[inline(always)]
    pub fn is_mr1i_0(&self) -> bool {
        *self == Mr1i::Mr1i0
    }
    #[doc = "Generates"]
    #[inline(always)]
    pub fn is_mr1i_1(&self) -> bool {
        *self == Mr1i::Mr1i1
    }
}
#[doc = "Field `MR1I` writer - Interrupt on MR1"]
pub type Mr1iW<'a, REG> = crate::BitWriter<'a, REG, Mr1i>;
impl<'a, REG> Mr1iW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not generate"]
    #[inline(always)]
    pub fn mr1i_0(self) -> &'a mut crate::W<REG> {
        self.variant(Mr1i::Mr1i0)
    }
    #[doc = "Generates"]
    #[inline(always)]
    pub fn mr1i_1(self) -> &'a mut crate::W<REG> {
        self.variant(Mr1i::Mr1i1)
    }
}
#[doc = "Reset on MR1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mr1r {
    #[doc = "0: Does not reset"]
    Mr1r0 = 0,
    #[doc = "1: Resets"]
    Mr1r1 = 1,
}
impl From<Mr1r> for bool {
    #[inline(always)]
    fn from(variant: Mr1r) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MR1R` reader - Reset on MR1"]
pub type Mr1rR = crate::BitReader<Mr1r>;
impl Mr1rR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mr1r {
        match self.bits {
            false => Mr1r::Mr1r0,
            true => Mr1r::Mr1r1,
        }
    }
    #[doc = "Does not reset"]
    #[inline(always)]
    pub fn is_mr1r_0(&self) -> bool {
        *self == Mr1r::Mr1r0
    }
    #[doc = "Resets"]
    #[inline(always)]
    pub fn is_mr1r_1(&self) -> bool {
        *self == Mr1r::Mr1r1
    }
}
#[doc = "Field `MR1R` writer - Reset on MR1"]
pub type Mr1rW<'a, REG> = crate::BitWriter<'a, REG, Mr1r>;
impl<'a, REG> Mr1rW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not reset"]
    #[inline(always)]
    pub fn mr1r_0(self) -> &'a mut crate::W<REG> {
        self.variant(Mr1r::Mr1r0)
    }
    #[doc = "Resets"]
    #[inline(always)]
    pub fn mr1r_1(self) -> &'a mut crate::W<REG> {
        self.variant(Mr1r::Mr1r1)
    }
}
#[doc = "Stop on MR1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mr1s {
    #[doc = "0: Does not stop"]
    Mris0 = 0,
    #[doc = "1: Stops"]
    Mris1 = 1,
}
impl From<Mr1s> for bool {
    #[inline(always)]
    fn from(variant: Mr1s) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MR1S` reader - Stop on MR1"]
pub type Mr1sR = crate::BitReader<Mr1s>;
impl Mr1sR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mr1s {
        match self.bits {
            false => Mr1s::Mris0,
            true => Mr1s::Mris1,
        }
    }
    #[doc = "Does not stop"]
    #[inline(always)]
    pub fn is_mris_0(&self) -> bool {
        *self == Mr1s::Mris0
    }
    #[doc = "Stops"]
    #[inline(always)]
    pub fn is_mris_1(&self) -> bool {
        *self == Mr1s::Mris1
    }
}
#[doc = "Field `MR1S` writer - Stop on MR1"]
pub type Mr1sW<'a, REG> = crate::BitWriter<'a, REG, Mr1s>;
impl<'a, REG> Mr1sW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not stop"]
    #[inline(always)]
    pub fn mris_0(self) -> &'a mut crate::W<REG> {
        self.variant(Mr1s::Mris0)
    }
    #[doc = "Stops"]
    #[inline(always)]
    pub fn mris_1(self) -> &'a mut crate::W<REG> {
        self.variant(Mr1s::Mris1)
    }
}
#[doc = "Interrupt on MR2\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mr2i {
    #[doc = "0: Does not generate"]
    Mr2i0 = 0,
    #[doc = "1: Generates"]
    Mr2i1 = 1,
}
impl From<Mr2i> for bool {
    #[inline(always)]
    fn from(variant: Mr2i) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MR2I` reader - Interrupt on MR2"]
pub type Mr2iR = crate::BitReader<Mr2i>;
impl Mr2iR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mr2i {
        match self.bits {
            false => Mr2i::Mr2i0,
            true => Mr2i::Mr2i1,
        }
    }
    #[doc = "Does not generate"]
    #[inline(always)]
    pub fn is_mr2i_0(&self) -> bool {
        *self == Mr2i::Mr2i0
    }
    #[doc = "Generates"]
    #[inline(always)]
    pub fn is_mr2i_1(&self) -> bool {
        *self == Mr2i::Mr2i1
    }
}
#[doc = "Field `MR2I` writer - Interrupt on MR2"]
pub type Mr2iW<'a, REG> = crate::BitWriter<'a, REG, Mr2i>;
impl<'a, REG> Mr2iW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not generate"]
    #[inline(always)]
    pub fn mr2i_0(self) -> &'a mut crate::W<REG> {
        self.variant(Mr2i::Mr2i0)
    }
    #[doc = "Generates"]
    #[inline(always)]
    pub fn mr2i_1(self) -> &'a mut crate::W<REG> {
        self.variant(Mr2i::Mr2i1)
    }
}
#[doc = "Reset on MR2\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mr2r {
    #[doc = "0: Does not reset"]
    Mr2r0 = 0,
    #[doc = "1: Resets"]
    Mr2r1 = 1,
}
impl From<Mr2r> for bool {
    #[inline(always)]
    fn from(variant: Mr2r) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MR2R` reader - Reset on MR2"]
pub type Mr2rR = crate::BitReader<Mr2r>;
impl Mr2rR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mr2r {
        match self.bits {
            false => Mr2r::Mr2r0,
            true => Mr2r::Mr2r1,
        }
    }
    #[doc = "Does not reset"]
    #[inline(always)]
    pub fn is_mr2r_0(&self) -> bool {
        *self == Mr2r::Mr2r0
    }
    #[doc = "Resets"]
    #[inline(always)]
    pub fn is_mr2r_1(&self) -> bool {
        *self == Mr2r::Mr2r1
    }
}
#[doc = "Field `MR2R` writer - Reset on MR2"]
pub type Mr2rW<'a, REG> = crate::BitWriter<'a, REG, Mr2r>;
impl<'a, REG> Mr2rW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not reset"]
    #[inline(always)]
    pub fn mr2r_0(self) -> &'a mut crate::W<REG> {
        self.variant(Mr2r::Mr2r0)
    }
    #[doc = "Resets"]
    #[inline(always)]
    pub fn mr2r_1(self) -> &'a mut crate::W<REG> {
        self.variant(Mr2r::Mr2r1)
    }
}
#[doc = "Stop on MR2\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mr2s {
    #[doc = "0: Does not stop"]
    Mr2s0 = 0,
    #[doc = "1: Stops"]
    Mr2s1 = 1,
}
impl From<Mr2s> for bool {
    #[inline(always)]
    fn from(variant: Mr2s) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MR2S` reader - Stop on MR2"]
pub type Mr2sR = crate::BitReader<Mr2s>;
impl Mr2sR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mr2s {
        match self.bits {
            false => Mr2s::Mr2s0,
            true => Mr2s::Mr2s1,
        }
    }
    #[doc = "Does not stop"]
    #[inline(always)]
    pub fn is_mr2s_0(&self) -> bool {
        *self == Mr2s::Mr2s0
    }
    #[doc = "Stops"]
    #[inline(always)]
    pub fn is_mr2s_1(&self) -> bool {
        *self == Mr2s::Mr2s1
    }
}
#[doc = "Field `MR2S` writer - Stop on MR2"]
pub type Mr2sW<'a, REG> = crate::BitWriter<'a, REG, Mr2s>;
impl<'a, REG> Mr2sW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not stop"]
    #[inline(always)]
    pub fn mr2s_0(self) -> &'a mut crate::W<REG> {
        self.variant(Mr2s::Mr2s0)
    }
    #[doc = "Stops"]
    #[inline(always)]
    pub fn mr2s_1(self) -> &'a mut crate::W<REG> {
        self.variant(Mr2s::Mr2s1)
    }
}
#[doc = "Interrupt on MR3\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mr3i {
    #[doc = "0: Does not generate"]
    Mr3i0 = 0,
    #[doc = "1: Generates"]
    Mr3i1 = 1,
}
impl From<Mr3i> for bool {
    #[inline(always)]
    fn from(variant: Mr3i) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MR3I` reader - Interrupt on MR3"]
pub type Mr3iR = crate::BitReader<Mr3i>;
impl Mr3iR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mr3i {
        match self.bits {
            false => Mr3i::Mr3i0,
            true => Mr3i::Mr3i1,
        }
    }
    #[doc = "Does not generate"]
    #[inline(always)]
    pub fn is_mr3i_0(&self) -> bool {
        *self == Mr3i::Mr3i0
    }
    #[doc = "Generates"]
    #[inline(always)]
    pub fn is_mr3i_1(&self) -> bool {
        *self == Mr3i::Mr3i1
    }
}
#[doc = "Field `MR3I` writer - Interrupt on MR3"]
pub type Mr3iW<'a, REG> = crate::BitWriter<'a, REG, Mr3i>;
impl<'a, REG> Mr3iW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not generate"]
    #[inline(always)]
    pub fn mr3i_0(self) -> &'a mut crate::W<REG> {
        self.variant(Mr3i::Mr3i0)
    }
    #[doc = "Generates"]
    #[inline(always)]
    pub fn mr3i_1(self) -> &'a mut crate::W<REG> {
        self.variant(Mr3i::Mr3i1)
    }
}
#[doc = "Reset on MR3\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mr3r {
    #[doc = "0: Does not reset"]
    Mr3r0 = 0,
    #[doc = "1: Resets"]
    Mr3r1 = 1,
}
impl From<Mr3r> for bool {
    #[inline(always)]
    fn from(variant: Mr3r) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MR3R` reader - Reset on MR3"]
pub type Mr3rR = crate::BitReader<Mr3r>;
impl Mr3rR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mr3r {
        match self.bits {
            false => Mr3r::Mr3r0,
            true => Mr3r::Mr3r1,
        }
    }
    #[doc = "Does not reset"]
    #[inline(always)]
    pub fn is_mr3r_0(&self) -> bool {
        *self == Mr3r::Mr3r0
    }
    #[doc = "Resets"]
    #[inline(always)]
    pub fn is_mr3r_1(&self) -> bool {
        *self == Mr3r::Mr3r1
    }
}
#[doc = "Field `MR3R` writer - Reset on MR3"]
pub type Mr3rW<'a, REG> = crate::BitWriter<'a, REG, Mr3r>;
impl<'a, REG> Mr3rW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not reset"]
    #[inline(always)]
    pub fn mr3r_0(self) -> &'a mut crate::W<REG> {
        self.variant(Mr3r::Mr3r0)
    }
    #[doc = "Resets"]
    #[inline(always)]
    pub fn mr3r_1(self) -> &'a mut crate::W<REG> {
        self.variant(Mr3r::Mr3r1)
    }
}
#[doc = "Stop on MR3\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mr3s {
    #[doc = "0: Does not stop"]
    Mr3s0 = 0,
    #[doc = "1: Stops"]
    Mr3s1 = 1,
}
impl From<Mr3s> for bool {
    #[inline(always)]
    fn from(variant: Mr3s) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MR3S` reader - Stop on MR3"]
pub type Mr3sR = crate::BitReader<Mr3s>;
impl Mr3sR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mr3s {
        match self.bits {
            false => Mr3s::Mr3s0,
            true => Mr3s::Mr3s1,
        }
    }
    #[doc = "Does not stop"]
    #[inline(always)]
    pub fn is_mr3s_0(&self) -> bool {
        *self == Mr3s::Mr3s0
    }
    #[doc = "Stops"]
    #[inline(always)]
    pub fn is_mr3s_1(&self) -> bool {
        *self == Mr3s::Mr3s1
    }
}
#[doc = "Field `MR3S` writer - Stop on MR3"]
pub type Mr3sW<'a, REG> = crate::BitWriter<'a, REG, Mr3s>;
impl<'a, REG> Mr3sW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not stop"]
    #[inline(always)]
    pub fn mr3s_0(self) -> &'a mut crate::W<REG> {
        self.variant(Mr3s::Mr3s0)
    }
    #[doc = "Stops"]
    #[inline(always)]
    pub fn mr3s_1(self) -> &'a mut crate::W<REG> {
        self.variant(Mr3s::Mr3s1)
    }
}
#[doc = "Reload MR\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mr0rl {
    #[doc = "0: Does not reload"]
    Mr0rl0 = 0,
    #[doc = "1: Reloads"]
    Mr0rl1 = 1,
}
impl From<Mr0rl> for bool {
    #[inline(always)]
    fn from(variant: Mr0rl) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MR0RL` reader - Reload MR"]
pub type Mr0rlR = crate::BitReader<Mr0rl>;
impl Mr0rlR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mr0rl {
        match self.bits {
            false => Mr0rl::Mr0rl0,
            true => Mr0rl::Mr0rl1,
        }
    }
    #[doc = "Does not reload"]
    #[inline(always)]
    pub fn is_mr0rl_0(&self) -> bool {
        *self == Mr0rl::Mr0rl0
    }
    #[doc = "Reloads"]
    #[inline(always)]
    pub fn is_mr0rl_1(&self) -> bool {
        *self == Mr0rl::Mr0rl1
    }
}
#[doc = "Field `MR0RL` writer - Reload MR"]
pub type Mr0rlW<'a, REG> = crate::BitWriter<'a, REG, Mr0rl>;
impl<'a, REG> Mr0rlW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not reload"]
    #[inline(always)]
    pub fn mr0rl_0(self) -> &'a mut crate::W<REG> {
        self.variant(Mr0rl::Mr0rl0)
    }
    #[doc = "Reloads"]
    #[inline(always)]
    pub fn mr0rl_1(self) -> &'a mut crate::W<REG> {
        self.variant(Mr0rl::Mr0rl1)
    }
}
#[doc = "Reload MR\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mr1rl {
    #[doc = "0: Does not reload"]
    Mr1rl0 = 0,
    #[doc = "1: Reloads"]
    Mr1rl1 = 1,
}
impl From<Mr1rl> for bool {
    #[inline(always)]
    fn from(variant: Mr1rl) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MR1RL` reader - Reload MR"]
pub type Mr1rlR = crate::BitReader<Mr1rl>;
impl Mr1rlR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mr1rl {
        match self.bits {
            false => Mr1rl::Mr1rl0,
            true => Mr1rl::Mr1rl1,
        }
    }
    #[doc = "Does not reload"]
    #[inline(always)]
    pub fn is_mr1rl_0(&self) -> bool {
        *self == Mr1rl::Mr1rl0
    }
    #[doc = "Reloads"]
    #[inline(always)]
    pub fn is_mr1rl_1(&self) -> bool {
        *self == Mr1rl::Mr1rl1
    }
}
#[doc = "Field `MR1RL` writer - Reload MR"]
pub type Mr1rlW<'a, REG> = crate::BitWriter<'a, REG, Mr1rl>;
impl<'a, REG> Mr1rlW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not reload"]
    #[inline(always)]
    pub fn mr1rl_0(self) -> &'a mut crate::W<REG> {
        self.variant(Mr1rl::Mr1rl0)
    }
    #[doc = "Reloads"]
    #[inline(always)]
    pub fn mr1rl_1(self) -> &'a mut crate::W<REG> {
        self.variant(Mr1rl::Mr1rl1)
    }
}
#[doc = "Reload MR\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mr2rl {
    #[doc = "0: Does not reload"]
    Mr2rl0 = 0,
    #[doc = "1: Reloads"]
    Mr2rl1 = 1,
}
impl From<Mr2rl> for bool {
    #[inline(always)]
    fn from(variant: Mr2rl) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MR2RL` reader - Reload MR"]
pub type Mr2rlR = crate::BitReader<Mr2rl>;
impl Mr2rlR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mr2rl {
        match self.bits {
            false => Mr2rl::Mr2rl0,
            true => Mr2rl::Mr2rl1,
        }
    }
    #[doc = "Does not reload"]
    #[inline(always)]
    pub fn is_mr2rl_0(&self) -> bool {
        *self == Mr2rl::Mr2rl0
    }
    #[doc = "Reloads"]
    #[inline(always)]
    pub fn is_mr2rl_1(&self) -> bool {
        *self == Mr2rl::Mr2rl1
    }
}
#[doc = "Field `MR2RL` writer - Reload MR"]
pub type Mr2rlW<'a, REG> = crate::BitWriter<'a, REG, Mr2rl>;
impl<'a, REG> Mr2rlW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not reload"]
    #[inline(always)]
    pub fn mr2rl_0(self) -> &'a mut crate::W<REG> {
        self.variant(Mr2rl::Mr2rl0)
    }
    #[doc = "Reloads"]
    #[inline(always)]
    pub fn mr2rl_1(self) -> &'a mut crate::W<REG> {
        self.variant(Mr2rl::Mr2rl1)
    }
}
#[doc = "Reload MR\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mr3rl {
    #[doc = "0: Does not reload"]
    Mr3rl0 = 0,
    #[doc = "1: Reloads"]
    Mr3rl1 = 1,
}
impl From<Mr3rl> for bool {
    #[inline(always)]
    fn from(variant: Mr3rl) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MR3RL` reader - Reload MR"]
pub type Mr3rlR = crate::BitReader<Mr3rl>;
impl Mr3rlR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mr3rl {
        match self.bits {
            false => Mr3rl::Mr3rl0,
            true => Mr3rl::Mr3rl1,
        }
    }
    #[doc = "Does not reload"]
    #[inline(always)]
    pub fn is_mr3rl_0(&self) -> bool {
        *self == Mr3rl::Mr3rl0
    }
    #[doc = "Reloads"]
    #[inline(always)]
    pub fn is_mr3rl_1(&self) -> bool {
        *self == Mr3rl::Mr3rl1
    }
}
#[doc = "Field `MR3RL` writer - Reload MR"]
pub type Mr3rlW<'a, REG> = crate::BitWriter<'a, REG, Mr3rl>;
impl<'a, REG> Mr3rlW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not reload"]
    #[inline(always)]
    pub fn mr3rl_0(self) -> &'a mut crate::W<REG> {
        self.variant(Mr3rl::Mr3rl0)
    }
    #[doc = "Reloads"]
    #[inline(always)]
    pub fn mr3rl_1(self) -> &'a mut crate::W<REG> {
        self.variant(Mr3rl::Mr3rl1)
    }
}
impl R {
    #[doc = "Bit 0 - Interrupt on MR0"]
    #[inline(always)]
    pub fn mr0i(&self) -> Mr0iR {
        Mr0iR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Reset on MR0"]
    #[inline(always)]
    pub fn mr0r(&self) -> Mr0rR {
        Mr0rR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Stop on MR0"]
    #[inline(always)]
    pub fn mr0s(&self) -> Mr0sR {
        Mr0sR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Interrupt on MR1"]
    #[inline(always)]
    pub fn mr1i(&self) -> Mr1iR {
        Mr1iR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Reset on MR1"]
    #[inline(always)]
    pub fn mr1r(&self) -> Mr1rR {
        Mr1rR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Stop on MR1"]
    #[inline(always)]
    pub fn mr1s(&self) -> Mr1sR {
        Mr1sR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Interrupt on MR2"]
    #[inline(always)]
    pub fn mr2i(&self) -> Mr2iR {
        Mr2iR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Reset on MR2"]
    #[inline(always)]
    pub fn mr2r(&self) -> Mr2rR {
        Mr2rR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Stop on MR2"]
    #[inline(always)]
    pub fn mr2s(&self) -> Mr2sR {
        Mr2sR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Interrupt on MR3"]
    #[inline(always)]
    pub fn mr3i(&self) -> Mr3iR {
        Mr3iR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Reset on MR3"]
    #[inline(always)]
    pub fn mr3r(&self) -> Mr3rR {
        Mr3rR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Stop on MR3"]
    #[inline(always)]
    pub fn mr3s(&self) -> Mr3sR {
        Mr3sR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 24 - Reload MR"]
    #[inline(always)]
    pub fn mr0rl(&self) -> Mr0rlR {
        Mr0rlR::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - Reload MR"]
    #[inline(always)]
    pub fn mr1rl(&self) -> Mr1rlR {
        Mr1rlR::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - Reload MR"]
    #[inline(always)]
    pub fn mr2rl(&self) -> Mr2rlR {
        Mr2rlR::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - Reload MR"]
    #[inline(always)]
    pub fn mr3rl(&self) -> Mr3rlR {
        Mr3rlR::new(((self.bits >> 27) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Interrupt on MR0"]
    #[inline(always)]
    pub fn mr0i(&mut self) -> Mr0iW<McrSpec> {
        Mr0iW::new(self, 0)
    }
    #[doc = "Bit 1 - Reset on MR0"]
    #[inline(always)]
    pub fn mr0r(&mut self) -> Mr0rW<McrSpec> {
        Mr0rW::new(self, 1)
    }
    #[doc = "Bit 2 - Stop on MR0"]
    #[inline(always)]
    pub fn mr0s(&mut self) -> Mr0sW<McrSpec> {
        Mr0sW::new(self, 2)
    }
    #[doc = "Bit 3 - Interrupt on MR1"]
    #[inline(always)]
    pub fn mr1i(&mut self) -> Mr1iW<McrSpec> {
        Mr1iW::new(self, 3)
    }
    #[doc = "Bit 4 - Reset on MR1"]
    #[inline(always)]
    pub fn mr1r(&mut self) -> Mr1rW<McrSpec> {
        Mr1rW::new(self, 4)
    }
    #[doc = "Bit 5 - Stop on MR1"]
    #[inline(always)]
    pub fn mr1s(&mut self) -> Mr1sW<McrSpec> {
        Mr1sW::new(self, 5)
    }
    #[doc = "Bit 6 - Interrupt on MR2"]
    #[inline(always)]
    pub fn mr2i(&mut self) -> Mr2iW<McrSpec> {
        Mr2iW::new(self, 6)
    }
    #[doc = "Bit 7 - Reset on MR2"]
    #[inline(always)]
    pub fn mr2r(&mut self) -> Mr2rW<McrSpec> {
        Mr2rW::new(self, 7)
    }
    #[doc = "Bit 8 - Stop on MR2"]
    #[inline(always)]
    pub fn mr2s(&mut self) -> Mr2sW<McrSpec> {
        Mr2sW::new(self, 8)
    }
    #[doc = "Bit 9 - Interrupt on MR3"]
    #[inline(always)]
    pub fn mr3i(&mut self) -> Mr3iW<McrSpec> {
        Mr3iW::new(self, 9)
    }
    #[doc = "Bit 10 - Reset on MR3"]
    #[inline(always)]
    pub fn mr3r(&mut self) -> Mr3rW<McrSpec> {
        Mr3rW::new(self, 10)
    }
    #[doc = "Bit 11 - Stop on MR3"]
    #[inline(always)]
    pub fn mr3s(&mut self) -> Mr3sW<McrSpec> {
        Mr3sW::new(self, 11)
    }
    #[doc = "Bit 24 - Reload MR"]
    #[inline(always)]
    pub fn mr0rl(&mut self) -> Mr0rlW<McrSpec> {
        Mr0rlW::new(self, 24)
    }
    #[doc = "Bit 25 - Reload MR"]
    #[inline(always)]
    pub fn mr1rl(&mut self) -> Mr1rlW<McrSpec> {
        Mr1rlW::new(self, 25)
    }
    #[doc = "Bit 26 - Reload MR"]
    #[inline(always)]
    pub fn mr2rl(&mut self) -> Mr2rlW<McrSpec> {
        Mr2rlW::new(self, 26)
    }
    #[doc = "Bit 27 - Reload MR"]
    #[inline(always)]
    pub fn mr3rl(&mut self) -> Mr3rlW<McrSpec> {
        Mr3rlW::new(self, 27)
    }
}
#[doc = "Match Control\n\nYou can [`read`](crate::Reg::read) this register and get [`mcr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mcr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct McrSpec;
impl crate::RegisterSpec for McrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mcr::R`](R) reader structure"]
impl crate::Readable for McrSpec {}
#[doc = "`write(|w| ..)` method takes [`mcr::W`](W) writer structure"]
impl crate::Writable for McrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MCR to value 0"]
impl crate::Resettable for McrSpec {}

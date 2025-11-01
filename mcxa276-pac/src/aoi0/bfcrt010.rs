#[doc = "Register `BFCRT010` reader"]
pub type R = crate::R<Bfcrt010Spec>;
#[doc = "Register `BFCRT010` writer"]
pub type W = crate::W<Bfcrt010Spec>;
#[doc = "Product Term 1, Input D Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pt1Dc {
    #[doc = "0: Force input D to become 0"]
    Force0 = 0,
    #[doc = "1: Pass input D"]
    Pass = 1,
    #[doc = "2: Complement input D"]
    Complement = 2,
    #[doc = "3: Force input D to become 1"]
    Force1 = 3,
}
impl From<Pt1Dc> for u8 {
    #[inline(always)]
    fn from(variant: Pt1Dc) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pt1Dc {
    type Ux = u8;
}
impl crate::IsEnum for Pt1Dc {}
#[doc = "Field `PT1_DC` reader - Product Term 1, Input D Configuration"]
pub type Pt1DcR = crate::FieldReader<Pt1Dc>;
impl Pt1DcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pt1Dc {
        match self.bits {
            0 => Pt1Dc::Force0,
            1 => Pt1Dc::Pass,
            2 => Pt1Dc::Complement,
            3 => Pt1Dc::Force1,
            _ => unreachable!(),
        }
    }
    #[doc = "Force input D to become 0"]
    #[inline(always)]
    pub fn is_force_0(&self) -> bool {
        *self == Pt1Dc::Force0
    }
    #[doc = "Pass input D"]
    #[inline(always)]
    pub fn is_pass(&self) -> bool {
        *self == Pt1Dc::Pass
    }
    #[doc = "Complement input D"]
    #[inline(always)]
    pub fn is_complement(&self) -> bool {
        *self == Pt1Dc::Complement
    }
    #[doc = "Force input D to become 1"]
    #[inline(always)]
    pub fn is_force_1(&self) -> bool {
        *self == Pt1Dc::Force1
    }
}
#[doc = "Field `PT1_DC` writer - Product Term 1, Input D Configuration"]
pub type Pt1DcW<'a, REG> = crate::FieldWriter<'a, REG, 2, Pt1Dc, crate::Safe>;
impl<'a, REG> Pt1DcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Force input D to become 0"]
    #[inline(always)]
    pub fn force_0(self) -> &'a mut crate::W<REG> {
        self.variant(Pt1Dc::Force0)
    }
    #[doc = "Pass input D"]
    #[inline(always)]
    pub fn pass(self) -> &'a mut crate::W<REG> {
        self.variant(Pt1Dc::Pass)
    }
    #[doc = "Complement input D"]
    #[inline(always)]
    pub fn complement(self) -> &'a mut crate::W<REG> {
        self.variant(Pt1Dc::Complement)
    }
    #[doc = "Force input D to become 1"]
    #[inline(always)]
    pub fn force_1(self) -> &'a mut crate::W<REG> {
        self.variant(Pt1Dc::Force1)
    }
}
#[doc = "Product Term 1, Input C Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pt1Cc {
    #[doc = "0: Force input C to become 0"]
    Force0 = 0,
    #[doc = "1: Pass input C"]
    Pass = 1,
    #[doc = "2: Complement input C"]
    Complement = 2,
    #[doc = "3: Force input C to become 1"]
    Force1 = 3,
}
impl From<Pt1Cc> for u8 {
    #[inline(always)]
    fn from(variant: Pt1Cc) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pt1Cc {
    type Ux = u8;
}
impl crate::IsEnum for Pt1Cc {}
#[doc = "Field `PT1_CC` reader - Product Term 1, Input C Configuration"]
pub type Pt1CcR = crate::FieldReader<Pt1Cc>;
impl Pt1CcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pt1Cc {
        match self.bits {
            0 => Pt1Cc::Force0,
            1 => Pt1Cc::Pass,
            2 => Pt1Cc::Complement,
            3 => Pt1Cc::Force1,
            _ => unreachable!(),
        }
    }
    #[doc = "Force input C to become 0"]
    #[inline(always)]
    pub fn is_force_0(&self) -> bool {
        *self == Pt1Cc::Force0
    }
    #[doc = "Pass input C"]
    #[inline(always)]
    pub fn is_pass(&self) -> bool {
        *self == Pt1Cc::Pass
    }
    #[doc = "Complement input C"]
    #[inline(always)]
    pub fn is_complement(&self) -> bool {
        *self == Pt1Cc::Complement
    }
    #[doc = "Force input C to become 1"]
    #[inline(always)]
    pub fn is_force_1(&self) -> bool {
        *self == Pt1Cc::Force1
    }
}
#[doc = "Field `PT1_CC` writer - Product Term 1, Input C Configuration"]
pub type Pt1CcW<'a, REG> = crate::FieldWriter<'a, REG, 2, Pt1Cc, crate::Safe>;
impl<'a, REG> Pt1CcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Force input C to become 0"]
    #[inline(always)]
    pub fn force_0(self) -> &'a mut crate::W<REG> {
        self.variant(Pt1Cc::Force0)
    }
    #[doc = "Pass input C"]
    #[inline(always)]
    pub fn pass(self) -> &'a mut crate::W<REG> {
        self.variant(Pt1Cc::Pass)
    }
    #[doc = "Complement input C"]
    #[inline(always)]
    pub fn complement(self) -> &'a mut crate::W<REG> {
        self.variant(Pt1Cc::Complement)
    }
    #[doc = "Force input C to become 1"]
    #[inline(always)]
    pub fn force_1(self) -> &'a mut crate::W<REG> {
        self.variant(Pt1Cc::Force1)
    }
}
#[doc = "Product Term 1, Input B Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pt1Bc {
    #[doc = "0: Force input B to become 0"]
    Force0 = 0,
    #[doc = "1: Pass input B"]
    Pass = 1,
    #[doc = "2: Complement input B"]
    Complement = 2,
    #[doc = "3: Force input B to become 1"]
    Force1 = 3,
}
impl From<Pt1Bc> for u8 {
    #[inline(always)]
    fn from(variant: Pt1Bc) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pt1Bc {
    type Ux = u8;
}
impl crate::IsEnum for Pt1Bc {}
#[doc = "Field `PT1_BC` reader - Product Term 1, Input B Configuration"]
pub type Pt1BcR = crate::FieldReader<Pt1Bc>;
impl Pt1BcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pt1Bc {
        match self.bits {
            0 => Pt1Bc::Force0,
            1 => Pt1Bc::Pass,
            2 => Pt1Bc::Complement,
            3 => Pt1Bc::Force1,
            _ => unreachable!(),
        }
    }
    #[doc = "Force input B to become 0"]
    #[inline(always)]
    pub fn is_force_0(&self) -> bool {
        *self == Pt1Bc::Force0
    }
    #[doc = "Pass input B"]
    #[inline(always)]
    pub fn is_pass(&self) -> bool {
        *self == Pt1Bc::Pass
    }
    #[doc = "Complement input B"]
    #[inline(always)]
    pub fn is_complement(&self) -> bool {
        *self == Pt1Bc::Complement
    }
    #[doc = "Force input B to become 1"]
    #[inline(always)]
    pub fn is_force_1(&self) -> bool {
        *self == Pt1Bc::Force1
    }
}
#[doc = "Field `PT1_BC` writer - Product Term 1, Input B Configuration"]
pub type Pt1BcW<'a, REG> = crate::FieldWriter<'a, REG, 2, Pt1Bc, crate::Safe>;
impl<'a, REG> Pt1BcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Force input B to become 0"]
    #[inline(always)]
    pub fn force_0(self) -> &'a mut crate::W<REG> {
        self.variant(Pt1Bc::Force0)
    }
    #[doc = "Pass input B"]
    #[inline(always)]
    pub fn pass(self) -> &'a mut crate::W<REG> {
        self.variant(Pt1Bc::Pass)
    }
    #[doc = "Complement input B"]
    #[inline(always)]
    pub fn complement(self) -> &'a mut crate::W<REG> {
        self.variant(Pt1Bc::Complement)
    }
    #[doc = "Force input B to become 1"]
    #[inline(always)]
    pub fn force_1(self) -> &'a mut crate::W<REG> {
        self.variant(Pt1Bc::Force1)
    }
}
#[doc = "Product Term 1, Input A Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pt1Ac {
    #[doc = "0: Force input A to become 0"]
    Force0 = 0,
    #[doc = "1: Pass input A"]
    Pass = 1,
    #[doc = "2: Complement input A"]
    Complement = 2,
    #[doc = "3: Force input A to become 1"]
    Force1 = 3,
}
impl From<Pt1Ac> for u8 {
    #[inline(always)]
    fn from(variant: Pt1Ac) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pt1Ac {
    type Ux = u8;
}
impl crate::IsEnum for Pt1Ac {}
#[doc = "Field `PT1_AC` reader - Product Term 1, Input A Configuration"]
pub type Pt1AcR = crate::FieldReader<Pt1Ac>;
impl Pt1AcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pt1Ac {
        match self.bits {
            0 => Pt1Ac::Force0,
            1 => Pt1Ac::Pass,
            2 => Pt1Ac::Complement,
            3 => Pt1Ac::Force1,
            _ => unreachable!(),
        }
    }
    #[doc = "Force input A to become 0"]
    #[inline(always)]
    pub fn is_force_0(&self) -> bool {
        *self == Pt1Ac::Force0
    }
    #[doc = "Pass input A"]
    #[inline(always)]
    pub fn is_pass(&self) -> bool {
        *self == Pt1Ac::Pass
    }
    #[doc = "Complement input A"]
    #[inline(always)]
    pub fn is_complement(&self) -> bool {
        *self == Pt1Ac::Complement
    }
    #[doc = "Force input A to become 1"]
    #[inline(always)]
    pub fn is_force_1(&self) -> bool {
        *self == Pt1Ac::Force1
    }
}
#[doc = "Field `PT1_AC` writer - Product Term 1, Input A Configuration"]
pub type Pt1AcW<'a, REG> = crate::FieldWriter<'a, REG, 2, Pt1Ac, crate::Safe>;
impl<'a, REG> Pt1AcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Force input A to become 0"]
    #[inline(always)]
    pub fn force_0(self) -> &'a mut crate::W<REG> {
        self.variant(Pt1Ac::Force0)
    }
    #[doc = "Pass input A"]
    #[inline(always)]
    pub fn pass(self) -> &'a mut crate::W<REG> {
        self.variant(Pt1Ac::Pass)
    }
    #[doc = "Complement input A"]
    #[inline(always)]
    pub fn complement(self) -> &'a mut crate::W<REG> {
        self.variant(Pt1Ac::Complement)
    }
    #[doc = "Force input A to become 1"]
    #[inline(always)]
    pub fn force_1(self) -> &'a mut crate::W<REG> {
        self.variant(Pt1Ac::Force1)
    }
}
#[doc = "Product Term 0, Input D Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pt0Dc {
    #[doc = "0: Force input D to become 0"]
    Force0 = 0,
    #[doc = "1: Pass input D"]
    Pass = 1,
    #[doc = "2: Complement input D"]
    Complement = 2,
    #[doc = "3: Force input D to become 1"]
    Force1 = 3,
}
impl From<Pt0Dc> for u8 {
    #[inline(always)]
    fn from(variant: Pt0Dc) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pt0Dc {
    type Ux = u8;
}
impl crate::IsEnum for Pt0Dc {}
#[doc = "Field `PT0_DC` reader - Product Term 0, Input D Configuration"]
pub type Pt0DcR = crate::FieldReader<Pt0Dc>;
impl Pt0DcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pt0Dc {
        match self.bits {
            0 => Pt0Dc::Force0,
            1 => Pt0Dc::Pass,
            2 => Pt0Dc::Complement,
            3 => Pt0Dc::Force1,
            _ => unreachable!(),
        }
    }
    #[doc = "Force input D to become 0"]
    #[inline(always)]
    pub fn is_force_0(&self) -> bool {
        *self == Pt0Dc::Force0
    }
    #[doc = "Pass input D"]
    #[inline(always)]
    pub fn is_pass(&self) -> bool {
        *self == Pt0Dc::Pass
    }
    #[doc = "Complement input D"]
    #[inline(always)]
    pub fn is_complement(&self) -> bool {
        *self == Pt0Dc::Complement
    }
    #[doc = "Force input D to become 1"]
    #[inline(always)]
    pub fn is_force_1(&self) -> bool {
        *self == Pt0Dc::Force1
    }
}
#[doc = "Field `PT0_DC` writer - Product Term 0, Input D Configuration"]
pub type Pt0DcW<'a, REG> = crate::FieldWriter<'a, REG, 2, Pt0Dc, crate::Safe>;
impl<'a, REG> Pt0DcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Force input D to become 0"]
    #[inline(always)]
    pub fn force_0(self) -> &'a mut crate::W<REG> {
        self.variant(Pt0Dc::Force0)
    }
    #[doc = "Pass input D"]
    #[inline(always)]
    pub fn pass(self) -> &'a mut crate::W<REG> {
        self.variant(Pt0Dc::Pass)
    }
    #[doc = "Complement input D"]
    #[inline(always)]
    pub fn complement(self) -> &'a mut crate::W<REG> {
        self.variant(Pt0Dc::Complement)
    }
    #[doc = "Force input D to become 1"]
    #[inline(always)]
    pub fn force_1(self) -> &'a mut crate::W<REG> {
        self.variant(Pt0Dc::Force1)
    }
}
#[doc = "Product Term 0, Input C Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pt0Cc {
    #[doc = "0: Force input C to become 0"]
    Force0 = 0,
    #[doc = "1: Pass input C"]
    Pass = 1,
    #[doc = "2: Complement input C"]
    Complement = 2,
    #[doc = "3: Force input C to become 1"]
    Force1 = 3,
}
impl From<Pt0Cc> for u8 {
    #[inline(always)]
    fn from(variant: Pt0Cc) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pt0Cc {
    type Ux = u8;
}
impl crate::IsEnum for Pt0Cc {}
#[doc = "Field `PT0_CC` reader - Product Term 0, Input C Configuration"]
pub type Pt0CcR = crate::FieldReader<Pt0Cc>;
impl Pt0CcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pt0Cc {
        match self.bits {
            0 => Pt0Cc::Force0,
            1 => Pt0Cc::Pass,
            2 => Pt0Cc::Complement,
            3 => Pt0Cc::Force1,
            _ => unreachable!(),
        }
    }
    #[doc = "Force input C to become 0"]
    #[inline(always)]
    pub fn is_force_0(&self) -> bool {
        *self == Pt0Cc::Force0
    }
    #[doc = "Pass input C"]
    #[inline(always)]
    pub fn is_pass(&self) -> bool {
        *self == Pt0Cc::Pass
    }
    #[doc = "Complement input C"]
    #[inline(always)]
    pub fn is_complement(&self) -> bool {
        *self == Pt0Cc::Complement
    }
    #[doc = "Force input C to become 1"]
    #[inline(always)]
    pub fn is_force_1(&self) -> bool {
        *self == Pt0Cc::Force1
    }
}
#[doc = "Field `PT0_CC` writer - Product Term 0, Input C Configuration"]
pub type Pt0CcW<'a, REG> = crate::FieldWriter<'a, REG, 2, Pt0Cc, crate::Safe>;
impl<'a, REG> Pt0CcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Force input C to become 0"]
    #[inline(always)]
    pub fn force_0(self) -> &'a mut crate::W<REG> {
        self.variant(Pt0Cc::Force0)
    }
    #[doc = "Pass input C"]
    #[inline(always)]
    pub fn pass(self) -> &'a mut crate::W<REG> {
        self.variant(Pt0Cc::Pass)
    }
    #[doc = "Complement input C"]
    #[inline(always)]
    pub fn complement(self) -> &'a mut crate::W<REG> {
        self.variant(Pt0Cc::Complement)
    }
    #[doc = "Force input C to become 1"]
    #[inline(always)]
    pub fn force_1(self) -> &'a mut crate::W<REG> {
        self.variant(Pt0Cc::Force1)
    }
}
#[doc = "Product Term 0, Input B Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pt0Bc {
    #[doc = "0: Force input B to become 0"]
    Force0 = 0,
    #[doc = "1: Pass input B"]
    Pass = 1,
    #[doc = "2: Complement input B"]
    Complement = 2,
    #[doc = "3: Force input B to become 1"]
    Force1 = 3,
}
impl From<Pt0Bc> for u8 {
    #[inline(always)]
    fn from(variant: Pt0Bc) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pt0Bc {
    type Ux = u8;
}
impl crate::IsEnum for Pt0Bc {}
#[doc = "Field `PT0_BC` reader - Product Term 0, Input B Configuration"]
pub type Pt0BcR = crate::FieldReader<Pt0Bc>;
impl Pt0BcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pt0Bc {
        match self.bits {
            0 => Pt0Bc::Force0,
            1 => Pt0Bc::Pass,
            2 => Pt0Bc::Complement,
            3 => Pt0Bc::Force1,
            _ => unreachable!(),
        }
    }
    #[doc = "Force input B to become 0"]
    #[inline(always)]
    pub fn is_force_0(&self) -> bool {
        *self == Pt0Bc::Force0
    }
    #[doc = "Pass input B"]
    #[inline(always)]
    pub fn is_pass(&self) -> bool {
        *self == Pt0Bc::Pass
    }
    #[doc = "Complement input B"]
    #[inline(always)]
    pub fn is_complement(&self) -> bool {
        *self == Pt0Bc::Complement
    }
    #[doc = "Force input B to become 1"]
    #[inline(always)]
    pub fn is_force_1(&self) -> bool {
        *self == Pt0Bc::Force1
    }
}
#[doc = "Field `PT0_BC` writer - Product Term 0, Input B Configuration"]
pub type Pt0BcW<'a, REG> = crate::FieldWriter<'a, REG, 2, Pt0Bc, crate::Safe>;
impl<'a, REG> Pt0BcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Force input B to become 0"]
    #[inline(always)]
    pub fn force_0(self) -> &'a mut crate::W<REG> {
        self.variant(Pt0Bc::Force0)
    }
    #[doc = "Pass input B"]
    #[inline(always)]
    pub fn pass(self) -> &'a mut crate::W<REG> {
        self.variant(Pt0Bc::Pass)
    }
    #[doc = "Complement input B"]
    #[inline(always)]
    pub fn complement(self) -> &'a mut crate::W<REG> {
        self.variant(Pt0Bc::Complement)
    }
    #[doc = "Force input B to become 1"]
    #[inline(always)]
    pub fn force_1(self) -> &'a mut crate::W<REG> {
        self.variant(Pt0Bc::Force1)
    }
}
#[doc = "Product Term 0, Input A Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pt0Ac {
    #[doc = "0: Force input A to become 0"]
    Force0 = 0,
    #[doc = "1: Pass input A"]
    Pass = 1,
    #[doc = "2: Complement input A"]
    Complement = 2,
    #[doc = "3: Force input A to become 1"]
    Force1 = 3,
}
impl From<Pt0Ac> for u8 {
    #[inline(always)]
    fn from(variant: Pt0Ac) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pt0Ac {
    type Ux = u8;
}
impl crate::IsEnum for Pt0Ac {}
#[doc = "Field `PT0_AC` reader - Product Term 0, Input A Configuration"]
pub type Pt0AcR = crate::FieldReader<Pt0Ac>;
impl Pt0AcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pt0Ac {
        match self.bits {
            0 => Pt0Ac::Force0,
            1 => Pt0Ac::Pass,
            2 => Pt0Ac::Complement,
            3 => Pt0Ac::Force1,
            _ => unreachable!(),
        }
    }
    #[doc = "Force input A to become 0"]
    #[inline(always)]
    pub fn is_force_0(&self) -> bool {
        *self == Pt0Ac::Force0
    }
    #[doc = "Pass input A"]
    #[inline(always)]
    pub fn is_pass(&self) -> bool {
        *self == Pt0Ac::Pass
    }
    #[doc = "Complement input A"]
    #[inline(always)]
    pub fn is_complement(&self) -> bool {
        *self == Pt0Ac::Complement
    }
    #[doc = "Force input A to become 1"]
    #[inline(always)]
    pub fn is_force_1(&self) -> bool {
        *self == Pt0Ac::Force1
    }
}
#[doc = "Field `PT0_AC` writer - Product Term 0, Input A Configuration"]
pub type Pt0AcW<'a, REG> = crate::FieldWriter<'a, REG, 2, Pt0Ac, crate::Safe>;
impl<'a, REG> Pt0AcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Force input A to become 0"]
    #[inline(always)]
    pub fn force_0(self) -> &'a mut crate::W<REG> {
        self.variant(Pt0Ac::Force0)
    }
    #[doc = "Pass input A"]
    #[inline(always)]
    pub fn pass(self) -> &'a mut crate::W<REG> {
        self.variant(Pt0Ac::Pass)
    }
    #[doc = "Complement input A"]
    #[inline(always)]
    pub fn complement(self) -> &'a mut crate::W<REG> {
        self.variant(Pt0Ac::Complement)
    }
    #[doc = "Force input A to become 1"]
    #[inline(always)]
    pub fn force_1(self) -> &'a mut crate::W<REG> {
        self.variant(Pt0Ac::Force1)
    }
}
impl R {
    #[doc = "Bits 0:1 - Product Term 1, Input D Configuration"]
    #[inline(always)]
    pub fn pt1_dc(&self) -> Pt1DcR {
        Pt1DcR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 2:3 - Product Term 1, Input C Configuration"]
    #[inline(always)]
    pub fn pt1_cc(&self) -> Pt1CcR {
        Pt1CcR::new(((self.bits >> 2) & 3) as u8)
    }
    #[doc = "Bits 4:5 - Product Term 1, Input B Configuration"]
    #[inline(always)]
    pub fn pt1_bc(&self) -> Pt1BcR {
        Pt1BcR::new(((self.bits >> 4) & 3) as u8)
    }
    #[doc = "Bits 6:7 - Product Term 1, Input A Configuration"]
    #[inline(always)]
    pub fn pt1_ac(&self) -> Pt1AcR {
        Pt1AcR::new(((self.bits >> 6) & 3) as u8)
    }
    #[doc = "Bits 8:9 - Product Term 0, Input D Configuration"]
    #[inline(always)]
    pub fn pt0_dc(&self) -> Pt0DcR {
        Pt0DcR::new(((self.bits >> 8) & 3) as u8)
    }
    #[doc = "Bits 10:11 - Product Term 0, Input C Configuration"]
    #[inline(always)]
    pub fn pt0_cc(&self) -> Pt0CcR {
        Pt0CcR::new(((self.bits >> 10) & 3) as u8)
    }
    #[doc = "Bits 12:13 - Product Term 0, Input B Configuration"]
    #[inline(always)]
    pub fn pt0_bc(&self) -> Pt0BcR {
        Pt0BcR::new(((self.bits >> 12) & 3) as u8)
    }
    #[doc = "Bits 14:15 - Product Term 0, Input A Configuration"]
    #[inline(always)]
    pub fn pt0_ac(&self) -> Pt0AcR {
        Pt0AcR::new(((self.bits >> 14) & 3) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - Product Term 1, Input D Configuration"]
    #[inline(always)]
    pub fn pt1_dc(&mut self) -> Pt1DcW<Bfcrt010Spec> {
        Pt1DcW::new(self, 0)
    }
    #[doc = "Bits 2:3 - Product Term 1, Input C Configuration"]
    #[inline(always)]
    pub fn pt1_cc(&mut self) -> Pt1CcW<Bfcrt010Spec> {
        Pt1CcW::new(self, 2)
    }
    #[doc = "Bits 4:5 - Product Term 1, Input B Configuration"]
    #[inline(always)]
    pub fn pt1_bc(&mut self) -> Pt1BcW<Bfcrt010Spec> {
        Pt1BcW::new(self, 4)
    }
    #[doc = "Bits 6:7 - Product Term 1, Input A Configuration"]
    #[inline(always)]
    pub fn pt1_ac(&mut self) -> Pt1AcW<Bfcrt010Spec> {
        Pt1AcW::new(self, 6)
    }
    #[doc = "Bits 8:9 - Product Term 0, Input D Configuration"]
    #[inline(always)]
    pub fn pt0_dc(&mut self) -> Pt0DcW<Bfcrt010Spec> {
        Pt0DcW::new(self, 8)
    }
    #[doc = "Bits 10:11 - Product Term 0, Input C Configuration"]
    #[inline(always)]
    pub fn pt0_cc(&mut self) -> Pt0CcW<Bfcrt010Spec> {
        Pt0CcW::new(self, 10)
    }
    #[doc = "Bits 12:13 - Product Term 0, Input B Configuration"]
    #[inline(always)]
    pub fn pt0_bc(&mut self) -> Pt0BcW<Bfcrt010Spec> {
        Pt0BcW::new(self, 12)
    }
    #[doc = "Bits 14:15 - Product Term 0, Input A Configuration"]
    #[inline(always)]
    pub fn pt0_ac(&mut self) -> Pt0AcW<Bfcrt010Spec> {
        Pt0AcW::new(self, 14)
    }
}
#[doc = "Boolean Function Term 0 and 1 Configuration for EVENT0\n\nYou can [`read`](crate::Reg::read) this register and get [`bfcrt010::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`bfcrt010::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Bfcrt010Spec;
impl crate::RegisterSpec for Bfcrt010Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`bfcrt010::R`](R) reader structure"]
impl crate::Readable for Bfcrt010Spec {}
#[doc = "`write(|w| ..)` method takes [`bfcrt010::W`](W) writer structure"]
impl crate::Writable for Bfcrt010Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets BFCRT010 to value 0"]
impl crate::Resettable for Bfcrt010Spec {}

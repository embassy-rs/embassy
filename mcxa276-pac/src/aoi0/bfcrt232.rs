#[doc = "Register `BFCRT232` reader"]
pub type R = crate::R<Bfcrt232Spec>;
#[doc = "Register `BFCRT232` writer"]
pub type W = crate::W<Bfcrt232Spec>;
#[doc = "Product Term 3, Input D Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pt3Dc {
    #[doc = "0: Force input D to become 0"]
    Force0 = 0,
    #[doc = "1: Pass input D"]
    Pass = 1,
    #[doc = "2: Complement input D"]
    Complement = 2,
    #[doc = "3: Force input D to become 1"]
    Force1 = 3,
}
impl From<Pt3Dc> for u8 {
    #[inline(always)]
    fn from(variant: Pt3Dc) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pt3Dc {
    type Ux = u8;
}
impl crate::IsEnum for Pt3Dc {}
#[doc = "Field `PT3_DC` reader - Product Term 3, Input D Configuration"]
pub type Pt3DcR = crate::FieldReader<Pt3Dc>;
impl Pt3DcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pt3Dc {
        match self.bits {
            0 => Pt3Dc::Force0,
            1 => Pt3Dc::Pass,
            2 => Pt3Dc::Complement,
            3 => Pt3Dc::Force1,
            _ => unreachable!(),
        }
    }
    #[doc = "Force input D to become 0"]
    #[inline(always)]
    pub fn is_force_0(&self) -> bool {
        *self == Pt3Dc::Force0
    }
    #[doc = "Pass input D"]
    #[inline(always)]
    pub fn is_pass(&self) -> bool {
        *self == Pt3Dc::Pass
    }
    #[doc = "Complement input D"]
    #[inline(always)]
    pub fn is_complement(&self) -> bool {
        *self == Pt3Dc::Complement
    }
    #[doc = "Force input D to become 1"]
    #[inline(always)]
    pub fn is_force_1(&self) -> bool {
        *self == Pt3Dc::Force1
    }
}
#[doc = "Field `PT3_DC` writer - Product Term 3, Input D Configuration"]
pub type Pt3DcW<'a, REG> = crate::FieldWriter<'a, REG, 2, Pt3Dc, crate::Safe>;
impl<'a, REG> Pt3DcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Force input D to become 0"]
    #[inline(always)]
    pub fn force_0(self) -> &'a mut crate::W<REG> {
        self.variant(Pt3Dc::Force0)
    }
    #[doc = "Pass input D"]
    #[inline(always)]
    pub fn pass(self) -> &'a mut crate::W<REG> {
        self.variant(Pt3Dc::Pass)
    }
    #[doc = "Complement input D"]
    #[inline(always)]
    pub fn complement(self) -> &'a mut crate::W<REG> {
        self.variant(Pt3Dc::Complement)
    }
    #[doc = "Force input D to become 1"]
    #[inline(always)]
    pub fn force_1(self) -> &'a mut crate::W<REG> {
        self.variant(Pt3Dc::Force1)
    }
}
#[doc = "Product Term 3, Input C Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pt3Cc {
    #[doc = "0: Force input C to become 0"]
    Force0 = 0,
    #[doc = "1: Pass input C"]
    Pass = 1,
    #[doc = "2: Complement input C"]
    Complement = 2,
    #[doc = "3: Force input C to become 1"]
    Force1 = 3,
}
impl From<Pt3Cc> for u8 {
    #[inline(always)]
    fn from(variant: Pt3Cc) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pt3Cc {
    type Ux = u8;
}
impl crate::IsEnum for Pt3Cc {}
#[doc = "Field `PT3_CC` reader - Product Term 3, Input C Configuration"]
pub type Pt3CcR = crate::FieldReader<Pt3Cc>;
impl Pt3CcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pt3Cc {
        match self.bits {
            0 => Pt3Cc::Force0,
            1 => Pt3Cc::Pass,
            2 => Pt3Cc::Complement,
            3 => Pt3Cc::Force1,
            _ => unreachable!(),
        }
    }
    #[doc = "Force input C to become 0"]
    #[inline(always)]
    pub fn is_force_0(&self) -> bool {
        *self == Pt3Cc::Force0
    }
    #[doc = "Pass input C"]
    #[inline(always)]
    pub fn is_pass(&self) -> bool {
        *self == Pt3Cc::Pass
    }
    #[doc = "Complement input C"]
    #[inline(always)]
    pub fn is_complement(&self) -> bool {
        *self == Pt3Cc::Complement
    }
    #[doc = "Force input C to become 1"]
    #[inline(always)]
    pub fn is_force_1(&self) -> bool {
        *self == Pt3Cc::Force1
    }
}
#[doc = "Field `PT3_CC` writer - Product Term 3, Input C Configuration"]
pub type Pt3CcW<'a, REG> = crate::FieldWriter<'a, REG, 2, Pt3Cc, crate::Safe>;
impl<'a, REG> Pt3CcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Force input C to become 0"]
    #[inline(always)]
    pub fn force_0(self) -> &'a mut crate::W<REG> {
        self.variant(Pt3Cc::Force0)
    }
    #[doc = "Pass input C"]
    #[inline(always)]
    pub fn pass(self) -> &'a mut crate::W<REG> {
        self.variant(Pt3Cc::Pass)
    }
    #[doc = "Complement input C"]
    #[inline(always)]
    pub fn complement(self) -> &'a mut crate::W<REG> {
        self.variant(Pt3Cc::Complement)
    }
    #[doc = "Force input C to become 1"]
    #[inline(always)]
    pub fn force_1(self) -> &'a mut crate::W<REG> {
        self.variant(Pt3Cc::Force1)
    }
}
#[doc = "Product Term 3, Input B Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pt3Bc {
    #[doc = "0: Force input B to become 0"]
    Force0 = 0,
    #[doc = "1: Pass input B"]
    Pass = 1,
    #[doc = "2: Complement input B"]
    Complement = 2,
    #[doc = "3: Force input B to become 1"]
    Force1 = 3,
}
impl From<Pt3Bc> for u8 {
    #[inline(always)]
    fn from(variant: Pt3Bc) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pt3Bc {
    type Ux = u8;
}
impl crate::IsEnum for Pt3Bc {}
#[doc = "Field `PT3_BC` reader - Product Term 3, Input B Configuration"]
pub type Pt3BcR = crate::FieldReader<Pt3Bc>;
impl Pt3BcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pt3Bc {
        match self.bits {
            0 => Pt3Bc::Force0,
            1 => Pt3Bc::Pass,
            2 => Pt3Bc::Complement,
            3 => Pt3Bc::Force1,
            _ => unreachable!(),
        }
    }
    #[doc = "Force input B to become 0"]
    #[inline(always)]
    pub fn is_force_0(&self) -> bool {
        *self == Pt3Bc::Force0
    }
    #[doc = "Pass input B"]
    #[inline(always)]
    pub fn is_pass(&self) -> bool {
        *self == Pt3Bc::Pass
    }
    #[doc = "Complement input B"]
    #[inline(always)]
    pub fn is_complement(&self) -> bool {
        *self == Pt3Bc::Complement
    }
    #[doc = "Force input B to become 1"]
    #[inline(always)]
    pub fn is_force_1(&self) -> bool {
        *self == Pt3Bc::Force1
    }
}
#[doc = "Field `PT3_BC` writer - Product Term 3, Input B Configuration"]
pub type Pt3BcW<'a, REG> = crate::FieldWriter<'a, REG, 2, Pt3Bc, crate::Safe>;
impl<'a, REG> Pt3BcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Force input B to become 0"]
    #[inline(always)]
    pub fn force_0(self) -> &'a mut crate::W<REG> {
        self.variant(Pt3Bc::Force0)
    }
    #[doc = "Pass input B"]
    #[inline(always)]
    pub fn pass(self) -> &'a mut crate::W<REG> {
        self.variant(Pt3Bc::Pass)
    }
    #[doc = "Complement input B"]
    #[inline(always)]
    pub fn complement(self) -> &'a mut crate::W<REG> {
        self.variant(Pt3Bc::Complement)
    }
    #[doc = "Force input B to become 1"]
    #[inline(always)]
    pub fn force_1(self) -> &'a mut crate::W<REG> {
        self.variant(Pt3Bc::Force1)
    }
}
#[doc = "Product Term 3, Input A Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pt3Ac {
    #[doc = "0: Force input A to become 0"]
    Force0 = 0,
    #[doc = "1: Pass input A"]
    Pass = 1,
    #[doc = "2: Complement input A"]
    Complement = 2,
    #[doc = "3: Force input to become 1"]
    Force1 = 3,
}
impl From<Pt3Ac> for u8 {
    #[inline(always)]
    fn from(variant: Pt3Ac) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pt3Ac {
    type Ux = u8;
}
impl crate::IsEnum for Pt3Ac {}
#[doc = "Field `PT3_AC` reader - Product Term 3, Input A Configuration"]
pub type Pt3AcR = crate::FieldReader<Pt3Ac>;
impl Pt3AcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pt3Ac {
        match self.bits {
            0 => Pt3Ac::Force0,
            1 => Pt3Ac::Pass,
            2 => Pt3Ac::Complement,
            3 => Pt3Ac::Force1,
            _ => unreachable!(),
        }
    }
    #[doc = "Force input A to become 0"]
    #[inline(always)]
    pub fn is_force_0(&self) -> bool {
        *self == Pt3Ac::Force0
    }
    #[doc = "Pass input A"]
    #[inline(always)]
    pub fn is_pass(&self) -> bool {
        *self == Pt3Ac::Pass
    }
    #[doc = "Complement input A"]
    #[inline(always)]
    pub fn is_complement(&self) -> bool {
        *self == Pt3Ac::Complement
    }
    #[doc = "Force input to become 1"]
    #[inline(always)]
    pub fn is_force_1(&self) -> bool {
        *self == Pt3Ac::Force1
    }
}
#[doc = "Field `PT3_AC` writer - Product Term 3, Input A Configuration"]
pub type Pt3AcW<'a, REG> = crate::FieldWriter<'a, REG, 2, Pt3Ac, crate::Safe>;
impl<'a, REG> Pt3AcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Force input A to become 0"]
    #[inline(always)]
    pub fn force_0(self) -> &'a mut crate::W<REG> {
        self.variant(Pt3Ac::Force0)
    }
    #[doc = "Pass input A"]
    #[inline(always)]
    pub fn pass(self) -> &'a mut crate::W<REG> {
        self.variant(Pt3Ac::Pass)
    }
    #[doc = "Complement input A"]
    #[inline(always)]
    pub fn complement(self) -> &'a mut crate::W<REG> {
        self.variant(Pt3Ac::Complement)
    }
    #[doc = "Force input to become 1"]
    #[inline(always)]
    pub fn force_1(self) -> &'a mut crate::W<REG> {
        self.variant(Pt3Ac::Force1)
    }
}
#[doc = "Product Term 2, Input D Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pt2Dc {
    #[doc = "0: Force input D to become 0"]
    Force0 = 0,
    #[doc = "1: Pass input D"]
    Pass = 1,
    #[doc = "2: Complement input D"]
    Complement = 2,
    #[doc = "3: Force input D to become 1"]
    Force1 = 3,
}
impl From<Pt2Dc> for u8 {
    #[inline(always)]
    fn from(variant: Pt2Dc) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pt2Dc {
    type Ux = u8;
}
impl crate::IsEnum for Pt2Dc {}
#[doc = "Field `PT2_DC` reader - Product Term 2, Input D Configuration"]
pub type Pt2DcR = crate::FieldReader<Pt2Dc>;
impl Pt2DcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pt2Dc {
        match self.bits {
            0 => Pt2Dc::Force0,
            1 => Pt2Dc::Pass,
            2 => Pt2Dc::Complement,
            3 => Pt2Dc::Force1,
            _ => unreachable!(),
        }
    }
    #[doc = "Force input D to become 0"]
    #[inline(always)]
    pub fn is_force_0(&self) -> bool {
        *self == Pt2Dc::Force0
    }
    #[doc = "Pass input D"]
    #[inline(always)]
    pub fn is_pass(&self) -> bool {
        *self == Pt2Dc::Pass
    }
    #[doc = "Complement input D"]
    #[inline(always)]
    pub fn is_complement(&self) -> bool {
        *self == Pt2Dc::Complement
    }
    #[doc = "Force input D to become 1"]
    #[inline(always)]
    pub fn is_force_1(&self) -> bool {
        *self == Pt2Dc::Force1
    }
}
#[doc = "Field `PT2_DC` writer - Product Term 2, Input D Configuration"]
pub type Pt2DcW<'a, REG> = crate::FieldWriter<'a, REG, 2, Pt2Dc, crate::Safe>;
impl<'a, REG> Pt2DcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Force input D to become 0"]
    #[inline(always)]
    pub fn force_0(self) -> &'a mut crate::W<REG> {
        self.variant(Pt2Dc::Force0)
    }
    #[doc = "Pass input D"]
    #[inline(always)]
    pub fn pass(self) -> &'a mut crate::W<REG> {
        self.variant(Pt2Dc::Pass)
    }
    #[doc = "Complement input D"]
    #[inline(always)]
    pub fn complement(self) -> &'a mut crate::W<REG> {
        self.variant(Pt2Dc::Complement)
    }
    #[doc = "Force input D to become 1"]
    #[inline(always)]
    pub fn force_1(self) -> &'a mut crate::W<REG> {
        self.variant(Pt2Dc::Force1)
    }
}
#[doc = "Product Term 2, Input C Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pt2Cc {
    #[doc = "0: Force input C to become 0"]
    Force0 = 0,
    #[doc = "1: Pass input C"]
    Pass = 1,
    #[doc = "2: Complement input C"]
    Complement = 2,
    #[doc = "3: Force input C to become 1"]
    Force1 = 3,
}
impl From<Pt2Cc> for u8 {
    #[inline(always)]
    fn from(variant: Pt2Cc) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pt2Cc {
    type Ux = u8;
}
impl crate::IsEnum for Pt2Cc {}
#[doc = "Field `PT2_CC` reader - Product Term 2, Input C Configuration"]
pub type Pt2CcR = crate::FieldReader<Pt2Cc>;
impl Pt2CcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pt2Cc {
        match self.bits {
            0 => Pt2Cc::Force0,
            1 => Pt2Cc::Pass,
            2 => Pt2Cc::Complement,
            3 => Pt2Cc::Force1,
            _ => unreachable!(),
        }
    }
    #[doc = "Force input C to become 0"]
    #[inline(always)]
    pub fn is_force_0(&self) -> bool {
        *self == Pt2Cc::Force0
    }
    #[doc = "Pass input C"]
    #[inline(always)]
    pub fn is_pass(&self) -> bool {
        *self == Pt2Cc::Pass
    }
    #[doc = "Complement input C"]
    #[inline(always)]
    pub fn is_complement(&self) -> bool {
        *self == Pt2Cc::Complement
    }
    #[doc = "Force input C to become 1"]
    #[inline(always)]
    pub fn is_force_1(&self) -> bool {
        *self == Pt2Cc::Force1
    }
}
#[doc = "Field `PT2_CC` writer - Product Term 2, Input C Configuration"]
pub type Pt2CcW<'a, REG> = crate::FieldWriter<'a, REG, 2, Pt2Cc, crate::Safe>;
impl<'a, REG> Pt2CcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Force input C to become 0"]
    #[inline(always)]
    pub fn force_0(self) -> &'a mut crate::W<REG> {
        self.variant(Pt2Cc::Force0)
    }
    #[doc = "Pass input C"]
    #[inline(always)]
    pub fn pass(self) -> &'a mut crate::W<REG> {
        self.variant(Pt2Cc::Pass)
    }
    #[doc = "Complement input C"]
    #[inline(always)]
    pub fn complement(self) -> &'a mut crate::W<REG> {
        self.variant(Pt2Cc::Complement)
    }
    #[doc = "Force input C to become 1"]
    #[inline(always)]
    pub fn force_1(self) -> &'a mut crate::W<REG> {
        self.variant(Pt2Cc::Force1)
    }
}
#[doc = "Product Term 2, Input B Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pt2Bc {
    #[doc = "0: Force input B to become 0"]
    Force0 = 0,
    #[doc = "1: Pass input B"]
    Pass = 1,
    #[doc = "2: Complement input B"]
    Complement = 2,
    #[doc = "3: Force input B to become 1"]
    Force1 = 3,
}
impl From<Pt2Bc> for u8 {
    #[inline(always)]
    fn from(variant: Pt2Bc) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pt2Bc {
    type Ux = u8;
}
impl crate::IsEnum for Pt2Bc {}
#[doc = "Field `PT2_BC` reader - Product Term 2, Input B Configuration"]
pub type Pt2BcR = crate::FieldReader<Pt2Bc>;
impl Pt2BcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pt2Bc {
        match self.bits {
            0 => Pt2Bc::Force0,
            1 => Pt2Bc::Pass,
            2 => Pt2Bc::Complement,
            3 => Pt2Bc::Force1,
            _ => unreachable!(),
        }
    }
    #[doc = "Force input B to become 0"]
    #[inline(always)]
    pub fn is_force_0(&self) -> bool {
        *self == Pt2Bc::Force0
    }
    #[doc = "Pass input B"]
    #[inline(always)]
    pub fn is_pass(&self) -> bool {
        *self == Pt2Bc::Pass
    }
    #[doc = "Complement input B"]
    #[inline(always)]
    pub fn is_complement(&self) -> bool {
        *self == Pt2Bc::Complement
    }
    #[doc = "Force input B to become 1"]
    #[inline(always)]
    pub fn is_force_1(&self) -> bool {
        *self == Pt2Bc::Force1
    }
}
#[doc = "Field `PT2_BC` writer - Product Term 2, Input B Configuration"]
pub type Pt2BcW<'a, REG> = crate::FieldWriter<'a, REG, 2, Pt2Bc, crate::Safe>;
impl<'a, REG> Pt2BcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Force input B to become 0"]
    #[inline(always)]
    pub fn force_0(self) -> &'a mut crate::W<REG> {
        self.variant(Pt2Bc::Force0)
    }
    #[doc = "Pass input B"]
    #[inline(always)]
    pub fn pass(self) -> &'a mut crate::W<REG> {
        self.variant(Pt2Bc::Pass)
    }
    #[doc = "Complement input B"]
    #[inline(always)]
    pub fn complement(self) -> &'a mut crate::W<REG> {
        self.variant(Pt2Bc::Complement)
    }
    #[doc = "Force input B to become 1"]
    #[inline(always)]
    pub fn force_1(self) -> &'a mut crate::W<REG> {
        self.variant(Pt2Bc::Force1)
    }
}
#[doc = "Product Term 2, Input A Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pt2Ac {
    #[doc = "0: Force input A to become 0"]
    Force0 = 0,
    #[doc = "1: Pass input A"]
    Pass = 1,
    #[doc = "2: Complement input A"]
    Complement = 2,
    #[doc = "3: Force input A to become 1"]
    Force1 = 3,
}
impl From<Pt2Ac> for u8 {
    #[inline(always)]
    fn from(variant: Pt2Ac) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pt2Ac {
    type Ux = u8;
}
impl crate::IsEnum for Pt2Ac {}
#[doc = "Field `PT2_AC` reader - Product Term 2, Input A Configuration"]
pub type Pt2AcR = crate::FieldReader<Pt2Ac>;
impl Pt2AcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pt2Ac {
        match self.bits {
            0 => Pt2Ac::Force0,
            1 => Pt2Ac::Pass,
            2 => Pt2Ac::Complement,
            3 => Pt2Ac::Force1,
            _ => unreachable!(),
        }
    }
    #[doc = "Force input A to become 0"]
    #[inline(always)]
    pub fn is_force_0(&self) -> bool {
        *self == Pt2Ac::Force0
    }
    #[doc = "Pass input A"]
    #[inline(always)]
    pub fn is_pass(&self) -> bool {
        *self == Pt2Ac::Pass
    }
    #[doc = "Complement input A"]
    #[inline(always)]
    pub fn is_complement(&self) -> bool {
        *self == Pt2Ac::Complement
    }
    #[doc = "Force input A to become 1"]
    #[inline(always)]
    pub fn is_force_1(&self) -> bool {
        *self == Pt2Ac::Force1
    }
}
#[doc = "Field `PT2_AC` writer - Product Term 2, Input A Configuration"]
pub type Pt2AcW<'a, REG> = crate::FieldWriter<'a, REG, 2, Pt2Ac, crate::Safe>;
impl<'a, REG> Pt2AcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Force input A to become 0"]
    #[inline(always)]
    pub fn force_0(self) -> &'a mut crate::W<REG> {
        self.variant(Pt2Ac::Force0)
    }
    #[doc = "Pass input A"]
    #[inline(always)]
    pub fn pass(self) -> &'a mut crate::W<REG> {
        self.variant(Pt2Ac::Pass)
    }
    #[doc = "Complement input A"]
    #[inline(always)]
    pub fn complement(self) -> &'a mut crate::W<REG> {
        self.variant(Pt2Ac::Complement)
    }
    #[doc = "Force input A to become 1"]
    #[inline(always)]
    pub fn force_1(self) -> &'a mut crate::W<REG> {
        self.variant(Pt2Ac::Force1)
    }
}
impl R {
    #[doc = "Bits 0:1 - Product Term 3, Input D Configuration"]
    #[inline(always)]
    pub fn pt3_dc(&self) -> Pt3DcR {
        Pt3DcR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 2:3 - Product Term 3, Input C Configuration"]
    #[inline(always)]
    pub fn pt3_cc(&self) -> Pt3CcR {
        Pt3CcR::new(((self.bits >> 2) & 3) as u8)
    }
    #[doc = "Bits 4:5 - Product Term 3, Input B Configuration"]
    #[inline(always)]
    pub fn pt3_bc(&self) -> Pt3BcR {
        Pt3BcR::new(((self.bits >> 4) & 3) as u8)
    }
    #[doc = "Bits 6:7 - Product Term 3, Input A Configuration"]
    #[inline(always)]
    pub fn pt3_ac(&self) -> Pt3AcR {
        Pt3AcR::new(((self.bits >> 6) & 3) as u8)
    }
    #[doc = "Bits 8:9 - Product Term 2, Input D Configuration"]
    #[inline(always)]
    pub fn pt2_dc(&self) -> Pt2DcR {
        Pt2DcR::new(((self.bits >> 8) & 3) as u8)
    }
    #[doc = "Bits 10:11 - Product Term 2, Input C Configuration"]
    #[inline(always)]
    pub fn pt2_cc(&self) -> Pt2CcR {
        Pt2CcR::new(((self.bits >> 10) & 3) as u8)
    }
    #[doc = "Bits 12:13 - Product Term 2, Input B Configuration"]
    #[inline(always)]
    pub fn pt2_bc(&self) -> Pt2BcR {
        Pt2BcR::new(((self.bits >> 12) & 3) as u8)
    }
    #[doc = "Bits 14:15 - Product Term 2, Input A Configuration"]
    #[inline(always)]
    pub fn pt2_ac(&self) -> Pt2AcR {
        Pt2AcR::new(((self.bits >> 14) & 3) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - Product Term 3, Input D Configuration"]
    #[inline(always)]
    pub fn pt3_dc(&mut self) -> Pt3DcW<Bfcrt232Spec> {
        Pt3DcW::new(self, 0)
    }
    #[doc = "Bits 2:3 - Product Term 3, Input C Configuration"]
    #[inline(always)]
    pub fn pt3_cc(&mut self) -> Pt3CcW<Bfcrt232Spec> {
        Pt3CcW::new(self, 2)
    }
    #[doc = "Bits 4:5 - Product Term 3, Input B Configuration"]
    #[inline(always)]
    pub fn pt3_bc(&mut self) -> Pt3BcW<Bfcrt232Spec> {
        Pt3BcW::new(self, 4)
    }
    #[doc = "Bits 6:7 - Product Term 3, Input A Configuration"]
    #[inline(always)]
    pub fn pt3_ac(&mut self) -> Pt3AcW<Bfcrt232Spec> {
        Pt3AcW::new(self, 6)
    }
    #[doc = "Bits 8:9 - Product Term 2, Input D Configuration"]
    #[inline(always)]
    pub fn pt2_dc(&mut self) -> Pt2DcW<Bfcrt232Spec> {
        Pt2DcW::new(self, 8)
    }
    #[doc = "Bits 10:11 - Product Term 2, Input C Configuration"]
    #[inline(always)]
    pub fn pt2_cc(&mut self) -> Pt2CcW<Bfcrt232Spec> {
        Pt2CcW::new(self, 10)
    }
    #[doc = "Bits 12:13 - Product Term 2, Input B Configuration"]
    #[inline(always)]
    pub fn pt2_bc(&mut self) -> Pt2BcW<Bfcrt232Spec> {
        Pt2BcW::new(self, 12)
    }
    #[doc = "Bits 14:15 - Product Term 2, Input A Configuration"]
    #[inline(always)]
    pub fn pt2_ac(&mut self) -> Pt2AcW<Bfcrt232Spec> {
        Pt2AcW::new(self, 14)
    }
}
#[doc = "Boolean Function Term 2 and 3 Configuration for EVENT2\n\nYou can [`read`](crate::Reg::read) this register and get [`bfcrt232::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`bfcrt232::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Bfcrt232Spec;
impl crate::RegisterSpec for Bfcrt232Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`bfcrt232::R`](R) reader structure"]
impl crate::Readable for Bfcrt232Spec {}
#[doc = "`write(|w| ..)` method takes [`bfcrt232::W`](W) writer structure"]
impl crate::Writable for Bfcrt232Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets BFCRT232 to value 0"]
impl crate::Resettable for Bfcrt232Spec {}

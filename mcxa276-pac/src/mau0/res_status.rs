#[doc = "Register `RES_STATUS` reader"]
pub type R = crate::R<ResStatusSpec>;
#[doc = "Register `RES_STATUS` writer"]
pub type W = crate::W<ResStatusSpec>;
#[doc = "RES0 IEEE Inexact Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res0Nx {
    #[doc = "0: The result is not rounded."]
    Res0NxNo = 0,
    #[doc = "1: The result is rounded, and as a result some digits lost."]
    Res0NxYes = 1,
}
impl From<Res0Nx> for bool {
    #[inline(always)]
    fn from(variant: Res0Nx) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES0_NX` reader - RES0 IEEE Inexact Flag"]
pub type Res0NxR = crate::BitReader<Res0Nx>;
impl Res0NxR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res0Nx {
        match self.bits {
            false => Res0Nx::Res0NxNo,
            true => Res0Nx::Res0NxYes,
        }
    }
    #[doc = "The result is not rounded."]
    #[inline(always)]
    pub fn is_res0_nx_no(&self) -> bool {
        *self == Res0Nx::Res0NxNo
    }
    #[doc = "The result is rounded, and as a result some digits lost."]
    #[inline(always)]
    pub fn is_res0_nx_yes(&self) -> bool {
        *self == Res0Nx::Res0NxYes
    }
}
#[doc = "Field `RES0_NX` writer - RES0 IEEE Inexact Flag"]
pub type Res0NxW<'a, REG> = crate::BitWriter<'a, REG, Res0Nx>;
impl<'a, REG> Res0NxW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The result is not rounded."]
    #[inline(always)]
    pub fn res0_nx_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res0Nx::Res0NxNo)
    }
    #[doc = "The result is rounded, and as a result some digits lost."]
    #[inline(always)]
    pub fn res0_nx_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res0Nx::Res0NxYes)
    }
}
#[doc = "RES0 IEEE Underflow Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res0Uf {
    #[doc = "0: No tiny non-zero result is detected."]
    Res0UfNo = 0,
    #[doc = "1: A tiny non-zero result is detected."]
    Res0UfYes = 1,
}
impl From<Res0Uf> for bool {
    #[inline(always)]
    fn from(variant: Res0Uf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES0_UF` reader - RES0 IEEE Underflow Flag"]
pub type Res0UfR = crate::BitReader<Res0Uf>;
impl Res0UfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res0Uf {
        match self.bits {
            false => Res0Uf::Res0UfNo,
            true => Res0Uf::Res0UfYes,
        }
    }
    #[doc = "No tiny non-zero result is detected."]
    #[inline(always)]
    pub fn is_res0_uf_no(&self) -> bool {
        *self == Res0Uf::Res0UfNo
    }
    #[doc = "A tiny non-zero result is detected."]
    #[inline(always)]
    pub fn is_res0_uf_yes(&self) -> bool {
        *self == Res0Uf::Res0UfYes
    }
}
#[doc = "Field `RES0_UF` writer - RES0 IEEE Underflow Flag"]
pub type Res0UfW<'a, REG> = crate::BitWriter<'a, REG, Res0Uf>;
impl<'a, REG> Res0UfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No tiny non-zero result is detected."]
    #[inline(always)]
    pub fn res0_uf_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res0Uf::Res0UfNo)
    }
    #[doc = "A tiny non-zero result is detected."]
    #[inline(always)]
    pub fn res0_uf_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res0Uf::Res0UfYes)
    }
}
#[doc = "RES0 IEEE Overflow Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res0Of {
    #[doc = "0: The result format's largest finite number is not exceeded."]
    Res0OfNo = 0,
    #[doc = "1: The result format's largest finite number is exceeded."]
    Res0OfYes = 1,
}
impl From<Res0Of> for bool {
    #[inline(always)]
    fn from(variant: Res0Of) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES0_OF` reader - RES0 IEEE Overflow Flag"]
pub type Res0OfR = crate::BitReader<Res0Of>;
impl Res0OfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res0Of {
        match self.bits {
            false => Res0Of::Res0OfNo,
            true => Res0Of::Res0OfYes,
        }
    }
    #[doc = "The result format's largest finite number is not exceeded."]
    #[inline(always)]
    pub fn is_res0_of_no(&self) -> bool {
        *self == Res0Of::Res0OfNo
    }
    #[doc = "The result format's largest finite number is exceeded."]
    #[inline(always)]
    pub fn is_res0_of_yes(&self) -> bool {
        *self == Res0Of::Res0OfYes
    }
}
#[doc = "Field `RES0_OF` writer - RES0 IEEE Overflow Flag"]
pub type Res0OfW<'a, REG> = crate::BitWriter<'a, REG, Res0Of>;
impl<'a, REG> Res0OfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The result format's largest finite number is not exceeded."]
    #[inline(always)]
    pub fn res0_of_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res0Of::Res0OfNo)
    }
    #[doc = "The result format's largest finite number is exceeded."]
    #[inline(always)]
    pub fn res0_of_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res0Of::Res0OfYes)
    }
}
#[doc = "RES0 IEEE Divide by Zero Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res0Dz {
    #[doc = "0: No exact infinite result is defined for an operation on finite operands."]
    Res0DzNo = 0,
    #[doc = "1: An exact infinite result is defined for an operation on finite operands."]
    Res0DzYes = 1,
}
impl From<Res0Dz> for bool {
    #[inline(always)]
    fn from(variant: Res0Dz) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES0_DZ` reader - RES0 IEEE Divide by Zero Flag"]
pub type Res0DzR = crate::BitReader<Res0Dz>;
impl Res0DzR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res0Dz {
        match self.bits {
            false => Res0Dz::Res0DzNo,
            true => Res0Dz::Res0DzYes,
        }
    }
    #[doc = "No exact infinite result is defined for an operation on finite operands."]
    #[inline(always)]
    pub fn is_res0_dz_no(&self) -> bool {
        *self == Res0Dz::Res0DzNo
    }
    #[doc = "An exact infinite result is defined for an operation on finite operands."]
    #[inline(always)]
    pub fn is_res0_dz_yes(&self) -> bool {
        *self == Res0Dz::Res0DzYes
    }
}
#[doc = "Field `RES0_DZ` writer - RES0 IEEE Divide by Zero Flag"]
pub type Res0DzW<'a, REG> = crate::BitWriter<'a, REG, Res0Dz>;
impl<'a, REG> Res0DzW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No exact infinite result is defined for an operation on finite operands."]
    #[inline(always)]
    pub fn res0_dz_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res0Dz::Res0DzNo)
    }
    #[doc = "An exact infinite result is defined for an operation on finite operands."]
    #[inline(always)]
    pub fn res0_dz_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res0Dz::Res0DzYes)
    }
}
#[doc = "RES0 IEEE Invalid Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res0Nv {
    #[doc = "0: There is usefully definable result."]
    Res0NvNo = 0,
    #[doc = "1: There is no usefully definable result."]
    Res0NvYes = 1,
}
impl From<Res0Nv> for bool {
    #[inline(always)]
    fn from(variant: Res0Nv) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES0_NV` reader - RES0 IEEE Invalid Flag"]
pub type Res0NvR = crate::BitReader<Res0Nv>;
impl Res0NvR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res0Nv {
        match self.bits {
            false => Res0Nv::Res0NvNo,
            true => Res0Nv::Res0NvYes,
        }
    }
    #[doc = "There is usefully definable result."]
    #[inline(always)]
    pub fn is_res0_nv_no(&self) -> bool {
        *self == Res0Nv::Res0NvNo
    }
    #[doc = "There is no usefully definable result."]
    #[inline(always)]
    pub fn is_res0_nv_yes(&self) -> bool {
        *self == Res0Nv::Res0NvYes
    }
}
#[doc = "Field `RES0_NV` writer - RES0 IEEE Invalid Flag"]
pub type Res0NvW<'a, REG> = crate::BitWriter<'a, REG, Res0Nv>;
impl<'a, REG> Res0NvW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "There is usefully definable result."]
    #[inline(always)]
    pub fn res0_nv_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res0Nv::Res0NvNo)
    }
    #[doc = "There is no usefully definable result."]
    #[inline(always)]
    pub fn res0_nv_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res0Nv::Res0NvYes)
    }
}
#[doc = "RES0 Indirect Operation Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res0Err {
    #[doc = "0: No invalid indirect operation is detected."]
    Res0ErrNo = 0,
    #[doc = "1: An invalid indirect operation error is detected."]
    Res0ErrYes = 1,
}
impl From<Res0Err> for bool {
    #[inline(always)]
    fn from(variant: Res0Err) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES0_ERR` reader - RES0 Indirect Operation Error Flag"]
pub type Res0ErrR = crate::BitReader<Res0Err>;
impl Res0ErrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res0Err {
        match self.bits {
            false => Res0Err::Res0ErrNo,
            true => Res0Err::Res0ErrYes,
        }
    }
    #[doc = "No invalid indirect operation is detected."]
    #[inline(always)]
    pub fn is_res0_err_no(&self) -> bool {
        *self == Res0Err::Res0ErrNo
    }
    #[doc = "An invalid indirect operation error is detected."]
    #[inline(always)]
    pub fn is_res0_err_yes(&self) -> bool {
        *self == Res0Err::Res0ErrYes
    }
}
#[doc = "Field `RES0_ERR` writer - RES0 Indirect Operation Error Flag"]
pub type Res0ErrW<'a, REG> = crate::BitWriter<'a, REG, Res0Err>;
impl<'a, REG> Res0ErrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No invalid indirect operation is detected."]
    #[inline(always)]
    pub fn res0_err_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res0Err::Res0ErrNo)
    }
    #[doc = "An invalid indirect operation error is detected."]
    #[inline(always)]
    pub fn res0_err_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res0Err::Res0ErrYes)
    }
}
#[doc = "RES0 Overwrite Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res0Ovwr {
    #[doc = "0: The value of RES0 has been read."]
    Res0OvwrNo = 0,
    #[doc = "1: The value of RES0 has not been read yet and is overwritten by a new MAUWRAP result."]
    Res0OvwrYes = 1,
}
impl From<Res0Ovwr> for bool {
    #[inline(always)]
    fn from(variant: Res0Ovwr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES0_OVWR` reader - RES0 Overwrite Flag"]
pub type Res0OvwrR = crate::BitReader<Res0Ovwr>;
impl Res0OvwrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res0Ovwr {
        match self.bits {
            false => Res0Ovwr::Res0OvwrNo,
            true => Res0Ovwr::Res0OvwrYes,
        }
    }
    #[doc = "The value of RES0 has been read."]
    #[inline(always)]
    pub fn is_res0_ovwr_no(&self) -> bool {
        *self == Res0Ovwr::Res0OvwrNo
    }
    #[doc = "The value of RES0 has not been read yet and is overwritten by a new MAUWRAP result."]
    #[inline(always)]
    pub fn is_res0_ovwr_yes(&self) -> bool {
        *self == Res0Ovwr::Res0OvwrYes
    }
}
#[doc = "Field `RES0_OVWR` writer - RES0 Overwrite Flag"]
pub type Res0OvwrW<'a, REG> = crate::BitWriter<'a, REG, Res0Ovwr>;
impl<'a, REG> Res0OvwrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The value of RES0 has been read."]
    #[inline(always)]
    pub fn res0_ovwr_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res0Ovwr::Res0OvwrNo)
    }
    #[doc = "The value of RES0 has not been read yet and is overwritten by a new MAUWRAP result."]
    #[inline(always)]
    pub fn res0_ovwr_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res0Ovwr::Res0OvwrYes)
    }
}
#[doc = "RES0 Full Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res0Full {
    #[doc = "0: RES0 has not updated and cannot be read."]
    Res0FullNo = 0,
    #[doc = "1: RES0 has updated and can be read."]
    Res0FullYes = 1,
}
impl From<Res0Full> for bool {
    #[inline(always)]
    fn from(variant: Res0Full) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES0_FULL` reader - RES0 Full Flag"]
pub type Res0FullR = crate::BitReader<Res0Full>;
impl Res0FullR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res0Full {
        match self.bits {
            false => Res0Full::Res0FullNo,
            true => Res0Full::Res0FullYes,
        }
    }
    #[doc = "RES0 has not updated and cannot be read."]
    #[inline(always)]
    pub fn is_res0_full_no(&self) -> bool {
        *self == Res0Full::Res0FullNo
    }
    #[doc = "RES0 has updated and can be read."]
    #[inline(always)]
    pub fn is_res0_full_yes(&self) -> bool {
        *self == Res0Full::Res0FullYes
    }
}
#[doc = "Field `RES0_FULL` writer - RES0 Full Flag"]
pub type Res0FullW<'a, REG> = crate::BitWriter<'a, REG, Res0Full>;
impl<'a, REG> Res0FullW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "RES0 has not updated and cannot be read."]
    #[inline(always)]
    pub fn res0_full_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res0Full::Res0FullNo)
    }
    #[doc = "RES0 has updated and can be read."]
    #[inline(always)]
    pub fn res0_full_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res0Full::Res0FullYes)
    }
}
#[doc = "RES1 IEEE Inexact Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res1Nx {
    #[doc = "0: The result is not rounded."]
    Res1NxNo = 0,
    #[doc = "1: The result is rounded, and as a result some digits lost."]
    Res1NxYes = 1,
}
impl From<Res1Nx> for bool {
    #[inline(always)]
    fn from(variant: Res1Nx) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES1_NX` reader - RES1 IEEE Inexact Flag"]
pub type Res1NxR = crate::BitReader<Res1Nx>;
impl Res1NxR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res1Nx {
        match self.bits {
            false => Res1Nx::Res1NxNo,
            true => Res1Nx::Res1NxYes,
        }
    }
    #[doc = "The result is not rounded."]
    #[inline(always)]
    pub fn is_res1_nx_no(&self) -> bool {
        *self == Res1Nx::Res1NxNo
    }
    #[doc = "The result is rounded, and as a result some digits lost."]
    #[inline(always)]
    pub fn is_res1_nx_yes(&self) -> bool {
        *self == Res1Nx::Res1NxYes
    }
}
#[doc = "Field `RES1_NX` writer - RES1 IEEE Inexact Flag"]
pub type Res1NxW<'a, REG> = crate::BitWriter<'a, REG, Res1Nx>;
impl<'a, REG> Res1NxW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The result is not rounded."]
    #[inline(always)]
    pub fn res1_nx_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res1Nx::Res1NxNo)
    }
    #[doc = "The result is rounded, and as a result some digits lost."]
    #[inline(always)]
    pub fn res1_nx_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res1Nx::Res1NxYes)
    }
}
#[doc = "RES1 IEEE Underflow Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res1Uf {
    #[doc = "0: No tiny non-zero result is detected."]
    Res1UfNo = 0,
    #[doc = "1: A tiny non-zero result is detected."]
    Res1UfYes = 1,
}
impl From<Res1Uf> for bool {
    #[inline(always)]
    fn from(variant: Res1Uf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES1_UF` reader - RES1 IEEE Underflow Flag"]
pub type Res1UfR = crate::BitReader<Res1Uf>;
impl Res1UfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res1Uf {
        match self.bits {
            false => Res1Uf::Res1UfNo,
            true => Res1Uf::Res1UfYes,
        }
    }
    #[doc = "No tiny non-zero result is detected."]
    #[inline(always)]
    pub fn is_res1_uf_no(&self) -> bool {
        *self == Res1Uf::Res1UfNo
    }
    #[doc = "A tiny non-zero result is detected."]
    #[inline(always)]
    pub fn is_res1_uf_yes(&self) -> bool {
        *self == Res1Uf::Res1UfYes
    }
}
#[doc = "Field `RES1_UF` writer - RES1 IEEE Underflow Flag"]
pub type Res1UfW<'a, REG> = crate::BitWriter<'a, REG, Res1Uf>;
impl<'a, REG> Res1UfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No tiny non-zero result is detected."]
    #[inline(always)]
    pub fn res1_uf_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res1Uf::Res1UfNo)
    }
    #[doc = "A tiny non-zero result is detected."]
    #[inline(always)]
    pub fn res1_uf_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res1Uf::Res1UfYes)
    }
}
#[doc = "RES1 IEEE Overflow Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res1Of {
    #[doc = "0: The result format's largest finite number is not exceeded."]
    Res1OfNo = 0,
    #[doc = "1: The result format's largest finite number is exceeded."]
    Res1OfYes = 1,
}
impl From<Res1Of> for bool {
    #[inline(always)]
    fn from(variant: Res1Of) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES1_OF` reader - RES1 IEEE Overflow Flag"]
pub type Res1OfR = crate::BitReader<Res1Of>;
impl Res1OfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res1Of {
        match self.bits {
            false => Res1Of::Res1OfNo,
            true => Res1Of::Res1OfYes,
        }
    }
    #[doc = "The result format's largest finite number is not exceeded."]
    #[inline(always)]
    pub fn is_res1_of_no(&self) -> bool {
        *self == Res1Of::Res1OfNo
    }
    #[doc = "The result format's largest finite number is exceeded."]
    #[inline(always)]
    pub fn is_res1_of_yes(&self) -> bool {
        *self == Res1Of::Res1OfYes
    }
}
#[doc = "Field `RES1_OF` writer - RES1 IEEE Overflow Flag"]
pub type Res1OfW<'a, REG> = crate::BitWriter<'a, REG, Res1Of>;
impl<'a, REG> Res1OfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The result format's largest finite number is not exceeded."]
    #[inline(always)]
    pub fn res1_of_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res1Of::Res1OfNo)
    }
    #[doc = "The result format's largest finite number is exceeded."]
    #[inline(always)]
    pub fn res1_of_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res1Of::Res1OfYes)
    }
}
#[doc = "RES1 IEEE Divide by Zero Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res1Dz {
    #[doc = "0: No exact infinite result is defined for an operation on finite operands."]
    Res1DzNo = 0,
    #[doc = "1: An exact infinite result is defined for an operation on finite operands."]
    Res1DzYes = 1,
}
impl From<Res1Dz> for bool {
    #[inline(always)]
    fn from(variant: Res1Dz) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES1_DZ` reader - RES1 IEEE Divide by Zero Flag"]
pub type Res1DzR = crate::BitReader<Res1Dz>;
impl Res1DzR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res1Dz {
        match self.bits {
            false => Res1Dz::Res1DzNo,
            true => Res1Dz::Res1DzYes,
        }
    }
    #[doc = "No exact infinite result is defined for an operation on finite operands."]
    #[inline(always)]
    pub fn is_res1_dz_no(&self) -> bool {
        *self == Res1Dz::Res1DzNo
    }
    #[doc = "An exact infinite result is defined for an operation on finite operands."]
    #[inline(always)]
    pub fn is_res1_dz_yes(&self) -> bool {
        *self == Res1Dz::Res1DzYes
    }
}
#[doc = "Field `RES1_DZ` writer - RES1 IEEE Divide by Zero Flag"]
pub type Res1DzW<'a, REG> = crate::BitWriter<'a, REG, Res1Dz>;
impl<'a, REG> Res1DzW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No exact infinite result is defined for an operation on finite operands."]
    #[inline(always)]
    pub fn res1_dz_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res1Dz::Res1DzNo)
    }
    #[doc = "An exact infinite result is defined for an operation on finite operands."]
    #[inline(always)]
    pub fn res1_dz_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res1Dz::Res1DzYes)
    }
}
#[doc = "RES1 IEEE Invalid Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res1Nv {
    #[doc = "0: There is usefully definable result."]
    Res1NvNo = 0,
    #[doc = "1: There is no usefully definable result."]
    Res1NvYes = 1,
}
impl From<Res1Nv> for bool {
    #[inline(always)]
    fn from(variant: Res1Nv) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES1_NV` reader - RES1 IEEE Invalid Flag"]
pub type Res1NvR = crate::BitReader<Res1Nv>;
impl Res1NvR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res1Nv {
        match self.bits {
            false => Res1Nv::Res1NvNo,
            true => Res1Nv::Res1NvYes,
        }
    }
    #[doc = "There is usefully definable result."]
    #[inline(always)]
    pub fn is_res1_nv_no(&self) -> bool {
        *self == Res1Nv::Res1NvNo
    }
    #[doc = "There is no usefully definable result."]
    #[inline(always)]
    pub fn is_res1_nv_yes(&self) -> bool {
        *self == Res1Nv::Res1NvYes
    }
}
#[doc = "Field `RES1_NV` writer - RES1 IEEE Invalid Flag"]
pub type Res1NvW<'a, REG> = crate::BitWriter<'a, REG, Res1Nv>;
impl<'a, REG> Res1NvW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "There is usefully definable result."]
    #[inline(always)]
    pub fn res1_nv_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res1Nv::Res1NvNo)
    }
    #[doc = "There is no usefully definable result."]
    #[inline(always)]
    pub fn res1_nv_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res1Nv::Res1NvYes)
    }
}
#[doc = "RES1 Indirect Operation Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res1Err {
    #[doc = "0: No invalid indirect operation is detected."]
    Res1ErrNo = 0,
    #[doc = "1: An invalid indirect operation error is detected."]
    Res1ErrYes = 1,
}
impl From<Res1Err> for bool {
    #[inline(always)]
    fn from(variant: Res1Err) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES1_ERR` reader - RES1 Indirect Operation Error Flag"]
pub type Res1ErrR = crate::BitReader<Res1Err>;
impl Res1ErrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res1Err {
        match self.bits {
            false => Res1Err::Res1ErrNo,
            true => Res1Err::Res1ErrYes,
        }
    }
    #[doc = "No invalid indirect operation is detected."]
    #[inline(always)]
    pub fn is_res1_err_no(&self) -> bool {
        *self == Res1Err::Res1ErrNo
    }
    #[doc = "An invalid indirect operation error is detected."]
    #[inline(always)]
    pub fn is_res1_err_yes(&self) -> bool {
        *self == Res1Err::Res1ErrYes
    }
}
#[doc = "Field `RES1_ERR` writer - RES1 Indirect Operation Error Flag"]
pub type Res1ErrW<'a, REG> = crate::BitWriter<'a, REG, Res1Err>;
impl<'a, REG> Res1ErrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No invalid indirect operation is detected."]
    #[inline(always)]
    pub fn res1_err_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res1Err::Res1ErrNo)
    }
    #[doc = "An invalid indirect operation error is detected."]
    #[inline(always)]
    pub fn res1_err_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res1Err::Res1ErrYes)
    }
}
#[doc = "RES1 Overwrite Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res1Ovwr {
    #[doc = "0: The value of RES1 has been read."]
    Res1OvwrNo = 0,
    #[doc = "1: The value of RES1 has not been read yet and is overwritten by a new MAUWRAP result."]
    Res1OvwrYes = 1,
}
impl From<Res1Ovwr> for bool {
    #[inline(always)]
    fn from(variant: Res1Ovwr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES1_OVWR` reader - RES1 Overwrite Flag"]
pub type Res1OvwrR = crate::BitReader<Res1Ovwr>;
impl Res1OvwrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res1Ovwr {
        match self.bits {
            false => Res1Ovwr::Res1OvwrNo,
            true => Res1Ovwr::Res1OvwrYes,
        }
    }
    #[doc = "The value of RES1 has been read."]
    #[inline(always)]
    pub fn is_res1_ovwr_no(&self) -> bool {
        *self == Res1Ovwr::Res1OvwrNo
    }
    #[doc = "The value of RES1 has not been read yet and is overwritten by a new MAUWRAP result."]
    #[inline(always)]
    pub fn is_res1_ovwr_yes(&self) -> bool {
        *self == Res1Ovwr::Res1OvwrYes
    }
}
#[doc = "Field `RES1_OVWR` writer - RES1 Overwrite Flag"]
pub type Res1OvwrW<'a, REG> = crate::BitWriter<'a, REG, Res1Ovwr>;
impl<'a, REG> Res1OvwrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The value of RES1 has been read."]
    #[inline(always)]
    pub fn res1_ovwr_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res1Ovwr::Res1OvwrNo)
    }
    #[doc = "The value of RES1 has not been read yet and is overwritten by a new MAUWRAP result."]
    #[inline(always)]
    pub fn res1_ovwr_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res1Ovwr::Res1OvwrYes)
    }
}
#[doc = "RES1 Full Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res1Full {
    #[doc = "0: RES1 has not updated and cannot be read."]
    Res1FullNo = 0,
    #[doc = "1: RES1 has updated and can be read."]
    Res1FullYes = 1,
}
impl From<Res1Full> for bool {
    #[inline(always)]
    fn from(variant: Res1Full) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES1_FULL` reader - RES1 Full Flag"]
pub type Res1FullR = crate::BitReader<Res1Full>;
impl Res1FullR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res1Full {
        match self.bits {
            false => Res1Full::Res1FullNo,
            true => Res1Full::Res1FullYes,
        }
    }
    #[doc = "RES1 has not updated and cannot be read."]
    #[inline(always)]
    pub fn is_res1_full_no(&self) -> bool {
        *self == Res1Full::Res1FullNo
    }
    #[doc = "RES1 has updated and can be read."]
    #[inline(always)]
    pub fn is_res1_full_yes(&self) -> bool {
        *self == Res1Full::Res1FullYes
    }
}
#[doc = "Field `RES1_FULL` writer - RES1 Full Flag"]
pub type Res1FullW<'a, REG> = crate::BitWriter<'a, REG, Res1Full>;
impl<'a, REG> Res1FullW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "RES1 has not updated and cannot be read."]
    #[inline(always)]
    pub fn res1_full_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res1Full::Res1FullNo)
    }
    #[doc = "RES1 has updated and can be read."]
    #[inline(always)]
    pub fn res1_full_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res1Full::Res1FullYes)
    }
}
#[doc = "RES2 IEEE Inexact Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res2Nx {
    #[doc = "0: The result is not rounded."]
    Res2NxNo = 0,
    #[doc = "1: The result is rounded, and as a result some digits lost."]
    Res2NxYes = 1,
}
impl From<Res2Nx> for bool {
    #[inline(always)]
    fn from(variant: Res2Nx) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES2_NX` reader - RES2 IEEE Inexact Flag"]
pub type Res2NxR = crate::BitReader<Res2Nx>;
impl Res2NxR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res2Nx {
        match self.bits {
            false => Res2Nx::Res2NxNo,
            true => Res2Nx::Res2NxYes,
        }
    }
    #[doc = "The result is not rounded."]
    #[inline(always)]
    pub fn is_res2_nx_no(&self) -> bool {
        *self == Res2Nx::Res2NxNo
    }
    #[doc = "The result is rounded, and as a result some digits lost."]
    #[inline(always)]
    pub fn is_res2_nx_yes(&self) -> bool {
        *self == Res2Nx::Res2NxYes
    }
}
#[doc = "Field `RES2_NX` writer - RES2 IEEE Inexact Flag"]
pub type Res2NxW<'a, REG> = crate::BitWriter<'a, REG, Res2Nx>;
impl<'a, REG> Res2NxW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The result is not rounded."]
    #[inline(always)]
    pub fn res2_nx_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res2Nx::Res2NxNo)
    }
    #[doc = "The result is rounded, and as a result some digits lost."]
    #[inline(always)]
    pub fn res2_nx_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res2Nx::Res2NxYes)
    }
}
#[doc = "RES2 IEEE Underflow Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res2Uf {
    #[doc = "0: No tiny non-zero result is detected."]
    Res2UfNo = 0,
    #[doc = "1: A tiny non-zero result is detected."]
    Res2UfYes = 1,
}
impl From<Res2Uf> for bool {
    #[inline(always)]
    fn from(variant: Res2Uf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES2_UF` reader - RES2 IEEE Underflow Flag"]
pub type Res2UfR = crate::BitReader<Res2Uf>;
impl Res2UfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res2Uf {
        match self.bits {
            false => Res2Uf::Res2UfNo,
            true => Res2Uf::Res2UfYes,
        }
    }
    #[doc = "No tiny non-zero result is detected."]
    #[inline(always)]
    pub fn is_res2_uf_no(&self) -> bool {
        *self == Res2Uf::Res2UfNo
    }
    #[doc = "A tiny non-zero result is detected."]
    #[inline(always)]
    pub fn is_res2_uf_yes(&self) -> bool {
        *self == Res2Uf::Res2UfYes
    }
}
#[doc = "Field `RES2_UF` writer - RES2 IEEE Underflow Flag"]
pub type Res2UfW<'a, REG> = crate::BitWriter<'a, REG, Res2Uf>;
impl<'a, REG> Res2UfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No tiny non-zero result is detected."]
    #[inline(always)]
    pub fn res2_uf_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res2Uf::Res2UfNo)
    }
    #[doc = "A tiny non-zero result is detected."]
    #[inline(always)]
    pub fn res2_uf_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res2Uf::Res2UfYes)
    }
}
#[doc = "RES2 IEEE Overflow Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res2Of {
    #[doc = "0: The result format's largest finite number is not exceeded."]
    Res2OfNo = 0,
    #[doc = "1: The result format's largest finite number is exceeded."]
    Res2OfYes = 1,
}
impl From<Res2Of> for bool {
    #[inline(always)]
    fn from(variant: Res2Of) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES2_OF` reader - RES2 IEEE Overflow Flag"]
pub type Res2OfR = crate::BitReader<Res2Of>;
impl Res2OfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res2Of {
        match self.bits {
            false => Res2Of::Res2OfNo,
            true => Res2Of::Res2OfYes,
        }
    }
    #[doc = "The result format's largest finite number is not exceeded."]
    #[inline(always)]
    pub fn is_res2_of_no(&self) -> bool {
        *self == Res2Of::Res2OfNo
    }
    #[doc = "The result format's largest finite number is exceeded."]
    #[inline(always)]
    pub fn is_res2_of_yes(&self) -> bool {
        *self == Res2Of::Res2OfYes
    }
}
#[doc = "Field `RES2_OF` writer - RES2 IEEE Overflow Flag"]
pub type Res2OfW<'a, REG> = crate::BitWriter<'a, REG, Res2Of>;
impl<'a, REG> Res2OfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The result format's largest finite number is not exceeded."]
    #[inline(always)]
    pub fn res2_of_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res2Of::Res2OfNo)
    }
    #[doc = "The result format's largest finite number is exceeded."]
    #[inline(always)]
    pub fn res2_of_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res2Of::Res2OfYes)
    }
}
#[doc = "RES2 IEEE Divide by Zero Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res2Dz {
    #[doc = "0: No exact infinite result is defined for an operation on finite operands."]
    Res2DzNo = 0,
    #[doc = "1: An exact infinite result is defined for an operation on finite operands."]
    Res2DzYes = 1,
}
impl From<Res2Dz> for bool {
    #[inline(always)]
    fn from(variant: Res2Dz) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES2_DZ` reader - RES2 IEEE Divide by Zero Flag"]
pub type Res2DzR = crate::BitReader<Res2Dz>;
impl Res2DzR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res2Dz {
        match self.bits {
            false => Res2Dz::Res2DzNo,
            true => Res2Dz::Res2DzYes,
        }
    }
    #[doc = "No exact infinite result is defined for an operation on finite operands."]
    #[inline(always)]
    pub fn is_res2_dz_no(&self) -> bool {
        *self == Res2Dz::Res2DzNo
    }
    #[doc = "An exact infinite result is defined for an operation on finite operands."]
    #[inline(always)]
    pub fn is_res2_dz_yes(&self) -> bool {
        *self == Res2Dz::Res2DzYes
    }
}
#[doc = "Field `RES2_DZ` writer - RES2 IEEE Divide by Zero Flag"]
pub type Res2DzW<'a, REG> = crate::BitWriter<'a, REG, Res2Dz>;
impl<'a, REG> Res2DzW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No exact infinite result is defined for an operation on finite operands."]
    #[inline(always)]
    pub fn res2_dz_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res2Dz::Res2DzNo)
    }
    #[doc = "An exact infinite result is defined for an operation on finite operands."]
    #[inline(always)]
    pub fn res2_dz_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res2Dz::Res2DzYes)
    }
}
#[doc = "RES2 IEEE Invalid Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res2Nv {
    #[doc = "0: There is usefully definable result."]
    Res2NvNo = 0,
    #[doc = "1: There is no usefully definable result."]
    Res2NvYes = 1,
}
impl From<Res2Nv> for bool {
    #[inline(always)]
    fn from(variant: Res2Nv) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES2_NV` reader - RES2 IEEE Invalid Flag"]
pub type Res2NvR = crate::BitReader<Res2Nv>;
impl Res2NvR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res2Nv {
        match self.bits {
            false => Res2Nv::Res2NvNo,
            true => Res2Nv::Res2NvYes,
        }
    }
    #[doc = "There is usefully definable result."]
    #[inline(always)]
    pub fn is_res2_nv_no(&self) -> bool {
        *self == Res2Nv::Res2NvNo
    }
    #[doc = "There is no usefully definable result."]
    #[inline(always)]
    pub fn is_res2_nv_yes(&self) -> bool {
        *self == Res2Nv::Res2NvYes
    }
}
#[doc = "Field `RES2_NV` writer - RES2 IEEE Invalid Flag"]
pub type Res2NvW<'a, REG> = crate::BitWriter<'a, REG, Res2Nv>;
impl<'a, REG> Res2NvW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "There is usefully definable result."]
    #[inline(always)]
    pub fn res2_nv_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res2Nv::Res2NvNo)
    }
    #[doc = "There is no usefully definable result."]
    #[inline(always)]
    pub fn res2_nv_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res2Nv::Res2NvYes)
    }
}
#[doc = "RES2 Indirect Operation Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res2Err {
    #[doc = "0: No invalid indirect operation is detected."]
    Res2ErrNo = 0,
    #[doc = "1: An invalid indirect operation error is detected."]
    Res2ErrYes = 1,
}
impl From<Res2Err> for bool {
    #[inline(always)]
    fn from(variant: Res2Err) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES2_ERR` reader - RES2 Indirect Operation Error Flag"]
pub type Res2ErrR = crate::BitReader<Res2Err>;
impl Res2ErrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res2Err {
        match self.bits {
            false => Res2Err::Res2ErrNo,
            true => Res2Err::Res2ErrYes,
        }
    }
    #[doc = "No invalid indirect operation is detected."]
    #[inline(always)]
    pub fn is_res2_err_no(&self) -> bool {
        *self == Res2Err::Res2ErrNo
    }
    #[doc = "An invalid indirect operation error is detected."]
    #[inline(always)]
    pub fn is_res2_err_yes(&self) -> bool {
        *self == Res2Err::Res2ErrYes
    }
}
#[doc = "Field `RES2_ERR` writer - RES2 Indirect Operation Error Flag"]
pub type Res2ErrW<'a, REG> = crate::BitWriter<'a, REG, Res2Err>;
impl<'a, REG> Res2ErrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No invalid indirect operation is detected."]
    #[inline(always)]
    pub fn res2_err_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res2Err::Res2ErrNo)
    }
    #[doc = "An invalid indirect operation error is detected."]
    #[inline(always)]
    pub fn res2_err_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res2Err::Res2ErrYes)
    }
}
#[doc = "RES2 Overwrite Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res2Ovwr {
    #[doc = "0: The value of RES2 has been read."]
    Res2OvwrNo = 0,
    #[doc = "1: The value of RES2 has not been read yet and is overwritten by a new MAUWRAP result."]
    Res2OvwrYes = 1,
}
impl From<Res2Ovwr> for bool {
    #[inline(always)]
    fn from(variant: Res2Ovwr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES2_OVWR` reader - RES2 Overwrite Flag"]
pub type Res2OvwrR = crate::BitReader<Res2Ovwr>;
impl Res2OvwrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res2Ovwr {
        match self.bits {
            false => Res2Ovwr::Res2OvwrNo,
            true => Res2Ovwr::Res2OvwrYes,
        }
    }
    #[doc = "The value of RES2 has been read."]
    #[inline(always)]
    pub fn is_res2_ovwr_no(&self) -> bool {
        *self == Res2Ovwr::Res2OvwrNo
    }
    #[doc = "The value of RES2 has not been read yet and is overwritten by a new MAUWRAP result."]
    #[inline(always)]
    pub fn is_res2_ovwr_yes(&self) -> bool {
        *self == Res2Ovwr::Res2OvwrYes
    }
}
#[doc = "Field `RES2_OVWR` writer - RES2 Overwrite Flag"]
pub type Res2OvwrW<'a, REG> = crate::BitWriter<'a, REG, Res2Ovwr>;
impl<'a, REG> Res2OvwrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The value of RES2 has been read."]
    #[inline(always)]
    pub fn res2_ovwr_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res2Ovwr::Res2OvwrNo)
    }
    #[doc = "The value of RES2 has not been read yet and is overwritten by a new MAUWRAP result."]
    #[inline(always)]
    pub fn res2_ovwr_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res2Ovwr::Res2OvwrYes)
    }
}
#[doc = "RES2 Full Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res2Full {
    #[doc = "0: RES2 has not updated and cannot be read."]
    Res2FullNo = 0,
    #[doc = "1: RES2 has updated and can be read."]
    Res2FullYes = 1,
}
impl From<Res2Full> for bool {
    #[inline(always)]
    fn from(variant: Res2Full) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES2_FULL` reader - RES2 Full Flag"]
pub type Res2FullR = crate::BitReader<Res2Full>;
impl Res2FullR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res2Full {
        match self.bits {
            false => Res2Full::Res2FullNo,
            true => Res2Full::Res2FullYes,
        }
    }
    #[doc = "RES2 has not updated and cannot be read."]
    #[inline(always)]
    pub fn is_res2_full_no(&self) -> bool {
        *self == Res2Full::Res2FullNo
    }
    #[doc = "RES2 has updated and can be read."]
    #[inline(always)]
    pub fn is_res2_full_yes(&self) -> bool {
        *self == Res2Full::Res2FullYes
    }
}
#[doc = "Field `RES2_FULL` writer - RES2 Full Flag"]
pub type Res2FullW<'a, REG> = crate::BitWriter<'a, REG, Res2Full>;
impl<'a, REG> Res2FullW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "RES2 has not updated and cannot be read."]
    #[inline(always)]
    pub fn res2_full_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res2Full::Res2FullNo)
    }
    #[doc = "RES2 has updated and can be read."]
    #[inline(always)]
    pub fn res2_full_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res2Full::Res2FullYes)
    }
}
#[doc = "RES3 IEEE Inexact Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res3Nx {
    #[doc = "0: The result is not rounded."]
    Res3NxNo = 0,
    #[doc = "1: The result is rounded, and as a result some digits lost."]
    Res3NxYes = 1,
}
impl From<Res3Nx> for bool {
    #[inline(always)]
    fn from(variant: Res3Nx) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES3_NX` reader - RES3 IEEE Inexact Flag"]
pub type Res3NxR = crate::BitReader<Res3Nx>;
impl Res3NxR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res3Nx {
        match self.bits {
            false => Res3Nx::Res3NxNo,
            true => Res3Nx::Res3NxYes,
        }
    }
    #[doc = "The result is not rounded."]
    #[inline(always)]
    pub fn is_res3_nx_no(&self) -> bool {
        *self == Res3Nx::Res3NxNo
    }
    #[doc = "The result is rounded, and as a result some digits lost."]
    #[inline(always)]
    pub fn is_res3_nx_yes(&self) -> bool {
        *self == Res3Nx::Res3NxYes
    }
}
#[doc = "Field `RES3_NX` writer - RES3 IEEE Inexact Flag"]
pub type Res3NxW<'a, REG> = crate::BitWriter<'a, REG, Res3Nx>;
impl<'a, REG> Res3NxW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The result is not rounded."]
    #[inline(always)]
    pub fn res3_nx_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res3Nx::Res3NxNo)
    }
    #[doc = "The result is rounded, and as a result some digits lost."]
    #[inline(always)]
    pub fn res3_nx_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res3Nx::Res3NxYes)
    }
}
#[doc = "RES3 IEEE Underflow Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res3Uf {
    #[doc = "0: No tiny non-zero result is detected."]
    Res3UfNo = 0,
    #[doc = "1: A tiny non-zero result is detected."]
    Res3UfYes = 1,
}
impl From<Res3Uf> for bool {
    #[inline(always)]
    fn from(variant: Res3Uf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES3_UF` reader - RES3 IEEE Underflow Flag"]
pub type Res3UfR = crate::BitReader<Res3Uf>;
impl Res3UfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res3Uf {
        match self.bits {
            false => Res3Uf::Res3UfNo,
            true => Res3Uf::Res3UfYes,
        }
    }
    #[doc = "No tiny non-zero result is detected."]
    #[inline(always)]
    pub fn is_res3_uf_no(&self) -> bool {
        *self == Res3Uf::Res3UfNo
    }
    #[doc = "A tiny non-zero result is detected."]
    #[inline(always)]
    pub fn is_res3_uf_yes(&self) -> bool {
        *self == Res3Uf::Res3UfYes
    }
}
#[doc = "Field `RES3_UF` writer - RES3 IEEE Underflow Flag"]
pub type Res3UfW<'a, REG> = crate::BitWriter<'a, REG, Res3Uf>;
impl<'a, REG> Res3UfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No tiny non-zero result is detected."]
    #[inline(always)]
    pub fn res3_uf_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res3Uf::Res3UfNo)
    }
    #[doc = "A tiny non-zero result is detected."]
    #[inline(always)]
    pub fn res3_uf_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res3Uf::Res3UfYes)
    }
}
#[doc = "RES3 IEEE Overflow Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res3Of {
    #[doc = "0: The result format's largest finite number is not exceeded."]
    Res3OfNo = 0,
    #[doc = "1: The result format's largest finite number is exceeded."]
    Res3OfYes = 1,
}
impl From<Res3Of> for bool {
    #[inline(always)]
    fn from(variant: Res3Of) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES3_OF` reader - RES3 IEEE Overflow Flag"]
pub type Res3OfR = crate::BitReader<Res3Of>;
impl Res3OfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res3Of {
        match self.bits {
            false => Res3Of::Res3OfNo,
            true => Res3Of::Res3OfYes,
        }
    }
    #[doc = "The result format's largest finite number is not exceeded."]
    #[inline(always)]
    pub fn is_res3_of_no(&self) -> bool {
        *self == Res3Of::Res3OfNo
    }
    #[doc = "The result format's largest finite number is exceeded."]
    #[inline(always)]
    pub fn is_res3_of_yes(&self) -> bool {
        *self == Res3Of::Res3OfYes
    }
}
#[doc = "Field `RES3_OF` writer - RES3 IEEE Overflow Flag"]
pub type Res3OfW<'a, REG> = crate::BitWriter<'a, REG, Res3Of>;
impl<'a, REG> Res3OfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The result format's largest finite number is not exceeded."]
    #[inline(always)]
    pub fn res3_of_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res3Of::Res3OfNo)
    }
    #[doc = "The result format's largest finite number is exceeded."]
    #[inline(always)]
    pub fn res3_of_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res3Of::Res3OfYes)
    }
}
#[doc = "RES3 IEEE Divide by Zero Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res3Dz {
    #[doc = "0: No exact infinite result is defined for an operation on finite operands."]
    Res3DzNo = 0,
    #[doc = "1: An exact infinite result is defined for an operation on finite operands."]
    Res3DzYes = 1,
}
impl From<Res3Dz> for bool {
    #[inline(always)]
    fn from(variant: Res3Dz) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES3_DZ` reader - RES3 IEEE Divide by Zero Flag"]
pub type Res3DzR = crate::BitReader<Res3Dz>;
impl Res3DzR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res3Dz {
        match self.bits {
            false => Res3Dz::Res3DzNo,
            true => Res3Dz::Res3DzYes,
        }
    }
    #[doc = "No exact infinite result is defined for an operation on finite operands."]
    #[inline(always)]
    pub fn is_res3_dz_no(&self) -> bool {
        *self == Res3Dz::Res3DzNo
    }
    #[doc = "An exact infinite result is defined for an operation on finite operands."]
    #[inline(always)]
    pub fn is_res3_dz_yes(&self) -> bool {
        *self == Res3Dz::Res3DzYes
    }
}
#[doc = "Field `RES3_DZ` writer - RES3 IEEE Divide by Zero Flag"]
pub type Res3DzW<'a, REG> = crate::BitWriter<'a, REG, Res3Dz>;
impl<'a, REG> Res3DzW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No exact infinite result is defined for an operation on finite operands."]
    #[inline(always)]
    pub fn res3_dz_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res3Dz::Res3DzNo)
    }
    #[doc = "An exact infinite result is defined for an operation on finite operands."]
    #[inline(always)]
    pub fn res3_dz_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res3Dz::Res3DzYes)
    }
}
#[doc = "RES3 IEEE Invalid Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res3Nv {
    #[doc = "0: There is usefully definable result."]
    Res3NvNo = 0,
    #[doc = "1: There is no usefully definable result."]
    Res3NvYes = 1,
}
impl From<Res3Nv> for bool {
    #[inline(always)]
    fn from(variant: Res3Nv) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES3_NV` reader - RES3 IEEE Invalid Flag"]
pub type Res3NvR = crate::BitReader<Res3Nv>;
impl Res3NvR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res3Nv {
        match self.bits {
            false => Res3Nv::Res3NvNo,
            true => Res3Nv::Res3NvYes,
        }
    }
    #[doc = "There is usefully definable result."]
    #[inline(always)]
    pub fn is_res3_nv_no(&self) -> bool {
        *self == Res3Nv::Res3NvNo
    }
    #[doc = "There is no usefully definable result."]
    #[inline(always)]
    pub fn is_res3_nv_yes(&self) -> bool {
        *self == Res3Nv::Res3NvYes
    }
}
#[doc = "Field `RES3_NV` writer - RES3 IEEE Invalid Flag"]
pub type Res3NvW<'a, REG> = crate::BitWriter<'a, REG, Res3Nv>;
impl<'a, REG> Res3NvW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "There is usefully definable result."]
    #[inline(always)]
    pub fn res3_nv_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res3Nv::Res3NvNo)
    }
    #[doc = "There is no usefully definable result."]
    #[inline(always)]
    pub fn res3_nv_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res3Nv::Res3NvYes)
    }
}
#[doc = "RES3 Indirect Operation Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res3Err {
    #[doc = "0: No invalid indirect operation is detected."]
    Res3ErrNo = 0,
    #[doc = "1: An invalid indirect operation error is detected."]
    Res3ErrYes = 1,
}
impl From<Res3Err> for bool {
    #[inline(always)]
    fn from(variant: Res3Err) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES3_ERR` reader - RES3 Indirect Operation Error Flag"]
pub type Res3ErrR = crate::BitReader<Res3Err>;
impl Res3ErrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res3Err {
        match self.bits {
            false => Res3Err::Res3ErrNo,
            true => Res3Err::Res3ErrYes,
        }
    }
    #[doc = "No invalid indirect operation is detected."]
    #[inline(always)]
    pub fn is_res3_err_no(&self) -> bool {
        *self == Res3Err::Res3ErrNo
    }
    #[doc = "An invalid indirect operation error is detected."]
    #[inline(always)]
    pub fn is_res3_err_yes(&self) -> bool {
        *self == Res3Err::Res3ErrYes
    }
}
#[doc = "Field `RES3_ERR` writer - RES3 Indirect Operation Error Flag"]
pub type Res3ErrW<'a, REG> = crate::BitWriter<'a, REG, Res3Err>;
impl<'a, REG> Res3ErrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No invalid indirect operation is detected."]
    #[inline(always)]
    pub fn res3_err_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res3Err::Res3ErrNo)
    }
    #[doc = "An invalid indirect operation error is detected."]
    #[inline(always)]
    pub fn res3_err_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res3Err::Res3ErrYes)
    }
}
#[doc = "RES3 Overwrite Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res3Ovwr {
    #[doc = "0: The value of RES3 has been read."]
    Res3OvwrNo = 0,
    #[doc = "1: The value of RES3 has not been read yet and is overwritten by a new MAUWRAP result."]
    Res3OvwrYes = 1,
}
impl From<Res3Ovwr> for bool {
    #[inline(always)]
    fn from(variant: Res3Ovwr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES3_OVWR` reader - RES3 Overwrite Flag"]
pub type Res3OvwrR = crate::BitReader<Res3Ovwr>;
impl Res3OvwrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res3Ovwr {
        match self.bits {
            false => Res3Ovwr::Res3OvwrNo,
            true => Res3Ovwr::Res3OvwrYes,
        }
    }
    #[doc = "The value of RES3 has been read."]
    #[inline(always)]
    pub fn is_res3_ovwr_no(&self) -> bool {
        *self == Res3Ovwr::Res3OvwrNo
    }
    #[doc = "The value of RES3 has not been read yet and is overwritten by a new MAUWRAP result."]
    #[inline(always)]
    pub fn is_res3_ovwr_yes(&self) -> bool {
        *self == Res3Ovwr::Res3OvwrYes
    }
}
#[doc = "Field `RES3_OVWR` writer - RES3 Overwrite Flag"]
pub type Res3OvwrW<'a, REG> = crate::BitWriter<'a, REG, Res3Ovwr>;
impl<'a, REG> Res3OvwrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The value of RES3 has been read."]
    #[inline(always)]
    pub fn res3_ovwr_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res3Ovwr::Res3OvwrNo)
    }
    #[doc = "The value of RES3 has not been read yet and is overwritten by a new MAUWRAP result."]
    #[inline(always)]
    pub fn res3_ovwr_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res3Ovwr::Res3OvwrYes)
    }
}
#[doc = "RES3 Full Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res3Full {
    #[doc = "0: RES3 has not updated and cannot be read."]
    Res3FullNo = 0,
    #[doc = "1: RES3 has updated and can be read."]
    Res3FullYes = 1,
}
impl From<Res3Full> for bool {
    #[inline(always)]
    fn from(variant: Res3Full) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES3_FULL` reader - RES3 Full Flag"]
pub type Res3FullR = crate::BitReader<Res3Full>;
impl Res3FullR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res3Full {
        match self.bits {
            false => Res3Full::Res3FullNo,
            true => Res3Full::Res3FullYes,
        }
    }
    #[doc = "RES3 has not updated and cannot be read."]
    #[inline(always)]
    pub fn is_res3_full_no(&self) -> bool {
        *self == Res3Full::Res3FullNo
    }
    #[doc = "RES3 has updated and can be read."]
    #[inline(always)]
    pub fn is_res3_full_yes(&self) -> bool {
        *self == Res3Full::Res3FullYes
    }
}
#[doc = "Field `RES3_FULL` writer - RES3 Full Flag"]
pub type Res3FullW<'a, REG> = crate::BitWriter<'a, REG, Res3Full>;
impl<'a, REG> Res3FullW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "RES3 has not updated and cannot be read."]
    #[inline(always)]
    pub fn res3_full_no(self) -> &'a mut crate::W<REG> {
        self.variant(Res3Full::Res3FullNo)
    }
    #[doc = "RES3 has updated and can be read."]
    #[inline(always)]
    pub fn res3_full_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Res3Full::Res3FullYes)
    }
}
impl R {
    #[doc = "Bit 0 - RES0 IEEE Inexact Flag"]
    #[inline(always)]
    pub fn res0_nx(&self) -> Res0NxR {
        Res0NxR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - RES0 IEEE Underflow Flag"]
    #[inline(always)]
    pub fn res0_uf(&self) -> Res0UfR {
        Res0UfR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - RES0 IEEE Overflow Flag"]
    #[inline(always)]
    pub fn res0_of(&self) -> Res0OfR {
        Res0OfR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - RES0 IEEE Divide by Zero Flag"]
    #[inline(always)]
    pub fn res0_dz(&self) -> Res0DzR {
        Res0DzR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - RES0 IEEE Invalid Flag"]
    #[inline(always)]
    pub fn res0_nv(&self) -> Res0NvR {
        Res0NvR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - RES0 Indirect Operation Error Flag"]
    #[inline(always)]
    pub fn res0_err(&self) -> Res0ErrR {
        Res0ErrR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - RES0 Overwrite Flag"]
    #[inline(always)]
    pub fn res0_ovwr(&self) -> Res0OvwrR {
        Res0OvwrR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - RES0 Full Flag"]
    #[inline(always)]
    pub fn res0_full(&self) -> Res0FullR {
        Res0FullR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - RES1 IEEE Inexact Flag"]
    #[inline(always)]
    pub fn res1_nx(&self) -> Res1NxR {
        Res1NxR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - RES1 IEEE Underflow Flag"]
    #[inline(always)]
    pub fn res1_uf(&self) -> Res1UfR {
        Res1UfR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - RES1 IEEE Overflow Flag"]
    #[inline(always)]
    pub fn res1_of(&self) -> Res1OfR {
        Res1OfR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - RES1 IEEE Divide by Zero Flag"]
    #[inline(always)]
    pub fn res1_dz(&self) -> Res1DzR {
        Res1DzR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - RES1 IEEE Invalid Flag"]
    #[inline(always)]
    pub fn res1_nv(&self) -> Res1NvR {
        Res1NvR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - RES1 Indirect Operation Error Flag"]
    #[inline(always)]
    pub fn res1_err(&self) -> Res1ErrR {
        Res1ErrR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - RES1 Overwrite Flag"]
    #[inline(always)]
    pub fn res1_ovwr(&self) -> Res1OvwrR {
        Res1OvwrR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - RES1 Full Flag"]
    #[inline(always)]
    pub fn res1_full(&self) -> Res1FullR {
        Res1FullR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - RES2 IEEE Inexact Flag"]
    #[inline(always)]
    pub fn res2_nx(&self) -> Res2NxR {
        Res2NxR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - RES2 IEEE Underflow Flag"]
    #[inline(always)]
    pub fn res2_uf(&self) -> Res2UfR {
        Res2UfR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - RES2 IEEE Overflow Flag"]
    #[inline(always)]
    pub fn res2_of(&self) -> Res2OfR {
        Res2OfR::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - RES2 IEEE Divide by Zero Flag"]
    #[inline(always)]
    pub fn res2_dz(&self) -> Res2DzR {
        Res2DzR::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - RES2 IEEE Invalid Flag"]
    #[inline(always)]
    pub fn res2_nv(&self) -> Res2NvR {
        Res2NvR::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - RES2 Indirect Operation Error Flag"]
    #[inline(always)]
    pub fn res2_err(&self) -> Res2ErrR {
        Res2ErrR::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - RES2 Overwrite Flag"]
    #[inline(always)]
    pub fn res2_ovwr(&self) -> Res2OvwrR {
        Res2OvwrR::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - RES2 Full Flag"]
    #[inline(always)]
    pub fn res2_full(&self) -> Res2FullR {
        Res2FullR::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bit 24 - RES3 IEEE Inexact Flag"]
    #[inline(always)]
    pub fn res3_nx(&self) -> Res3NxR {
        Res3NxR::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - RES3 IEEE Underflow Flag"]
    #[inline(always)]
    pub fn res3_uf(&self) -> Res3UfR {
        Res3UfR::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - RES3 IEEE Overflow Flag"]
    #[inline(always)]
    pub fn res3_of(&self) -> Res3OfR {
        Res3OfR::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - RES3 IEEE Divide by Zero Flag"]
    #[inline(always)]
    pub fn res3_dz(&self) -> Res3DzR {
        Res3DzR::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bit 28 - RES3 IEEE Invalid Flag"]
    #[inline(always)]
    pub fn res3_nv(&self) -> Res3NvR {
        Res3NvR::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 29 - RES3 Indirect Operation Error Flag"]
    #[inline(always)]
    pub fn res3_err(&self) -> Res3ErrR {
        Res3ErrR::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - RES3 Overwrite Flag"]
    #[inline(always)]
    pub fn res3_ovwr(&self) -> Res3OvwrR {
        Res3OvwrR::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - RES3 Full Flag"]
    #[inline(always)]
    pub fn res3_full(&self) -> Res3FullR {
        Res3FullR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - RES0 IEEE Inexact Flag"]
    #[inline(always)]
    pub fn res0_nx(&mut self) -> Res0NxW<ResStatusSpec> {
        Res0NxW::new(self, 0)
    }
    #[doc = "Bit 1 - RES0 IEEE Underflow Flag"]
    #[inline(always)]
    pub fn res0_uf(&mut self) -> Res0UfW<ResStatusSpec> {
        Res0UfW::new(self, 1)
    }
    #[doc = "Bit 2 - RES0 IEEE Overflow Flag"]
    #[inline(always)]
    pub fn res0_of(&mut self) -> Res0OfW<ResStatusSpec> {
        Res0OfW::new(self, 2)
    }
    #[doc = "Bit 3 - RES0 IEEE Divide by Zero Flag"]
    #[inline(always)]
    pub fn res0_dz(&mut self) -> Res0DzW<ResStatusSpec> {
        Res0DzW::new(self, 3)
    }
    #[doc = "Bit 4 - RES0 IEEE Invalid Flag"]
    #[inline(always)]
    pub fn res0_nv(&mut self) -> Res0NvW<ResStatusSpec> {
        Res0NvW::new(self, 4)
    }
    #[doc = "Bit 5 - RES0 Indirect Operation Error Flag"]
    #[inline(always)]
    pub fn res0_err(&mut self) -> Res0ErrW<ResStatusSpec> {
        Res0ErrW::new(self, 5)
    }
    #[doc = "Bit 6 - RES0 Overwrite Flag"]
    #[inline(always)]
    pub fn res0_ovwr(&mut self) -> Res0OvwrW<ResStatusSpec> {
        Res0OvwrW::new(self, 6)
    }
    #[doc = "Bit 7 - RES0 Full Flag"]
    #[inline(always)]
    pub fn res0_full(&mut self) -> Res0FullW<ResStatusSpec> {
        Res0FullW::new(self, 7)
    }
    #[doc = "Bit 8 - RES1 IEEE Inexact Flag"]
    #[inline(always)]
    pub fn res1_nx(&mut self) -> Res1NxW<ResStatusSpec> {
        Res1NxW::new(self, 8)
    }
    #[doc = "Bit 9 - RES1 IEEE Underflow Flag"]
    #[inline(always)]
    pub fn res1_uf(&mut self) -> Res1UfW<ResStatusSpec> {
        Res1UfW::new(self, 9)
    }
    #[doc = "Bit 10 - RES1 IEEE Overflow Flag"]
    #[inline(always)]
    pub fn res1_of(&mut self) -> Res1OfW<ResStatusSpec> {
        Res1OfW::new(self, 10)
    }
    #[doc = "Bit 11 - RES1 IEEE Divide by Zero Flag"]
    #[inline(always)]
    pub fn res1_dz(&mut self) -> Res1DzW<ResStatusSpec> {
        Res1DzW::new(self, 11)
    }
    #[doc = "Bit 12 - RES1 IEEE Invalid Flag"]
    #[inline(always)]
    pub fn res1_nv(&mut self) -> Res1NvW<ResStatusSpec> {
        Res1NvW::new(self, 12)
    }
    #[doc = "Bit 13 - RES1 Indirect Operation Error Flag"]
    #[inline(always)]
    pub fn res1_err(&mut self) -> Res1ErrW<ResStatusSpec> {
        Res1ErrW::new(self, 13)
    }
    #[doc = "Bit 14 - RES1 Overwrite Flag"]
    #[inline(always)]
    pub fn res1_ovwr(&mut self) -> Res1OvwrW<ResStatusSpec> {
        Res1OvwrW::new(self, 14)
    }
    #[doc = "Bit 15 - RES1 Full Flag"]
    #[inline(always)]
    pub fn res1_full(&mut self) -> Res1FullW<ResStatusSpec> {
        Res1FullW::new(self, 15)
    }
    #[doc = "Bit 16 - RES2 IEEE Inexact Flag"]
    #[inline(always)]
    pub fn res2_nx(&mut self) -> Res2NxW<ResStatusSpec> {
        Res2NxW::new(self, 16)
    }
    #[doc = "Bit 17 - RES2 IEEE Underflow Flag"]
    #[inline(always)]
    pub fn res2_uf(&mut self) -> Res2UfW<ResStatusSpec> {
        Res2UfW::new(self, 17)
    }
    #[doc = "Bit 18 - RES2 IEEE Overflow Flag"]
    #[inline(always)]
    pub fn res2_of(&mut self) -> Res2OfW<ResStatusSpec> {
        Res2OfW::new(self, 18)
    }
    #[doc = "Bit 19 - RES2 IEEE Divide by Zero Flag"]
    #[inline(always)]
    pub fn res2_dz(&mut self) -> Res2DzW<ResStatusSpec> {
        Res2DzW::new(self, 19)
    }
    #[doc = "Bit 20 - RES2 IEEE Invalid Flag"]
    #[inline(always)]
    pub fn res2_nv(&mut self) -> Res2NvW<ResStatusSpec> {
        Res2NvW::new(self, 20)
    }
    #[doc = "Bit 21 - RES2 Indirect Operation Error Flag"]
    #[inline(always)]
    pub fn res2_err(&mut self) -> Res2ErrW<ResStatusSpec> {
        Res2ErrW::new(self, 21)
    }
    #[doc = "Bit 22 - RES2 Overwrite Flag"]
    #[inline(always)]
    pub fn res2_ovwr(&mut self) -> Res2OvwrW<ResStatusSpec> {
        Res2OvwrW::new(self, 22)
    }
    #[doc = "Bit 23 - RES2 Full Flag"]
    #[inline(always)]
    pub fn res2_full(&mut self) -> Res2FullW<ResStatusSpec> {
        Res2FullW::new(self, 23)
    }
    #[doc = "Bit 24 - RES3 IEEE Inexact Flag"]
    #[inline(always)]
    pub fn res3_nx(&mut self) -> Res3NxW<ResStatusSpec> {
        Res3NxW::new(self, 24)
    }
    #[doc = "Bit 25 - RES3 IEEE Underflow Flag"]
    #[inline(always)]
    pub fn res3_uf(&mut self) -> Res3UfW<ResStatusSpec> {
        Res3UfW::new(self, 25)
    }
    #[doc = "Bit 26 - RES3 IEEE Overflow Flag"]
    #[inline(always)]
    pub fn res3_of(&mut self) -> Res3OfW<ResStatusSpec> {
        Res3OfW::new(self, 26)
    }
    #[doc = "Bit 27 - RES3 IEEE Divide by Zero Flag"]
    #[inline(always)]
    pub fn res3_dz(&mut self) -> Res3DzW<ResStatusSpec> {
        Res3DzW::new(self, 27)
    }
    #[doc = "Bit 28 - RES3 IEEE Invalid Flag"]
    #[inline(always)]
    pub fn res3_nv(&mut self) -> Res3NvW<ResStatusSpec> {
        Res3NvW::new(self, 28)
    }
    #[doc = "Bit 29 - RES3 Indirect Operation Error Flag"]
    #[inline(always)]
    pub fn res3_err(&mut self) -> Res3ErrW<ResStatusSpec> {
        Res3ErrW::new(self, 29)
    }
    #[doc = "Bit 30 - RES3 Overwrite Flag"]
    #[inline(always)]
    pub fn res3_ovwr(&mut self) -> Res3OvwrW<ResStatusSpec> {
        Res3OvwrW::new(self, 30)
    }
    #[doc = "Bit 31 - RES3 Full Flag"]
    #[inline(always)]
    pub fn res3_full(&mut self) -> Res3FullW<ResStatusSpec> {
        Res3FullW::new(self, 31)
    }
}
#[doc = "Result Status\n\nYou can [`read`](crate::Reg::read) this register and get [`res_status::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`res_status::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ResStatusSpec;
impl crate::RegisterSpec for ResStatusSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`res_status::R`](R) reader structure"]
impl crate::Readable for ResStatusSpec {}
#[doc = "`write(|w| ..)` method takes [`res_status::W`](W) writer structure"]
impl crate::Writable for ResStatusSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RES_STATUS to value 0"]
impl crate::Resettable for ResStatusSpec {}

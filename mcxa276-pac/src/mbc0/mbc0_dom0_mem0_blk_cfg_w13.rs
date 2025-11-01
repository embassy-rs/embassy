#[doc = "Register `MBC0_DOM0_MEM0_BLK_CFG_W13` reader"]
pub type R = crate::R<Mbc0Dom0Mem0BlkCfgW13Spec>;
#[doc = "Register `MBC0_DOM0_MEM0_BLK_CFG_W13` writer"]
pub type W = crate::W<Mbc0Dom0Mem0BlkCfgW13Spec>;
#[doc = "Memory Block Access Control Select for block B\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Mbacsel0 {
    #[doc = "0: select MBC_MEMN_GLBAC0 access control policy for block B"]
    Glbac0 = 0,
    #[doc = "1: select MBC_MEMN_GLBAC1 access control policy for block B"]
    Glbac1 = 1,
    #[doc = "2: select MBC_MEMN_GLBAC2 access control policy for block B"]
    Glbac2 = 2,
    #[doc = "3: select MBC_MEMN_GLBAC3 access control policy for block B"]
    Glbac3 = 3,
    #[doc = "4: select MBC_MEMN_GLBAC4 access control policy for block B"]
    Glbac4 = 4,
    #[doc = "5: select MBC_MEMN_GLBAC5 access control policy for block B"]
    Glbac5 = 5,
    #[doc = "6: select MBC_MEMN_GLBAC6 access control policy for block B"]
    Glbac6 = 6,
    #[doc = "7: select MBC_MEMN_GLBAC7 access control policy for block B"]
    Glbac7 = 7,
}
impl From<Mbacsel0> for u8 {
    #[inline(always)]
    fn from(variant: Mbacsel0) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Mbacsel0 {
    type Ux = u8;
}
impl crate::IsEnum for Mbacsel0 {}
#[doc = "Field `MBACSEL0` reader - Memory Block Access Control Select for block B"]
pub type Mbacsel0R = crate::FieldReader<Mbacsel0>;
impl Mbacsel0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mbacsel0 {
        match self.bits {
            0 => Mbacsel0::Glbac0,
            1 => Mbacsel0::Glbac1,
            2 => Mbacsel0::Glbac2,
            3 => Mbacsel0::Glbac3,
            4 => Mbacsel0::Glbac4,
            5 => Mbacsel0::Glbac5,
            6 => Mbacsel0::Glbac6,
            7 => Mbacsel0::Glbac7,
            _ => unreachable!(),
        }
    }
    #[doc = "select MBC_MEMN_GLBAC0 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac0(&self) -> bool {
        *self == Mbacsel0::Glbac0
    }
    #[doc = "select MBC_MEMN_GLBAC1 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac1(&self) -> bool {
        *self == Mbacsel0::Glbac1
    }
    #[doc = "select MBC_MEMN_GLBAC2 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac2(&self) -> bool {
        *self == Mbacsel0::Glbac2
    }
    #[doc = "select MBC_MEMN_GLBAC3 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac3(&self) -> bool {
        *self == Mbacsel0::Glbac3
    }
    #[doc = "select MBC_MEMN_GLBAC4 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac4(&self) -> bool {
        *self == Mbacsel0::Glbac4
    }
    #[doc = "select MBC_MEMN_GLBAC5 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac5(&self) -> bool {
        *self == Mbacsel0::Glbac5
    }
    #[doc = "select MBC_MEMN_GLBAC6 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac6(&self) -> bool {
        *self == Mbacsel0::Glbac6
    }
    #[doc = "select MBC_MEMN_GLBAC7 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac7(&self) -> bool {
        *self == Mbacsel0::Glbac7
    }
}
#[doc = "Field `MBACSEL0` writer - Memory Block Access Control Select for block B"]
pub type Mbacsel0W<'a, REG> = crate::FieldWriter<'a, REG, 3, Mbacsel0, crate::Safe>;
impl<'a, REG> Mbacsel0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "select MBC_MEMN_GLBAC0 access control policy for block B"]
    #[inline(always)]
    pub fn glbac0(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel0::Glbac0)
    }
    #[doc = "select MBC_MEMN_GLBAC1 access control policy for block B"]
    #[inline(always)]
    pub fn glbac1(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel0::Glbac1)
    }
    #[doc = "select MBC_MEMN_GLBAC2 access control policy for block B"]
    #[inline(always)]
    pub fn glbac2(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel0::Glbac2)
    }
    #[doc = "select MBC_MEMN_GLBAC3 access control policy for block B"]
    #[inline(always)]
    pub fn glbac3(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel0::Glbac3)
    }
    #[doc = "select MBC_MEMN_GLBAC4 access control policy for block B"]
    #[inline(always)]
    pub fn glbac4(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel0::Glbac4)
    }
    #[doc = "select MBC_MEMN_GLBAC5 access control policy for block B"]
    #[inline(always)]
    pub fn glbac5(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel0::Glbac5)
    }
    #[doc = "select MBC_MEMN_GLBAC6 access control policy for block B"]
    #[inline(always)]
    pub fn glbac6(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel0::Glbac6)
    }
    #[doc = "select MBC_MEMN_GLBAC7 access control policy for block B"]
    #[inline(always)]
    pub fn glbac7(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel0::Glbac7)
    }
}
#[doc = "NonSecure Enable for block B\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nse0 {
    #[doc = "0: Secure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\]), nonsecure accesses to block B are not allowed."]
    Allowed = 0,
    #[doc = "1: Secure accesses to block B are not allowed, nonsecure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\])."]
    Notallowed = 1,
}
impl From<Nse0> for bool {
    #[inline(always)]
    fn from(variant: Nse0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NSE0` reader - NonSecure Enable for block B"]
pub type Nse0R = crate::BitReader<Nse0>;
impl Nse0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nse0 {
        match self.bits {
            false => Nse0::Allowed,
            true => Nse0::Notallowed,
        }
    }
    #[doc = "Secure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\]), nonsecure accesses to block B are not allowed."]
    #[inline(always)]
    pub fn is_allowed(&self) -> bool {
        *self == Nse0::Allowed
    }
    #[doc = "Secure accesses to block B are not allowed, nonsecure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\])."]
    #[inline(always)]
    pub fn is_notallowed(&self) -> bool {
        *self == Nse0::Notallowed
    }
}
#[doc = "Field `NSE0` writer - NonSecure Enable for block B"]
pub type Nse0W<'a, REG> = crate::BitWriter<'a, REG, Nse0>;
impl<'a, REG> Nse0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Secure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\]), nonsecure accesses to block B are not allowed."]
    #[inline(always)]
    pub fn allowed(self) -> &'a mut crate::W<REG> {
        self.variant(Nse0::Allowed)
    }
    #[doc = "Secure accesses to block B are not allowed, nonsecure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\])."]
    #[inline(always)]
    pub fn notallowed(self) -> &'a mut crate::W<REG> {
        self.variant(Nse0::Notallowed)
    }
}
#[doc = "Memory Block Access Control Select for block B\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Mbacsel1 {
    #[doc = "0: select MBC_MEMN_GLBAC0 access control policy for block B"]
    Glbac0 = 0,
    #[doc = "1: select MBC_MEMN_GLBAC1 access control policy for block B"]
    Glbac1 = 1,
    #[doc = "2: select MBC_MEMN_GLBAC2 access control policy for block B"]
    Glbac2 = 2,
    #[doc = "3: select MBC_MEMN_GLBAC3 access control policy for block B"]
    Glbac3 = 3,
    #[doc = "4: select MBC_MEMN_GLBAC4 access control policy for block B"]
    Glbac4 = 4,
    #[doc = "5: select MBC_MEMN_GLBAC5 access control policy for block B"]
    Glbac5 = 5,
    #[doc = "6: select MBC_MEMN_GLBAC6 access control policy for block B"]
    Glbac6 = 6,
    #[doc = "7: select MBC_MEMN_GLBAC7 access control policy for block B"]
    Glbac7 = 7,
}
impl From<Mbacsel1> for u8 {
    #[inline(always)]
    fn from(variant: Mbacsel1) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Mbacsel1 {
    type Ux = u8;
}
impl crate::IsEnum for Mbacsel1 {}
#[doc = "Field `MBACSEL1` reader - Memory Block Access Control Select for block B"]
pub type Mbacsel1R = crate::FieldReader<Mbacsel1>;
impl Mbacsel1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mbacsel1 {
        match self.bits {
            0 => Mbacsel1::Glbac0,
            1 => Mbacsel1::Glbac1,
            2 => Mbacsel1::Glbac2,
            3 => Mbacsel1::Glbac3,
            4 => Mbacsel1::Glbac4,
            5 => Mbacsel1::Glbac5,
            6 => Mbacsel1::Glbac6,
            7 => Mbacsel1::Glbac7,
            _ => unreachable!(),
        }
    }
    #[doc = "select MBC_MEMN_GLBAC0 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac0(&self) -> bool {
        *self == Mbacsel1::Glbac0
    }
    #[doc = "select MBC_MEMN_GLBAC1 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac1(&self) -> bool {
        *self == Mbacsel1::Glbac1
    }
    #[doc = "select MBC_MEMN_GLBAC2 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac2(&self) -> bool {
        *self == Mbacsel1::Glbac2
    }
    #[doc = "select MBC_MEMN_GLBAC3 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac3(&self) -> bool {
        *self == Mbacsel1::Glbac3
    }
    #[doc = "select MBC_MEMN_GLBAC4 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac4(&self) -> bool {
        *self == Mbacsel1::Glbac4
    }
    #[doc = "select MBC_MEMN_GLBAC5 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac5(&self) -> bool {
        *self == Mbacsel1::Glbac5
    }
    #[doc = "select MBC_MEMN_GLBAC6 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac6(&self) -> bool {
        *self == Mbacsel1::Glbac6
    }
    #[doc = "select MBC_MEMN_GLBAC7 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac7(&self) -> bool {
        *self == Mbacsel1::Glbac7
    }
}
#[doc = "Field `MBACSEL1` writer - Memory Block Access Control Select for block B"]
pub type Mbacsel1W<'a, REG> = crate::FieldWriter<'a, REG, 3, Mbacsel1, crate::Safe>;
impl<'a, REG> Mbacsel1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "select MBC_MEMN_GLBAC0 access control policy for block B"]
    #[inline(always)]
    pub fn glbac0(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel1::Glbac0)
    }
    #[doc = "select MBC_MEMN_GLBAC1 access control policy for block B"]
    #[inline(always)]
    pub fn glbac1(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel1::Glbac1)
    }
    #[doc = "select MBC_MEMN_GLBAC2 access control policy for block B"]
    #[inline(always)]
    pub fn glbac2(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel1::Glbac2)
    }
    #[doc = "select MBC_MEMN_GLBAC3 access control policy for block B"]
    #[inline(always)]
    pub fn glbac3(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel1::Glbac3)
    }
    #[doc = "select MBC_MEMN_GLBAC4 access control policy for block B"]
    #[inline(always)]
    pub fn glbac4(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel1::Glbac4)
    }
    #[doc = "select MBC_MEMN_GLBAC5 access control policy for block B"]
    #[inline(always)]
    pub fn glbac5(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel1::Glbac5)
    }
    #[doc = "select MBC_MEMN_GLBAC6 access control policy for block B"]
    #[inline(always)]
    pub fn glbac6(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel1::Glbac6)
    }
    #[doc = "select MBC_MEMN_GLBAC7 access control policy for block B"]
    #[inline(always)]
    pub fn glbac7(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel1::Glbac7)
    }
}
#[doc = "NonSecure Enable for block B\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nse1 {
    #[doc = "0: Secure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\]), nonsecure accesses to block B are not allowed."]
    Allowed = 0,
    #[doc = "1: Secure accesses to block B are not allowed, nonsecure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\])."]
    Notallowed = 1,
}
impl From<Nse1> for bool {
    #[inline(always)]
    fn from(variant: Nse1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NSE1` reader - NonSecure Enable for block B"]
pub type Nse1R = crate::BitReader<Nse1>;
impl Nse1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nse1 {
        match self.bits {
            false => Nse1::Allowed,
            true => Nse1::Notallowed,
        }
    }
    #[doc = "Secure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\]), nonsecure accesses to block B are not allowed."]
    #[inline(always)]
    pub fn is_allowed(&self) -> bool {
        *self == Nse1::Allowed
    }
    #[doc = "Secure accesses to block B are not allowed, nonsecure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\])."]
    #[inline(always)]
    pub fn is_notallowed(&self) -> bool {
        *self == Nse1::Notallowed
    }
}
#[doc = "Field `NSE1` writer - NonSecure Enable for block B"]
pub type Nse1W<'a, REG> = crate::BitWriter<'a, REG, Nse1>;
impl<'a, REG> Nse1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Secure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\]), nonsecure accesses to block B are not allowed."]
    #[inline(always)]
    pub fn allowed(self) -> &'a mut crate::W<REG> {
        self.variant(Nse1::Allowed)
    }
    #[doc = "Secure accesses to block B are not allowed, nonsecure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\])."]
    #[inline(always)]
    pub fn notallowed(self) -> &'a mut crate::W<REG> {
        self.variant(Nse1::Notallowed)
    }
}
#[doc = "Memory Block Access Control Select for block B\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Mbacsel2 {
    #[doc = "0: select MBC_MEMN_GLBAC0 access control policy for block B"]
    Glbac0 = 0,
    #[doc = "1: select MBC_MEMN_GLBAC1 access control policy for block B"]
    Glbac1 = 1,
    #[doc = "2: select MBC_MEMN_GLBAC2 access control policy for block B"]
    Glbac2 = 2,
    #[doc = "3: select MBC_MEMN_GLBAC3 access control policy for block B"]
    Glbac3 = 3,
    #[doc = "4: select MBC_MEMN_GLBAC4 access control policy for block B"]
    Glbac4 = 4,
    #[doc = "5: select MBC_MEMN_GLBAC5 access control policy for block B"]
    Glbac5 = 5,
    #[doc = "6: select MBC_MEMN_GLBAC6 access control policy for block B"]
    Glbac6 = 6,
    #[doc = "7: select MBC_MEMN_GLBAC7 access control policy for block B"]
    Glbac7 = 7,
}
impl From<Mbacsel2> for u8 {
    #[inline(always)]
    fn from(variant: Mbacsel2) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Mbacsel2 {
    type Ux = u8;
}
impl crate::IsEnum for Mbacsel2 {}
#[doc = "Field `MBACSEL2` reader - Memory Block Access Control Select for block B"]
pub type Mbacsel2R = crate::FieldReader<Mbacsel2>;
impl Mbacsel2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mbacsel2 {
        match self.bits {
            0 => Mbacsel2::Glbac0,
            1 => Mbacsel2::Glbac1,
            2 => Mbacsel2::Glbac2,
            3 => Mbacsel2::Glbac3,
            4 => Mbacsel2::Glbac4,
            5 => Mbacsel2::Glbac5,
            6 => Mbacsel2::Glbac6,
            7 => Mbacsel2::Glbac7,
            _ => unreachable!(),
        }
    }
    #[doc = "select MBC_MEMN_GLBAC0 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac0(&self) -> bool {
        *self == Mbacsel2::Glbac0
    }
    #[doc = "select MBC_MEMN_GLBAC1 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac1(&self) -> bool {
        *self == Mbacsel2::Glbac1
    }
    #[doc = "select MBC_MEMN_GLBAC2 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac2(&self) -> bool {
        *self == Mbacsel2::Glbac2
    }
    #[doc = "select MBC_MEMN_GLBAC3 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac3(&self) -> bool {
        *self == Mbacsel2::Glbac3
    }
    #[doc = "select MBC_MEMN_GLBAC4 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac4(&self) -> bool {
        *self == Mbacsel2::Glbac4
    }
    #[doc = "select MBC_MEMN_GLBAC5 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac5(&self) -> bool {
        *self == Mbacsel2::Glbac5
    }
    #[doc = "select MBC_MEMN_GLBAC6 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac6(&self) -> bool {
        *self == Mbacsel2::Glbac6
    }
    #[doc = "select MBC_MEMN_GLBAC7 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac7(&self) -> bool {
        *self == Mbacsel2::Glbac7
    }
}
#[doc = "Field `MBACSEL2` writer - Memory Block Access Control Select for block B"]
pub type Mbacsel2W<'a, REG> = crate::FieldWriter<'a, REG, 3, Mbacsel2, crate::Safe>;
impl<'a, REG> Mbacsel2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "select MBC_MEMN_GLBAC0 access control policy for block B"]
    #[inline(always)]
    pub fn glbac0(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel2::Glbac0)
    }
    #[doc = "select MBC_MEMN_GLBAC1 access control policy for block B"]
    #[inline(always)]
    pub fn glbac1(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel2::Glbac1)
    }
    #[doc = "select MBC_MEMN_GLBAC2 access control policy for block B"]
    #[inline(always)]
    pub fn glbac2(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel2::Glbac2)
    }
    #[doc = "select MBC_MEMN_GLBAC3 access control policy for block B"]
    #[inline(always)]
    pub fn glbac3(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel2::Glbac3)
    }
    #[doc = "select MBC_MEMN_GLBAC4 access control policy for block B"]
    #[inline(always)]
    pub fn glbac4(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel2::Glbac4)
    }
    #[doc = "select MBC_MEMN_GLBAC5 access control policy for block B"]
    #[inline(always)]
    pub fn glbac5(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel2::Glbac5)
    }
    #[doc = "select MBC_MEMN_GLBAC6 access control policy for block B"]
    #[inline(always)]
    pub fn glbac6(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel2::Glbac6)
    }
    #[doc = "select MBC_MEMN_GLBAC7 access control policy for block B"]
    #[inline(always)]
    pub fn glbac7(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel2::Glbac7)
    }
}
#[doc = "NonSecure Enable for block B\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nse2 {
    #[doc = "0: Secure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\]), nonsecure accesses to block B are not allowed."]
    Allowed = 0,
    #[doc = "1: Secure accesses to block B are not allowed, nonsecure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\])."]
    Notallowed = 1,
}
impl From<Nse2> for bool {
    #[inline(always)]
    fn from(variant: Nse2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NSE2` reader - NonSecure Enable for block B"]
pub type Nse2R = crate::BitReader<Nse2>;
impl Nse2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nse2 {
        match self.bits {
            false => Nse2::Allowed,
            true => Nse2::Notallowed,
        }
    }
    #[doc = "Secure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\]), nonsecure accesses to block B are not allowed."]
    #[inline(always)]
    pub fn is_allowed(&self) -> bool {
        *self == Nse2::Allowed
    }
    #[doc = "Secure accesses to block B are not allowed, nonsecure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\])."]
    #[inline(always)]
    pub fn is_notallowed(&self) -> bool {
        *self == Nse2::Notallowed
    }
}
#[doc = "Field `NSE2` writer - NonSecure Enable for block B"]
pub type Nse2W<'a, REG> = crate::BitWriter<'a, REG, Nse2>;
impl<'a, REG> Nse2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Secure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\]), nonsecure accesses to block B are not allowed."]
    #[inline(always)]
    pub fn allowed(self) -> &'a mut crate::W<REG> {
        self.variant(Nse2::Allowed)
    }
    #[doc = "Secure accesses to block B are not allowed, nonsecure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\])."]
    #[inline(always)]
    pub fn notallowed(self) -> &'a mut crate::W<REG> {
        self.variant(Nse2::Notallowed)
    }
}
#[doc = "Memory Block Access Control Select for block B\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Mbacsel3 {
    #[doc = "0: select MBC_MEMN_GLBAC0 access control policy for block B"]
    Glbac0 = 0,
    #[doc = "1: select MBC_MEMN_GLBAC1 access control policy for block B"]
    Glbac1 = 1,
    #[doc = "2: select MBC_MEMN_GLBAC2 access control policy for block B"]
    Glbac2 = 2,
    #[doc = "3: select MBC_MEMN_GLBAC3 access control policy for block B"]
    Glbac3 = 3,
    #[doc = "4: select MBC_MEMN_GLBAC4 access control policy for block B"]
    Glbac4 = 4,
    #[doc = "5: select MBC_MEMN_GLBAC5 access control policy for block B"]
    Glbac5 = 5,
    #[doc = "6: select MBC_MEMN_GLBAC6 access control policy for block B"]
    Glbac6 = 6,
    #[doc = "7: select MBC_MEMN_GLBAC7 access control policy for block B"]
    Glbac7 = 7,
}
impl From<Mbacsel3> for u8 {
    #[inline(always)]
    fn from(variant: Mbacsel3) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Mbacsel3 {
    type Ux = u8;
}
impl crate::IsEnum for Mbacsel3 {}
#[doc = "Field `MBACSEL3` reader - Memory Block Access Control Select for block B"]
pub type Mbacsel3R = crate::FieldReader<Mbacsel3>;
impl Mbacsel3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mbacsel3 {
        match self.bits {
            0 => Mbacsel3::Glbac0,
            1 => Mbacsel3::Glbac1,
            2 => Mbacsel3::Glbac2,
            3 => Mbacsel3::Glbac3,
            4 => Mbacsel3::Glbac4,
            5 => Mbacsel3::Glbac5,
            6 => Mbacsel3::Glbac6,
            7 => Mbacsel3::Glbac7,
            _ => unreachable!(),
        }
    }
    #[doc = "select MBC_MEMN_GLBAC0 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac0(&self) -> bool {
        *self == Mbacsel3::Glbac0
    }
    #[doc = "select MBC_MEMN_GLBAC1 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac1(&self) -> bool {
        *self == Mbacsel3::Glbac1
    }
    #[doc = "select MBC_MEMN_GLBAC2 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac2(&self) -> bool {
        *self == Mbacsel3::Glbac2
    }
    #[doc = "select MBC_MEMN_GLBAC3 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac3(&self) -> bool {
        *self == Mbacsel3::Glbac3
    }
    #[doc = "select MBC_MEMN_GLBAC4 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac4(&self) -> bool {
        *self == Mbacsel3::Glbac4
    }
    #[doc = "select MBC_MEMN_GLBAC5 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac5(&self) -> bool {
        *self == Mbacsel3::Glbac5
    }
    #[doc = "select MBC_MEMN_GLBAC6 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac6(&self) -> bool {
        *self == Mbacsel3::Glbac6
    }
    #[doc = "select MBC_MEMN_GLBAC7 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac7(&self) -> bool {
        *self == Mbacsel3::Glbac7
    }
}
#[doc = "Field `MBACSEL3` writer - Memory Block Access Control Select for block B"]
pub type Mbacsel3W<'a, REG> = crate::FieldWriter<'a, REG, 3, Mbacsel3, crate::Safe>;
impl<'a, REG> Mbacsel3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "select MBC_MEMN_GLBAC0 access control policy for block B"]
    #[inline(always)]
    pub fn glbac0(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel3::Glbac0)
    }
    #[doc = "select MBC_MEMN_GLBAC1 access control policy for block B"]
    #[inline(always)]
    pub fn glbac1(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel3::Glbac1)
    }
    #[doc = "select MBC_MEMN_GLBAC2 access control policy for block B"]
    #[inline(always)]
    pub fn glbac2(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel3::Glbac2)
    }
    #[doc = "select MBC_MEMN_GLBAC3 access control policy for block B"]
    #[inline(always)]
    pub fn glbac3(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel3::Glbac3)
    }
    #[doc = "select MBC_MEMN_GLBAC4 access control policy for block B"]
    #[inline(always)]
    pub fn glbac4(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel3::Glbac4)
    }
    #[doc = "select MBC_MEMN_GLBAC5 access control policy for block B"]
    #[inline(always)]
    pub fn glbac5(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel3::Glbac5)
    }
    #[doc = "select MBC_MEMN_GLBAC6 access control policy for block B"]
    #[inline(always)]
    pub fn glbac6(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel3::Glbac6)
    }
    #[doc = "select MBC_MEMN_GLBAC7 access control policy for block B"]
    #[inline(always)]
    pub fn glbac7(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel3::Glbac7)
    }
}
#[doc = "NonSecure Enable for block B\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nse3 {
    #[doc = "0: Secure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\]), nonsecure accesses to block B are not allowed."]
    Allowed = 0,
    #[doc = "1: Secure accesses to block B are not allowed, nonsecure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\])."]
    Notallowed = 1,
}
impl From<Nse3> for bool {
    #[inline(always)]
    fn from(variant: Nse3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NSE3` reader - NonSecure Enable for block B"]
pub type Nse3R = crate::BitReader<Nse3>;
impl Nse3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nse3 {
        match self.bits {
            false => Nse3::Allowed,
            true => Nse3::Notallowed,
        }
    }
    #[doc = "Secure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\]), nonsecure accesses to block B are not allowed."]
    #[inline(always)]
    pub fn is_allowed(&self) -> bool {
        *self == Nse3::Allowed
    }
    #[doc = "Secure accesses to block B are not allowed, nonsecure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\])."]
    #[inline(always)]
    pub fn is_notallowed(&self) -> bool {
        *self == Nse3::Notallowed
    }
}
#[doc = "Field `NSE3` writer - NonSecure Enable for block B"]
pub type Nse3W<'a, REG> = crate::BitWriter<'a, REG, Nse3>;
impl<'a, REG> Nse3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Secure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\]), nonsecure accesses to block B are not allowed."]
    #[inline(always)]
    pub fn allowed(self) -> &'a mut crate::W<REG> {
        self.variant(Nse3::Allowed)
    }
    #[doc = "Secure accesses to block B are not allowed, nonsecure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\])."]
    #[inline(always)]
    pub fn notallowed(self) -> &'a mut crate::W<REG> {
        self.variant(Nse3::Notallowed)
    }
}
#[doc = "Memory Block Access Control Select for block B\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Mbacsel4 {
    #[doc = "0: select MBC_MEMN_GLBAC0 access control policy for block B"]
    Glbac0 = 0,
    #[doc = "1: select MBC_MEMN_GLBAC1 access control policy for block B"]
    Glbac1 = 1,
    #[doc = "2: select MBC_MEMN_GLBAC2 access control policy for block B"]
    Glbac2 = 2,
    #[doc = "3: select MBC_MEMN_GLBAC3 access control policy for block B"]
    Glbac3 = 3,
    #[doc = "4: select MBC_MEMN_GLBAC4 access control policy for block B"]
    Glbac4 = 4,
    #[doc = "5: select MBC_MEMN_GLBAC5 access control policy for block B"]
    Glbac5 = 5,
    #[doc = "6: select MBC_MEMN_GLBAC6 access control policy for block B"]
    Glbac6 = 6,
    #[doc = "7: select MBC_MEMN_GLBAC7 access control policy for block B"]
    Glbac7 = 7,
}
impl From<Mbacsel4> for u8 {
    #[inline(always)]
    fn from(variant: Mbacsel4) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Mbacsel4 {
    type Ux = u8;
}
impl crate::IsEnum for Mbacsel4 {}
#[doc = "Field `MBACSEL4` reader - Memory Block Access Control Select for block B"]
pub type Mbacsel4R = crate::FieldReader<Mbacsel4>;
impl Mbacsel4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mbacsel4 {
        match self.bits {
            0 => Mbacsel4::Glbac0,
            1 => Mbacsel4::Glbac1,
            2 => Mbacsel4::Glbac2,
            3 => Mbacsel4::Glbac3,
            4 => Mbacsel4::Glbac4,
            5 => Mbacsel4::Glbac5,
            6 => Mbacsel4::Glbac6,
            7 => Mbacsel4::Glbac7,
            _ => unreachable!(),
        }
    }
    #[doc = "select MBC_MEMN_GLBAC0 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac0(&self) -> bool {
        *self == Mbacsel4::Glbac0
    }
    #[doc = "select MBC_MEMN_GLBAC1 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac1(&self) -> bool {
        *self == Mbacsel4::Glbac1
    }
    #[doc = "select MBC_MEMN_GLBAC2 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac2(&self) -> bool {
        *self == Mbacsel4::Glbac2
    }
    #[doc = "select MBC_MEMN_GLBAC3 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac3(&self) -> bool {
        *self == Mbacsel4::Glbac3
    }
    #[doc = "select MBC_MEMN_GLBAC4 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac4(&self) -> bool {
        *self == Mbacsel4::Glbac4
    }
    #[doc = "select MBC_MEMN_GLBAC5 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac5(&self) -> bool {
        *self == Mbacsel4::Glbac5
    }
    #[doc = "select MBC_MEMN_GLBAC6 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac6(&self) -> bool {
        *self == Mbacsel4::Glbac6
    }
    #[doc = "select MBC_MEMN_GLBAC7 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac7(&self) -> bool {
        *self == Mbacsel4::Glbac7
    }
}
#[doc = "Field `MBACSEL4` writer - Memory Block Access Control Select for block B"]
pub type Mbacsel4W<'a, REG> = crate::FieldWriter<'a, REG, 3, Mbacsel4, crate::Safe>;
impl<'a, REG> Mbacsel4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "select MBC_MEMN_GLBAC0 access control policy for block B"]
    #[inline(always)]
    pub fn glbac0(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel4::Glbac0)
    }
    #[doc = "select MBC_MEMN_GLBAC1 access control policy for block B"]
    #[inline(always)]
    pub fn glbac1(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel4::Glbac1)
    }
    #[doc = "select MBC_MEMN_GLBAC2 access control policy for block B"]
    #[inline(always)]
    pub fn glbac2(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel4::Glbac2)
    }
    #[doc = "select MBC_MEMN_GLBAC3 access control policy for block B"]
    #[inline(always)]
    pub fn glbac3(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel4::Glbac3)
    }
    #[doc = "select MBC_MEMN_GLBAC4 access control policy for block B"]
    #[inline(always)]
    pub fn glbac4(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel4::Glbac4)
    }
    #[doc = "select MBC_MEMN_GLBAC5 access control policy for block B"]
    #[inline(always)]
    pub fn glbac5(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel4::Glbac5)
    }
    #[doc = "select MBC_MEMN_GLBAC6 access control policy for block B"]
    #[inline(always)]
    pub fn glbac6(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel4::Glbac6)
    }
    #[doc = "select MBC_MEMN_GLBAC7 access control policy for block B"]
    #[inline(always)]
    pub fn glbac7(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel4::Glbac7)
    }
}
#[doc = "NonSecure Enable for block B\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nse4 {
    #[doc = "0: Secure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\]), nonsecure accesses to block B are not allowed."]
    Allowed = 0,
    #[doc = "1: Secure accesses to block B are not allowed, nonsecure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\])."]
    Notallowed = 1,
}
impl From<Nse4> for bool {
    #[inline(always)]
    fn from(variant: Nse4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NSE4` reader - NonSecure Enable for block B"]
pub type Nse4R = crate::BitReader<Nse4>;
impl Nse4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nse4 {
        match self.bits {
            false => Nse4::Allowed,
            true => Nse4::Notallowed,
        }
    }
    #[doc = "Secure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\]), nonsecure accesses to block B are not allowed."]
    #[inline(always)]
    pub fn is_allowed(&self) -> bool {
        *self == Nse4::Allowed
    }
    #[doc = "Secure accesses to block B are not allowed, nonsecure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\])."]
    #[inline(always)]
    pub fn is_notallowed(&self) -> bool {
        *self == Nse4::Notallowed
    }
}
#[doc = "Field `NSE4` writer - NonSecure Enable for block B"]
pub type Nse4W<'a, REG> = crate::BitWriter<'a, REG, Nse4>;
impl<'a, REG> Nse4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Secure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\]), nonsecure accesses to block B are not allowed."]
    #[inline(always)]
    pub fn allowed(self) -> &'a mut crate::W<REG> {
        self.variant(Nse4::Allowed)
    }
    #[doc = "Secure accesses to block B are not allowed, nonsecure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\])."]
    #[inline(always)]
    pub fn notallowed(self) -> &'a mut crate::W<REG> {
        self.variant(Nse4::Notallowed)
    }
}
#[doc = "Memory Block Access Control Select for block B\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Mbacsel5 {
    #[doc = "0: select MBC_MEMN_GLBAC0 access control policy for block B"]
    Glbac0 = 0,
    #[doc = "1: select MBC_MEMN_GLBAC1 access control policy for block B"]
    Glbac1 = 1,
    #[doc = "2: select MBC_MEMN_GLBAC2 access control policy for block B"]
    Glbac2 = 2,
    #[doc = "3: select MBC_MEMN_GLBAC3 access control policy for block B"]
    Glbac3 = 3,
    #[doc = "4: select MBC_MEMN_GLBAC4 access control policy for block B"]
    Glbac4 = 4,
    #[doc = "5: select MBC_MEMN_GLBAC5 access control policy for block B"]
    Glbac5 = 5,
    #[doc = "6: select MBC_MEMN_GLBAC6 access control policy for block B"]
    Glbac6 = 6,
    #[doc = "7: select MBC_MEMN_GLBAC7 access control policy for block B"]
    Glbac7 = 7,
}
impl From<Mbacsel5> for u8 {
    #[inline(always)]
    fn from(variant: Mbacsel5) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Mbacsel5 {
    type Ux = u8;
}
impl crate::IsEnum for Mbacsel5 {}
#[doc = "Field `MBACSEL5` reader - Memory Block Access Control Select for block B"]
pub type Mbacsel5R = crate::FieldReader<Mbacsel5>;
impl Mbacsel5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mbacsel5 {
        match self.bits {
            0 => Mbacsel5::Glbac0,
            1 => Mbacsel5::Glbac1,
            2 => Mbacsel5::Glbac2,
            3 => Mbacsel5::Glbac3,
            4 => Mbacsel5::Glbac4,
            5 => Mbacsel5::Glbac5,
            6 => Mbacsel5::Glbac6,
            7 => Mbacsel5::Glbac7,
            _ => unreachable!(),
        }
    }
    #[doc = "select MBC_MEMN_GLBAC0 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac0(&self) -> bool {
        *self == Mbacsel5::Glbac0
    }
    #[doc = "select MBC_MEMN_GLBAC1 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac1(&self) -> bool {
        *self == Mbacsel5::Glbac1
    }
    #[doc = "select MBC_MEMN_GLBAC2 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac2(&self) -> bool {
        *self == Mbacsel5::Glbac2
    }
    #[doc = "select MBC_MEMN_GLBAC3 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac3(&self) -> bool {
        *self == Mbacsel5::Glbac3
    }
    #[doc = "select MBC_MEMN_GLBAC4 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac4(&self) -> bool {
        *self == Mbacsel5::Glbac4
    }
    #[doc = "select MBC_MEMN_GLBAC5 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac5(&self) -> bool {
        *self == Mbacsel5::Glbac5
    }
    #[doc = "select MBC_MEMN_GLBAC6 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac6(&self) -> bool {
        *self == Mbacsel5::Glbac6
    }
    #[doc = "select MBC_MEMN_GLBAC7 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac7(&self) -> bool {
        *self == Mbacsel5::Glbac7
    }
}
#[doc = "Field `MBACSEL5` writer - Memory Block Access Control Select for block B"]
pub type Mbacsel5W<'a, REG> = crate::FieldWriter<'a, REG, 3, Mbacsel5, crate::Safe>;
impl<'a, REG> Mbacsel5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "select MBC_MEMN_GLBAC0 access control policy for block B"]
    #[inline(always)]
    pub fn glbac0(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel5::Glbac0)
    }
    #[doc = "select MBC_MEMN_GLBAC1 access control policy for block B"]
    #[inline(always)]
    pub fn glbac1(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel5::Glbac1)
    }
    #[doc = "select MBC_MEMN_GLBAC2 access control policy for block B"]
    #[inline(always)]
    pub fn glbac2(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel5::Glbac2)
    }
    #[doc = "select MBC_MEMN_GLBAC3 access control policy for block B"]
    #[inline(always)]
    pub fn glbac3(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel5::Glbac3)
    }
    #[doc = "select MBC_MEMN_GLBAC4 access control policy for block B"]
    #[inline(always)]
    pub fn glbac4(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel5::Glbac4)
    }
    #[doc = "select MBC_MEMN_GLBAC5 access control policy for block B"]
    #[inline(always)]
    pub fn glbac5(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel5::Glbac5)
    }
    #[doc = "select MBC_MEMN_GLBAC6 access control policy for block B"]
    #[inline(always)]
    pub fn glbac6(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel5::Glbac6)
    }
    #[doc = "select MBC_MEMN_GLBAC7 access control policy for block B"]
    #[inline(always)]
    pub fn glbac7(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel5::Glbac7)
    }
}
#[doc = "NonSecure Enable for block B\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nse5 {
    #[doc = "0: Secure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\]), nonsecure accesses to block B are not allowed."]
    Allowed = 0,
    #[doc = "1: Secure accesses to block B are not allowed, nonsecure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\])."]
    Notallowed = 1,
}
impl From<Nse5> for bool {
    #[inline(always)]
    fn from(variant: Nse5) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NSE5` reader - NonSecure Enable for block B"]
pub type Nse5R = crate::BitReader<Nse5>;
impl Nse5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nse5 {
        match self.bits {
            false => Nse5::Allowed,
            true => Nse5::Notallowed,
        }
    }
    #[doc = "Secure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\]), nonsecure accesses to block B are not allowed."]
    #[inline(always)]
    pub fn is_allowed(&self) -> bool {
        *self == Nse5::Allowed
    }
    #[doc = "Secure accesses to block B are not allowed, nonsecure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\])."]
    #[inline(always)]
    pub fn is_notallowed(&self) -> bool {
        *self == Nse5::Notallowed
    }
}
#[doc = "Field `NSE5` writer - NonSecure Enable for block B"]
pub type Nse5W<'a, REG> = crate::BitWriter<'a, REG, Nse5>;
impl<'a, REG> Nse5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Secure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\]), nonsecure accesses to block B are not allowed."]
    #[inline(always)]
    pub fn allowed(self) -> &'a mut crate::W<REG> {
        self.variant(Nse5::Allowed)
    }
    #[doc = "Secure accesses to block B are not allowed, nonsecure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\])."]
    #[inline(always)]
    pub fn notallowed(self) -> &'a mut crate::W<REG> {
        self.variant(Nse5::Notallowed)
    }
}
#[doc = "Memory Block Access Control Select for block B\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Mbacsel6 {
    #[doc = "0: select MBC_MEMN_GLBAC0 access control policy for block B"]
    Glbac0 = 0,
    #[doc = "1: select MBC_MEMN_GLBAC1 access control policy for block B"]
    Glbac1 = 1,
    #[doc = "2: select MBC_MEMN_GLBAC2 access control policy for block B"]
    Glbac2 = 2,
    #[doc = "3: select MBC_MEMN_GLBAC3 access control policy for block B"]
    Glbac3 = 3,
    #[doc = "4: select MBC_MEMN_GLBAC4 access control policy for block B"]
    Glbac4 = 4,
    #[doc = "5: select MBC_MEMN_GLBAC5 access control policy for block B"]
    Glbac5 = 5,
    #[doc = "6: select MBC_MEMN_GLBAC6 access control policy for block B"]
    Glbac6 = 6,
    #[doc = "7: select MBC_MEMN_GLBAC7 access control policy for block B"]
    Glbac7 = 7,
}
impl From<Mbacsel6> for u8 {
    #[inline(always)]
    fn from(variant: Mbacsel6) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Mbacsel6 {
    type Ux = u8;
}
impl crate::IsEnum for Mbacsel6 {}
#[doc = "Field `MBACSEL6` reader - Memory Block Access Control Select for block B"]
pub type Mbacsel6R = crate::FieldReader<Mbacsel6>;
impl Mbacsel6R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mbacsel6 {
        match self.bits {
            0 => Mbacsel6::Glbac0,
            1 => Mbacsel6::Glbac1,
            2 => Mbacsel6::Glbac2,
            3 => Mbacsel6::Glbac3,
            4 => Mbacsel6::Glbac4,
            5 => Mbacsel6::Glbac5,
            6 => Mbacsel6::Glbac6,
            7 => Mbacsel6::Glbac7,
            _ => unreachable!(),
        }
    }
    #[doc = "select MBC_MEMN_GLBAC0 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac0(&self) -> bool {
        *self == Mbacsel6::Glbac0
    }
    #[doc = "select MBC_MEMN_GLBAC1 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac1(&self) -> bool {
        *self == Mbacsel6::Glbac1
    }
    #[doc = "select MBC_MEMN_GLBAC2 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac2(&self) -> bool {
        *self == Mbacsel6::Glbac2
    }
    #[doc = "select MBC_MEMN_GLBAC3 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac3(&self) -> bool {
        *self == Mbacsel6::Glbac3
    }
    #[doc = "select MBC_MEMN_GLBAC4 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac4(&self) -> bool {
        *self == Mbacsel6::Glbac4
    }
    #[doc = "select MBC_MEMN_GLBAC5 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac5(&self) -> bool {
        *self == Mbacsel6::Glbac5
    }
    #[doc = "select MBC_MEMN_GLBAC6 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac6(&self) -> bool {
        *self == Mbacsel6::Glbac6
    }
    #[doc = "select MBC_MEMN_GLBAC7 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac7(&self) -> bool {
        *self == Mbacsel6::Glbac7
    }
}
#[doc = "Field `MBACSEL6` writer - Memory Block Access Control Select for block B"]
pub type Mbacsel6W<'a, REG> = crate::FieldWriter<'a, REG, 3, Mbacsel6, crate::Safe>;
impl<'a, REG> Mbacsel6W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "select MBC_MEMN_GLBAC0 access control policy for block B"]
    #[inline(always)]
    pub fn glbac0(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel6::Glbac0)
    }
    #[doc = "select MBC_MEMN_GLBAC1 access control policy for block B"]
    #[inline(always)]
    pub fn glbac1(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel6::Glbac1)
    }
    #[doc = "select MBC_MEMN_GLBAC2 access control policy for block B"]
    #[inline(always)]
    pub fn glbac2(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel6::Glbac2)
    }
    #[doc = "select MBC_MEMN_GLBAC3 access control policy for block B"]
    #[inline(always)]
    pub fn glbac3(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel6::Glbac3)
    }
    #[doc = "select MBC_MEMN_GLBAC4 access control policy for block B"]
    #[inline(always)]
    pub fn glbac4(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel6::Glbac4)
    }
    #[doc = "select MBC_MEMN_GLBAC5 access control policy for block B"]
    #[inline(always)]
    pub fn glbac5(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel6::Glbac5)
    }
    #[doc = "select MBC_MEMN_GLBAC6 access control policy for block B"]
    #[inline(always)]
    pub fn glbac6(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel6::Glbac6)
    }
    #[doc = "select MBC_MEMN_GLBAC7 access control policy for block B"]
    #[inline(always)]
    pub fn glbac7(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel6::Glbac7)
    }
}
#[doc = "NonSecure Enable for block B\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nse6 {
    #[doc = "0: Secure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\]), nonsecure accesses to block B are not allowed."]
    Allowed = 0,
    #[doc = "1: Secure accesses to block B are not allowed, nonsecure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\])."]
    Notallowed = 1,
}
impl From<Nse6> for bool {
    #[inline(always)]
    fn from(variant: Nse6) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NSE6` reader - NonSecure Enable for block B"]
pub type Nse6R = crate::BitReader<Nse6>;
impl Nse6R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nse6 {
        match self.bits {
            false => Nse6::Allowed,
            true => Nse6::Notallowed,
        }
    }
    #[doc = "Secure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\]), nonsecure accesses to block B are not allowed."]
    #[inline(always)]
    pub fn is_allowed(&self) -> bool {
        *self == Nse6::Allowed
    }
    #[doc = "Secure accesses to block B are not allowed, nonsecure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\])."]
    #[inline(always)]
    pub fn is_notallowed(&self) -> bool {
        *self == Nse6::Notallowed
    }
}
#[doc = "Field `NSE6` writer - NonSecure Enable for block B"]
pub type Nse6W<'a, REG> = crate::BitWriter<'a, REG, Nse6>;
impl<'a, REG> Nse6W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Secure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\]), nonsecure accesses to block B are not allowed."]
    #[inline(always)]
    pub fn allowed(self) -> &'a mut crate::W<REG> {
        self.variant(Nse6::Allowed)
    }
    #[doc = "Secure accesses to block B are not allowed, nonsecure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\])."]
    #[inline(always)]
    pub fn notallowed(self) -> &'a mut crate::W<REG> {
        self.variant(Nse6::Notallowed)
    }
}
#[doc = "Memory Block Access Control Select for block B\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Mbacsel7 {
    #[doc = "0: select MBC_MEMN_GLBAC0 access control policy for block B"]
    Glbac0 = 0,
    #[doc = "1: select MBC_MEMN_GLBAC1 access control policy for block B"]
    Glbac1 = 1,
    #[doc = "2: select MBC_MEMN_GLBAC2 access control policy for block B"]
    Glbac2 = 2,
    #[doc = "3: select MBC_MEMN_GLBAC3 access control policy for block B"]
    Glbac3 = 3,
    #[doc = "4: select MBC_MEMN_GLBAC4 access control policy for block B"]
    Glbac4 = 4,
    #[doc = "5: select MBC_MEMN_GLBAC5 access control policy for block B"]
    Glbac5 = 5,
    #[doc = "6: select MBC_MEMN_GLBAC6 access control policy for block B"]
    Glbac6 = 6,
    #[doc = "7: select MBC_MEMN_GLBAC7 access control policy for block B"]
    Glbac7 = 7,
}
impl From<Mbacsel7> for u8 {
    #[inline(always)]
    fn from(variant: Mbacsel7) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Mbacsel7 {
    type Ux = u8;
}
impl crate::IsEnum for Mbacsel7 {}
#[doc = "Field `MBACSEL7` reader - Memory Block Access Control Select for block B"]
pub type Mbacsel7R = crate::FieldReader<Mbacsel7>;
impl Mbacsel7R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mbacsel7 {
        match self.bits {
            0 => Mbacsel7::Glbac0,
            1 => Mbacsel7::Glbac1,
            2 => Mbacsel7::Glbac2,
            3 => Mbacsel7::Glbac3,
            4 => Mbacsel7::Glbac4,
            5 => Mbacsel7::Glbac5,
            6 => Mbacsel7::Glbac6,
            7 => Mbacsel7::Glbac7,
            _ => unreachable!(),
        }
    }
    #[doc = "select MBC_MEMN_GLBAC0 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac0(&self) -> bool {
        *self == Mbacsel7::Glbac0
    }
    #[doc = "select MBC_MEMN_GLBAC1 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac1(&self) -> bool {
        *self == Mbacsel7::Glbac1
    }
    #[doc = "select MBC_MEMN_GLBAC2 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac2(&self) -> bool {
        *self == Mbacsel7::Glbac2
    }
    #[doc = "select MBC_MEMN_GLBAC3 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac3(&self) -> bool {
        *self == Mbacsel7::Glbac3
    }
    #[doc = "select MBC_MEMN_GLBAC4 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac4(&self) -> bool {
        *self == Mbacsel7::Glbac4
    }
    #[doc = "select MBC_MEMN_GLBAC5 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac5(&self) -> bool {
        *self == Mbacsel7::Glbac5
    }
    #[doc = "select MBC_MEMN_GLBAC6 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac6(&self) -> bool {
        *self == Mbacsel7::Glbac6
    }
    #[doc = "select MBC_MEMN_GLBAC7 access control policy for block B"]
    #[inline(always)]
    pub fn is_glbac7(&self) -> bool {
        *self == Mbacsel7::Glbac7
    }
}
#[doc = "Field `MBACSEL7` writer - Memory Block Access Control Select for block B"]
pub type Mbacsel7W<'a, REG> = crate::FieldWriter<'a, REG, 3, Mbacsel7, crate::Safe>;
impl<'a, REG> Mbacsel7W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "select MBC_MEMN_GLBAC0 access control policy for block B"]
    #[inline(always)]
    pub fn glbac0(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel7::Glbac0)
    }
    #[doc = "select MBC_MEMN_GLBAC1 access control policy for block B"]
    #[inline(always)]
    pub fn glbac1(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel7::Glbac1)
    }
    #[doc = "select MBC_MEMN_GLBAC2 access control policy for block B"]
    #[inline(always)]
    pub fn glbac2(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel7::Glbac2)
    }
    #[doc = "select MBC_MEMN_GLBAC3 access control policy for block B"]
    #[inline(always)]
    pub fn glbac3(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel7::Glbac3)
    }
    #[doc = "select MBC_MEMN_GLBAC4 access control policy for block B"]
    #[inline(always)]
    pub fn glbac4(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel7::Glbac4)
    }
    #[doc = "select MBC_MEMN_GLBAC5 access control policy for block B"]
    #[inline(always)]
    pub fn glbac5(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel7::Glbac5)
    }
    #[doc = "select MBC_MEMN_GLBAC6 access control policy for block B"]
    #[inline(always)]
    pub fn glbac6(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel7::Glbac6)
    }
    #[doc = "select MBC_MEMN_GLBAC7 access control policy for block B"]
    #[inline(always)]
    pub fn glbac7(self) -> &'a mut crate::W<REG> {
        self.variant(Mbacsel7::Glbac7)
    }
}
#[doc = "NonSecure Enable for block B\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nse7 {
    #[doc = "0: Secure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\]), nonsecure accesses to block B are not allowed."]
    Allowed = 0,
    #[doc = "1: Secure accesses to block B are not allowed, nonsecure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\])."]
    Notallowed = 1,
}
impl From<Nse7> for bool {
    #[inline(always)]
    fn from(variant: Nse7) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NSE7` reader - NonSecure Enable for block B"]
pub type Nse7R = crate::BitReader<Nse7>;
impl Nse7R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nse7 {
        match self.bits {
            false => Nse7::Allowed,
            true => Nse7::Notallowed,
        }
    }
    #[doc = "Secure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\]), nonsecure accesses to block B are not allowed."]
    #[inline(always)]
    pub fn is_allowed(&self) -> bool {
        *self == Nse7::Allowed
    }
    #[doc = "Secure accesses to block B are not allowed, nonsecure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\])."]
    #[inline(always)]
    pub fn is_notallowed(&self) -> bool {
        *self == Nse7::Notallowed
    }
}
#[doc = "Field `NSE7` writer - NonSecure Enable for block B"]
pub type Nse7W<'a, REG> = crate::BitWriter<'a, REG, Nse7>;
impl<'a, REG> Nse7W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Secure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\]), nonsecure accesses to block B are not allowed."]
    #[inline(always)]
    pub fn allowed(self) -> &'a mut crate::W<REG> {
        self.variant(Nse7::Allowed)
    }
    #[doc = "Secure accesses to block B are not allowed, nonsecure accesses to block B are based on corresponding MBACSEL field in this register (MBCm_DOMd_MEMs_BLK_CFG_Ww\\[MBACSEL\\])."]
    #[inline(always)]
    pub fn notallowed(self) -> &'a mut crate::W<REG> {
        self.variant(Nse7::Notallowed)
    }
}
impl R {
    #[doc = "Bits 0:2 - Memory Block Access Control Select for block B"]
    #[inline(always)]
    pub fn mbacsel0(&self) -> Mbacsel0R {
        Mbacsel0R::new((self.bits & 7) as u8)
    }
    #[doc = "Bit 3 - NonSecure Enable for block B"]
    #[inline(always)]
    pub fn nse0(&self) -> Nse0R {
        Nse0R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bits 4:6 - Memory Block Access Control Select for block B"]
    #[inline(always)]
    pub fn mbacsel1(&self) -> Mbacsel1R {
        Mbacsel1R::new(((self.bits >> 4) & 7) as u8)
    }
    #[doc = "Bit 7 - NonSecure Enable for block B"]
    #[inline(always)]
    pub fn nse1(&self) -> Nse1R {
        Nse1R::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bits 8:10 - Memory Block Access Control Select for block B"]
    #[inline(always)]
    pub fn mbacsel2(&self) -> Mbacsel2R {
        Mbacsel2R::new(((self.bits >> 8) & 7) as u8)
    }
    #[doc = "Bit 11 - NonSecure Enable for block B"]
    #[inline(always)]
    pub fn nse2(&self) -> Nse2R {
        Nse2R::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bits 12:14 - Memory Block Access Control Select for block B"]
    #[inline(always)]
    pub fn mbacsel3(&self) -> Mbacsel3R {
        Mbacsel3R::new(((self.bits >> 12) & 7) as u8)
    }
    #[doc = "Bit 15 - NonSecure Enable for block B"]
    #[inline(always)]
    pub fn nse3(&self) -> Nse3R {
        Nse3R::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bits 16:18 - Memory Block Access Control Select for block B"]
    #[inline(always)]
    pub fn mbacsel4(&self) -> Mbacsel4R {
        Mbacsel4R::new(((self.bits >> 16) & 7) as u8)
    }
    #[doc = "Bit 19 - NonSecure Enable for block B"]
    #[inline(always)]
    pub fn nse4(&self) -> Nse4R {
        Nse4R::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bits 20:22 - Memory Block Access Control Select for block B"]
    #[inline(always)]
    pub fn mbacsel5(&self) -> Mbacsel5R {
        Mbacsel5R::new(((self.bits >> 20) & 7) as u8)
    }
    #[doc = "Bit 23 - NonSecure Enable for block B"]
    #[inline(always)]
    pub fn nse5(&self) -> Nse5R {
        Nse5R::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bits 24:26 - Memory Block Access Control Select for block B"]
    #[inline(always)]
    pub fn mbacsel6(&self) -> Mbacsel6R {
        Mbacsel6R::new(((self.bits >> 24) & 7) as u8)
    }
    #[doc = "Bit 27 - NonSecure Enable for block B"]
    #[inline(always)]
    pub fn nse6(&self) -> Nse6R {
        Nse6R::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bits 28:30 - Memory Block Access Control Select for block B"]
    #[inline(always)]
    pub fn mbacsel7(&self) -> Mbacsel7R {
        Mbacsel7R::new(((self.bits >> 28) & 7) as u8)
    }
    #[doc = "Bit 31 - NonSecure Enable for block B"]
    #[inline(always)]
    pub fn nse7(&self) -> Nse7R {
        Nse7R::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:2 - Memory Block Access Control Select for block B"]
    #[inline(always)]
    pub fn mbacsel0(&mut self) -> Mbacsel0W<Mbc0Dom0Mem0BlkCfgW13Spec> {
        Mbacsel0W::new(self, 0)
    }
    #[doc = "Bit 3 - NonSecure Enable for block B"]
    #[inline(always)]
    pub fn nse0(&mut self) -> Nse0W<Mbc0Dom0Mem0BlkCfgW13Spec> {
        Nse0W::new(self, 3)
    }
    #[doc = "Bits 4:6 - Memory Block Access Control Select for block B"]
    #[inline(always)]
    pub fn mbacsel1(&mut self) -> Mbacsel1W<Mbc0Dom0Mem0BlkCfgW13Spec> {
        Mbacsel1W::new(self, 4)
    }
    #[doc = "Bit 7 - NonSecure Enable for block B"]
    #[inline(always)]
    pub fn nse1(&mut self) -> Nse1W<Mbc0Dom0Mem0BlkCfgW13Spec> {
        Nse1W::new(self, 7)
    }
    #[doc = "Bits 8:10 - Memory Block Access Control Select for block B"]
    #[inline(always)]
    pub fn mbacsel2(&mut self) -> Mbacsel2W<Mbc0Dom0Mem0BlkCfgW13Spec> {
        Mbacsel2W::new(self, 8)
    }
    #[doc = "Bit 11 - NonSecure Enable for block B"]
    #[inline(always)]
    pub fn nse2(&mut self) -> Nse2W<Mbc0Dom0Mem0BlkCfgW13Spec> {
        Nse2W::new(self, 11)
    }
    #[doc = "Bits 12:14 - Memory Block Access Control Select for block B"]
    #[inline(always)]
    pub fn mbacsel3(&mut self) -> Mbacsel3W<Mbc0Dom0Mem0BlkCfgW13Spec> {
        Mbacsel3W::new(self, 12)
    }
    #[doc = "Bit 15 - NonSecure Enable for block B"]
    #[inline(always)]
    pub fn nse3(&mut self) -> Nse3W<Mbc0Dom0Mem0BlkCfgW13Spec> {
        Nse3W::new(self, 15)
    }
    #[doc = "Bits 16:18 - Memory Block Access Control Select for block B"]
    #[inline(always)]
    pub fn mbacsel4(&mut self) -> Mbacsel4W<Mbc0Dom0Mem0BlkCfgW13Spec> {
        Mbacsel4W::new(self, 16)
    }
    #[doc = "Bit 19 - NonSecure Enable for block B"]
    #[inline(always)]
    pub fn nse4(&mut self) -> Nse4W<Mbc0Dom0Mem0BlkCfgW13Spec> {
        Nse4W::new(self, 19)
    }
    #[doc = "Bits 20:22 - Memory Block Access Control Select for block B"]
    #[inline(always)]
    pub fn mbacsel5(&mut self) -> Mbacsel5W<Mbc0Dom0Mem0BlkCfgW13Spec> {
        Mbacsel5W::new(self, 20)
    }
    #[doc = "Bit 23 - NonSecure Enable for block B"]
    #[inline(always)]
    pub fn nse5(&mut self) -> Nse5W<Mbc0Dom0Mem0BlkCfgW13Spec> {
        Nse5W::new(self, 23)
    }
    #[doc = "Bits 24:26 - Memory Block Access Control Select for block B"]
    #[inline(always)]
    pub fn mbacsel6(&mut self) -> Mbacsel6W<Mbc0Dom0Mem0BlkCfgW13Spec> {
        Mbacsel6W::new(self, 24)
    }
    #[doc = "Bit 27 - NonSecure Enable for block B"]
    #[inline(always)]
    pub fn nse6(&mut self) -> Nse6W<Mbc0Dom0Mem0BlkCfgW13Spec> {
        Nse6W::new(self, 27)
    }
    #[doc = "Bits 28:30 - Memory Block Access Control Select for block B"]
    #[inline(always)]
    pub fn mbacsel7(&mut self) -> Mbacsel7W<Mbc0Dom0Mem0BlkCfgW13Spec> {
        Mbacsel7W::new(self, 28)
    }
    #[doc = "Bit 31 - NonSecure Enable for block B"]
    #[inline(always)]
    pub fn nse7(&mut self) -> Nse7W<Mbc0Dom0Mem0BlkCfgW13Spec> {
        Nse7W::new(self, 31)
    }
}
#[doc = "MBC Memory Block Configuration Word\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_dom0_mem0_blk_cfg_w13::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_dom0_mem0_blk_cfg_w13::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Mbc0Dom0Mem0BlkCfgW13Spec;
impl crate::RegisterSpec for Mbc0Dom0Mem0BlkCfgW13Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mbc0_dom0_mem0_blk_cfg_w13::R`](R) reader structure"]
impl crate::Readable for Mbc0Dom0Mem0BlkCfgW13Spec {}
#[doc = "`write(|w| ..)` method takes [`mbc0_dom0_mem0_blk_cfg_w13::W`](W) writer structure"]
impl crate::Writable for Mbc0Dom0Mem0BlkCfgW13Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MBC0_DOM0_MEM0_BLK_CFG_W13 to value 0"]
impl crate::Resettable for Mbc0Dom0Mem0BlkCfgW13Spec {}

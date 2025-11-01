#[doc = "Register `CCR` reader"]
pub type R = crate::R<CcrSpec>;
#[doc = "Register `CCR` writer"]
pub type W = crate::W<CcrSpec>;
#[doc = "Rising Edge of Capture Channel 0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cap0re {
    #[doc = "0: Does not load"]
    Cap0re0 = 0,
    #[doc = "1: Loads"]
    Capore1 = 1,
}
impl From<Cap0re> for bool {
    #[inline(always)]
    fn from(variant: Cap0re) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CAP0RE` reader - Rising Edge of Capture Channel 0"]
pub type Cap0reR = crate::BitReader<Cap0re>;
impl Cap0reR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cap0re {
        match self.bits {
            false => Cap0re::Cap0re0,
            true => Cap0re::Capore1,
        }
    }
    #[doc = "Does not load"]
    #[inline(always)]
    pub fn is_cap0re_0(&self) -> bool {
        *self == Cap0re::Cap0re0
    }
    #[doc = "Loads"]
    #[inline(always)]
    pub fn is_capore_1(&self) -> bool {
        *self == Cap0re::Capore1
    }
}
#[doc = "Field `CAP0RE` writer - Rising Edge of Capture Channel 0"]
pub type Cap0reW<'a, REG> = crate::BitWriter<'a, REG, Cap0re>;
impl<'a, REG> Cap0reW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not load"]
    #[inline(always)]
    pub fn cap0re_0(self) -> &'a mut crate::W<REG> {
        self.variant(Cap0re::Cap0re0)
    }
    #[doc = "Loads"]
    #[inline(always)]
    pub fn capore_1(self) -> &'a mut crate::W<REG> {
        self.variant(Cap0re::Capore1)
    }
}
#[doc = "Falling Edge of Capture Channel 0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cap0fe {
    #[doc = "0: Does not load"]
    Cap0fe0 = 0,
    #[doc = "1: Loads"]
    Capofe1 = 1,
}
impl From<Cap0fe> for bool {
    #[inline(always)]
    fn from(variant: Cap0fe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CAP0FE` reader - Falling Edge of Capture Channel 0"]
pub type Cap0feR = crate::BitReader<Cap0fe>;
impl Cap0feR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cap0fe {
        match self.bits {
            false => Cap0fe::Cap0fe0,
            true => Cap0fe::Capofe1,
        }
    }
    #[doc = "Does not load"]
    #[inline(always)]
    pub fn is_cap0fe_0(&self) -> bool {
        *self == Cap0fe::Cap0fe0
    }
    #[doc = "Loads"]
    #[inline(always)]
    pub fn is_capofe_1(&self) -> bool {
        *self == Cap0fe::Capofe1
    }
}
#[doc = "Field `CAP0FE` writer - Falling Edge of Capture Channel 0"]
pub type Cap0feW<'a, REG> = crate::BitWriter<'a, REG, Cap0fe>;
impl<'a, REG> Cap0feW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not load"]
    #[inline(always)]
    pub fn cap0fe_0(self) -> &'a mut crate::W<REG> {
        self.variant(Cap0fe::Cap0fe0)
    }
    #[doc = "Loads"]
    #[inline(always)]
    pub fn capofe_1(self) -> &'a mut crate::W<REG> {
        self.variant(Cap0fe::Capofe1)
    }
}
#[doc = "Generate Interrupt on Channel 0 Capture Event\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cap0i {
    #[doc = "0: Does not generate"]
    Cap0i0 = 0,
    #[doc = "1: Generates"]
    Capoi1 = 1,
}
impl From<Cap0i> for bool {
    #[inline(always)]
    fn from(variant: Cap0i) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CAP0I` reader - Generate Interrupt on Channel 0 Capture Event"]
pub type Cap0iR = crate::BitReader<Cap0i>;
impl Cap0iR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cap0i {
        match self.bits {
            false => Cap0i::Cap0i0,
            true => Cap0i::Capoi1,
        }
    }
    #[doc = "Does not generate"]
    #[inline(always)]
    pub fn is_cap0i_0(&self) -> bool {
        *self == Cap0i::Cap0i0
    }
    #[doc = "Generates"]
    #[inline(always)]
    pub fn is_capoi_1(&self) -> bool {
        *self == Cap0i::Capoi1
    }
}
#[doc = "Field `CAP0I` writer - Generate Interrupt on Channel 0 Capture Event"]
pub type Cap0iW<'a, REG> = crate::BitWriter<'a, REG, Cap0i>;
impl<'a, REG> Cap0iW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not generate"]
    #[inline(always)]
    pub fn cap0i_0(self) -> &'a mut crate::W<REG> {
        self.variant(Cap0i::Cap0i0)
    }
    #[doc = "Generates"]
    #[inline(always)]
    pub fn capoi_1(self) -> &'a mut crate::W<REG> {
        self.variant(Cap0i::Capoi1)
    }
}
#[doc = "Rising Edge of Capture Channel 1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cap1re {
    #[doc = "0: Does not load"]
    Cap1re0 = 0,
    #[doc = "1: Loads"]
    Cap1re1 = 1,
}
impl From<Cap1re> for bool {
    #[inline(always)]
    fn from(variant: Cap1re) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CAP1RE` reader - Rising Edge of Capture Channel 1"]
pub type Cap1reR = crate::BitReader<Cap1re>;
impl Cap1reR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cap1re {
        match self.bits {
            false => Cap1re::Cap1re0,
            true => Cap1re::Cap1re1,
        }
    }
    #[doc = "Does not load"]
    #[inline(always)]
    pub fn is_cap1re_0(&self) -> bool {
        *self == Cap1re::Cap1re0
    }
    #[doc = "Loads"]
    #[inline(always)]
    pub fn is_cap1re_1(&self) -> bool {
        *self == Cap1re::Cap1re1
    }
}
#[doc = "Field `CAP1RE` writer - Rising Edge of Capture Channel 1"]
pub type Cap1reW<'a, REG> = crate::BitWriter<'a, REG, Cap1re>;
impl<'a, REG> Cap1reW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not load"]
    #[inline(always)]
    pub fn cap1re_0(self) -> &'a mut crate::W<REG> {
        self.variant(Cap1re::Cap1re0)
    }
    #[doc = "Loads"]
    #[inline(always)]
    pub fn cap1re_1(self) -> &'a mut crate::W<REG> {
        self.variant(Cap1re::Cap1re1)
    }
}
#[doc = "Falling Edge of Capture Channel 1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cap1fe {
    #[doc = "0: Does not load"]
    Cap1fe0 = 0,
    #[doc = "1: Loads"]
    Cap1fe1 = 1,
}
impl From<Cap1fe> for bool {
    #[inline(always)]
    fn from(variant: Cap1fe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CAP1FE` reader - Falling Edge of Capture Channel 1"]
pub type Cap1feR = crate::BitReader<Cap1fe>;
impl Cap1feR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cap1fe {
        match self.bits {
            false => Cap1fe::Cap1fe0,
            true => Cap1fe::Cap1fe1,
        }
    }
    #[doc = "Does not load"]
    #[inline(always)]
    pub fn is_cap1fe_0(&self) -> bool {
        *self == Cap1fe::Cap1fe0
    }
    #[doc = "Loads"]
    #[inline(always)]
    pub fn is_cap1fe_1(&self) -> bool {
        *self == Cap1fe::Cap1fe1
    }
}
#[doc = "Field `CAP1FE` writer - Falling Edge of Capture Channel 1"]
pub type Cap1feW<'a, REG> = crate::BitWriter<'a, REG, Cap1fe>;
impl<'a, REG> Cap1feW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not load"]
    #[inline(always)]
    pub fn cap1fe_0(self) -> &'a mut crate::W<REG> {
        self.variant(Cap1fe::Cap1fe0)
    }
    #[doc = "Loads"]
    #[inline(always)]
    pub fn cap1fe_1(self) -> &'a mut crate::W<REG> {
        self.variant(Cap1fe::Cap1fe1)
    }
}
#[doc = "Generate Interrupt on Channel 1 Capture Event\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cap1i {
    #[doc = "0: Does not generates"]
    Cap1i0 = 0,
    #[doc = "1: Generates"]
    Cap1i1 = 1,
}
impl From<Cap1i> for bool {
    #[inline(always)]
    fn from(variant: Cap1i) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CAP1I` reader - Generate Interrupt on Channel 1 Capture Event"]
pub type Cap1iR = crate::BitReader<Cap1i>;
impl Cap1iR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cap1i {
        match self.bits {
            false => Cap1i::Cap1i0,
            true => Cap1i::Cap1i1,
        }
    }
    #[doc = "Does not generates"]
    #[inline(always)]
    pub fn is_cap1i_0(&self) -> bool {
        *self == Cap1i::Cap1i0
    }
    #[doc = "Generates"]
    #[inline(always)]
    pub fn is_cap1i_1(&self) -> bool {
        *self == Cap1i::Cap1i1
    }
}
#[doc = "Field `CAP1I` writer - Generate Interrupt on Channel 1 Capture Event"]
pub type Cap1iW<'a, REG> = crate::BitWriter<'a, REG, Cap1i>;
impl<'a, REG> Cap1iW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not generates"]
    #[inline(always)]
    pub fn cap1i_0(self) -> &'a mut crate::W<REG> {
        self.variant(Cap1i::Cap1i0)
    }
    #[doc = "Generates"]
    #[inline(always)]
    pub fn cap1i_1(self) -> &'a mut crate::W<REG> {
        self.variant(Cap1i::Cap1i1)
    }
}
#[doc = "Rising Edge of Capture Channel 2\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cap2re {
    #[doc = "0: Does not load"]
    Cap2re0 = 0,
    #[doc = "1: Loads"]
    Cap2re1 = 1,
}
impl From<Cap2re> for bool {
    #[inline(always)]
    fn from(variant: Cap2re) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CAP2RE` reader - Rising Edge of Capture Channel 2"]
pub type Cap2reR = crate::BitReader<Cap2re>;
impl Cap2reR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cap2re {
        match self.bits {
            false => Cap2re::Cap2re0,
            true => Cap2re::Cap2re1,
        }
    }
    #[doc = "Does not load"]
    #[inline(always)]
    pub fn is_cap2re_0(&self) -> bool {
        *self == Cap2re::Cap2re0
    }
    #[doc = "Loads"]
    #[inline(always)]
    pub fn is_cap2re_1(&self) -> bool {
        *self == Cap2re::Cap2re1
    }
}
#[doc = "Field `CAP2RE` writer - Rising Edge of Capture Channel 2"]
pub type Cap2reW<'a, REG> = crate::BitWriter<'a, REG, Cap2re>;
impl<'a, REG> Cap2reW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not load"]
    #[inline(always)]
    pub fn cap2re_0(self) -> &'a mut crate::W<REG> {
        self.variant(Cap2re::Cap2re0)
    }
    #[doc = "Loads"]
    #[inline(always)]
    pub fn cap2re_1(self) -> &'a mut crate::W<REG> {
        self.variant(Cap2re::Cap2re1)
    }
}
#[doc = "Falling Edge of Capture Channel 2\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cap2fe {
    #[doc = "0: Does not load"]
    Cap2fe0 = 0,
    #[doc = "1: Loads"]
    Cap2fe1 = 1,
}
impl From<Cap2fe> for bool {
    #[inline(always)]
    fn from(variant: Cap2fe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CAP2FE` reader - Falling Edge of Capture Channel 2"]
pub type Cap2feR = crate::BitReader<Cap2fe>;
impl Cap2feR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cap2fe {
        match self.bits {
            false => Cap2fe::Cap2fe0,
            true => Cap2fe::Cap2fe1,
        }
    }
    #[doc = "Does not load"]
    #[inline(always)]
    pub fn is_cap2fe_0(&self) -> bool {
        *self == Cap2fe::Cap2fe0
    }
    #[doc = "Loads"]
    #[inline(always)]
    pub fn is_cap2fe_1(&self) -> bool {
        *self == Cap2fe::Cap2fe1
    }
}
#[doc = "Field `CAP2FE` writer - Falling Edge of Capture Channel 2"]
pub type Cap2feW<'a, REG> = crate::BitWriter<'a, REG, Cap2fe>;
impl<'a, REG> Cap2feW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not load"]
    #[inline(always)]
    pub fn cap2fe_0(self) -> &'a mut crate::W<REG> {
        self.variant(Cap2fe::Cap2fe0)
    }
    #[doc = "Loads"]
    #[inline(always)]
    pub fn cap2fe_1(self) -> &'a mut crate::W<REG> {
        self.variant(Cap2fe::Cap2fe1)
    }
}
#[doc = "Generate Interrupt on Channel 2 Capture Event\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cap2i {
    #[doc = "0: Does not generate"]
    Cap2i0 = 0,
    #[doc = "1: Generates"]
    Cap2i1 = 1,
}
impl From<Cap2i> for bool {
    #[inline(always)]
    fn from(variant: Cap2i) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CAP2I` reader - Generate Interrupt on Channel 2 Capture Event"]
pub type Cap2iR = crate::BitReader<Cap2i>;
impl Cap2iR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cap2i {
        match self.bits {
            false => Cap2i::Cap2i0,
            true => Cap2i::Cap2i1,
        }
    }
    #[doc = "Does not generate"]
    #[inline(always)]
    pub fn is_cap2i_0(&self) -> bool {
        *self == Cap2i::Cap2i0
    }
    #[doc = "Generates"]
    #[inline(always)]
    pub fn is_cap2i_1(&self) -> bool {
        *self == Cap2i::Cap2i1
    }
}
#[doc = "Field `CAP2I` writer - Generate Interrupt on Channel 2 Capture Event"]
pub type Cap2iW<'a, REG> = crate::BitWriter<'a, REG, Cap2i>;
impl<'a, REG> Cap2iW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not generate"]
    #[inline(always)]
    pub fn cap2i_0(self) -> &'a mut crate::W<REG> {
        self.variant(Cap2i::Cap2i0)
    }
    #[doc = "Generates"]
    #[inline(always)]
    pub fn cap2i_1(self) -> &'a mut crate::W<REG> {
        self.variant(Cap2i::Cap2i1)
    }
}
#[doc = "Rising Edge of Capture Channel 3\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cap3re {
    #[doc = "0: Does not load"]
    Cap3re0 = 0,
    #[doc = "1: Loads"]
    Cap3re1 = 1,
}
impl From<Cap3re> for bool {
    #[inline(always)]
    fn from(variant: Cap3re) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CAP3RE` reader - Rising Edge of Capture Channel 3"]
pub type Cap3reR = crate::BitReader<Cap3re>;
impl Cap3reR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cap3re {
        match self.bits {
            false => Cap3re::Cap3re0,
            true => Cap3re::Cap3re1,
        }
    }
    #[doc = "Does not load"]
    #[inline(always)]
    pub fn is_cap3re_0(&self) -> bool {
        *self == Cap3re::Cap3re0
    }
    #[doc = "Loads"]
    #[inline(always)]
    pub fn is_cap3re_1(&self) -> bool {
        *self == Cap3re::Cap3re1
    }
}
#[doc = "Field `CAP3RE` writer - Rising Edge of Capture Channel 3"]
pub type Cap3reW<'a, REG> = crate::BitWriter<'a, REG, Cap3re>;
impl<'a, REG> Cap3reW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not load"]
    #[inline(always)]
    pub fn cap3re_0(self) -> &'a mut crate::W<REG> {
        self.variant(Cap3re::Cap3re0)
    }
    #[doc = "Loads"]
    #[inline(always)]
    pub fn cap3re_1(self) -> &'a mut crate::W<REG> {
        self.variant(Cap3re::Cap3re1)
    }
}
#[doc = "Falling Edge of Capture Channel 3\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cap3fe {
    #[doc = "0: Does not load"]
    Cap3fe0 = 0,
    #[doc = "1: Loads"]
    Cap3fe1 = 1,
}
impl From<Cap3fe> for bool {
    #[inline(always)]
    fn from(variant: Cap3fe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CAP3FE` reader - Falling Edge of Capture Channel 3"]
pub type Cap3feR = crate::BitReader<Cap3fe>;
impl Cap3feR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cap3fe {
        match self.bits {
            false => Cap3fe::Cap3fe0,
            true => Cap3fe::Cap3fe1,
        }
    }
    #[doc = "Does not load"]
    #[inline(always)]
    pub fn is_cap3fe_0(&self) -> bool {
        *self == Cap3fe::Cap3fe0
    }
    #[doc = "Loads"]
    #[inline(always)]
    pub fn is_cap3fe_1(&self) -> bool {
        *self == Cap3fe::Cap3fe1
    }
}
#[doc = "Field `CAP3FE` writer - Falling Edge of Capture Channel 3"]
pub type Cap3feW<'a, REG> = crate::BitWriter<'a, REG, Cap3fe>;
impl<'a, REG> Cap3feW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not load"]
    #[inline(always)]
    pub fn cap3fe_0(self) -> &'a mut crate::W<REG> {
        self.variant(Cap3fe::Cap3fe0)
    }
    #[doc = "Loads"]
    #[inline(always)]
    pub fn cap3fe_1(self) -> &'a mut crate::W<REG> {
        self.variant(Cap3fe::Cap3fe1)
    }
}
#[doc = "Generate Interrupt on Channel 3 Capture Event\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cap3i {
    #[doc = "0: Does not generate"]
    Cap3i0 = 0,
    #[doc = "1: Generates"]
    Cap3i1 = 1,
}
impl From<Cap3i> for bool {
    #[inline(always)]
    fn from(variant: Cap3i) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CAP3I` reader - Generate Interrupt on Channel 3 Capture Event"]
pub type Cap3iR = crate::BitReader<Cap3i>;
impl Cap3iR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cap3i {
        match self.bits {
            false => Cap3i::Cap3i0,
            true => Cap3i::Cap3i1,
        }
    }
    #[doc = "Does not generate"]
    #[inline(always)]
    pub fn is_cap3i_0(&self) -> bool {
        *self == Cap3i::Cap3i0
    }
    #[doc = "Generates"]
    #[inline(always)]
    pub fn is_cap3i_1(&self) -> bool {
        *self == Cap3i::Cap3i1
    }
}
#[doc = "Field `CAP3I` writer - Generate Interrupt on Channel 3 Capture Event"]
pub type Cap3iW<'a, REG> = crate::BitWriter<'a, REG, Cap3i>;
impl<'a, REG> Cap3iW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not generate"]
    #[inline(always)]
    pub fn cap3i_0(self) -> &'a mut crate::W<REG> {
        self.variant(Cap3i::Cap3i0)
    }
    #[doc = "Generates"]
    #[inline(always)]
    pub fn cap3i_1(self) -> &'a mut crate::W<REG> {
        self.variant(Cap3i::Cap3i1)
    }
}
impl R {
    #[doc = "Bit 0 - Rising Edge of Capture Channel 0"]
    #[inline(always)]
    pub fn cap0re(&self) -> Cap0reR {
        Cap0reR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Falling Edge of Capture Channel 0"]
    #[inline(always)]
    pub fn cap0fe(&self) -> Cap0feR {
        Cap0feR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Generate Interrupt on Channel 0 Capture Event"]
    #[inline(always)]
    pub fn cap0i(&self) -> Cap0iR {
        Cap0iR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Rising Edge of Capture Channel 1"]
    #[inline(always)]
    pub fn cap1re(&self) -> Cap1reR {
        Cap1reR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Falling Edge of Capture Channel 1"]
    #[inline(always)]
    pub fn cap1fe(&self) -> Cap1feR {
        Cap1feR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Generate Interrupt on Channel 1 Capture Event"]
    #[inline(always)]
    pub fn cap1i(&self) -> Cap1iR {
        Cap1iR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Rising Edge of Capture Channel 2"]
    #[inline(always)]
    pub fn cap2re(&self) -> Cap2reR {
        Cap2reR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Falling Edge of Capture Channel 2"]
    #[inline(always)]
    pub fn cap2fe(&self) -> Cap2feR {
        Cap2feR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Generate Interrupt on Channel 2 Capture Event"]
    #[inline(always)]
    pub fn cap2i(&self) -> Cap2iR {
        Cap2iR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Rising Edge of Capture Channel 3"]
    #[inline(always)]
    pub fn cap3re(&self) -> Cap3reR {
        Cap3reR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Falling Edge of Capture Channel 3"]
    #[inline(always)]
    pub fn cap3fe(&self) -> Cap3feR {
        Cap3feR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Generate Interrupt on Channel 3 Capture Event"]
    #[inline(always)]
    pub fn cap3i(&self) -> Cap3iR {
        Cap3iR::new(((self.bits >> 11) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Rising Edge of Capture Channel 0"]
    #[inline(always)]
    pub fn cap0re(&mut self) -> Cap0reW<CcrSpec> {
        Cap0reW::new(self, 0)
    }
    #[doc = "Bit 1 - Falling Edge of Capture Channel 0"]
    #[inline(always)]
    pub fn cap0fe(&mut self) -> Cap0feW<CcrSpec> {
        Cap0feW::new(self, 1)
    }
    #[doc = "Bit 2 - Generate Interrupt on Channel 0 Capture Event"]
    #[inline(always)]
    pub fn cap0i(&mut self) -> Cap0iW<CcrSpec> {
        Cap0iW::new(self, 2)
    }
    #[doc = "Bit 3 - Rising Edge of Capture Channel 1"]
    #[inline(always)]
    pub fn cap1re(&mut self) -> Cap1reW<CcrSpec> {
        Cap1reW::new(self, 3)
    }
    #[doc = "Bit 4 - Falling Edge of Capture Channel 1"]
    #[inline(always)]
    pub fn cap1fe(&mut self) -> Cap1feW<CcrSpec> {
        Cap1feW::new(self, 4)
    }
    #[doc = "Bit 5 - Generate Interrupt on Channel 1 Capture Event"]
    #[inline(always)]
    pub fn cap1i(&mut self) -> Cap1iW<CcrSpec> {
        Cap1iW::new(self, 5)
    }
    #[doc = "Bit 6 - Rising Edge of Capture Channel 2"]
    #[inline(always)]
    pub fn cap2re(&mut self) -> Cap2reW<CcrSpec> {
        Cap2reW::new(self, 6)
    }
    #[doc = "Bit 7 - Falling Edge of Capture Channel 2"]
    #[inline(always)]
    pub fn cap2fe(&mut self) -> Cap2feW<CcrSpec> {
        Cap2feW::new(self, 7)
    }
    #[doc = "Bit 8 - Generate Interrupt on Channel 2 Capture Event"]
    #[inline(always)]
    pub fn cap2i(&mut self) -> Cap2iW<CcrSpec> {
        Cap2iW::new(self, 8)
    }
    #[doc = "Bit 9 - Rising Edge of Capture Channel 3"]
    #[inline(always)]
    pub fn cap3re(&mut self) -> Cap3reW<CcrSpec> {
        Cap3reW::new(self, 9)
    }
    #[doc = "Bit 10 - Falling Edge of Capture Channel 3"]
    #[inline(always)]
    pub fn cap3fe(&mut self) -> Cap3feW<CcrSpec> {
        Cap3feW::new(self, 10)
    }
    #[doc = "Bit 11 - Generate Interrupt on Channel 3 Capture Event"]
    #[inline(always)]
    pub fn cap3i(&mut self) -> Cap3iW<CcrSpec> {
        Cap3iW::new(self, 11)
    }
}
#[doc = "Capture Control\n\nYou can [`read`](crate::Reg::read) this register and get [`ccr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ccr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CcrSpec;
impl crate::RegisterSpec for CcrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ccr::R`](R) reader structure"]
impl crate::Readable for CcrSpec {}
#[doc = "`write(|w| ..)` method takes [`ccr::W`](W) writer structure"]
impl crate::Writable for CcrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CCR to value 0"]
impl crate::Resettable for CcrSpec {}

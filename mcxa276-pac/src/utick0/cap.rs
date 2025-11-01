#[doc = "Register `CAP[%s]` reader"]
pub type R = crate::R<CapSpec>;
#[doc = "Field `CAP_VALUE` reader - Captured Value for the Related Capture Event"]
pub type CapValueR = crate::FieldReader<u32>;
#[doc = "Captured Value Valid Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Valid {
    #[doc = "0: Valid value not captured"]
    Notvalid = 0,
    #[doc = "1: Valid value captured"]
    Valid = 1,
}
impl From<Valid> for bool {
    #[inline(always)]
    fn from(variant: Valid) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `VALID` reader - Captured Value Valid Flag"]
pub type ValidR = crate::BitReader<Valid>;
impl ValidR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Valid {
        match self.bits {
            false => Valid::Notvalid,
            true => Valid::Valid,
        }
    }
    #[doc = "Valid value not captured"]
    #[inline(always)]
    pub fn is_notvalid(&self) -> bool {
        *self == Valid::Notvalid
    }
    #[doc = "Valid value captured"]
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        *self == Valid::Valid
    }
}
impl R {
    #[doc = "Bits 0:30 - Captured Value for the Related Capture Event"]
    #[inline(always)]
    pub fn cap_value(&self) -> CapValueR {
        CapValueR::new(self.bits & 0x7fff_ffff)
    }
    #[doc = "Bit 31 - Captured Value Valid Flag"]
    #[inline(always)]
    pub fn valid(&self) -> ValidR {
        ValidR::new(((self.bits >> 31) & 1) != 0)
    }
}
#[doc = "Capture\n\nYou can [`read`](crate::Reg::read) this register and get [`cap::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CapSpec;
impl crate::RegisterSpec for CapSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`cap::R`](R) reader structure"]
impl crate::Readable for CapSpec {}
#[doc = "`reset()` method sets CAP[%s] to value 0"]
impl crate::Resettable for CapSpec {}

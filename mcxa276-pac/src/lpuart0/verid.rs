#[doc = "Register `VERID` reader"]
pub type R = crate::R<VeridSpec>;
#[doc = "Feature Identification Number\n\nValue on reset: 3"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum Feature {
    #[doc = "1: Standard feature set"]
    Standard = 1,
    #[doc = "3: Standard feature set with MODEM and IrDA support"]
    Modem = 3,
}
impl From<Feature> for u16 {
    #[inline(always)]
    fn from(variant: Feature) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Feature {
    type Ux = u16;
}
impl crate::IsEnum for Feature {}
#[doc = "Field `FEATURE` reader - Feature Identification Number"]
pub type FeatureR = crate::FieldReader<Feature>;
impl FeatureR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Feature> {
        match self.bits {
            1 => Some(Feature::Standard),
            3 => Some(Feature::Modem),
            _ => None,
        }
    }
    #[doc = "Standard feature set"]
    #[inline(always)]
    pub fn is_standard(&self) -> bool {
        *self == Feature::Standard
    }
    #[doc = "Standard feature set with MODEM and IrDA support"]
    #[inline(always)]
    pub fn is_modem(&self) -> bool {
        *self == Feature::Modem
    }
}
#[doc = "Field `MINOR` reader - Minor Version Number"]
pub type MinorR = crate::FieldReader;
#[doc = "Field `MAJOR` reader - Major Version Number"]
pub type MajorR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:15 - Feature Identification Number"]
    #[inline(always)]
    pub fn feature(&self) -> FeatureR {
        FeatureR::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bits 16:23 - Minor Version Number"]
    #[inline(always)]
    pub fn minor(&self) -> MinorR {
        MinorR::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - Major Version Number"]
    #[inline(always)]
    pub fn major(&self) -> MajorR {
        MajorR::new(((self.bits >> 24) & 0xff) as u8)
    }
}
#[doc = "Version ID\n\nYou can [`read`](crate::Reg::read) this register and get [`verid::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct VeridSpec;
impl crate::RegisterSpec for VeridSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`verid::R`](R) reader structure"]
impl crate::Readable for VeridSpec {}
#[doc = "`reset()` method sets VERID to value 0x0405_0003"]
impl crate::Resettable for VeridSpec {
    const RESET_VALUE: u32 = 0x0405_0003;
}

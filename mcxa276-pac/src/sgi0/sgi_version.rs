#[doc = "Register `sgi_version` reader"]
pub type R = crate::R<SgiVersionSpec>;
#[doc = "Field `z` reader - Extended revision number in X.Y1Y2.Z, e.g. 1.20.3."]
pub type ZR = crate::FieldReader;
#[doc = "Field `y2` reader - Minor revision number 2 in X.Y1Y2.Z, e.g. 1.20.3."]
pub type Y2R = crate::FieldReader;
#[doc = "Field `y1` reader - Minor revision number 1 in X.Y1Y2.Z, e.g. 1.20.3."]
pub type Y1R = crate::FieldReader;
#[doc = "Field `x` reader - Major revision number in X.Y1Y2.Z, e.g. 1.20.3."]
pub type XR = crate::FieldReader;
#[doc = "Field `milestone` reader - Release milestone. 00-PREL, 01-BR, 10-SI, 11-GO."]
pub type MilestoneR = crate::FieldReader;
#[doc = "Field `version_rsvd_1` reader - Reserved for Future Use"]
pub type VersionRsvd1R = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:3 - Extended revision number in X.Y1Y2.Z, e.g. 1.20.3."]
    #[inline(always)]
    pub fn z(&self) -> ZR {
        ZR::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bits 4:7 - Minor revision number 2 in X.Y1Y2.Z, e.g. 1.20.3."]
    #[inline(always)]
    pub fn y2(&self) -> Y2R {
        Y2R::new(((self.bits >> 4) & 0x0f) as u8)
    }
    #[doc = "Bits 8:11 - Minor revision number 1 in X.Y1Y2.Z, e.g. 1.20.3."]
    #[inline(always)]
    pub fn y1(&self) -> Y1R {
        Y1R::new(((self.bits >> 8) & 0x0f) as u8)
    }
    #[doc = "Bits 12:15 - Major revision number in X.Y1Y2.Z, e.g. 1.20.3."]
    #[inline(always)]
    pub fn x(&self) -> XR {
        XR::new(((self.bits >> 12) & 0x0f) as u8)
    }
    #[doc = "Bits 16:17 - Release milestone. 00-PREL, 01-BR, 10-SI, 11-GO."]
    #[inline(always)]
    pub fn milestone(&self) -> MilestoneR {
        MilestoneR::new(((self.bits >> 16) & 3) as u8)
    }
    #[doc = "Bits 18:31 - Reserved for Future Use"]
    #[inline(always)]
    pub fn version_rsvd_1(&self) -> VersionRsvd1R {
        VersionRsvd1R::new(((self.bits >> 18) & 0x3fff) as u16)
    }
}
#[doc = "SGI Version\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_version::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiVersionSpec;
impl crate::RegisterSpec for SgiVersionSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_version::R`](R) reader structure"]
impl crate::Readable for SgiVersionSpec {}
#[doc = "`reset()` method sets sgi_version to value 0"]
impl crate::Resettable for SgiVersionSpec {}

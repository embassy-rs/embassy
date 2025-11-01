#[doc = "Register `VERSION` reader"]
pub type R = crate::R<VersionSpec>;
#[doc = "Field `Reserved3` reader - Reserved"]
pub type Reserved3R = crate::FieldReader;
#[doc = "Field `Reserved7` reader - Reserved"]
pub type Reserved7R = crate::FieldReader;
#[doc = "Field `Reserved11` reader - Reserved"]
pub type Reserved11R = crate::FieldReader;
#[doc = "Field `Reserved15` reader - Reserved"]
pub type Reserved15R = crate::FieldReader;
#[doc = "Field `MILESTONE` reader - Release milestone. 00-PREL, 01-BR, 10-SI, 11-GO."]
pub type MilestoneR = crate::FieldReader;
#[doc = "Field `FSM_CONFIG` reader - 0:4 step, 1:8 step"]
pub type FsmConfigR = crate::BitReader;
#[doc = "Field `INDEX_CONFIG` reader - Configured number of addressable indexes"]
pub type IndexConfigR = crate::FieldReader;
#[doc = "Field `Reserved31` reader - Reserved for Future Use"]
pub type Reserved31R = crate::FieldReader;
impl R {
    #[doc = "Bits 0:3 - Reserved"]
    #[inline(always)]
    pub fn reserved3(&self) -> Reserved3R {
        Reserved3R::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bits 4:7 - Reserved"]
    #[inline(always)]
    pub fn reserved7(&self) -> Reserved7R {
        Reserved7R::new(((self.bits >> 4) & 0x0f) as u8)
    }
    #[doc = "Bits 8:11 - Reserved"]
    #[inline(always)]
    pub fn reserved11(&self) -> Reserved11R {
        Reserved11R::new(((self.bits >> 8) & 0x0f) as u8)
    }
    #[doc = "Bits 12:15 - Reserved"]
    #[inline(always)]
    pub fn reserved15(&self) -> Reserved15R {
        Reserved15R::new(((self.bits >> 12) & 0x0f) as u8)
    }
    #[doc = "Bits 16:17 - Release milestone. 00-PREL, 01-BR, 10-SI, 11-GO."]
    #[inline(always)]
    pub fn milestone(&self) -> MilestoneR {
        MilestoneR::new(((self.bits >> 16) & 3) as u8)
    }
    #[doc = "Bit 18 - 0:4 step, 1:8 step"]
    #[inline(always)]
    pub fn fsm_config(&self) -> FsmConfigR {
        FsmConfigR::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bits 19:26 - Configured number of addressable indexes"]
    #[inline(always)]
    pub fn index_config(&self) -> IndexConfigR {
        IndexConfigR::new(((self.bits >> 19) & 0xff) as u8)
    }
    #[doc = "Bits 27:31 - Reserved for Future Use"]
    #[inline(always)]
    pub fn reserved31(&self) -> Reserved31R {
        Reserved31R::new(((self.bits >> 27) & 0x1f) as u8)
    }
}
#[doc = "IP Version\n\nYou can [`read`](crate::Reg::read) this register and get [`version::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct VersionSpec;
impl crate::RegisterSpec for VersionSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`version::R`](R) reader structure"]
impl crate::Readable for VersionSpec {}
#[doc = "`reset()` method sets VERSION to value 0x007b_0100"]
impl crate::Resettable for VersionSpec {
    const RESET_VALUE: u32 = 0x007b_0100;
}

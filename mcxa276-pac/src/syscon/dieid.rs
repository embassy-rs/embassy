#[doc = "Register `DIEID` reader"]
pub type R = crate::R<DieidSpec>;
#[doc = "Field `MINOR_REVISION` reader - Chip minor revision"]
pub type MinorRevisionR = crate::FieldReader;
#[doc = "Field `MAJOR_REVISION` reader - Chip major revision"]
pub type MajorRevisionR = crate::FieldReader;
#[doc = "Field `MCO_NUM_IN_DIE_ID` reader - Chip number"]
pub type McoNumInDieIdR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:3 - Chip minor revision"]
    #[inline(always)]
    pub fn minor_revision(&self) -> MinorRevisionR {
        MinorRevisionR::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bits 4:7 - Chip major revision"]
    #[inline(always)]
    pub fn major_revision(&self) -> MajorRevisionR {
        MajorRevisionR::new(((self.bits >> 4) & 0x0f) as u8)
    }
    #[doc = "Bits 8:27 - Chip number"]
    #[inline(always)]
    pub fn mco_num_in_die_id(&self) -> McoNumInDieIdR {
        McoNumInDieIdR::new((self.bits >> 8) & 0x000f_ffff)
    }
}
#[doc = "Chip Revision ID and Number\n\nYou can [`read`](crate::Reg::read) this register and get [`dieid::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct DieidSpec;
impl crate::RegisterSpec for DieidSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`dieid::R`](R) reader structure"]
impl crate::Readable for DieidSpec {}
#[doc = "`reset()` method sets DIEID to value 0x005d_c1a0"]
impl crate::Resettable for DieidSpec {
    const RESET_VALUE: u32 = 0x005d_c1a0;
}

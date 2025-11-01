#[doc = "Register `PKC_MODULE_ID` reader"]
pub type R = crate::R<PkcModuleIdSpec>;
#[doc = "Field `SIZE` reader - Address space of the IP"]
pub type SizeR = crate::FieldReader;
#[doc = "Field `MINOR_REV` reader - Minor revision"]
pub type MinorRevR = crate::FieldReader;
#[doc = "Field `MAJOR_REV` reader - Major revision"]
pub type MajorRevR = crate::FieldReader;
#[doc = "Field `ID` reader - Module ID"]
pub type IdR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:7 - Address space of the IP"]
    #[inline(always)]
    pub fn size(&self) -> SizeR {
        SizeR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:11 - Minor revision"]
    #[inline(always)]
    pub fn minor_rev(&self) -> MinorRevR {
        MinorRevR::new(((self.bits >> 8) & 0x0f) as u8)
    }
    #[doc = "Bits 12:15 - Major revision"]
    #[inline(always)]
    pub fn major_rev(&self) -> MajorRevR {
        MajorRevR::new(((self.bits >> 12) & 0x0f) as u8)
    }
    #[doc = "Bits 16:31 - Module ID"]
    #[inline(always)]
    pub fn id(&self) -> IdR {
        IdR::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
#[doc = "Module ID\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_module_id::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkcModuleIdSpec;
impl crate::RegisterSpec for PkcModuleIdSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pkc_module_id::R`](R) reader structure"]
impl crate::Readable for PkcModuleIdSpec {}
#[doc = "`reset()` method sets PKC_MODULE_ID to value 0xe103_1280"]
impl crate::Resettable for PkcModuleIdSpec {
    const RESET_VALUE: u32 = 0xe103_1280;
}

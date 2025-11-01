#[doc = "Register `CORELDO_CFG` reader"]
pub type R = crate::R<CoreldoCfgSpec>;
impl core::fmt::Debug for R {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.bits())
    }
}
#[doc = "LDO_CORE Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`coreldo_cfg::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CoreldoCfgSpec;
impl crate::RegisterSpec for CoreldoCfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`coreldo_cfg::R`](R) reader structure"]
impl crate::Readable for CoreldoCfgSpec {}
#[doc = "`reset()` method sets CORELDO_CFG to value 0"]
impl crate::Resettable for CoreldoCfgSpec {}

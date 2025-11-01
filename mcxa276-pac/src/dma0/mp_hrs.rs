#[doc = "Register `MP_HRS` reader"]
pub type R = crate::R<MpHrsSpec>;
#[doc = "Field `HRS` reader - Hardware Request Status"]
pub type HrsR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:31 - Hardware Request Status"]
    #[inline(always)]
    pub fn hrs(&self) -> HrsR {
        HrsR::new(self.bits)
    }
}
#[doc = "Management Page Hardware Request Status\n\nYou can [`read`](crate::Reg::read) this register and get [`mp_hrs::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MpHrsSpec;
impl crate::RegisterSpec for MpHrsSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mp_hrs::R`](R) reader structure"]
impl crate::Readable for MpHrsSpec {}
#[doc = "`reset()` method sets MP_HRS to value 0"]
impl crate::Resettable for MpHrsSpec {}

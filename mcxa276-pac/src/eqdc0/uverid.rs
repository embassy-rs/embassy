#[doc = "Register `UVERID` reader"]
pub type R = crate::R<UveridSpec>;
#[doc = "Field `UVERID` reader - UVERID"]
pub type UveridR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - UVERID"]
    #[inline(always)]
    pub fn uverid(&self) -> UveridR {
        UveridR::new(self.bits)
    }
}
#[doc = "Upper VERID\n\nYou can [`read`](crate::Reg::read) this register and get [`uverid::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct UveridSpec;
impl crate::RegisterSpec for UveridSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`uverid::R`](R) reader structure"]
impl crate::Readable for UveridSpec {}
#[doc = "`reset()` method sets UVERID to value 0x01"]
impl crate::Resettable for UveridSpec {
    const RESET_VALUE: u16 = 0x01;
}

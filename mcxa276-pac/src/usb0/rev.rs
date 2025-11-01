#[doc = "Register `REV` reader"]
pub type R = crate::R<RevSpec>;
#[doc = "Field `REV` reader - Revision"]
pub type RevR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:7 - Revision"]
    #[inline(always)]
    pub fn rev(&self) -> RevR {
        RevR::new(self.bits)
    }
}
#[doc = "Peripheral Revision\n\nYou can [`read`](crate::Reg::read) this register and get [`rev::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RevSpec;
impl crate::RegisterSpec for RevSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`rev::R`](R) reader structure"]
impl crate::Readable for RevSpec {}
#[doc = "`reset()` method sets REV to value 0x33"]
impl crate::Resettable for RevSpec {
    const RESET_VALUE: u8 = 0x33;
}

#[doc = "Register `MP_INT` reader"]
pub type R = crate::R<MpIntSpec>;
#[doc = "Field `INT` reader - Interrupt Request Status"]
pub type IntR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:7 - Interrupt Request Status"]
    #[inline(always)]
    pub fn int(&self) -> IntR {
        IntR::new((self.bits & 0xff) as u8)
    }
}
#[doc = "Management Page Interrupt Request Status\n\nYou can [`read`](crate::Reg::read) this register and get [`mp_int::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MpIntSpec;
impl crate::RegisterSpec for MpIntSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mp_int::R`](R) reader structure"]
impl crate::Readable for MpIntSpec {}
#[doc = "`reset()` method sets MP_INT to value 0"]
impl crate::Resettable for MpIntSpec {}

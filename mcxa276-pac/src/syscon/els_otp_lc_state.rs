#[doc = "Register `ELS_OTP_LC_STATE` reader"]
pub type R = crate::R<ElsOtpLcStateSpec>;
#[doc = "Field `OTP_LC_STATE` reader - OTP life cycle state"]
pub type OtpLcStateR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:7 - OTP life cycle state"]
    #[inline(always)]
    pub fn otp_lc_state(&self) -> OtpLcStateR {
        OtpLcStateR::new((self.bits & 0xff) as u8)
    }
}
#[doc = "Life Cycle State Register\n\nYou can [`read`](crate::Reg::read) this register and get [`els_otp_lc_state::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ElsOtpLcStateSpec;
impl crate::RegisterSpec for ElsOtpLcStateSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`els_otp_lc_state::R`](R) reader structure"]
impl crate::Readable for ElsOtpLcStateSpec {}
#[doc = "`reset()` method sets ELS_OTP_LC_STATE to value 0"]
impl crate::Resettable for ElsOtpLcStateSpec {}

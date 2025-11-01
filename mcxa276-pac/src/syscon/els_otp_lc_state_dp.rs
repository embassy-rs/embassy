#[doc = "Register `ELS_OTP_LC_STATE_DP` reader"]
pub type R = crate::R<ElsOtpLcStateDpSpec>;
#[doc = "Field `OTP_LC_STATE_DP` reader - OTP life cycle state"]
pub type OtpLcStateDpR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:7 - OTP life cycle state"]
    #[inline(always)]
    pub fn otp_lc_state_dp(&self) -> OtpLcStateDpR {
        OtpLcStateDpR::new((self.bits & 0xff) as u8)
    }
}
#[doc = "Life Cycle State Register (Duplicate)\n\nYou can [`read`](crate::Reg::read) this register and get [`els_otp_lc_state_dp::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ElsOtpLcStateDpSpec;
impl crate::RegisterSpec for ElsOtpLcStateDpSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`els_otp_lc_state_dp::R`](R) reader structure"]
impl crate::Readable for ElsOtpLcStateDpSpec {}
#[doc = "`reset()` method sets ELS_OTP_LC_STATE_DP to value 0"]
impl crate::Resettable for ElsOtpLcStateDpSpec {}

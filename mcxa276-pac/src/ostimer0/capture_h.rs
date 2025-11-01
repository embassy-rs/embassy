#[doc = "Register `CAPTURE_H` reader"]
pub type R = crate::R<CaptureHSpec>;
#[doc = "Field `CAPTURE_VALUE` reader - EVTimer Capture Value"]
pub type CaptureValueR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:9 - EVTimer Capture Value"]
    #[inline(always)]
    pub fn capture_value(&self) -> CaptureValueR {
        CaptureValueR::new((self.bits & 0x03ff) as u16)
    }
}
#[doc = "Local Capture High for CPU\n\nYou can [`read`](crate::Reg::read) this register and get [`capture_h::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CaptureHSpec;
impl crate::RegisterSpec for CaptureHSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`capture_h::R`](R) reader structure"]
impl crate::Readable for CaptureHSpec {}
#[doc = "`reset()` method sets CAPTURE_H to value 0"]
impl crate::Resettable for CaptureHSpec {}

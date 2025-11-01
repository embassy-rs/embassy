#[doc = "Register `STATUS` reader"]
pub type R = crate::R<StatusSpec>;
#[doc = "Field `NUMTOF` reader - Number of TIMEOUT faults (FLAGS\\[TIMEOUT_FLAG\\]) since the last POR"]
pub type NumtofR = crate::FieldReader;
#[doc = "Field `NUMMISCOMPF` reader - Number of MISCOMPARE faults (FLAGS\\[MISCOMPARE_FLAG\\]) since the last POR"]
pub type NummiscompfR = crate::FieldReader;
#[doc = "Field `NUMILSEQF` reader - Number of SEQUENCE faults (FLAGS\\[SEQUENCE_FLAG\\]) since the last POR"]
pub type NumilseqfR = crate::FieldReader;
#[doc = "Field `CURST` reader - Current State"]
pub type CurstR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:7 - Number of TIMEOUT faults (FLAGS\\[TIMEOUT_FLAG\\]) since the last POR"]
    #[inline(always)]
    pub fn numtof(&self) -> NumtofR {
        NumtofR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Number of MISCOMPARE faults (FLAGS\\[MISCOMPARE_FLAG\\]) since the last POR"]
    #[inline(always)]
    pub fn nummiscompf(&self) -> NummiscompfR {
        NummiscompfR::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Number of SEQUENCE faults (FLAGS\\[SEQUENCE_FLAG\\]) since the last POR"]
    #[inline(always)]
    pub fn numilseqf(&self) -> NumilseqfR {
        NumilseqfR::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 28:31 - Current State"]
    #[inline(always)]
    pub fn curst(&self) -> CurstR {
        CurstR::new(((self.bits >> 28) & 0x0f) as u8)
    }
}
#[doc = "Status 1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`status::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct StatusSpec;
impl crate::RegisterSpec for StatusSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`status::R`](R) reader structure"]
impl crate::Readable for StatusSpec {}
#[doc = "`reset()` method sets STATUS to value 0x5000_0000"]
impl crate::Resettable for StatusSpec {
    const RESET_VALUE: u32 = 0x5000_0000;
}

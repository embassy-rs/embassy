#[doc = "Register `FPR` reader"]
pub type R = crate::R<FprSpec>;
#[doc = "Field `FIFO_RPT` reader - FIFO Read Pointer"]
pub type FifoRptR = crate::FieldReader;
#[doc = "Field `FIFO_WPT` reader - FIFO Write Pointer"]
pub type FifoWptR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:3 - FIFO Read Pointer"]
    #[inline(always)]
    pub fn fifo_rpt(&self) -> FifoRptR {
        FifoRptR::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bits 16:19 - FIFO Write Pointer"]
    #[inline(always)]
    pub fn fifo_wpt(&self) -> FifoWptR {
        FifoWptR::new(((self.bits >> 16) & 0x0f) as u8)
    }
}
#[doc = "DAC FIFO Pointer\n\nYou can [`read`](crate::Reg::read) this register and get [`fpr::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FprSpec;
impl crate::RegisterSpec for FprSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`fpr::R`](R) reader structure"]
impl crate::Readable for FprSpec {}
#[doc = "`reset()` method sets FPR to value 0"]
impl crate::Resettable for FprSpec {}

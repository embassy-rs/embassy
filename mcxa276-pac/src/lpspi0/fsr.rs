#[doc = "Register `FSR` reader"]
pub type R = crate::R<FsrSpec>;
#[doc = "Field `TXCOUNT` reader - Transmit FIFO Count"]
pub type TxcountR = crate::FieldReader;
#[doc = "Field `RXCOUNT` reader - Receive FIFO Count"]
pub type RxcountR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:2 - Transmit FIFO Count"]
    #[inline(always)]
    pub fn txcount(&self) -> TxcountR {
        TxcountR::new((self.bits & 7) as u8)
    }
    #[doc = "Bits 16:18 - Receive FIFO Count"]
    #[inline(always)]
    pub fn rxcount(&self) -> RxcountR {
        RxcountR::new(((self.bits >> 16) & 7) as u8)
    }
}
#[doc = "FIFO Status\n\nYou can [`read`](crate::Reg::read) this register and get [`fsr::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FsrSpec;
impl crate::RegisterSpec for FsrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`fsr::R`](R) reader structure"]
impl crate::Readable for FsrSpec {}
#[doc = "`reset()` method sets FSR to value 0"]
impl crate::Resettable for FsrSpec {}

#[doc = "Register `FDCRC` reader"]
pub type R = crate::R<FdcrcSpec>;
#[doc = "Field `FD_TXCRC` reader - Extended Transmitted CRC value"]
pub type FdTxcrcR = crate::FieldReader<u32>;
#[doc = "Field `FD_MBCRC` reader - CRC Message Buffer Number for FD_TXCRC"]
pub type FdMbcrcR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:20 - Extended Transmitted CRC value"]
    #[inline(always)]
    pub fn fd_txcrc(&self) -> FdTxcrcR {
        FdTxcrcR::new(self.bits & 0x001f_ffff)
    }
    #[doc = "Bits 24:30 - CRC Message Buffer Number for FD_TXCRC"]
    #[inline(always)]
    pub fn fd_mbcrc(&self) -> FdMbcrcR {
        FdMbcrcR::new(((self.bits >> 24) & 0x7f) as u8)
    }
}
#[doc = "CAN FD CRC\n\nYou can [`read`](crate::Reg::read) this register and get [`fdcrc::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FdcrcSpec;
impl crate::RegisterSpec for FdcrcSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`fdcrc::R`](R) reader structure"]
impl crate::Readable for FdcrcSpec {}
#[doc = "`reset()` method sets FDCRC to value 0"]
impl crate::Resettable for FdcrcSpec {}

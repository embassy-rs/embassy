#[doc = "Register `PKC_ACCESS_ERR` reader"]
pub type R = crate::R<PkcAccessErrSpec>;
#[doc = "Field `APB_NOTAV` reader - APB Error"]
pub type ApbNotavR = crate::BitReader;
#[doc = "Field `APB_WRGMD` reader - APB Error"]
pub type ApbWrgmdR = crate::BitReader;
#[doc = "Field `APB_MASTER` reader - APB Master that triggered first APB error (APB_WRGMD or APB_NOTAV)"]
pub type ApbMasterR = crate::FieldReader;
#[doc = "Field `AHB` reader - AHB Error"]
pub type AhbR = crate::BitReader;
#[doc = "Field `PKCC` reader - Error in PKC coprocessor kernel"]
pub type PkccR = crate::BitReader;
#[doc = "Field `FDET` reader - Error due to error detection circuitry"]
pub type FdetR = crate::BitReader;
#[doc = "Field `CTRL` reader - Error in PKC software control"]
pub type CtrlR = crate::BitReader;
#[doc = "Field `UCRC` reader - Error in layer2 CRC check."]
pub type UcrcR = crate::BitReader;
impl R {
    #[doc = "Bit 0 - APB Error"]
    #[inline(always)]
    pub fn apb_notav(&self) -> ApbNotavR {
        ApbNotavR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - APB Error"]
    #[inline(always)]
    pub fn apb_wrgmd(&self) -> ApbWrgmdR {
        ApbWrgmdR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bits 4:7 - APB Master that triggered first APB error (APB_WRGMD or APB_NOTAV)"]
    #[inline(always)]
    pub fn apb_master(&self) -> ApbMasterR {
        ApbMasterR::new(((self.bits >> 4) & 0x0f) as u8)
    }
    #[doc = "Bit 10 - AHB Error"]
    #[inline(always)]
    pub fn ahb(&self) -> AhbR {
        AhbR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 16 - Error in PKC coprocessor kernel"]
    #[inline(always)]
    pub fn pkcc(&self) -> PkccR {
        PkccR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Error due to error detection circuitry"]
    #[inline(always)]
    pub fn fdet(&self) -> FdetR {
        FdetR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Error in PKC software control"]
    #[inline(always)]
    pub fn ctrl(&self) -> CtrlR {
        CtrlR::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - Error in layer2 CRC check."]
    #[inline(always)]
    pub fn ucrc(&self) -> UcrcR {
        UcrcR::new(((self.bits >> 19) & 1) != 0)
    }
}
#[doc = "Access Error\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_access_err::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkcAccessErrSpec;
impl crate::RegisterSpec for PkcAccessErrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pkc_access_err::R`](R) reader structure"]
impl crate::Readable for PkcAccessErrSpec {}
#[doc = "`reset()` method sets PKC_ACCESS_ERR to value 0"]
impl crate::Resettable for PkcAccessErrSpec {}

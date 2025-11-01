#[doc = "Register `sgi_config` reader"]
pub type R = crate::R<SgiConfigSpec>;
#[doc = "Field `row` reader - SGI Diversified for 'ROW'"]
pub type RowR = crate::BitReader;
#[doc = "Field `china` reader - SGI Diversified for 'CHINA'"]
pub type ChinaR = crate::BitReader;
#[doc = "Field `cc` reader - SGI Diversified for 'CC'"]
pub type CcR = crate::BitReader;
#[doc = "Field `has_aes` reader - HAS AES"]
pub type HasAesR = crate::BitReader;
#[doc = "Field `has_des` reader - HAS DES"]
pub type HasDesR = crate::BitReader;
#[doc = "Field `has_sha` reader - HAS SHA"]
pub type HasShaR = crate::BitReader;
#[doc = "Field `has_movem` reader - HAS MOVEM"]
pub type HasMovemR = crate::BitReader;
#[doc = "Field `has_cmac` reader - HAS CMAC"]
pub type HasCmacR = crate::BitReader;
#[doc = "Field `has_gfmul` reader - HAS GFMUL"]
pub type HasGfmulR = crate::BitReader;
#[doc = "Field `internal_prng` reader - HAS INTERNAL PRNG"]
pub type InternalPrngR = crate::BitReader;
#[doc = "Field `key_digest` reader - HAS KEY DIGEST"]
pub type KeyDigestR = crate::BitReader;
#[doc = "Field `count_size` reader - 0 - COUNT=16, 1 - COUNT=32"]
pub type CountSizeR = crate::BitReader;
#[doc = "Field `configc_rsvd` reader - reserved"]
pub type ConfigcRsvdR = crate::BitReader;
#[doc = "Field `fa` reader - HAS FA protection"]
pub type FaR = crate::BitReader;
#[doc = "Field `configb2_rsvd` reader - reserved"]
pub type Configb2RsvdR = crate::BitReader;
#[doc = "Field `bus_width` reader - 0 - BUS_WIDTH=16, 1 - BUS_WIDTH=32"]
pub type BusWidthR = crate::BitReader;
#[doc = "Field `num_datin` reader - NUMBER OF DATIN REGBANKS"]
pub type NumDatinR = crate::FieldReader;
#[doc = "Field `num_key` reader - NUMBER OR KEY REGBANKS"]
pub type NumKeyR = crate::FieldReader;
#[doc = "Field `edc` reader - DATIN to KERNEL End-to-end EDC is enabled"]
pub type EdcR = crate::BitReader;
#[doc = "Field `configb_rsvd` reader - reserved"]
pub type ConfigbRsvdR = crate::FieldReader;
#[doc = "Field `sha_256_only` reader - HAS SHA-256 ONLY"]
pub type Sha256OnlyR = crate::BitReader;
#[doc = "Field `spb_support` reader - ID_CFG_SGI_SPB_SUPPORT is set"]
pub type SpbSupportR = crate::BitReader;
#[doc = "Field `spb_masking` reader - ID_CFG_SGI_SPB_MASKING is set"]
pub type SpbMaskingR = crate::BitReader;
#[doc = "Field `sfr_sw_mask` reader - ID_CFG_SGI_USE_SFR_SW_MASK is set"]
pub type SfrSwMaskR = crate::BitReader;
#[doc = "Field `configa_rsvd` reader - reserved"]
pub type ConfigaRsvdR = crate::FieldReader;
impl R {
    #[doc = "Bit 0 - SGI Diversified for 'ROW'"]
    #[inline(always)]
    pub fn row(&self) -> RowR {
        RowR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - SGI Diversified for 'CHINA'"]
    #[inline(always)]
    pub fn china(&self) -> ChinaR {
        ChinaR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - SGI Diversified for 'CC'"]
    #[inline(always)]
    pub fn cc(&self) -> CcR {
        CcR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - HAS AES"]
    #[inline(always)]
    pub fn has_aes(&self) -> HasAesR {
        HasAesR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - HAS DES"]
    #[inline(always)]
    pub fn has_des(&self) -> HasDesR {
        HasDesR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - HAS SHA"]
    #[inline(always)]
    pub fn has_sha(&self) -> HasShaR {
        HasShaR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - HAS MOVEM"]
    #[inline(always)]
    pub fn has_movem(&self) -> HasMovemR {
        HasMovemR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - HAS CMAC"]
    #[inline(always)]
    pub fn has_cmac(&self) -> HasCmacR {
        HasCmacR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - HAS GFMUL"]
    #[inline(always)]
    pub fn has_gfmul(&self) -> HasGfmulR {
        HasGfmulR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - HAS INTERNAL PRNG"]
    #[inline(always)]
    pub fn internal_prng(&self) -> InternalPrngR {
        InternalPrngR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - HAS KEY DIGEST"]
    #[inline(always)]
    pub fn key_digest(&self) -> KeyDigestR {
        KeyDigestR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - 0 - COUNT=16, 1 - COUNT=32"]
    #[inline(always)]
    pub fn count_size(&self) -> CountSizeR {
        CountSizeR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - reserved"]
    #[inline(always)]
    pub fn configc_rsvd(&self) -> ConfigcRsvdR {
        ConfigcRsvdR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - HAS FA protection"]
    #[inline(always)]
    pub fn fa(&self) -> FaR {
        FaR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - reserved"]
    #[inline(always)]
    pub fn configb2_rsvd(&self) -> Configb2RsvdR {
        Configb2RsvdR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - 0 - BUS_WIDTH=16, 1 - BUS_WIDTH=32"]
    #[inline(always)]
    pub fn bus_width(&self) -> BusWidthR {
        BusWidthR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bits 16:17 - NUMBER OF DATIN REGBANKS"]
    #[inline(always)]
    pub fn num_datin(&self) -> NumDatinR {
        NumDatinR::new(((self.bits >> 16) & 3) as u8)
    }
    #[doc = "Bits 18:20 - NUMBER OR KEY REGBANKS"]
    #[inline(always)]
    pub fn num_key(&self) -> NumKeyR {
        NumKeyR::new(((self.bits >> 18) & 7) as u8)
    }
    #[doc = "Bit 21 - DATIN to KERNEL End-to-end EDC is enabled"]
    #[inline(always)]
    pub fn edc(&self) -> EdcR {
        EdcR::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bits 22:23 - reserved"]
    #[inline(always)]
    pub fn configb_rsvd(&self) -> ConfigbRsvdR {
        ConfigbRsvdR::new(((self.bits >> 22) & 3) as u8)
    }
    #[doc = "Bit 24 - HAS SHA-256 ONLY"]
    #[inline(always)]
    pub fn sha_256_only(&self) -> Sha256OnlyR {
        Sha256OnlyR::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - ID_CFG_SGI_SPB_SUPPORT is set"]
    #[inline(always)]
    pub fn spb_support(&self) -> SpbSupportR {
        SpbSupportR::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - ID_CFG_SGI_SPB_MASKING is set"]
    #[inline(always)]
    pub fn spb_masking(&self) -> SpbMaskingR {
        SpbMaskingR::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - ID_CFG_SGI_USE_SFR_SW_MASK is set"]
    #[inline(always)]
    pub fn sfr_sw_mask(&self) -> SfrSwMaskR {
        SfrSwMaskR::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bits 28:31 - reserved"]
    #[inline(always)]
    pub fn configa_rsvd(&self) -> ConfigaRsvdR {
        ConfigaRsvdR::new(((self.bits >> 28) & 0x0f) as u8)
    }
}
#[doc = "SHA Configuration Reg\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_config::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiConfigSpec;
impl crate::RegisterSpec for SgiConfigSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_config::R`](R) reader structure"]
impl crate::Readable for SgiConfigSpec {}
#[doc = "`reset()` method sets sgi_config to value 0"]
impl crate::Resettable for SgiConfigSpec {}

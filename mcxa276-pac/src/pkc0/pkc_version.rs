#[doc = "Register `PKC_VERSION` reader"]
pub type R = crate::R<PkcVersionSpec>;
#[doc = "Native multiplier size and operand granularity\n\nValue on reset: 2"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Mulsize {
    #[doc = "2: 64-bit multiplier"]
    Bit64 = 2,
}
impl From<Mulsize> for u8 {
    #[inline(always)]
    fn from(variant: Mulsize) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Mulsize {
    type Ux = u8;
}
impl crate::IsEnum for Mulsize {}
#[doc = "Field `MULSIZE` reader - Native multiplier size and operand granularity"]
pub type MulsizeR = crate::FieldReader<Mulsize>;
impl MulsizeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Mulsize> {
        match self.bits {
            2 => Some(Mulsize::Bit64),
            _ => None,
        }
    }
    #[doc = "64-bit multiplier"]
    #[inline(always)]
    pub fn is_bit64(&self) -> bool {
        *self == Mulsize::Bit64
    }
}
#[doc = "Field `MCAVAIL` reader - MC feature (layer1 calculation) is available"]
pub type McavailR = crate::BitReader;
#[doc = "Field `UPAVAIL` reader - UP feature (layer2 calculation) is available"]
pub type UpavailR = crate::BitReader;
#[doc = "Field `UPCACHEAVAIL` reader - UP cache is available"]
pub type UpcacheavailR = crate::BitReader;
#[doc = "Field `GF2AVAIL` reader - GF2 calculation modes are available"]
pub type Gf2availR = crate::BitReader;
#[doc = "Field `PARAMNUM` reader - Number of parameter sets for real calculation"]
pub type ParamnumR = crate::FieldReader;
#[doc = "Field `SBX0AVAIL` reader - SBX0 operation is available"]
pub type Sbx0availR = crate::BitReader;
#[doc = "Field `SBX1AVAIL` reader - SBX1 operation is available"]
pub type Sbx1availR = crate::BitReader;
#[doc = "Field `SBX2AVAIL` reader - SBX2 operation is available"]
pub type Sbx2availR = crate::BitReader;
#[doc = "Field `SBX3AVAIL` reader - SBX3 operation is available"]
pub type Sbx3availR = crate::BitReader;
#[doc = "Field `MCRECONF_SIZE` reader - Size of reconfigurable MC table in bytes."]
pub type McreconfSizeR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:1 - Native multiplier size and operand granularity"]
    #[inline(always)]
    pub fn mulsize(&self) -> MulsizeR {
        MulsizeR::new((self.bits & 3) as u8)
    }
    #[doc = "Bit 2 - MC feature (layer1 calculation) is available"]
    #[inline(always)]
    pub fn mcavail(&self) -> McavailR {
        McavailR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - UP feature (layer2 calculation) is available"]
    #[inline(always)]
    pub fn upavail(&self) -> UpavailR {
        UpavailR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - UP cache is available"]
    #[inline(always)]
    pub fn upcacheavail(&self) -> UpcacheavailR {
        UpcacheavailR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - GF2 calculation modes are available"]
    #[inline(always)]
    pub fn gf2avail(&self) -> Gf2availR {
        Gf2availR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bits 6:7 - Number of parameter sets for real calculation"]
    #[inline(always)]
    pub fn paramnum(&self) -> ParamnumR {
        ParamnumR::new(((self.bits >> 6) & 3) as u8)
    }
    #[doc = "Bit 8 - SBX0 operation is available"]
    #[inline(always)]
    pub fn sbx0avail(&self) -> Sbx0availR {
        Sbx0availR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - SBX1 operation is available"]
    #[inline(always)]
    pub fn sbx1avail(&self) -> Sbx1availR {
        Sbx1availR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - SBX2 operation is available"]
    #[inline(always)]
    pub fn sbx2avail(&self) -> Sbx2availR {
        Sbx2availR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - SBX3 operation is available"]
    #[inline(always)]
    pub fn sbx3avail(&self) -> Sbx3availR {
        Sbx3availR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bits 12:19 - Size of reconfigurable MC table in bytes."]
    #[inline(always)]
    pub fn mcreconf_size(&self) -> McreconfSizeR {
        McreconfSizeR::new(((self.bits >> 12) & 0xff) as u8)
    }
}
#[doc = "PKC version register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_version::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkcVersionSpec;
impl crate::RegisterSpec for PkcVersionSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pkc_version::R`](R) reader structure"]
impl crate::Readable for PkcVersionSpec {}
#[doc = "`reset()` method sets PKC_VERSION to value 0xbe"]
impl crate::Resettable for PkcVersionSpec {
    const RESET_VALUE: u32 = 0xbe;
}

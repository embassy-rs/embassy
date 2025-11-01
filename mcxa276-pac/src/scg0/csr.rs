#[doc = "Register `CSR` reader"]
pub type R = crate::R<CsrSpec>;
#[doc = "System Clock Source\n\nValue on reset: 3"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Scs {
    #[doc = "1: SOSC"]
    Sosc = 1,
    #[doc = "2: SIRC"]
    Sirc = 2,
    #[doc = "3: FIRC"]
    Firc = 3,
    #[doc = "4: ROSC"]
    Rosc = 4,
    #[doc = "6: SPLL"]
    Spll = 6,
}
impl From<Scs> for u8 {
    #[inline(always)]
    fn from(variant: Scs) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Scs {
    type Ux = u8;
}
impl crate::IsEnum for Scs {}
#[doc = "Field `SCS` reader - System Clock Source"]
pub type ScsR = crate::FieldReader<Scs>;
impl ScsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Scs> {
        match self.bits {
            1 => Some(Scs::Sosc),
            2 => Some(Scs::Sirc),
            3 => Some(Scs::Firc),
            4 => Some(Scs::Rosc),
            6 => Some(Scs::Spll),
            _ => None,
        }
    }
    #[doc = "SOSC"]
    #[inline(always)]
    pub fn is_sosc(&self) -> bool {
        *self == Scs::Sosc
    }
    #[doc = "SIRC"]
    #[inline(always)]
    pub fn is_sirc(&self) -> bool {
        *self == Scs::Sirc
    }
    #[doc = "FIRC"]
    #[inline(always)]
    pub fn is_firc(&self) -> bool {
        *self == Scs::Firc
    }
    #[doc = "ROSC"]
    #[inline(always)]
    pub fn is_rosc(&self) -> bool {
        *self == Scs::Rosc
    }
    #[doc = "SPLL"]
    #[inline(always)]
    pub fn is_spll(&self) -> bool {
        *self == Scs::Spll
    }
}
impl R {
    #[doc = "Bits 24:26 - System Clock Source"]
    #[inline(always)]
    pub fn scs(&self) -> ScsR {
        ScsR::new(((self.bits >> 24) & 7) as u8)
    }
}
#[doc = "Clock Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`csr::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CsrSpec;
impl crate::RegisterSpec for CsrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`csr::R`](R) reader structure"]
impl crate::Readable for CsrSpec {}
#[doc = "`reset()` method sets CSR to value 0x0300_0000"]
impl crate::Resettable for CsrSpec {
    const RESET_VALUE: u32 = 0x0300_0000;
}

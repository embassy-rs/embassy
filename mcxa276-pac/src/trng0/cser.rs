#[doc = "Register `CSER` reader"]
pub type R = crate::R<CserSpec>;
#[doc = "Redundant Signals error/fault Detected\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RedSigs {
    #[doc = "0: No redundant signal error/fault"]
    RedSigsNoerr = 0,
    #[doc = "1: Redundant signal error/fault detected."]
    RedSigsErr = 1,
}
impl From<RedSigs> for bool {
    #[inline(always)]
    fn from(variant: RedSigs) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RED_SIGS` reader - Redundant Signals error/fault Detected"]
pub type RedSigsR = crate::BitReader<RedSigs>;
impl RedSigsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RedSigs {
        match self.bits {
            false => RedSigs::RedSigsNoerr,
            true => RedSigs::RedSigsErr,
        }
    }
    #[doc = "No redundant signal error/fault"]
    #[inline(always)]
    pub fn is_red_sigs_noerr(&self) -> bool {
        *self == RedSigs::RedSigsNoerr
    }
    #[doc = "Redundant signal error/fault detected."]
    #[inline(always)]
    pub fn is_red_sigs_err(&self) -> bool {
        *self == RedSigs::RedSigsErr
    }
}
#[doc = "Redundant FSM error/fault detected\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RedFsm {
    #[doc = "0: No redundant FSM error/fault"]
    RedFsmNoerr = 0,
    #[doc = "1: Redundant FSM error/fault detected."]
    RedFsmErr = 1,
}
impl From<RedFsm> for bool {
    #[inline(always)]
    fn from(variant: RedFsm) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RED_FSM` reader - Redundant FSM error/fault detected"]
pub type RedFsmR = crate::BitReader<RedFsm>;
impl RedFsmR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RedFsm {
        match self.bits {
            false => RedFsm::RedFsmNoerr,
            true => RedFsm::RedFsmErr,
        }
    }
    #[doc = "No redundant FSM error/fault"]
    #[inline(always)]
    pub fn is_red_fsm_noerr(&self) -> bool {
        *self == RedFsm::RedFsmNoerr
    }
    #[doc = "Redundant FSM error/fault detected."]
    #[inline(always)]
    pub fn is_red_fsm_err(&self) -> bool {
        *self == RedFsm::RedFsmErr
    }
}
#[doc = "Local-EDC error/fault detected\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalEdc {
    #[doc = "0: No Local-EDC error/fault detected."]
    LocalEdcNoerr = 0,
    #[doc = "1: Local-EDC error/fault detected."]
    LocalEdcErr = 1,
}
impl From<LocalEdc> for bool {
    #[inline(always)]
    fn from(variant: LocalEdc) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LOCAL_EDC` reader - Local-EDC error/fault detected"]
pub type LocalEdcR = crate::BitReader<LocalEdc>;
impl LocalEdcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> LocalEdc {
        match self.bits {
            false => LocalEdc::LocalEdcNoerr,
            true => LocalEdc::LocalEdcErr,
        }
    }
    #[doc = "No Local-EDC error/fault detected."]
    #[inline(always)]
    pub fn is_local_edc_noerr(&self) -> bool {
        *self == LocalEdc::LocalEdcNoerr
    }
    #[doc = "Local-EDC error/fault detected."]
    #[inline(always)]
    pub fn is_local_edc_err(&self) -> bool {
        *self == LocalEdc::LocalEdcErr
    }
}
#[doc = "Bus-EDC error/fault detected\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BusEdc {
    #[doc = "0: No Bus-EDC error/fault detected."]
    BusEdcNoerr = 0,
    #[doc = "1: Bus-EDC error/fault detected."]
    BusEdcErr = 1,
}
impl From<BusEdc> for bool {
    #[inline(always)]
    fn from(variant: BusEdc) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BUS_EDC` reader - Bus-EDC error/fault detected"]
pub type BusEdcR = crate::BitReader<BusEdc>;
impl BusEdcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> BusEdc {
        match self.bits {
            false => BusEdc::BusEdcNoerr,
            true => BusEdc::BusEdcErr,
        }
    }
    #[doc = "No Bus-EDC error/fault detected."]
    #[inline(always)]
    pub fn is_bus_edc_noerr(&self) -> bool {
        *self == BusEdc::BusEdcNoerr
    }
    #[doc = "Bus-EDC error/fault detected."]
    #[inline(always)]
    pub fn is_bus_edc_err(&self) -> bool {
        *self == BusEdc::BusEdcErr
    }
}
impl R {
    #[doc = "Bit 0 - Redundant Signals error/fault Detected"]
    #[inline(always)]
    pub fn red_sigs(&self) -> RedSigsR {
        RedSigsR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Redundant FSM error/fault detected"]
    #[inline(always)]
    pub fn red_fsm(&self) -> RedFsmR {
        RedFsmR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Local-EDC error/fault detected"]
    #[inline(always)]
    pub fn local_edc(&self) -> LocalEdcR {
        LocalEdcR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Bus-EDC error/fault detected"]
    #[inline(always)]
    pub fn bus_edc(&self) -> BusEdcR {
        BusEdcR::new(((self.bits >> 3) & 1) != 0)
    }
}
#[doc = "Common Security Error Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cser::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CserSpec;
impl crate::RegisterSpec for CserSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`cser::R`](R) reader structure"]
impl crate::Readable for CserSpec {}
#[doc = "`reset()` method sets CSER to value 0"]
impl crate::Resettable for CserSpec {}

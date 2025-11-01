#[doc = "Register `INT_STATUS` reader"]
pub type R = crate::R<IntStatusSpec>;
#[doc = "Read: TRNG Error. Any error in the TRNG will trigger this interrupt.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HwErr {
    #[doc = "0: No error."]
    HwErrNo = 0,
    #[doc = "1: Error detected."]
    HwErrYes = 1,
}
impl From<HwErr> for bool {
    #[inline(always)]
    fn from(variant: HwErr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HW_ERR` reader - Read: TRNG Error. Any error in the TRNG will trigger this interrupt."]
pub type HwErrR = crate::BitReader<HwErr>;
impl HwErrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> HwErr {
        match self.bits {
            false => HwErr::HwErrNo,
            true => HwErr::HwErrYes,
        }
    }
    #[doc = "No error."]
    #[inline(always)]
    pub fn is_hw_err_no(&self) -> bool {
        *self == HwErr::HwErrNo
    }
    #[doc = "Error detected."]
    #[inline(always)]
    pub fn is_hw_err_yes(&self) -> bool {
        *self == HwErr::HwErrYes
    }
}
#[doc = "Entropy Valid\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntVal {
    #[doc = "0: Busy generating entropy. Any value read from the Entropy registers is invalid."]
    EntValInvalid = 0,
    #[doc = "1: Values read from the Entropy registers are valid."]
    EntValValid = 1,
}
impl From<EntVal> for bool {
    #[inline(always)]
    fn from(variant: EntVal) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ENT_VAL` reader - Entropy Valid"]
pub type EntValR = crate::BitReader<EntVal>;
impl EntValR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> EntVal {
        match self.bits {
            false => EntVal::EntValInvalid,
            true => EntVal::EntValValid,
        }
    }
    #[doc = "Busy generating entropy. Any value read from the Entropy registers is invalid."]
    #[inline(always)]
    pub fn is_ent_val_invalid(&self) -> bool {
        *self == EntVal::EntValInvalid
    }
    #[doc = "Values read from the Entropy registers are valid."]
    #[inline(always)]
    pub fn is_ent_val_valid(&self) -> bool {
        *self == EntVal::EntValValid
    }
}
#[doc = "Frequency Count Fail\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrqCtFail {
    #[doc = "0: No hardware nor self test frequency errors."]
    FrqCtFailNoErr = 0,
    #[doc = "1: The frequency counter has detected a failure."]
    FrqCtFailErr = 1,
}
impl From<FrqCtFail> for bool {
    #[inline(always)]
    fn from(variant: FrqCtFail) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FRQ_CT_FAIL` reader - Frequency Count Fail"]
pub type FrqCtFailR = crate::BitReader<FrqCtFail>;
impl FrqCtFailR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> FrqCtFail {
        match self.bits {
            false => FrqCtFail::FrqCtFailNoErr,
            true => FrqCtFail::FrqCtFailErr,
        }
    }
    #[doc = "No hardware nor self test frequency errors."]
    #[inline(always)]
    pub fn is_frq_ct_fail_no_err(&self) -> bool {
        *self == FrqCtFail::FrqCtFailNoErr
    }
    #[doc = "The frequency counter has detected a failure."]
    #[inline(always)]
    pub fn is_frq_ct_fail_err(&self) -> bool {
        *self == FrqCtFail::FrqCtFailErr
    }
}
#[doc = "Integrity Fault. An internal fault has occurred.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntgFlt {
    #[doc = "0: No internal fault has been detected."]
    IntgFltNoErr = 0,
    #[doc = "1: TRNG has detected internal fault."]
    IntgFltErr = 1,
}
impl From<IntgFlt> for bool {
    #[inline(always)]
    fn from(variant: IntgFlt) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INTG_FLT` reader - Integrity Fault. An internal fault has occurred."]
pub type IntgFltR = crate::BitReader<IntgFlt>;
impl IntgFltR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> IntgFlt {
        match self.bits {
            false => IntgFlt::IntgFltNoErr,
            true => IntgFlt::IntgFltErr,
        }
    }
    #[doc = "No internal fault has been detected."]
    #[inline(always)]
    pub fn is_intg_flt_no_err(&self) -> bool {
        *self == IntgFlt::IntgFltNoErr
    }
    #[doc = "TRNG has detected internal fault."]
    #[inline(always)]
    pub fn is_intg_flt_err(&self) -> bool {
        *self == IntgFlt::IntgFltErr
    }
}
impl R {
    #[doc = "Bit 0 - Read: TRNG Error. Any error in the TRNG will trigger this interrupt."]
    #[inline(always)]
    pub fn hw_err(&self) -> HwErrR {
        HwErrR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Entropy Valid"]
    #[inline(always)]
    pub fn ent_val(&self) -> EntValR {
        EntValR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Frequency Count Fail"]
    #[inline(always)]
    pub fn frq_ct_fail(&self) -> FrqCtFailR {
        FrqCtFailR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Integrity Fault. An internal fault has occurred."]
    #[inline(always)]
    pub fn intg_flt(&self) -> IntgFltR {
        IntgFltR::new(((self.bits >> 3) & 1) != 0)
    }
}
#[doc = "Interrupt Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`int_status::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct IntStatusSpec;
impl crate::RegisterSpec for IntStatusSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`int_status::R`](R) reader structure"]
impl crate::Readable for IntStatusSpec {}
#[doc = "`reset()` method sets INT_STATUS to value 0"]
impl crate::Resettable for IntStatusSpec {}

#[doc = "Register `INT_CTRL` reader"]
pub type R = crate::R<IntCtrlSpec>;
#[doc = "Register `INT_CTRL` writer"]
pub type W = crate::W<IntCtrlSpec>;
#[doc = "Clear the HW_ERR interrupt.\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HwErr {
    #[doc = "0: Clears the INT_STATUS\\[HW_ERR\\] bit. Will automatically set after writing."]
    HwErrClear = 0,
    #[doc = "1: Enables the INT_STATUS\\[HW_ERR\\] bit to be set, thereby enabling interrupt generation for the HW_ERR condition."]
    HwErrOn = 1,
}
impl From<HwErr> for bool {
    #[inline(always)]
    fn from(variant: HwErr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HW_ERR` reader - Clear the HW_ERR interrupt."]
pub type HwErrR = crate::BitReader<HwErr>;
impl HwErrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> HwErr {
        match self.bits {
            false => HwErr::HwErrClear,
            true => HwErr::HwErrOn,
        }
    }
    #[doc = "Clears the INT_STATUS\\[HW_ERR\\] bit. Will automatically set after writing."]
    #[inline(always)]
    pub fn is_hw_err_clear(&self) -> bool {
        *self == HwErr::HwErrClear
    }
    #[doc = "Enables the INT_STATUS\\[HW_ERR\\] bit to be set, thereby enabling interrupt generation for the HW_ERR condition."]
    #[inline(always)]
    pub fn is_hw_err_on(&self) -> bool {
        *self == HwErr::HwErrOn
    }
}
#[doc = "Field `HW_ERR` writer - Clear the HW_ERR interrupt."]
pub type HwErrW<'a, REG> = crate::BitWriter<'a, REG, HwErr>;
impl<'a, REG> HwErrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Clears the INT_STATUS\\[HW_ERR\\] bit. Will automatically set after writing."]
    #[inline(always)]
    pub fn hw_err_clear(self) -> &'a mut crate::W<REG> {
        self.variant(HwErr::HwErrClear)
    }
    #[doc = "Enables the INT_STATUS\\[HW_ERR\\] bit to be set, thereby enabling interrupt generation for the HW_ERR condition."]
    #[inline(always)]
    pub fn hw_err_on(self) -> &'a mut crate::W<REG> {
        self.variant(HwErr::HwErrOn)
    }
}
#[doc = "Clear the ENT_VAL interrupt.\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntVal {
    #[doc = "0: Clears the INT_STATUS\\[ENT_VAL\\] bit. Will automatically set after writing."]
    EntValClear = 0,
    #[doc = "1: Enables the INT_STATUS\\[ENT_VAL\\] bit to be set, thereby enabling interrupt generation for the ENT_VAL condition."]
    EntValOn = 1,
}
impl From<EntVal> for bool {
    #[inline(always)]
    fn from(variant: EntVal) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ENT_VAL` reader - Clear the ENT_VAL interrupt."]
pub type EntValR = crate::BitReader<EntVal>;
impl EntValR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> EntVal {
        match self.bits {
            false => EntVal::EntValClear,
            true => EntVal::EntValOn,
        }
    }
    #[doc = "Clears the INT_STATUS\\[ENT_VAL\\] bit. Will automatically set after writing."]
    #[inline(always)]
    pub fn is_ent_val_clear(&self) -> bool {
        *self == EntVal::EntValClear
    }
    #[doc = "Enables the INT_STATUS\\[ENT_VAL\\] bit to be set, thereby enabling interrupt generation for the ENT_VAL condition."]
    #[inline(always)]
    pub fn is_ent_val_on(&self) -> bool {
        *self == EntVal::EntValOn
    }
}
#[doc = "Field `ENT_VAL` writer - Clear the ENT_VAL interrupt."]
pub type EntValW<'a, REG> = crate::BitWriter<'a, REG, EntVal>;
impl<'a, REG> EntValW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Clears the INT_STATUS\\[ENT_VAL\\] bit. Will automatically set after writing."]
    #[inline(always)]
    pub fn ent_val_clear(self) -> &'a mut crate::W<REG> {
        self.variant(EntVal::EntValClear)
    }
    #[doc = "Enables the INT_STATUS\\[ENT_VAL\\] bit to be set, thereby enabling interrupt generation for the ENT_VAL condition."]
    #[inline(always)]
    pub fn ent_val_on(self) -> &'a mut crate::W<REG> {
        self.variant(EntVal::EntValOn)
    }
}
#[doc = "Clear the FRQ_CT_FAIL interrupt.\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrqCtFail {
    #[doc = "0: Clears the INT_STATUS\\[FRQ_CT_FAIL\\] bit. Will automatically set after writing."]
    FrqCtFailClear = 0,
    #[doc = "1: Enables the INT_STATUS\\[FRQ_CT_FAIL\\] bit to be set, thereby enabling interrupt generation for the FRQ_CT_FAIL condition."]
    FrqCtFailOn = 1,
}
impl From<FrqCtFail> for bool {
    #[inline(always)]
    fn from(variant: FrqCtFail) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FRQ_CT_FAIL` reader - Clear the FRQ_CT_FAIL interrupt."]
pub type FrqCtFailR = crate::BitReader<FrqCtFail>;
impl FrqCtFailR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> FrqCtFail {
        match self.bits {
            false => FrqCtFail::FrqCtFailClear,
            true => FrqCtFail::FrqCtFailOn,
        }
    }
    #[doc = "Clears the INT_STATUS\\[FRQ_CT_FAIL\\] bit. Will automatically set after writing."]
    #[inline(always)]
    pub fn is_frq_ct_fail_clear(&self) -> bool {
        *self == FrqCtFail::FrqCtFailClear
    }
    #[doc = "Enables the INT_STATUS\\[FRQ_CT_FAIL\\] bit to be set, thereby enabling interrupt generation for the FRQ_CT_FAIL condition."]
    #[inline(always)]
    pub fn is_frq_ct_fail_on(&self) -> bool {
        *self == FrqCtFail::FrqCtFailOn
    }
}
#[doc = "Field `FRQ_CT_FAIL` writer - Clear the FRQ_CT_FAIL interrupt."]
pub type FrqCtFailW<'a, REG> = crate::BitWriter<'a, REG, FrqCtFail>;
impl<'a, REG> FrqCtFailW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Clears the INT_STATUS\\[FRQ_CT_FAIL\\] bit. Will automatically set after writing."]
    #[inline(always)]
    pub fn frq_ct_fail_clear(self) -> &'a mut crate::W<REG> {
        self.variant(FrqCtFail::FrqCtFailClear)
    }
    #[doc = "Enables the INT_STATUS\\[FRQ_CT_FAIL\\] bit to be set, thereby enabling interrupt generation for the FRQ_CT_FAIL condition."]
    #[inline(always)]
    pub fn frq_ct_fail_on(self) -> &'a mut crate::W<REG> {
        self.variant(FrqCtFail::FrqCtFailOn)
    }
}
#[doc = "Clear the INTG_FLT interrupt.\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntgFlt {
    #[doc = "0: Clears the INT_STATUS\\[INTG_FLT\\] bit. Will automatically set after writing."]
    IntgFltClear = 0,
    #[doc = "1: Enables the INT_STATUS\\[INTG_FLT\\] bit to be set, thereby enabling interrupt generation for the INTG_FLT condition."]
    IntgFltOn = 1,
}
impl From<IntgFlt> for bool {
    #[inline(always)]
    fn from(variant: IntgFlt) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INTG_FLT` reader - Clear the INTG_FLT interrupt."]
pub type IntgFltR = crate::BitReader<IntgFlt>;
impl IntgFltR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> IntgFlt {
        match self.bits {
            false => IntgFlt::IntgFltClear,
            true => IntgFlt::IntgFltOn,
        }
    }
    #[doc = "Clears the INT_STATUS\\[INTG_FLT\\] bit. Will automatically set after writing."]
    #[inline(always)]
    pub fn is_intg_flt_clear(&self) -> bool {
        *self == IntgFlt::IntgFltClear
    }
    #[doc = "Enables the INT_STATUS\\[INTG_FLT\\] bit to be set, thereby enabling interrupt generation for the INTG_FLT condition."]
    #[inline(always)]
    pub fn is_intg_flt_on(&self) -> bool {
        *self == IntgFlt::IntgFltOn
    }
}
#[doc = "Field `INTG_FLT` writer - Clear the INTG_FLT interrupt."]
pub type IntgFltW<'a, REG> = crate::BitWriter<'a, REG, IntgFlt>;
impl<'a, REG> IntgFltW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Clears the INT_STATUS\\[INTG_FLT\\] bit. Will automatically set after writing."]
    #[inline(always)]
    pub fn intg_flt_clear(self) -> &'a mut crate::W<REG> {
        self.variant(IntgFlt::IntgFltClear)
    }
    #[doc = "Enables the INT_STATUS\\[INTG_FLT\\] bit to be set, thereby enabling interrupt generation for the INTG_FLT condition."]
    #[inline(always)]
    pub fn intg_flt_on(self) -> &'a mut crate::W<REG> {
        self.variant(IntgFlt::IntgFltOn)
    }
}
impl R {
    #[doc = "Bit 0 - Clear the HW_ERR interrupt."]
    #[inline(always)]
    pub fn hw_err(&self) -> HwErrR {
        HwErrR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Clear the ENT_VAL interrupt."]
    #[inline(always)]
    pub fn ent_val(&self) -> EntValR {
        EntValR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Clear the FRQ_CT_FAIL interrupt."]
    #[inline(always)]
    pub fn frq_ct_fail(&self) -> FrqCtFailR {
        FrqCtFailR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Clear the INTG_FLT interrupt."]
    #[inline(always)]
    pub fn intg_flt(&self) -> IntgFltR {
        IntgFltR::new(((self.bits >> 3) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Clear the HW_ERR interrupt."]
    #[inline(always)]
    pub fn hw_err(&mut self) -> HwErrW<IntCtrlSpec> {
        HwErrW::new(self, 0)
    }
    #[doc = "Bit 1 - Clear the ENT_VAL interrupt."]
    #[inline(always)]
    pub fn ent_val(&mut self) -> EntValW<IntCtrlSpec> {
        EntValW::new(self, 1)
    }
    #[doc = "Bit 2 - Clear the FRQ_CT_FAIL interrupt."]
    #[inline(always)]
    pub fn frq_ct_fail(&mut self) -> FrqCtFailW<IntCtrlSpec> {
        FrqCtFailW::new(self, 2)
    }
    #[doc = "Bit 3 - Clear the INTG_FLT interrupt."]
    #[inline(always)]
    pub fn intg_flt(&mut self) -> IntgFltW<IntCtrlSpec> {
        IntgFltW::new(self, 3)
    }
}
#[doc = "Interrupt Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`int_ctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`int_ctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct IntCtrlSpec;
impl crate::RegisterSpec for IntCtrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`int_ctrl::R`](R) reader structure"]
impl crate::Readable for IntCtrlSpec {}
#[doc = "`write(|w| ..)` method takes [`int_ctrl::W`](W) writer structure"]
impl crate::Writable for IntCtrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets INT_CTRL to value 0x0f"]
impl crate::Resettable for IntCtrlSpec {
    const RESET_VALUE: u32 = 0x0f;
}

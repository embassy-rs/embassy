#[doc = "Register `INT_MASK` reader"]
pub type R = crate::R<IntMaskSpec>;
#[doc = "Register `INT_MASK` writer"]
pub type W = crate::W<IntMaskSpec>;
#[doc = "Mask the HW_ERR interrupt.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HwErr {
    #[doc = "0: HW_ERR interrupt is disabled."]
    HwErrMasked = 0,
    #[doc = "1: HW_ERR interrupt is enabled."]
    HwErrActive = 1,
}
impl From<HwErr> for bool {
    #[inline(always)]
    fn from(variant: HwErr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HW_ERR` reader - Mask the HW_ERR interrupt."]
pub type HwErrR = crate::BitReader<HwErr>;
impl HwErrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> HwErr {
        match self.bits {
            false => HwErr::HwErrMasked,
            true => HwErr::HwErrActive,
        }
    }
    #[doc = "HW_ERR interrupt is disabled."]
    #[inline(always)]
    pub fn is_hw_err_masked(&self) -> bool {
        *self == HwErr::HwErrMasked
    }
    #[doc = "HW_ERR interrupt is enabled."]
    #[inline(always)]
    pub fn is_hw_err_active(&self) -> bool {
        *self == HwErr::HwErrActive
    }
}
#[doc = "Field `HW_ERR` writer - Mask the HW_ERR interrupt."]
pub type HwErrW<'a, REG> = crate::BitWriter<'a, REG, HwErr>;
impl<'a, REG> HwErrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "HW_ERR interrupt is disabled."]
    #[inline(always)]
    pub fn hw_err_masked(self) -> &'a mut crate::W<REG> {
        self.variant(HwErr::HwErrMasked)
    }
    #[doc = "HW_ERR interrupt is enabled."]
    #[inline(always)]
    pub fn hw_err_active(self) -> &'a mut crate::W<REG> {
        self.variant(HwErr::HwErrActive)
    }
}
#[doc = "Mask the ENT_VAL interrupt.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntVal {
    #[doc = "0: ENT_VAL interrupt is disabled."]
    EntValMasked = 0,
    #[doc = "1: ENT_VAL interrupt is enabled."]
    EntValActive = 1,
}
impl From<EntVal> for bool {
    #[inline(always)]
    fn from(variant: EntVal) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ENT_VAL` reader - Mask the ENT_VAL interrupt."]
pub type EntValR = crate::BitReader<EntVal>;
impl EntValR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> EntVal {
        match self.bits {
            false => EntVal::EntValMasked,
            true => EntVal::EntValActive,
        }
    }
    #[doc = "ENT_VAL interrupt is disabled."]
    #[inline(always)]
    pub fn is_ent_val_masked(&self) -> bool {
        *self == EntVal::EntValMasked
    }
    #[doc = "ENT_VAL interrupt is enabled."]
    #[inline(always)]
    pub fn is_ent_val_active(&self) -> bool {
        *self == EntVal::EntValActive
    }
}
#[doc = "Field `ENT_VAL` writer - Mask the ENT_VAL interrupt."]
pub type EntValW<'a, REG> = crate::BitWriter<'a, REG, EntVal>;
impl<'a, REG> EntValW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "ENT_VAL interrupt is disabled."]
    #[inline(always)]
    pub fn ent_val_masked(self) -> &'a mut crate::W<REG> {
        self.variant(EntVal::EntValMasked)
    }
    #[doc = "ENT_VAL interrupt is enabled."]
    #[inline(always)]
    pub fn ent_val_active(self) -> &'a mut crate::W<REG> {
        self.variant(EntVal::EntValActive)
    }
}
#[doc = "Mask the FRQ_CT_FAIL interrupt.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrqCtFail {
    #[doc = "0: FRQ_CT_FAIL interrupt is disabled."]
    FrqCtFailMasked = 0,
    #[doc = "1: FRQ_CT_FAIL interrupt is enabled."]
    FrqCtFailActive = 1,
}
impl From<FrqCtFail> for bool {
    #[inline(always)]
    fn from(variant: FrqCtFail) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FRQ_CT_FAIL` reader - Mask the FRQ_CT_FAIL interrupt."]
pub type FrqCtFailR = crate::BitReader<FrqCtFail>;
impl FrqCtFailR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> FrqCtFail {
        match self.bits {
            false => FrqCtFail::FrqCtFailMasked,
            true => FrqCtFail::FrqCtFailActive,
        }
    }
    #[doc = "FRQ_CT_FAIL interrupt is disabled."]
    #[inline(always)]
    pub fn is_frq_ct_fail_masked(&self) -> bool {
        *self == FrqCtFail::FrqCtFailMasked
    }
    #[doc = "FRQ_CT_FAIL interrupt is enabled."]
    #[inline(always)]
    pub fn is_frq_ct_fail_active(&self) -> bool {
        *self == FrqCtFail::FrqCtFailActive
    }
}
#[doc = "Field `FRQ_CT_FAIL` writer - Mask the FRQ_CT_FAIL interrupt."]
pub type FrqCtFailW<'a, REG> = crate::BitWriter<'a, REG, FrqCtFail>;
impl<'a, REG> FrqCtFailW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "FRQ_CT_FAIL interrupt is disabled."]
    #[inline(always)]
    pub fn frq_ct_fail_masked(self) -> &'a mut crate::W<REG> {
        self.variant(FrqCtFail::FrqCtFailMasked)
    }
    #[doc = "FRQ_CT_FAIL interrupt is enabled."]
    #[inline(always)]
    pub fn frq_ct_fail_active(self) -> &'a mut crate::W<REG> {
        self.variant(FrqCtFail::FrqCtFailActive)
    }
}
#[doc = "Mask the INTG_FLT interrupt.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntgFlt {
    #[doc = "0: INTG_FLT interrupt is disabled."]
    IntgFltMasked = 0,
    #[doc = "1: INTG_FLT interrupt is enabled."]
    IntgFltActive = 1,
}
impl From<IntgFlt> for bool {
    #[inline(always)]
    fn from(variant: IntgFlt) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INTG_FLT` reader - Mask the INTG_FLT interrupt."]
pub type IntgFltR = crate::BitReader<IntgFlt>;
impl IntgFltR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> IntgFlt {
        match self.bits {
            false => IntgFlt::IntgFltMasked,
            true => IntgFlt::IntgFltActive,
        }
    }
    #[doc = "INTG_FLT interrupt is disabled."]
    #[inline(always)]
    pub fn is_intg_flt_masked(&self) -> bool {
        *self == IntgFlt::IntgFltMasked
    }
    #[doc = "INTG_FLT interrupt is enabled."]
    #[inline(always)]
    pub fn is_intg_flt_active(&self) -> bool {
        *self == IntgFlt::IntgFltActive
    }
}
#[doc = "Field `INTG_FLT` writer - Mask the INTG_FLT interrupt."]
pub type IntgFltW<'a, REG> = crate::BitWriter<'a, REG, IntgFlt>;
impl<'a, REG> IntgFltW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "INTG_FLT interrupt is disabled."]
    #[inline(always)]
    pub fn intg_flt_masked(self) -> &'a mut crate::W<REG> {
        self.variant(IntgFlt::IntgFltMasked)
    }
    #[doc = "INTG_FLT interrupt is enabled."]
    #[inline(always)]
    pub fn intg_flt_active(self) -> &'a mut crate::W<REG> {
        self.variant(IntgFlt::IntgFltActive)
    }
}
impl R {
    #[doc = "Bit 0 - Mask the HW_ERR interrupt."]
    #[inline(always)]
    pub fn hw_err(&self) -> HwErrR {
        HwErrR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Mask the ENT_VAL interrupt."]
    #[inline(always)]
    pub fn ent_val(&self) -> EntValR {
        EntValR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Mask the FRQ_CT_FAIL interrupt."]
    #[inline(always)]
    pub fn frq_ct_fail(&self) -> FrqCtFailR {
        FrqCtFailR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Mask the INTG_FLT interrupt."]
    #[inline(always)]
    pub fn intg_flt(&self) -> IntgFltR {
        IntgFltR::new(((self.bits >> 3) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Mask the HW_ERR interrupt."]
    #[inline(always)]
    pub fn hw_err(&mut self) -> HwErrW<IntMaskSpec> {
        HwErrW::new(self, 0)
    }
    #[doc = "Bit 1 - Mask the ENT_VAL interrupt."]
    #[inline(always)]
    pub fn ent_val(&mut self) -> EntValW<IntMaskSpec> {
        EntValW::new(self, 1)
    }
    #[doc = "Bit 2 - Mask the FRQ_CT_FAIL interrupt."]
    #[inline(always)]
    pub fn frq_ct_fail(&mut self) -> FrqCtFailW<IntMaskSpec> {
        FrqCtFailW::new(self, 2)
    }
    #[doc = "Bit 3 - Mask the INTG_FLT interrupt."]
    #[inline(always)]
    pub fn intg_flt(&mut self) -> IntgFltW<IntMaskSpec> {
        IntgFltW::new(self, 3)
    }
}
#[doc = "Mask Register\n\nYou can [`read`](crate::Reg::read) this register and get [`int_mask::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`int_mask::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct IntMaskSpec;
impl crate::RegisterSpec for IntMaskSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`int_mask::R`](R) reader structure"]
impl crate::Readable for IntMaskSpec {}
#[doc = "`write(|w| ..)` method takes [`int_mask::W`](W) writer structure"]
impl crate::Writable for IntMaskSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets INT_MASK to value 0"]
impl crate::Resettable for IntMaskSpec {}

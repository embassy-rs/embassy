#[doc = "Register `OPAMP_CTRL` reader"]
pub type R = crate::R<OpampCtrlSpec>;
#[doc = "Register `OPAMP_CTRL` writer"]
pub type W = crate::W<OpampCtrlSpec>;
#[doc = "OPAMP Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpaEn {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<OpaEn> for bool {
    #[inline(always)]
    fn from(variant: OpaEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OPA_EN` reader - OPAMP Enable"]
pub type OpaEnR = crate::BitReader<OpaEn>;
impl OpaEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> OpaEn {
        match self.bits {
            false => OpaEn::Disable,
            true => OpaEn::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == OpaEn::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == OpaEn::Enable
    }
}
#[doc = "Field `OPA_EN` writer - OPAMP Enable"]
pub type OpaEnW<'a, REG> = crate::BitWriter<'a, REG, OpaEn>;
impl<'a, REG> OpaEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(OpaEn::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(OpaEn::Enable)
    }
}
#[doc = "Compensation capcitor config selection\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum OpaCcSel {
    #[doc = "0: Fit 2X gains"]
    Tbd1 = 0,
    #[doc = "1: Fit 4X gains"]
    Tbd2 = 1,
    #[doc = "2: Fit 8X gains"]
    Tbd3 = 2,
    #[doc = "3: Fit 16X gains"]
    Tbd4 = 3,
}
impl From<OpaCcSel> for u8 {
    #[inline(always)]
    fn from(variant: OpaCcSel) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for OpaCcSel {
    type Ux = u8;
}
impl crate::IsEnum for OpaCcSel {}
#[doc = "Field `OPA_CC_SEL` reader - Compensation capcitor config selection"]
pub type OpaCcSelR = crate::FieldReader<OpaCcSel>;
impl OpaCcSelR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> OpaCcSel {
        match self.bits {
            0 => OpaCcSel::Tbd1,
            1 => OpaCcSel::Tbd2,
            2 => OpaCcSel::Tbd3,
            3 => OpaCcSel::Tbd4,
            _ => unreachable!(),
        }
    }
    #[doc = "Fit 2X gains"]
    #[inline(always)]
    pub fn is_tbd1(&self) -> bool {
        *self == OpaCcSel::Tbd1
    }
    #[doc = "Fit 4X gains"]
    #[inline(always)]
    pub fn is_tbd2(&self) -> bool {
        *self == OpaCcSel::Tbd2
    }
    #[doc = "Fit 8X gains"]
    #[inline(always)]
    pub fn is_tbd3(&self) -> bool {
        *self == OpaCcSel::Tbd3
    }
    #[doc = "Fit 16X gains"]
    #[inline(always)]
    pub fn is_tbd4(&self) -> bool {
        *self == OpaCcSel::Tbd4
    }
}
#[doc = "Field `OPA_CC_SEL` writer - Compensation capcitor config selection"]
pub type OpaCcSelW<'a, REG> = crate::FieldWriter<'a, REG, 2, OpaCcSel, crate::Safe>;
impl<'a, REG> OpaCcSelW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Fit 2X gains"]
    #[inline(always)]
    pub fn tbd1(self) -> &'a mut crate::W<REG> {
        self.variant(OpaCcSel::Tbd1)
    }
    #[doc = "Fit 4X gains"]
    #[inline(always)]
    pub fn tbd2(self) -> &'a mut crate::W<REG> {
        self.variant(OpaCcSel::Tbd2)
    }
    #[doc = "Fit 8X gains"]
    #[inline(always)]
    pub fn tbd3(self) -> &'a mut crate::W<REG> {
        self.variant(OpaCcSel::Tbd3)
    }
    #[doc = "Fit 16X gains"]
    #[inline(always)]
    pub fn tbd4(self) -> &'a mut crate::W<REG> {
        self.variant(OpaCcSel::Tbd4)
    }
}
#[doc = "Bias current config selection\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum OpaBcSel {
    #[doc = "0: Default value. Keep power consumption constant"]
    Tbd1 = 0,
    #[doc = "1: Reduce power consumption to 1/4"]
    Tbd2 = 1,
    #[doc = "2: Reduce power consumption to 1/2"]
    Tbd3 = 2,
    #[doc = "3: Double the power consumption"]
    Tbd4 = 3,
}
impl From<OpaBcSel> for u8 {
    #[inline(always)]
    fn from(variant: OpaBcSel) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for OpaBcSel {
    type Ux = u8;
}
impl crate::IsEnum for OpaBcSel {}
#[doc = "Field `OPA_BC_SEL` reader - Bias current config selection"]
pub type OpaBcSelR = crate::FieldReader<OpaBcSel>;
impl OpaBcSelR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> OpaBcSel {
        match self.bits {
            0 => OpaBcSel::Tbd1,
            1 => OpaBcSel::Tbd2,
            2 => OpaBcSel::Tbd3,
            3 => OpaBcSel::Tbd4,
            _ => unreachable!(),
        }
    }
    #[doc = "Default value. Keep power consumption constant"]
    #[inline(always)]
    pub fn is_tbd1(&self) -> bool {
        *self == OpaBcSel::Tbd1
    }
    #[doc = "Reduce power consumption to 1/4"]
    #[inline(always)]
    pub fn is_tbd2(&self) -> bool {
        *self == OpaBcSel::Tbd2
    }
    #[doc = "Reduce power consumption to 1/2"]
    #[inline(always)]
    pub fn is_tbd3(&self) -> bool {
        *self == OpaBcSel::Tbd3
    }
    #[doc = "Double the power consumption"]
    #[inline(always)]
    pub fn is_tbd4(&self) -> bool {
        *self == OpaBcSel::Tbd4
    }
}
#[doc = "Field `OPA_BC_SEL` writer - Bias current config selection"]
pub type OpaBcSelW<'a, REG> = crate::FieldWriter<'a, REG, 2, OpaBcSel, crate::Safe>;
impl<'a, REG> OpaBcSelW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Default value. Keep power consumption constant"]
    #[inline(always)]
    pub fn tbd1(self) -> &'a mut crate::W<REG> {
        self.variant(OpaBcSel::Tbd1)
    }
    #[doc = "Reduce power consumption to 1/4"]
    #[inline(always)]
    pub fn tbd2(self) -> &'a mut crate::W<REG> {
        self.variant(OpaBcSel::Tbd2)
    }
    #[doc = "Reduce power consumption to 1/2"]
    #[inline(always)]
    pub fn tbd3(self) -> &'a mut crate::W<REG> {
        self.variant(OpaBcSel::Tbd3)
    }
    #[doc = "Double the power consumption"]
    #[inline(always)]
    pub fn tbd4(self) -> &'a mut crate::W<REG> {
        self.variant(OpaBcSel::Tbd4)
    }
}
impl R {
    #[doc = "Bit 0 - OPAMP Enable"]
    #[inline(always)]
    pub fn opa_en(&self) -> OpaEnR {
        OpaEnR::new((self.bits & 1) != 0)
    }
    #[doc = "Bits 4:5 - Compensation capcitor config selection"]
    #[inline(always)]
    pub fn opa_cc_sel(&self) -> OpaCcSelR {
        OpaCcSelR::new(((self.bits >> 4) & 3) as u8)
    }
    #[doc = "Bits 6:7 - Bias current config selection"]
    #[inline(always)]
    pub fn opa_bc_sel(&self) -> OpaBcSelR {
        OpaBcSelR::new(((self.bits >> 6) & 3) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - OPAMP Enable"]
    #[inline(always)]
    pub fn opa_en(&mut self) -> OpaEnW<OpampCtrlSpec> {
        OpaEnW::new(self, 0)
    }
    #[doc = "Bits 4:5 - Compensation capcitor config selection"]
    #[inline(always)]
    pub fn opa_cc_sel(&mut self) -> OpaCcSelW<OpampCtrlSpec> {
        OpaCcSelW::new(self, 4)
    }
    #[doc = "Bits 6:7 - Bias current config selection"]
    #[inline(always)]
    pub fn opa_bc_sel(&mut self) -> OpaBcSelW<OpampCtrlSpec> {
        OpaBcSelW::new(self, 6)
    }
}
#[doc = "OPAMP Control\n\nYou can [`read`](crate::Reg::read) this register and get [`opamp_ctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`opamp_ctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct OpampCtrlSpec;
impl crate::RegisterSpec for OpampCtrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`opamp_ctrl::R`](R) reader structure"]
impl crate::Readable for OpampCtrlSpec {}
#[doc = "`write(|w| ..)` method takes [`opamp_ctrl::W`](W) writer structure"]
impl crate::Writable for OpampCtrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets OPAMP_CTRL to value 0"]
impl crate::Resettable for OpampCtrlSpec {}

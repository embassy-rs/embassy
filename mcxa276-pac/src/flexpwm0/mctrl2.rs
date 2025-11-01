#[doc = "Register `MCTRL2` reader"]
pub type R = crate::R<Mctrl2Spec>;
#[doc = "Register `MCTRL2` writer"]
pub type W = crate::W<Mctrl2Spec>;
#[doc = "Write protect\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wrprot {
    #[doc = "0: Write protection off (default)."]
    Disabled = 0,
    #[doc = "1: Write protection on."]
    Enabled = 1,
    #[doc = "2: Write protection off and locked until chip reset."]
    DisabledLocked = 2,
    #[doc = "3: Write protection on and locked until chip reset."]
    EnabledLocked = 3,
}
impl From<Wrprot> for u8 {
    #[inline(always)]
    fn from(variant: Wrprot) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wrprot {
    type Ux = u8;
}
impl crate::IsEnum for Wrprot {}
#[doc = "Field `WRPROT` reader - Write protect"]
pub type WrprotR = crate::FieldReader<Wrprot>;
impl WrprotR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wrprot {
        match self.bits {
            0 => Wrprot::Disabled,
            1 => Wrprot::Enabled,
            2 => Wrprot::DisabledLocked,
            3 => Wrprot::EnabledLocked,
            _ => unreachable!(),
        }
    }
    #[doc = "Write protection off (default)."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Wrprot::Disabled
    }
    #[doc = "Write protection on."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Wrprot::Enabled
    }
    #[doc = "Write protection off and locked until chip reset."]
    #[inline(always)]
    pub fn is_disabled_locked(&self) -> bool {
        *self == Wrprot::DisabledLocked
    }
    #[doc = "Write protection on and locked until chip reset."]
    #[inline(always)]
    pub fn is_enabled_locked(&self) -> bool {
        *self == Wrprot::EnabledLocked
    }
}
#[doc = "Field `WRPROT` writer - Write protect"]
pub type WrprotW<'a, REG> = crate::FieldWriter<'a, REG, 2, Wrprot, crate::Safe>;
impl<'a, REG> WrprotW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Write protection off (default)."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Wrprot::Disabled)
    }
    #[doc = "Write protection on."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Wrprot::Enabled)
    }
    #[doc = "Write protection off and locked until chip reset."]
    #[inline(always)]
    pub fn disabled_locked(self) -> &'a mut crate::W<REG> {
        self.variant(Wrprot::DisabledLocked)
    }
    #[doc = "Write protection on and locked until chip reset."]
    #[inline(always)]
    pub fn enabled_locked(self) -> &'a mut crate::W<REG> {
        self.variant(Wrprot::EnabledLocked)
    }
}
#[doc = "Stretch IPBus clock count prescaler for mux0_trig/mux1_trig/out0_trig/out1_trig/pwma_trig/pwmb_trig\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum StretchCntPrsc {
    #[doc = "0: Stretch count is zero, no stretch."]
    Disabled = 0,
    #[doc = "1: Stretch mux0_trig/mux1_trig/out0_trig/out1_trig/pwma_trig/pwmb_trig for 2 IPBus clock period."]
    Enabled = 1,
    #[doc = "2: Stretch mux0_trig/mux1_trig/out0_trig/out1_trig/pwma_trig/pwmb_trig for 4 IPBus clock period."]
    DisabledLocked = 2,
    #[doc = "3: Stretch mux0_trig/mux1_trig/out0_trig/out1_trig/pwma_trig/pwmb_trig for 8 IPBus clock period."]
    EnabledLocked = 3,
}
impl From<StretchCntPrsc> for u8 {
    #[inline(always)]
    fn from(variant: StretchCntPrsc) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for StretchCntPrsc {
    type Ux = u8;
}
impl crate::IsEnum for StretchCntPrsc {}
#[doc = "Field `STRETCH_CNT_PRSC` reader - Stretch IPBus clock count prescaler for mux0_trig/mux1_trig/out0_trig/out1_trig/pwma_trig/pwmb_trig"]
pub type StretchCntPrscR = crate::FieldReader<StretchCntPrsc>;
impl StretchCntPrscR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StretchCntPrsc {
        match self.bits {
            0 => StretchCntPrsc::Disabled,
            1 => StretchCntPrsc::Enabled,
            2 => StretchCntPrsc::DisabledLocked,
            3 => StretchCntPrsc::EnabledLocked,
            _ => unreachable!(),
        }
    }
    #[doc = "Stretch count is zero, no stretch."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == StretchCntPrsc::Disabled
    }
    #[doc = "Stretch mux0_trig/mux1_trig/out0_trig/out1_trig/pwma_trig/pwmb_trig for 2 IPBus clock period."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == StretchCntPrsc::Enabled
    }
    #[doc = "Stretch mux0_trig/mux1_trig/out0_trig/out1_trig/pwma_trig/pwmb_trig for 4 IPBus clock period."]
    #[inline(always)]
    pub fn is_disabled_locked(&self) -> bool {
        *self == StretchCntPrsc::DisabledLocked
    }
    #[doc = "Stretch mux0_trig/mux1_trig/out0_trig/out1_trig/pwma_trig/pwmb_trig for 8 IPBus clock period."]
    #[inline(always)]
    pub fn is_enabled_locked(&self) -> bool {
        *self == StretchCntPrsc::EnabledLocked
    }
}
#[doc = "Field `STRETCH_CNT_PRSC` writer - Stretch IPBus clock count prescaler for mux0_trig/mux1_trig/out0_trig/out1_trig/pwma_trig/pwmb_trig"]
pub type StretchCntPrscW<'a, REG> = crate::FieldWriter<'a, REG, 2, StretchCntPrsc, crate::Safe>;
impl<'a, REG> StretchCntPrscW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Stretch count is zero, no stretch."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(StretchCntPrsc::Disabled)
    }
    #[doc = "Stretch mux0_trig/mux1_trig/out0_trig/out1_trig/pwma_trig/pwmb_trig for 2 IPBus clock period."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(StretchCntPrsc::Enabled)
    }
    #[doc = "Stretch mux0_trig/mux1_trig/out0_trig/out1_trig/pwma_trig/pwmb_trig for 4 IPBus clock period."]
    #[inline(always)]
    pub fn disabled_locked(self) -> &'a mut crate::W<REG> {
        self.variant(StretchCntPrsc::DisabledLocked)
    }
    #[doc = "Stretch mux0_trig/mux1_trig/out0_trig/out1_trig/pwma_trig/pwmb_trig for 8 IPBus clock period."]
    #[inline(always)]
    pub fn enabled_locked(self) -> &'a mut crate::W<REG> {
        self.variant(StretchCntPrsc::EnabledLocked)
    }
}
impl R {
    #[doc = "Bits 2:3 - Write protect"]
    #[inline(always)]
    pub fn wrprot(&self) -> WrprotR {
        WrprotR::new(((self.bits >> 2) & 3) as u8)
    }
    #[doc = "Bits 6:7 - Stretch IPBus clock count prescaler for mux0_trig/mux1_trig/out0_trig/out1_trig/pwma_trig/pwmb_trig"]
    #[inline(always)]
    pub fn stretch_cnt_prsc(&self) -> StretchCntPrscR {
        StretchCntPrscR::new(((self.bits >> 6) & 3) as u8)
    }
}
impl W {
    #[doc = "Bits 2:3 - Write protect"]
    #[inline(always)]
    pub fn wrprot(&mut self) -> WrprotW<Mctrl2Spec> {
        WrprotW::new(self, 2)
    }
    #[doc = "Bits 6:7 - Stretch IPBus clock count prescaler for mux0_trig/mux1_trig/out0_trig/out1_trig/pwma_trig/pwmb_trig"]
    #[inline(always)]
    pub fn stretch_cnt_prsc(&mut self) -> StretchCntPrscW<Mctrl2Spec> {
        StretchCntPrscW::new(self, 6)
    }
}
#[doc = "Master Control 2 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mctrl2::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mctrl2::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Mctrl2Spec;
impl crate::RegisterSpec for Mctrl2Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`mctrl2::R`](R) reader structure"]
impl crate::Readable for Mctrl2Spec {}
#[doc = "`write(|w| ..)` method takes [`mctrl2::W`](W) writer structure"]
impl crate::Writable for Mctrl2Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MCTRL2 to value 0"]
impl crate::Resettable for Mctrl2Spec {}

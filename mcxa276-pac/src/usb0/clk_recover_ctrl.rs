#[doc = "Register `CLK_RECOVER_CTRL` reader"]
pub type R = crate::R<ClkRecoverCtrlSpec>;
#[doc = "Register `CLK_RECOVER_CTRL` writer"]
pub type W = crate::W<ClkRecoverCtrlSpec>;
#[doc = "Selects the source for the initial FIRC trim fine value used after a reset.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrimInitValSel {
    #[doc = "0: Mid-scale"]
    InitTrimFineMid = 0,
    #[doc = "1: IFR"]
    InitTrimFineIfr = 1,
}
impl From<TrimInitValSel> for bool {
    #[inline(always)]
    fn from(variant: TrimInitValSel) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TRIM_INIT_VAL_SEL` reader - Selects the source for the initial FIRC trim fine value used after a reset."]
pub type TrimInitValSelR = crate::BitReader<TrimInitValSel>;
impl TrimInitValSelR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> TrimInitValSel {
        match self.bits {
            false => TrimInitValSel::InitTrimFineMid,
            true => TrimInitValSel::InitTrimFineIfr,
        }
    }
    #[doc = "Mid-scale"]
    #[inline(always)]
    pub fn is_init_trim_fine_mid(&self) -> bool {
        *self == TrimInitValSel::InitTrimFineMid
    }
    #[doc = "IFR"]
    #[inline(always)]
    pub fn is_init_trim_fine_ifr(&self) -> bool {
        *self == TrimInitValSel::InitTrimFineIfr
    }
}
#[doc = "Field `TRIM_INIT_VAL_SEL` writer - Selects the source for the initial FIRC trim fine value used after a reset."]
pub type TrimInitValSelW<'a, REG> = crate::BitWriter<'a, REG, TrimInitValSel>;
impl<'a, REG> TrimInitValSelW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Mid-scale"]
    #[inline(always)]
    pub fn init_trim_fine_mid(self) -> &'a mut crate::W<REG> {
        self.variant(TrimInitValSel::InitTrimFineMid)
    }
    #[doc = "IFR"]
    #[inline(always)]
    pub fn init_trim_fine_ifr(self) -> &'a mut crate::W<REG> {
        self.variant(TrimInitValSel::InitTrimFineIfr)
    }
}
#[doc = "Restart from IFR Trim Value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestartIfrtrimEn {
    #[doc = "0: Trim fine adjustment always works based on the previous updated trim fine value."]
    LoadTrimFineMid = 0,
    #[doc = "1: Trim fine restarts from the IFR trim value whenever you detect bus_reset or bus_resume or deassert module enable."]
    LoadTrimFineIfr = 1,
}
impl From<RestartIfrtrimEn> for bool {
    #[inline(always)]
    fn from(variant: RestartIfrtrimEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RESTART_IFRTRIM_EN` reader - Restart from IFR Trim Value"]
pub type RestartIfrtrimEnR = crate::BitReader<RestartIfrtrimEn>;
impl RestartIfrtrimEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RestartIfrtrimEn {
        match self.bits {
            false => RestartIfrtrimEn::LoadTrimFineMid,
            true => RestartIfrtrimEn::LoadTrimFineIfr,
        }
    }
    #[doc = "Trim fine adjustment always works based on the previous updated trim fine value."]
    #[inline(always)]
    pub fn is_load_trim_fine_mid(&self) -> bool {
        *self == RestartIfrtrimEn::LoadTrimFineMid
    }
    #[doc = "Trim fine restarts from the IFR trim value whenever you detect bus_reset or bus_resume or deassert module enable."]
    #[inline(always)]
    pub fn is_load_trim_fine_ifr(&self) -> bool {
        *self == RestartIfrtrimEn::LoadTrimFineIfr
    }
}
#[doc = "Field `RESTART_IFRTRIM_EN` writer - Restart from IFR Trim Value"]
pub type RestartIfrtrimEnW<'a, REG> = crate::BitWriter<'a, REG, RestartIfrtrimEn>;
impl<'a, REG> RestartIfrtrimEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Trim fine adjustment always works based on the previous updated trim fine value."]
    #[inline(always)]
    pub fn load_trim_fine_mid(self) -> &'a mut crate::W<REG> {
        self.variant(RestartIfrtrimEn::LoadTrimFineMid)
    }
    #[doc = "Trim fine restarts from the IFR trim value whenever you detect bus_reset or bus_resume or deassert module enable."]
    #[inline(always)]
    pub fn load_trim_fine_ifr(self) -> &'a mut crate::W<REG> {
        self.variant(RestartIfrtrimEn::LoadTrimFineIfr)
    }
}
#[doc = "Reset or Resume to Rough Phase Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResetResumeRoughEn {
    #[doc = "0: Always works in tracking phase after the first time rough phase, to track transition."]
    KeepTrimFineOnReset = 0,
    #[doc = "1: Go back to rough stage whenever a bus reset or bus resume occurs."]
    UseIfrTrimFineOnReset = 1,
}
impl From<ResetResumeRoughEn> for bool {
    #[inline(always)]
    fn from(variant: ResetResumeRoughEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RESET_RESUME_ROUGH_EN` reader - Reset or Resume to Rough Phase Enable"]
pub type ResetResumeRoughEnR = crate::BitReader<ResetResumeRoughEn>;
impl ResetResumeRoughEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> ResetResumeRoughEn {
        match self.bits {
            false => ResetResumeRoughEn::KeepTrimFineOnReset,
            true => ResetResumeRoughEn::UseIfrTrimFineOnReset,
        }
    }
    #[doc = "Always works in tracking phase after the first time rough phase, to track transition."]
    #[inline(always)]
    pub fn is_keep_trim_fine_on_reset(&self) -> bool {
        *self == ResetResumeRoughEn::KeepTrimFineOnReset
    }
    #[doc = "Go back to rough stage whenever a bus reset or bus resume occurs."]
    #[inline(always)]
    pub fn is_use_ifr_trim_fine_on_reset(&self) -> bool {
        *self == ResetResumeRoughEn::UseIfrTrimFineOnReset
    }
}
#[doc = "Field `RESET_RESUME_ROUGH_EN` writer - Reset or Resume to Rough Phase Enable"]
pub type ResetResumeRoughEnW<'a, REG> = crate::BitWriter<'a, REG, ResetResumeRoughEn>;
impl<'a, REG> ResetResumeRoughEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Always works in tracking phase after the first time rough phase, to track transition."]
    #[inline(always)]
    pub fn keep_trim_fine_on_reset(self) -> &'a mut crate::W<REG> {
        self.variant(ResetResumeRoughEn::KeepTrimFineOnReset)
    }
    #[doc = "Go back to rough stage whenever a bus reset or bus resume occurs."]
    #[inline(always)]
    pub fn use_ifr_trim_fine_on_reset(self) -> &'a mut crate::W<REG> {
        self.variant(ResetResumeRoughEn::UseIfrTrimFineOnReset)
    }
}
#[doc = "Crystal-Less USB Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockRecoverEn {
    #[doc = "0: Disable"]
    DisClkRecover = 0,
    #[doc = "1: Enable"]
    EnClkRecover = 1,
}
impl From<ClockRecoverEn> for bool {
    #[inline(always)]
    fn from(variant: ClockRecoverEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CLOCK_RECOVER_EN` reader - Crystal-Less USB Enable"]
pub type ClockRecoverEnR = crate::BitReader<ClockRecoverEn>;
impl ClockRecoverEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> ClockRecoverEn {
        match self.bits {
            false => ClockRecoverEn::DisClkRecover,
            true => ClockRecoverEn::EnClkRecover,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_clk_recover(&self) -> bool {
        *self == ClockRecoverEn::DisClkRecover
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_clk_recover(&self) -> bool {
        *self == ClockRecoverEn::EnClkRecover
    }
}
#[doc = "Field `CLOCK_RECOVER_EN` writer - Crystal-Less USB Enable"]
pub type ClockRecoverEnW<'a, REG> = crate::BitWriter<'a, REG, ClockRecoverEn>;
impl<'a, REG> ClockRecoverEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_clk_recover(self) -> &'a mut crate::W<REG> {
        self.variant(ClockRecoverEn::DisClkRecover)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_clk_recover(self) -> &'a mut crate::W<REG> {
        self.variant(ClockRecoverEn::EnClkRecover)
    }
}
impl R {
    #[doc = "Bit 3 - Selects the source for the initial FIRC trim fine value used after a reset."]
    #[inline(always)]
    pub fn trim_init_val_sel(&self) -> TrimInitValSelR {
        TrimInitValSelR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 5 - Restart from IFR Trim Value"]
    #[inline(always)]
    pub fn restart_ifrtrim_en(&self) -> RestartIfrtrimEnR {
        RestartIfrtrimEnR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Reset or Resume to Rough Phase Enable"]
    #[inline(always)]
    pub fn reset_resume_rough_en(&self) -> ResetResumeRoughEnR {
        ResetResumeRoughEnR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Crystal-Less USB Enable"]
    #[inline(always)]
    pub fn clock_recover_en(&self) -> ClockRecoverEnR {
        ClockRecoverEnR::new(((self.bits >> 7) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 3 - Selects the source for the initial FIRC trim fine value used after a reset."]
    #[inline(always)]
    pub fn trim_init_val_sel(&mut self) -> TrimInitValSelW<ClkRecoverCtrlSpec> {
        TrimInitValSelW::new(self, 3)
    }
    #[doc = "Bit 5 - Restart from IFR Trim Value"]
    #[inline(always)]
    pub fn restart_ifrtrim_en(&mut self) -> RestartIfrtrimEnW<ClkRecoverCtrlSpec> {
        RestartIfrtrimEnW::new(self, 5)
    }
    #[doc = "Bit 6 - Reset or Resume to Rough Phase Enable"]
    #[inline(always)]
    pub fn reset_resume_rough_en(&mut self) -> ResetResumeRoughEnW<ClkRecoverCtrlSpec> {
        ResetResumeRoughEnW::new(self, 6)
    }
    #[doc = "Bit 7 - Crystal-Less USB Enable"]
    #[inline(always)]
    pub fn clock_recover_en(&mut self) -> ClockRecoverEnW<ClkRecoverCtrlSpec> {
        ClockRecoverEnW::new(self, 7)
    }
}
#[doc = "USB Clock Recovery Control\n\nYou can [`read`](crate::Reg::read) this register and get [`clk_recover_ctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`clk_recover_ctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ClkRecoverCtrlSpec;
impl crate::RegisterSpec for ClkRecoverCtrlSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`clk_recover_ctrl::R`](R) reader structure"]
impl crate::Readable for ClkRecoverCtrlSpec {}
#[doc = "`write(|w| ..)` method takes [`clk_recover_ctrl::W`](W) writer structure"]
impl crate::Writable for ClkRecoverCtrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CLK_RECOVER_CTRL to value 0"]
impl crate::Resettable for ClkRecoverCtrlSpec {}

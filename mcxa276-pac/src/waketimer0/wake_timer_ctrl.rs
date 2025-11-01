#[doc = "Register `WAKE_TIMER_CTRL` reader"]
pub type R = crate::R<WakeTimerCtrlSpec>;
#[doc = "Register `WAKE_TIMER_CTRL` writer"]
pub type W = crate::W<WakeTimerCtrlSpec>;
#[doc = "Wake Timer Status Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WakeFlag {
    #[doc = "0: Wake timer has not timed out."]
    Disable = 0,
    #[doc = "1: Wake timer has timed out."]
    Enable = 1,
}
impl From<WakeFlag> for bool {
    #[inline(always)]
    fn from(variant: WakeFlag) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WAKE_FLAG` reader - Wake Timer Status Flag"]
pub type WakeFlagR = crate::BitReader<WakeFlag>;
impl WakeFlagR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> WakeFlag {
        match self.bits {
            false => WakeFlag::Disable,
            true => WakeFlag::Enable,
        }
    }
    #[doc = "Wake timer has not timed out."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == WakeFlag::Disable
    }
    #[doc = "Wake timer has timed out."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == WakeFlag::Enable
    }
}
#[doc = "Field `WAKE_FLAG` writer - Wake Timer Status Flag"]
pub type WakeFlagW<'a, REG> = crate::BitWriter1C<'a, REG, WakeFlag>;
impl<'a, REG> WakeFlagW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Wake timer has not timed out."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(WakeFlag::Disable)
    }
    #[doc = "Wake timer has timed out."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(WakeFlag::Enable)
    }
}
#[doc = "Clear Wake Timer\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClrWakeTimer {
    #[doc = "0: No effect."]
    Disable = 0,
    #[doc = "1: Clears the wake timer counter and halts operation until a new count value is loaded."]
    Enable = 1,
}
impl From<ClrWakeTimer> for bool {
    #[inline(always)]
    fn from(variant: ClrWakeTimer) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CLR_WAKE_TIMER` writer - Clear Wake Timer"]
pub type ClrWakeTimerW<'a, REG> = crate::BitWriter<'a, REG, ClrWakeTimer>;
impl<'a, REG> ClrWakeTimerW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(ClrWakeTimer::Disable)
    }
    #[doc = "Clears the wake timer counter and halts operation until a new count value is loaded."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(ClrWakeTimer::Enable)
    }
}
#[doc = "OSC Divide Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OscDivEna {
    #[doc = "0: Disabled"]
    Disable = 0,
    #[doc = "1: Enabled"]
    Enable = 1,
}
impl From<OscDivEna> for bool {
    #[inline(always)]
    fn from(variant: OscDivEna) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OSC_DIV_ENA` reader - OSC Divide Enable"]
pub type OscDivEnaR = crate::BitReader<OscDivEna>;
impl OscDivEnaR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> OscDivEna {
        match self.bits {
            false => OscDivEna::Disable,
            true => OscDivEna::Enable,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == OscDivEna::Disable
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == OscDivEna::Enable
    }
}
#[doc = "Field `OSC_DIV_ENA` writer - OSC Divide Enable"]
pub type OscDivEnaW<'a, REG> = crate::BitWriter<'a, REG, OscDivEna>;
impl<'a, REG> OscDivEnaW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(OscDivEna::Disable)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(OscDivEna::Enable)
    }
}
#[doc = "Enable Interrupt\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntrEn {
    #[doc = "0: Disabled"]
    Disable = 0,
    #[doc = "1: Enabled"]
    Enable = 1,
}
impl From<IntrEn> for bool {
    #[inline(always)]
    fn from(variant: IntrEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INTR_EN` reader - Enable Interrupt"]
pub type IntrEnR = crate::BitReader<IntrEn>;
impl IntrEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> IntrEn {
        match self.bits {
            false => IntrEn::Disable,
            true => IntrEn::Enable,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == IntrEn::Disable
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == IntrEn::Enable
    }
}
#[doc = "Field `INTR_EN` writer - Enable Interrupt"]
pub type IntrEnW<'a, REG> = crate::BitWriter<'a, REG, IntrEn>;
impl<'a, REG> IntrEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(IntrEn::Disable)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(IntrEn::Enable)
    }
}
impl R {
    #[doc = "Bit 1 - Wake Timer Status Flag"]
    #[inline(always)]
    pub fn wake_flag(&self) -> WakeFlagR {
        WakeFlagR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 4 - OSC Divide Enable"]
    #[inline(always)]
    pub fn osc_div_ena(&self) -> OscDivEnaR {
        OscDivEnaR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Enable Interrupt"]
    #[inline(always)]
    pub fn intr_en(&self) -> IntrEnR {
        IntrEnR::new(((self.bits >> 5) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 1 - Wake Timer Status Flag"]
    #[inline(always)]
    pub fn wake_flag(&mut self) -> WakeFlagW<WakeTimerCtrlSpec> {
        WakeFlagW::new(self, 1)
    }
    #[doc = "Bit 2 - Clear Wake Timer"]
    #[inline(always)]
    pub fn clr_wake_timer(&mut self) -> ClrWakeTimerW<WakeTimerCtrlSpec> {
        ClrWakeTimerW::new(self, 2)
    }
    #[doc = "Bit 4 - OSC Divide Enable"]
    #[inline(always)]
    pub fn osc_div_ena(&mut self) -> OscDivEnaW<WakeTimerCtrlSpec> {
        OscDivEnaW::new(self, 4)
    }
    #[doc = "Bit 5 - Enable Interrupt"]
    #[inline(always)]
    pub fn intr_en(&mut self) -> IntrEnW<WakeTimerCtrlSpec> {
        IntrEnW::new(self, 5)
    }
}
#[doc = "Wake Timer Control\n\nYou can [`read`](crate::Reg::read) this register and get [`wake_timer_ctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`wake_timer_ctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct WakeTimerCtrlSpec;
impl crate::RegisterSpec for WakeTimerCtrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`wake_timer_ctrl::R`](R) reader structure"]
impl crate::Readable for WakeTimerCtrlSpec {}
#[doc = "`write(|w| ..)` method takes [`wake_timer_ctrl::W`](W) writer structure"]
impl crate::Writable for WakeTimerCtrlSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x02;
}
#[doc = "`reset()` method sets WAKE_TIMER_CTRL to value 0"]
impl crate::Resettable for WakeTimerCtrlSpec {}

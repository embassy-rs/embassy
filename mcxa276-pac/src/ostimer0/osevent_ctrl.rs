#[doc = "Register `OSEVENT_CTRL` reader"]
pub type R = crate::R<OseventCtrlSpec>;
#[doc = "Register `OSEVENT_CTRL` writer"]
pub type W = crate::W<OseventCtrlSpec>;
#[doc = "Field `OSTIMER_INTRFLAG` reader - Interrupt Flag"]
pub type OstimerIntrflagR = crate::BitReader;
#[doc = "Field `OSTIMER_INTRFLAG` writer - Interrupt Flag"]
pub type OstimerIntrflagW<'a, REG> = crate::BitWriter1C<'a, REG>;
#[doc = "Interrupt or Wake-Up Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OstimerIntena {
    #[doc = "0: Interrupts blocked"]
    InterruptsBlocked = 0,
    #[doc = "1: Interrupts enabled"]
    InterruptsEnabled = 1,
}
impl From<OstimerIntena> for bool {
    #[inline(always)]
    fn from(variant: OstimerIntena) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OSTIMER_INTENA` reader - Interrupt or Wake-Up Request"]
pub type OstimerIntenaR = crate::BitReader<OstimerIntena>;
impl OstimerIntenaR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> OstimerIntena {
        match self.bits {
            false => OstimerIntena::InterruptsBlocked,
            true => OstimerIntena::InterruptsEnabled,
        }
    }
    #[doc = "Interrupts blocked"]
    #[inline(always)]
    pub fn is_interrupts_blocked(&self) -> bool {
        *self == OstimerIntena::InterruptsBlocked
    }
    #[doc = "Interrupts enabled"]
    #[inline(always)]
    pub fn is_interrupts_enabled(&self) -> bool {
        *self == OstimerIntena::InterruptsEnabled
    }
}
#[doc = "Field `OSTIMER_INTENA` writer - Interrupt or Wake-Up Request"]
pub type OstimerIntenaW<'a, REG> = crate::BitWriter<'a, REG, OstimerIntena>;
impl<'a, REG> OstimerIntenaW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Interrupts blocked"]
    #[inline(always)]
    pub fn interrupts_blocked(self) -> &'a mut crate::W<REG> {
        self.variant(OstimerIntena::InterruptsBlocked)
    }
    #[doc = "Interrupts enabled"]
    #[inline(always)]
    pub fn interrupts_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(OstimerIntena::InterruptsEnabled)
    }
}
#[doc = "Field `MATCH_WR_RDY` reader - EVTimer Match Write Ready"]
pub type MatchWrRdyR = crate::BitReader;
#[doc = "Debug Enable\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DebugEn {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<DebugEn> for bool {
    #[inline(always)]
    fn from(variant: DebugEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DEBUG_EN` reader - Debug Enable"]
pub type DebugEnR = crate::BitReader<DebugEn>;
impl DebugEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> DebugEn {
        match self.bits {
            false => DebugEn::Disable,
            true => DebugEn::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == DebugEn::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == DebugEn::Enable
    }
}
#[doc = "Field `DEBUG_EN` writer - Debug Enable"]
pub type DebugEnW<'a, REG> = crate::BitWriter<'a, REG, DebugEn>;
impl<'a, REG> DebugEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(DebugEn::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(DebugEn::Enable)
    }
}
impl R {
    #[doc = "Bit 0 - Interrupt Flag"]
    #[inline(always)]
    pub fn ostimer_intrflag(&self) -> OstimerIntrflagR {
        OstimerIntrflagR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Interrupt or Wake-Up Request"]
    #[inline(always)]
    pub fn ostimer_intena(&self) -> OstimerIntenaR {
        OstimerIntenaR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - EVTimer Match Write Ready"]
    #[inline(always)]
    pub fn match_wr_rdy(&self) -> MatchWrRdyR {
        MatchWrRdyR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Debug Enable"]
    #[inline(always)]
    pub fn debug_en(&self) -> DebugEnR {
        DebugEnR::new(((self.bits >> 3) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Interrupt Flag"]
    #[inline(always)]
    pub fn ostimer_intrflag(&mut self) -> OstimerIntrflagW<OseventCtrlSpec> {
        OstimerIntrflagW::new(self, 0)
    }
    #[doc = "Bit 1 - Interrupt or Wake-Up Request"]
    #[inline(always)]
    pub fn ostimer_intena(&mut self) -> OstimerIntenaW<OseventCtrlSpec> {
        OstimerIntenaW::new(self, 1)
    }
    #[doc = "Bit 3 - Debug Enable"]
    #[inline(always)]
    pub fn debug_en(&mut self) -> DebugEnW<OseventCtrlSpec> {
        DebugEnW::new(self, 3)
    }
}
#[doc = "OSTIMER Control for CPU\n\nYou can [`read`](crate::Reg::read) this register and get [`osevent_ctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`osevent_ctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct OseventCtrlSpec;
impl crate::RegisterSpec for OseventCtrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`osevent_ctrl::R`](R) reader structure"]
impl crate::Readable for OseventCtrlSpec {}
#[doc = "`write(|w| ..)` method takes [`osevent_ctrl::W`](W) writer structure"]
impl crate::Writable for OseventCtrlSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x01;
}
#[doc = "`reset()` method sets OSEVENT_CTRL to value 0x08"]
impl crate::Resettable for OseventCtrlSpec {
    const RESET_VALUE: u32 = 0x08;
}

#[doc = "Register `CSCLR` writer"]
pub type W = crate::W<CsclrSpec>;
#[doc = "Redundant Signals error/fault Detected\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RedSigsClr {
    #[doc = "0: No effect, ignored"]
    RedSigsNoeffect = 0,
    #[doc = "1: Clears the CSER\\[RED_SIGS\\] bit."]
    RedSigsClear = 1,
}
impl From<RedSigsClr> for bool {
    #[inline(always)]
    fn from(variant: RedSigsClr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RED_SIGS_CLR` writer - Redundant Signals error/fault Detected"]
pub type RedSigsClrW<'a, REG> = crate::BitWriter<'a, REG, RedSigsClr>;
impl<'a, REG> RedSigsClrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect, ignored"]
    #[inline(always)]
    pub fn red_sigs_noeffect(self) -> &'a mut crate::W<REG> {
        self.variant(RedSigsClr::RedSigsNoeffect)
    }
    #[doc = "Clears the CSER\\[RED_SIGS\\] bit."]
    #[inline(always)]
    pub fn red_sigs_clear(self) -> &'a mut crate::W<REG> {
        self.variant(RedSigsClr::RedSigsClear)
    }
}
#[doc = "Read only: Redundant FSM error/fault detected\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RedFsmClr {
    #[doc = "0: No effect, ignored"]
    RedFsmNoeffect = 0,
    #[doc = "1: Clears the CSER\\[RED_FSM\\] bit."]
    RedFsmClear = 1,
}
impl From<RedFsmClr> for bool {
    #[inline(always)]
    fn from(variant: RedFsmClr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RED_FSM_CLR` writer - Read only: Redundant FSM error/fault detected"]
pub type RedFsmClrW<'a, REG> = crate::BitWriter<'a, REG, RedFsmClr>;
impl<'a, REG> RedFsmClrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect, ignored"]
    #[inline(always)]
    pub fn red_fsm_noeffect(self) -> &'a mut crate::W<REG> {
        self.variant(RedFsmClr::RedFsmNoeffect)
    }
    #[doc = "Clears the CSER\\[RED_FSM\\] bit."]
    #[inline(always)]
    pub fn red_fsm_clear(self) -> &'a mut crate::W<REG> {
        self.variant(RedFsmClr::RedFsmClear)
    }
}
#[doc = "Read only: Local-EDC error/fault detected\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalEdcClr {
    #[doc = "0: No effect, ignored"]
    LocalEdcNoeffect = 0,
    #[doc = "1: Clears the CSER\\[LOCAL_EDC\\] bit."]
    LocalEdcClear = 1,
}
impl From<LocalEdcClr> for bool {
    #[inline(always)]
    fn from(variant: LocalEdcClr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LOCAL_EDC_CLR` writer - Read only: Local-EDC error/fault detected"]
pub type LocalEdcClrW<'a, REG> = crate::BitWriter<'a, REG, LocalEdcClr>;
impl<'a, REG> LocalEdcClrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect, ignored"]
    #[inline(always)]
    pub fn local_edc_noeffect(self) -> &'a mut crate::W<REG> {
        self.variant(LocalEdcClr::LocalEdcNoeffect)
    }
    #[doc = "Clears the CSER\\[LOCAL_EDC\\] bit."]
    #[inline(always)]
    pub fn local_edc_clear(self) -> &'a mut crate::W<REG> {
        self.variant(LocalEdcClr::LocalEdcClear)
    }
}
#[doc = "Read only: Bus-EDC error/fault detected\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BusEdcClr {
    #[doc = "0: No effect, ignored"]
    BusEdcNoeffect = 0,
    #[doc = "1: Clears the CSER\\[BUS_EDC\\] bit."]
    BusEdcClear = 1,
}
impl From<BusEdcClr> for bool {
    #[inline(always)]
    fn from(variant: BusEdcClr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BUS_EDC_CLR` writer - Read only: Bus-EDC error/fault detected"]
pub type BusEdcClrW<'a, REG> = crate::BitWriter<'a, REG, BusEdcClr>;
impl<'a, REG> BusEdcClrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect, ignored"]
    #[inline(always)]
    pub fn bus_edc_noeffect(self) -> &'a mut crate::W<REG> {
        self.variant(BusEdcClr::BusEdcNoeffect)
    }
    #[doc = "Clears the CSER\\[BUS_EDC\\] bit."]
    #[inline(always)]
    pub fn bus_edc_clear(self) -> &'a mut crate::W<REG> {
        self.variant(BusEdcClr::BusEdcClear)
    }
}
impl W {
    #[doc = "Bit 0 - Redundant Signals error/fault Detected"]
    #[inline(always)]
    pub fn red_sigs_clr(&mut self) -> RedSigsClrW<CsclrSpec> {
        RedSigsClrW::new(self, 0)
    }
    #[doc = "Bit 1 - Read only: Redundant FSM error/fault detected"]
    #[inline(always)]
    pub fn red_fsm_clr(&mut self) -> RedFsmClrW<CsclrSpec> {
        RedFsmClrW::new(self, 1)
    }
    #[doc = "Bit 2 - Read only: Local-EDC error/fault detected"]
    #[inline(always)]
    pub fn local_edc_clr(&mut self) -> LocalEdcClrW<CsclrSpec> {
        LocalEdcClrW::new(self, 2)
    }
    #[doc = "Bit 3 - Read only: Bus-EDC error/fault detected"]
    #[inline(always)]
    pub fn bus_edc_clr(&mut self) -> BusEdcClrW<CsclrSpec> {
        BusEdcClrW::new(self, 3)
    }
}
#[doc = "Common Security Clear Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`csclr::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CsclrSpec;
impl crate::RegisterSpec for CsclrSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`csclr::W`](W) writer structure"]
impl crate::Writable for CsclrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CSCLR to value 0"]
impl crate::Resettable for CsclrSpec {}

#[doc = "Register `CTCR` reader"]
pub type R = crate::R<CtcrSpec>;
#[doc = "Register `CTCR` writer"]
pub type W = crate::W<CtcrSpec>;
#[doc = "Counter Timer Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Ctmode {
    #[doc = "0: Timer mode"]
    Timer = 0,
    #[doc = "1: Counter mode rising edge"]
    CounterRisingEdge = 1,
    #[doc = "2: Counter mode falling edge"]
    CounterFallingEdge = 2,
    #[doc = "3: Counter mode dual edge"]
    CounterDualEdge = 3,
}
impl From<Ctmode> for u8 {
    #[inline(always)]
    fn from(variant: Ctmode) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Ctmode {
    type Ux = u8;
}
impl crate::IsEnum for Ctmode {}
#[doc = "Field `CTMODE` reader - Counter Timer Mode"]
pub type CtmodeR = crate::FieldReader<Ctmode>;
impl CtmodeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ctmode {
        match self.bits {
            0 => Ctmode::Timer,
            1 => Ctmode::CounterRisingEdge,
            2 => Ctmode::CounterFallingEdge,
            3 => Ctmode::CounterDualEdge,
            _ => unreachable!(),
        }
    }
    #[doc = "Timer mode"]
    #[inline(always)]
    pub fn is_timer(&self) -> bool {
        *self == Ctmode::Timer
    }
    #[doc = "Counter mode rising edge"]
    #[inline(always)]
    pub fn is_counter_rising_edge(&self) -> bool {
        *self == Ctmode::CounterRisingEdge
    }
    #[doc = "Counter mode falling edge"]
    #[inline(always)]
    pub fn is_counter_falling_edge(&self) -> bool {
        *self == Ctmode::CounterFallingEdge
    }
    #[doc = "Counter mode dual edge"]
    #[inline(always)]
    pub fn is_counter_dual_edge(&self) -> bool {
        *self == Ctmode::CounterDualEdge
    }
}
#[doc = "Field `CTMODE` writer - Counter Timer Mode"]
pub type CtmodeW<'a, REG> = crate::FieldWriter<'a, REG, 2, Ctmode, crate::Safe>;
impl<'a, REG> CtmodeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Timer mode"]
    #[inline(always)]
    pub fn timer(self) -> &'a mut crate::W<REG> {
        self.variant(Ctmode::Timer)
    }
    #[doc = "Counter mode rising edge"]
    #[inline(always)]
    pub fn counter_rising_edge(self) -> &'a mut crate::W<REG> {
        self.variant(Ctmode::CounterRisingEdge)
    }
    #[doc = "Counter mode falling edge"]
    #[inline(always)]
    pub fn counter_falling_edge(self) -> &'a mut crate::W<REG> {
        self.variant(Ctmode::CounterFallingEdge)
    }
    #[doc = "Counter mode dual edge"]
    #[inline(always)]
    pub fn counter_dual_edge(self) -> &'a mut crate::W<REG> {
        self.variant(Ctmode::CounterDualEdge)
    }
}
#[doc = "Count Input Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Cinsel {
    #[doc = "0: Channel 0, CAPn\\[0\\] for CTIMERn"]
    Channel0 = 0,
    #[doc = "1: Channel 1, CAPn\\[1\\] for CTIMERn"]
    Channel1 = 1,
    #[doc = "2: Channel 2, CAPn\\[2\\] for CTIMERn"]
    Channel2 = 2,
    #[doc = "3: Channel 3, CAPn\\[3\\] for CTIMERn"]
    Channel3 = 3,
}
impl From<Cinsel> for u8 {
    #[inline(always)]
    fn from(variant: Cinsel) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Cinsel {
    type Ux = u8;
}
impl crate::IsEnum for Cinsel {}
#[doc = "Field `CINSEL` reader - Count Input Select"]
pub type CinselR = crate::FieldReader<Cinsel>;
impl CinselR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cinsel {
        match self.bits {
            0 => Cinsel::Channel0,
            1 => Cinsel::Channel1,
            2 => Cinsel::Channel2,
            3 => Cinsel::Channel3,
            _ => unreachable!(),
        }
    }
    #[doc = "Channel 0, CAPn\\[0\\] for CTIMERn"]
    #[inline(always)]
    pub fn is_channel_0(&self) -> bool {
        *self == Cinsel::Channel0
    }
    #[doc = "Channel 1, CAPn\\[1\\] for CTIMERn"]
    #[inline(always)]
    pub fn is_channel_1(&self) -> bool {
        *self == Cinsel::Channel1
    }
    #[doc = "Channel 2, CAPn\\[2\\] for CTIMERn"]
    #[inline(always)]
    pub fn is_channel_2(&self) -> bool {
        *self == Cinsel::Channel2
    }
    #[doc = "Channel 3, CAPn\\[3\\] for CTIMERn"]
    #[inline(always)]
    pub fn is_channel_3(&self) -> bool {
        *self == Cinsel::Channel3
    }
}
#[doc = "Field `CINSEL` writer - Count Input Select"]
pub type CinselW<'a, REG> = crate::FieldWriter<'a, REG, 2, Cinsel, crate::Safe>;
impl<'a, REG> CinselW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Channel 0, CAPn\\[0\\] for CTIMERn"]
    #[inline(always)]
    pub fn channel_0(self) -> &'a mut crate::W<REG> {
        self.variant(Cinsel::Channel0)
    }
    #[doc = "Channel 1, CAPn\\[1\\] for CTIMERn"]
    #[inline(always)]
    pub fn channel_1(self) -> &'a mut crate::W<REG> {
        self.variant(Cinsel::Channel1)
    }
    #[doc = "Channel 2, CAPn\\[2\\] for CTIMERn"]
    #[inline(always)]
    pub fn channel_2(self) -> &'a mut crate::W<REG> {
        self.variant(Cinsel::Channel2)
    }
    #[doc = "Channel 3, CAPn\\[3\\] for CTIMERn"]
    #[inline(always)]
    pub fn channel_3(self) -> &'a mut crate::W<REG> {
        self.variant(Cinsel::Channel3)
    }
}
#[doc = "Field `ENCC` reader - Capture Channel Enable"]
pub type EnccR = crate::BitReader;
#[doc = "Field `ENCC` writer - Capture Channel Enable"]
pub type EnccW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Edge Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Selcc {
    #[doc = "0: Capture channel 0 rising edge"]
    Channel0Rising = 0,
    #[doc = "1: Capture channel 0 falling edge"]
    Channel0Falling = 1,
    #[doc = "2: Capture channel 1 rising edge"]
    Channel1Rising = 2,
    #[doc = "3: Capture channel 1 falling edge"]
    Channel1Falling = 3,
    #[doc = "4: Capture channel 2 rising edge"]
    Channel2Rising = 4,
    #[doc = "5: Capture channel 2 falling edge"]
    Channel2Falling = 5,
    #[doc = "6: Capture channel 3 rising edge"]
    Channel3Rising = 6,
    #[doc = "7: Capture channel 3 falling edge"]
    Channel3Falling = 7,
}
impl From<Selcc> for u8 {
    #[inline(always)]
    fn from(variant: Selcc) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Selcc {
    type Ux = u8;
}
impl crate::IsEnum for Selcc {}
#[doc = "Field `SELCC` reader - Edge Select"]
pub type SelccR = crate::FieldReader<Selcc>;
impl SelccR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Selcc {
        match self.bits {
            0 => Selcc::Channel0Rising,
            1 => Selcc::Channel0Falling,
            2 => Selcc::Channel1Rising,
            3 => Selcc::Channel1Falling,
            4 => Selcc::Channel2Rising,
            5 => Selcc::Channel2Falling,
            6 => Selcc::Channel3Rising,
            7 => Selcc::Channel3Falling,
            _ => unreachable!(),
        }
    }
    #[doc = "Capture channel 0 rising edge"]
    #[inline(always)]
    pub fn is_channel_0_rising(&self) -> bool {
        *self == Selcc::Channel0Rising
    }
    #[doc = "Capture channel 0 falling edge"]
    #[inline(always)]
    pub fn is_channel_0_falling(&self) -> bool {
        *self == Selcc::Channel0Falling
    }
    #[doc = "Capture channel 1 rising edge"]
    #[inline(always)]
    pub fn is_channel_1_rising(&self) -> bool {
        *self == Selcc::Channel1Rising
    }
    #[doc = "Capture channel 1 falling edge"]
    #[inline(always)]
    pub fn is_channel_1_falling(&self) -> bool {
        *self == Selcc::Channel1Falling
    }
    #[doc = "Capture channel 2 rising edge"]
    #[inline(always)]
    pub fn is_channel_2_rising(&self) -> bool {
        *self == Selcc::Channel2Rising
    }
    #[doc = "Capture channel 2 falling edge"]
    #[inline(always)]
    pub fn is_channel_2_falling(&self) -> bool {
        *self == Selcc::Channel2Falling
    }
    #[doc = "Capture channel 3 rising edge"]
    #[inline(always)]
    pub fn is_channel_3_rising(&self) -> bool {
        *self == Selcc::Channel3Rising
    }
    #[doc = "Capture channel 3 falling edge"]
    #[inline(always)]
    pub fn is_channel_3_falling(&self) -> bool {
        *self == Selcc::Channel3Falling
    }
}
#[doc = "Field `SELCC` writer - Edge Select"]
pub type SelccW<'a, REG> = crate::FieldWriter<'a, REG, 3, Selcc, crate::Safe>;
impl<'a, REG> SelccW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Capture channel 0 rising edge"]
    #[inline(always)]
    pub fn channel_0_rising(self) -> &'a mut crate::W<REG> {
        self.variant(Selcc::Channel0Rising)
    }
    #[doc = "Capture channel 0 falling edge"]
    #[inline(always)]
    pub fn channel_0_falling(self) -> &'a mut crate::W<REG> {
        self.variant(Selcc::Channel0Falling)
    }
    #[doc = "Capture channel 1 rising edge"]
    #[inline(always)]
    pub fn channel_1_rising(self) -> &'a mut crate::W<REG> {
        self.variant(Selcc::Channel1Rising)
    }
    #[doc = "Capture channel 1 falling edge"]
    #[inline(always)]
    pub fn channel_1_falling(self) -> &'a mut crate::W<REG> {
        self.variant(Selcc::Channel1Falling)
    }
    #[doc = "Capture channel 2 rising edge"]
    #[inline(always)]
    pub fn channel_2_rising(self) -> &'a mut crate::W<REG> {
        self.variant(Selcc::Channel2Rising)
    }
    #[doc = "Capture channel 2 falling edge"]
    #[inline(always)]
    pub fn channel_2_falling(self) -> &'a mut crate::W<REG> {
        self.variant(Selcc::Channel2Falling)
    }
    #[doc = "Capture channel 3 rising edge"]
    #[inline(always)]
    pub fn channel_3_rising(self) -> &'a mut crate::W<REG> {
        self.variant(Selcc::Channel3Rising)
    }
    #[doc = "Capture channel 3 falling edge"]
    #[inline(always)]
    pub fn channel_3_falling(self) -> &'a mut crate::W<REG> {
        self.variant(Selcc::Channel3Falling)
    }
}
impl R {
    #[doc = "Bits 0:1 - Counter Timer Mode"]
    #[inline(always)]
    pub fn ctmode(&self) -> CtmodeR {
        CtmodeR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 2:3 - Count Input Select"]
    #[inline(always)]
    pub fn cinsel(&self) -> CinselR {
        CinselR::new(((self.bits >> 2) & 3) as u8)
    }
    #[doc = "Bit 4 - Capture Channel Enable"]
    #[inline(always)]
    pub fn encc(&self) -> EnccR {
        EnccR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bits 5:7 - Edge Select"]
    #[inline(always)]
    pub fn selcc(&self) -> SelccR {
        SelccR::new(((self.bits >> 5) & 7) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - Counter Timer Mode"]
    #[inline(always)]
    pub fn ctmode(&mut self) -> CtmodeW<CtcrSpec> {
        CtmodeW::new(self, 0)
    }
    #[doc = "Bits 2:3 - Count Input Select"]
    #[inline(always)]
    pub fn cinsel(&mut self) -> CinselW<CtcrSpec> {
        CinselW::new(self, 2)
    }
    #[doc = "Bit 4 - Capture Channel Enable"]
    #[inline(always)]
    pub fn encc(&mut self) -> EnccW<CtcrSpec> {
        EnccW::new(self, 4)
    }
    #[doc = "Bits 5:7 - Edge Select"]
    #[inline(always)]
    pub fn selcc(&mut self) -> SelccW<CtcrSpec> {
        SelccW::new(self, 5)
    }
}
#[doc = "Count Control\n\nYou can [`read`](crate::Reg::read) this register and get [`ctcr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctcr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CtcrSpec;
impl crate::RegisterSpec for CtcrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ctcr::R`](R) reader structure"]
impl crate::Readable for CtcrSpec {}
#[doc = "`write(|w| ..)` method takes [`ctcr::W`](W) writer structure"]
impl crate::Writable for CtcrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CTCR to value 0"]
impl crate::Resettable for CtcrSpec {}

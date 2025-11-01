#[doc = "Register `SCR` reader"]
pub type R = crate::R<ScrSpec>;
#[doc = "Register `SCR` writer"]
pub type W = crate::W<ScrSpec>;
#[doc = "Target Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sen {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Sen> for bool {
    #[inline(always)]
    fn from(variant: Sen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SEN` reader - Target Enable"]
pub type SenR = crate::BitReader<Sen>;
impl SenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sen {
        match self.bits {
            false => Sen::Disabled,
            true => Sen::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Sen::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Sen::Enabled
    }
}
#[doc = "Field `SEN` writer - Target Enable"]
pub type SenW<'a, REG> = crate::BitWriter<'a, REG, Sen>;
impl<'a, REG> SenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Sen::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Sen::Enabled)
    }
}
#[doc = "Software Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rst {
    #[doc = "0: Not reset"]
    NotReset = 0,
    #[doc = "1: Reset"]
    Reset = 1,
}
impl From<Rst> for bool {
    #[inline(always)]
    fn from(variant: Rst) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RST` reader - Software Reset"]
pub type RstR = crate::BitReader<Rst>;
impl RstR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rst {
        match self.bits {
            false => Rst::NotReset,
            true => Rst::Reset,
        }
    }
    #[doc = "Not reset"]
    #[inline(always)]
    pub fn is_not_reset(&self) -> bool {
        *self == Rst::NotReset
    }
    #[doc = "Reset"]
    #[inline(always)]
    pub fn is_reset(&self) -> bool {
        *self == Rst::Reset
    }
}
#[doc = "Field `RST` writer - Software Reset"]
pub type RstW<'a, REG> = crate::BitWriter<'a, REG, Rst>;
impl<'a, REG> RstW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not reset"]
    #[inline(always)]
    pub fn not_reset(self) -> &'a mut crate::W<REG> {
        self.variant(Rst::NotReset)
    }
    #[doc = "Reset"]
    #[inline(always)]
    pub fn reset(self) -> &'a mut crate::W<REG> {
        self.variant(Rst::Reset)
    }
}
#[doc = "Filter Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filten {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Filten> for bool {
    #[inline(always)]
    fn from(variant: Filten) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FILTEN` reader - Filter Enable"]
pub type FiltenR = crate::BitReader<Filten>;
impl FiltenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Filten {
        match self.bits {
            false => Filten::Disable,
            true => Filten::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Filten::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Filten::Enable
    }
}
#[doc = "Field `FILTEN` writer - Filter Enable"]
pub type FiltenW<'a, REG> = crate::BitWriter<'a, REG, Filten>;
impl<'a, REG> FiltenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Filten::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Filten::Enable)
    }
}
#[doc = "Filter Doze Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filtdz {
    #[doc = "0: Enable"]
    FilterEnabled = 0,
    #[doc = "1: Disable"]
    FilterDisabled = 1,
}
impl From<Filtdz> for bool {
    #[inline(always)]
    fn from(variant: Filtdz) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FILTDZ` reader - Filter Doze Enable"]
pub type FiltdzR = crate::BitReader<Filtdz>;
impl FiltdzR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Filtdz {
        match self.bits {
            false => Filtdz::FilterEnabled,
            true => Filtdz::FilterDisabled,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_filter_enabled(&self) -> bool {
        *self == Filtdz::FilterEnabled
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_filter_disabled(&self) -> bool {
        *self == Filtdz::FilterDisabled
    }
}
#[doc = "Field `FILTDZ` writer - Filter Doze Enable"]
pub type FiltdzW<'a, REG> = crate::BitWriter<'a, REG, Filtdz>;
impl<'a, REG> FiltdzW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn filter_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Filtdz::FilterEnabled)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn filter_disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Filtdz::FilterDisabled)
    }
}
#[doc = "Reset Transmit FIFO\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rtf {
    #[doc = "0: No effect"]
    NoEffect = 0,
    #[doc = "1: STDR is now empty"]
    NowEmpty = 1,
}
impl From<Rtf> for bool {
    #[inline(always)]
    fn from(variant: Rtf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RTF` reader - Reset Transmit FIFO"]
pub type RtfR = crate::BitReader<Rtf>;
impl RtfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rtf {
        match self.bits {
            false => Rtf::NoEffect,
            true => Rtf::NowEmpty,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_no_effect(&self) -> bool {
        *self == Rtf::NoEffect
    }
    #[doc = "STDR is now empty"]
    #[inline(always)]
    pub fn is_now_empty(&self) -> bool {
        *self == Rtf::NowEmpty
    }
}
#[doc = "Field `RTF` writer - Reset Transmit FIFO"]
pub type RtfW<'a, REG> = crate::BitWriter<'a, REG, Rtf>;
impl<'a, REG> RtfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn no_effect(self) -> &'a mut crate::W<REG> {
        self.variant(Rtf::NoEffect)
    }
    #[doc = "STDR is now empty"]
    #[inline(always)]
    pub fn now_empty(self) -> &'a mut crate::W<REG> {
        self.variant(Rtf::NowEmpty)
    }
}
#[doc = "Reset Receive FIFO\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rrf {
    #[doc = "0: No effect"]
    NoEffect = 0,
    #[doc = "1: SRDR is now empty"]
    NowEmpty = 1,
}
impl From<Rrf> for bool {
    #[inline(always)]
    fn from(variant: Rrf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RRF` reader - Reset Receive FIFO"]
pub type RrfR = crate::BitReader<Rrf>;
impl RrfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rrf {
        match self.bits {
            false => Rrf::NoEffect,
            true => Rrf::NowEmpty,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_no_effect(&self) -> bool {
        *self == Rrf::NoEffect
    }
    #[doc = "SRDR is now empty"]
    #[inline(always)]
    pub fn is_now_empty(&self) -> bool {
        *self == Rrf::NowEmpty
    }
}
#[doc = "Field `RRF` writer - Reset Receive FIFO"]
pub type RrfW<'a, REG> = crate::BitWriter<'a, REG, Rrf>;
impl<'a, REG> RrfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn no_effect(self) -> &'a mut crate::W<REG> {
        self.variant(Rrf::NoEffect)
    }
    #[doc = "SRDR is now empty"]
    #[inline(always)]
    pub fn now_empty(self) -> &'a mut crate::W<REG> {
        self.variant(Rrf::NowEmpty)
    }
}
impl R {
    #[doc = "Bit 0 - Target Enable"]
    #[inline(always)]
    pub fn sen(&self) -> SenR {
        SenR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Software Reset"]
    #[inline(always)]
    pub fn rst(&self) -> RstR {
        RstR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 4 - Filter Enable"]
    #[inline(always)]
    pub fn filten(&self) -> FiltenR {
        FiltenR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Filter Doze Enable"]
    #[inline(always)]
    pub fn filtdz(&self) -> FiltdzR {
        FiltdzR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 8 - Reset Transmit FIFO"]
    #[inline(always)]
    pub fn rtf(&self) -> RtfR {
        RtfR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Reset Receive FIFO"]
    #[inline(always)]
    pub fn rrf(&self) -> RrfR {
        RrfR::new(((self.bits >> 9) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Target Enable"]
    #[inline(always)]
    pub fn sen(&mut self) -> SenW<ScrSpec> {
        SenW::new(self, 0)
    }
    #[doc = "Bit 1 - Software Reset"]
    #[inline(always)]
    pub fn rst(&mut self) -> RstW<ScrSpec> {
        RstW::new(self, 1)
    }
    #[doc = "Bit 4 - Filter Enable"]
    #[inline(always)]
    pub fn filten(&mut self) -> FiltenW<ScrSpec> {
        FiltenW::new(self, 4)
    }
    #[doc = "Bit 5 - Filter Doze Enable"]
    #[inline(always)]
    pub fn filtdz(&mut self) -> FiltdzW<ScrSpec> {
        FiltdzW::new(self, 5)
    }
    #[doc = "Bit 8 - Reset Transmit FIFO"]
    #[inline(always)]
    pub fn rtf(&mut self) -> RtfW<ScrSpec> {
        RtfW::new(self, 8)
    }
    #[doc = "Bit 9 - Reset Receive FIFO"]
    #[inline(always)]
    pub fn rrf(&mut self) -> RrfW<ScrSpec> {
        RrfW::new(self, 9)
    }
}
#[doc = "Target Control\n\nYou can [`read`](crate::Reg::read) this register and get [`scr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`scr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ScrSpec;
impl crate::RegisterSpec for ScrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`scr::R`](R) reader structure"]
impl crate::Readable for ScrSpec {}
#[doc = "`write(|w| ..)` method takes [`scr::W`](W) writer structure"]
impl crate::Writable for ScrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SCR to value 0"]
impl crate::Resettable for ScrSpec {}

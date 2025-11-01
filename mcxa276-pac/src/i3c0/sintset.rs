#[doc = "Register `SINTSET` reader"]
pub type R = crate::R<SintsetSpec>;
#[doc = "Register `SINTSET` writer"]
pub type W = crate::W<SintsetSpec>;
#[doc = "Start Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Start {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Start> for bool {
    #[inline(always)]
    fn from(variant: Start) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `START` reader - Start Interrupt Enable"]
pub type StartR = crate::BitReader<Start>;
impl StartR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Start {
        match self.bits {
            false => Start::Disable,
            true => Start::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Start::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Start::Enable
    }
}
#[doc = "Field `START` writer - Start Interrupt Enable"]
pub type StartW<'a, REG> = crate::BitWriter1S<'a, REG, Start>;
impl<'a, REG> StartW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Start::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Start::Enable)
    }
}
#[doc = "Match Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Matched {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Matched> for bool {
    #[inline(always)]
    fn from(variant: Matched) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MATCHED` reader - Match Interrupt Enable"]
pub type MatchedR = crate::BitReader<Matched>;
impl MatchedR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Matched {
        match self.bits {
            false => Matched::Disable,
            true => Matched::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Matched::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Matched::Enable
    }
}
#[doc = "Field `MATCHED` writer - Match Interrupt Enable"]
pub type MatchedW<'a, REG> = crate::BitWriter1S<'a, REG, Matched>;
impl<'a, REG> MatchedW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Matched::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Matched::Enable)
    }
}
#[doc = "Stop Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stop {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Stop> for bool {
    #[inline(always)]
    fn from(variant: Stop) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STOP` reader - Stop Interrupt Enable"]
pub type StopR = crate::BitReader<Stop>;
impl StopR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Stop {
        match self.bits {
            false => Stop::Disable,
            true => Stop::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Stop::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Stop::Enable
    }
}
#[doc = "Field `STOP` writer - Stop Interrupt Enable"]
pub type StopW<'a, REG> = crate::BitWriter1S<'a, REG, Stop>;
impl<'a, REG> StopW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Stop::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Stop::Enable)
    }
}
#[doc = "Receive Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rxpend {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Rxpend> for bool {
    #[inline(always)]
    fn from(variant: Rxpend) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RXPEND` reader - Receive Interrupt Enable"]
pub type RxpendR = crate::BitReader<Rxpend>;
impl RxpendR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rxpend {
        match self.bits {
            false => Rxpend::Disable,
            true => Rxpend::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Rxpend::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Rxpend::Enable
    }
}
#[doc = "Field `RXPEND` writer - Receive Interrupt Enable"]
pub type RxpendW<'a, REG> = crate::BitWriter1S<'a, REG, Rxpend>;
impl<'a, REG> RxpendW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Rxpend::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Rxpend::Enable)
    }
}
#[doc = "Transmit Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Txsend {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Txsend> for bool {
    #[inline(always)]
    fn from(variant: Txsend) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TXSEND` reader - Transmit Interrupt Enable"]
pub type TxsendR = crate::BitReader<Txsend>;
impl TxsendR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Txsend {
        match self.bits {
            false => Txsend::Disable,
            true => Txsend::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Txsend::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Txsend::Enable
    }
}
#[doc = "Field `TXSEND` writer - Transmit Interrupt Enable"]
pub type TxsendW<'a, REG> = crate::BitWriter1S<'a, REG, Txsend>;
impl<'a, REG> TxsendW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Txsend::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Txsend::Enable)
    }
}
#[doc = "Dynamic Address Change Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dachg {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Dachg> for bool {
    #[inline(always)]
    fn from(variant: Dachg) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DACHG` reader - Dynamic Address Change Interrupt Enable"]
pub type DachgR = crate::BitReader<Dachg>;
impl DachgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dachg {
        match self.bits {
            false => Dachg::Disable,
            true => Dachg::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Dachg::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Dachg::Enable
    }
}
#[doc = "Field `DACHG` writer - Dynamic Address Change Interrupt Enable"]
pub type DachgW<'a, REG> = crate::BitWriter1S<'a, REG, Dachg>;
impl<'a, REG> DachgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Dachg::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Dachg::Enable)
    }
}
#[doc = "Common Command Code (CCC) Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ccc {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Ccc> for bool {
    #[inline(always)]
    fn from(variant: Ccc) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CCC` reader - Common Command Code (CCC) Interrupt Enable"]
pub type CccR = crate::BitReader<Ccc>;
impl CccR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ccc {
        match self.bits {
            false => Ccc::Disable,
            true => Ccc::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Ccc::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Ccc::Enable
    }
}
#[doc = "Field `CCC` writer - Common Command Code (CCC) Interrupt Enable"]
pub type CccW<'a, REG> = crate::BitWriter1S<'a, REG, Ccc>;
impl<'a, REG> CccW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Ccc::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Ccc::Enable)
    }
}
#[doc = "Error or Warning Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Errwarn {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Errwarn> for bool {
    #[inline(always)]
    fn from(variant: Errwarn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERRWARN` reader - Error or Warning Interrupt Enable"]
pub type ErrwarnR = crate::BitReader<Errwarn>;
impl ErrwarnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Errwarn {
        match self.bits {
            false => Errwarn::Disable,
            true => Errwarn::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Errwarn::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Errwarn::Enable
    }
}
#[doc = "Field `ERRWARN` writer - Error or Warning Interrupt Enable"]
pub type ErrwarnW<'a, REG> = crate::BitWriter1S<'a, REG, Errwarn>;
impl<'a, REG> ErrwarnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Errwarn::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Errwarn::Enable)
    }
}
#[doc = "Double Data Rate Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ddrmatched {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Ddrmatched> for bool {
    #[inline(always)]
    fn from(variant: Ddrmatched) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DDRMATCHED` reader - Double Data Rate Interrupt Enable"]
pub type DdrmatchedR = crate::BitReader<Ddrmatched>;
impl DdrmatchedR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ddrmatched {
        match self.bits {
            false => Ddrmatched::Disable,
            true => Ddrmatched::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Ddrmatched::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Ddrmatched::Enable
    }
}
#[doc = "Field `DDRMATCHED` writer - Double Data Rate Interrupt Enable"]
pub type DdrmatchedW<'a, REG> = crate::BitWriter1S<'a, REG, Ddrmatched>;
impl<'a, REG> DdrmatchedW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Ddrmatched::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Ddrmatched::Enable)
    }
}
#[doc = "Common Command Code (CCC) Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chandled {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Chandled> for bool {
    #[inline(always)]
    fn from(variant: Chandled) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CHANDLED` reader - Common Command Code (CCC) Interrupt Enable"]
pub type ChandledR = crate::BitReader<Chandled>;
impl ChandledR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Chandled {
        match self.bits {
            false => Chandled::Disable,
            true => Chandled::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Chandled::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Chandled::Enable
    }
}
#[doc = "Field `CHANDLED` writer - Common Command Code (CCC) Interrupt Enable"]
pub type ChandledW<'a, REG> = crate::BitWriter1S<'a, REG, Chandled>;
impl<'a, REG> ChandledW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Chandled::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Chandled::Enable)
    }
}
#[doc = "Event Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Event> for bool {
    #[inline(always)]
    fn from(variant: Event) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EVENT` reader - Event Interrupt Enable"]
pub type EventR = crate::BitReader<Event>;
impl EventR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Event {
        match self.bits {
            false => Event::Disable,
            true => Event::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Event::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Event::Enable
    }
}
#[doc = "Field `EVENT` writer - Event Interrupt Enable"]
pub type EventW<'a, REG> = crate::BitWriter1S<'a, REG, Event>;
impl<'a, REG> EventW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Event::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Event::Enable)
    }
}
impl R {
    #[doc = "Bit 8 - Start Interrupt Enable"]
    #[inline(always)]
    pub fn start(&self) -> StartR {
        StartR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Match Interrupt Enable"]
    #[inline(always)]
    pub fn matched(&self) -> MatchedR {
        MatchedR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Stop Interrupt Enable"]
    #[inline(always)]
    pub fn stop(&self) -> StopR {
        StopR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Receive Interrupt Enable"]
    #[inline(always)]
    pub fn rxpend(&self) -> RxpendR {
        RxpendR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Transmit Interrupt Enable"]
    #[inline(always)]
    pub fn txsend(&self) -> TxsendR {
        TxsendR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Dynamic Address Change Interrupt Enable"]
    #[inline(always)]
    pub fn dachg(&self) -> DachgR {
        DachgR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Common Command Code (CCC) Interrupt Enable"]
    #[inline(always)]
    pub fn ccc(&self) -> CccR {
        CccR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Error or Warning Interrupt Enable"]
    #[inline(always)]
    pub fn errwarn(&self) -> ErrwarnR {
        ErrwarnR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - Double Data Rate Interrupt Enable"]
    #[inline(always)]
    pub fn ddrmatched(&self) -> DdrmatchedR {
        DdrmatchedR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Common Command Code (CCC) Interrupt Enable"]
    #[inline(always)]
    pub fn chandled(&self) -> ChandledR {
        ChandledR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Event Interrupt Enable"]
    #[inline(always)]
    pub fn event(&self) -> EventR {
        EventR::new(((self.bits >> 18) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 8 - Start Interrupt Enable"]
    #[inline(always)]
    pub fn start(&mut self) -> StartW<SintsetSpec> {
        StartW::new(self, 8)
    }
    #[doc = "Bit 9 - Match Interrupt Enable"]
    #[inline(always)]
    pub fn matched(&mut self) -> MatchedW<SintsetSpec> {
        MatchedW::new(self, 9)
    }
    #[doc = "Bit 10 - Stop Interrupt Enable"]
    #[inline(always)]
    pub fn stop(&mut self) -> StopW<SintsetSpec> {
        StopW::new(self, 10)
    }
    #[doc = "Bit 11 - Receive Interrupt Enable"]
    #[inline(always)]
    pub fn rxpend(&mut self) -> RxpendW<SintsetSpec> {
        RxpendW::new(self, 11)
    }
    #[doc = "Bit 12 - Transmit Interrupt Enable"]
    #[inline(always)]
    pub fn txsend(&mut self) -> TxsendW<SintsetSpec> {
        TxsendW::new(self, 12)
    }
    #[doc = "Bit 13 - Dynamic Address Change Interrupt Enable"]
    #[inline(always)]
    pub fn dachg(&mut self) -> DachgW<SintsetSpec> {
        DachgW::new(self, 13)
    }
    #[doc = "Bit 14 - Common Command Code (CCC) Interrupt Enable"]
    #[inline(always)]
    pub fn ccc(&mut self) -> CccW<SintsetSpec> {
        CccW::new(self, 14)
    }
    #[doc = "Bit 15 - Error or Warning Interrupt Enable"]
    #[inline(always)]
    pub fn errwarn(&mut self) -> ErrwarnW<SintsetSpec> {
        ErrwarnW::new(self, 15)
    }
    #[doc = "Bit 16 - Double Data Rate Interrupt Enable"]
    #[inline(always)]
    pub fn ddrmatched(&mut self) -> DdrmatchedW<SintsetSpec> {
        DdrmatchedW::new(self, 16)
    }
    #[doc = "Bit 17 - Common Command Code (CCC) Interrupt Enable"]
    #[inline(always)]
    pub fn chandled(&mut self) -> ChandledW<SintsetSpec> {
        ChandledW::new(self, 17)
    }
    #[doc = "Bit 18 - Event Interrupt Enable"]
    #[inline(always)]
    pub fn event(&mut self) -> EventW<SintsetSpec> {
        EventW::new(self, 18)
    }
}
#[doc = "Target Interrupt Set\n\nYou can [`read`](crate::Reg::read) this register and get [`sintset::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sintset::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SintsetSpec;
impl crate::RegisterSpec for SintsetSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sintset::R`](R) reader structure"]
impl crate::Readable for SintsetSpec {}
#[doc = "`write(|w| ..)` method takes [`sintset::W`](W) writer structure"]
impl crate::Writable for SintsetSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x0007_ff00;
}
#[doc = "`reset()` method sets SINTSET to value 0"]
impl crate::Resettable for SintsetSpec {}

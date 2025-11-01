#[doc = "Register `MDATACTRL` reader"]
pub type R = crate::R<MdatactrlSpec>;
#[doc = "Register `MDATACTRL` writer"]
pub type W = crate::W<MdatactrlSpec>;
#[doc = "Flush To-Bus Buffer or FIFO\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flushtb {
    #[doc = "0: No action"]
    NoAction = 0,
    #[doc = "1: Flush the buffer"]
    Flush = 1,
}
impl From<Flushtb> for bool {
    #[inline(always)]
    fn from(variant: Flushtb) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FLUSHTB` writer - Flush To-Bus Buffer or FIFO"]
pub type FlushtbW<'a, REG> = crate::BitWriter<'a, REG, Flushtb>;
impl<'a, REG> FlushtbW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No action"]
    #[inline(always)]
    pub fn no_action(self) -> &'a mut crate::W<REG> {
        self.variant(Flushtb::NoAction)
    }
    #[doc = "Flush the buffer"]
    #[inline(always)]
    pub fn flush(self) -> &'a mut crate::W<REG> {
        self.variant(Flushtb::Flush)
    }
}
#[doc = "Flush From-Bus Buffer or FIFO\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flushfb {
    #[doc = "0: No action"]
    NoAction = 0,
    #[doc = "1: Flush the buffer"]
    Flush = 1,
}
impl From<Flushfb> for bool {
    #[inline(always)]
    fn from(variant: Flushfb) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FLUSHFB` writer - Flush From-Bus Buffer or FIFO"]
pub type FlushfbW<'a, REG> = crate::BitWriter<'a, REG, Flushfb>;
impl<'a, REG> FlushfbW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No action"]
    #[inline(always)]
    pub fn no_action(self) -> &'a mut crate::W<REG> {
        self.variant(Flushfb::NoAction)
    }
    #[doc = "Flush the buffer"]
    #[inline(always)]
    pub fn flush(self) -> &'a mut crate::W<REG> {
        self.variant(Flushfb::Flush)
    }
}
#[doc = "Unlock\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unlock {
    #[doc = "0: Locked"]
    Disabled = 0,
    #[doc = "1: Unlocked"]
    Enabled = 1,
}
impl From<Unlock> for bool {
    #[inline(always)]
    fn from(variant: Unlock) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `UNLOCK` writer - Unlock"]
pub type UnlockW<'a, REG> = crate::BitWriter<'a, REG, Unlock>;
impl<'a, REG> UnlockW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Locked"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Unlock::Disabled)
    }
    #[doc = "Unlocked"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Unlock::Enabled)
    }
}
#[doc = "Transmit Trigger Level\n\nValue on reset: 3"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Txtrig {
    #[doc = "0: Trigger when empty"]
    Empty = 0,
    #[doc = "1: Trigger when 1/4 full or less"]
    QuarterOrLess = 1,
    #[doc = "2: Trigger when 1/2 full or less"]
    HalfOrLess = 2,
    #[doc = "3: Trigger when 1 less than full or less (default)"]
    FullOrLess = 3,
}
impl From<Txtrig> for u8 {
    #[inline(always)]
    fn from(variant: Txtrig) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Txtrig {
    type Ux = u8;
}
impl crate::IsEnum for Txtrig {}
#[doc = "Field `TXTRIG` reader - Transmit Trigger Level"]
pub type TxtrigR = crate::FieldReader<Txtrig>;
impl TxtrigR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Txtrig {
        match self.bits {
            0 => Txtrig::Empty,
            1 => Txtrig::QuarterOrLess,
            2 => Txtrig::HalfOrLess,
            3 => Txtrig::FullOrLess,
            _ => unreachable!(),
        }
    }
    #[doc = "Trigger when empty"]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        *self == Txtrig::Empty
    }
    #[doc = "Trigger when 1/4 full or less"]
    #[inline(always)]
    pub fn is_quarter_or_less(&self) -> bool {
        *self == Txtrig::QuarterOrLess
    }
    #[doc = "Trigger when 1/2 full or less"]
    #[inline(always)]
    pub fn is_half_or_less(&self) -> bool {
        *self == Txtrig::HalfOrLess
    }
    #[doc = "Trigger when 1 less than full or less (default)"]
    #[inline(always)]
    pub fn is_full_or_less(&self) -> bool {
        *self == Txtrig::FullOrLess
    }
}
#[doc = "Field `TXTRIG` writer - Transmit Trigger Level"]
pub type TxtrigW<'a, REG> = crate::FieldWriter<'a, REG, 2, Txtrig, crate::Safe>;
impl<'a, REG> TxtrigW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Trigger when empty"]
    #[inline(always)]
    pub fn empty(self) -> &'a mut crate::W<REG> {
        self.variant(Txtrig::Empty)
    }
    #[doc = "Trigger when 1/4 full or less"]
    #[inline(always)]
    pub fn quarter_or_less(self) -> &'a mut crate::W<REG> {
        self.variant(Txtrig::QuarterOrLess)
    }
    #[doc = "Trigger when 1/2 full or less"]
    #[inline(always)]
    pub fn half_or_less(self) -> &'a mut crate::W<REG> {
        self.variant(Txtrig::HalfOrLess)
    }
    #[doc = "Trigger when 1 less than full or less (default)"]
    #[inline(always)]
    pub fn full_or_less(self) -> &'a mut crate::W<REG> {
        self.variant(Txtrig::FullOrLess)
    }
}
#[doc = "Receive Trigger Level\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Rxtrig {
    #[doc = "0: Trigger when not empty (default)"]
    NotEmpty = 0,
    #[doc = "1: Trigger when 1/4 full or more"]
    QuarterOrMore = 1,
    #[doc = "2: Trigger when 1/2 full or more"]
    HalfOrMore = 2,
    #[doc = "3: Trigger when 3/4 full or more"]
    ThreeQuarterOrMore = 3,
}
impl From<Rxtrig> for u8 {
    #[inline(always)]
    fn from(variant: Rxtrig) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Rxtrig {
    type Ux = u8;
}
impl crate::IsEnum for Rxtrig {}
#[doc = "Field `RXTRIG` reader - Receive Trigger Level"]
pub type RxtrigR = crate::FieldReader<Rxtrig>;
impl RxtrigR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rxtrig {
        match self.bits {
            0 => Rxtrig::NotEmpty,
            1 => Rxtrig::QuarterOrMore,
            2 => Rxtrig::HalfOrMore,
            3 => Rxtrig::ThreeQuarterOrMore,
            _ => unreachable!(),
        }
    }
    #[doc = "Trigger when not empty (default)"]
    #[inline(always)]
    pub fn is_not_empty(&self) -> bool {
        *self == Rxtrig::NotEmpty
    }
    #[doc = "Trigger when 1/4 full or more"]
    #[inline(always)]
    pub fn is_quarter_or_more(&self) -> bool {
        *self == Rxtrig::QuarterOrMore
    }
    #[doc = "Trigger when 1/2 full or more"]
    #[inline(always)]
    pub fn is_half_or_more(&self) -> bool {
        *self == Rxtrig::HalfOrMore
    }
    #[doc = "Trigger when 3/4 full or more"]
    #[inline(always)]
    pub fn is_three_quarter_or_more(&self) -> bool {
        *self == Rxtrig::ThreeQuarterOrMore
    }
}
#[doc = "Field `RXTRIG` writer - Receive Trigger Level"]
pub type RxtrigW<'a, REG> = crate::FieldWriter<'a, REG, 2, Rxtrig, crate::Safe>;
impl<'a, REG> RxtrigW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Trigger when not empty (default)"]
    #[inline(always)]
    pub fn not_empty(self) -> &'a mut crate::W<REG> {
        self.variant(Rxtrig::NotEmpty)
    }
    #[doc = "Trigger when 1/4 full or more"]
    #[inline(always)]
    pub fn quarter_or_more(self) -> &'a mut crate::W<REG> {
        self.variant(Rxtrig::QuarterOrMore)
    }
    #[doc = "Trigger when 1/2 full or more"]
    #[inline(always)]
    pub fn half_or_more(self) -> &'a mut crate::W<REG> {
        self.variant(Rxtrig::HalfOrMore)
    }
    #[doc = "Trigger when 3/4 full or more"]
    #[inline(always)]
    pub fn three_quarter_or_more(self) -> &'a mut crate::W<REG> {
        self.variant(Rxtrig::ThreeQuarterOrMore)
    }
}
#[doc = "Field `TXCOUNT` reader - Transmit Entry Count"]
pub type TxcountR = crate::FieldReader;
#[doc = "Field `RXCOUNT` reader - Receive Entry Count"]
pub type RxcountR = crate::FieldReader;
#[doc = "Transmit is Full\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Txfull {
    #[doc = "0: Not full"]
    NotFull = 0,
    #[doc = "1: Full"]
    Full = 1,
}
impl From<Txfull> for bool {
    #[inline(always)]
    fn from(variant: Txfull) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TXFULL` reader - Transmit is Full"]
pub type TxfullR = crate::BitReader<Txfull>;
impl TxfullR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Txfull {
        match self.bits {
            false => Txfull::NotFull,
            true => Txfull::Full,
        }
    }
    #[doc = "Not full"]
    #[inline(always)]
    pub fn is_not_full(&self) -> bool {
        *self == Txfull::NotFull
    }
    #[doc = "Full"]
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        *self == Txfull::Full
    }
}
#[doc = "Receive is Empty\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rxempty {
    #[doc = "0: Not empty"]
    NotEmpty = 0,
    #[doc = "1: Empty"]
    Empty = 1,
}
impl From<Rxempty> for bool {
    #[inline(always)]
    fn from(variant: Rxempty) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RXEMPTY` reader - Receive is Empty"]
pub type RxemptyR = crate::BitReader<Rxempty>;
impl RxemptyR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rxempty {
        match self.bits {
            false => Rxempty::NotEmpty,
            true => Rxempty::Empty,
        }
    }
    #[doc = "Not empty"]
    #[inline(always)]
    pub fn is_not_empty(&self) -> bool {
        *self == Rxempty::NotEmpty
    }
    #[doc = "Empty"]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        *self == Rxempty::Empty
    }
}
impl R {
    #[doc = "Bits 4:5 - Transmit Trigger Level"]
    #[inline(always)]
    pub fn txtrig(&self) -> TxtrigR {
        TxtrigR::new(((self.bits >> 4) & 3) as u8)
    }
    #[doc = "Bits 6:7 - Receive Trigger Level"]
    #[inline(always)]
    pub fn rxtrig(&self) -> RxtrigR {
        RxtrigR::new(((self.bits >> 6) & 3) as u8)
    }
    #[doc = "Bits 16:20 - Transmit Entry Count"]
    #[inline(always)]
    pub fn txcount(&self) -> TxcountR {
        TxcountR::new(((self.bits >> 16) & 0x1f) as u8)
    }
    #[doc = "Bits 24:28 - Receive Entry Count"]
    #[inline(always)]
    pub fn rxcount(&self) -> RxcountR {
        RxcountR::new(((self.bits >> 24) & 0x1f) as u8)
    }
    #[doc = "Bit 30 - Transmit is Full"]
    #[inline(always)]
    pub fn txfull(&self) -> TxfullR {
        TxfullR::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Receive is Empty"]
    #[inline(always)]
    pub fn rxempty(&self) -> RxemptyR {
        RxemptyR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Flush To-Bus Buffer or FIFO"]
    #[inline(always)]
    pub fn flushtb(&mut self) -> FlushtbW<MdatactrlSpec> {
        FlushtbW::new(self, 0)
    }
    #[doc = "Bit 1 - Flush From-Bus Buffer or FIFO"]
    #[inline(always)]
    pub fn flushfb(&mut self) -> FlushfbW<MdatactrlSpec> {
        FlushfbW::new(self, 1)
    }
    #[doc = "Bit 3 - Unlock"]
    #[inline(always)]
    pub fn unlock(&mut self) -> UnlockW<MdatactrlSpec> {
        UnlockW::new(self, 3)
    }
    #[doc = "Bits 4:5 - Transmit Trigger Level"]
    #[inline(always)]
    pub fn txtrig(&mut self) -> TxtrigW<MdatactrlSpec> {
        TxtrigW::new(self, 4)
    }
    #[doc = "Bits 6:7 - Receive Trigger Level"]
    #[inline(always)]
    pub fn rxtrig(&mut self) -> RxtrigW<MdatactrlSpec> {
        RxtrigW::new(self, 6)
    }
}
#[doc = "Controller Data Control\n\nYou can [`read`](crate::Reg::read) this register and get [`mdatactrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mdatactrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MdatactrlSpec;
impl crate::RegisterSpec for MdatactrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mdatactrl::R`](R) reader structure"]
impl crate::Readable for MdatactrlSpec {}
#[doc = "`write(|w| ..)` method takes [`mdatactrl::W`](W) writer structure"]
impl crate::Writable for MdatactrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MDATACTRL to value 0x8000_0030"]
impl crate::Resettable for MdatactrlSpec {
    const RESET_VALUE: u32 = 0x8000_0030;
}

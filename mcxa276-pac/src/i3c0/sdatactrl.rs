#[doc = "Register `SDATACTRL` reader"]
pub type R = crate::R<SdatactrlSpec>;
#[doc = "Register `SDATACTRL` writer"]
pub type W = crate::W<SdatactrlSpec>;
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
    #[doc = "0: Cannot be changed"]
    Disabled = 0,
    #[doc = "1: Can be changed"]
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
    #[doc = "Cannot be changed"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Unlock::Disabled)
    }
    #[doc = "Can be changed"]
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
    Triggrempty = 0,
    #[doc = "1: Trigger when 1/4 full or less"]
    Triggronefourth = 1,
    #[doc = "2: Trigger when 1/2 full or less"]
    Triggronehalf = 2,
    #[doc = "3: Default (trigger when 1 less than full or less)"]
    Triggroneless = 3,
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
            0 => Txtrig::Triggrempty,
            1 => Txtrig::Triggronefourth,
            2 => Txtrig::Triggronehalf,
            3 => Txtrig::Triggroneless,
            _ => unreachable!(),
        }
    }
    #[doc = "Trigger when empty"]
    #[inline(always)]
    pub fn is_triggrempty(&self) -> bool {
        *self == Txtrig::Triggrempty
    }
    #[doc = "Trigger when 1/4 full or less"]
    #[inline(always)]
    pub fn is_triggronefourth(&self) -> bool {
        *self == Txtrig::Triggronefourth
    }
    #[doc = "Trigger when 1/2 full or less"]
    #[inline(always)]
    pub fn is_triggronehalf(&self) -> bool {
        *self == Txtrig::Triggronehalf
    }
    #[doc = "Default (trigger when 1 less than full or less)"]
    #[inline(always)]
    pub fn is_triggroneless(&self) -> bool {
        *self == Txtrig::Triggroneless
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
    pub fn triggrempty(self) -> &'a mut crate::W<REG> {
        self.variant(Txtrig::Triggrempty)
    }
    #[doc = "Trigger when 1/4 full or less"]
    #[inline(always)]
    pub fn triggronefourth(self) -> &'a mut crate::W<REG> {
        self.variant(Txtrig::Triggronefourth)
    }
    #[doc = "Trigger when 1/2 full or less"]
    #[inline(always)]
    pub fn triggronehalf(self) -> &'a mut crate::W<REG> {
        self.variant(Txtrig::Triggronehalf)
    }
    #[doc = "Default (trigger when 1 less than full or less)"]
    #[inline(always)]
    pub fn triggroneless(self) -> &'a mut crate::W<REG> {
        self.variant(Txtrig::Triggroneless)
    }
}
#[doc = "Receive Trigger Level\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Rxtrig {
    #[doc = "0: Trigger when not empty (default)"]
    Triggrnotempty = 0,
    #[doc = "1: Trigger when 1/4 or more full"]
    Triggronefourth = 1,
    #[doc = "2: Trigger when 1/2 or more full"]
    Triggronehalf = 2,
    #[doc = "3: Trigger when 3/4 or more full"]
    Triggrthreefourths = 3,
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
            0 => Rxtrig::Triggrnotempty,
            1 => Rxtrig::Triggronefourth,
            2 => Rxtrig::Triggronehalf,
            3 => Rxtrig::Triggrthreefourths,
            _ => unreachable!(),
        }
    }
    #[doc = "Trigger when not empty (default)"]
    #[inline(always)]
    pub fn is_triggrnotempty(&self) -> bool {
        *self == Rxtrig::Triggrnotempty
    }
    #[doc = "Trigger when 1/4 or more full"]
    #[inline(always)]
    pub fn is_triggronefourth(&self) -> bool {
        *self == Rxtrig::Triggronefourth
    }
    #[doc = "Trigger when 1/2 or more full"]
    #[inline(always)]
    pub fn is_triggronehalf(&self) -> bool {
        *self == Rxtrig::Triggronehalf
    }
    #[doc = "Trigger when 3/4 or more full"]
    #[inline(always)]
    pub fn is_triggrthreefourths(&self) -> bool {
        *self == Rxtrig::Triggrthreefourths
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
    pub fn triggrnotempty(self) -> &'a mut crate::W<REG> {
        self.variant(Rxtrig::Triggrnotempty)
    }
    #[doc = "Trigger when 1/4 or more full"]
    #[inline(always)]
    pub fn triggronefourth(self) -> &'a mut crate::W<REG> {
        self.variant(Rxtrig::Triggronefourth)
    }
    #[doc = "Trigger when 1/2 or more full"]
    #[inline(always)]
    pub fn triggronehalf(self) -> &'a mut crate::W<REG> {
        self.variant(Rxtrig::Triggronehalf)
    }
    #[doc = "Trigger when 3/4 or more full"]
    #[inline(always)]
    pub fn triggrthreefourths(self) -> &'a mut crate::W<REG> {
        self.variant(Rxtrig::Triggrthreefourths)
    }
}
#[doc = "Field `TXCOUNT` reader - Count of Entries in Transmit"]
pub type TxcountR = crate::FieldReader;
#[doc = "Field `RXCOUNT` reader - Count of Entries in Receive"]
pub type RxcountR = crate::FieldReader;
#[doc = "Transmit is Full\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Txfull {
    #[doc = "0: Not full"]
    Txisnotfull = 0,
    #[doc = "1: Full"]
    Txisfull = 1,
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
            false => Txfull::Txisnotfull,
            true => Txfull::Txisfull,
        }
    }
    #[doc = "Not full"]
    #[inline(always)]
    pub fn is_txisnotfull(&self) -> bool {
        *self == Txfull::Txisnotfull
    }
    #[doc = "Full"]
    #[inline(always)]
    pub fn is_txisfull(&self) -> bool {
        *self == Txfull::Txisfull
    }
}
#[doc = "Receive is Empty\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rxempty {
    #[doc = "0: Not empty"]
    Rxisnotempty = 0,
    #[doc = "1: Empty"]
    Rxisempty = 1,
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
            false => Rxempty::Rxisnotempty,
            true => Rxempty::Rxisempty,
        }
    }
    #[doc = "Not empty"]
    #[inline(always)]
    pub fn is_rxisnotempty(&self) -> bool {
        *self == Rxempty::Rxisnotempty
    }
    #[doc = "Empty"]
    #[inline(always)]
    pub fn is_rxisempty(&self) -> bool {
        *self == Rxempty::Rxisempty
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
    #[doc = "Bits 16:20 - Count of Entries in Transmit"]
    #[inline(always)]
    pub fn txcount(&self) -> TxcountR {
        TxcountR::new(((self.bits >> 16) & 0x1f) as u8)
    }
    #[doc = "Bits 24:28 - Count of Entries in Receive"]
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
    pub fn flushtb(&mut self) -> FlushtbW<SdatactrlSpec> {
        FlushtbW::new(self, 0)
    }
    #[doc = "Bit 1 - Flush From-Bus Buffer or FIFO"]
    #[inline(always)]
    pub fn flushfb(&mut self) -> FlushfbW<SdatactrlSpec> {
        FlushfbW::new(self, 1)
    }
    #[doc = "Bit 3 - Unlock"]
    #[inline(always)]
    pub fn unlock(&mut self) -> UnlockW<SdatactrlSpec> {
        UnlockW::new(self, 3)
    }
    #[doc = "Bits 4:5 - Transmit Trigger Level"]
    #[inline(always)]
    pub fn txtrig(&mut self) -> TxtrigW<SdatactrlSpec> {
        TxtrigW::new(self, 4)
    }
    #[doc = "Bits 6:7 - Receive Trigger Level"]
    #[inline(always)]
    pub fn rxtrig(&mut self) -> RxtrigW<SdatactrlSpec> {
        RxtrigW::new(self, 6)
    }
}
#[doc = "Target Data Control\n\nYou can [`read`](crate::Reg::read) this register and get [`sdatactrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sdatactrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SdatactrlSpec;
impl crate::RegisterSpec for SdatactrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sdatactrl::R`](R) reader structure"]
impl crate::Readable for SdatactrlSpec {}
#[doc = "`write(|w| ..)` method takes [`sdatactrl::W`](W) writer structure"]
impl crate::Writable for SdatactrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SDATACTRL to value 0x8000_0030"]
impl crate::Resettable for SdatactrlSpec {
    const RESET_VALUE: u32 = 0x8000_0030;
}

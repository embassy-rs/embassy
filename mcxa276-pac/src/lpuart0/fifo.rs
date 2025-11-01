#[doc = "Register `FIFO` reader"]
pub type R = crate::R<FifoSpec>;
#[doc = "Register `FIFO` writer"]
pub type W = crate::W<FifoSpec>;
#[doc = "Receive FIFO Buffer Depth\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Rxfifosize {
    #[doc = "0: 1"]
    Fifo1 = 0,
    #[doc = "1: 4"]
    Fifo4 = 1,
    #[doc = "2: 8"]
    Fifo8 = 2,
    #[doc = "3: 16"]
    Fifo16 = 3,
    #[doc = "4: 32"]
    Fifo32 = 4,
    #[doc = "5: 64"]
    Fifo64 = 5,
    #[doc = "6: 128"]
    Fifo128 = 6,
    #[doc = "7: 256"]
    Fifo256 = 7,
}
impl From<Rxfifosize> for u8 {
    #[inline(always)]
    fn from(variant: Rxfifosize) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Rxfifosize {
    type Ux = u8;
}
impl crate::IsEnum for Rxfifosize {}
#[doc = "Field `RXFIFOSIZE` reader - Receive FIFO Buffer Depth"]
pub type RxfifosizeR = crate::FieldReader<Rxfifosize>;
impl RxfifosizeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rxfifosize {
        match self.bits {
            0 => Rxfifosize::Fifo1,
            1 => Rxfifosize::Fifo4,
            2 => Rxfifosize::Fifo8,
            3 => Rxfifosize::Fifo16,
            4 => Rxfifosize::Fifo32,
            5 => Rxfifosize::Fifo64,
            6 => Rxfifosize::Fifo128,
            7 => Rxfifosize::Fifo256,
            _ => unreachable!(),
        }
    }
    #[doc = "1"]
    #[inline(always)]
    pub fn is_fifo_1(&self) -> bool {
        *self == Rxfifosize::Fifo1
    }
    #[doc = "4"]
    #[inline(always)]
    pub fn is_fifo_4(&self) -> bool {
        *self == Rxfifosize::Fifo4
    }
    #[doc = "8"]
    #[inline(always)]
    pub fn is_fifo_8(&self) -> bool {
        *self == Rxfifosize::Fifo8
    }
    #[doc = "16"]
    #[inline(always)]
    pub fn is_fifo_16(&self) -> bool {
        *self == Rxfifosize::Fifo16
    }
    #[doc = "32"]
    #[inline(always)]
    pub fn is_fifo_32(&self) -> bool {
        *self == Rxfifosize::Fifo32
    }
    #[doc = "64"]
    #[inline(always)]
    pub fn is_fifo_64(&self) -> bool {
        *self == Rxfifosize::Fifo64
    }
    #[doc = "128"]
    #[inline(always)]
    pub fn is_fifo_128(&self) -> bool {
        *self == Rxfifosize::Fifo128
    }
    #[doc = "256"]
    #[inline(always)]
    pub fn is_fifo_256(&self) -> bool {
        *self == Rxfifosize::Fifo256
    }
}
#[doc = "Receive FIFO Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rxfe {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Rxfe> for bool {
    #[inline(always)]
    fn from(variant: Rxfe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RXFE` reader - Receive FIFO Enable"]
pub type RxfeR = crate::BitReader<Rxfe>;
impl RxfeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rxfe {
        match self.bits {
            false => Rxfe::Disabled,
            true => Rxfe::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Rxfe::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Rxfe::Enabled
    }
}
#[doc = "Field `RXFE` writer - Receive FIFO Enable"]
pub type RxfeW<'a, REG> = crate::BitWriter<'a, REG, Rxfe>;
impl<'a, REG> RxfeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rxfe::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rxfe::Enabled)
    }
}
#[doc = "Transmit FIFO Buffer Depth\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Txfifosize {
    #[doc = "0: 1"]
    Fifo1 = 0,
    #[doc = "1: 4"]
    Fifo4 = 1,
    #[doc = "2: 8"]
    Fifo8 = 2,
    #[doc = "3: 16"]
    Fifo16 = 3,
    #[doc = "4: 32"]
    Fifo32 = 4,
    #[doc = "5: 64"]
    Fifo64 = 5,
    #[doc = "6: 128"]
    Fifo128 = 6,
    #[doc = "7: 256"]
    Fifo256 = 7,
}
impl From<Txfifosize> for u8 {
    #[inline(always)]
    fn from(variant: Txfifosize) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Txfifosize {
    type Ux = u8;
}
impl crate::IsEnum for Txfifosize {}
#[doc = "Field `TXFIFOSIZE` reader - Transmit FIFO Buffer Depth"]
pub type TxfifosizeR = crate::FieldReader<Txfifosize>;
impl TxfifosizeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Txfifosize {
        match self.bits {
            0 => Txfifosize::Fifo1,
            1 => Txfifosize::Fifo4,
            2 => Txfifosize::Fifo8,
            3 => Txfifosize::Fifo16,
            4 => Txfifosize::Fifo32,
            5 => Txfifosize::Fifo64,
            6 => Txfifosize::Fifo128,
            7 => Txfifosize::Fifo256,
            _ => unreachable!(),
        }
    }
    #[doc = "1"]
    #[inline(always)]
    pub fn is_fifo_1(&self) -> bool {
        *self == Txfifosize::Fifo1
    }
    #[doc = "4"]
    #[inline(always)]
    pub fn is_fifo_4(&self) -> bool {
        *self == Txfifosize::Fifo4
    }
    #[doc = "8"]
    #[inline(always)]
    pub fn is_fifo_8(&self) -> bool {
        *self == Txfifosize::Fifo8
    }
    #[doc = "16"]
    #[inline(always)]
    pub fn is_fifo_16(&self) -> bool {
        *self == Txfifosize::Fifo16
    }
    #[doc = "32"]
    #[inline(always)]
    pub fn is_fifo_32(&self) -> bool {
        *self == Txfifosize::Fifo32
    }
    #[doc = "64"]
    #[inline(always)]
    pub fn is_fifo_64(&self) -> bool {
        *self == Txfifosize::Fifo64
    }
    #[doc = "128"]
    #[inline(always)]
    pub fn is_fifo_128(&self) -> bool {
        *self == Txfifosize::Fifo128
    }
    #[doc = "256"]
    #[inline(always)]
    pub fn is_fifo_256(&self) -> bool {
        *self == Txfifosize::Fifo256
    }
}
#[doc = "Transmit FIFO Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Txfe {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Txfe> for bool {
    #[inline(always)]
    fn from(variant: Txfe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TXFE` reader - Transmit FIFO Enable"]
pub type TxfeR = crate::BitReader<Txfe>;
impl TxfeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Txfe {
        match self.bits {
            false => Txfe::Disabled,
            true => Txfe::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Txfe::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Txfe::Enabled
    }
}
#[doc = "Field `TXFE` writer - Transmit FIFO Enable"]
pub type TxfeW<'a, REG> = crate::BitWriter<'a, REG, Txfe>;
impl<'a, REG> TxfeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Txfe::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Txfe::Enabled)
    }
}
#[doc = "Receive FIFO Underflow Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rxufe {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Rxufe> for bool {
    #[inline(always)]
    fn from(variant: Rxufe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RXUFE` reader - Receive FIFO Underflow Interrupt Enable"]
pub type RxufeR = crate::BitReader<Rxufe>;
impl RxufeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rxufe {
        match self.bits {
            false => Rxufe::Disabled,
            true => Rxufe::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Rxufe::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Rxufe::Enabled
    }
}
#[doc = "Field `RXUFE` writer - Receive FIFO Underflow Interrupt Enable"]
pub type RxufeW<'a, REG> = crate::BitWriter<'a, REG, Rxufe>;
impl<'a, REG> RxufeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rxufe::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rxufe::Enabled)
    }
}
#[doc = "Transmit FIFO Overflow Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Txofe {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Txofe> for bool {
    #[inline(always)]
    fn from(variant: Txofe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TXOFE` reader - Transmit FIFO Overflow Interrupt Enable"]
pub type TxofeR = crate::BitReader<Txofe>;
impl TxofeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Txofe {
        match self.bits {
            false => Txofe::Disabled,
            true => Txofe::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Txofe::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Txofe::Enabled
    }
}
#[doc = "Field `TXOFE` writer - Transmit FIFO Overflow Interrupt Enable"]
pub type TxofeW<'a, REG> = crate::BitWriter<'a, REG, Txofe>;
impl<'a, REG> TxofeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Txofe::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Txofe::Enabled)
    }
}
#[doc = "Receiver Idle Empty Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Rxiden {
    #[doc = "0: Disable STAT\\[RDRF\\] to become 1 because of partially filled FIFO when the receiver is idle"]
    Disabled = 0,
    #[doc = "1: Enable STAT\\[RDRF\\] to become 1 because of partially filled FIFO when the receiver is idle for one character"]
    Idle1 = 1,
    #[doc = "2: Enable STAT\\[RDRF\\] to become 1 because of partially filled FIFO when the receiver is idle for two characters"]
    Idle2 = 2,
    #[doc = "3: Enable STAT\\[RDRF\\] to become 1 because of partially filled FIFO when the receiver is idle for four characters"]
    Idle4 = 3,
    #[doc = "4: Enable STAT\\[RDRF\\] to become 1 because of partially filled FIFO when the receiver is idle for eight characters"]
    Idle8 = 4,
    #[doc = "5: Enable STAT\\[RDRF\\] to become 1 because of partially filled FIFO when the receiver is idle for 16 characters"]
    Idle16 = 5,
    #[doc = "6: Enable STAT\\[RDRF\\] to become 1 because of partially filled FIFO when the receiver is idle for 32 characters"]
    Idle32 = 6,
    #[doc = "7: Enable STAT\\[RDRF\\] to become 1 because of partially filled FIFO when the receiver is idle for 64 characters"]
    Idle64 = 7,
}
impl From<Rxiden> for u8 {
    #[inline(always)]
    fn from(variant: Rxiden) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Rxiden {
    type Ux = u8;
}
impl crate::IsEnum for Rxiden {}
#[doc = "Field `RXIDEN` reader - Receiver Idle Empty Enable"]
pub type RxidenR = crate::FieldReader<Rxiden>;
impl RxidenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rxiden {
        match self.bits {
            0 => Rxiden::Disabled,
            1 => Rxiden::Idle1,
            2 => Rxiden::Idle2,
            3 => Rxiden::Idle4,
            4 => Rxiden::Idle8,
            5 => Rxiden::Idle16,
            6 => Rxiden::Idle32,
            7 => Rxiden::Idle64,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable STAT\\[RDRF\\] to become 1 because of partially filled FIFO when the receiver is idle"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Rxiden::Disabled
    }
    #[doc = "Enable STAT\\[RDRF\\] to become 1 because of partially filled FIFO when the receiver is idle for one character"]
    #[inline(always)]
    pub fn is_idle_1(&self) -> bool {
        *self == Rxiden::Idle1
    }
    #[doc = "Enable STAT\\[RDRF\\] to become 1 because of partially filled FIFO when the receiver is idle for two characters"]
    #[inline(always)]
    pub fn is_idle_2(&self) -> bool {
        *self == Rxiden::Idle2
    }
    #[doc = "Enable STAT\\[RDRF\\] to become 1 because of partially filled FIFO when the receiver is idle for four characters"]
    #[inline(always)]
    pub fn is_idle_4(&self) -> bool {
        *self == Rxiden::Idle4
    }
    #[doc = "Enable STAT\\[RDRF\\] to become 1 because of partially filled FIFO when the receiver is idle for eight characters"]
    #[inline(always)]
    pub fn is_idle_8(&self) -> bool {
        *self == Rxiden::Idle8
    }
    #[doc = "Enable STAT\\[RDRF\\] to become 1 because of partially filled FIFO when the receiver is idle for 16 characters"]
    #[inline(always)]
    pub fn is_idle_16(&self) -> bool {
        *self == Rxiden::Idle16
    }
    #[doc = "Enable STAT\\[RDRF\\] to become 1 because of partially filled FIFO when the receiver is idle for 32 characters"]
    #[inline(always)]
    pub fn is_idle_32(&self) -> bool {
        *self == Rxiden::Idle32
    }
    #[doc = "Enable STAT\\[RDRF\\] to become 1 because of partially filled FIFO when the receiver is idle for 64 characters"]
    #[inline(always)]
    pub fn is_idle_64(&self) -> bool {
        *self == Rxiden::Idle64
    }
}
#[doc = "Field `RXIDEN` writer - Receiver Idle Empty Enable"]
pub type RxidenW<'a, REG> = crate::FieldWriter<'a, REG, 3, Rxiden, crate::Safe>;
impl<'a, REG> RxidenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable STAT\\[RDRF\\] to become 1 because of partially filled FIFO when the receiver is idle"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rxiden::Disabled)
    }
    #[doc = "Enable STAT\\[RDRF\\] to become 1 because of partially filled FIFO when the receiver is idle for one character"]
    #[inline(always)]
    pub fn idle_1(self) -> &'a mut crate::W<REG> {
        self.variant(Rxiden::Idle1)
    }
    #[doc = "Enable STAT\\[RDRF\\] to become 1 because of partially filled FIFO when the receiver is idle for two characters"]
    #[inline(always)]
    pub fn idle_2(self) -> &'a mut crate::W<REG> {
        self.variant(Rxiden::Idle2)
    }
    #[doc = "Enable STAT\\[RDRF\\] to become 1 because of partially filled FIFO when the receiver is idle for four characters"]
    #[inline(always)]
    pub fn idle_4(self) -> &'a mut crate::W<REG> {
        self.variant(Rxiden::Idle4)
    }
    #[doc = "Enable STAT\\[RDRF\\] to become 1 because of partially filled FIFO when the receiver is idle for eight characters"]
    #[inline(always)]
    pub fn idle_8(self) -> &'a mut crate::W<REG> {
        self.variant(Rxiden::Idle8)
    }
    #[doc = "Enable STAT\\[RDRF\\] to become 1 because of partially filled FIFO when the receiver is idle for 16 characters"]
    #[inline(always)]
    pub fn idle_16(self) -> &'a mut crate::W<REG> {
        self.variant(Rxiden::Idle16)
    }
    #[doc = "Enable STAT\\[RDRF\\] to become 1 because of partially filled FIFO when the receiver is idle for 32 characters"]
    #[inline(always)]
    pub fn idle_32(self) -> &'a mut crate::W<REG> {
        self.variant(Rxiden::Idle32)
    }
    #[doc = "Enable STAT\\[RDRF\\] to become 1 because of partially filled FIFO when the receiver is idle for 64 characters"]
    #[inline(always)]
    pub fn idle_64(self) -> &'a mut crate::W<REG> {
        self.variant(Rxiden::Idle64)
    }
}
#[doc = "Receive FIFO Flush\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rxflush {
    #[doc = "0: No effect"]
    NoEffect = 0,
    #[doc = "1: All data flushed out"]
    RxfifoRst = 1,
}
impl From<Rxflush> for bool {
    #[inline(always)]
    fn from(variant: Rxflush) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RXFLUSH` reader - Receive FIFO Flush"]
pub type RxflushR = crate::BitReader<Rxflush>;
impl RxflushR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rxflush {
        match self.bits {
            false => Rxflush::NoEffect,
            true => Rxflush::RxfifoRst,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_no_effect(&self) -> bool {
        *self == Rxflush::NoEffect
    }
    #[doc = "All data flushed out"]
    #[inline(always)]
    pub fn is_rxfifo_rst(&self) -> bool {
        *self == Rxflush::RxfifoRst
    }
}
#[doc = "Field `RXFLUSH` writer - Receive FIFO Flush"]
pub type RxflushW<'a, REG> = crate::BitWriter<'a, REG, Rxflush>;
impl<'a, REG> RxflushW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn no_effect(self) -> &'a mut crate::W<REG> {
        self.variant(Rxflush::NoEffect)
    }
    #[doc = "All data flushed out"]
    #[inline(always)]
    pub fn rxfifo_rst(self) -> &'a mut crate::W<REG> {
        self.variant(Rxflush::RxfifoRst)
    }
}
#[doc = "Transmit FIFO Flush\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Txflush {
    #[doc = "0: No effect"]
    NoEffect = 0,
    #[doc = "1: All data flushed out"]
    TxfifoRst = 1,
}
impl From<Txflush> for bool {
    #[inline(always)]
    fn from(variant: Txflush) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TXFLUSH` reader - Transmit FIFO Flush"]
pub type TxflushR = crate::BitReader<Txflush>;
impl TxflushR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Txflush {
        match self.bits {
            false => Txflush::NoEffect,
            true => Txflush::TxfifoRst,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_no_effect(&self) -> bool {
        *self == Txflush::NoEffect
    }
    #[doc = "All data flushed out"]
    #[inline(always)]
    pub fn is_txfifo_rst(&self) -> bool {
        *self == Txflush::TxfifoRst
    }
}
#[doc = "Field `TXFLUSH` writer - Transmit FIFO Flush"]
pub type TxflushW<'a, REG> = crate::BitWriter<'a, REG, Txflush>;
impl<'a, REG> TxflushW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn no_effect(self) -> &'a mut crate::W<REG> {
        self.variant(Txflush::NoEffect)
    }
    #[doc = "All data flushed out"]
    #[inline(always)]
    pub fn txfifo_rst(self) -> &'a mut crate::W<REG> {
        self.variant(Txflush::TxfifoRst)
    }
}
#[doc = "Receiver FIFO Underflow Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rxuf {
    #[doc = "0: No underflow"]
    NoUnderflow = 0,
    #[doc = "1: Underflow"]
    Underflow = 1,
}
impl From<Rxuf> for bool {
    #[inline(always)]
    fn from(variant: Rxuf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RXUF` reader - Receiver FIFO Underflow Flag"]
pub type RxufR = crate::BitReader<Rxuf>;
impl RxufR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rxuf {
        match self.bits {
            false => Rxuf::NoUnderflow,
            true => Rxuf::Underflow,
        }
    }
    #[doc = "No underflow"]
    #[inline(always)]
    pub fn is_no_underflow(&self) -> bool {
        *self == Rxuf::NoUnderflow
    }
    #[doc = "Underflow"]
    #[inline(always)]
    pub fn is_underflow(&self) -> bool {
        *self == Rxuf::Underflow
    }
}
#[doc = "Field `RXUF` writer - Receiver FIFO Underflow Flag"]
pub type RxufW<'a, REG> = crate::BitWriter1C<'a, REG, Rxuf>;
impl<'a, REG> RxufW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No underflow"]
    #[inline(always)]
    pub fn no_underflow(self) -> &'a mut crate::W<REG> {
        self.variant(Rxuf::NoUnderflow)
    }
    #[doc = "Underflow"]
    #[inline(always)]
    pub fn underflow(self) -> &'a mut crate::W<REG> {
        self.variant(Rxuf::Underflow)
    }
}
#[doc = "Transmitter FIFO Overflow Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Txof {
    #[doc = "0: No overflow"]
    NoOverflow = 0,
    #[doc = "1: Overflow"]
    Overflow = 1,
}
impl From<Txof> for bool {
    #[inline(always)]
    fn from(variant: Txof) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TXOF` reader - Transmitter FIFO Overflow Flag"]
pub type TxofR = crate::BitReader<Txof>;
impl TxofR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Txof {
        match self.bits {
            false => Txof::NoOverflow,
            true => Txof::Overflow,
        }
    }
    #[doc = "No overflow"]
    #[inline(always)]
    pub fn is_no_overflow(&self) -> bool {
        *self == Txof::NoOverflow
    }
    #[doc = "Overflow"]
    #[inline(always)]
    pub fn is_overflow(&self) -> bool {
        *self == Txof::Overflow
    }
}
#[doc = "Field `TXOF` writer - Transmitter FIFO Overflow Flag"]
pub type TxofW<'a, REG> = crate::BitWriter1C<'a, REG, Txof>;
impl<'a, REG> TxofW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No overflow"]
    #[inline(always)]
    pub fn no_overflow(self) -> &'a mut crate::W<REG> {
        self.variant(Txof::NoOverflow)
    }
    #[doc = "Overflow"]
    #[inline(always)]
    pub fn overflow(self) -> &'a mut crate::W<REG> {
        self.variant(Txof::Overflow)
    }
}
#[doc = "Receive FIFO Or Buffer Empty\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rxempt {
    #[doc = "0: Not empty"]
    NotEmpty = 0,
    #[doc = "1: Empty"]
    Empty = 1,
}
impl From<Rxempt> for bool {
    #[inline(always)]
    fn from(variant: Rxempt) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RXEMPT` reader - Receive FIFO Or Buffer Empty"]
pub type RxemptR = crate::BitReader<Rxempt>;
impl RxemptR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rxempt {
        match self.bits {
            false => Rxempt::NotEmpty,
            true => Rxempt::Empty,
        }
    }
    #[doc = "Not empty"]
    #[inline(always)]
    pub fn is_not_empty(&self) -> bool {
        *self == Rxempt::NotEmpty
    }
    #[doc = "Empty"]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        *self == Rxempt::Empty
    }
}
#[doc = "Transmit FIFO Or Buffer Empty\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Txempt {
    #[doc = "0: Not empty"]
    NotEmpty = 0,
    #[doc = "1: Empty"]
    Empty = 1,
}
impl From<Txempt> for bool {
    #[inline(always)]
    fn from(variant: Txempt) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TXEMPT` reader - Transmit FIFO Or Buffer Empty"]
pub type TxemptR = crate::BitReader<Txempt>;
impl TxemptR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Txempt {
        match self.bits {
            false => Txempt::NotEmpty,
            true => Txempt::Empty,
        }
    }
    #[doc = "Not empty"]
    #[inline(always)]
    pub fn is_not_empty(&self) -> bool {
        *self == Txempt::NotEmpty
    }
    #[doc = "Empty"]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        *self == Txempt::Empty
    }
}
impl R {
    #[doc = "Bits 0:2 - Receive FIFO Buffer Depth"]
    #[inline(always)]
    pub fn rxfifosize(&self) -> RxfifosizeR {
        RxfifosizeR::new((self.bits & 7) as u8)
    }
    #[doc = "Bit 3 - Receive FIFO Enable"]
    #[inline(always)]
    pub fn rxfe(&self) -> RxfeR {
        RxfeR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bits 4:6 - Transmit FIFO Buffer Depth"]
    #[inline(always)]
    pub fn txfifosize(&self) -> TxfifosizeR {
        TxfifosizeR::new(((self.bits >> 4) & 7) as u8)
    }
    #[doc = "Bit 7 - Transmit FIFO Enable"]
    #[inline(always)]
    pub fn txfe(&self) -> TxfeR {
        TxfeR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Receive FIFO Underflow Interrupt Enable"]
    #[inline(always)]
    pub fn rxufe(&self) -> RxufeR {
        RxufeR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Transmit FIFO Overflow Interrupt Enable"]
    #[inline(always)]
    pub fn txofe(&self) -> TxofeR {
        TxofeR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bits 10:12 - Receiver Idle Empty Enable"]
    #[inline(always)]
    pub fn rxiden(&self) -> RxidenR {
        RxidenR::new(((self.bits >> 10) & 7) as u8)
    }
    #[doc = "Bit 14 - Receive FIFO Flush"]
    #[inline(always)]
    pub fn rxflush(&self) -> RxflushR {
        RxflushR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Transmit FIFO Flush"]
    #[inline(always)]
    pub fn txflush(&self) -> TxflushR {
        TxflushR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - Receiver FIFO Underflow Flag"]
    #[inline(always)]
    pub fn rxuf(&self) -> RxufR {
        RxufR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Transmitter FIFO Overflow Flag"]
    #[inline(always)]
    pub fn txof(&self) -> TxofR {
        TxofR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 22 - Receive FIFO Or Buffer Empty"]
    #[inline(always)]
    pub fn rxempt(&self) -> RxemptR {
        RxemptR::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - Transmit FIFO Or Buffer Empty"]
    #[inline(always)]
    pub fn txempt(&self) -> TxemptR {
        TxemptR::new(((self.bits >> 23) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 3 - Receive FIFO Enable"]
    #[inline(always)]
    pub fn rxfe(&mut self) -> RxfeW<FifoSpec> {
        RxfeW::new(self, 3)
    }
    #[doc = "Bit 7 - Transmit FIFO Enable"]
    #[inline(always)]
    pub fn txfe(&mut self) -> TxfeW<FifoSpec> {
        TxfeW::new(self, 7)
    }
    #[doc = "Bit 8 - Receive FIFO Underflow Interrupt Enable"]
    #[inline(always)]
    pub fn rxufe(&mut self) -> RxufeW<FifoSpec> {
        RxufeW::new(self, 8)
    }
    #[doc = "Bit 9 - Transmit FIFO Overflow Interrupt Enable"]
    #[inline(always)]
    pub fn txofe(&mut self) -> TxofeW<FifoSpec> {
        TxofeW::new(self, 9)
    }
    #[doc = "Bits 10:12 - Receiver Idle Empty Enable"]
    #[inline(always)]
    pub fn rxiden(&mut self) -> RxidenW<FifoSpec> {
        RxidenW::new(self, 10)
    }
    #[doc = "Bit 14 - Receive FIFO Flush"]
    #[inline(always)]
    pub fn rxflush(&mut self) -> RxflushW<FifoSpec> {
        RxflushW::new(self, 14)
    }
    #[doc = "Bit 15 - Transmit FIFO Flush"]
    #[inline(always)]
    pub fn txflush(&mut self) -> TxflushW<FifoSpec> {
        TxflushW::new(self, 15)
    }
    #[doc = "Bit 16 - Receiver FIFO Underflow Flag"]
    #[inline(always)]
    pub fn rxuf(&mut self) -> RxufW<FifoSpec> {
        RxufW::new(self, 16)
    }
    #[doc = "Bit 17 - Transmitter FIFO Overflow Flag"]
    #[inline(always)]
    pub fn txof(&mut self) -> TxofW<FifoSpec> {
        TxofW::new(self, 17)
    }
}
#[doc = "FIFO\n\nYou can [`read`](crate::Reg::read) this register and get [`fifo::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fifo::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FifoSpec;
impl crate::RegisterSpec for FifoSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`fifo::R`](R) reader structure"]
impl crate::Readable for FifoSpec {}
#[doc = "`write(|w| ..)` method takes [`fifo::W`](W) writer structure"]
impl crate::Writable for FifoSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x0003_0000;
}
#[doc = "`reset()` method sets FIFO to value 0x00c0_0011"]
impl crate::Resettable for FifoSpec {
    const RESET_VALUE: u32 = 0x00c0_0011;
}

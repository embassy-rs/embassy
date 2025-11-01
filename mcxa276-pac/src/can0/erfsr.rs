#[doc = "Register `ERFSR` reader"]
pub type R = crate::R<ErfsrSpec>;
#[doc = "Register `ERFSR` writer"]
pub type W = crate::W<ErfsrSpec>;
#[doc = "Field `ERFEL` reader - Enhanced RX FIFO Elements"]
pub type ErfelR = crate::FieldReader;
#[doc = "Enhanced RX FIFO Full Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Erff {
    #[doc = "0: Not full"]
    NotFull = 0,
    #[doc = "1: Full"]
    Full = 1,
}
impl From<Erff> for bool {
    #[inline(always)]
    fn from(variant: Erff) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERFF` reader - Enhanced RX FIFO Full Flag"]
pub type ErffR = crate::BitReader<Erff>;
impl ErffR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Erff {
        match self.bits {
            false => Erff::NotFull,
            true => Erff::Full,
        }
    }
    #[doc = "Not full"]
    #[inline(always)]
    pub fn is_not_full(&self) -> bool {
        *self == Erff::NotFull
    }
    #[doc = "Full"]
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        *self == Erff::Full
    }
}
#[doc = "Enhanced RX FIFO Empty Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Erfe {
    #[doc = "0: Not empty"]
    NotEmpty = 0,
    #[doc = "1: Empty"]
    Empty = 1,
}
impl From<Erfe> for bool {
    #[inline(always)]
    fn from(variant: Erfe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERFE` reader - Enhanced RX FIFO Empty Flag"]
pub type ErfeR = crate::BitReader<Erfe>;
impl ErfeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Erfe {
        match self.bits {
            false => Erfe::NotEmpty,
            true => Erfe::Empty,
        }
    }
    #[doc = "Not empty"]
    #[inline(always)]
    pub fn is_not_empty(&self) -> bool {
        *self == Erfe::NotEmpty
    }
    #[doc = "Empty"]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        *self == Erfe::Empty
    }
}
#[doc = "Enhanced RX FIFO Clear\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Erfclr {
    #[doc = "0: No effect"]
    NoEffect = 0,
    #[doc = "1: Clear enhanced RX FIFO content"]
    Clear = 1,
}
impl From<Erfclr> for bool {
    #[inline(always)]
    fn from(variant: Erfclr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERFCLR` reader - Enhanced RX FIFO Clear"]
pub type ErfclrR = crate::BitReader<Erfclr>;
impl ErfclrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Erfclr {
        match self.bits {
            false => Erfclr::NoEffect,
            true => Erfclr::Clear,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_no_effect(&self) -> bool {
        *self == Erfclr::NoEffect
    }
    #[doc = "Clear enhanced RX FIFO content"]
    #[inline(always)]
    pub fn is_clear(&self) -> bool {
        *self == Erfclr::Clear
    }
}
#[doc = "Field `ERFCLR` writer - Enhanced RX FIFO Clear"]
pub type ErfclrW<'a, REG> = crate::BitWriter<'a, REG, Erfclr>;
impl<'a, REG> ErfclrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn no_effect(self) -> &'a mut crate::W<REG> {
        self.variant(Erfclr::NoEffect)
    }
    #[doc = "Clear enhanced RX FIFO content"]
    #[inline(always)]
    pub fn clear(self) -> &'a mut crate::W<REG> {
        self.variant(Erfclr::Clear)
    }
}
#[doc = "Enhanced RX FIFO Data Available Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Erfda {
    #[doc = "0: No such occurrence"]
    NoMessageStored = 0,
    #[doc = "1: At least one message stored in Enhanced RX FIFO"]
    MessageStored = 1,
}
impl From<Erfda> for bool {
    #[inline(always)]
    fn from(variant: Erfda) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERFDA` reader - Enhanced RX FIFO Data Available Flag"]
pub type ErfdaR = crate::BitReader<Erfda>;
impl ErfdaR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Erfda {
        match self.bits {
            false => Erfda::NoMessageStored,
            true => Erfda::MessageStored,
        }
    }
    #[doc = "No such occurrence"]
    #[inline(always)]
    pub fn is_no_message_stored(&self) -> bool {
        *self == Erfda::NoMessageStored
    }
    #[doc = "At least one message stored in Enhanced RX FIFO"]
    #[inline(always)]
    pub fn is_message_stored(&self) -> bool {
        *self == Erfda::MessageStored
    }
}
#[doc = "Field `ERFDA` writer - Enhanced RX FIFO Data Available Flag"]
pub type ErfdaW<'a, REG> = crate::BitWriter1C<'a, REG, Erfda>;
impl<'a, REG> ErfdaW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No such occurrence"]
    #[inline(always)]
    pub fn no_message_stored(self) -> &'a mut crate::W<REG> {
        self.variant(Erfda::NoMessageStored)
    }
    #[doc = "At least one message stored in Enhanced RX FIFO"]
    #[inline(always)]
    pub fn message_stored(self) -> &'a mut crate::W<REG> {
        self.variant(Erfda::MessageStored)
    }
}
#[doc = "Enhanced RX FIFO Watermark Indication Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Erfwmi {
    #[doc = "0: No such occurrence"]
    WatermarkNo = 0,
    #[doc = "1: Number of messages in FIFO is greater than the watermark"]
    WatermarkYes = 1,
}
impl From<Erfwmi> for bool {
    #[inline(always)]
    fn from(variant: Erfwmi) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERFWMI` reader - Enhanced RX FIFO Watermark Indication Flag"]
pub type ErfwmiR = crate::BitReader<Erfwmi>;
impl ErfwmiR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Erfwmi {
        match self.bits {
            false => Erfwmi::WatermarkNo,
            true => Erfwmi::WatermarkYes,
        }
    }
    #[doc = "No such occurrence"]
    #[inline(always)]
    pub fn is_watermark_no(&self) -> bool {
        *self == Erfwmi::WatermarkNo
    }
    #[doc = "Number of messages in FIFO is greater than the watermark"]
    #[inline(always)]
    pub fn is_watermark_yes(&self) -> bool {
        *self == Erfwmi::WatermarkYes
    }
}
#[doc = "Field `ERFWMI` writer - Enhanced RX FIFO Watermark Indication Flag"]
pub type ErfwmiW<'a, REG> = crate::BitWriter1C<'a, REG, Erfwmi>;
impl<'a, REG> ErfwmiW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No such occurrence"]
    #[inline(always)]
    pub fn watermark_no(self) -> &'a mut crate::W<REG> {
        self.variant(Erfwmi::WatermarkNo)
    }
    #[doc = "Number of messages in FIFO is greater than the watermark"]
    #[inline(always)]
    pub fn watermark_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Erfwmi::WatermarkYes)
    }
}
#[doc = "Enhanced RX FIFO Overflow Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Erfovf {
    #[doc = "0: No such occurrence"]
    NoOverflow = 0,
    #[doc = "1: Overflow"]
    Overflow = 1,
}
impl From<Erfovf> for bool {
    #[inline(always)]
    fn from(variant: Erfovf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERFOVF` reader - Enhanced RX FIFO Overflow Flag"]
pub type ErfovfR = crate::BitReader<Erfovf>;
impl ErfovfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Erfovf {
        match self.bits {
            false => Erfovf::NoOverflow,
            true => Erfovf::Overflow,
        }
    }
    #[doc = "No such occurrence"]
    #[inline(always)]
    pub fn is_no_overflow(&self) -> bool {
        *self == Erfovf::NoOverflow
    }
    #[doc = "Overflow"]
    #[inline(always)]
    pub fn is_overflow(&self) -> bool {
        *self == Erfovf::Overflow
    }
}
#[doc = "Field `ERFOVF` writer - Enhanced RX FIFO Overflow Flag"]
pub type ErfovfW<'a, REG> = crate::BitWriter1C<'a, REG, Erfovf>;
impl<'a, REG> ErfovfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No such occurrence"]
    #[inline(always)]
    pub fn no_overflow(self) -> &'a mut crate::W<REG> {
        self.variant(Erfovf::NoOverflow)
    }
    #[doc = "Overflow"]
    #[inline(always)]
    pub fn overflow(self) -> &'a mut crate::W<REG> {
        self.variant(Erfovf::Overflow)
    }
}
#[doc = "Enhanced RX FIFO Underflow Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Erfufw {
    #[doc = "0: No such occurrence"]
    NoUnderflow = 0,
    #[doc = "1: Underflow"]
    Underflow = 1,
}
impl From<Erfufw> for bool {
    #[inline(always)]
    fn from(variant: Erfufw) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERFUFW` reader - Enhanced RX FIFO Underflow Flag"]
pub type ErfufwR = crate::BitReader<Erfufw>;
impl ErfufwR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Erfufw {
        match self.bits {
            false => Erfufw::NoUnderflow,
            true => Erfufw::Underflow,
        }
    }
    #[doc = "No such occurrence"]
    #[inline(always)]
    pub fn is_no_underflow(&self) -> bool {
        *self == Erfufw::NoUnderflow
    }
    #[doc = "Underflow"]
    #[inline(always)]
    pub fn is_underflow(&self) -> bool {
        *self == Erfufw::Underflow
    }
}
#[doc = "Field `ERFUFW` writer - Enhanced RX FIFO Underflow Flag"]
pub type ErfufwW<'a, REG> = crate::BitWriter1C<'a, REG, Erfufw>;
impl<'a, REG> ErfufwW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No such occurrence"]
    #[inline(always)]
    pub fn no_underflow(self) -> &'a mut crate::W<REG> {
        self.variant(Erfufw::NoUnderflow)
    }
    #[doc = "Underflow"]
    #[inline(always)]
    pub fn underflow(self) -> &'a mut crate::W<REG> {
        self.variant(Erfufw::Underflow)
    }
}
impl R {
    #[doc = "Bits 0:5 - Enhanced RX FIFO Elements"]
    #[inline(always)]
    pub fn erfel(&self) -> ErfelR {
        ErfelR::new((self.bits & 0x3f) as u8)
    }
    #[doc = "Bit 16 - Enhanced RX FIFO Full Flag"]
    #[inline(always)]
    pub fn erff(&self) -> ErffR {
        ErffR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Enhanced RX FIFO Empty Flag"]
    #[inline(always)]
    pub fn erfe(&self) -> ErfeR {
        ErfeR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 27 - Enhanced RX FIFO Clear"]
    #[inline(always)]
    pub fn erfclr(&self) -> ErfclrR {
        ErfclrR::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bit 28 - Enhanced RX FIFO Data Available Flag"]
    #[inline(always)]
    pub fn erfda(&self) -> ErfdaR {
        ErfdaR::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 29 - Enhanced RX FIFO Watermark Indication Flag"]
    #[inline(always)]
    pub fn erfwmi(&self) -> ErfwmiR {
        ErfwmiR::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - Enhanced RX FIFO Overflow Flag"]
    #[inline(always)]
    pub fn erfovf(&self) -> ErfovfR {
        ErfovfR::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Enhanced RX FIFO Underflow Flag"]
    #[inline(always)]
    pub fn erfufw(&self) -> ErfufwR {
        ErfufwR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 27 - Enhanced RX FIFO Clear"]
    #[inline(always)]
    pub fn erfclr(&mut self) -> ErfclrW<ErfsrSpec> {
        ErfclrW::new(self, 27)
    }
    #[doc = "Bit 28 - Enhanced RX FIFO Data Available Flag"]
    #[inline(always)]
    pub fn erfda(&mut self) -> ErfdaW<ErfsrSpec> {
        ErfdaW::new(self, 28)
    }
    #[doc = "Bit 29 - Enhanced RX FIFO Watermark Indication Flag"]
    #[inline(always)]
    pub fn erfwmi(&mut self) -> ErfwmiW<ErfsrSpec> {
        ErfwmiW::new(self, 29)
    }
    #[doc = "Bit 30 - Enhanced RX FIFO Overflow Flag"]
    #[inline(always)]
    pub fn erfovf(&mut self) -> ErfovfW<ErfsrSpec> {
        ErfovfW::new(self, 30)
    }
    #[doc = "Bit 31 - Enhanced RX FIFO Underflow Flag"]
    #[inline(always)]
    pub fn erfufw(&mut self) -> ErfufwW<ErfsrSpec> {
        ErfufwW::new(self, 31)
    }
}
#[doc = "Enhanced RX FIFO Status\n\nYou can [`read`](crate::Reg::read) this register and get [`erfsr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`erfsr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ErfsrSpec;
impl crate::RegisterSpec for ErfsrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`erfsr::R`](R) reader structure"]
impl crate::Readable for ErfsrSpec {}
#[doc = "`write(|w| ..)` method takes [`erfsr::W`](W) writer structure"]
impl crate::Writable for ErfsrSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0xf000_0000;
}
#[doc = "`reset()` method sets ERFSR to value 0"]
impl crate::Resettable for ErfsrSpec {}

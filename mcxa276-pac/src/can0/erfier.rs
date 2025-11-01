#[doc = "Register `ERFIER` reader"]
pub type R = crate::R<ErfierSpec>;
#[doc = "Register `ERFIER` writer"]
pub type W = crate::W<ErfierSpec>;
#[doc = "Enhanced RX FIFO Data Available Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Erfdaie {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Erfdaie> for bool {
    #[inline(always)]
    fn from(variant: Erfdaie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERFDAIE` reader - Enhanced RX FIFO Data Available Interrupt Enable"]
pub type ErfdaieR = crate::BitReader<Erfdaie>;
impl ErfdaieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Erfdaie {
        match self.bits {
            false => Erfdaie::Disable,
            true => Erfdaie::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Erfdaie::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Erfdaie::Enable
    }
}
#[doc = "Field `ERFDAIE` writer - Enhanced RX FIFO Data Available Interrupt Enable"]
pub type ErfdaieW<'a, REG> = crate::BitWriter<'a, REG, Erfdaie>;
impl<'a, REG> ErfdaieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Erfdaie::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Erfdaie::Enable)
    }
}
#[doc = "Enhanced RX FIFO Watermark Indication Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Erfwmiie {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Erfwmiie> for bool {
    #[inline(always)]
    fn from(variant: Erfwmiie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERFWMIIE` reader - Enhanced RX FIFO Watermark Indication Interrupt Enable"]
pub type ErfwmiieR = crate::BitReader<Erfwmiie>;
impl ErfwmiieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Erfwmiie {
        match self.bits {
            false => Erfwmiie::Disable,
            true => Erfwmiie::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Erfwmiie::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Erfwmiie::Enable
    }
}
#[doc = "Field `ERFWMIIE` writer - Enhanced RX FIFO Watermark Indication Interrupt Enable"]
pub type ErfwmiieW<'a, REG> = crate::BitWriter<'a, REG, Erfwmiie>;
impl<'a, REG> ErfwmiieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Erfwmiie::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Erfwmiie::Enable)
    }
}
#[doc = "Enhanced RX FIFO Overflow Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Erfovfie {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Erfovfie> for bool {
    #[inline(always)]
    fn from(variant: Erfovfie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERFOVFIE` reader - Enhanced RX FIFO Overflow Interrupt Enable"]
pub type ErfovfieR = crate::BitReader<Erfovfie>;
impl ErfovfieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Erfovfie {
        match self.bits {
            false => Erfovfie::Disable,
            true => Erfovfie::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Erfovfie::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Erfovfie::Enable
    }
}
#[doc = "Field `ERFOVFIE` writer - Enhanced RX FIFO Overflow Interrupt Enable"]
pub type ErfovfieW<'a, REG> = crate::BitWriter<'a, REG, Erfovfie>;
impl<'a, REG> ErfovfieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Erfovfie::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Erfovfie::Enable)
    }
}
#[doc = "Enhanced RX FIFO Underflow Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Erfufwie {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Erfufwie> for bool {
    #[inline(always)]
    fn from(variant: Erfufwie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERFUFWIE` reader - Enhanced RX FIFO Underflow Interrupt Enable"]
pub type ErfufwieR = crate::BitReader<Erfufwie>;
impl ErfufwieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Erfufwie {
        match self.bits {
            false => Erfufwie::Disable,
            true => Erfufwie::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Erfufwie::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Erfufwie::Enable
    }
}
#[doc = "Field `ERFUFWIE` writer - Enhanced RX FIFO Underflow Interrupt Enable"]
pub type ErfufwieW<'a, REG> = crate::BitWriter<'a, REG, Erfufwie>;
impl<'a, REG> ErfufwieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Erfufwie::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Erfufwie::Enable)
    }
}
impl R {
    #[doc = "Bit 28 - Enhanced RX FIFO Data Available Interrupt Enable"]
    #[inline(always)]
    pub fn erfdaie(&self) -> ErfdaieR {
        ErfdaieR::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 29 - Enhanced RX FIFO Watermark Indication Interrupt Enable"]
    #[inline(always)]
    pub fn erfwmiie(&self) -> ErfwmiieR {
        ErfwmiieR::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - Enhanced RX FIFO Overflow Interrupt Enable"]
    #[inline(always)]
    pub fn erfovfie(&self) -> ErfovfieR {
        ErfovfieR::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Enhanced RX FIFO Underflow Interrupt Enable"]
    #[inline(always)]
    pub fn erfufwie(&self) -> ErfufwieR {
        ErfufwieR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 28 - Enhanced RX FIFO Data Available Interrupt Enable"]
    #[inline(always)]
    pub fn erfdaie(&mut self) -> ErfdaieW<ErfierSpec> {
        ErfdaieW::new(self, 28)
    }
    #[doc = "Bit 29 - Enhanced RX FIFO Watermark Indication Interrupt Enable"]
    #[inline(always)]
    pub fn erfwmiie(&mut self) -> ErfwmiieW<ErfierSpec> {
        ErfwmiieW::new(self, 29)
    }
    #[doc = "Bit 30 - Enhanced RX FIFO Overflow Interrupt Enable"]
    #[inline(always)]
    pub fn erfovfie(&mut self) -> ErfovfieW<ErfierSpec> {
        ErfovfieW::new(self, 30)
    }
    #[doc = "Bit 31 - Enhanced RX FIFO Underflow Interrupt Enable"]
    #[inline(always)]
    pub fn erfufwie(&mut self) -> ErfufwieW<ErfierSpec> {
        ErfufwieW::new(self, 31)
    }
}
#[doc = "Enhanced RX FIFO Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`erfier::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`erfier::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ErfierSpec;
impl crate::RegisterSpec for ErfierSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`erfier::R`](R) reader structure"]
impl crate::Readable for ErfierSpec {}
#[doc = "`write(|w| ..)` method takes [`erfier::W`](W) writer structure"]
impl crate::Writable for ErfierSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets ERFIER to value 0"]
impl crate::Resettable for ErfierSpec {}

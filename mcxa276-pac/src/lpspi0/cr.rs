#[doc = "Register `CR` reader"]
pub type R = crate::R<CrSpec>;
#[doc = "Register `CR` writer"]
pub type W = crate::W<CrSpec>;
#[doc = "Module Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Men {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Men> for bool {
    #[inline(always)]
    fn from(variant: Men) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MEN` reader - Module Enable"]
pub type MenR = crate::BitReader<Men>;
impl MenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Men {
        match self.bits {
            false => Men::Disabled,
            true => Men::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Men::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Men::Enabled
    }
}
#[doc = "Field `MEN` writer - Module Enable"]
pub type MenW<'a, REG> = crate::BitWriter<'a, REG, Men>;
impl<'a, REG> MenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Men::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Men::Enabled)
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
#[doc = "Debug Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dbgen {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Dbgen> for bool {
    #[inline(always)]
    fn from(variant: Dbgen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DBGEN` reader - Debug Enable"]
pub type DbgenR = crate::BitReader<Dbgen>;
impl DbgenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dbgen {
        match self.bits {
            false => Dbgen::Disabled,
            true => Dbgen::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Dbgen::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Dbgen::Enabled
    }
}
#[doc = "Field `DBGEN` writer - Debug Enable"]
pub type DbgenW<'a, REG> = crate::BitWriter<'a, REG, Dbgen>;
impl<'a, REG> DbgenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Dbgen::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Dbgen::Enabled)
    }
}
#[doc = "Reset Transmit FIFO\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rtf {
    #[doc = "0: No effect"]
    NoEffect = 0,
    #[doc = "1: Reset"]
    TxfifoRst = 1,
}
impl From<Rtf> for bool {
    #[inline(always)]
    fn from(variant: Rtf) -> Self {
        variant as u8 != 0
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
    #[doc = "Reset"]
    #[inline(always)]
    pub fn txfifo_rst(self) -> &'a mut crate::W<REG> {
        self.variant(Rtf::TxfifoRst)
    }
}
#[doc = "Reset Receive FIFO\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rrf {
    #[doc = "0: No effect"]
    NoEffect = 0,
    #[doc = "1: Reset"]
    RxfifoRst = 1,
}
impl From<Rrf> for bool {
    #[inline(always)]
    fn from(variant: Rrf) -> Self {
        variant as u8 != 0
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
    #[doc = "Reset"]
    #[inline(always)]
    pub fn rxfifo_rst(self) -> &'a mut crate::W<REG> {
        self.variant(Rrf::RxfifoRst)
    }
}
impl R {
    #[doc = "Bit 0 - Module Enable"]
    #[inline(always)]
    pub fn men(&self) -> MenR {
        MenR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Software Reset"]
    #[inline(always)]
    pub fn rst(&self) -> RstR {
        RstR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 3 - Debug Enable"]
    #[inline(always)]
    pub fn dbgen(&self) -> DbgenR {
        DbgenR::new(((self.bits >> 3) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Module Enable"]
    #[inline(always)]
    pub fn men(&mut self) -> MenW<CrSpec> {
        MenW::new(self, 0)
    }
    #[doc = "Bit 1 - Software Reset"]
    #[inline(always)]
    pub fn rst(&mut self) -> RstW<CrSpec> {
        RstW::new(self, 1)
    }
    #[doc = "Bit 3 - Debug Enable"]
    #[inline(always)]
    pub fn dbgen(&mut self) -> DbgenW<CrSpec> {
        DbgenW::new(self, 3)
    }
    #[doc = "Bit 8 - Reset Transmit FIFO"]
    #[inline(always)]
    pub fn rtf(&mut self) -> RtfW<CrSpec> {
        RtfW::new(self, 8)
    }
    #[doc = "Bit 9 - Reset Receive FIFO"]
    #[inline(always)]
    pub fn rrf(&mut self) -> RrfW<CrSpec> {
        RrfW::new(self, 9)
    }
}
#[doc = "Control\n\nYou can [`read`](crate::Reg::read) this register and get [`cr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CrSpec;
impl crate::RegisterSpec for CrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`cr::R`](R) reader structure"]
impl crate::Readable for CrSpec {}
#[doc = "`write(|w| ..)` method takes [`cr::W`](W) writer structure"]
impl crate::Writable for CrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CR to value 0"]
impl crate::Resettable for CrSpec {}

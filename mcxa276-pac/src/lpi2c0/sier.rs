#[doc = "Register `SIER` reader"]
pub type R = crate::R<SierSpec>;
#[doc = "Register `SIER` writer"]
pub type W = crate::W<SierSpec>;
#[doc = "Transmit Data Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tdie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Tdie> for bool {
    #[inline(always)]
    fn from(variant: Tdie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TDIE` reader - Transmit Data Interrupt Enable"]
pub type TdieR = crate::BitReader<Tdie>;
impl TdieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tdie {
        match self.bits {
            false => Tdie::Disabled,
            true => Tdie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Tdie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Tdie::Enabled
    }
}
#[doc = "Field `TDIE` writer - Transmit Data Interrupt Enable"]
pub type TdieW<'a, REG> = crate::BitWriter<'a, REG, Tdie>;
impl<'a, REG> TdieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Tdie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Tdie::Enabled)
    }
}
#[doc = "Receive Data Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rdie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Rdie> for bool {
    #[inline(always)]
    fn from(variant: Rdie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RDIE` reader - Receive Data Interrupt Enable"]
pub type RdieR = crate::BitReader<Rdie>;
impl RdieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rdie {
        match self.bits {
            false => Rdie::Disabled,
            true => Rdie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Rdie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Rdie::Enabled
    }
}
#[doc = "Field `RDIE` writer - Receive Data Interrupt Enable"]
pub type RdieW<'a, REG> = crate::BitWriter<'a, REG, Rdie>;
impl<'a, REG> RdieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rdie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rdie::Enabled)
    }
}
#[doc = "Address Valid Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Avie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Avie> for bool {
    #[inline(always)]
    fn from(variant: Avie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `AVIE` reader - Address Valid Interrupt Enable"]
pub type AvieR = crate::BitReader<Avie>;
impl AvieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Avie {
        match self.bits {
            false => Avie::Disabled,
            true => Avie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Avie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Avie::Enabled
    }
}
#[doc = "Field `AVIE` writer - Address Valid Interrupt Enable"]
pub type AvieW<'a, REG> = crate::BitWriter<'a, REG, Avie>;
impl<'a, REG> AvieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Avie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Avie::Enabled)
    }
}
#[doc = "Transmit ACK Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Taie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Taie> for bool {
    #[inline(always)]
    fn from(variant: Taie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TAIE` reader - Transmit ACK Interrupt Enable"]
pub type TaieR = crate::BitReader<Taie>;
impl TaieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Taie {
        match self.bits {
            false => Taie::Disabled,
            true => Taie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Taie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Taie::Enabled
    }
}
#[doc = "Field `TAIE` writer - Transmit ACK Interrupt Enable"]
pub type TaieW<'a, REG> = crate::BitWriter<'a, REG, Taie>;
impl<'a, REG> TaieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Taie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Taie::Enabled)
    }
}
#[doc = "Repeated Start Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rsie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Rsie> for bool {
    #[inline(always)]
    fn from(variant: Rsie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RSIE` reader - Repeated Start Interrupt Enable"]
pub type RsieR = crate::BitReader<Rsie>;
impl RsieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rsie {
        match self.bits {
            false => Rsie::Disabled,
            true => Rsie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Rsie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Rsie::Enabled
    }
}
#[doc = "Field `RSIE` writer - Repeated Start Interrupt Enable"]
pub type RsieW<'a, REG> = crate::BitWriter<'a, REG, Rsie>;
impl<'a, REG> RsieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rsie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rsie::Enabled)
    }
}
#[doc = "Stop Detect Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sdie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Sdie> for bool {
    #[inline(always)]
    fn from(variant: Sdie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SDIE` reader - Stop Detect Interrupt Enable"]
pub type SdieR = crate::BitReader<Sdie>;
impl SdieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sdie {
        match self.bits {
            false => Sdie::Disabled,
            true => Sdie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Sdie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Sdie::Enabled
    }
}
#[doc = "Field `SDIE` writer - Stop Detect Interrupt Enable"]
pub type SdieW<'a, REG> = crate::BitWriter<'a, REG, Sdie>;
impl<'a, REG> SdieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Sdie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Sdie::Enabled)
    }
}
#[doc = "Bit Error Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Beie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Beie> for bool {
    #[inline(always)]
    fn from(variant: Beie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BEIE` reader - Bit Error Interrupt Enable"]
pub type BeieR = crate::BitReader<Beie>;
impl BeieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Beie {
        match self.bits {
            false => Beie::Disabled,
            true => Beie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Beie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Beie::Enabled
    }
}
#[doc = "Field `BEIE` writer - Bit Error Interrupt Enable"]
pub type BeieW<'a, REG> = crate::BitWriter<'a, REG, Beie>;
impl<'a, REG> BeieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Beie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Beie::Enabled)
    }
}
#[doc = "FIFO Error Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Feie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Feie> for bool {
    #[inline(always)]
    fn from(variant: Feie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FEIE` reader - FIFO Error Interrupt Enable"]
pub type FeieR = crate::BitReader<Feie>;
impl FeieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Feie {
        match self.bits {
            false => Feie::Disabled,
            true => Feie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Feie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Feie::Enabled
    }
}
#[doc = "Field `FEIE` writer - FIFO Error Interrupt Enable"]
pub type FeieW<'a, REG> = crate::BitWriter<'a, REG, Feie>;
impl<'a, REG> FeieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Feie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Feie::Enabled)
    }
}
#[doc = "Address Match 0 Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Am0ie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Am0ie> for bool {
    #[inline(always)]
    fn from(variant: Am0ie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `AM0IE` reader - Address Match 0 Interrupt Enable"]
pub type Am0ieR = crate::BitReader<Am0ie>;
impl Am0ieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Am0ie {
        match self.bits {
            false => Am0ie::Disabled,
            true => Am0ie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Am0ie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Am0ie::Enabled
    }
}
#[doc = "Field `AM0IE` writer - Address Match 0 Interrupt Enable"]
pub type Am0ieW<'a, REG> = crate::BitWriter<'a, REG, Am0ie>;
impl<'a, REG> Am0ieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Am0ie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Am0ie::Enabled)
    }
}
#[doc = "Address Match 1 Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Am1ie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Am1ie> for bool {
    #[inline(always)]
    fn from(variant: Am1ie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `AM1IE` reader - Address Match 1 Interrupt Enable"]
pub type Am1ieR = crate::BitReader<Am1ie>;
impl Am1ieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Am1ie {
        match self.bits {
            false => Am1ie::Disabled,
            true => Am1ie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Am1ie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Am1ie::Enabled
    }
}
#[doc = "Field `AM1IE` writer - Address Match 1 Interrupt Enable"]
pub type Am1ieW<'a, REG> = crate::BitWriter<'a, REG, Am1ie>;
impl<'a, REG> Am1ieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Am1ie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Am1ie::Enabled)
    }
}
#[doc = "General Call Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gcie {
    #[doc = "0: Disabled"]
    Disabled = 0,
    #[doc = "1: Enabled"]
    Enabled = 1,
}
impl From<Gcie> for bool {
    #[inline(always)]
    fn from(variant: Gcie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GCIE` reader - General Call Interrupt Enable"]
pub type GcieR = crate::BitReader<Gcie>;
impl GcieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gcie {
        match self.bits {
            false => Gcie::Disabled,
            true => Gcie::Enabled,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Gcie::Disabled
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Gcie::Enabled
    }
}
#[doc = "Field `GCIE` writer - General Call Interrupt Enable"]
pub type GcieW<'a, REG> = crate::BitWriter<'a, REG, Gcie>;
impl<'a, REG> GcieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Gcie::Disabled)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Gcie::Enabled)
    }
}
#[doc = "SMBus Alert Response Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sarie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Sarie> for bool {
    #[inline(always)]
    fn from(variant: Sarie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SARIE` reader - SMBus Alert Response Interrupt Enable"]
pub type SarieR = crate::BitReader<Sarie>;
impl SarieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sarie {
        match self.bits {
            false => Sarie::Disabled,
            true => Sarie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Sarie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Sarie::Enabled
    }
}
#[doc = "Field `SARIE` writer - SMBus Alert Response Interrupt Enable"]
pub type SarieW<'a, REG> = crate::BitWriter<'a, REG, Sarie>;
impl<'a, REG> SarieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Sarie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Sarie::Enabled)
    }
}
impl R {
    #[doc = "Bit 0 - Transmit Data Interrupt Enable"]
    #[inline(always)]
    pub fn tdie(&self) -> TdieR {
        TdieR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Receive Data Interrupt Enable"]
    #[inline(always)]
    pub fn rdie(&self) -> RdieR {
        RdieR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Address Valid Interrupt Enable"]
    #[inline(always)]
    pub fn avie(&self) -> AvieR {
        AvieR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Transmit ACK Interrupt Enable"]
    #[inline(always)]
    pub fn taie(&self) -> TaieR {
        TaieR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 8 - Repeated Start Interrupt Enable"]
    #[inline(always)]
    pub fn rsie(&self) -> RsieR {
        RsieR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Stop Detect Interrupt Enable"]
    #[inline(always)]
    pub fn sdie(&self) -> SdieR {
        SdieR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Bit Error Interrupt Enable"]
    #[inline(always)]
    pub fn beie(&self) -> BeieR {
        BeieR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - FIFO Error Interrupt Enable"]
    #[inline(always)]
    pub fn feie(&self) -> FeieR {
        FeieR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Address Match 0 Interrupt Enable"]
    #[inline(always)]
    pub fn am0ie(&self) -> Am0ieR {
        Am0ieR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Address Match 1 Interrupt Enable"]
    #[inline(always)]
    pub fn am1ie(&self) -> Am1ieR {
        Am1ieR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - General Call Interrupt Enable"]
    #[inline(always)]
    pub fn gcie(&self) -> GcieR {
        GcieR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - SMBus Alert Response Interrupt Enable"]
    #[inline(always)]
    pub fn sarie(&self) -> SarieR {
        SarieR::new(((self.bits >> 15) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Transmit Data Interrupt Enable"]
    #[inline(always)]
    pub fn tdie(&mut self) -> TdieW<SierSpec> {
        TdieW::new(self, 0)
    }
    #[doc = "Bit 1 - Receive Data Interrupt Enable"]
    #[inline(always)]
    pub fn rdie(&mut self) -> RdieW<SierSpec> {
        RdieW::new(self, 1)
    }
    #[doc = "Bit 2 - Address Valid Interrupt Enable"]
    #[inline(always)]
    pub fn avie(&mut self) -> AvieW<SierSpec> {
        AvieW::new(self, 2)
    }
    #[doc = "Bit 3 - Transmit ACK Interrupt Enable"]
    #[inline(always)]
    pub fn taie(&mut self) -> TaieW<SierSpec> {
        TaieW::new(self, 3)
    }
    #[doc = "Bit 8 - Repeated Start Interrupt Enable"]
    #[inline(always)]
    pub fn rsie(&mut self) -> RsieW<SierSpec> {
        RsieW::new(self, 8)
    }
    #[doc = "Bit 9 - Stop Detect Interrupt Enable"]
    #[inline(always)]
    pub fn sdie(&mut self) -> SdieW<SierSpec> {
        SdieW::new(self, 9)
    }
    #[doc = "Bit 10 - Bit Error Interrupt Enable"]
    #[inline(always)]
    pub fn beie(&mut self) -> BeieW<SierSpec> {
        BeieW::new(self, 10)
    }
    #[doc = "Bit 11 - FIFO Error Interrupt Enable"]
    #[inline(always)]
    pub fn feie(&mut self) -> FeieW<SierSpec> {
        FeieW::new(self, 11)
    }
    #[doc = "Bit 12 - Address Match 0 Interrupt Enable"]
    #[inline(always)]
    pub fn am0ie(&mut self) -> Am0ieW<SierSpec> {
        Am0ieW::new(self, 12)
    }
    #[doc = "Bit 13 - Address Match 1 Interrupt Enable"]
    #[inline(always)]
    pub fn am1ie(&mut self) -> Am1ieW<SierSpec> {
        Am1ieW::new(self, 13)
    }
    #[doc = "Bit 14 - General Call Interrupt Enable"]
    #[inline(always)]
    pub fn gcie(&mut self) -> GcieW<SierSpec> {
        GcieW::new(self, 14)
    }
    #[doc = "Bit 15 - SMBus Alert Response Interrupt Enable"]
    #[inline(always)]
    pub fn sarie(&mut self) -> SarieW<SierSpec> {
        SarieW::new(self, 15)
    }
}
#[doc = "Target Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`sier::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sier::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SierSpec;
impl crate::RegisterSpec for SierSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sier::R`](R) reader structure"]
impl crate::Readable for SierSpec {}
#[doc = "`write(|w| ..)` method takes [`sier::W`](W) writer structure"]
impl crate::Writable for SierSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SIER to value 0"]
impl crate::Resettable for SierSpec {}

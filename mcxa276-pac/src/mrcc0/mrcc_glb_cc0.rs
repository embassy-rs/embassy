#[doc = "Register `MRCC_GLB_CC0` reader"]
pub type R = crate::R<MrccGlbCc0Spec>;
#[doc = "Register `MRCC_GLB_CC0` writer"]
pub type W = crate::W<MrccGlbCc0Spec>;
#[doc = "INPUTMUX0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inputmux0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Inputmux0> for bool {
    #[inline(always)]
    fn from(variant: Inputmux0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INPUTMUX0` reader - INPUTMUX0"]
pub type Inputmux0R = crate::BitReader<Inputmux0>;
impl Inputmux0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Inputmux0 {
        match self.bits {
            false => Inputmux0::Disabled,
            true => Inputmux0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Inputmux0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Inputmux0::Enabled
    }
}
#[doc = "Field `INPUTMUX0` writer - INPUTMUX0"]
pub type Inputmux0W<'a, REG> = crate::BitWriter<'a, REG, Inputmux0>;
impl<'a, REG> Inputmux0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Inputmux0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Inputmux0::Enabled)
    }
}
#[doc = "I3C0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum I3c0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<I3c0> for bool {
    #[inline(always)]
    fn from(variant: I3c0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `I3C0` reader - I3C0"]
pub type I3c0R = crate::BitReader<I3c0>;
impl I3c0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> I3c0 {
        match self.bits {
            false => I3c0::Disabled,
            true => I3c0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == I3c0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == I3c0::Enabled
    }
}
#[doc = "Field `I3C0` writer - I3C0"]
pub type I3c0W<'a, REG> = crate::BitWriter<'a, REG, I3c0>;
impl<'a, REG> I3c0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(I3c0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(I3c0::Enabled)
    }
}
#[doc = "CTIMER0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ctimer0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Ctimer0> for bool {
    #[inline(always)]
    fn from(variant: Ctimer0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CTIMER0` reader - CTIMER0"]
pub type Ctimer0R = crate::BitReader<Ctimer0>;
impl Ctimer0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ctimer0 {
        match self.bits {
            false => Ctimer0::Disabled,
            true => Ctimer0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Ctimer0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Ctimer0::Enabled
    }
}
#[doc = "Field `CTIMER0` writer - CTIMER0"]
pub type Ctimer0W<'a, REG> = crate::BitWriter<'a, REG, Ctimer0>;
impl<'a, REG> Ctimer0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ctimer0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ctimer0::Enabled)
    }
}
#[doc = "CTIMER1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ctimer1 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Ctimer1> for bool {
    #[inline(always)]
    fn from(variant: Ctimer1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CTIMER1` reader - CTIMER1"]
pub type Ctimer1R = crate::BitReader<Ctimer1>;
impl Ctimer1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ctimer1 {
        match self.bits {
            false => Ctimer1::Disabled,
            true => Ctimer1::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Ctimer1::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Ctimer1::Enabled
    }
}
#[doc = "Field `CTIMER1` writer - CTIMER1"]
pub type Ctimer1W<'a, REG> = crate::BitWriter<'a, REG, Ctimer1>;
impl<'a, REG> Ctimer1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ctimer1::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ctimer1::Enabled)
    }
}
#[doc = "CTIMER2\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ctimer2 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Ctimer2> for bool {
    #[inline(always)]
    fn from(variant: Ctimer2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CTIMER2` reader - CTIMER2"]
pub type Ctimer2R = crate::BitReader<Ctimer2>;
impl Ctimer2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ctimer2 {
        match self.bits {
            false => Ctimer2::Disabled,
            true => Ctimer2::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Ctimer2::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Ctimer2::Enabled
    }
}
#[doc = "Field `CTIMER2` writer - CTIMER2"]
pub type Ctimer2W<'a, REG> = crate::BitWriter<'a, REG, Ctimer2>;
impl<'a, REG> Ctimer2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ctimer2::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ctimer2::Enabled)
    }
}
#[doc = "CTIMER3\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ctimer3 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Ctimer3> for bool {
    #[inline(always)]
    fn from(variant: Ctimer3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CTIMER3` reader - CTIMER3"]
pub type Ctimer3R = crate::BitReader<Ctimer3>;
impl Ctimer3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ctimer3 {
        match self.bits {
            false => Ctimer3::Disabled,
            true => Ctimer3::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Ctimer3::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Ctimer3::Enabled
    }
}
#[doc = "Field `CTIMER3` writer - CTIMER3"]
pub type Ctimer3W<'a, REG> = crate::BitWriter<'a, REG, Ctimer3>;
impl<'a, REG> Ctimer3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ctimer3::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ctimer3::Enabled)
    }
}
#[doc = "CTIMER4\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ctimer4 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Ctimer4> for bool {
    #[inline(always)]
    fn from(variant: Ctimer4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CTIMER4` reader - CTIMER4"]
pub type Ctimer4R = crate::BitReader<Ctimer4>;
impl Ctimer4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ctimer4 {
        match self.bits {
            false => Ctimer4::Disabled,
            true => Ctimer4::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Ctimer4::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Ctimer4::Enabled
    }
}
#[doc = "Field `CTIMER4` writer - CTIMER4"]
pub type Ctimer4W<'a, REG> = crate::BitWriter<'a, REG, Ctimer4>;
impl<'a, REG> Ctimer4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ctimer4::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ctimer4::Enabled)
    }
}
#[doc = "FREQME\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Freqme {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Freqme> for bool {
    #[inline(always)]
    fn from(variant: Freqme) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FREQME` reader - FREQME"]
pub type FreqmeR = crate::BitReader<Freqme>;
impl FreqmeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Freqme {
        match self.bits {
            false => Freqme::Disabled,
            true => Freqme::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Freqme::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Freqme::Enabled
    }
}
#[doc = "Field `FREQME` writer - FREQME"]
pub type FreqmeW<'a, REG> = crate::BitWriter<'a, REG, Freqme>;
impl<'a, REG> FreqmeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Freqme::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Freqme::Enabled)
    }
}
#[doc = "UTICK0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Utick0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Utick0> for bool {
    #[inline(always)]
    fn from(variant: Utick0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `UTICK0` reader - UTICK0"]
pub type Utick0R = crate::BitReader<Utick0>;
impl Utick0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Utick0 {
        match self.bits {
            false => Utick0::Disabled,
            true => Utick0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Utick0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Utick0::Enabled
    }
}
#[doc = "Field `UTICK0` writer - UTICK0"]
pub type Utick0W<'a, REG> = crate::BitWriter<'a, REG, Utick0>;
impl<'a, REG> Utick0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Utick0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Utick0::Enabled)
    }
}
#[doc = "WWDT0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wwdt0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Wwdt0> for bool {
    #[inline(always)]
    fn from(variant: Wwdt0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WWDT0` reader - WWDT0"]
pub type Wwdt0R = crate::BitReader<Wwdt0>;
impl Wwdt0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wwdt0 {
        match self.bits {
            false => Wwdt0::Disabled,
            true => Wwdt0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Wwdt0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Wwdt0::Enabled
    }
}
#[doc = "Field `WWDT0` writer - WWDT0"]
pub type Wwdt0W<'a, REG> = crate::BitWriter<'a, REG, Wwdt0>;
impl<'a, REG> Wwdt0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Wwdt0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Wwdt0::Enabled)
    }
}
#[doc = "SMARTDMA0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Smartdma0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Smartdma0> for bool {
    #[inline(always)]
    fn from(variant: Smartdma0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SMARTDMA0` reader - SMARTDMA0"]
pub type Smartdma0R = crate::BitReader<Smartdma0>;
impl Smartdma0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Smartdma0 {
        match self.bits {
            false => Smartdma0::Disabled,
            true => Smartdma0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Smartdma0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Smartdma0::Enabled
    }
}
#[doc = "Field `SMARTDMA0` writer - SMARTDMA0"]
pub type Smartdma0W<'a, REG> = crate::BitWriter<'a, REG, Smartdma0>;
impl<'a, REG> Smartdma0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Smartdma0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Smartdma0::Enabled)
    }
}
#[doc = "DMA0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dma0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Dma0> for bool {
    #[inline(always)]
    fn from(variant: Dma0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DMA0` reader - DMA0"]
pub type Dma0R = crate::BitReader<Dma0>;
impl Dma0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dma0 {
        match self.bits {
            false => Dma0::Disabled,
            true => Dma0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Dma0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Dma0::Enabled
    }
}
#[doc = "Field `DMA0` writer - DMA0"]
pub type Dma0W<'a, REG> = crate::BitWriter<'a, REG, Dma0>;
impl<'a, REG> Dma0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Dma0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Dma0::Enabled)
    }
}
#[doc = "AOI0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aoi0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Aoi0> for bool {
    #[inline(always)]
    fn from(variant: Aoi0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `AOI0` reader - AOI0"]
pub type Aoi0R = crate::BitReader<Aoi0>;
impl Aoi0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Aoi0 {
        match self.bits {
            false => Aoi0::Disabled,
            true => Aoi0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Aoi0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Aoi0::Enabled
    }
}
#[doc = "Field `AOI0` writer - AOI0"]
pub type Aoi0W<'a, REG> = crate::BitWriter<'a, REG, Aoi0>;
impl<'a, REG> Aoi0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Aoi0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Aoi0::Enabled)
    }
}
#[doc = "CRC0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Crc0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Crc0> for bool {
    #[inline(always)]
    fn from(variant: Crc0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CRC0` reader - CRC0"]
pub type Crc0R = crate::BitReader<Crc0>;
impl Crc0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Crc0 {
        match self.bits {
            false => Crc0::Disabled,
            true => Crc0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Crc0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Crc0::Enabled
    }
}
#[doc = "Field `CRC0` writer - CRC0"]
pub type Crc0W<'a, REG> = crate::BitWriter<'a, REG, Crc0>;
impl<'a, REG> Crc0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Crc0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Crc0::Enabled)
    }
}
#[doc = "EIM0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Eim0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Eim0> for bool {
    #[inline(always)]
    fn from(variant: Eim0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EIM0` reader - EIM0"]
pub type Eim0R = crate::BitReader<Eim0>;
impl Eim0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Eim0 {
        match self.bits {
            false => Eim0::Disabled,
            true => Eim0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Eim0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Eim0::Enabled
    }
}
#[doc = "Field `EIM0` writer - EIM0"]
pub type Eim0W<'a, REG> = crate::BitWriter<'a, REG, Eim0>;
impl<'a, REG> Eim0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Eim0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Eim0::Enabled)
    }
}
#[doc = "ERM0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Erm0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Erm0> for bool {
    #[inline(always)]
    fn from(variant: Erm0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERM0` reader - ERM0"]
pub type Erm0R = crate::BitReader<Erm0>;
impl Erm0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Erm0 {
        match self.bits {
            false => Erm0::Disabled,
            true => Erm0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Erm0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Erm0::Enabled
    }
}
#[doc = "Field `ERM0` writer - ERM0"]
pub type Erm0W<'a, REG> = crate::BitWriter<'a, REG, Erm0>;
impl<'a, REG> Erm0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Erm0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Erm0::Enabled)
    }
}
#[doc = "FMC\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fmc {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Fmc> for bool {
    #[inline(always)]
    fn from(variant: Fmc) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FMC` reader - FMC"]
pub type FmcR = crate::BitReader<Fmc>;
impl FmcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fmc {
        match self.bits {
            false => Fmc::Disabled,
            true => Fmc::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Fmc::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Fmc::Enabled
    }
}
#[doc = "Field `FMC` writer - FMC"]
pub type FmcW<'a, REG> = crate::BitWriter<'a, REG, Fmc>;
impl<'a, REG> FmcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Fmc::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Fmc::Enabled)
    }
}
#[doc = "AOI1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aoi1 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Aoi1> for bool {
    #[inline(always)]
    fn from(variant: Aoi1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `AOI1` reader - AOI1"]
pub type Aoi1R = crate::BitReader<Aoi1>;
impl Aoi1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Aoi1 {
        match self.bits {
            false => Aoi1::Disabled,
            true => Aoi1::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Aoi1::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Aoi1::Enabled
    }
}
#[doc = "Field `AOI1` writer - AOI1"]
pub type Aoi1W<'a, REG> = crate::BitWriter<'a, REG, Aoi1>;
impl<'a, REG> Aoi1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Aoi1::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Aoi1::Enabled)
    }
}
#[doc = "FLEXIO0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flexio0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Flexio0> for bool {
    #[inline(always)]
    fn from(variant: Flexio0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FLEXIO0` reader - FLEXIO0"]
pub type Flexio0R = crate::BitReader<Flexio0>;
impl Flexio0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Flexio0 {
        match self.bits {
            false => Flexio0::Disabled,
            true => Flexio0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Flexio0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Flexio0::Enabled
    }
}
#[doc = "Field `FLEXIO0` writer - FLEXIO0"]
pub type Flexio0W<'a, REG> = crate::BitWriter<'a, REG, Flexio0>;
impl<'a, REG> Flexio0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Flexio0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Flexio0::Enabled)
    }
}
#[doc = "LPI2C0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lpi2c0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Lpi2c0> for bool {
    #[inline(always)]
    fn from(variant: Lpi2c0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LPI2C0` reader - LPI2C0"]
pub type Lpi2c0R = crate::BitReader<Lpi2c0>;
impl Lpi2c0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lpi2c0 {
        match self.bits {
            false => Lpi2c0::Disabled,
            true => Lpi2c0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Lpi2c0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Lpi2c0::Enabled
    }
}
#[doc = "Field `LPI2C0` writer - LPI2C0"]
pub type Lpi2c0W<'a, REG> = crate::BitWriter<'a, REG, Lpi2c0>;
impl<'a, REG> Lpi2c0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpi2c0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpi2c0::Enabled)
    }
}
#[doc = "LPI2C1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lpi2c1 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Lpi2c1> for bool {
    #[inline(always)]
    fn from(variant: Lpi2c1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LPI2C1` reader - LPI2C1"]
pub type Lpi2c1R = crate::BitReader<Lpi2c1>;
impl Lpi2c1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lpi2c1 {
        match self.bits {
            false => Lpi2c1::Disabled,
            true => Lpi2c1::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Lpi2c1::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Lpi2c1::Enabled
    }
}
#[doc = "Field `LPI2C1` writer - LPI2C1"]
pub type Lpi2c1W<'a, REG> = crate::BitWriter<'a, REG, Lpi2c1>;
impl<'a, REG> Lpi2c1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpi2c1::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpi2c1::Enabled)
    }
}
#[doc = "LPSPI0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lpspi0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Lpspi0> for bool {
    #[inline(always)]
    fn from(variant: Lpspi0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LPSPI0` reader - LPSPI0"]
pub type Lpspi0R = crate::BitReader<Lpspi0>;
impl Lpspi0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lpspi0 {
        match self.bits {
            false => Lpspi0::Disabled,
            true => Lpspi0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Lpspi0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Lpspi0::Enabled
    }
}
#[doc = "Field `LPSPI0` writer - LPSPI0"]
pub type Lpspi0W<'a, REG> = crate::BitWriter<'a, REG, Lpspi0>;
impl<'a, REG> Lpspi0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpspi0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpspi0::Enabled)
    }
}
#[doc = "LPSPI1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lpspi1 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Lpspi1> for bool {
    #[inline(always)]
    fn from(variant: Lpspi1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LPSPI1` reader - LPSPI1"]
pub type Lpspi1R = crate::BitReader<Lpspi1>;
impl Lpspi1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lpspi1 {
        match self.bits {
            false => Lpspi1::Disabled,
            true => Lpspi1::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Lpspi1::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Lpspi1::Enabled
    }
}
#[doc = "Field `LPSPI1` writer - LPSPI1"]
pub type Lpspi1W<'a, REG> = crate::BitWriter<'a, REG, Lpspi1>;
impl<'a, REG> Lpspi1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpspi1::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpspi1::Enabled)
    }
}
#[doc = "LPUART0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lpuart0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Lpuart0> for bool {
    #[inline(always)]
    fn from(variant: Lpuart0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LPUART0` reader - LPUART0"]
pub type Lpuart0R = crate::BitReader<Lpuart0>;
impl Lpuart0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lpuart0 {
        match self.bits {
            false => Lpuart0::Disabled,
            true => Lpuart0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Lpuart0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Lpuart0::Enabled
    }
}
#[doc = "Field `LPUART0` writer - LPUART0"]
pub type Lpuart0W<'a, REG> = crate::BitWriter<'a, REG, Lpuart0>;
impl<'a, REG> Lpuart0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpuart0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpuart0::Enabled)
    }
}
#[doc = "LPUART1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lpuart1 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Lpuart1> for bool {
    #[inline(always)]
    fn from(variant: Lpuart1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LPUART1` reader - LPUART1"]
pub type Lpuart1R = crate::BitReader<Lpuart1>;
impl Lpuart1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lpuart1 {
        match self.bits {
            false => Lpuart1::Disabled,
            true => Lpuart1::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Lpuart1::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Lpuart1::Enabled
    }
}
#[doc = "Field `LPUART1` writer - LPUART1"]
pub type Lpuart1W<'a, REG> = crate::BitWriter<'a, REG, Lpuart1>;
impl<'a, REG> Lpuart1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpuart1::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpuart1::Enabled)
    }
}
#[doc = "LPUART2\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lpuart2 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Lpuart2> for bool {
    #[inline(always)]
    fn from(variant: Lpuart2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LPUART2` reader - LPUART2"]
pub type Lpuart2R = crate::BitReader<Lpuart2>;
impl Lpuart2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lpuart2 {
        match self.bits {
            false => Lpuart2::Disabled,
            true => Lpuart2::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Lpuart2::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Lpuart2::Enabled
    }
}
#[doc = "Field `LPUART2` writer - LPUART2"]
pub type Lpuart2W<'a, REG> = crate::BitWriter<'a, REG, Lpuart2>;
impl<'a, REG> Lpuart2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpuart2::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpuart2::Enabled)
    }
}
#[doc = "LPUART3\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lpuart3 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Lpuart3> for bool {
    #[inline(always)]
    fn from(variant: Lpuart3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LPUART3` reader - LPUART3"]
pub type Lpuart3R = crate::BitReader<Lpuart3>;
impl Lpuart3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lpuart3 {
        match self.bits {
            false => Lpuart3::Disabled,
            true => Lpuart3::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Lpuart3::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Lpuart3::Enabled
    }
}
#[doc = "Field `LPUART3` writer - LPUART3"]
pub type Lpuart3W<'a, REG> = crate::BitWriter<'a, REG, Lpuart3>;
impl<'a, REG> Lpuart3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpuart3::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpuart3::Enabled)
    }
}
#[doc = "LPUART4\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lpuart4 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Lpuart4> for bool {
    #[inline(always)]
    fn from(variant: Lpuart4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LPUART4` reader - LPUART4"]
pub type Lpuart4R = crate::BitReader<Lpuart4>;
impl Lpuart4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lpuart4 {
        match self.bits {
            false => Lpuart4::Disabled,
            true => Lpuart4::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Lpuart4::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Lpuart4::Enabled
    }
}
#[doc = "Field `LPUART4` writer - LPUART4"]
pub type Lpuart4W<'a, REG> = crate::BitWriter<'a, REG, Lpuart4>;
impl<'a, REG> Lpuart4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpuart4::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpuart4::Enabled)
    }
}
#[doc = "USB0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Usb0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Usb0> for bool {
    #[inline(always)]
    fn from(variant: Usb0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `USB0` reader - USB0"]
pub type Usb0R = crate::BitReader<Usb0>;
impl Usb0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Usb0 {
        match self.bits {
            false => Usb0::Disabled,
            true => Usb0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Usb0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Usb0::Enabled
    }
}
#[doc = "Field `USB0` writer - USB0"]
pub type Usb0W<'a, REG> = crate::BitWriter<'a, REG, Usb0>;
impl<'a, REG> Usb0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Usb0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Usb0::Enabled)
    }
}
#[doc = "QDC0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Qdc0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Qdc0> for bool {
    #[inline(always)]
    fn from(variant: Qdc0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `QDC0` reader - QDC0"]
pub type Qdc0R = crate::BitReader<Qdc0>;
impl Qdc0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Qdc0 {
        match self.bits {
            false => Qdc0::Disabled,
            true => Qdc0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Qdc0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Qdc0::Enabled
    }
}
#[doc = "Field `QDC0` writer - QDC0"]
pub type Qdc0W<'a, REG> = crate::BitWriter<'a, REG, Qdc0>;
impl<'a, REG> Qdc0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Qdc0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Qdc0::Enabled)
    }
}
#[doc = "QDC1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Qdc1 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Qdc1> for bool {
    #[inline(always)]
    fn from(variant: Qdc1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `QDC1` reader - QDC1"]
pub type Qdc1R = crate::BitReader<Qdc1>;
impl Qdc1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Qdc1 {
        match self.bits {
            false => Qdc1::Disabled,
            true => Qdc1::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Qdc1::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Qdc1::Enabled
    }
}
#[doc = "Field `QDC1` writer - QDC1"]
pub type Qdc1W<'a, REG> = crate::BitWriter<'a, REG, Qdc1>;
impl<'a, REG> Qdc1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Qdc1::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Qdc1::Enabled)
    }
}
#[doc = "FLEXPWM0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flexpwm0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Flexpwm0> for bool {
    #[inline(always)]
    fn from(variant: Flexpwm0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FLEXPWM0` reader - FLEXPWM0"]
pub type Flexpwm0R = crate::BitReader<Flexpwm0>;
impl Flexpwm0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Flexpwm0 {
        match self.bits {
            false => Flexpwm0::Disabled,
            true => Flexpwm0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Flexpwm0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Flexpwm0::Enabled
    }
}
#[doc = "Field `FLEXPWM0` writer - FLEXPWM0"]
pub type Flexpwm0W<'a, REG> = crate::BitWriter<'a, REG, Flexpwm0>;
impl<'a, REG> Flexpwm0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Flexpwm0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Flexpwm0::Enabled)
    }
}
impl R {
    #[doc = "Bit 0 - INPUTMUX0"]
    #[inline(always)]
    pub fn inputmux0(&self) -> Inputmux0R {
        Inputmux0R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - I3C0"]
    #[inline(always)]
    pub fn i3c0(&self) -> I3c0R {
        I3c0R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - CTIMER0"]
    #[inline(always)]
    pub fn ctimer0(&self) -> Ctimer0R {
        Ctimer0R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - CTIMER1"]
    #[inline(always)]
    pub fn ctimer1(&self) -> Ctimer1R {
        Ctimer1R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - CTIMER2"]
    #[inline(always)]
    pub fn ctimer2(&self) -> Ctimer2R {
        Ctimer2R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - CTIMER3"]
    #[inline(always)]
    pub fn ctimer3(&self) -> Ctimer3R {
        Ctimer3R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - CTIMER4"]
    #[inline(always)]
    pub fn ctimer4(&self) -> Ctimer4R {
        Ctimer4R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - FREQME"]
    #[inline(always)]
    pub fn freqme(&self) -> FreqmeR {
        FreqmeR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - UTICK0"]
    #[inline(always)]
    pub fn utick0(&self) -> Utick0R {
        Utick0R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - WWDT0"]
    #[inline(always)]
    pub fn wwdt0(&self) -> Wwdt0R {
        Wwdt0R::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - SMARTDMA0"]
    #[inline(always)]
    pub fn smartdma0(&self) -> Smartdma0R {
        Smartdma0R::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - DMA0"]
    #[inline(always)]
    pub fn dma0(&self) -> Dma0R {
        Dma0R::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - AOI0"]
    #[inline(always)]
    pub fn aoi0(&self) -> Aoi0R {
        Aoi0R::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - CRC0"]
    #[inline(always)]
    pub fn crc0(&self) -> Crc0R {
        Crc0R::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - EIM0"]
    #[inline(always)]
    pub fn eim0(&self) -> Eim0R {
        Eim0R::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - ERM0"]
    #[inline(always)]
    pub fn erm0(&self) -> Erm0R {
        Erm0R::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - FMC"]
    #[inline(always)]
    pub fn fmc(&self) -> FmcR {
        FmcR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - AOI1"]
    #[inline(always)]
    pub fn aoi1(&self) -> Aoi1R {
        Aoi1R::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - FLEXIO0"]
    #[inline(always)]
    pub fn flexio0(&self) -> Flexio0R {
        Flexio0R::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - LPI2C0"]
    #[inline(always)]
    pub fn lpi2c0(&self) -> Lpi2c0R {
        Lpi2c0R::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - LPI2C1"]
    #[inline(always)]
    pub fn lpi2c1(&self) -> Lpi2c1R {
        Lpi2c1R::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - LPSPI0"]
    #[inline(always)]
    pub fn lpspi0(&self) -> Lpspi0R {
        Lpspi0R::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - LPSPI1"]
    #[inline(always)]
    pub fn lpspi1(&self) -> Lpspi1R {
        Lpspi1R::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - LPUART0"]
    #[inline(always)]
    pub fn lpuart0(&self) -> Lpuart0R {
        Lpuart0R::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bit 24 - LPUART1"]
    #[inline(always)]
    pub fn lpuart1(&self) -> Lpuart1R {
        Lpuart1R::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - LPUART2"]
    #[inline(always)]
    pub fn lpuart2(&self) -> Lpuart2R {
        Lpuart2R::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - LPUART3"]
    #[inline(always)]
    pub fn lpuart3(&self) -> Lpuart3R {
        Lpuart3R::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - LPUART4"]
    #[inline(always)]
    pub fn lpuart4(&self) -> Lpuart4R {
        Lpuart4R::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bit 28 - USB0"]
    #[inline(always)]
    pub fn usb0(&self) -> Usb0R {
        Usb0R::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 29 - QDC0"]
    #[inline(always)]
    pub fn qdc0(&self) -> Qdc0R {
        Qdc0R::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - QDC1"]
    #[inline(always)]
    pub fn qdc1(&self) -> Qdc1R {
        Qdc1R::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - FLEXPWM0"]
    #[inline(always)]
    pub fn flexpwm0(&self) -> Flexpwm0R {
        Flexpwm0R::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - INPUTMUX0"]
    #[inline(always)]
    pub fn inputmux0(&mut self) -> Inputmux0W<MrccGlbCc0Spec> {
        Inputmux0W::new(self, 0)
    }
    #[doc = "Bit 1 - I3C0"]
    #[inline(always)]
    pub fn i3c0(&mut self) -> I3c0W<MrccGlbCc0Spec> {
        I3c0W::new(self, 1)
    }
    #[doc = "Bit 2 - CTIMER0"]
    #[inline(always)]
    pub fn ctimer0(&mut self) -> Ctimer0W<MrccGlbCc0Spec> {
        Ctimer0W::new(self, 2)
    }
    #[doc = "Bit 3 - CTIMER1"]
    #[inline(always)]
    pub fn ctimer1(&mut self) -> Ctimer1W<MrccGlbCc0Spec> {
        Ctimer1W::new(self, 3)
    }
    #[doc = "Bit 4 - CTIMER2"]
    #[inline(always)]
    pub fn ctimer2(&mut self) -> Ctimer2W<MrccGlbCc0Spec> {
        Ctimer2W::new(self, 4)
    }
    #[doc = "Bit 5 - CTIMER3"]
    #[inline(always)]
    pub fn ctimer3(&mut self) -> Ctimer3W<MrccGlbCc0Spec> {
        Ctimer3W::new(self, 5)
    }
    #[doc = "Bit 6 - CTIMER4"]
    #[inline(always)]
    pub fn ctimer4(&mut self) -> Ctimer4W<MrccGlbCc0Spec> {
        Ctimer4W::new(self, 6)
    }
    #[doc = "Bit 7 - FREQME"]
    #[inline(always)]
    pub fn freqme(&mut self) -> FreqmeW<MrccGlbCc0Spec> {
        FreqmeW::new(self, 7)
    }
    #[doc = "Bit 8 - UTICK0"]
    #[inline(always)]
    pub fn utick0(&mut self) -> Utick0W<MrccGlbCc0Spec> {
        Utick0W::new(self, 8)
    }
    #[doc = "Bit 9 - WWDT0"]
    #[inline(always)]
    pub fn wwdt0(&mut self) -> Wwdt0W<MrccGlbCc0Spec> {
        Wwdt0W::new(self, 9)
    }
    #[doc = "Bit 10 - SMARTDMA0"]
    #[inline(always)]
    pub fn smartdma0(&mut self) -> Smartdma0W<MrccGlbCc0Spec> {
        Smartdma0W::new(self, 10)
    }
    #[doc = "Bit 11 - DMA0"]
    #[inline(always)]
    pub fn dma0(&mut self) -> Dma0W<MrccGlbCc0Spec> {
        Dma0W::new(self, 11)
    }
    #[doc = "Bit 12 - AOI0"]
    #[inline(always)]
    pub fn aoi0(&mut self) -> Aoi0W<MrccGlbCc0Spec> {
        Aoi0W::new(self, 12)
    }
    #[doc = "Bit 13 - CRC0"]
    #[inline(always)]
    pub fn crc0(&mut self) -> Crc0W<MrccGlbCc0Spec> {
        Crc0W::new(self, 13)
    }
    #[doc = "Bit 14 - EIM0"]
    #[inline(always)]
    pub fn eim0(&mut self) -> Eim0W<MrccGlbCc0Spec> {
        Eim0W::new(self, 14)
    }
    #[doc = "Bit 15 - ERM0"]
    #[inline(always)]
    pub fn erm0(&mut self) -> Erm0W<MrccGlbCc0Spec> {
        Erm0W::new(self, 15)
    }
    #[doc = "Bit 16 - FMC"]
    #[inline(always)]
    pub fn fmc(&mut self) -> FmcW<MrccGlbCc0Spec> {
        FmcW::new(self, 16)
    }
    #[doc = "Bit 17 - AOI1"]
    #[inline(always)]
    pub fn aoi1(&mut self) -> Aoi1W<MrccGlbCc0Spec> {
        Aoi1W::new(self, 17)
    }
    #[doc = "Bit 18 - FLEXIO0"]
    #[inline(always)]
    pub fn flexio0(&mut self) -> Flexio0W<MrccGlbCc0Spec> {
        Flexio0W::new(self, 18)
    }
    #[doc = "Bit 19 - LPI2C0"]
    #[inline(always)]
    pub fn lpi2c0(&mut self) -> Lpi2c0W<MrccGlbCc0Spec> {
        Lpi2c0W::new(self, 19)
    }
    #[doc = "Bit 20 - LPI2C1"]
    #[inline(always)]
    pub fn lpi2c1(&mut self) -> Lpi2c1W<MrccGlbCc0Spec> {
        Lpi2c1W::new(self, 20)
    }
    #[doc = "Bit 21 - LPSPI0"]
    #[inline(always)]
    pub fn lpspi0(&mut self) -> Lpspi0W<MrccGlbCc0Spec> {
        Lpspi0W::new(self, 21)
    }
    #[doc = "Bit 22 - LPSPI1"]
    #[inline(always)]
    pub fn lpspi1(&mut self) -> Lpspi1W<MrccGlbCc0Spec> {
        Lpspi1W::new(self, 22)
    }
    #[doc = "Bit 23 - LPUART0"]
    #[inline(always)]
    pub fn lpuart0(&mut self) -> Lpuart0W<MrccGlbCc0Spec> {
        Lpuart0W::new(self, 23)
    }
    #[doc = "Bit 24 - LPUART1"]
    #[inline(always)]
    pub fn lpuart1(&mut self) -> Lpuart1W<MrccGlbCc0Spec> {
        Lpuart1W::new(self, 24)
    }
    #[doc = "Bit 25 - LPUART2"]
    #[inline(always)]
    pub fn lpuart2(&mut self) -> Lpuart2W<MrccGlbCc0Spec> {
        Lpuart2W::new(self, 25)
    }
    #[doc = "Bit 26 - LPUART3"]
    #[inline(always)]
    pub fn lpuart3(&mut self) -> Lpuart3W<MrccGlbCc0Spec> {
        Lpuart3W::new(self, 26)
    }
    #[doc = "Bit 27 - LPUART4"]
    #[inline(always)]
    pub fn lpuart4(&mut self) -> Lpuart4W<MrccGlbCc0Spec> {
        Lpuart4W::new(self, 27)
    }
    #[doc = "Bit 28 - USB0"]
    #[inline(always)]
    pub fn usb0(&mut self) -> Usb0W<MrccGlbCc0Spec> {
        Usb0W::new(self, 28)
    }
    #[doc = "Bit 29 - QDC0"]
    #[inline(always)]
    pub fn qdc0(&mut self) -> Qdc0W<MrccGlbCc0Spec> {
        Qdc0W::new(self, 29)
    }
    #[doc = "Bit 30 - QDC1"]
    #[inline(always)]
    pub fn qdc1(&mut self) -> Qdc1W<MrccGlbCc0Spec> {
        Qdc1W::new(self, 30)
    }
    #[doc = "Bit 31 - FLEXPWM0"]
    #[inline(always)]
    pub fn flexpwm0(&mut self) -> Flexpwm0W<MrccGlbCc0Spec> {
        Flexpwm0W::new(self, 31)
    }
}
#[doc = "AHB Clock Control 0\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_glb_cc0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_cc0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MrccGlbCc0Spec;
impl crate::RegisterSpec for MrccGlbCc0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mrcc_glb_cc0::R`](R) reader structure"]
impl crate::Readable for MrccGlbCc0Spec {}
#[doc = "`write(|w| ..)` method takes [`mrcc_glb_cc0::W`](W) writer structure"]
impl crate::Writable for MrccGlbCc0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MRCC_GLB_CC0 to value 0x0001_0000"]
impl crate::Resettable for MrccGlbCc0Spec {
    const RESET_VALUE: u32 = 0x0001_0000;
}

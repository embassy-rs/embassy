#[doc = "Register `SDER` reader"]
pub type R = crate::R<SderSpec>;
#[doc = "Register `SDER` writer"]
pub type W = crate::W<SderSpec>;
#[doc = "Transmit Data DMA Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tdde {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Tdde> for bool {
    #[inline(always)]
    fn from(variant: Tdde) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TDDE` reader - Transmit Data DMA Enable"]
pub type TddeR = crate::BitReader<Tdde>;
impl TddeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tdde {
        match self.bits {
            false => Tdde::Disabled,
            true => Tdde::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Tdde::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Tdde::Enabled
    }
}
#[doc = "Field `TDDE` writer - Transmit Data DMA Enable"]
pub type TddeW<'a, REG> = crate::BitWriter<'a, REG, Tdde>;
impl<'a, REG> TddeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Tdde::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Tdde::Enabled)
    }
}
#[doc = "Receive Data DMA Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rdde {
    #[doc = "0: Disable DMA request"]
    Disabled = 0,
    #[doc = "1: Enable DMA request"]
    Enabled = 1,
}
impl From<Rdde> for bool {
    #[inline(always)]
    fn from(variant: Rdde) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RDDE` reader - Receive Data DMA Enable"]
pub type RddeR = crate::BitReader<Rdde>;
impl RddeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rdde {
        match self.bits {
            false => Rdde::Disabled,
            true => Rdde::Enabled,
        }
    }
    #[doc = "Disable DMA request"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Rdde::Disabled
    }
    #[doc = "Enable DMA request"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Rdde::Enabled
    }
}
#[doc = "Field `RDDE` writer - Receive Data DMA Enable"]
pub type RddeW<'a, REG> = crate::BitWriter<'a, REG, Rdde>;
impl<'a, REG> RddeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable DMA request"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rdde::Disabled)
    }
    #[doc = "Enable DMA request"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rdde::Enabled)
    }
}
#[doc = "Address Valid DMA Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Avde {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Avde> for bool {
    #[inline(always)]
    fn from(variant: Avde) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `AVDE` reader - Address Valid DMA Enable"]
pub type AvdeR = crate::BitReader<Avde>;
impl AvdeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Avde {
        match self.bits {
            false => Avde::Disabled,
            true => Avde::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Avde::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Avde::Enabled
    }
}
#[doc = "Field `AVDE` writer - Address Valid DMA Enable"]
pub type AvdeW<'a, REG> = crate::BitWriter<'a, REG, Avde>;
impl<'a, REG> AvdeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Avde::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Avde::Enabled)
    }
}
#[doc = "Repeated Start DMA Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rsde {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Rsde> for bool {
    #[inline(always)]
    fn from(variant: Rsde) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RSDE` reader - Repeated Start DMA Enable"]
pub type RsdeR = crate::BitReader<Rsde>;
impl RsdeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rsde {
        match self.bits {
            false => Rsde::Disabled,
            true => Rsde::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Rsde::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Rsde::Enabled
    }
}
#[doc = "Field `RSDE` writer - Repeated Start DMA Enable"]
pub type RsdeW<'a, REG> = crate::BitWriter<'a, REG, Rsde>;
impl<'a, REG> RsdeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rsde::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rsde::Enabled)
    }
}
#[doc = "Stop Detect DMA Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sdde {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Sdde> for bool {
    #[inline(always)]
    fn from(variant: Sdde) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SDDE` reader - Stop Detect DMA Enable"]
pub type SddeR = crate::BitReader<Sdde>;
impl SddeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sdde {
        match self.bits {
            false => Sdde::Disabled,
            true => Sdde::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Sdde::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Sdde::Enabled
    }
}
#[doc = "Field `SDDE` writer - Stop Detect DMA Enable"]
pub type SddeW<'a, REG> = crate::BitWriter<'a, REG, Sdde>;
impl<'a, REG> SddeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Sdde::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Sdde::Enabled)
    }
}
impl R {
    #[doc = "Bit 0 - Transmit Data DMA Enable"]
    #[inline(always)]
    pub fn tdde(&self) -> TddeR {
        TddeR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Receive Data DMA Enable"]
    #[inline(always)]
    pub fn rdde(&self) -> RddeR {
        RddeR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Address Valid DMA Enable"]
    #[inline(always)]
    pub fn avde(&self) -> AvdeR {
        AvdeR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 8 - Repeated Start DMA Enable"]
    #[inline(always)]
    pub fn rsde(&self) -> RsdeR {
        RsdeR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Stop Detect DMA Enable"]
    #[inline(always)]
    pub fn sdde(&self) -> SddeR {
        SddeR::new(((self.bits >> 9) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Transmit Data DMA Enable"]
    #[inline(always)]
    pub fn tdde(&mut self) -> TddeW<SderSpec> {
        TddeW::new(self, 0)
    }
    #[doc = "Bit 1 - Receive Data DMA Enable"]
    #[inline(always)]
    pub fn rdde(&mut self) -> RddeW<SderSpec> {
        RddeW::new(self, 1)
    }
    #[doc = "Bit 2 - Address Valid DMA Enable"]
    #[inline(always)]
    pub fn avde(&mut self) -> AvdeW<SderSpec> {
        AvdeW::new(self, 2)
    }
    #[doc = "Bit 8 - Repeated Start DMA Enable"]
    #[inline(always)]
    pub fn rsde(&mut self) -> RsdeW<SderSpec> {
        RsdeW::new(self, 8)
    }
    #[doc = "Bit 9 - Stop Detect DMA Enable"]
    #[inline(always)]
    pub fn sdde(&mut self) -> SddeW<SderSpec> {
        SddeW::new(self, 9)
    }
}
#[doc = "Target DMA Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`sder::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sder::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SderSpec;
impl crate::RegisterSpec for SderSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sder::R`](R) reader structure"]
impl crate::Readable for SderSpec {}
#[doc = "`write(|w| ..)` method takes [`sder::W`](W) writer structure"]
impl crate::Writable for SderSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SDER to value 0"]
impl crate::Resettable for SderSpec {}

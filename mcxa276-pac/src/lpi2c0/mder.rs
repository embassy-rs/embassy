#[doc = "Register `MDER` reader"]
pub type R = crate::R<MderSpec>;
#[doc = "Register `MDER` writer"]
pub type W = crate::W<MderSpec>;
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
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
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
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Rdde::Disabled
    }
    #[doc = "Enable"]
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
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rdde::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rdde::Enabled)
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
}
impl W {
    #[doc = "Bit 0 - Transmit Data DMA Enable"]
    #[inline(always)]
    pub fn tdde(&mut self) -> TddeW<MderSpec> {
        TddeW::new(self, 0)
    }
    #[doc = "Bit 1 - Receive Data DMA Enable"]
    #[inline(always)]
    pub fn rdde(&mut self) -> RddeW<MderSpec> {
        RddeW::new(self, 1)
    }
}
#[doc = "Controller DMA Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`mder::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mder::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MderSpec;
impl crate::RegisterSpec for MderSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mder::R`](R) reader structure"]
impl crate::Readable for MderSpec {}
#[doc = "`write(|w| ..)` method takes [`mder::W`](W) writer structure"]
impl crate::Writable for MderSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MDER to value 0"]
impl crate::Resettable for MderSpec {}

#[doc = "Register `DER` reader"]
pub type R = crate::R<DerSpec>;
#[doc = "Register `DER` writer"]
pub type W = crate::W<DerSpec>;
#[doc = "Transmit Data DMA Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tdde {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
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
            false => Tdde::Disable,
            true => Tdde::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tdde::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tdde::Enable
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
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tdde::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tdde::Enable)
    }
}
#[doc = "Receive Data DMA Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rdde {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
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
            false => Rdde::Disable,
            true => Rdde::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Rdde::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Rdde::Enable
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
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Rdde::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Rdde::Enable)
    }
}
#[doc = "Frame Complete DMA Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fcde {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Fcde> for bool {
    #[inline(always)]
    fn from(variant: Fcde) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FCDE` reader - Frame Complete DMA Enable"]
pub type FcdeR = crate::BitReader<Fcde>;
impl FcdeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fcde {
        match self.bits {
            false => Fcde::Disable,
            true => Fcde::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Fcde::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Fcde::Enable
    }
}
#[doc = "Field `FCDE` writer - Frame Complete DMA Enable"]
pub type FcdeW<'a, REG> = crate::BitWriter<'a, REG, Fcde>;
impl<'a, REG> FcdeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Fcde::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Fcde::Enable)
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
    #[doc = "Bit 9 - Frame Complete DMA Enable"]
    #[inline(always)]
    pub fn fcde(&self) -> FcdeR {
        FcdeR::new(((self.bits >> 9) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Transmit Data DMA Enable"]
    #[inline(always)]
    pub fn tdde(&mut self) -> TddeW<DerSpec> {
        TddeW::new(self, 0)
    }
    #[doc = "Bit 1 - Receive Data DMA Enable"]
    #[inline(always)]
    pub fn rdde(&mut self) -> RddeW<DerSpec> {
        RddeW::new(self, 1)
    }
    #[doc = "Bit 9 - Frame Complete DMA Enable"]
    #[inline(always)]
    pub fn fcde(&mut self) -> FcdeW<DerSpec> {
        FcdeW::new(self, 9)
    }
}
#[doc = "DMA Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`der::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`der::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct DerSpec;
impl crate::RegisterSpec for DerSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`der::R`](R) reader structure"]
impl crate::Readable for DerSpec {}
#[doc = "`write(|w| ..)` method takes [`der::W`](W) writer structure"]
impl crate::Writable for DerSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets DER to value 0"]
impl crate::Resettable for DerSpec {}

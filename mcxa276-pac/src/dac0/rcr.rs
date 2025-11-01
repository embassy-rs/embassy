#[doc = "Register `RCR` reader"]
pub type R = crate::R<RcrSpec>;
#[doc = "Register `RCR` writer"]
pub type W = crate::W<RcrSpec>;
#[doc = "Software Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Swrst {
    #[doc = "0: No effect"]
    NoEffect = 0,
    #[doc = "1: Software reset"]
    SoftwareReset = 1,
}
impl From<Swrst> for bool {
    #[inline(always)]
    fn from(variant: Swrst) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SWRST` reader - Software Reset"]
pub type SwrstR = crate::BitReader<Swrst>;
impl SwrstR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Swrst {
        match self.bits {
            false => Swrst::NoEffect,
            true => Swrst::SoftwareReset,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_no_effect(&self) -> bool {
        *self == Swrst::NoEffect
    }
    #[doc = "Software reset"]
    #[inline(always)]
    pub fn is_software_reset(&self) -> bool {
        *self == Swrst::SoftwareReset
    }
}
#[doc = "Field `SWRST` writer - Software Reset"]
pub type SwrstW<'a, REG> = crate::BitWriter<'a, REG, Swrst>;
impl<'a, REG> SwrstW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn no_effect(self) -> &'a mut crate::W<REG> {
        self.variant(Swrst::NoEffect)
    }
    #[doc = "Software reset"]
    #[inline(always)]
    pub fn software_reset(self) -> &'a mut crate::W<REG> {
        self.variant(Swrst::SoftwareReset)
    }
}
#[doc = "FIFO Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fiforst {
    #[doc = "0: No effect"]
    NoEffect = 0,
    #[doc = "1: FIFO reset"]
    FifoReset = 1,
}
impl From<Fiforst> for bool {
    #[inline(always)]
    fn from(variant: Fiforst) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FIFORST` reader - FIFO Reset"]
pub type FiforstR = crate::BitReader<Fiforst>;
impl FiforstR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fiforst {
        match self.bits {
            false => Fiforst::NoEffect,
            true => Fiforst::FifoReset,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_no_effect(&self) -> bool {
        *self == Fiforst::NoEffect
    }
    #[doc = "FIFO reset"]
    #[inline(always)]
    pub fn is_fifo_reset(&self) -> bool {
        *self == Fiforst::FifoReset
    }
}
#[doc = "Field `FIFORST` writer - FIFO Reset"]
pub type FiforstW<'a, REG> = crate::BitWriter<'a, REG, Fiforst>;
impl<'a, REG> FiforstW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn no_effect(self) -> &'a mut crate::W<REG> {
        self.variant(Fiforst::NoEffect)
    }
    #[doc = "FIFO reset"]
    #[inline(always)]
    pub fn fifo_reset(self) -> &'a mut crate::W<REG> {
        self.variant(Fiforst::FifoReset)
    }
}
impl R {
    #[doc = "Bit 0 - Software Reset"]
    #[inline(always)]
    pub fn swrst(&self) -> SwrstR {
        SwrstR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - FIFO Reset"]
    #[inline(always)]
    pub fn fiforst(&self) -> FiforstR {
        FiforstR::new(((self.bits >> 1) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Software Reset"]
    #[inline(always)]
    pub fn swrst(&mut self) -> SwrstW<RcrSpec> {
        SwrstW::new(self, 0)
    }
    #[doc = "Bit 1 - FIFO Reset"]
    #[inline(always)]
    pub fn fiforst(&mut self) -> FiforstW<RcrSpec> {
        FiforstW::new(self, 1)
    }
}
#[doc = "Reset Control\n\nYou can [`read`](crate::Reg::read) this register and get [`rcr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rcr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RcrSpec;
impl crate::RegisterSpec for RcrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`rcr::R`](R) reader structure"]
impl crate::Readable for RcrSpec {}
#[doc = "`write(|w| ..)` method takes [`rcr::W`](W) writer structure"]
impl crate::Writable for RcrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RCR to value 0"]
impl crate::Resettable for RcrSpec {}

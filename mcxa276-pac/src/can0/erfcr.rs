#[doc = "Register `ERFCR` reader"]
pub type R = crate::R<ErfcrSpec>;
#[doc = "Register `ERFCR` writer"]
pub type W = crate::W<ErfcrSpec>;
#[doc = "Field `ERFWM` reader - Enhanced RX FIFO Watermark"]
pub type ErfwmR = crate::FieldReader;
#[doc = "Field `ERFWM` writer - Enhanced RX FIFO Watermark"]
pub type ErfwmW<'a, REG> = crate::FieldWriter<'a, REG, 5>;
#[doc = "Field `NFE` reader - Number of Enhanced RX FIFO Filter Elements"]
pub type NfeR = crate::FieldReader;
#[doc = "Field `NFE` writer - Number of Enhanced RX FIFO Filter Elements"]
pub type NfeW<'a, REG> = crate::FieldWriter<'a, REG, 6>;
#[doc = "Field `NEXIF` reader - Number of Extended ID Filter Elements"]
pub type NexifR = crate::FieldReader;
#[doc = "Field `NEXIF` writer - Number of Extended ID Filter Elements"]
pub type NexifW<'a, REG> = crate::FieldWriter<'a, REG, 7>;
#[doc = "Field `DMALW` reader - DMA Last Word"]
pub type DmalwR = crate::FieldReader;
#[doc = "Field `DMALW` writer - DMA Last Word"]
pub type DmalwW<'a, REG> = crate::FieldWriter<'a, REG, 5>;
#[doc = "Enhanced RX FIFO enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Erfen {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Erfen> for bool {
    #[inline(always)]
    fn from(variant: Erfen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERFEN` reader - Enhanced RX FIFO enable"]
pub type ErfenR = crate::BitReader<Erfen>;
impl ErfenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Erfen {
        match self.bits {
            false => Erfen::Disable,
            true => Erfen::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Erfen::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Erfen::Enable
    }
}
#[doc = "Field `ERFEN` writer - Enhanced RX FIFO enable"]
pub type ErfenW<'a, REG> = crate::BitWriter<'a, REG, Erfen>;
impl<'a, REG> ErfenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Erfen::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Erfen::Enable)
    }
}
impl R {
    #[doc = "Bits 0:4 - Enhanced RX FIFO Watermark"]
    #[inline(always)]
    pub fn erfwm(&self) -> ErfwmR {
        ErfwmR::new((self.bits & 0x1f) as u8)
    }
    #[doc = "Bits 8:13 - Number of Enhanced RX FIFO Filter Elements"]
    #[inline(always)]
    pub fn nfe(&self) -> NfeR {
        NfeR::new(((self.bits >> 8) & 0x3f) as u8)
    }
    #[doc = "Bits 16:22 - Number of Extended ID Filter Elements"]
    #[inline(always)]
    pub fn nexif(&self) -> NexifR {
        NexifR::new(((self.bits >> 16) & 0x7f) as u8)
    }
    #[doc = "Bits 26:30 - DMA Last Word"]
    #[inline(always)]
    pub fn dmalw(&self) -> DmalwR {
        DmalwR::new(((self.bits >> 26) & 0x1f) as u8)
    }
    #[doc = "Bit 31 - Enhanced RX FIFO enable"]
    #[inline(always)]
    pub fn erfen(&self) -> ErfenR {
        ErfenR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:4 - Enhanced RX FIFO Watermark"]
    #[inline(always)]
    pub fn erfwm(&mut self) -> ErfwmW<ErfcrSpec> {
        ErfwmW::new(self, 0)
    }
    #[doc = "Bits 8:13 - Number of Enhanced RX FIFO Filter Elements"]
    #[inline(always)]
    pub fn nfe(&mut self) -> NfeW<ErfcrSpec> {
        NfeW::new(self, 8)
    }
    #[doc = "Bits 16:22 - Number of Extended ID Filter Elements"]
    #[inline(always)]
    pub fn nexif(&mut self) -> NexifW<ErfcrSpec> {
        NexifW::new(self, 16)
    }
    #[doc = "Bits 26:30 - DMA Last Word"]
    #[inline(always)]
    pub fn dmalw(&mut self) -> DmalwW<ErfcrSpec> {
        DmalwW::new(self, 26)
    }
    #[doc = "Bit 31 - Enhanced RX FIFO enable"]
    #[inline(always)]
    pub fn erfen(&mut self) -> ErfenW<ErfcrSpec> {
        ErfenW::new(self, 31)
    }
}
#[doc = "Enhanced RX FIFO Control\n\nYou can [`read`](crate::Reg::read) this register and get [`erfcr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`erfcr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ErfcrSpec;
impl crate::RegisterSpec for ErfcrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`erfcr::R`](R) reader structure"]
impl crate::Readable for ErfcrSpec {}
#[doc = "`write(|w| ..)` method takes [`erfcr::W`](W) writer structure"]
impl crate::Writable for ErfcrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets ERFCR to value 0"]
impl crate::Resettable for ErfcrSpec {}

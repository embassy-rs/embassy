#[doc = "Register `DER` reader"]
pub type R = crate::R<DerSpec>;
#[doc = "Register `DER` writer"]
pub type W = crate::W<DerSpec>;
#[doc = "FIFO Empty DMA Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmptyDmaen {
    #[doc = "0: Disables"]
    Disabled = 0,
    #[doc = "1: Enables"]
    Enabled = 1,
}
impl From<EmptyDmaen> for bool {
    #[inline(always)]
    fn from(variant: EmptyDmaen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EMPTY_DMAEN` reader - FIFO Empty DMA Enable"]
pub type EmptyDmaenR = crate::BitReader<EmptyDmaen>;
impl EmptyDmaenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> EmptyDmaen {
        match self.bits {
            false => EmptyDmaen::Disabled,
            true => EmptyDmaen::Enabled,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == EmptyDmaen::Disabled
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == EmptyDmaen::Enabled
    }
}
#[doc = "Field `EMPTY_DMAEN` writer - FIFO Empty DMA Enable"]
pub type EmptyDmaenW<'a, REG> = crate::BitWriter<'a, REG, EmptyDmaen>;
impl<'a, REG> EmptyDmaenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(EmptyDmaen::Disabled)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(EmptyDmaen::Enabled)
    }
}
#[doc = "FIFO Watermark DMA Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WmDmaen {
    #[doc = "0: Disables"]
    Disabled = 0,
    #[doc = "1: Enables"]
    Enabled = 1,
}
impl From<WmDmaen> for bool {
    #[inline(always)]
    fn from(variant: WmDmaen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WM_DMAEN` reader - FIFO Watermark DMA Enable"]
pub type WmDmaenR = crate::BitReader<WmDmaen>;
impl WmDmaenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> WmDmaen {
        match self.bits {
            false => WmDmaen::Disabled,
            true => WmDmaen::Enabled,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == WmDmaen::Disabled
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == WmDmaen::Enabled
    }
}
#[doc = "Field `WM_DMAEN` writer - FIFO Watermark DMA Enable"]
pub type WmDmaenW<'a, REG> = crate::BitWriter<'a, REG, WmDmaen>;
impl<'a, REG> WmDmaenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(WmDmaen::Disabled)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(WmDmaen::Enabled)
    }
}
impl R {
    #[doc = "Bit 1 - FIFO Empty DMA Enable"]
    #[inline(always)]
    pub fn empty_dmaen(&self) -> EmptyDmaenR {
        EmptyDmaenR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - FIFO Watermark DMA Enable"]
    #[inline(always)]
    pub fn wm_dmaen(&self) -> WmDmaenR {
        WmDmaenR::new(((self.bits >> 2) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 1 - FIFO Empty DMA Enable"]
    #[inline(always)]
    pub fn empty_dmaen(&mut self) -> EmptyDmaenW<DerSpec> {
        EmptyDmaenW::new(self, 1)
    }
    #[doc = "Bit 2 - FIFO Watermark DMA Enable"]
    #[inline(always)]
    pub fn wm_dmaen(&mut self) -> WmDmaenW<DerSpec> {
        WmDmaenW::new(self, 2)
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

#[doc = "Register `SDMACTRL` reader"]
pub type R = crate::R<SdmactrlSpec>;
#[doc = "Register `SDMACTRL` writer"]
pub type W = crate::W<SdmactrlSpec>;
#[doc = "DMA Read (From-Bus) Trigger\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Dmafb {
    #[doc = "0: DMA not used"]
    NotUsed = 0,
    #[doc = "1: DMA enabled for one frame"]
    EnableOneFrame = 1,
    #[doc = "2: DMA enabled until turned off"]
    Enable = 2,
}
impl From<Dmafb> for u8 {
    #[inline(always)]
    fn from(variant: Dmafb) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Dmafb {
    type Ux = u8;
}
impl crate::IsEnum for Dmafb {}
#[doc = "Field `DMAFB` reader - DMA Read (From-Bus) Trigger"]
pub type DmafbR = crate::FieldReader<Dmafb>;
impl DmafbR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Dmafb> {
        match self.bits {
            0 => Some(Dmafb::NotUsed),
            1 => Some(Dmafb::EnableOneFrame),
            2 => Some(Dmafb::Enable),
            _ => None,
        }
    }
    #[doc = "DMA not used"]
    #[inline(always)]
    pub fn is_not_used(&self) -> bool {
        *self == Dmafb::NotUsed
    }
    #[doc = "DMA enabled for one frame"]
    #[inline(always)]
    pub fn is_enable_one_frame(&self) -> bool {
        *self == Dmafb::EnableOneFrame
    }
    #[doc = "DMA enabled until turned off"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Dmafb::Enable
    }
}
#[doc = "Field `DMAFB` writer - DMA Read (From-Bus) Trigger"]
pub type DmafbW<'a, REG> = crate::FieldWriter<'a, REG, 2, Dmafb>;
impl<'a, REG> DmafbW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "DMA not used"]
    #[inline(always)]
    pub fn not_used(self) -> &'a mut crate::W<REG> {
        self.variant(Dmafb::NotUsed)
    }
    #[doc = "DMA enabled for one frame"]
    #[inline(always)]
    pub fn enable_one_frame(self) -> &'a mut crate::W<REG> {
        self.variant(Dmafb::EnableOneFrame)
    }
    #[doc = "DMA enabled until turned off"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Dmafb::Enable)
    }
}
#[doc = "DMA Write (To-Bus) Trigger\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Dmatb {
    #[doc = "0: DMA not used"]
    NotUsed = 0,
    #[doc = "1: DMA enabled for one frame"]
    EnableOneFrame = 1,
    #[doc = "2: DMA enabled until turned off"]
    Enable = 2,
}
impl From<Dmatb> for u8 {
    #[inline(always)]
    fn from(variant: Dmatb) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Dmatb {
    type Ux = u8;
}
impl crate::IsEnum for Dmatb {}
#[doc = "Field `DMATB` reader - DMA Write (To-Bus) Trigger"]
pub type DmatbR = crate::FieldReader<Dmatb>;
impl DmatbR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Dmatb> {
        match self.bits {
            0 => Some(Dmatb::NotUsed),
            1 => Some(Dmatb::EnableOneFrame),
            2 => Some(Dmatb::Enable),
            _ => None,
        }
    }
    #[doc = "DMA not used"]
    #[inline(always)]
    pub fn is_not_used(&self) -> bool {
        *self == Dmatb::NotUsed
    }
    #[doc = "DMA enabled for one frame"]
    #[inline(always)]
    pub fn is_enable_one_frame(&self) -> bool {
        *self == Dmatb::EnableOneFrame
    }
    #[doc = "DMA enabled until turned off"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Dmatb::Enable
    }
}
#[doc = "Field `DMATB` writer - DMA Write (To-Bus) Trigger"]
pub type DmatbW<'a, REG> = crate::FieldWriter<'a, REG, 2, Dmatb>;
impl<'a, REG> DmatbW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "DMA not used"]
    #[inline(always)]
    pub fn not_used(self) -> &'a mut crate::W<REG> {
        self.variant(Dmatb::NotUsed)
    }
    #[doc = "DMA enabled for one frame"]
    #[inline(always)]
    pub fn enable_one_frame(self) -> &'a mut crate::W<REG> {
        self.variant(Dmatb::EnableOneFrame)
    }
    #[doc = "DMA enabled until turned off"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Dmatb::Enable)
    }
}
#[doc = "Width of DMA Operations\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Dmawidth {
    #[doc = "0: Byte"]
    Byte0 = 0,
    #[doc = "1: Byte"]
    Byte1 = 1,
    #[doc = "2: Halfword (16 bits) (this value ensures that two bytes are available in the FIFO)"]
    HalfWord = 2,
}
impl From<Dmawidth> for u8 {
    #[inline(always)]
    fn from(variant: Dmawidth) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Dmawidth {
    type Ux = u8;
}
impl crate::IsEnum for Dmawidth {}
#[doc = "Field `DMAWIDTH` reader - Width of DMA Operations"]
pub type DmawidthR = crate::FieldReader<Dmawidth>;
impl DmawidthR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Dmawidth> {
        match self.bits {
            0 => Some(Dmawidth::Byte0),
            1 => Some(Dmawidth::Byte1),
            2 => Some(Dmawidth::HalfWord),
            _ => None,
        }
    }
    #[doc = "Byte"]
    #[inline(always)]
    pub fn is_byte_0(&self) -> bool {
        *self == Dmawidth::Byte0
    }
    #[doc = "Byte"]
    #[inline(always)]
    pub fn is_byte_1(&self) -> bool {
        *self == Dmawidth::Byte1
    }
    #[doc = "Halfword (16 bits) (this value ensures that two bytes are available in the FIFO)"]
    #[inline(always)]
    pub fn is_half_word(&self) -> bool {
        *self == Dmawidth::HalfWord
    }
}
#[doc = "Field `DMAWIDTH` writer - Width of DMA Operations"]
pub type DmawidthW<'a, REG> = crate::FieldWriter<'a, REG, 2, Dmawidth>;
impl<'a, REG> DmawidthW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Byte"]
    #[inline(always)]
    pub fn byte_0(self) -> &'a mut crate::W<REG> {
        self.variant(Dmawidth::Byte0)
    }
    #[doc = "Byte"]
    #[inline(always)]
    pub fn byte_1(self) -> &'a mut crate::W<REG> {
        self.variant(Dmawidth::Byte1)
    }
    #[doc = "Halfword (16 bits) (this value ensures that two bytes are available in the FIFO)"]
    #[inline(always)]
    pub fn half_word(self) -> &'a mut crate::W<REG> {
        self.variant(Dmawidth::HalfWord)
    }
}
impl R {
    #[doc = "Bits 0:1 - DMA Read (From-Bus) Trigger"]
    #[inline(always)]
    pub fn dmafb(&self) -> DmafbR {
        DmafbR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 2:3 - DMA Write (To-Bus) Trigger"]
    #[inline(always)]
    pub fn dmatb(&self) -> DmatbR {
        DmatbR::new(((self.bits >> 2) & 3) as u8)
    }
    #[doc = "Bits 4:5 - Width of DMA Operations"]
    #[inline(always)]
    pub fn dmawidth(&self) -> DmawidthR {
        DmawidthR::new(((self.bits >> 4) & 3) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - DMA Read (From-Bus) Trigger"]
    #[inline(always)]
    pub fn dmafb(&mut self) -> DmafbW<SdmactrlSpec> {
        DmafbW::new(self, 0)
    }
    #[doc = "Bits 2:3 - DMA Write (To-Bus) Trigger"]
    #[inline(always)]
    pub fn dmatb(&mut self) -> DmatbW<SdmactrlSpec> {
        DmatbW::new(self, 2)
    }
    #[doc = "Bits 4:5 - Width of DMA Operations"]
    #[inline(always)]
    pub fn dmawidth(&mut self) -> DmawidthW<SdmactrlSpec> {
        DmawidthW::new(self, 4)
    }
}
#[doc = "Target DMA Control\n\nYou can [`read`](crate::Reg::read) this register and get [`sdmactrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sdmactrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SdmactrlSpec;
impl crate::RegisterSpec for SdmactrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sdmactrl::R`](R) reader structure"]
impl crate::Readable for SdmactrlSpec {}
#[doc = "`write(|w| ..)` method takes [`sdmactrl::W`](W) writer structure"]
impl crate::Writable for SdmactrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SDMACTRL to value 0x10"]
impl crate::Resettable for SdmactrlSpec {
    const RESET_VALUE: u32 = 0x10;
}

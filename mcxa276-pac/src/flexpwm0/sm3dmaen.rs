#[doc = "Register `SM3DMAEN` reader"]
pub type R = crate::R<Sm3dmaenSpec>;
#[doc = "Register `SM3DMAEN` writer"]
pub type W = crate::W<Sm3dmaenSpec>;
#[doc = "Field `CX0DE` reader - Capture X0 FIFO DMA Enable"]
pub type Cx0deR = crate::BitReader;
#[doc = "Field `CX0DE` writer - Capture X0 FIFO DMA Enable"]
pub type Cx0deW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CX1DE` reader - Capture X1 FIFO DMA Enable"]
pub type Cx1deR = crate::BitReader;
#[doc = "Field `CX1DE` writer - Capture X1 FIFO DMA Enable"]
pub type Cx1deW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Capture DMA Enable Source Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Captde {
    #[doc = "0: Read DMA requests disabled."]
    Disabled = 0,
    #[doc = "1: Exceeding a FIFO watermark sets the DMA read request. This requires at least one of DMAEN\\[CA1DE\\], DMAEN\\[CA0DE\\], DMAEN\\[CB1DE\\], DMAEN\\[CB0DE\\], DMAEN\\[CX1DE\\], or DMAEN\\[CX0DE\\] to be set to determine which watermark(s) the DMA request is sensitive."]
    Exceedfifo = 1,
    #[doc = "2: A local synchronization (VAL1 matches counter) sets the read DMA request."]
    LocalSync = 2,
    #[doc = "3: A local reload (STS\\[RF\\] being set) sets the read DMA request."]
    LocalReload = 3,
}
impl From<Captde> for u8 {
    #[inline(always)]
    fn from(variant: Captde) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Captde {
    type Ux = u8;
}
impl crate::IsEnum for Captde {}
#[doc = "Field `CAPTDE` reader - Capture DMA Enable Source Select"]
pub type CaptdeR = crate::FieldReader<Captde>;
impl CaptdeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Captde {
        match self.bits {
            0 => Captde::Disabled,
            1 => Captde::Exceedfifo,
            2 => Captde::LocalSync,
            3 => Captde::LocalReload,
            _ => unreachable!(),
        }
    }
    #[doc = "Read DMA requests disabled."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Captde::Disabled
    }
    #[doc = "Exceeding a FIFO watermark sets the DMA read request. This requires at least one of DMAEN\\[CA1DE\\], DMAEN\\[CA0DE\\], DMAEN\\[CB1DE\\], DMAEN\\[CB0DE\\], DMAEN\\[CX1DE\\], or DMAEN\\[CX0DE\\] to be set to determine which watermark(s) the DMA request is sensitive."]
    #[inline(always)]
    pub fn is_exceedfifo(&self) -> bool {
        *self == Captde::Exceedfifo
    }
    #[doc = "A local synchronization (VAL1 matches counter) sets the read DMA request."]
    #[inline(always)]
    pub fn is_local_sync(&self) -> bool {
        *self == Captde::LocalSync
    }
    #[doc = "A local reload (STS\\[RF\\] being set) sets the read DMA request."]
    #[inline(always)]
    pub fn is_local_reload(&self) -> bool {
        *self == Captde::LocalReload
    }
}
#[doc = "Field `CAPTDE` writer - Capture DMA Enable Source Select"]
pub type CaptdeW<'a, REG> = crate::FieldWriter<'a, REG, 2, Captde, crate::Safe>;
impl<'a, REG> CaptdeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Read DMA requests disabled."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Captde::Disabled)
    }
    #[doc = "Exceeding a FIFO watermark sets the DMA read request. This requires at least one of DMAEN\\[CA1DE\\], DMAEN\\[CA0DE\\], DMAEN\\[CB1DE\\], DMAEN\\[CB0DE\\], DMAEN\\[CX1DE\\], or DMAEN\\[CX0DE\\] to be set to determine which watermark(s) the DMA request is sensitive."]
    #[inline(always)]
    pub fn exceedfifo(self) -> &'a mut crate::W<REG> {
        self.variant(Captde::Exceedfifo)
    }
    #[doc = "A local synchronization (VAL1 matches counter) sets the read DMA request."]
    #[inline(always)]
    pub fn local_sync(self) -> &'a mut crate::W<REG> {
        self.variant(Captde::LocalSync)
    }
    #[doc = "A local reload (STS\\[RF\\] being set) sets the read DMA request."]
    #[inline(always)]
    pub fn local_reload(self) -> &'a mut crate::W<REG> {
        self.variant(Captde::LocalReload)
    }
}
#[doc = "FIFO Watermark AND Control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fand {
    #[doc = "0: Selected FIFO watermarks are OR'ed together."]
    Or = 0,
    #[doc = "1: Selected FIFO watermarks are AND'ed together."]
    And = 1,
}
impl From<Fand> for bool {
    #[inline(always)]
    fn from(variant: Fand) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FAND` reader - FIFO Watermark AND Control"]
pub type FandR = crate::BitReader<Fand>;
impl FandR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fand {
        match self.bits {
            false => Fand::Or,
            true => Fand::And,
        }
    }
    #[doc = "Selected FIFO watermarks are OR'ed together."]
    #[inline(always)]
    pub fn is_or(&self) -> bool {
        *self == Fand::Or
    }
    #[doc = "Selected FIFO watermarks are AND'ed together."]
    #[inline(always)]
    pub fn is_and(&self) -> bool {
        *self == Fand::And
    }
}
#[doc = "Field `FAND` writer - FIFO Watermark AND Control"]
pub type FandW<'a, REG> = crate::BitWriter<'a, REG, Fand>;
impl<'a, REG> FandW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Selected FIFO watermarks are OR'ed together."]
    #[inline(always)]
    pub fn or(self) -> &'a mut crate::W<REG> {
        self.variant(Fand::Or)
    }
    #[doc = "Selected FIFO watermarks are AND'ed together."]
    #[inline(always)]
    pub fn and(self) -> &'a mut crate::W<REG> {
        self.variant(Fand::And)
    }
}
#[doc = "Value Registers DMA Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Valde {
    #[doc = "0: DMA write requests disabled"]
    Disabled = 0,
    #[doc = "1: Enabled"]
    Enabled = 1,
}
impl From<Valde> for bool {
    #[inline(always)]
    fn from(variant: Valde) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `VALDE` reader - Value Registers DMA Enable"]
pub type ValdeR = crate::BitReader<Valde>;
impl ValdeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Valde {
        match self.bits {
            false => Valde::Disabled,
            true => Valde::Enabled,
        }
    }
    #[doc = "DMA write requests disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Valde::Disabled
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Valde::Enabled
    }
}
#[doc = "Field `VALDE` writer - Value Registers DMA Enable"]
pub type ValdeW<'a, REG> = crate::BitWriter<'a, REG, Valde>;
impl<'a, REG> ValdeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "DMA write requests disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Valde::Disabled)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Valde::Enabled)
    }
}
impl R {
    #[doc = "Bit 0 - Capture X0 FIFO DMA Enable"]
    #[inline(always)]
    pub fn cx0de(&self) -> Cx0deR {
        Cx0deR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Capture X1 FIFO DMA Enable"]
    #[inline(always)]
    pub fn cx1de(&self) -> Cx1deR {
        Cx1deR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bits 6:7 - Capture DMA Enable Source Select"]
    #[inline(always)]
    pub fn captde(&self) -> CaptdeR {
        CaptdeR::new(((self.bits >> 6) & 3) as u8)
    }
    #[doc = "Bit 8 - FIFO Watermark AND Control"]
    #[inline(always)]
    pub fn fand(&self) -> FandR {
        FandR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Value Registers DMA Enable"]
    #[inline(always)]
    pub fn valde(&self) -> ValdeR {
        ValdeR::new(((self.bits >> 9) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Capture X0 FIFO DMA Enable"]
    #[inline(always)]
    pub fn cx0de(&mut self) -> Cx0deW<Sm3dmaenSpec> {
        Cx0deW::new(self, 0)
    }
    #[doc = "Bit 1 - Capture X1 FIFO DMA Enable"]
    #[inline(always)]
    pub fn cx1de(&mut self) -> Cx1deW<Sm3dmaenSpec> {
        Cx1deW::new(self, 1)
    }
    #[doc = "Bits 6:7 - Capture DMA Enable Source Select"]
    #[inline(always)]
    pub fn captde(&mut self) -> CaptdeW<Sm3dmaenSpec> {
        CaptdeW::new(self, 6)
    }
    #[doc = "Bit 8 - FIFO Watermark AND Control"]
    #[inline(always)]
    pub fn fand(&mut self) -> FandW<Sm3dmaenSpec> {
        FandW::new(self, 8)
    }
    #[doc = "Bit 9 - Value Registers DMA Enable"]
    #[inline(always)]
    pub fn valde(&mut self) -> ValdeW<Sm3dmaenSpec> {
        ValdeW::new(self, 9)
    }
}
#[doc = "DMA Enable Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3dmaen::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm3dmaen::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm3dmaenSpec;
impl crate::RegisterSpec for Sm3dmaenSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm3dmaen::R`](R) reader structure"]
impl crate::Readable for Sm3dmaenSpec {}
#[doc = "`write(|w| ..)` method takes [`sm3dmaen::W`](W) writer structure"]
impl crate::Writable for Sm3dmaenSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SM3DMAEN to value 0"]
impl crate::Resettable for Sm3dmaenSpec {}

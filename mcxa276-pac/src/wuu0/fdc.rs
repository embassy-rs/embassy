#[doc = "Register `FDC` reader"]
pub type R = crate::R<FdcSpec>;
#[doc = "Register `FDC` writer"]
pub type W = crate::W<FdcSpec>;
#[doc = "Filter Configuration for FILTn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Filtc1 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
}
impl From<Filtc1> for u8 {
    #[inline(always)]
    fn from(variant: Filtc1) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Filtc1 {
    type Ux = u8;
}
impl crate::IsEnum for Filtc1 {}
#[doc = "Field `FILTC1` reader - Filter Configuration for FILTn"]
pub type Filtc1R = crate::FieldReader<Filtc1>;
impl Filtc1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Filtc1> {
        match self.bits {
            0 => Some(Filtc1::Interrupt),
            1 => Some(Filtc1::DmaReq),
            2 => Some(Filtc1::Trigger),
            _ => None,
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Filtc1::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Filtc1::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Filtc1::Trigger
    }
}
#[doc = "Field `FILTC1` writer - Filter Configuration for FILTn"]
pub type Filtc1W<'a, REG> = crate::FieldWriter<'a, REG, 2, Filtc1>;
impl<'a, REG> Filtc1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Filtc1::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Filtc1::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Filtc1::Trigger)
    }
}
#[doc = "Filter Configuration for FILTn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Filtc2 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
}
impl From<Filtc2> for u8 {
    #[inline(always)]
    fn from(variant: Filtc2) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Filtc2 {
    type Ux = u8;
}
impl crate::IsEnum for Filtc2 {}
#[doc = "Field `FILTC2` reader - Filter Configuration for FILTn"]
pub type Filtc2R = crate::FieldReader<Filtc2>;
impl Filtc2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Filtc2> {
        match self.bits {
            0 => Some(Filtc2::Interrupt),
            1 => Some(Filtc2::DmaReq),
            2 => Some(Filtc2::Trigger),
            _ => None,
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Filtc2::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Filtc2::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Filtc2::Trigger
    }
}
#[doc = "Field `FILTC2` writer - Filter Configuration for FILTn"]
pub type Filtc2W<'a, REG> = crate::FieldWriter<'a, REG, 2, Filtc2>;
impl<'a, REG> Filtc2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Filtc2::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Filtc2::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Filtc2::Trigger)
    }
}
impl R {
    #[doc = "Bits 0:1 - Filter Configuration for FILTn"]
    #[inline(always)]
    pub fn filtc1(&self) -> Filtc1R {
        Filtc1R::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 2:3 - Filter Configuration for FILTn"]
    #[inline(always)]
    pub fn filtc2(&self) -> Filtc2R {
        Filtc2R::new(((self.bits >> 2) & 3) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - Filter Configuration for FILTn"]
    #[inline(always)]
    pub fn filtc1(&mut self) -> Filtc1W<FdcSpec> {
        Filtc1W::new(self, 0)
    }
    #[doc = "Bits 2:3 - Filter Configuration for FILTn"]
    #[inline(always)]
    pub fn filtc2(&mut self) -> Filtc2W<FdcSpec> {
        Filtc2W::new(self, 2)
    }
}
#[doc = "Pin Filter DMA/Trigger Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`fdc::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fdc::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FdcSpec;
impl crate::RegisterSpec for FdcSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`fdc::R`](R) reader structure"]
impl crate::Readable for FdcSpec {}
#[doc = "`write(|w| ..)` method takes [`fdc::W`](W) writer structure"]
impl crate::Writable for FdcSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FDC to value 0"]
impl crate::Resettable for FdcSpec {}

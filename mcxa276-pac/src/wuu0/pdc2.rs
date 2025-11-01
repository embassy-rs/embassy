#[doc = "Register `PDC2` reader"]
pub type R = crate::R<Pdc2Spec>;
#[doc = "Register `PDC2` writer"]
pub type W = crate::W<Pdc2Spec>;
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc16 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc16> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc16) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc16 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc16 {}
#[doc = "Field `WUPDC16` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc16R = crate::FieldReader<Wupdc16>;
impl Wupdc16R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc16 {
        match self.bits {
            0 => Wupdc16::Interrupt,
            1 => Wupdc16::DmaReq,
            2 => Wupdc16::Trigger,
            3 => Wupdc16::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc16::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc16::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc16::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc16::Res
    }
}
#[doc = "Field `WUPDC16` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc16W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc16, crate::Safe>;
impl<'a, REG> Wupdc16W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc16::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc16::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc16::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc16::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc17 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc17> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc17) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc17 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc17 {}
#[doc = "Field `WUPDC17` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc17R = crate::FieldReader<Wupdc17>;
impl Wupdc17R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc17 {
        match self.bits {
            0 => Wupdc17::Interrupt,
            1 => Wupdc17::DmaReq,
            2 => Wupdc17::Trigger,
            3 => Wupdc17::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc17::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc17::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc17::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc17::Res
    }
}
#[doc = "Field `WUPDC17` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc17W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc17, crate::Safe>;
impl<'a, REG> Wupdc17W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc17::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc17::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc17::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc17::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc18 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc18> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc18) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc18 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc18 {}
#[doc = "Field `WUPDC18` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc18R = crate::FieldReader<Wupdc18>;
impl Wupdc18R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc18 {
        match self.bits {
            0 => Wupdc18::Interrupt,
            1 => Wupdc18::DmaReq,
            2 => Wupdc18::Trigger,
            3 => Wupdc18::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc18::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc18::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc18::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc18::Res
    }
}
#[doc = "Field `WUPDC18` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc18W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc18, crate::Safe>;
impl<'a, REG> Wupdc18W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc18::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc18::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc18::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc18::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc19 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc19> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc19) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc19 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc19 {}
#[doc = "Field `WUPDC19` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc19R = crate::FieldReader<Wupdc19>;
impl Wupdc19R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc19 {
        match self.bits {
            0 => Wupdc19::Interrupt,
            1 => Wupdc19::DmaReq,
            2 => Wupdc19::Trigger,
            3 => Wupdc19::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc19::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc19::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc19::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc19::Res
    }
}
#[doc = "Field `WUPDC19` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc19W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc19, crate::Safe>;
impl<'a, REG> Wupdc19W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc19::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc19::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc19::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc19::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc20 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc20> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc20) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc20 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc20 {}
#[doc = "Field `WUPDC20` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc20R = crate::FieldReader<Wupdc20>;
impl Wupdc20R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc20 {
        match self.bits {
            0 => Wupdc20::Interrupt,
            1 => Wupdc20::DmaReq,
            2 => Wupdc20::Trigger,
            3 => Wupdc20::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc20::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc20::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc20::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc20::Res
    }
}
#[doc = "Field `WUPDC20` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc20W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc20, crate::Safe>;
impl<'a, REG> Wupdc20W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc20::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc20::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc20::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc20::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc21 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc21> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc21) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc21 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc21 {}
#[doc = "Field `WUPDC21` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc21R = crate::FieldReader<Wupdc21>;
impl Wupdc21R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc21 {
        match self.bits {
            0 => Wupdc21::Interrupt,
            1 => Wupdc21::DmaReq,
            2 => Wupdc21::Trigger,
            3 => Wupdc21::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc21::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc21::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc21::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc21::Res
    }
}
#[doc = "Field `WUPDC21` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc21W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc21, crate::Safe>;
impl<'a, REG> Wupdc21W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc21::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc21::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc21::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc21::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc22 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc22> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc22) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc22 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc22 {}
#[doc = "Field `WUPDC22` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc22R = crate::FieldReader<Wupdc22>;
impl Wupdc22R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc22 {
        match self.bits {
            0 => Wupdc22::Interrupt,
            1 => Wupdc22::DmaReq,
            2 => Wupdc22::Trigger,
            3 => Wupdc22::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc22::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc22::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc22::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc22::Res
    }
}
#[doc = "Field `WUPDC22` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc22W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc22, crate::Safe>;
impl<'a, REG> Wupdc22W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc22::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc22::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc22::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc22::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc23 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc23> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc23) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc23 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc23 {}
#[doc = "Field `WUPDC23` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc23R = crate::FieldReader<Wupdc23>;
impl Wupdc23R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc23 {
        match self.bits {
            0 => Wupdc23::Interrupt,
            1 => Wupdc23::DmaReq,
            2 => Wupdc23::Trigger,
            3 => Wupdc23::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc23::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc23::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc23::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc23::Res
    }
}
#[doc = "Field `WUPDC23` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc23W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc23, crate::Safe>;
impl<'a, REG> Wupdc23W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc23::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc23::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc23::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc23::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc24 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc24> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc24) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc24 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc24 {}
#[doc = "Field `WUPDC24` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc24R = crate::FieldReader<Wupdc24>;
impl Wupdc24R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc24 {
        match self.bits {
            0 => Wupdc24::Interrupt,
            1 => Wupdc24::DmaReq,
            2 => Wupdc24::Trigger,
            3 => Wupdc24::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc24::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc24::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc24::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc24::Res
    }
}
#[doc = "Field `WUPDC24` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc24W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc24, crate::Safe>;
impl<'a, REG> Wupdc24W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc24::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc24::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc24::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc24::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc25 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc25> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc25) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc25 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc25 {}
#[doc = "Field `WUPDC25` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc25R = crate::FieldReader<Wupdc25>;
impl Wupdc25R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc25 {
        match self.bits {
            0 => Wupdc25::Interrupt,
            1 => Wupdc25::DmaReq,
            2 => Wupdc25::Trigger,
            3 => Wupdc25::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc25::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc25::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc25::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc25::Res
    }
}
#[doc = "Field `WUPDC25` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc25W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc25, crate::Safe>;
impl<'a, REG> Wupdc25W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc25::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc25::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc25::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc25::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc26 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc26> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc26) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc26 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc26 {}
#[doc = "Field `WUPDC26` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc26R = crate::FieldReader<Wupdc26>;
impl Wupdc26R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc26 {
        match self.bits {
            0 => Wupdc26::Interrupt,
            1 => Wupdc26::DmaReq,
            2 => Wupdc26::Trigger,
            3 => Wupdc26::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc26::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc26::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc26::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc26::Res
    }
}
#[doc = "Field `WUPDC26` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc26W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc26, crate::Safe>;
impl<'a, REG> Wupdc26W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc26::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc26::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc26::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc26::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc27 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc27> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc27) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc27 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc27 {}
#[doc = "Field `WUPDC27` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc27R = crate::FieldReader<Wupdc27>;
impl Wupdc27R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc27 {
        match self.bits {
            0 => Wupdc27::Interrupt,
            1 => Wupdc27::DmaReq,
            2 => Wupdc27::Trigger,
            3 => Wupdc27::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc27::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc27::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc27::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc27::Res
    }
}
#[doc = "Field `WUPDC27` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc27W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc27, crate::Safe>;
impl<'a, REG> Wupdc27W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc27::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc27::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc27::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc27::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc28 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc28> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc28) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc28 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc28 {}
#[doc = "Field `WUPDC28` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc28R = crate::FieldReader<Wupdc28>;
impl Wupdc28R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc28 {
        match self.bits {
            0 => Wupdc28::Interrupt,
            1 => Wupdc28::DmaReq,
            2 => Wupdc28::Trigger,
            3 => Wupdc28::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc28::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc28::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc28::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc28::Res
    }
}
#[doc = "Field `WUPDC28` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc28W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc28, crate::Safe>;
impl<'a, REG> Wupdc28W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc28::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc28::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc28::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc28::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc29 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc29> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc29) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc29 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc29 {}
#[doc = "Field `WUPDC29` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc29R = crate::FieldReader<Wupdc29>;
impl Wupdc29R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc29 {
        match self.bits {
            0 => Wupdc29::Interrupt,
            1 => Wupdc29::DmaReq,
            2 => Wupdc29::Trigger,
            3 => Wupdc29::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc29::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc29::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc29::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc29::Res
    }
}
#[doc = "Field `WUPDC29` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc29W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc29, crate::Safe>;
impl<'a, REG> Wupdc29W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc29::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc29::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc29::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc29::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc30 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc30> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc30) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc30 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc30 {}
#[doc = "Field `WUPDC30` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc30R = crate::FieldReader<Wupdc30>;
impl Wupdc30R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc30 {
        match self.bits {
            0 => Wupdc30::Interrupt,
            1 => Wupdc30::DmaReq,
            2 => Wupdc30::Trigger,
            3 => Wupdc30::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc30::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc30::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc30::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc30::Res
    }
}
#[doc = "Field `WUPDC30` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc30W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc30, crate::Safe>;
impl<'a, REG> Wupdc30W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc30::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc30::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc30::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc30::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc31 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc31> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc31) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc31 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc31 {}
#[doc = "Field `WUPDC31` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc31R = crate::FieldReader<Wupdc31>;
impl Wupdc31R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc31 {
        match self.bits {
            0 => Wupdc31::Interrupt,
            1 => Wupdc31::DmaReq,
            2 => Wupdc31::Trigger,
            3 => Wupdc31::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc31::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc31::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc31::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc31::Res
    }
}
#[doc = "Field `WUPDC31` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc31W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc31, crate::Safe>;
impl<'a, REG> Wupdc31W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc31::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc31::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc31::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc31::Res)
    }
}
impl R {
    #[doc = "Bits 0:1 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc16(&self) -> Wupdc16R {
        Wupdc16R::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 2:3 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc17(&self) -> Wupdc17R {
        Wupdc17R::new(((self.bits >> 2) & 3) as u8)
    }
    #[doc = "Bits 4:5 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc18(&self) -> Wupdc18R {
        Wupdc18R::new(((self.bits >> 4) & 3) as u8)
    }
    #[doc = "Bits 6:7 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc19(&self) -> Wupdc19R {
        Wupdc19R::new(((self.bits >> 6) & 3) as u8)
    }
    #[doc = "Bits 8:9 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc20(&self) -> Wupdc20R {
        Wupdc20R::new(((self.bits >> 8) & 3) as u8)
    }
    #[doc = "Bits 10:11 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc21(&self) -> Wupdc21R {
        Wupdc21R::new(((self.bits >> 10) & 3) as u8)
    }
    #[doc = "Bits 12:13 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc22(&self) -> Wupdc22R {
        Wupdc22R::new(((self.bits >> 12) & 3) as u8)
    }
    #[doc = "Bits 14:15 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc23(&self) -> Wupdc23R {
        Wupdc23R::new(((self.bits >> 14) & 3) as u8)
    }
    #[doc = "Bits 16:17 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc24(&self) -> Wupdc24R {
        Wupdc24R::new(((self.bits >> 16) & 3) as u8)
    }
    #[doc = "Bits 18:19 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc25(&self) -> Wupdc25R {
        Wupdc25R::new(((self.bits >> 18) & 3) as u8)
    }
    #[doc = "Bits 20:21 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc26(&self) -> Wupdc26R {
        Wupdc26R::new(((self.bits >> 20) & 3) as u8)
    }
    #[doc = "Bits 22:23 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc27(&self) -> Wupdc27R {
        Wupdc27R::new(((self.bits >> 22) & 3) as u8)
    }
    #[doc = "Bits 24:25 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc28(&self) -> Wupdc28R {
        Wupdc28R::new(((self.bits >> 24) & 3) as u8)
    }
    #[doc = "Bits 26:27 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc29(&self) -> Wupdc29R {
        Wupdc29R::new(((self.bits >> 26) & 3) as u8)
    }
    #[doc = "Bits 28:29 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc30(&self) -> Wupdc30R {
        Wupdc30R::new(((self.bits >> 28) & 3) as u8)
    }
    #[doc = "Bits 30:31 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc31(&self) -> Wupdc31R {
        Wupdc31R::new(((self.bits >> 30) & 3) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc16(&mut self) -> Wupdc16W<Pdc2Spec> {
        Wupdc16W::new(self, 0)
    }
    #[doc = "Bits 2:3 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc17(&mut self) -> Wupdc17W<Pdc2Spec> {
        Wupdc17W::new(self, 2)
    }
    #[doc = "Bits 4:5 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc18(&mut self) -> Wupdc18W<Pdc2Spec> {
        Wupdc18W::new(self, 4)
    }
    #[doc = "Bits 6:7 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc19(&mut self) -> Wupdc19W<Pdc2Spec> {
        Wupdc19W::new(self, 6)
    }
    #[doc = "Bits 8:9 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc20(&mut self) -> Wupdc20W<Pdc2Spec> {
        Wupdc20W::new(self, 8)
    }
    #[doc = "Bits 10:11 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc21(&mut self) -> Wupdc21W<Pdc2Spec> {
        Wupdc21W::new(self, 10)
    }
    #[doc = "Bits 12:13 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc22(&mut self) -> Wupdc22W<Pdc2Spec> {
        Wupdc22W::new(self, 12)
    }
    #[doc = "Bits 14:15 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc23(&mut self) -> Wupdc23W<Pdc2Spec> {
        Wupdc23W::new(self, 14)
    }
    #[doc = "Bits 16:17 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc24(&mut self) -> Wupdc24W<Pdc2Spec> {
        Wupdc24W::new(self, 16)
    }
    #[doc = "Bits 18:19 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc25(&mut self) -> Wupdc25W<Pdc2Spec> {
        Wupdc25W::new(self, 18)
    }
    #[doc = "Bits 20:21 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc26(&mut self) -> Wupdc26W<Pdc2Spec> {
        Wupdc26W::new(self, 20)
    }
    #[doc = "Bits 22:23 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc27(&mut self) -> Wupdc27W<Pdc2Spec> {
        Wupdc27W::new(self, 22)
    }
    #[doc = "Bits 24:25 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc28(&mut self) -> Wupdc28W<Pdc2Spec> {
        Wupdc28W::new(self, 24)
    }
    #[doc = "Bits 26:27 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc29(&mut self) -> Wupdc29W<Pdc2Spec> {
        Wupdc29W::new(self, 26)
    }
    #[doc = "Bits 28:29 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc30(&mut self) -> Wupdc30W<Pdc2Spec> {
        Wupdc30W::new(self, 28)
    }
    #[doc = "Bits 30:31 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc31(&mut self) -> Wupdc31W<Pdc2Spec> {
        Wupdc31W::new(self, 30)
    }
}
#[doc = "Pin DMA/Trigger Configuration 2\n\nYou can [`read`](crate::Reg::read) this register and get [`pdc2::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pdc2::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Pdc2Spec;
impl crate::RegisterSpec for Pdc2Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pdc2::R`](R) reader structure"]
impl crate::Readable for Pdc2Spec {}
#[doc = "`write(|w| ..)` method takes [`pdc2::W`](W) writer structure"]
impl crate::Writable for Pdc2Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PDC2 to value 0"]
impl crate::Resettable for Pdc2Spec {}

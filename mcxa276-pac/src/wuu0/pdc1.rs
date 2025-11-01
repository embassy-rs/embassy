#[doc = "Register `PDC1` reader"]
pub type R = crate::R<Pdc1Spec>;
#[doc = "Register `PDC1` writer"]
pub type W = crate::W<Pdc1Spec>;
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc0 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc0> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc0) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc0 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc0 {}
#[doc = "Field `WUPDC0` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc0R = crate::FieldReader<Wupdc0>;
impl Wupdc0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc0 {
        match self.bits {
            0 => Wupdc0::Interrupt,
            1 => Wupdc0::DmaReq,
            2 => Wupdc0::Trigger,
            3 => Wupdc0::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc0::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc0::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc0::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc0::Res
    }
}
#[doc = "Field `WUPDC0` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc0W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc0, crate::Safe>;
impl<'a, REG> Wupdc0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc0::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc0::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc0::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc0::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc1 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc1> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc1) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc1 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc1 {}
#[doc = "Field `WUPDC1` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc1R = crate::FieldReader<Wupdc1>;
impl Wupdc1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc1 {
        match self.bits {
            0 => Wupdc1::Interrupt,
            1 => Wupdc1::DmaReq,
            2 => Wupdc1::Trigger,
            3 => Wupdc1::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc1::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc1::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc1::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc1::Res
    }
}
#[doc = "Field `WUPDC1` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc1W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc1, crate::Safe>;
impl<'a, REG> Wupdc1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc1::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc1::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc1::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc1::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc2 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc2> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc2) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc2 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc2 {}
#[doc = "Field `WUPDC2` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc2R = crate::FieldReader<Wupdc2>;
impl Wupdc2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc2 {
        match self.bits {
            0 => Wupdc2::Interrupt,
            1 => Wupdc2::DmaReq,
            2 => Wupdc2::Trigger,
            3 => Wupdc2::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc2::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc2::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc2::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc2::Res
    }
}
#[doc = "Field `WUPDC2` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc2W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc2, crate::Safe>;
impl<'a, REG> Wupdc2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc2::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc2::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc2::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc2::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc3 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc3> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc3) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc3 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc3 {}
#[doc = "Field `WUPDC3` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc3R = crate::FieldReader<Wupdc3>;
impl Wupdc3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc3 {
        match self.bits {
            0 => Wupdc3::Interrupt,
            1 => Wupdc3::DmaReq,
            2 => Wupdc3::Trigger,
            3 => Wupdc3::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc3::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc3::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc3::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc3::Res
    }
}
#[doc = "Field `WUPDC3` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc3W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc3, crate::Safe>;
impl<'a, REG> Wupdc3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc3::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc3::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc3::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc3::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc4 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc4> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc4) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc4 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc4 {}
#[doc = "Field `WUPDC4` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc4R = crate::FieldReader<Wupdc4>;
impl Wupdc4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc4 {
        match self.bits {
            0 => Wupdc4::Interrupt,
            1 => Wupdc4::DmaReq,
            2 => Wupdc4::Trigger,
            3 => Wupdc4::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc4::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc4::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc4::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc4::Res
    }
}
#[doc = "Field `WUPDC4` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc4W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc4, crate::Safe>;
impl<'a, REG> Wupdc4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc4::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc4::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc4::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc4::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc5 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc5> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc5) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc5 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc5 {}
#[doc = "Field `WUPDC5` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc5R = crate::FieldReader<Wupdc5>;
impl Wupdc5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc5 {
        match self.bits {
            0 => Wupdc5::Interrupt,
            1 => Wupdc5::DmaReq,
            2 => Wupdc5::Trigger,
            3 => Wupdc5::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc5::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc5::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc5::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc5::Res
    }
}
#[doc = "Field `WUPDC5` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc5W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc5, crate::Safe>;
impl<'a, REG> Wupdc5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc5::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc5::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc5::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc5::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc6 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc6> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc6) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc6 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc6 {}
#[doc = "Field `WUPDC6` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc6R = crate::FieldReader<Wupdc6>;
impl Wupdc6R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc6 {
        match self.bits {
            0 => Wupdc6::Interrupt,
            1 => Wupdc6::DmaReq,
            2 => Wupdc6::Trigger,
            3 => Wupdc6::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc6::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc6::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc6::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc6::Res
    }
}
#[doc = "Field `WUPDC6` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc6W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc6, crate::Safe>;
impl<'a, REG> Wupdc6W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc6::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc6::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc6::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc6::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc7 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc7> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc7) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc7 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc7 {}
#[doc = "Field `WUPDC7` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc7R = crate::FieldReader<Wupdc7>;
impl Wupdc7R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc7 {
        match self.bits {
            0 => Wupdc7::Interrupt,
            1 => Wupdc7::DmaReq,
            2 => Wupdc7::Trigger,
            3 => Wupdc7::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc7::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc7::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc7::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc7::Res
    }
}
#[doc = "Field `WUPDC7` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc7W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc7, crate::Safe>;
impl<'a, REG> Wupdc7W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc7::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc7::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc7::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc7::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc8 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc8> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc8) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc8 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc8 {}
#[doc = "Field `WUPDC8` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc8R = crate::FieldReader<Wupdc8>;
impl Wupdc8R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc8 {
        match self.bits {
            0 => Wupdc8::Interrupt,
            1 => Wupdc8::DmaReq,
            2 => Wupdc8::Trigger,
            3 => Wupdc8::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc8::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc8::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc8::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc8::Res
    }
}
#[doc = "Field `WUPDC8` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc8W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc8, crate::Safe>;
impl<'a, REG> Wupdc8W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc8::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc8::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc8::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc8::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc9 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc9> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc9) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc9 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc9 {}
#[doc = "Field `WUPDC9` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc9R = crate::FieldReader<Wupdc9>;
impl Wupdc9R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc9 {
        match self.bits {
            0 => Wupdc9::Interrupt,
            1 => Wupdc9::DmaReq,
            2 => Wupdc9::Trigger,
            3 => Wupdc9::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc9::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc9::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc9::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc9::Res
    }
}
#[doc = "Field `WUPDC9` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc9W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc9, crate::Safe>;
impl<'a, REG> Wupdc9W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc9::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc9::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc9::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc9::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc10 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc10> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc10) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc10 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc10 {}
#[doc = "Field `WUPDC10` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc10R = crate::FieldReader<Wupdc10>;
impl Wupdc10R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc10 {
        match self.bits {
            0 => Wupdc10::Interrupt,
            1 => Wupdc10::DmaReq,
            2 => Wupdc10::Trigger,
            3 => Wupdc10::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc10::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc10::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc10::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc10::Res
    }
}
#[doc = "Field `WUPDC10` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc10W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc10, crate::Safe>;
impl<'a, REG> Wupdc10W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc10::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc10::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc10::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc10::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc11 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc11> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc11) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc11 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc11 {}
#[doc = "Field `WUPDC11` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc11R = crate::FieldReader<Wupdc11>;
impl Wupdc11R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc11 {
        match self.bits {
            0 => Wupdc11::Interrupt,
            1 => Wupdc11::DmaReq,
            2 => Wupdc11::Trigger,
            3 => Wupdc11::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc11::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc11::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc11::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc11::Res
    }
}
#[doc = "Field `WUPDC11` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc11W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc11, crate::Safe>;
impl<'a, REG> Wupdc11W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc11::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc11::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc11::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc11::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc12 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc12> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc12) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc12 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc12 {}
#[doc = "Field `WUPDC12` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc12R = crate::FieldReader<Wupdc12>;
impl Wupdc12R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc12 {
        match self.bits {
            0 => Wupdc12::Interrupt,
            1 => Wupdc12::DmaReq,
            2 => Wupdc12::Trigger,
            3 => Wupdc12::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc12::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc12::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc12::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc12::Res
    }
}
#[doc = "Field `WUPDC12` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc12W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc12, crate::Safe>;
impl<'a, REG> Wupdc12W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc12::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc12::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc12::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc12::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc13 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc13> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc13) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc13 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc13 {}
#[doc = "Field `WUPDC13` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc13R = crate::FieldReader<Wupdc13>;
impl Wupdc13R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc13 {
        match self.bits {
            0 => Wupdc13::Interrupt,
            1 => Wupdc13::DmaReq,
            2 => Wupdc13::Trigger,
            3 => Wupdc13::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc13::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc13::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc13::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc13::Res
    }
}
#[doc = "Field `WUPDC13` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc13W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc13, crate::Safe>;
impl<'a, REG> Wupdc13W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc13::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc13::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc13::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc13::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc14 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc14> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc14) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc14 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc14 {}
#[doc = "Field `WUPDC14` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc14R = crate::FieldReader<Wupdc14>;
impl Wupdc14R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc14 {
        match self.bits {
            0 => Wupdc14::Interrupt,
            1 => Wupdc14::DmaReq,
            2 => Wupdc14::Trigger,
            3 => Wupdc14::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc14::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc14::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc14::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc14::Res
    }
}
#[doc = "Field `WUPDC14` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc14W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc14, crate::Safe>;
impl<'a, REG> Wupdc14W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc14::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc14::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc14::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc14::Res)
    }
}
#[doc = "Wake-up Pin Configuration for WUU_Pn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Wupdc15 {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: DMA request"]
    DmaReq = 1,
    #[doc = "2: Trigger event"]
    Trigger = 2,
    #[doc = "3: Reserved"]
    Res = 3,
}
impl From<Wupdc15> for u8 {
    #[inline(always)]
    fn from(variant: Wupdc15) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Wupdc15 {
    type Ux = u8;
}
impl crate::IsEnum for Wupdc15 {}
#[doc = "Field `WUPDC15` reader - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc15R = crate::FieldReader<Wupdc15>;
impl Wupdc15R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wupdc15 {
        match self.bits {
            0 => Wupdc15::Interrupt,
            1 => Wupdc15::DmaReq,
            2 => Wupdc15::Trigger,
            3 => Wupdc15::Res,
            _ => unreachable!(),
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wupdc15::Interrupt
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn is_dma_req(&self) -> bool {
        *self == Wupdc15::DmaReq
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn is_trigger(&self) -> bool {
        *self == Wupdc15::Trigger
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn is_res(&self) -> bool {
        *self == Wupdc15::Res
    }
}
#[doc = "Field `WUPDC15` writer - Wake-up Pin Configuration for WUU_Pn"]
pub type Wupdc15W<'a, REG> = crate::FieldWriter<'a, REG, 2, Wupdc15, crate::Safe>;
impl<'a, REG> Wupdc15W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc15::Interrupt)
    }
    #[doc = "DMA request"]
    #[inline(always)]
    pub fn dma_req(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc15::DmaReq)
    }
    #[doc = "Trigger event"]
    #[inline(always)]
    pub fn trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc15::Trigger)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn res(self) -> &'a mut crate::W<REG> {
        self.variant(Wupdc15::Res)
    }
}
impl R {
    #[doc = "Bits 0:1 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc0(&self) -> Wupdc0R {
        Wupdc0R::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 2:3 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc1(&self) -> Wupdc1R {
        Wupdc1R::new(((self.bits >> 2) & 3) as u8)
    }
    #[doc = "Bits 4:5 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc2(&self) -> Wupdc2R {
        Wupdc2R::new(((self.bits >> 4) & 3) as u8)
    }
    #[doc = "Bits 6:7 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc3(&self) -> Wupdc3R {
        Wupdc3R::new(((self.bits >> 6) & 3) as u8)
    }
    #[doc = "Bits 8:9 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc4(&self) -> Wupdc4R {
        Wupdc4R::new(((self.bits >> 8) & 3) as u8)
    }
    #[doc = "Bits 10:11 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc5(&self) -> Wupdc5R {
        Wupdc5R::new(((self.bits >> 10) & 3) as u8)
    }
    #[doc = "Bits 12:13 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc6(&self) -> Wupdc6R {
        Wupdc6R::new(((self.bits >> 12) & 3) as u8)
    }
    #[doc = "Bits 14:15 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc7(&self) -> Wupdc7R {
        Wupdc7R::new(((self.bits >> 14) & 3) as u8)
    }
    #[doc = "Bits 16:17 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc8(&self) -> Wupdc8R {
        Wupdc8R::new(((self.bits >> 16) & 3) as u8)
    }
    #[doc = "Bits 18:19 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc9(&self) -> Wupdc9R {
        Wupdc9R::new(((self.bits >> 18) & 3) as u8)
    }
    #[doc = "Bits 20:21 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc10(&self) -> Wupdc10R {
        Wupdc10R::new(((self.bits >> 20) & 3) as u8)
    }
    #[doc = "Bits 22:23 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc11(&self) -> Wupdc11R {
        Wupdc11R::new(((self.bits >> 22) & 3) as u8)
    }
    #[doc = "Bits 24:25 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc12(&self) -> Wupdc12R {
        Wupdc12R::new(((self.bits >> 24) & 3) as u8)
    }
    #[doc = "Bits 26:27 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc13(&self) -> Wupdc13R {
        Wupdc13R::new(((self.bits >> 26) & 3) as u8)
    }
    #[doc = "Bits 28:29 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc14(&self) -> Wupdc14R {
        Wupdc14R::new(((self.bits >> 28) & 3) as u8)
    }
    #[doc = "Bits 30:31 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc15(&self) -> Wupdc15R {
        Wupdc15R::new(((self.bits >> 30) & 3) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc0(&mut self) -> Wupdc0W<Pdc1Spec> {
        Wupdc0W::new(self, 0)
    }
    #[doc = "Bits 2:3 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc1(&mut self) -> Wupdc1W<Pdc1Spec> {
        Wupdc1W::new(self, 2)
    }
    #[doc = "Bits 4:5 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc2(&mut self) -> Wupdc2W<Pdc1Spec> {
        Wupdc2W::new(self, 4)
    }
    #[doc = "Bits 6:7 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc3(&mut self) -> Wupdc3W<Pdc1Spec> {
        Wupdc3W::new(self, 6)
    }
    #[doc = "Bits 8:9 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc4(&mut self) -> Wupdc4W<Pdc1Spec> {
        Wupdc4W::new(self, 8)
    }
    #[doc = "Bits 10:11 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc5(&mut self) -> Wupdc5W<Pdc1Spec> {
        Wupdc5W::new(self, 10)
    }
    #[doc = "Bits 12:13 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc6(&mut self) -> Wupdc6W<Pdc1Spec> {
        Wupdc6W::new(self, 12)
    }
    #[doc = "Bits 14:15 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc7(&mut self) -> Wupdc7W<Pdc1Spec> {
        Wupdc7W::new(self, 14)
    }
    #[doc = "Bits 16:17 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc8(&mut self) -> Wupdc8W<Pdc1Spec> {
        Wupdc8W::new(self, 16)
    }
    #[doc = "Bits 18:19 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc9(&mut self) -> Wupdc9W<Pdc1Spec> {
        Wupdc9W::new(self, 18)
    }
    #[doc = "Bits 20:21 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc10(&mut self) -> Wupdc10W<Pdc1Spec> {
        Wupdc10W::new(self, 20)
    }
    #[doc = "Bits 22:23 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc11(&mut self) -> Wupdc11W<Pdc1Spec> {
        Wupdc11W::new(self, 22)
    }
    #[doc = "Bits 24:25 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc12(&mut self) -> Wupdc12W<Pdc1Spec> {
        Wupdc12W::new(self, 24)
    }
    #[doc = "Bits 26:27 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc13(&mut self) -> Wupdc13W<Pdc1Spec> {
        Wupdc13W::new(self, 26)
    }
    #[doc = "Bits 28:29 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc14(&mut self) -> Wupdc14W<Pdc1Spec> {
        Wupdc14W::new(self, 28)
    }
    #[doc = "Bits 30:31 - Wake-up Pin Configuration for WUU_Pn"]
    #[inline(always)]
    pub fn wupdc15(&mut self) -> Wupdc15W<Pdc1Spec> {
        Wupdc15W::new(self, 30)
    }
}
#[doc = "Pin DMA/Trigger Configuration 1\n\nYou can [`read`](crate::Reg::read) this register and get [`pdc1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pdc1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Pdc1Spec;
impl crate::RegisterSpec for Pdc1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pdc1::R`](R) reader structure"]
impl crate::Readable for Pdc1Spec {}
#[doc = "`write(|w| ..)` method takes [`pdc1::W`](W) writer structure"]
impl crate::Writable for Pdc1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PDC1 to value 0"]
impl crate::Resettable for Pdc1Spec {}

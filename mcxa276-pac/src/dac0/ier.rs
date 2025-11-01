#[doc = "Register `IER` reader"]
pub type R = crate::R<IerSpec>;
#[doc = "Register `IER` writer"]
pub type W = crate::W<IerSpec>;
#[doc = "FIFO Full Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FullIe {
    #[doc = "0: Disables"]
    Disabled = 0,
    #[doc = "1: Enables"]
    Enabled = 1,
}
impl From<FullIe> for bool {
    #[inline(always)]
    fn from(variant: FullIe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FULL_IE` reader - FIFO Full Interrupt Enable"]
pub type FullIeR = crate::BitReader<FullIe>;
impl FullIeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> FullIe {
        match self.bits {
            false => FullIe::Disabled,
            true => FullIe::Enabled,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == FullIe::Disabled
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == FullIe::Enabled
    }
}
#[doc = "Field `FULL_IE` writer - FIFO Full Interrupt Enable"]
pub type FullIeW<'a, REG> = crate::BitWriter<'a, REG, FullIe>;
impl<'a, REG> FullIeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(FullIe::Disabled)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(FullIe::Enabled)
    }
}
#[doc = "FIFO Empty Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmptyIe {
    #[doc = "0: Disables"]
    Disabled = 0,
    #[doc = "1: Enables"]
    Enabled = 1,
}
impl From<EmptyIe> for bool {
    #[inline(always)]
    fn from(variant: EmptyIe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EMPTY_IE` reader - FIFO Empty Interrupt Enable"]
pub type EmptyIeR = crate::BitReader<EmptyIe>;
impl EmptyIeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> EmptyIe {
        match self.bits {
            false => EmptyIe::Disabled,
            true => EmptyIe::Enabled,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == EmptyIe::Disabled
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == EmptyIe::Enabled
    }
}
#[doc = "Field `EMPTY_IE` writer - FIFO Empty Interrupt Enable"]
pub type EmptyIeW<'a, REG> = crate::BitWriter<'a, REG, EmptyIe>;
impl<'a, REG> EmptyIeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(EmptyIe::Disabled)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(EmptyIe::Enabled)
    }
}
#[doc = "FIFO Watermark Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WmIe {
    #[doc = "0: Disables"]
    Disabled = 0,
    #[doc = "1: Enables"]
    Enabled = 1,
}
impl From<WmIe> for bool {
    #[inline(always)]
    fn from(variant: WmIe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WM_IE` reader - FIFO Watermark Interrupt Enable"]
pub type WmIeR = crate::BitReader<WmIe>;
impl WmIeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> WmIe {
        match self.bits {
            false => WmIe::Disabled,
            true => WmIe::Enabled,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == WmIe::Disabled
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == WmIe::Enabled
    }
}
#[doc = "Field `WM_IE` writer - FIFO Watermark Interrupt Enable"]
pub type WmIeW<'a, REG> = crate::BitWriter<'a, REG, WmIe>;
impl<'a, REG> WmIeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(WmIe::Disabled)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(WmIe::Enabled)
    }
}
#[doc = "Swing Back One Cycle Complete Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwbkIe {
    #[doc = "0: Disables"]
    Disabled = 0,
    #[doc = "1: Enables"]
    Enabled = 1,
}
impl From<SwbkIe> for bool {
    #[inline(always)]
    fn from(variant: SwbkIe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SWBK_IE` reader - Swing Back One Cycle Complete Interrupt Enable"]
pub type SwbkIeR = crate::BitReader<SwbkIe>;
impl SwbkIeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> SwbkIe {
        match self.bits {
            false => SwbkIe::Disabled,
            true => SwbkIe::Enabled,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == SwbkIe::Disabled
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == SwbkIe::Enabled
    }
}
#[doc = "Field `SWBK_IE` writer - Swing Back One Cycle Complete Interrupt Enable"]
pub type SwbkIeW<'a, REG> = crate::BitWriter<'a, REG, SwbkIe>;
impl<'a, REG> SwbkIeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(SwbkIe::Disabled)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(SwbkIe::Enabled)
    }
}
#[doc = "FIFO Overflow Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OfIe {
    #[doc = "0: Disables"]
    Disabled = 0,
    #[doc = "1: Enables"]
    Enabled = 1,
}
impl From<OfIe> for bool {
    #[inline(always)]
    fn from(variant: OfIe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OF_IE` reader - FIFO Overflow Interrupt Enable"]
pub type OfIeR = crate::BitReader<OfIe>;
impl OfIeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> OfIe {
        match self.bits {
            false => OfIe::Disabled,
            true => OfIe::Enabled,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == OfIe::Disabled
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == OfIe::Enabled
    }
}
#[doc = "Field `OF_IE` writer - FIFO Overflow Interrupt Enable"]
pub type OfIeW<'a, REG> = crate::BitWriter<'a, REG, OfIe>;
impl<'a, REG> OfIeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(OfIe::Disabled)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(OfIe::Enabled)
    }
}
#[doc = "FIFO Underflow Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UfIe {
    #[doc = "0: Disables"]
    Disabled = 0,
    #[doc = "1: Enables"]
    Enabled = 1,
}
impl From<UfIe> for bool {
    #[inline(always)]
    fn from(variant: UfIe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `UF_IE` reader - FIFO Underflow Interrupt Enable"]
pub type UfIeR = crate::BitReader<UfIe>;
impl UfIeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> UfIe {
        match self.bits {
            false => UfIe::Disabled,
            true => UfIe::Enabled,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == UfIe::Disabled
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == UfIe::Enabled
    }
}
#[doc = "Field `UF_IE` writer - FIFO Underflow Interrupt Enable"]
pub type UfIeW<'a, REG> = crate::BitWriter<'a, REG, UfIe>;
impl<'a, REG> UfIeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(UfIe::Disabled)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(UfIe::Enabled)
    }
}
#[doc = "PTG Mode Conversion Complete Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PtgcocoIe {
    #[doc = "0: Disables"]
    Disabled = 0,
    #[doc = "1: Enables"]
    Enabled = 1,
}
impl From<PtgcocoIe> for bool {
    #[inline(always)]
    fn from(variant: PtgcocoIe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTGCOCO_IE` reader - PTG Mode Conversion Complete Interrupt Enable"]
pub type PtgcocoIeR = crate::BitReader<PtgcocoIe>;
impl PtgcocoIeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> PtgcocoIe {
        match self.bits {
            false => PtgcocoIe::Disabled,
            true => PtgcocoIe::Enabled,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == PtgcocoIe::Disabled
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == PtgcocoIe::Enabled
    }
}
#[doc = "Field `PTGCOCO_IE` writer - PTG Mode Conversion Complete Interrupt Enable"]
pub type PtgcocoIeW<'a, REG> = crate::BitWriter<'a, REG, PtgcocoIe>;
impl<'a, REG> PtgcocoIeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(PtgcocoIe::Disabled)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(PtgcocoIe::Enabled)
    }
}
impl R {
    #[doc = "Bit 0 - FIFO Full Interrupt Enable"]
    #[inline(always)]
    pub fn full_ie(&self) -> FullIeR {
        FullIeR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - FIFO Empty Interrupt Enable"]
    #[inline(always)]
    pub fn empty_ie(&self) -> EmptyIeR {
        EmptyIeR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - FIFO Watermark Interrupt Enable"]
    #[inline(always)]
    pub fn wm_ie(&self) -> WmIeR {
        WmIeR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Swing Back One Cycle Complete Interrupt Enable"]
    #[inline(always)]
    pub fn swbk_ie(&self) -> SwbkIeR {
        SwbkIeR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 6 - FIFO Overflow Interrupt Enable"]
    #[inline(always)]
    pub fn of_ie(&self) -> OfIeR {
        OfIeR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - FIFO Underflow Interrupt Enable"]
    #[inline(always)]
    pub fn uf_ie(&self) -> UfIeR {
        UfIeR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - PTG Mode Conversion Complete Interrupt Enable"]
    #[inline(always)]
    pub fn ptgcoco_ie(&self) -> PtgcocoIeR {
        PtgcocoIeR::new(((self.bits >> 8) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - FIFO Full Interrupt Enable"]
    #[inline(always)]
    pub fn full_ie(&mut self) -> FullIeW<IerSpec> {
        FullIeW::new(self, 0)
    }
    #[doc = "Bit 1 - FIFO Empty Interrupt Enable"]
    #[inline(always)]
    pub fn empty_ie(&mut self) -> EmptyIeW<IerSpec> {
        EmptyIeW::new(self, 1)
    }
    #[doc = "Bit 2 - FIFO Watermark Interrupt Enable"]
    #[inline(always)]
    pub fn wm_ie(&mut self) -> WmIeW<IerSpec> {
        WmIeW::new(self, 2)
    }
    #[doc = "Bit 3 - Swing Back One Cycle Complete Interrupt Enable"]
    #[inline(always)]
    pub fn swbk_ie(&mut self) -> SwbkIeW<IerSpec> {
        SwbkIeW::new(self, 3)
    }
    #[doc = "Bit 6 - FIFO Overflow Interrupt Enable"]
    #[inline(always)]
    pub fn of_ie(&mut self) -> OfIeW<IerSpec> {
        OfIeW::new(self, 6)
    }
    #[doc = "Bit 7 - FIFO Underflow Interrupt Enable"]
    #[inline(always)]
    pub fn uf_ie(&mut self) -> UfIeW<IerSpec> {
        UfIeW::new(self, 7)
    }
    #[doc = "Bit 8 - PTG Mode Conversion Complete Interrupt Enable"]
    #[inline(always)]
    pub fn ptgcoco_ie(&mut self) -> PtgcocoIeW<IerSpec> {
        PtgcocoIeW::new(self, 8)
    }
}
#[doc = "Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`ier::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ier::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct IerSpec;
impl crate::RegisterSpec for IerSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ier::R`](R) reader structure"]
impl crate::Readable for IerSpec {}
#[doc = "`write(|w| ..)` method takes [`ier::W`](W) writer structure"]
impl crate::Writable for IerSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets IER to value 0"]
impl crate::Resettable for IerSpec {}

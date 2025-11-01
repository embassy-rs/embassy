#[doc = "Register `FSR` reader"]
pub type R = crate::R<FsrSpec>;
#[doc = "Register `FSR` writer"]
pub type W = crate::W<FsrSpec>;
#[doc = "FIFO Full Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Full {
    #[doc = "0: Not full"]
    NotFull = 0,
    #[doc = "1: Full"]
    Full = 1,
}
impl From<Full> for bool {
    #[inline(always)]
    fn from(variant: Full) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FULL` reader - FIFO Full Flag"]
pub type FullR = crate::BitReader<Full>;
impl FullR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Full {
        match self.bits {
            false => Full::NotFull,
            true => Full::Full,
        }
    }
    #[doc = "Not full"]
    #[inline(always)]
    pub fn is_not_full(&self) -> bool {
        *self == Full::NotFull
    }
    #[doc = "Full"]
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        *self == Full::Full
    }
}
#[doc = "FIFO Empty Flag\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Empty {
    #[doc = "0: Not empty"]
    NotEmpty = 0,
    #[doc = "1: Empty"]
    Empty = 1,
}
impl From<Empty> for bool {
    #[inline(always)]
    fn from(variant: Empty) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EMPTY` reader - FIFO Empty Flag"]
pub type EmptyR = crate::BitReader<Empty>;
impl EmptyR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Empty {
        match self.bits {
            false => Empty::NotEmpty,
            true => Empty::Empty,
        }
    }
    #[doc = "Not empty"]
    #[inline(always)]
    pub fn is_not_empty(&self) -> bool {
        *self == Empty::NotEmpty
    }
    #[doc = "Empty"]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        *self == Empty::Empty
    }
}
#[doc = "FIFO Watermark Status Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wm {
    #[doc = "0: Data in FIFO is more than watermark level"]
    MoreThanWlevel = 0,
    #[doc = "1: Data in FIFO is less than or equal to watermark level"]
    LessThanWlevel = 1,
}
impl From<Wm> for bool {
    #[inline(always)]
    fn from(variant: Wm) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WM` reader - FIFO Watermark Status Flag"]
pub type WmR = crate::BitReader<Wm>;
impl WmR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wm {
        match self.bits {
            false => Wm::MoreThanWlevel,
            true => Wm::LessThanWlevel,
        }
    }
    #[doc = "Data in FIFO is more than watermark level"]
    #[inline(always)]
    pub fn is_more_than_wlevel(&self) -> bool {
        *self == Wm::MoreThanWlevel
    }
    #[doc = "Data in FIFO is less than or equal to watermark level"]
    #[inline(always)]
    pub fn is_less_than_wlevel(&self) -> bool {
        *self == Wm::LessThanWlevel
    }
}
#[doc = "Swing Back One Cycle Complete Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Swbk {
    #[doc = "0: No swing back cycle has completed since the last time the flag was cleared"]
    NoSwing = 0,
    #[doc = "1: At least one swing back cycle has occurred since the last time the flag was cleared"]
    SwingBack = 1,
}
impl From<Swbk> for bool {
    #[inline(always)]
    fn from(variant: Swbk) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SWBK` reader - Swing Back One Cycle Complete Flag"]
pub type SwbkR = crate::BitReader<Swbk>;
impl SwbkR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Swbk {
        match self.bits {
            false => Swbk::NoSwing,
            true => Swbk::SwingBack,
        }
    }
    #[doc = "No swing back cycle has completed since the last time the flag was cleared"]
    #[inline(always)]
    pub fn is_no_swing(&self) -> bool {
        *self == Swbk::NoSwing
    }
    #[doc = "At least one swing back cycle has occurred since the last time the flag was cleared"]
    #[inline(always)]
    pub fn is_swing_back(&self) -> bool {
        *self == Swbk::SwingBack
    }
}
#[doc = "Field `SWBK` writer - Swing Back One Cycle Complete Flag"]
pub type SwbkW<'a, REG> = crate::BitWriter1C<'a, REG, Swbk>;
impl<'a, REG> SwbkW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No swing back cycle has completed since the last time the flag was cleared"]
    #[inline(always)]
    pub fn no_swing(self) -> &'a mut crate::W<REG> {
        self.variant(Swbk::NoSwing)
    }
    #[doc = "At least one swing back cycle has occurred since the last time the flag was cleared"]
    #[inline(always)]
    pub fn swing_back(self) -> &'a mut crate::W<REG> {
        self.variant(Swbk::SwingBack)
    }
}
#[doc = "FIFO Overflow Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Of {
    #[doc = "0: No overflow has occurred since the last time the flag was cleared"]
    NoOverflow = 0,
    #[doc = "1: At least one FIFO overflow has occurred since the last time the flag was cleared"]
    Overflow = 1,
}
impl From<Of> for bool {
    #[inline(always)]
    fn from(variant: Of) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OF` reader - FIFO Overflow Flag"]
pub type OfR = crate::BitReader<Of>;
impl OfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Of {
        match self.bits {
            false => Of::NoOverflow,
            true => Of::Overflow,
        }
    }
    #[doc = "No overflow has occurred since the last time the flag was cleared"]
    #[inline(always)]
    pub fn is_no_overflow(&self) -> bool {
        *self == Of::NoOverflow
    }
    #[doc = "At least one FIFO overflow has occurred since the last time the flag was cleared"]
    #[inline(always)]
    pub fn is_overflow(&self) -> bool {
        *self == Of::Overflow
    }
}
#[doc = "Field `OF` writer - FIFO Overflow Flag"]
pub type OfW<'a, REG> = crate::BitWriter1C<'a, REG, Of>;
impl<'a, REG> OfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No overflow has occurred since the last time the flag was cleared"]
    #[inline(always)]
    pub fn no_overflow(self) -> &'a mut crate::W<REG> {
        self.variant(Of::NoOverflow)
    }
    #[doc = "At least one FIFO overflow has occurred since the last time the flag was cleared"]
    #[inline(always)]
    pub fn overflow(self) -> &'a mut crate::W<REG> {
        self.variant(Of::Overflow)
    }
}
#[doc = "FIFO Underflow Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Uf {
    #[doc = "0: No underflow has occurred since the last time the flag was cleared"]
    NoUnderflow = 0,
    #[doc = "1: At least one trigger underflow has occurred since the last time the flag was cleared"]
    Underflow = 1,
}
impl From<Uf> for bool {
    #[inline(always)]
    fn from(variant: Uf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `UF` reader - FIFO Underflow Flag"]
pub type UfR = crate::BitReader<Uf>;
impl UfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Uf {
        match self.bits {
            false => Uf::NoUnderflow,
            true => Uf::Underflow,
        }
    }
    #[doc = "No underflow has occurred since the last time the flag was cleared"]
    #[inline(always)]
    pub fn is_no_underflow(&self) -> bool {
        *self == Uf::NoUnderflow
    }
    #[doc = "At least one trigger underflow has occurred since the last time the flag was cleared"]
    #[inline(always)]
    pub fn is_underflow(&self) -> bool {
        *self == Uf::Underflow
    }
}
#[doc = "Field `UF` writer - FIFO Underflow Flag"]
pub type UfW<'a, REG> = crate::BitWriter1C<'a, REG, Uf>;
impl<'a, REG> UfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No underflow has occurred since the last time the flag was cleared"]
    #[inline(always)]
    pub fn no_underflow(self) -> &'a mut crate::W<REG> {
        self.variant(Uf::NoUnderflow)
    }
    #[doc = "At least one trigger underflow has occurred since the last time the flag was cleared"]
    #[inline(always)]
    pub fn underflow(self) -> &'a mut crate::W<REG> {
        self.variant(Uf::Underflow)
    }
}
#[doc = "Period Trigger Mode Conversion Complete Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptgcoco {
    #[doc = "0: Not completed or not started"]
    NotStart = 0,
    #[doc = "1: Completed"]
    Completed = 1,
}
impl From<Ptgcoco> for bool {
    #[inline(always)]
    fn from(variant: Ptgcoco) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTGCOCO` reader - Period Trigger Mode Conversion Complete Flag"]
pub type PtgcocoR = crate::BitReader<Ptgcoco>;
impl PtgcocoR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptgcoco {
        match self.bits {
            false => Ptgcoco::NotStart,
            true => Ptgcoco::Completed,
        }
    }
    #[doc = "Not completed or not started"]
    #[inline(always)]
    pub fn is_not_start(&self) -> bool {
        *self == Ptgcoco::NotStart
    }
    #[doc = "Completed"]
    #[inline(always)]
    pub fn is_completed(&self) -> bool {
        *self == Ptgcoco::Completed
    }
}
#[doc = "Field `PTGCOCO` writer - Period Trigger Mode Conversion Complete Flag"]
pub type PtgcocoW<'a, REG> = crate::BitWriter1C<'a, REG, Ptgcoco>;
impl<'a, REG> PtgcocoW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not completed or not started"]
    #[inline(always)]
    pub fn not_start(self) -> &'a mut crate::W<REG> {
        self.variant(Ptgcoco::NotStart)
    }
    #[doc = "Completed"]
    #[inline(always)]
    pub fn completed(self) -> &'a mut crate::W<REG> {
        self.variant(Ptgcoco::Completed)
    }
}
impl R {
    #[doc = "Bit 0 - FIFO Full Flag"]
    #[inline(always)]
    pub fn full(&self) -> FullR {
        FullR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - FIFO Empty Flag"]
    #[inline(always)]
    pub fn empty(&self) -> EmptyR {
        EmptyR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - FIFO Watermark Status Flag"]
    #[inline(always)]
    pub fn wm(&self) -> WmR {
        WmR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Swing Back One Cycle Complete Flag"]
    #[inline(always)]
    pub fn swbk(&self) -> SwbkR {
        SwbkR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 6 - FIFO Overflow Flag"]
    #[inline(always)]
    pub fn of(&self) -> OfR {
        OfR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - FIFO Underflow Flag"]
    #[inline(always)]
    pub fn uf(&self) -> UfR {
        UfR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Period Trigger Mode Conversion Complete Flag"]
    #[inline(always)]
    pub fn ptgcoco(&self) -> PtgcocoR {
        PtgcocoR::new(((self.bits >> 8) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 3 - Swing Back One Cycle Complete Flag"]
    #[inline(always)]
    pub fn swbk(&mut self) -> SwbkW<FsrSpec> {
        SwbkW::new(self, 3)
    }
    #[doc = "Bit 6 - FIFO Overflow Flag"]
    #[inline(always)]
    pub fn of(&mut self) -> OfW<FsrSpec> {
        OfW::new(self, 6)
    }
    #[doc = "Bit 7 - FIFO Underflow Flag"]
    #[inline(always)]
    pub fn uf(&mut self) -> UfW<FsrSpec> {
        UfW::new(self, 7)
    }
    #[doc = "Bit 8 - Period Trigger Mode Conversion Complete Flag"]
    #[inline(always)]
    pub fn ptgcoco(&mut self) -> PtgcocoW<FsrSpec> {
        PtgcocoW::new(self, 8)
    }
}
#[doc = "FIFO Status\n\nYou can [`read`](crate::Reg::read) this register and get [`fsr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fsr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FsrSpec;
impl crate::RegisterSpec for FsrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`fsr::R`](R) reader structure"]
impl crate::Readable for FsrSpec {}
#[doc = "`write(|w| ..)` method takes [`fsr::W`](W) writer structure"]
impl crate::Writable for FsrSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x01c8;
}
#[doc = "`reset()` method sets FSR to value 0x02"]
impl crate::Resettable for FsrSpec {
    const RESET_VALUE: u32 = 0x02;
}

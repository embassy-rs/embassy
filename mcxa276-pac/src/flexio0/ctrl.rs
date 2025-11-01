#[doc = "Register `CTRL` reader"]
pub type R = crate::R<CtrlSpec>;
#[doc = "Register `CTRL` writer"]
pub type W = crate::W<CtrlSpec>;
#[doc = "FLEXIO Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flexen {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Flexen> for bool {
    #[inline(always)]
    fn from(variant: Flexen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FLEXEN` reader - FLEXIO Enable"]
pub type FlexenR = crate::BitReader<Flexen>;
impl FlexenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Flexen {
        match self.bits {
            false => Flexen::Disable,
            true => Flexen::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Flexen::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Flexen::Enable
    }
}
#[doc = "Field `FLEXEN` writer - FLEXIO Enable"]
pub type FlexenW<'a, REG> = crate::BitWriter<'a, REG, Flexen>;
impl<'a, REG> FlexenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Flexen::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Flexen::Enable)
    }
}
#[doc = "Software Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Swrst {
    #[doc = "0: Disabled"]
    Disable = 0,
    #[doc = "1: Enabled"]
    Enable = 1,
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
            false => Swrst::Disable,
            true => Swrst::Enable,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Swrst::Disable
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Swrst::Enable
    }
}
#[doc = "Field `SWRST` writer - Software Reset"]
pub type SwrstW<'a, REG> = crate::BitWriter<'a, REG, Swrst>;
impl<'a, REG> SwrstW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Swrst::Disable)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Swrst::Enable)
    }
}
#[doc = "Fast Access\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fastacc {
    #[doc = "0: Normal"]
    Normal = 0,
    #[doc = "1: Fast"]
    Fast = 1,
}
impl From<Fastacc> for bool {
    #[inline(always)]
    fn from(variant: Fastacc) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FASTACC` reader - Fast Access"]
pub type FastaccR = crate::BitReader<Fastacc>;
impl FastaccR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fastacc {
        match self.bits {
            false => Fastacc::Normal,
            true => Fastacc::Fast,
        }
    }
    #[doc = "Normal"]
    #[inline(always)]
    pub fn is_normal(&self) -> bool {
        *self == Fastacc::Normal
    }
    #[doc = "Fast"]
    #[inline(always)]
    pub fn is_fast(&self) -> bool {
        *self == Fastacc::Fast
    }
}
#[doc = "Field `FASTACC` writer - Fast Access"]
pub type FastaccW<'a, REG> = crate::BitWriter<'a, REG, Fastacc>;
impl<'a, REG> FastaccW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Normal"]
    #[inline(always)]
    pub fn normal(self) -> &'a mut crate::W<REG> {
        self.variant(Fastacc::Normal)
    }
    #[doc = "Fast"]
    #[inline(always)]
    pub fn fast(self) -> &'a mut crate::W<REG> {
        self.variant(Fastacc::Fast)
    }
}
#[doc = "Debug Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dbge {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Emable = 1,
}
impl From<Dbge> for bool {
    #[inline(always)]
    fn from(variant: Dbge) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DBGE` reader - Debug Enable"]
pub type DbgeR = crate::BitReader<Dbge>;
impl DbgeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dbge {
        match self.bits {
            false => Dbge::Disable,
            true => Dbge::Emable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Dbge::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_emable(&self) -> bool {
        *self == Dbge::Emable
    }
}
#[doc = "Field `DBGE` writer - Debug Enable"]
pub type DbgeW<'a, REG> = crate::BitWriter<'a, REG, Dbge>;
impl<'a, REG> DbgeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Dbge::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn emable(self) -> &'a mut crate::W<REG> {
        self.variant(Dbge::Emable)
    }
}
#[doc = "Doze Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dozen {
    #[doc = "0: Enable"]
    Enable = 0,
    #[doc = "1: Disable"]
    Disable = 1,
}
impl From<Dozen> for bool {
    #[inline(always)]
    fn from(variant: Dozen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DOZEN` reader - Doze Enable"]
pub type DozenR = crate::BitReader<Dozen>;
impl DozenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dozen {
        match self.bits {
            false => Dozen::Enable,
            true => Dozen::Disable,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Dozen::Enable
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Dozen::Disable
    }
}
#[doc = "Field `DOZEN` writer - Doze Enable"]
pub type DozenW<'a, REG> = crate::BitWriter<'a, REG, Dozen>;
impl<'a, REG> DozenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Dozen::Enable)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Dozen::Disable)
    }
}
impl R {
    #[doc = "Bit 0 - FLEXIO Enable"]
    #[inline(always)]
    pub fn flexen(&self) -> FlexenR {
        FlexenR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Software Reset"]
    #[inline(always)]
    pub fn swrst(&self) -> SwrstR {
        SwrstR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Fast Access"]
    #[inline(always)]
    pub fn fastacc(&self) -> FastaccR {
        FastaccR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 30 - Debug Enable"]
    #[inline(always)]
    pub fn dbge(&self) -> DbgeR {
        DbgeR::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Doze Enable"]
    #[inline(always)]
    pub fn dozen(&self) -> DozenR {
        DozenR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - FLEXIO Enable"]
    #[inline(always)]
    pub fn flexen(&mut self) -> FlexenW<CtrlSpec> {
        FlexenW::new(self, 0)
    }
    #[doc = "Bit 1 - Software Reset"]
    #[inline(always)]
    pub fn swrst(&mut self) -> SwrstW<CtrlSpec> {
        SwrstW::new(self, 1)
    }
    #[doc = "Bit 2 - Fast Access"]
    #[inline(always)]
    pub fn fastacc(&mut self) -> FastaccW<CtrlSpec> {
        FastaccW::new(self, 2)
    }
    #[doc = "Bit 30 - Debug Enable"]
    #[inline(always)]
    pub fn dbge(&mut self) -> DbgeW<CtrlSpec> {
        DbgeW::new(self, 30)
    }
    #[doc = "Bit 31 - Doze Enable"]
    #[inline(always)]
    pub fn dozen(&mut self) -> DozenW<CtrlSpec> {
        DozenW::new(self, 31)
    }
}
#[doc = "FLEXIO Control\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CtrlSpec;
impl crate::RegisterSpec for CtrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ctrl::R`](R) reader structure"]
impl crate::Readable for CtrlSpec {}
#[doc = "`write(|w| ..)` method takes [`ctrl::W`](W) writer structure"]
impl crate::Writable for CtrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CTRL to value 0"]
impl crate::Resettable for CtrlSpec {}

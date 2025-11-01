#[doc = "Register `CTIMERGLOBALSTARTEN` reader"]
pub type R = crate::R<CtimerglobalstartenSpec>;
#[doc = "Register `CTIMERGLOBALSTARTEN` writer"]
pub type W = crate::W<CtimerglobalstartenSpec>;
#[doc = "Enables the CTIMER0 function clock\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ctimer0ClkEn {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Ctimer0ClkEn> for bool {
    #[inline(always)]
    fn from(variant: Ctimer0ClkEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CTIMER0_CLK_EN` reader - Enables the CTIMER0 function clock"]
pub type Ctimer0ClkEnR = crate::BitReader<Ctimer0ClkEn>;
impl Ctimer0ClkEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ctimer0ClkEn {
        match self.bits {
            false => Ctimer0ClkEn::Disable,
            true => Ctimer0ClkEn::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Ctimer0ClkEn::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Ctimer0ClkEn::Enable
    }
}
#[doc = "Field `CTIMER0_CLK_EN` writer - Enables the CTIMER0 function clock"]
pub type Ctimer0ClkEnW<'a, REG> = crate::BitWriter<'a, REG, Ctimer0ClkEn>;
impl<'a, REG> Ctimer0ClkEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Ctimer0ClkEn::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Ctimer0ClkEn::Enable)
    }
}
#[doc = "Enables the CTIMER1 function clock\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ctimer1ClkEn {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Ctimer1ClkEn> for bool {
    #[inline(always)]
    fn from(variant: Ctimer1ClkEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CTIMER1_CLK_EN` reader - Enables the CTIMER1 function clock"]
pub type Ctimer1ClkEnR = crate::BitReader<Ctimer1ClkEn>;
impl Ctimer1ClkEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ctimer1ClkEn {
        match self.bits {
            false => Ctimer1ClkEn::Disable,
            true => Ctimer1ClkEn::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Ctimer1ClkEn::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Ctimer1ClkEn::Enable
    }
}
#[doc = "Field `CTIMER1_CLK_EN` writer - Enables the CTIMER1 function clock"]
pub type Ctimer1ClkEnW<'a, REG> = crate::BitWriter<'a, REG, Ctimer1ClkEn>;
impl<'a, REG> Ctimer1ClkEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Ctimer1ClkEn::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Ctimer1ClkEn::Enable)
    }
}
#[doc = "Enables the CTIMER2 function clock\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ctimer2ClkEn {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Ctimer2ClkEn> for bool {
    #[inline(always)]
    fn from(variant: Ctimer2ClkEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CTIMER2_CLK_EN` reader - Enables the CTIMER2 function clock"]
pub type Ctimer2ClkEnR = crate::BitReader<Ctimer2ClkEn>;
impl Ctimer2ClkEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ctimer2ClkEn {
        match self.bits {
            false => Ctimer2ClkEn::Disable,
            true => Ctimer2ClkEn::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Ctimer2ClkEn::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Ctimer2ClkEn::Enable
    }
}
#[doc = "Field `CTIMER2_CLK_EN` writer - Enables the CTIMER2 function clock"]
pub type Ctimer2ClkEnW<'a, REG> = crate::BitWriter<'a, REG, Ctimer2ClkEn>;
impl<'a, REG> Ctimer2ClkEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Ctimer2ClkEn::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Ctimer2ClkEn::Enable)
    }
}
#[doc = "Enables the CTIMER3 function clock\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ctimer3ClkEn {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Ctimer3ClkEn> for bool {
    #[inline(always)]
    fn from(variant: Ctimer3ClkEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CTIMER3_CLK_EN` reader - Enables the CTIMER3 function clock"]
pub type Ctimer3ClkEnR = crate::BitReader<Ctimer3ClkEn>;
impl Ctimer3ClkEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ctimer3ClkEn {
        match self.bits {
            false => Ctimer3ClkEn::Disable,
            true => Ctimer3ClkEn::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Ctimer3ClkEn::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Ctimer3ClkEn::Enable
    }
}
#[doc = "Field `CTIMER3_CLK_EN` writer - Enables the CTIMER3 function clock"]
pub type Ctimer3ClkEnW<'a, REG> = crate::BitWriter<'a, REG, Ctimer3ClkEn>;
impl<'a, REG> Ctimer3ClkEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Ctimer3ClkEn::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Ctimer3ClkEn::Enable)
    }
}
#[doc = "Enables the CTIMER4 function clock\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ctimer4ClkEn {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Ctimer4ClkEn> for bool {
    #[inline(always)]
    fn from(variant: Ctimer4ClkEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CTIMER4_CLK_EN` reader - Enables the CTIMER4 function clock"]
pub type Ctimer4ClkEnR = crate::BitReader<Ctimer4ClkEn>;
impl Ctimer4ClkEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ctimer4ClkEn {
        match self.bits {
            false => Ctimer4ClkEn::Disable,
            true => Ctimer4ClkEn::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Ctimer4ClkEn::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Ctimer4ClkEn::Enable
    }
}
#[doc = "Field `CTIMER4_CLK_EN` writer - Enables the CTIMER4 function clock"]
pub type Ctimer4ClkEnW<'a, REG> = crate::BitWriter<'a, REG, Ctimer4ClkEn>;
impl<'a, REG> Ctimer4ClkEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Ctimer4ClkEn::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Ctimer4ClkEn::Enable)
    }
}
impl R {
    #[doc = "Bit 0 - Enables the CTIMER0 function clock"]
    #[inline(always)]
    pub fn ctimer0_clk_en(&self) -> Ctimer0ClkEnR {
        Ctimer0ClkEnR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Enables the CTIMER1 function clock"]
    #[inline(always)]
    pub fn ctimer1_clk_en(&self) -> Ctimer1ClkEnR {
        Ctimer1ClkEnR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Enables the CTIMER2 function clock"]
    #[inline(always)]
    pub fn ctimer2_clk_en(&self) -> Ctimer2ClkEnR {
        Ctimer2ClkEnR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Enables the CTIMER3 function clock"]
    #[inline(always)]
    pub fn ctimer3_clk_en(&self) -> Ctimer3ClkEnR {
        Ctimer3ClkEnR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Enables the CTIMER4 function clock"]
    #[inline(always)]
    pub fn ctimer4_clk_en(&self) -> Ctimer4ClkEnR {
        Ctimer4ClkEnR::new(((self.bits >> 4) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Enables the CTIMER0 function clock"]
    #[inline(always)]
    pub fn ctimer0_clk_en(&mut self) -> Ctimer0ClkEnW<CtimerglobalstartenSpec> {
        Ctimer0ClkEnW::new(self, 0)
    }
    #[doc = "Bit 1 - Enables the CTIMER1 function clock"]
    #[inline(always)]
    pub fn ctimer1_clk_en(&mut self) -> Ctimer1ClkEnW<CtimerglobalstartenSpec> {
        Ctimer1ClkEnW::new(self, 1)
    }
    #[doc = "Bit 2 - Enables the CTIMER2 function clock"]
    #[inline(always)]
    pub fn ctimer2_clk_en(&mut self) -> Ctimer2ClkEnW<CtimerglobalstartenSpec> {
        Ctimer2ClkEnW::new(self, 2)
    }
    #[doc = "Bit 3 - Enables the CTIMER3 function clock"]
    #[inline(always)]
    pub fn ctimer3_clk_en(&mut self) -> Ctimer3ClkEnW<CtimerglobalstartenSpec> {
        Ctimer3ClkEnW::new(self, 3)
    }
    #[doc = "Bit 4 - Enables the CTIMER4 function clock"]
    #[inline(always)]
    pub fn ctimer4_clk_en(&mut self) -> Ctimer4ClkEnW<CtimerglobalstartenSpec> {
        Ctimer4ClkEnW::new(self, 4)
    }
}
#[doc = "CTIMER Global Start Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`ctimerglobalstarten::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctimerglobalstarten::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CtimerglobalstartenSpec;
impl crate::RegisterSpec for CtimerglobalstartenSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ctimerglobalstarten::R`](R) reader structure"]
impl crate::Readable for CtimerglobalstartenSpec {}
#[doc = "`write(|w| ..)` method takes [`ctimerglobalstarten::W`](W) writer structure"]
impl crate::Writable for CtimerglobalstartenSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CTIMERGLOBALSTARTEN to value 0"]
impl crate::Resettable for CtimerglobalstartenSpec {}

#[doc = "Register `ME` reader"]
pub type R = crate::R<MeSpec>;
#[doc = "Register `ME` writer"]
pub type W = crate::W<MeSpec>;
#[doc = "Module Interrupt Wake-up Enable for Module 0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wume0 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Wume0> for bool {
    #[inline(always)]
    fn from(variant: Wume0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUME0` reader - Module Interrupt Wake-up Enable for Module 0"]
pub type Wume0R = crate::BitReader<Wume0>;
impl Wume0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wume0 {
        match self.bits {
            false => Wume0::Disable,
            true => Wume0::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wume0::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Wume0::Enable
    }
}
#[doc = "Field `WUME0` writer - Module Interrupt Wake-up Enable for Module 0"]
pub type Wume0W<'a, REG> = crate::BitWriter<'a, REG, Wume0>;
impl<'a, REG> Wume0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wume0::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Wume0::Enable)
    }
}
#[doc = "Module Interrupt Wake-up Enable for Module 1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wume1 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Wume1> for bool {
    #[inline(always)]
    fn from(variant: Wume1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUME1` reader - Module Interrupt Wake-up Enable for Module 1"]
pub type Wume1R = crate::BitReader<Wume1>;
impl Wume1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wume1 {
        match self.bits {
            false => Wume1::Disable,
            true => Wume1::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wume1::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Wume1::Enable
    }
}
#[doc = "Field `WUME1` writer - Module Interrupt Wake-up Enable for Module 1"]
pub type Wume1W<'a, REG> = crate::BitWriter<'a, REG, Wume1>;
impl<'a, REG> Wume1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wume1::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Wume1::Enable)
    }
}
#[doc = "Module Interrupt Wake-up Enable for Module 2\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wume2 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Wume2> for bool {
    #[inline(always)]
    fn from(variant: Wume2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUME2` reader - Module Interrupt Wake-up Enable for Module 2"]
pub type Wume2R = crate::BitReader<Wume2>;
impl Wume2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wume2 {
        match self.bits {
            false => Wume2::Disable,
            true => Wume2::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wume2::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Wume2::Enable
    }
}
#[doc = "Field `WUME2` writer - Module Interrupt Wake-up Enable for Module 2"]
pub type Wume2W<'a, REG> = crate::BitWriter<'a, REG, Wume2>;
impl<'a, REG> Wume2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wume2::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Wume2::Enable)
    }
}
#[doc = "Module Interrupt Wake-up Enable for Module 3\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wume3 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Wume3> for bool {
    #[inline(always)]
    fn from(variant: Wume3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUME3` reader - Module Interrupt Wake-up Enable for Module 3"]
pub type Wume3R = crate::BitReader<Wume3>;
impl Wume3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wume3 {
        match self.bits {
            false => Wume3::Disable,
            true => Wume3::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wume3::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Wume3::Enable
    }
}
#[doc = "Field `WUME3` writer - Module Interrupt Wake-up Enable for Module 3"]
pub type Wume3W<'a, REG> = crate::BitWriter<'a, REG, Wume3>;
impl<'a, REG> Wume3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wume3::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Wume3::Enable)
    }
}
#[doc = "Module Interrupt Wake-up Enable for Module 6\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wume6 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Wume6> for bool {
    #[inline(always)]
    fn from(variant: Wume6) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUME6` reader - Module Interrupt Wake-up Enable for Module 6"]
pub type Wume6R = crate::BitReader<Wume6>;
impl Wume6R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wume6 {
        match self.bits {
            false => Wume6::Disable,
            true => Wume6::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wume6::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Wume6::Enable
    }
}
#[doc = "Field `WUME6` writer - Module Interrupt Wake-up Enable for Module 6"]
pub type Wume6W<'a, REG> = crate::BitWriter<'a, REG, Wume6>;
impl<'a, REG> Wume6W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wume6::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Wume6::Enable)
    }
}
#[doc = "Module Interrupt Wake-up Enable for Module 8\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wume8 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Wume8> for bool {
    #[inline(always)]
    fn from(variant: Wume8) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUME8` reader - Module Interrupt Wake-up Enable for Module 8"]
pub type Wume8R = crate::BitReader<Wume8>;
impl Wume8R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wume8 {
        match self.bits {
            false => Wume8::Disable,
            true => Wume8::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wume8::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Wume8::Enable
    }
}
#[doc = "Field `WUME8` writer - Module Interrupt Wake-up Enable for Module 8"]
pub type Wume8W<'a, REG> = crate::BitWriter<'a, REG, Wume8>;
impl<'a, REG> Wume8W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wume8::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Wume8::Enable)
    }
}
impl R {
    #[doc = "Bit 0 - Module Interrupt Wake-up Enable for Module 0"]
    #[inline(always)]
    pub fn wume0(&self) -> Wume0R {
        Wume0R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Module Interrupt Wake-up Enable for Module 1"]
    #[inline(always)]
    pub fn wume1(&self) -> Wume1R {
        Wume1R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Module Interrupt Wake-up Enable for Module 2"]
    #[inline(always)]
    pub fn wume2(&self) -> Wume2R {
        Wume2R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Module Interrupt Wake-up Enable for Module 3"]
    #[inline(always)]
    pub fn wume3(&self) -> Wume3R {
        Wume3R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 6 - Module Interrupt Wake-up Enable for Module 6"]
    #[inline(always)]
    pub fn wume6(&self) -> Wume6R {
        Wume6R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 8 - Module Interrupt Wake-up Enable for Module 8"]
    #[inline(always)]
    pub fn wume8(&self) -> Wume8R {
        Wume8R::new(((self.bits >> 8) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Module Interrupt Wake-up Enable for Module 0"]
    #[inline(always)]
    pub fn wume0(&mut self) -> Wume0W<MeSpec> {
        Wume0W::new(self, 0)
    }
    #[doc = "Bit 1 - Module Interrupt Wake-up Enable for Module 1"]
    #[inline(always)]
    pub fn wume1(&mut self) -> Wume1W<MeSpec> {
        Wume1W::new(self, 1)
    }
    #[doc = "Bit 2 - Module Interrupt Wake-up Enable for Module 2"]
    #[inline(always)]
    pub fn wume2(&mut self) -> Wume2W<MeSpec> {
        Wume2W::new(self, 2)
    }
    #[doc = "Bit 3 - Module Interrupt Wake-up Enable for Module 3"]
    #[inline(always)]
    pub fn wume3(&mut self) -> Wume3W<MeSpec> {
        Wume3W::new(self, 3)
    }
    #[doc = "Bit 6 - Module Interrupt Wake-up Enable for Module 6"]
    #[inline(always)]
    pub fn wume6(&mut self) -> Wume6W<MeSpec> {
        Wume6W::new(self, 6)
    }
    #[doc = "Bit 8 - Module Interrupt Wake-up Enable for Module 8"]
    #[inline(always)]
    pub fn wume8(&mut self) -> Wume8W<MeSpec> {
        Wume8W::new(self, 8)
    }
}
#[doc = "Module Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`me::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`me::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MeSpec;
impl crate::RegisterSpec for MeSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`me::R`](R) reader structure"]
impl crate::Readable for MeSpec {}
#[doc = "`write(|w| ..)` method takes [`me::W`](W) writer structure"]
impl crate::Writable for MeSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets ME to value 0"]
impl crate::Resettable for MeSpec {}

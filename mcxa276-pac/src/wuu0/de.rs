#[doc = "Register `DE` reader"]
pub type R = crate::R<DeSpec>;
#[doc = "Register `DE` writer"]
pub type W = crate::W<DeSpec>;
#[doc = "DMA/Trigger Wake-up Enable for Module 4\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wude4 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Wude4> for bool {
    #[inline(always)]
    fn from(variant: Wude4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUDE4` reader - DMA/Trigger Wake-up Enable for Module 4"]
pub type Wude4R = crate::BitReader<Wude4>;
impl Wude4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wude4 {
        match self.bits {
            false => Wude4::Disable,
            true => Wude4::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wude4::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Wude4::Enable
    }
}
#[doc = "Field `WUDE4` writer - DMA/Trigger Wake-up Enable for Module 4"]
pub type Wude4W<'a, REG> = crate::BitWriter<'a, REG, Wude4>;
impl<'a, REG> Wude4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wude4::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Wude4::Enable)
    }
}
#[doc = "DMA/Trigger Wake-up Enable for Module 6\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wude6 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Wude6> for bool {
    #[inline(always)]
    fn from(variant: Wude6) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUDE6` reader - DMA/Trigger Wake-up Enable for Module 6"]
pub type Wude6R = crate::BitReader<Wude6>;
impl Wude6R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wude6 {
        match self.bits {
            false => Wude6::Disable,
            true => Wude6::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wude6::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Wude6::Enable
    }
}
#[doc = "Field `WUDE6` writer - DMA/Trigger Wake-up Enable for Module 6"]
pub type Wude6W<'a, REG> = crate::BitWriter<'a, REG, Wude6>;
impl<'a, REG> Wude6W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wude6::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Wude6::Enable)
    }
}
#[doc = "DMA/Trigger Wake-up Enable for Module 8\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wude8 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Wude8> for bool {
    #[inline(always)]
    fn from(variant: Wude8) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUDE8` reader - DMA/Trigger Wake-up Enable for Module 8"]
pub type Wude8R = crate::BitReader<Wude8>;
impl Wude8R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wude8 {
        match self.bits {
            false => Wude8::Disable,
            true => Wude8::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wude8::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Wude8::Enable
    }
}
#[doc = "Field `WUDE8` writer - DMA/Trigger Wake-up Enable for Module 8"]
pub type Wude8W<'a, REG> = crate::BitWriter<'a, REG, Wude8>;
impl<'a, REG> Wude8W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wude8::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Wude8::Enable)
    }
}
impl R {
    #[doc = "Bit 4 - DMA/Trigger Wake-up Enable for Module 4"]
    #[inline(always)]
    pub fn wude4(&self) -> Wude4R {
        Wude4R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 6 - DMA/Trigger Wake-up Enable for Module 6"]
    #[inline(always)]
    pub fn wude6(&self) -> Wude6R {
        Wude6R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 8 - DMA/Trigger Wake-up Enable for Module 8"]
    #[inline(always)]
    pub fn wude8(&self) -> Wude8R {
        Wude8R::new(((self.bits >> 8) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 4 - DMA/Trigger Wake-up Enable for Module 4"]
    #[inline(always)]
    pub fn wude4(&mut self) -> Wude4W<DeSpec> {
        Wude4W::new(self, 4)
    }
    #[doc = "Bit 6 - DMA/Trigger Wake-up Enable for Module 6"]
    #[inline(always)]
    pub fn wude6(&mut self) -> Wude6W<DeSpec> {
        Wude6W::new(self, 6)
    }
    #[doc = "Bit 8 - DMA/Trigger Wake-up Enable for Module 8"]
    #[inline(always)]
    pub fn wude8(&mut self) -> Wude8W<DeSpec> {
        Wude8W::new(self, 8)
    }
}
#[doc = "Module DMA/Trigger Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`de::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`de::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct DeSpec;
impl crate::RegisterSpec for DeSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`de::R`](R) reader structure"]
impl crate::Readable for DeSpec {}
#[doc = "`write(|w| ..)` method takes [`de::W`](W) writer structure"]
impl crate::Writable for DeSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets DE to value 0"]
impl crate::Resettable for DeSpec {}

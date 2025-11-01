#[doc = "Register `FLASHCR` reader"]
pub type R = crate::R<FlashcrSpec>;
#[doc = "Register `FLASHCR` writer"]
pub type W = crate::W<FlashcrSpec>;
#[doc = "Flash Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flashdis {
    #[doc = "0: No effect"]
    Disabled = 0,
    #[doc = "1: Flash memory is disabled"]
    Enabled = 1,
}
impl From<Flashdis> for bool {
    #[inline(always)]
    fn from(variant: Flashdis) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FLASHDIS` reader - Flash Disable"]
pub type FlashdisR = crate::BitReader<Flashdis>;
impl FlashdisR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Flashdis {
        match self.bits {
            false => Flashdis::Disabled,
            true => Flashdis::Enabled,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Flashdis::Disabled
    }
    #[doc = "Flash memory is disabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Flashdis::Enabled
    }
}
#[doc = "Field `FLASHDIS` writer - Flash Disable"]
pub type FlashdisW<'a, REG> = crate::BitWriter<'a, REG, Flashdis>;
impl<'a, REG> FlashdisW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Flashdis::Disabled)
    }
    #[doc = "Flash memory is disabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Flashdis::Enabled)
    }
}
#[doc = "Flash Doze\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flashdoze {
    #[doc = "0: No effect"]
    Disabled = 0,
    #[doc = "1: Flash memory is disabled when core is sleeping (CKMODE > 0)"]
    Enabled = 1,
}
impl From<Flashdoze> for bool {
    #[inline(always)]
    fn from(variant: Flashdoze) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FLASHDOZE` reader - Flash Doze"]
pub type FlashdozeR = crate::BitReader<Flashdoze>;
impl FlashdozeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Flashdoze {
        match self.bits {
            false => Flashdoze::Disabled,
            true => Flashdoze::Enabled,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Flashdoze::Disabled
    }
    #[doc = "Flash memory is disabled when core is sleeping (CKMODE > 0)"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Flashdoze::Enabled
    }
}
#[doc = "Field `FLASHDOZE` writer - Flash Doze"]
pub type FlashdozeW<'a, REG> = crate::BitWriter<'a, REG, Flashdoze>;
impl<'a, REG> FlashdozeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Flashdoze::Disabled)
    }
    #[doc = "Flash memory is disabled when core is sleeping (CKMODE > 0)"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Flashdoze::Enabled)
    }
}
#[doc = "Flash Wake\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flashwake {
    #[doc = "0: No effect"]
    Disabled = 0,
    #[doc = "1: Flash memory is not disabled during flash memory accesses"]
    Enabled = 1,
}
impl From<Flashwake> for bool {
    #[inline(always)]
    fn from(variant: Flashwake) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FLASHWAKE` reader - Flash Wake"]
pub type FlashwakeR = crate::BitReader<Flashwake>;
impl FlashwakeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Flashwake {
        match self.bits {
            false => Flashwake::Disabled,
            true => Flashwake::Enabled,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Flashwake::Disabled
    }
    #[doc = "Flash memory is not disabled during flash memory accesses"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Flashwake::Enabled
    }
}
#[doc = "Field `FLASHWAKE` writer - Flash Wake"]
pub type FlashwakeW<'a, REG> = crate::BitWriter<'a, REG, Flashwake>;
impl<'a, REG> FlashwakeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Flashwake::Disabled)
    }
    #[doc = "Flash memory is not disabled during flash memory accesses"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Flashwake::Enabled)
    }
}
impl R {
    #[doc = "Bit 0 - Flash Disable"]
    #[inline(always)]
    pub fn flashdis(&self) -> FlashdisR {
        FlashdisR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Flash Doze"]
    #[inline(always)]
    pub fn flashdoze(&self) -> FlashdozeR {
        FlashdozeR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Flash Wake"]
    #[inline(always)]
    pub fn flashwake(&self) -> FlashwakeR {
        FlashwakeR::new(((self.bits >> 2) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Flash Disable"]
    #[inline(always)]
    pub fn flashdis(&mut self) -> FlashdisW<FlashcrSpec> {
        FlashdisW::new(self, 0)
    }
    #[doc = "Bit 1 - Flash Doze"]
    #[inline(always)]
    pub fn flashdoze(&mut self) -> FlashdozeW<FlashcrSpec> {
        FlashdozeW::new(self, 1)
    }
    #[doc = "Bit 2 - Flash Wake"]
    #[inline(always)]
    pub fn flashwake(&mut self) -> FlashwakeW<FlashcrSpec> {
        FlashwakeW::new(self, 2)
    }
}
#[doc = "Flash Control\n\nYou can [`read`](crate::Reg::read) this register and get [`flashcr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flashcr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FlashcrSpec;
impl crate::RegisterSpec for FlashcrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`flashcr::R`](R) reader structure"]
impl crate::Readable for FlashcrSpec {}
#[doc = "`write(|w| ..)` method takes [`flashcr::W`](W) writer structure"]
impl crate::Writable for FlashcrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FLASHCR to value 0"]
impl crate::Resettable for FlashcrSpec {}

#[doc = "Register `CFG2` reader"]
pub type R = crate::R<Cfg2Spec>;
#[doc = "Register `CFG2` writer"]
pub type W = crate::W<Cfg2Spec>;
#[doc = "Field `JLEFT` reader - Justified Left Enable register"]
pub type JleftR = crate::BitReader;
#[doc = "Field `JLEFT` writer - Justified Left Enable register"]
pub type JleftW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "High Speed Enable register\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hs {
    #[doc = "0: High speed conversion mode disabled"]
    Disabled = 0,
    #[doc = "1: High speed conversion mode enabled"]
    Enabled = 1,
}
impl From<Hs> for bool {
    #[inline(always)]
    fn from(variant: Hs) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HS` reader - High Speed Enable register"]
pub type HsR = crate::BitReader<Hs>;
impl HsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Hs {
        match self.bits {
            false => Hs::Disabled,
            true => Hs::Enabled,
        }
    }
    #[doc = "High speed conversion mode disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Hs::Disabled
    }
    #[doc = "High speed conversion mode enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Hs::Enabled
    }
}
#[doc = "Field `HS` writer - High Speed Enable register"]
pub type HsW<'a, REG> = crate::BitWriter<'a, REG, Hs>;
impl<'a, REG> HsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "High speed conversion mode disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Hs::Disabled)
    }
    #[doc = "High speed conversion mode enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Hs::Enabled)
    }
}
#[doc = "High Speed Extra register\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hsextra {
    #[doc = "0: No extra cycle added"]
    Hsextra0 = 0,
    #[doc = "1: Extra cycle added"]
    Hsextra1 = 1,
}
impl From<Hsextra> for bool {
    #[inline(always)]
    fn from(variant: Hsextra) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HSEXTRA` reader - High Speed Extra register"]
pub type HsextraR = crate::BitReader<Hsextra>;
impl HsextraR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Hsextra {
        match self.bits {
            false => Hsextra::Hsextra0,
            true => Hsextra::Hsextra1,
        }
    }
    #[doc = "No extra cycle added"]
    #[inline(always)]
    pub fn is_hsextra_0(&self) -> bool {
        *self == Hsextra::Hsextra0
    }
    #[doc = "Extra cycle added"]
    #[inline(always)]
    pub fn is_hsextra_1(&self) -> bool {
        *self == Hsextra::Hsextra1
    }
}
#[doc = "Field `HSEXTRA` writer - High Speed Extra register"]
pub type HsextraW<'a, REG> = crate::BitWriter<'a, REG, Hsextra>;
impl<'a, REG> HsextraW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No extra cycle added"]
    #[inline(always)]
    pub fn hsextra_0(self) -> &'a mut crate::W<REG> {
        self.variant(Hsextra::Hsextra0)
    }
    #[doc = "Extra cycle added"]
    #[inline(always)]
    pub fn hsextra_1(self) -> &'a mut crate::W<REG> {
        self.variant(Hsextra::Hsextra1)
    }
}
#[doc = "Field `TUNE` reader - Tune Mode register"]
pub type TuneR = crate::FieldReader;
#[doc = "Field `TUNE` writer - Tune Mode register"]
pub type TuneW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
impl R {
    #[doc = "Bit 8 - Justified Left Enable register"]
    #[inline(always)]
    pub fn jleft(&self) -> JleftR {
        JleftR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - High Speed Enable register"]
    #[inline(always)]
    pub fn hs(&self) -> HsR {
        HsR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - High Speed Extra register"]
    #[inline(always)]
    pub fn hsextra(&self) -> HsextraR {
        HsextraR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bits 12:13 - Tune Mode register"]
    #[inline(always)]
    pub fn tune(&self) -> TuneR {
        TuneR::new(((self.bits >> 12) & 3) as u8)
    }
}
impl W {
    #[doc = "Bit 8 - Justified Left Enable register"]
    #[inline(always)]
    pub fn jleft(&mut self) -> JleftW<Cfg2Spec> {
        JleftW::new(self, 8)
    }
    #[doc = "Bit 9 - High Speed Enable register"]
    #[inline(always)]
    pub fn hs(&mut self) -> HsW<Cfg2Spec> {
        HsW::new(self, 9)
    }
    #[doc = "Bit 10 - High Speed Extra register"]
    #[inline(always)]
    pub fn hsextra(&mut self) -> HsextraW<Cfg2Spec> {
        HsextraW::new(self, 10)
    }
    #[doc = "Bits 12:13 - Tune Mode register"]
    #[inline(always)]
    pub fn tune(&mut self) -> TuneW<Cfg2Spec> {
        TuneW::new(self, 12)
    }
}
#[doc = "Configuration 2 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cfg2::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cfg2::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Cfg2Spec;
impl crate::RegisterSpec for Cfg2Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`cfg2::R`](R) reader structure"]
impl crate::Readable for Cfg2Spec {}
#[doc = "`write(|w| ..)` method takes [`cfg2::W`](W) writer structure"]
impl crate::Writable for Cfg2Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CFG2 to value 0x1000"]
impl crate::Resettable for Cfg2Spec {
    const RESET_VALUE: u32 = 0x1000;
}

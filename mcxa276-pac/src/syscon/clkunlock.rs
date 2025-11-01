#[doc = "Register `CLKUNLOCK` reader"]
pub type R = crate::R<ClkunlockSpec>;
#[doc = "Register `CLKUNLOCK` writer"]
pub type W = crate::W<ClkunlockSpec>;
#[doc = "Controls clock configuration registers access (for example, SLOWCLKDIV, BUSCLKDIV, AHBCLKDIV, FROHFDIV, FROLFDIV, PLLxCLKDIV, MRCC_xxx_CLKDIV, MRCC_xxx_CLKSEL, MRCC_GLB_xxx)\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unlock {
    #[doc = "0: Updates are allowed to all clock configuration registers"]
    Enable = 0,
    #[doc = "1: Freezes all clock configuration registers update."]
    Freeze = 1,
}
impl From<Unlock> for bool {
    #[inline(always)]
    fn from(variant: Unlock) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `UNLOCK` reader - Controls clock configuration registers access (for example, SLOWCLKDIV, BUSCLKDIV, AHBCLKDIV, FROHFDIV, FROLFDIV, PLLxCLKDIV, MRCC_xxx_CLKDIV, MRCC_xxx_CLKSEL, MRCC_GLB_xxx)"]
pub type UnlockR = crate::BitReader<Unlock>;
impl UnlockR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Unlock {
        match self.bits {
            false => Unlock::Enable,
            true => Unlock::Freeze,
        }
    }
    #[doc = "Updates are allowed to all clock configuration registers"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Unlock::Enable
    }
    #[doc = "Freezes all clock configuration registers update."]
    #[inline(always)]
    pub fn is_freeze(&self) -> bool {
        *self == Unlock::Freeze
    }
}
#[doc = "Field `UNLOCK` writer - Controls clock configuration registers access (for example, SLOWCLKDIV, BUSCLKDIV, AHBCLKDIV, FROHFDIV, FROLFDIV, PLLxCLKDIV, MRCC_xxx_CLKDIV, MRCC_xxx_CLKSEL, MRCC_GLB_xxx)"]
pub type UnlockW<'a, REG> = crate::BitWriter<'a, REG, Unlock>;
impl<'a, REG> UnlockW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Updates are allowed to all clock configuration registers"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Unlock::Enable)
    }
    #[doc = "Freezes all clock configuration registers update."]
    #[inline(always)]
    pub fn freeze(self) -> &'a mut crate::W<REG> {
        self.variant(Unlock::Freeze)
    }
}
impl R {
    #[doc = "Bit 0 - Controls clock configuration registers access (for example, SLOWCLKDIV, BUSCLKDIV, AHBCLKDIV, FROHFDIV, FROLFDIV, PLLxCLKDIV, MRCC_xxx_CLKDIV, MRCC_xxx_CLKSEL, MRCC_GLB_xxx)"]
    #[inline(always)]
    pub fn unlock(&self) -> UnlockR {
        UnlockR::new((self.bits & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Controls clock configuration registers access (for example, SLOWCLKDIV, BUSCLKDIV, AHBCLKDIV, FROHFDIV, FROLFDIV, PLLxCLKDIV, MRCC_xxx_CLKDIV, MRCC_xxx_CLKSEL, MRCC_GLB_xxx)"]
    #[inline(always)]
    pub fn unlock(&mut self) -> UnlockW<ClkunlockSpec> {
        UnlockW::new(self, 0)
    }
}
#[doc = "Clock Configuration Unlock\n\nYou can [`read`](crate::Reg::read) this register and get [`clkunlock::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`clkunlock::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ClkunlockSpec;
impl crate::RegisterSpec for ClkunlockSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`clkunlock::R`](R) reader structure"]
impl crate::Readable for ClkunlockSpec {}
#[doc = "`write(|w| ..)` method takes [`clkunlock::W`](W) writer structure"]
impl crate::Writable for ClkunlockSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CLKUNLOCK to value 0"]
impl crate::Resettable for ClkunlockSpec {}

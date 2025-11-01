#[doc = "Register `FM0` reader"]
pub type R = crate::R<Fm0Spec>;
#[doc = "Register `FM0` writer"]
pub type W = crate::W<Fm0Spec>;
#[doc = "Boot Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Forcecfg {
    #[doc = "0: No effect"]
    Disabled = 0,
    #[doc = "1: Asserts"]
    Enabled = 1,
}
impl From<Forcecfg> for bool {
    #[inline(always)]
    fn from(variant: Forcecfg) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FORCECFG` reader - Boot Configuration"]
pub type ForcecfgR = crate::BitReader<Forcecfg>;
impl ForcecfgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Forcecfg {
        match self.bits {
            false => Forcecfg::Disabled,
            true => Forcecfg::Enabled,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Forcecfg::Disabled
    }
    #[doc = "Asserts"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Forcecfg::Enabled
    }
}
#[doc = "Field `FORCECFG` writer - Boot Configuration"]
pub type ForcecfgW<'a, REG> = crate::BitWriter<'a, REG, Forcecfg>;
impl<'a, REG> ForcecfgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Forcecfg::Disabled)
    }
    #[doc = "Asserts"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Forcecfg::Enabled)
    }
}
impl R {
    #[doc = "Bit 0 - Boot Configuration"]
    #[inline(always)]
    pub fn forcecfg(&self) -> ForcecfgR {
        ForcecfgR::new((self.bits & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Boot Configuration"]
    #[inline(always)]
    pub fn forcecfg(&mut self) -> ForcecfgW<Fm0Spec> {
        ForcecfgW::new(self, 0)
    }
}
#[doc = "Force Mode\n\nYou can [`read`](crate::Reg::read) this register and get [`fm0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fm0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Fm0Spec;
impl crate::RegisterSpec for Fm0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`fm0::R`](R) reader structure"]
impl crate::Readable for Fm0Spec {}
#[doc = "`write(|w| ..)` method takes [`fm0::W`](W) writer structure"]
impl crate::Writable for Fm0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FM0 to value 0"]
impl crate::Resettable for Fm0Spec {}

#[doc = "Register `CORECTL` reader"]
pub type R = crate::R<CorectlSpec>;
#[doc = "Register `CORECTL` writer"]
pub type W = crate::W<CorectlSpec>;
#[doc = "Non-maskable Pin Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Npie {
    #[doc = "0: Disables"]
    Disabled = 0,
    #[doc = "1: Enables"]
    Enabled = 1,
}
impl From<Npie> for bool {
    #[inline(always)]
    fn from(variant: Npie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NPIE` reader - Non-maskable Pin Interrupt Enable"]
pub type NpieR = crate::BitReader<Npie>;
impl NpieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Npie {
        match self.bits {
            false => Npie::Disabled,
            true => Npie::Enabled,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Npie::Disabled
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Npie::Enabled
    }
}
#[doc = "Field `NPIE` writer - Non-maskable Pin Interrupt Enable"]
pub type NpieW<'a, REG> = crate::BitWriter<'a, REG, Npie>;
impl<'a, REG> NpieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Npie::Disabled)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Npie::Enabled)
    }
}
impl R {
    #[doc = "Bit 0 - Non-maskable Pin Interrupt Enable"]
    #[inline(always)]
    pub fn npie(&self) -> NpieR {
        NpieR::new((self.bits & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Non-maskable Pin Interrupt Enable"]
    #[inline(always)]
    pub fn npie(&mut self) -> NpieW<CorectlSpec> {
        NpieW::new(self, 0)
    }
}
#[doc = "Core Control\n\nYou can [`read`](crate::Reg::read) this register and get [`corectl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`corectl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CorectlSpec;
impl crate::RegisterSpec for CorectlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`corectl::R`](R) reader structure"]
impl crate::Readable for CorectlSpec {}
#[doc = "`write(|w| ..)` method takes [`corectl::W`](W) writer structure"]
impl crate::Writable for CorectlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CORECTL to value 0"]
impl crate::Resettable for CorectlSpec {}

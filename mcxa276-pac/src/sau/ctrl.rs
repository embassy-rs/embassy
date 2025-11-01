#[doc = "Register `CTRL` reader"]
pub type R = crate::R<CtrlSpec>;
#[doc = "Register `CTRL` writer"]
pub type W = crate::W<CtrlSpec>;
#[doc = "Enable. Enables the SAU. This bit is RAZ/WI when the Security Extension is implemented without an SAU region.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Enable {
    #[doc = "0: The SAU is disabled."]
    Disabled = 0,
    #[doc = "1: The SAU is enabled."]
    Enabled = 1,
}
impl From<Enable> for bool {
    #[inline(always)]
    fn from(variant: Enable) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ENABLE` reader - Enable. Enables the SAU. This bit is RAZ/WI when the Security Extension is implemented without an SAU region."]
pub type EnableR = crate::BitReader<Enable>;
impl EnableR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Enable {
        match self.bits {
            false => Enable::Disabled,
            true => Enable::Enabled,
        }
    }
    #[doc = "The SAU is disabled."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Enable::Disabled
    }
    #[doc = "The SAU is enabled."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Enable::Enabled
    }
}
#[doc = "Field `ENABLE` writer - Enable. Enables the SAU. This bit is RAZ/WI when the Security Extension is implemented without an SAU region."]
pub type EnableW<'a, REG> = crate::BitWriter<'a, REG, Enable>;
impl<'a, REG> EnableW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The SAU is disabled."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Enable::Disabled)
    }
    #[doc = "The SAU is enabled."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Enable::Enabled)
    }
}
#[doc = "All Non-secure.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Allns {
    #[doc = "0: Memory is marked as Secure and is not Non-secure callable."]
    SecuredMemory = 0,
    #[doc = "1: Memory is marked as Non-secure."]
    NonSecuredMemory = 1,
}
impl From<Allns> for bool {
    #[inline(always)]
    fn from(variant: Allns) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ALLNS` reader - All Non-secure."]
pub type AllnsR = crate::BitReader<Allns>;
impl AllnsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Allns {
        match self.bits {
            false => Allns::SecuredMemory,
            true => Allns::NonSecuredMemory,
        }
    }
    #[doc = "Memory is marked as Secure and is not Non-secure callable."]
    #[inline(always)]
    pub fn is_secured_memory(&self) -> bool {
        *self == Allns::SecuredMemory
    }
    #[doc = "Memory is marked as Non-secure."]
    #[inline(always)]
    pub fn is_non_secured_memory(&self) -> bool {
        *self == Allns::NonSecuredMemory
    }
}
#[doc = "Field `ALLNS` writer - All Non-secure."]
pub type AllnsW<'a, REG> = crate::BitWriter<'a, REG, Allns>;
impl<'a, REG> AllnsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Memory is marked as Secure and is not Non-secure callable."]
    #[inline(always)]
    pub fn secured_memory(self) -> &'a mut crate::W<REG> {
        self.variant(Allns::SecuredMemory)
    }
    #[doc = "Memory is marked as Non-secure."]
    #[inline(always)]
    pub fn non_secured_memory(self) -> &'a mut crate::W<REG> {
        self.variant(Allns::NonSecuredMemory)
    }
}
impl R {
    #[doc = "Bit 0 - Enable. Enables the SAU. This bit is RAZ/WI when the Security Extension is implemented without an SAU region."]
    #[inline(always)]
    pub fn enable(&self) -> EnableR {
        EnableR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - All Non-secure."]
    #[inline(always)]
    pub fn allns(&self) -> AllnsR {
        AllnsR::new(((self.bits >> 1) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Enable. Enables the SAU. This bit is RAZ/WI when the Security Extension is implemented without an SAU region."]
    #[inline(always)]
    pub fn enable(&mut self) -> EnableW<CtrlSpec> {
        EnableW::new(self, 0)
    }
    #[doc = "Bit 1 - All Non-secure."]
    #[inline(always)]
    pub fn allns(&mut self) -> AllnsW<CtrlSpec> {
        AllnsW::new(self, 1)
    }
}
#[doc = "Security Attribution Unit Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
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

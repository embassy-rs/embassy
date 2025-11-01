#[doc = "Register `RLAR` reader"]
pub type R = crate::R<RlarSpec>;
#[doc = "Register `RLAR` writer"]
pub type W = crate::W<RlarSpec>;
#[doc = "Enable. SAU region enable.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Enable {
    #[doc = "0: SAU region is enabled."]
    Enabled = 0,
    #[doc = "1: SAU region is disabled."]
    Disabled = 1,
}
impl From<Enable> for bool {
    #[inline(always)]
    fn from(variant: Enable) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ENABLE` reader - Enable. SAU region enable."]
pub type EnableR = crate::BitReader<Enable>;
impl EnableR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Enable {
        match self.bits {
            false => Enable::Enabled,
            true => Enable::Disabled,
        }
    }
    #[doc = "SAU region is enabled."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Enable::Enabled
    }
    #[doc = "SAU region is disabled."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Enable::Disabled
    }
}
#[doc = "Field `ENABLE` writer - Enable. SAU region enable."]
pub type EnableW<'a, REG> = crate::BitWriter<'a, REG, Enable>;
impl<'a, REG> EnableW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "SAU region is enabled."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Enable::Enabled)
    }
    #[doc = "SAU region is disabled."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Enable::Disabled)
    }
}
#[doc = "Non-secure callable. Controls whether Non-secure state is permitted to execute an SG instruction from this region.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nsc {
    #[doc = "0: Region is not Non-secure callable."]
    NotNonSecureCallable = 0,
    #[doc = "1: Region is Non-secure callable."]
    NonSecureCallable = 1,
}
impl From<Nsc> for bool {
    #[inline(always)]
    fn from(variant: Nsc) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NSC` reader - Non-secure callable. Controls whether Non-secure state is permitted to execute an SG instruction from this region."]
pub type NscR = crate::BitReader<Nsc>;
impl NscR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nsc {
        match self.bits {
            false => Nsc::NotNonSecureCallable,
            true => Nsc::NonSecureCallable,
        }
    }
    #[doc = "Region is not Non-secure callable."]
    #[inline(always)]
    pub fn is_not_non_secure_callable(&self) -> bool {
        *self == Nsc::NotNonSecureCallable
    }
    #[doc = "Region is Non-secure callable."]
    #[inline(always)]
    pub fn is_non_secure_callable(&self) -> bool {
        *self == Nsc::NonSecureCallable
    }
}
#[doc = "Field `NSC` writer - Non-secure callable. Controls whether Non-secure state is permitted to execute an SG instruction from this region."]
pub type NscW<'a, REG> = crate::BitWriter<'a, REG, Nsc>;
impl<'a, REG> NscW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Region is not Non-secure callable."]
    #[inline(always)]
    pub fn not_non_secure_callable(self) -> &'a mut crate::W<REG> {
        self.variant(Nsc::NotNonSecureCallable)
    }
    #[doc = "Region is Non-secure callable."]
    #[inline(always)]
    pub fn non_secure_callable(self) -> &'a mut crate::W<REG> {
        self.variant(Nsc::NonSecureCallable)
    }
}
#[doc = "Field `LADDR` reader - Limit address. Holds bits\\[31:5\\] of the limit address for the selected SAU region. Bits\\[4:0\\] of the limit address are defined as 0x1F."]
pub type LaddrR = crate::FieldReader<u32>;
#[doc = "Field `LADDR` writer - Limit address. Holds bits\\[31:5\\] of the limit address for the selected SAU region. Bits\\[4:0\\] of the limit address are defined as 0x1F."]
pub type LaddrW<'a, REG> = crate::FieldWriter<'a, REG, 27, u32>;
impl R {
    #[doc = "Bit 0 - Enable. SAU region enable."]
    #[inline(always)]
    pub fn enable(&self) -> EnableR {
        EnableR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Non-secure callable. Controls whether Non-secure state is permitted to execute an SG instruction from this region."]
    #[inline(always)]
    pub fn nsc(&self) -> NscR {
        NscR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bits 5:31 - Limit address. Holds bits\\[31:5\\] of the limit address for the selected SAU region. Bits\\[4:0\\] of the limit address are defined as 0x1F."]
    #[inline(always)]
    pub fn laddr(&self) -> LaddrR {
        LaddrR::new((self.bits >> 5) & 0x07ff_ffff)
    }
}
impl W {
    #[doc = "Bit 0 - Enable. SAU region enable."]
    #[inline(always)]
    pub fn enable(&mut self) -> EnableW<RlarSpec> {
        EnableW::new(self, 0)
    }
    #[doc = "Bit 1 - Non-secure callable. Controls whether Non-secure state is permitted to execute an SG instruction from this region."]
    #[inline(always)]
    pub fn nsc(&mut self) -> NscW<RlarSpec> {
        NscW::new(self, 1)
    }
    #[doc = "Bits 5:31 - Limit address. Holds bits\\[31:5\\] of the limit address for the selected SAU region. Bits\\[4:0\\] of the limit address are defined as 0x1F."]
    #[inline(always)]
    pub fn laddr(&mut self) -> LaddrW<RlarSpec> {
        LaddrW::new(self, 5)
    }
}
#[doc = "Security Attribution Unit Region Limit Address Register\n\nYou can [`read`](crate::Reg::read) this register and get [`rlar::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rlar::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RlarSpec;
impl crate::RegisterSpec for RlarSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`rlar::R`](R) reader structure"]
impl crate::Readable for RlarSpec {}
#[doc = "`write(|w| ..)` method takes [`rlar::W`](W) writer structure"]
impl crate::Writable for RlarSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RLAR to value 0"]
impl crate::Resettable for RlarSpec {}

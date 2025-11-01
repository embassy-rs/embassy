#[doc = "Register `DBGCTL` reader"]
pub type R = crate::R<DbgctlSpec>;
#[doc = "Register `DBGCTL` writer"]
pub type W = crate::W<DbgctlSpec>;
#[doc = "Sleep Or Debug\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sod {
    #[doc = "0: Remains enabled"]
    Disabled = 0,
    #[doc = "1: Disabled"]
    Enabled = 1,
}
impl From<Sod> for bool {
    #[inline(always)]
    fn from(variant: Sod) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SOD` reader - Sleep Or Debug"]
pub type SodR = crate::BitReader<Sod>;
impl SodR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sod {
        match self.bits {
            false => Sod::Disabled,
            true => Sod::Enabled,
        }
    }
    #[doc = "Remains enabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Sod::Disabled
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Sod::Enabled
    }
}
#[doc = "Field `SOD` writer - Sleep Or Debug"]
pub type SodW<'a, REG> = crate::BitWriter<'a, REG, Sod>;
impl<'a, REG> SodW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Remains enabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Sod::Disabled)
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Sod::Enabled)
    }
}
impl R {
    #[doc = "Bit 0 - Sleep Or Debug"]
    #[inline(always)]
    pub fn sod(&self) -> SodR {
        SodR::new((self.bits & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Sleep Or Debug"]
    #[inline(always)]
    pub fn sod(&mut self) -> SodW<DbgctlSpec> {
        SodW::new(self, 0)
    }
}
#[doc = "Debug Control\n\nYou can [`read`](crate::Reg::read) this register and get [`dbgctl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`dbgctl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct DbgctlSpec;
impl crate::RegisterSpec for DbgctlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`dbgctl::R`](R) reader structure"]
impl crate::Readable for DbgctlSpec {}
#[doc = "`write(|w| ..)` method takes [`dbgctl::W`](W) writer structure"]
impl crate::Writable for DbgctlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets DBGCTL to value 0"]
impl crate::Resettable for DbgctlSpec {}

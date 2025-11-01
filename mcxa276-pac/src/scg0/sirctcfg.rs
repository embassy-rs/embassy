#[doc = "Register `SIRCTCFG` reader"]
pub type R = crate::R<SirctcfgSpec>;
#[doc = "Register `SIRCTCFG` writer"]
pub type W = crate::W<SirctcfgSpec>;
#[doc = "Trim Source\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Trimsrc {
    #[doc = "2: SOSC. This option requires that SOSC be divided using the TRIMDIV field to get a frequency of 1 MHz."]
    Sosc = 2,
}
impl From<Trimsrc> for u8 {
    #[inline(always)]
    fn from(variant: Trimsrc) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Trimsrc {
    type Ux = u8;
}
impl crate::IsEnum for Trimsrc {}
#[doc = "Field `TRIMSRC` reader - Trim Source"]
pub type TrimsrcR = crate::FieldReader<Trimsrc>;
impl TrimsrcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Trimsrc> {
        match self.bits {
            2 => Some(Trimsrc::Sosc),
            _ => None,
        }
    }
    #[doc = "SOSC. This option requires that SOSC be divided using the TRIMDIV field to get a frequency of 1 MHz."]
    #[inline(always)]
    pub fn is_sosc(&self) -> bool {
        *self == Trimsrc::Sosc
    }
}
#[doc = "Field `TRIMSRC` writer - Trim Source"]
pub type TrimsrcW<'a, REG> = crate::FieldWriter<'a, REG, 2, Trimsrc>;
impl<'a, REG> TrimsrcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "SOSC. This option requires that SOSC be divided using the TRIMDIV field to get a frequency of 1 MHz."]
    #[inline(always)]
    pub fn sosc(self) -> &'a mut crate::W<REG> {
        self.variant(Trimsrc::Sosc)
    }
}
#[doc = "Field `TRIMDIV` reader - SIRC Trim Pre-divider"]
pub type TrimdivR = crate::FieldReader;
#[doc = "Field `TRIMDIV` writer - SIRC Trim Pre-divider"]
pub type TrimdivW<'a, REG> = crate::FieldWriter<'a, REG, 7>;
impl R {
    #[doc = "Bits 0:1 - Trim Source"]
    #[inline(always)]
    pub fn trimsrc(&self) -> TrimsrcR {
        TrimsrcR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 16:22 - SIRC Trim Pre-divider"]
    #[inline(always)]
    pub fn trimdiv(&self) -> TrimdivR {
        TrimdivR::new(((self.bits >> 16) & 0x7f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - Trim Source"]
    #[inline(always)]
    pub fn trimsrc(&mut self) -> TrimsrcW<SirctcfgSpec> {
        TrimsrcW::new(self, 0)
    }
    #[doc = "Bits 16:22 - SIRC Trim Pre-divider"]
    #[inline(always)]
    pub fn trimdiv(&mut self) -> TrimdivW<SirctcfgSpec> {
        TrimdivW::new(self, 16)
    }
}
#[doc = "SIRC Trim Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sirctcfg::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sirctcfg::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SirctcfgSpec;
impl crate::RegisterSpec for SirctcfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sirctcfg::R`](R) reader structure"]
impl crate::Readable for SirctcfgSpec {}
#[doc = "`write(|w| ..)` method takes [`sirctcfg::W`](W) writer structure"]
impl crate::Writable for SirctcfgSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SIRCTCFG to value 0"]
impl crate::Resettable for SirctcfgSpec {}

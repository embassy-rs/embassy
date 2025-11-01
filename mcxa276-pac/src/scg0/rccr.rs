#[doc = "Register `RCCR` reader"]
pub type R = crate::R<RccrSpec>;
#[doc = "Register `RCCR` writer"]
pub type W = crate::W<RccrSpec>;
#[doc = "System Clock Source\n\nValue on reset: 3"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Scs {
    #[doc = "1: SOSC"]
    Sosc = 1,
    #[doc = "2: SIRC"]
    Sirc = 2,
    #[doc = "3: FIRC"]
    Firc = 3,
    #[doc = "4: ROSC"]
    Rosc = 4,
    #[doc = "6: SPLL"]
    Spll = 6,
}
impl From<Scs> for u8 {
    #[inline(always)]
    fn from(variant: Scs) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Scs {
    type Ux = u8;
}
impl crate::IsEnum for Scs {}
#[doc = "Field `SCS` reader - System Clock Source"]
pub type ScsR = crate::FieldReader<Scs>;
impl ScsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Scs> {
        match self.bits {
            1 => Some(Scs::Sosc),
            2 => Some(Scs::Sirc),
            3 => Some(Scs::Firc),
            4 => Some(Scs::Rosc),
            6 => Some(Scs::Spll),
            _ => None,
        }
    }
    #[doc = "SOSC"]
    #[inline(always)]
    pub fn is_sosc(&self) -> bool {
        *self == Scs::Sosc
    }
    #[doc = "SIRC"]
    #[inline(always)]
    pub fn is_sirc(&self) -> bool {
        *self == Scs::Sirc
    }
    #[doc = "FIRC"]
    #[inline(always)]
    pub fn is_firc(&self) -> bool {
        *self == Scs::Firc
    }
    #[doc = "ROSC"]
    #[inline(always)]
    pub fn is_rosc(&self) -> bool {
        *self == Scs::Rosc
    }
    #[doc = "SPLL"]
    #[inline(always)]
    pub fn is_spll(&self) -> bool {
        *self == Scs::Spll
    }
}
#[doc = "Field `SCS` writer - System Clock Source"]
pub type ScsW<'a, REG> = crate::FieldWriter<'a, REG, 3, Scs>;
impl<'a, REG> ScsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "SOSC"]
    #[inline(always)]
    pub fn sosc(self) -> &'a mut crate::W<REG> {
        self.variant(Scs::Sosc)
    }
    #[doc = "SIRC"]
    #[inline(always)]
    pub fn sirc(self) -> &'a mut crate::W<REG> {
        self.variant(Scs::Sirc)
    }
    #[doc = "FIRC"]
    #[inline(always)]
    pub fn firc(self) -> &'a mut crate::W<REG> {
        self.variant(Scs::Firc)
    }
    #[doc = "ROSC"]
    #[inline(always)]
    pub fn rosc(self) -> &'a mut crate::W<REG> {
        self.variant(Scs::Rosc)
    }
    #[doc = "SPLL"]
    #[inline(always)]
    pub fn spll(self) -> &'a mut crate::W<REG> {
        self.variant(Scs::Spll)
    }
}
impl R {
    #[doc = "Bits 24:26 - System Clock Source"]
    #[inline(always)]
    pub fn scs(&self) -> ScsR {
        ScsR::new(((self.bits >> 24) & 7) as u8)
    }
}
impl W {
    #[doc = "Bits 24:26 - System Clock Source"]
    #[inline(always)]
    pub fn scs(&mut self) -> ScsW<RccrSpec> {
        ScsW::new(self, 24)
    }
}
#[doc = "Run Clock Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`rccr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rccr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RccrSpec;
impl crate::RegisterSpec for RccrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`rccr::R`](R) reader structure"]
impl crate::Readable for RccrSpec {}
#[doc = "`write(|w| ..)` method takes [`rccr::W`](W) writer structure"]
impl crate::Writable for RccrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RCCR to value 0x0300_0000"]
impl crate::Resettable for RccrSpec {
    const RESET_VALUE: u32 = 0x0300_0000;
}

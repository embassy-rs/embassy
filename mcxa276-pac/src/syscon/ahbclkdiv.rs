#[doc = "Register `AHBCLKDIV` reader"]
pub type R = crate::R<AhbclkdivSpec>;
#[doc = "Register `AHBCLKDIV` writer"]
pub type W = crate::W<AhbclkdivSpec>;
#[doc = "Field `DIV` reader - Clock divider value"]
pub type DivR = crate::FieldReader;
#[doc = "Field `DIV` writer - Clock divider value"]
pub type DivW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Divider status flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unstab {
    #[doc = "0: Divider clock is stable"]
    Stable = 0,
    #[doc = "1: Clock frequency is not stable"]
    Ongoing = 1,
}
impl From<Unstab> for bool {
    #[inline(always)]
    fn from(variant: Unstab) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `UNSTAB` reader - Divider status flag"]
pub type UnstabR = crate::BitReader<Unstab>;
impl UnstabR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Unstab {
        match self.bits {
            false => Unstab::Stable,
            true => Unstab::Ongoing,
        }
    }
    #[doc = "Divider clock is stable"]
    #[inline(always)]
    pub fn is_stable(&self) -> bool {
        *self == Unstab::Stable
    }
    #[doc = "Clock frequency is not stable"]
    #[inline(always)]
    pub fn is_ongoing(&self) -> bool {
        *self == Unstab::Ongoing
    }
}
impl R {
    #[doc = "Bits 0:7 - Clock divider value"]
    #[inline(always)]
    pub fn div(&self) -> DivR {
        DivR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bit 31 - Divider status flag"]
    #[inline(always)]
    pub fn unstab(&self) -> UnstabR {
        UnstabR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:7 - Clock divider value"]
    #[inline(always)]
    pub fn div(&mut self) -> DivW<AhbclkdivSpec> {
        DivW::new(self, 0)
    }
}
#[doc = "System Clock Divider\n\nYou can [`read`](crate::Reg::read) this register and get [`ahbclkdiv::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ahbclkdiv::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct AhbclkdivSpec;
impl crate::RegisterSpec for AhbclkdivSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ahbclkdiv::R`](R) reader structure"]
impl crate::Readable for AhbclkdivSpec {}
#[doc = "`write(|w| ..)` method takes [`ahbclkdiv::W`](W) writer structure"]
impl crate::Writable for AhbclkdivSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets AHBCLKDIV to value 0"]
impl crate::Resettable for AhbclkdivSpec {}

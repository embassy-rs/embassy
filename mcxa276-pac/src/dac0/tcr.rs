#[doc = "Register `TCR` reader"]
pub type R = crate::R<TcrSpec>;
#[doc = "Register `TCR` writer"]
pub type W = crate::W<TcrSpec>;
#[doc = "Software Trigger\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Swtrg {
    #[doc = "0: Not valid"]
    NotValid = 0,
    #[doc = "1: Valid"]
    Valid = 1,
}
impl From<Swtrg> for bool {
    #[inline(always)]
    fn from(variant: Swtrg) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SWTRG` reader - Software Trigger"]
pub type SwtrgR = crate::BitReader<Swtrg>;
impl SwtrgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Swtrg {
        match self.bits {
            false => Swtrg::NotValid,
            true => Swtrg::Valid,
        }
    }
    #[doc = "Not valid"]
    #[inline(always)]
    pub fn is_not_valid(&self) -> bool {
        *self == Swtrg::NotValid
    }
    #[doc = "Valid"]
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        *self == Swtrg::Valid
    }
}
#[doc = "Field `SWTRG` writer - Software Trigger"]
pub type SwtrgW<'a, REG> = crate::BitWriter<'a, REG, Swtrg>;
impl<'a, REG> SwtrgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not valid"]
    #[inline(always)]
    pub fn not_valid(self) -> &'a mut crate::W<REG> {
        self.variant(Swtrg::NotValid)
    }
    #[doc = "Valid"]
    #[inline(always)]
    pub fn valid(self) -> &'a mut crate::W<REG> {
        self.variant(Swtrg::Valid)
    }
}
impl R {
    #[doc = "Bit 0 - Software Trigger"]
    #[inline(always)]
    pub fn swtrg(&self) -> SwtrgR {
        SwtrgR::new((self.bits & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Software Trigger"]
    #[inline(always)]
    pub fn swtrg(&mut self) -> SwtrgW<TcrSpec> {
        SwtrgW::new(self, 0)
    }
}
#[doc = "Trigger Control\n\nYou can [`read`](crate::Reg::read) this register and get [`tcr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tcr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TcrSpec;
impl crate::RegisterSpec for TcrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`tcr::R`](R) reader structure"]
impl crate::Readable for TcrSpec {}
#[doc = "`write(|w| ..)` method takes [`tcr::W`](W) writer structure"]
impl crate::Writable for TcrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TCR to value 0"]
impl crate::Resettable for TcrSpec {}

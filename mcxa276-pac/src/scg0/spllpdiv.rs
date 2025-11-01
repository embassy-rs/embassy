#[doc = "Register `SPLLPDIV` reader"]
pub type R = crate::R<SpllpdivSpec>;
#[doc = "Register `SPLLPDIV` writer"]
pub type W = crate::W<SpllpdivSpec>;
#[doc = "Field `PDIV` reader - Post-divider divider ratio (P-divider)"]
pub type PdivR = crate::FieldReader;
#[doc = "Field `PDIV` writer - Post-divider divider ratio (P-divider)"]
pub type PdivW<'a, REG> = crate::FieldWriter<'a, REG, 5>;
#[doc = "Post-divider ratio change request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preq {
    #[doc = "0: Post-divider ratio change is not requested"]
    Disabled = 0,
    #[doc = "1: Post-divider ratio change is requested"]
    Enabled = 1,
}
impl From<Preq> for bool {
    #[inline(always)]
    fn from(variant: Preq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PREQ` reader - Post-divider ratio change request"]
pub type PreqR = crate::BitReader<Preq>;
impl PreqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Preq {
        match self.bits {
            false => Preq::Disabled,
            true => Preq::Enabled,
        }
    }
    #[doc = "Post-divider ratio change is not requested"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Preq::Disabled
    }
    #[doc = "Post-divider ratio change is requested"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Preq::Enabled
    }
}
#[doc = "Field `PREQ` writer - Post-divider ratio change request"]
pub type PreqW<'a, REG> = crate::BitWriter<'a, REG, Preq>;
impl<'a, REG> PreqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Post-divider ratio change is not requested"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Preq::Disabled)
    }
    #[doc = "Post-divider ratio change is requested"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Preq::Enabled)
    }
}
impl R {
    #[doc = "Bits 0:4 - Post-divider divider ratio (P-divider)"]
    #[inline(always)]
    pub fn pdiv(&self) -> PdivR {
        PdivR::new((self.bits & 0x1f) as u8)
    }
    #[doc = "Bit 31 - Post-divider ratio change request"]
    #[inline(always)]
    pub fn preq(&self) -> PreqR {
        PreqR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:4 - Post-divider divider ratio (P-divider)"]
    #[inline(always)]
    pub fn pdiv(&mut self) -> PdivW<SpllpdivSpec> {
        PdivW::new(self, 0)
    }
    #[doc = "Bit 31 - Post-divider ratio change request"]
    #[inline(always)]
    pub fn preq(&mut self) -> PreqW<SpllpdivSpec> {
        PreqW::new(self, 31)
    }
}
#[doc = "SPLL P Divider Register\n\nYou can [`read`](crate::Reg::read) this register and get [`spllpdiv::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`spllpdiv::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SpllpdivSpec;
impl crate::RegisterSpec for SpllpdivSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`spllpdiv::R`](R) reader structure"]
impl crate::Readable for SpllpdivSpec {}
#[doc = "`write(|w| ..)` method takes [`spllpdiv::W`](W) writer structure"]
impl crate::Writable for SpllpdivSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SPLLPDIV to value 0x01"]
impl crate::Resettable for SpllpdivSpec {
    const RESET_VALUE: u32 = 0x01;
}

#[doc = "Register `SPLLNDIV` reader"]
pub type R = crate::R<SpllndivSpec>;
#[doc = "Register `SPLLNDIV` writer"]
pub type W = crate::W<SpllndivSpec>;
#[doc = "Field `NDIV` reader - Pre-divider divider ratio (N-divider)."]
pub type NdivR = crate::FieldReader;
#[doc = "Field `NDIV` writer - Pre-divider divider ratio (N-divider)."]
pub type NdivW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Pre-divider ratio change request.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nreq {
    #[doc = "0: Pre-divider ratio change is not requested"]
    Disabled = 0,
    #[doc = "1: Pre-divider ratio change is requested"]
    Enabled = 1,
}
impl From<Nreq> for bool {
    #[inline(always)]
    fn from(variant: Nreq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NREQ` reader - Pre-divider ratio change request."]
pub type NreqR = crate::BitReader<Nreq>;
impl NreqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nreq {
        match self.bits {
            false => Nreq::Disabled,
            true => Nreq::Enabled,
        }
    }
    #[doc = "Pre-divider ratio change is not requested"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Nreq::Disabled
    }
    #[doc = "Pre-divider ratio change is requested"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Nreq::Enabled
    }
}
#[doc = "Field `NREQ` writer - Pre-divider ratio change request."]
pub type NreqW<'a, REG> = crate::BitWriter<'a, REG, Nreq>;
impl<'a, REG> NreqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pre-divider ratio change is not requested"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Nreq::Disabled)
    }
    #[doc = "Pre-divider ratio change is requested"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Nreq::Enabled)
    }
}
impl R {
    #[doc = "Bits 0:7 - Pre-divider divider ratio (N-divider)."]
    #[inline(always)]
    pub fn ndiv(&self) -> NdivR {
        NdivR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bit 31 - Pre-divider ratio change request."]
    #[inline(always)]
    pub fn nreq(&self) -> NreqR {
        NreqR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:7 - Pre-divider divider ratio (N-divider)."]
    #[inline(always)]
    pub fn ndiv(&mut self) -> NdivW<SpllndivSpec> {
        NdivW::new(self, 0)
    }
    #[doc = "Bit 31 - Pre-divider ratio change request."]
    #[inline(always)]
    pub fn nreq(&mut self) -> NreqW<SpllndivSpec> {
        NreqW::new(self, 31)
    }
}
#[doc = "SPLL N Divider Register\n\nYou can [`read`](crate::Reg::read) this register and get [`spllndiv::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`spllndiv::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SpllndivSpec;
impl crate::RegisterSpec for SpllndivSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`spllndiv::R`](R) reader structure"]
impl crate::Readable for SpllndivSpec {}
#[doc = "`write(|w| ..)` method takes [`spllndiv::W`](W) writer structure"]
impl crate::Writable for SpllndivSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SPLLNDIV to value 0x01"]
impl crate::Resettable for SpllndivSpec {
    const RESET_VALUE: u32 = 0x01;
}

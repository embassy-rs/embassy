#[doc = "Register `SPLLMDIV` reader"]
pub type R = crate::R<SpllmdivSpec>;
#[doc = "Register `SPLLMDIV` writer"]
pub type W = crate::W<SpllmdivSpec>;
#[doc = "Field `MDIV` reader - Feedback divider ratio (M-divider)."]
pub type MdivR = crate::FieldReader<u16>;
#[doc = "Field `MDIV` writer - Feedback divider ratio (M-divider)."]
pub type MdivW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
#[doc = "Feedback ratio change request.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mreq {
    #[doc = "0: Feedback ratio change is not requested"]
    Disabled = 0,
    #[doc = "1: Feedback ratio change is requested"]
    Enabled = 1,
}
impl From<Mreq> for bool {
    #[inline(always)]
    fn from(variant: Mreq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MREQ` reader - Feedback ratio change request."]
pub type MreqR = crate::BitReader<Mreq>;
impl MreqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mreq {
        match self.bits {
            false => Mreq::Disabled,
            true => Mreq::Enabled,
        }
    }
    #[doc = "Feedback ratio change is not requested"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Mreq::Disabled
    }
    #[doc = "Feedback ratio change is requested"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Mreq::Enabled
    }
}
#[doc = "Field `MREQ` writer - Feedback ratio change request."]
pub type MreqW<'a, REG> = crate::BitWriter<'a, REG, Mreq>;
impl<'a, REG> MreqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Feedback ratio change is not requested"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Mreq::Disabled)
    }
    #[doc = "Feedback ratio change is requested"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Mreq::Enabled)
    }
}
impl R {
    #[doc = "Bits 0:15 - Feedback divider ratio (M-divider)."]
    #[inline(always)]
    pub fn mdiv(&self) -> MdivR {
        MdivR::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bit 31 - Feedback ratio change request."]
    #[inline(always)]
    pub fn mreq(&self) -> MreqR {
        MreqR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:15 - Feedback divider ratio (M-divider)."]
    #[inline(always)]
    pub fn mdiv(&mut self) -> MdivW<SpllmdivSpec> {
        MdivW::new(self, 0)
    }
    #[doc = "Bit 31 - Feedback ratio change request."]
    #[inline(always)]
    pub fn mreq(&mut self) -> MreqW<SpllmdivSpec> {
        MreqW::new(self, 31)
    }
}
#[doc = "SPLL M Divider Register\n\nYou can [`read`](crate::Reg::read) this register and get [`spllmdiv::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`spllmdiv::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SpllmdivSpec;
impl crate::RegisterSpec for SpllmdivSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`spllmdiv::R`](R) reader structure"]
impl crate::Readable for SpllmdivSpec {}
#[doc = "`write(|w| ..)` method takes [`spllmdiv::W`](W) writer structure"]
impl crate::Writable for SpllmdivSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SPLLMDIV to value 0x01"]
impl crate::Resettable for SpllmdivSpec {
    const RESET_VALUE: u32 = 0x01;
}

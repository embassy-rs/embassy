#[doc = "Register `SPLLSSCG1` reader"]
pub type R = crate::R<Spllsscg1Spec>;
#[doc = "Register `SPLLSSCG1` writer"]
pub type W = crate::W<Spllsscg1Spec>;
#[doc = "Field `SS_MDIV_MSB` reader - SS_MDIV\\[32\\]"]
pub type SsMdivMsbR = crate::BitReader;
#[doc = "Field `SS_MDIV_MSB` writer - SS_MDIV\\[32\\]"]
pub type SsMdivMsbW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "SS_MDIV\\[32:0\\] change request.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SsMdivReq {
    #[doc = "0: SS_MDIV change is not requested"]
    Disabled = 0,
    #[doc = "1: SS_MDIV change is requested"]
    Enabled = 1,
}
impl From<SsMdivReq> for bool {
    #[inline(always)]
    fn from(variant: SsMdivReq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SS_MDIV_REQ` reader - SS_MDIV\\[32:0\\] change request."]
pub type SsMdivReqR = crate::BitReader<SsMdivReq>;
impl SsMdivReqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> SsMdivReq {
        match self.bits {
            false => SsMdivReq::Disabled,
            true => SsMdivReq::Enabled,
        }
    }
    #[doc = "SS_MDIV change is not requested"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == SsMdivReq::Disabled
    }
    #[doc = "SS_MDIV change is requested"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == SsMdivReq::Enabled
    }
}
#[doc = "Field `SS_MDIV_REQ` writer - SS_MDIV\\[32:0\\] change request."]
pub type SsMdivReqW<'a, REG> = crate::BitWriter<'a, REG, SsMdivReq>;
impl<'a, REG> SsMdivReqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "SS_MDIV change is not requested"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(SsMdivReq::Disabled)
    }
    #[doc = "SS_MDIV change is requested"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(SsMdivReq::Enabled)
    }
}
#[doc = "Field `MF` reader - Modulation Frequency Control"]
pub type MfR = crate::FieldReader;
#[doc = "Field `MF` writer - Modulation Frequency Control"]
pub type MfW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `MR` reader - Modulation Depth Control"]
pub type MrR = crate::FieldReader;
#[doc = "Field `MR` writer - Modulation Depth Control"]
pub type MrW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `MC` reader - Modulation Waveform Control"]
pub type McR = crate::FieldReader;
#[doc = "Field `MC` writer - Modulation Waveform Control"]
pub type McW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Dither Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dither {
    #[doc = "0: Dither is not enabled"]
    Disabled = 0,
    #[doc = "1: Dither is enabled"]
    Enabled = 1,
}
impl From<Dither> for bool {
    #[inline(always)]
    fn from(variant: Dither) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DITHER` reader - Dither Enable"]
pub type DitherR = crate::BitReader<Dither>;
impl DitherR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dither {
        match self.bits {
            false => Dither::Disabled,
            true => Dither::Enabled,
        }
    }
    #[doc = "Dither is not enabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Dither::Disabled
    }
    #[doc = "Dither is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Dither::Enabled
    }
}
#[doc = "Field `DITHER` writer - Dither Enable"]
pub type DitherW<'a, REG> = crate::BitWriter<'a, REG, Dither>;
impl<'a, REG> DitherW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Dither is not enabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Dither::Disabled)
    }
    #[doc = "Dither is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Dither::Enabled)
    }
}
#[doc = "SS_MDIV select.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelSsMdiv {
    #[doc = "0: Feedback divider ratio is MDIV\\[15:0\\]"]
    Disabled = 0,
    #[doc = "1: Feedback divider ratio is SS_MDIV\\[32:0\\]"]
    Enabled = 1,
}
impl From<SelSsMdiv> for bool {
    #[inline(always)]
    fn from(variant: SelSsMdiv) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SEL_SS_MDIV` reader - SS_MDIV select."]
pub type SelSsMdivR = crate::BitReader<SelSsMdiv>;
impl SelSsMdivR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> SelSsMdiv {
        match self.bits {
            false => SelSsMdiv::Disabled,
            true => SelSsMdiv::Enabled,
        }
    }
    #[doc = "Feedback divider ratio is MDIV\\[15:0\\]"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == SelSsMdiv::Disabled
    }
    #[doc = "Feedback divider ratio is SS_MDIV\\[32:0\\]"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == SelSsMdiv::Enabled
    }
}
#[doc = "Field `SEL_SS_MDIV` writer - SS_MDIV select."]
pub type SelSsMdivW<'a, REG> = crate::BitWriter<'a, REG, SelSsMdiv>;
impl<'a, REG> SelSsMdivW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Feedback divider ratio is MDIV\\[15:0\\]"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(SelSsMdiv::Disabled)
    }
    #[doc = "Feedback divider ratio is SS_MDIV\\[32:0\\]"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(SelSsMdiv::Enabled)
    }
}
#[doc = "SSCG Power Down\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SsPd {
    #[doc = "0: SSCG is powered on"]
    Disabled = 0,
    #[doc = "1: SSCG is powered off"]
    Enabled = 1,
}
impl From<SsPd> for bool {
    #[inline(always)]
    fn from(variant: SsPd) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SS_PD` reader - SSCG Power Down"]
pub type SsPdR = crate::BitReader<SsPd>;
impl SsPdR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> SsPd {
        match self.bits {
            false => SsPd::Disabled,
            true => SsPd::Enabled,
        }
    }
    #[doc = "SSCG is powered on"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == SsPd::Disabled
    }
    #[doc = "SSCG is powered off"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == SsPd::Enabled
    }
}
#[doc = "Field `SS_PD` writer - SSCG Power Down"]
pub type SsPdW<'a, REG> = crate::BitWriter<'a, REG, SsPd>;
impl<'a, REG> SsPdW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "SSCG is powered on"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(SsPd::Disabled)
    }
    #[doc = "SSCG is powered off"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(SsPd::Enabled)
    }
}
impl R {
    #[doc = "Bit 0 - SS_MDIV\\[32\\]"]
    #[inline(always)]
    pub fn ss_mdiv_msb(&self) -> SsMdivMsbR {
        SsMdivMsbR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - SS_MDIV\\[32:0\\] change request."]
    #[inline(always)]
    pub fn ss_mdiv_req(&self) -> SsMdivReqR {
        SsMdivReqR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bits 2:4 - Modulation Frequency Control"]
    #[inline(always)]
    pub fn mf(&self) -> MfR {
        MfR::new(((self.bits >> 2) & 7) as u8)
    }
    #[doc = "Bits 5:7 - Modulation Depth Control"]
    #[inline(always)]
    pub fn mr(&self) -> MrR {
        MrR::new(((self.bits >> 5) & 7) as u8)
    }
    #[doc = "Bits 8:9 - Modulation Waveform Control"]
    #[inline(always)]
    pub fn mc(&self) -> McR {
        McR::new(((self.bits >> 8) & 3) as u8)
    }
    #[doc = "Bit 10 - Dither Enable"]
    #[inline(always)]
    pub fn dither(&self) -> DitherR {
        DitherR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - SS_MDIV select."]
    #[inline(always)]
    pub fn sel_ss_mdiv(&self) -> SelSsMdivR {
        SelSsMdivR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 31 - SSCG Power Down"]
    #[inline(always)]
    pub fn ss_pd(&self) -> SsPdR {
        SsPdR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - SS_MDIV\\[32\\]"]
    #[inline(always)]
    pub fn ss_mdiv_msb(&mut self) -> SsMdivMsbW<Spllsscg1Spec> {
        SsMdivMsbW::new(self, 0)
    }
    #[doc = "Bit 1 - SS_MDIV\\[32:0\\] change request."]
    #[inline(always)]
    pub fn ss_mdiv_req(&mut self) -> SsMdivReqW<Spllsscg1Spec> {
        SsMdivReqW::new(self, 1)
    }
    #[doc = "Bits 2:4 - Modulation Frequency Control"]
    #[inline(always)]
    pub fn mf(&mut self) -> MfW<Spllsscg1Spec> {
        MfW::new(self, 2)
    }
    #[doc = "Bits 5:7 - Modulation Depth Control"]
    #[inline(always)]
    pub fn mr(&mut self) -> MrW<Spllsscg1Spec> {
        MrW::new(self, 5)
    }
    #[doc = "Bits 8:9 - Modulation Waveform Control"]
    #[inline(always)]
    pub fn mc(&mut self) -> McW<Spllsscg1Spec> {
        McW::new(self, 8)
    }
    #[doc = "Bit 10 - Dither Enable"]
    #[inline(always)]
    pub fn dither(&mut self) -> DitherW<Spllsscg1Spec> {
        DitherW::new(self, 10)
    }
    #[doc = "Bit 11 - SS_MDIV select."]
    #[inline(always)]
    pub fn sel_ss_mdiv(&mut self) -> SelSsMdivW<Spllsscg1Spec> {
        SelSsMdivW::new(self, 11)
    }
    #[doc = "Bit 31 - SSCG Power Down"]
    #[inline(always)]
    pub fn ss_pd(&mut self) -> SsPdW<Spllsscg1Spec> {
        SsPdW::new(self, 31)
    }
}
#[doc = "SPLL Spread Spectrum Control 1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`spllsscg1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`spllsscg1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Spllsscg1Spec;
impl crate::RegisterSpec for Spllsscg1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`spllsscg1::R`](R) reader structure"]
impl crate::Readable for Spllsscg1Spec {}
#[doc = "`write(|w| ..)` method takes [`spllsscg1::W`](W) writer structure"]
impl crate::Writable for Spllsscg1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SPLLSSCG1 to value 0x8000_0000"]
impl crate::Resettable for Spllsscg1Spec {
    const RESET_VALUE: u32 = 0x8000_0000;
}

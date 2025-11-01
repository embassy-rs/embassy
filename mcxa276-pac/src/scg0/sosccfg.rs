#[doc = "Register `SOSCCFG` reader"]
pub type R = crate::R<SosccfgSpec>;
#[doc = "Register `SOSCCFG` writer"]
pub type W = crate::W<SosccfgSpec>;
#[doc = "External Reference Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Erefs {
    #[doc = "0: External reference clock selected."]
    External = 0,
    #[doc = "1: Internal crystal oscillator of OSC selected."]
    Internal = 1,
}
impl From<Erefs> for bool {
    #[inline(always)]
    fn from(variant: Erefs) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EREFS` reader - External Reference Select"]
pub type ErefsR = crate::BitReader<Erefs>;
impl ErefsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Erefs {
        match self.bits {
            false => Erefs::External,
            true => Erefs::Internal,
        }
    }
    #[doc = "External reference clock selected."]
    #[inline(always)]
    pub fn is_external(&self) -> bool {
        *self == Erefs::External
    }
    #[doc = "Internal crystal oscillator of OSC selected."]
    #[inline(always)]
    pub fn is_internal(&self) -> bool {
        *self == Erefs::Internal
    }
}
#[doc = "Field `EREFS` writer - External Reference Select"]
pub type ErefsW<'a, REG> = crate::BitWriter<'a, REG, Erefs>;
impl<'a, REG> ErefsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "External reference clock selected."]
    #[inline(always)]
    pub fn external(self) -> &'a mut crate::W<REG> {
        self.variant(Erefs::External)
    }
    #[doc = "Internal crystal oscillator of OSC selected."]
    #[inline(always)]
    pub fn internal(self) -> &'a mut crate::W<REG> {
        self.variant(Erefs::Internal)
    }
}
#[doc = "SOSC Range Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Range {
    #[doc = "0: Frequency range select of 8-16 MHz."]
    Freq16to20mhz = 0,
    #[doc = "1: Frequency range select of 16-25 MHz."]
    LowFreq = 1,
    #[doc = "2: Frequency range select of 25-40 MHz."]
    MediumFreq = 2,
    #[doc = "3: Frequency range select of 40-50 MHz."]
    HighFreq = 3,
}
impl From<Range> for u8 {
    #[inline(always)]
    fn from(variant: Range) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Range {
    type Ux = u8;
}
impl crate::IsEnum for Range {}
#[doc = "Field `RANGE` reader - SOSC Range Select"]
pub type RangeR = crate::FieldReader<Range>;
impl RangeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Range {
        match self.bits {
            0 => Range::Freq16to20mhz,
            1 => Range::LowFreq,
            2 => Range::MediumFreq,
            3 => Range::HighFreq,
            _ => unreachable!(),
        }
    }
    #[doc = "Frequency range select of 8-16 MHz."]
    #[inline(always)]
    pub fn is_freq_16to20mhz(&self) -> bool {
        *self == Range::Freq16to20mhz
    }
    #[doc = "Frequency range select of 16-25 MHz."]
    #[inline(always)]
    pub fn is_low_freq(&self) -> bool {
        *self == Range::LowFreq
    }
    #[doc = "Frequency range select of 25-40 MHz."]
    #[inline(always)]
    pub fn is_medium_freq(&self) -> bool {
        *self == Range::MediumFreq
    }
    #[doc = "Frequency range select of 40-50 MHz."]
    #[inline(always)]
    pub fn is_high_freq(&self) -> bool {
        *self == Range::HighFreq
    }
}
#[doc = "Field `RANGE` writer - SOSC Range Select"]
pub type RangeW<'a, REG> = crate::FieldWriter<'a, REG, 2, Range, crate::Safe>;
impl<'a, REG> RangeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Frequency range select of 8-16 MHz."]
    #[inline(always)]
    pub fn freq_16to20mhz(self) -> &'a mut crate::W<REG> {
        self.variant(Range::Freq16to20mhz)
    }
    #[doc = "Frequency range select of 16-25 MHz."]
    #[inline(always)]
    pub fn low_freq(self) -> &'a mut crate::W<REG> {
        self.variant(Range::LowFreq)
    }
    #[doc = "Frequency range select of 25-40 MHz."]
    #[inline(always)]
    pub fn medium_freq(self) -> &'a mut crate::W<REG> {
        self.variant(Range::MediumFreq)
    }
    #[doc = "Frequency range select of 40-50 MHz."]
    #[inline(always)]
    pub fn high_freq(self) -> &'a mut crate::W<REG> {
        self.variant(Range::HighFreq)
    }
}
impl R {
    #[doc = "Bit 2 - External Reference Select"]
    #[inline(always)]
    pub fn erefs(&self) -> ErefsR {
        ErefsR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bits 4:5 - SOSC Range Select"]
    #[inline(always)]
    pub fn range(&self) -> RangeR {
        RangeR::new(((self.bits >> 4) & 3) as u8)
    }
}
impl W {
    #[doc = "Bit 2 - External Reference Select"]
    #[inline(always)]
    pub fn erefs(&mut self) -> ErefsW<SosccfgSpec> {
        ErefsW::new(self, 2)
    }
    #[doc = "Bits 4:5 - SOSC Range Select"]
    #[inline(always)]
    pub fn range(&mut self) -> RangeW<SosccfgSpec> {
        RangeW::new(self, 4)
    }
}
#[doc = "SOSC Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sosccfg::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sosccfg::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SosccfgSpec;
impl crate::RegisterSpec for SosccfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sosccfg::R`](R) reader structure"]
impl crate::Readable for SosccfgSpec {}
#[doc = "`write(|w| ..)` method takes [`sosccfg::W`](W) writer structure"]
impl crate::Writable for SosccfgSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SOSCCFG to value 0"]
impl crate::Resettable for SosccfgSpec {}

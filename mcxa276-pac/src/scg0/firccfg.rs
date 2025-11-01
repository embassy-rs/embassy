#[doc = "Register `FIRCCFG` reader"]
pub type R = crate::R<FirccfgSpec>;
#[doc = "Register `FIRCCFG` writer"]
pub type W = crate::W<FirccfgSpec>;
#[doc = "Frequency select\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum FreqSel {
    #[doc = "1: 45 MHz FIRC clock selected, divided from 180 MHz"]
    Firc48mhz192s = 1,
    #[doc = "3: 60 MHz FIRC clock selected"]
    Firc64mhz = 3,
    #[doc = "5: 90 MHz FIRC clock selected"]
    Firc96mhz = 5,
    #[doc = "7: 180 MHz FIRC clock selected"]
    Firc192mhz = 7,
}
impl From<FreqSel> for u8 {
    #[inline(always)]
    fn from(variant: FreqSel) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for FreqSel {
    type Ux = u8;
}
impl crate::IsEnum for FreqSel {}
#[doc = "Field `FREQ_SEL` reader - Frequency select"]
pub type FreqSelR = crate::FieldReader<FreqSel>;
impl FreqSelR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<FreqSel> {
        match self.bits {
            1 => Some(FreqSel::Firc48mhz192s),
            3 => Some(FreqSel::Firc64mhz),
            5 => Some(FreqSel::Firc96mhz),
            7 => Some(FreqSel::Firc192mhz),
            _ => None,
        }
    }
    #[doc = "45 MHz FIRC clock selected, divided from 180 MHz"]
    #[inline(always)]
    pub fn is_firc_48mhz_192s(&self) -> bool {
        *self == FreqSel::Firc48mhz192s
    }
    #[doc = "60 MHz FIRC clock selected"]
    #[inline(always)]
    pub fn is_firc_64mhz(&self) -> bool {
        *self == FreqSel::Firc64mhz
    }
    #[doc = "90 MHz FIRC clock selected"]
    #[inline(always)]
    pub fn is_firc_96mhz(&self) -> bool {
        *self == FreqSel::Firc96mhz
    }
    #[doc = "180 MHz FIRC clock selected"]
    #[inline(always)]
    pub fn is_firc_192mhz(&self) -> bool {
        *self == FreqSel::Firc192mhz
    }
}
#[doc = "Field `FREQ_SEL` writer - Frequency select"]
pub type FreqSelW<'a, REG> = crate::FieldWriter<'a, REG, 3, FreqSel>;
impl<'a, REG> FreqSelW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "45 MHz FIRC clock selected, divided from 180 MHz"]
    #[inline(always)]
    pub fn firc_48mhz_192s(self) -> &'a mut crate::W<REG> {
        self.variant(FreqSel::Firc48mhz192s)
    }
    #[doc = "60 MHz FIRC clock selected"]
    #[inline(always)]
    pub fn firc_64mhz(self) -> &'a mut crate::W<REG> {
        self.variant(FreqSel::Firc64mhz)
    }
    #[doc = "90 MHz FIRC clock selected"]
    #[inline(always)]
    pub fn firc_96mhz(self) -> &'a mut crate::W<REG> {
        self.variant(FreqSel::Firc96mhz)
    }
    #[doc = "180 MHz FIRC clock selected"]
    #[inline(always)]
    pub fn firc_192mhz(self) -> &'a mut crate::W<REG> {
        self.variant(FreqSel::Firc192mhz)
    }
}
impl R {
    #[doc = "Bits 1:3 - Frequency select"]
    #[inline(always)]
    pub fn freq_sel(&self) -> FreqSelR {
        FreqSelR::new(((self.bits >> 1) & 7) as u8)
    }
}
impl W {
    #[doc = "Bits 1:3 - Frequency select"]
    #[inline(always)]
    pub fn freq_sel(&mut self) -> FreqSelW<FirccfgSpec> {
        FreqSelW::new(self, 1)
    }
}
#[doc = "FIRC Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`firccfg::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`firccfg::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FirccfgSpec;
impl crate::RegisterSpec for FirccfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`firccfg::R`](R) reader structure"]
impl crate::Readable for FirccfgSpec {}
#[doc = "`write(|w| ..)` method takes [`firccfg::W`](W) writer structure"]
impl crate::Writable for FirccfgSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FIRCCFG to value 0x03"]
impl crate::Resettable for FirccfgSpec {
    const RESET_VALUE: u32 = 0x03;
}

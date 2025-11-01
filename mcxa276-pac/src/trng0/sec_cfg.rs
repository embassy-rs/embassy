#[doc = "Register `SEC_CFG` reader"]
pub type R = crate::R<SecCfgSpec>;
#[doc = "Register `SEC_CFG` writer"]
pub type W = crate::W<SecCfgSpec>;
#[doc = "If set, below mentioned TRNG configuration registers cannot be programmed: Oscillator 2 Control Register (OSC2_CTL): TRNG Entropy Generation Control \\[1:0\\] Oscillator 2 Divider \\[3:2\\] Oscillator Fail Safe Limit \\[13:12\\] Oscillator Fail Safe Test \\[14\\] TRNG Seed Control Register (SDCTL) TRNG Frequency Count Minimum Limit Register (FRQMIN) TRNG Frequency Count Maximum Limit Register (FRQMAX) TRNG Statistical Check Monobit Limit Register (SCML) TRNG Statistical Check Run Length 1 Limit Register (SCR1L) TRNG Statistical Check Run Length 2 Limit Register (SCR2L) TRNG Statistical Check Run Length 3 Limit Register (SCR3L) TRNG Miscellaneous Control Register (MCTL): Sample Mode \\[1:0\\] Oscillator Divider \\[3:2\\] Reset Defaults \\[6\\] Force System Clock \\[7\\] Long Runs Continuation Mode \\[14\\] After this bit has been written to a 1, it cannot be changed\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoPrgm {
    #[doc = "0: TRNG configuration registers can be modified."]
    NoPrgmOff = 0,
    #[doc = "1: TRNG configuration registers cannot be modified."]
    NoPrgmOn = 1,
}
impl From<NoPrgm> for bool {
    #[inline(always)]
    fn from(variant: NoPrgm) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NO_PRGM` reader - If set, below mentioned TRNG configuration registers cannot be programmed: Oscillator 2 Control Register (OSC2_CTL): TRNG Entropy Generation Control \\[1:0\\] Oscillator 2 Divider \\[3:2\\] Oscillator Fail Safe Limit \\[13:12\\] Oscillator Fail Safe Test \\[14\\] TRNG Seed Control Register (SDCTL) TRNG Frequency Count Minimum Limit Register (FRQMIN) TRNG Frequency Count Maximum Limit Register (FRQMAX) TRNG Statistical Check Monobit Limit Register (SCML) TRNG Statistical Check Run Length 1 Limit Register (SCR1L) TRNG Statistical Check Run Length 2 Limit Register (SCR2L) TRNG Statistical Check Run Length 3 Limit Register (SCR3L) TRNG Miscellaneous Control Register (MCTL): Sample Mode \\[1:0\\] Oscillator Divider \\[3:2\\] Reset Defaults \\[6\\] Force System Clock \\[7\\] Long Runs Continuation Mode \\[14\\] After this bit has been written to a 1, it cannot be changed"]
pub type NoPrgmR = crate::BitReader<NoPrgm>;
impl NoPrgmR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> NoPrgm {
        match self.bits {
            false => NoPrgm::NoPrgmOff,
            true => NoPrgm::NoPrgmOn,
        }
    }
    #[doc = "TRNG configuration registers can be modified."]
    #[inline(always)]
    pub fn is_no_prgm_off(&self) -> bool {
        *self == NoPrgm::NoPrgmOff
    }
    #[doc = "TRNG configuration registers cannot be modified."]
    #[inline(always)]
    pub fn is_no_prgm_on(&self) -> bool {
        *self == NoPrgm::NoPrgmOn
    }
}
#[doc = "Field `NO_PRGM` writer - If set, below mentioned TRNG configuration registers cannot be programmed: Oscillator 2 Control Register (OSC2_CTL): TRNG Entropy Generation Control \\[1:0\\] Oscillator 2 Divider \\[3:2\\] Oscillator Fail Safe Limit \\[13:12\\] Oscillator Fail Safe Test \\[14\\] TRNG Seed Control Register (SDCTL) TRNG Frequency Count Minimum Limit Register (FRQMIN) TRNG Frequency Count Maximum Limit Register (FRQMAX) TRNG Statistical Check Monobit Limit Register (SCML) TRNG Statistical Check Run Length 1 Limit Register (SCR1L) TRNG Statistical Check Run Length 2 Limit Register (SCR2L) TRNG Statistical Check Run Length 3 Limit Register (SCR3L) TRNG Miscellaneous Control Register (MCTL): Sample Mode \\[1:0\\] Oscillator Divider \\[3:2\\] Reset Defaults \\[6\\] Force System Clock \\[7\\] Long Runs Continuation Mode \\[14\\] After this bit has been written to a 1, it cannot be changed"]
pub type NoPrgmW<'a, REG> = crate::BitWriter<'a, REG, NoPrgm>;
impl<'a, REG> NoPrgmW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "TRNG configuration registers can be modified."]
    #[inline(always)]
    pub fn no_prgm_off(self) -> &'a mut crate::W<REG> {
        self.variant(NoPrgm::NoPrgmOff)
    }
    #[doc = "TRNG configuration registers cannot be modified."]
    #[inline(always)]
    pub fn no_prgm_on(self) -> &'a mut crate::W<REG> {
        self.variant(NoPrgm::NoPrgmOn)
    }
}
impl R {
    #[doc = "Bit 1 - If set, below mentioned TRNG configuration registers cannot be programmed: Oscillator 2 Control Register (OSC2_CTL): TRNG Entropy Generation Control \\[1:0\\] Oscillator 2 Divider \\[3:2\\] Oscillator Fail Safe Limit \\[13:12\\] Oscillator Fail Safe Test \\[14\\] TRNG Seed Control Register (SDCTL) TRNG Frequency Count Minimum Limit Register (FRQMIN) TRNG Frequency Count Maximum Limit Register (FRQMAX) TRNG Statistical Check Monobit Limit Register (SCML) TRNG Statistical Check Run Length 1 Limit Register (SCR1L) TRNG Statistical Check Run Length 2 Limit Register (SCR2L) TRNG Statistical Check Run Length 3 Limit Register (SCR3L) TRNG Miscellaneous Control Register (MCTL): Sample Mode \\[1:0\\] Oscillator Divider \\[3:2\\] Reset Defaults \\[6\\] Force System Clock \\[7\\] Long Runs Continuation Mode \\[14\\] After this bit has been written to a 1, it cannot be changed"]
    #[inline(always)]
    pub fn no_prgm(&self) -> NoPrgmR {
        NoPrgmR::new(((self.bits >> 1) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 1 - If set, below mentioned TRNG configuration registers cannot be programmed: Oscillator 2 Control Register (OSC2_CTL): TRNG Entropy Generation Control \\[1:0\\] Oscillator 2 Divider \\[3:2\\] Oscillator Fail Safe Limit \\[13:12\\] Oscillator Fail Safe Test \\[14\\] TRNG Seed Control Register (SDCTL) TRNG Frequency Count Minimum Limit Register (FRQMIN) TRNG Frequency Count Maximum Limit Register (FRQMAX) TRNG Statistical Check Monobit Limit Register (SCML) TRNG Statistical Check Run Length 1 Limit Register (SCR1L) TRNG Statistical Check Run Length 2 Limit Register (SCR2L) TRNG Statistical Check Run Length 3 Limit Register (SCR3L) TRNG Miscellaneous Control Register (MCTL): Sample Mode \\[1:0\\] Oscillator Divider \\[3:2\\] Reset Defaults \\[6\\] Force System Clock \\[7\\] Long Runs Continuation Mode \\[14\\] After this bit has been written to a 1, it cannot be changed"]
    #[inline(always)]
    pub fn no_prgm(&mut self) -> NoPrgmW<SecCfgSpec> {
        NoPrgmW::new(self, 1)
    }
}
#[doc = "Security Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sec_cfg::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sec_cfg::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SecCfgSpec;
impl crate::RegisterSpec for SecCfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sec_cfg::R`](R) reader structure"]
impl crate::Readable for SecCfgSpec {}
#[doc = "`write(|w| ..)` method takes [`sec_cfg::W`](W) writer structure"]
impl crate::Writable for SecCfgSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SEC_CFG to value 0"]
impl crate::Resettable for SecCfgSpec {}

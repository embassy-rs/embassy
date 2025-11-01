#[doc = "Register `OSC2_CTL` reader"]
pub type R = crate::R<Osc2CtlSpec>;
#[doc = "Register `OSC2_CTL` writer"]
pub type W = crate::W<Osc2CtlSpec>;
#[doc = "TRNG entropy generation control.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TrngEntCtl {
    #[doc = "0: Single oscillator mode, using OSC1 (default)"]
    TrngEntCtlSingleOsc1 = 0,
    #[doc = "1: Dual oscillator mode"]
    TrngEntCtlDualOscs = 1,
    #[doc = "2: Single oscillator mode, using OSC2"]
    TrngEntCtlSingleOsc2 = 2,
    #[doc = "3: Unused, (bit field cannot be written to this value)"]
    Osc2DivDiv8 = 3,
}
impl From<TrngEntCtl> for u8 {
    #[inline(always)]
    fn from(variant: TrngEntCtl) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for TrngEntCtl {
    type Ux = u8;
}
impl crate::IsEnum for TrngEntCtl {}
#[doc = "Field `TRNG_ENT_CTL` reader - TRNG entropy generation control."]
pub type TrngEntCtlR = crate::FieldReader<TrngEntCtl>;
impl TrngEntCtlR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> TrngEntCtl {
        match self.bits {
            0 => TrngEntCtl::TrngEntCtlSingleOsc1,
            1 => TrngEntCtl::TrngEntCtlDualOscs,
            2 => TrngEntCtl::TrngEntCtlSingleOsc2,
            3 => TrngEntCtl::Osc2DivDiv8,
            _ => unreachable!(),
        }
    }
    #[doc = "Single oscillator mode, using OSC1 (default)"]
    #[inline(always)]
    pub fn is_trng_ent_ctl_single_osc1(&self) -> bool {
        *self == TrngEntCtl::TrngEntCtlSingleOsc1
    }
    #[doc = "Dual oscillator mode"]
    #[inline(always)]
    pub fn is_trng_ent_ctl_dual_oscs(&self) -> bool {
        *self == TrngEntCtl::TrngEntCtlDualOscs
    }
    #[doc = "Single oscillator mode, using OSC2"]
    #[inline(always)]
    pub fn is_trng_ent_ctl_single_osc2(&self) -> bool {
        *self == TrngEntCtl::TrngEntCtlSingleOsc2
    }
    #[doc = "Unused, (bit field cannot be written to this value)"]
    #[inline(always)]
    pub fn is_osc2_div_div8(&self) -> bool {
        *self == TrngEntCtl::Osc2DivDiv8
    }
}
#[doc = "Field `TRNG_ENT_CTL` writer - TRNG entropy generation control."]
pub type TrngEntCtlW<'a, REG> = crate::FieldWriter<'a, REG, 2, TrngEntCtl, crate::Safe>;
impl<'a, REG> TrngEntCtlW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Single oscillator mode, using OSC1 (default)"]
    #[inline(always)]
    pub fn trng_ent_ctl_single_osc1(self) -> &'a mut crate::W<REG> {
        self.variant(TrngEntCtl::TrngEntCtlSingleOsc1)
    }
    #[doc = "Dual oscillator mode"]
    #[inline(always)]
    pub fn trng_ent_ctl_dual_oscs(self) -> &'a mut crate::W<REG> {
        self.variant(TrngEntCtl::TrngEntCtlDualOscs)
    }
    #[doc = "Single oscillator mode, using OSC2"]
    #[inline(always)]
    pub fn trng_ent_ctl_single_osc2(self) -> &'a mut crate::W<REG> {
        self.variant(TrngEntCtl::TrngEntCtlSingleOsc2)
    }
    #[doc = "Unused, (bit field cannot be written to this value)"]
    #[inline(always)]
    pub fn osc2_div_div8(self) -> &'a mut crate::W<REG> {
        self.variant(TrngEntCtl::Osc2DivDiv8)
    }
}
#[doc = "Oscillator 2 Divide.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Osc2Div {
    #[doc = "0: Use ring oscillator 2 with no divide"]
    Osc2DivDiv1 = 0,
    #[doc = "1: Use ring oscillator 2 divided-by-2"]
    Osc2DivDiv2 = 1,
    #[doc = "2: Use ring oscillator 2 divided-by-4"]
    Osc2DivDiv4 = 2,
    #[doc = "3: Use ring oscillator 2 divided-by-8"]
    Osc2DivDiv8 = 3,
}
impl From<Osc2Div> for u8 {
    #[inline(always)]
    fn from(variant: Osc2Div) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Osc2Div {
    type Ux = u8;
}
impl crate::IsEnum for Osc2Div {}
#[doc = "Field `OSC2_DIV` reader - Oscillator 2 Divide."]
pub type Osc2DivR = crate::FieldReader<Osc2Div>;
impl Osc2DivR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Osc2Div {
        match self.bits {
            0 => Osc2Div::Osc2DivDiv1,
            1 => Osc2Div::Osc2DivDiv2,
            2 => Osc2Div::Osc2DivDiv4,
            3 => Osc2Div::Osc2DivDiv8,
            _ => unreachable!(),
        }
    }
    #[doc = "Use ring oscillator 2 with no divide"]
    #[inline(always)]
    pub fn is_osc2_div_div1(&self) -> bool {
        *self == Osc2Div::Osc2DivDiv1
    }
    #[doc = "Use ring oscillator 2 divided-by-2"]
    #[inline(always)]
    pub fn is_osc2_div_div2(&self) -> bool {
        *self == Osc2Div::Osc2DivDiv2
    }
    #[doc = "Use ring oscillator 2 divided-by-4"]
    #[inline(always)]
    pub fn is_osc2_div_div4(&self) -> bool {
        *self == Osc2Div::Osc2DivDiv4
    }
    #[doc = "Use ring oscillator 2 divided-by-8"]
    #[inline(always)]
    pub fn is_osc2_div_div8(&self) -> bool {
        *self == Osc2Div::Osc2DivDiv8
    }
}
#[doc = "Field `OSC2_DIV` writer - Oscillator 2 Divide."]
pub type Osc2DivW<'a, REG> = crate::FieldWriter<'a, REG, 2, Osc2Div, crate::Safe>;
impl<'a, REG> Osc2DivW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Use ring oscillator 2 with no divide"]
    #[inline(always)]
    pub fn osc2_div_div1(self) -> &'a mut crate::W<REG> {
        self.variant(Osc2Div::Osc2DivDiv1)
    }
    #[doc = "Use ring oscillator 2 divided-by-2"]
    #[inline(always)]
    pub fn osc2_div_div2(self) -> &'a mut crate::W<REG> {
        self.variant(Osc2Div::Osc2DivDiv2)
    }
    #[doc = "Use ring oscillator 2 divided-by-4"]
    #[inline(always)]
    pub fn osc2_div_div4(self) -> &'a mut crate::W<REG> {
        self.variant(Osc2Div::Osc2DivDiv4)
    }
    #[doc = "Use ring oscillator 2 divided-by-8"]
    #[inline(always)]
    pub fn osc2_div_div8(self) -> &'a mut crate::W<REG> {
        self.variant(Osc2Div::Osc2DivDiv8)
    }
}
#[doc = "Oscillator 2 Clock Output Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Osc2OutEn {
    #[doc = "0: Ring oscillator 2 output is gated to an output pad."]
    Osc2OutEn0 = 0,
    #[doc = "1: Allows external viewing of divided-by-2 ring oscillator 2 if MCTL\\[PRGM\\] = 1 mode is also selected, else ring oscillator 2 output is gated to an output pad."]
    Osc2OutEn1 = 1,
}
impl From<Osc2OutEn> for bool {
    #[inline(always)]
    fn from(variant: Osc2OutEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OSC2_OUT_EN` reader - Oscillator 2 Clock Output Enable"]
pub type Osc2OutEnR = crate::BitReader<Osc2OutEn>;
impl Osc2OutEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Osc2OutEn {
        match self.bits {
            false => Osc2OutEn::Osc2OutEn0,
            true => Osc2OutEn::Osc2OutEn1,
        }
    }
    #[doc = "Ring oscillator 2 output is gated to an output pad."]
    #[inline(always)]
    pub fn is_osc2_out_en_0(&self) -> bool {
        *self == Osc2OutEn::Osc2OutEn0
    }
    #[doc = "Allows external viewing of divided-by-2 ring oscillator 2 if MCTL\\[PRGM\\] = 1 mode is also selected, else ring oscillator 2 output is gated to an output pad."]
    #[inline(always)]
    pub fn is_osc2_out_en_1(&self) -> bool {
        *self == Osc2OutEn::Osc2OutEn1
    }
}
#[doc = "Field `OSC2_OUT_EN` writer - Oscillator 2 Clock Output Enable"]
pub type Osc2OutEnW<'a, REG> = crate::BitWriter<'a, REG, Osc2OutEn>;
impl<'a, REG> Osc2OutEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Ring oscillator 2 output is gated to an output pad."]
    #[inline(always)]
    pub fn osc2_out_en_0(self) -> &'a mut crate::W<REG> {
        self.variant(Osc2OutEn::Osc2OutEn0)
    }
    #[doc = "Allows external viewing of divided-by-2 ring oscillator 2 if MCTL\\[PRGM\\] = 1 mode is also selected, else ring oscillator 2 output is gated to an output pad."]
    #[inline(always)]
    pub fn osc2_out_en_1(self) -> &'a mut crate::W<REG> {
        self.variant(Osc2OutEn::Osc2OutEn1)
    }
}
#[doc = "TRNG Oscillator 2 Frequency Count Valid\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Osc2FctVal {
    #[doc = "0: Frequency count is invalid."]
    Disable = 0,
    #[doc = "1: If TRNG_ENT_CTL = 10b, valid frequency count may be read from OSC2_FRQCNT."]
    Enable = 1,
}
impl From<Osc2FctVal> for bool {
    #[inline(always)]
    fn from(variant: Osc2FctVal) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OSC2_FCT_VAL` reader - TRNG Oscillator 2 Frequency Count Valid"]
pub type Osc2FctValR = crate::BitReader<Osc2FctVal>;
impl Osc2FctValR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Osc2FctVal {
        match self.bits {
            false => Osc2FctVal::Disable,
            true => Osc2FctVal::Enable,
        }
    }
    #[doc = "Frequency count is invalid."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Osc2FctVal::Disable
    }
    #[doc = "If TRNG_ENT_CTL = 10b, valid frequency count may be read from OSC2_FRQCNT."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Osc2FctVal::Enable
    }
}
#[doc = "Oscillator fail safe limit.\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum OscFailsafeLmt {
    #[doc = "0: The limit N is 4096 (2^12) system clocks."]
    OscFailsafeLmt4k = 0,
    #[doc = "1: The limit N is 65536 (2^16) system clocks. (default)"]
    OscFailsafeLmt64k = 1,
    #[doc = "2: N is 2^20 system clocks."]
    OscFailsafeLmt1m = 2,
    #[doc = "3: N is 2^22 system clocks (full range of the counter being used)."]
    OscFailsafeLmt4m = 3,
}
impl From<OscFailsafeLmt> for u8 {
    #[inline(always)]
    fn from(variant: OscFailsafeLmt) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for OscFailsafeLmt {
    type Ux = u8;
}
impl crate::IsEnum for OscFailsafeLmt {}
#[doc = "Field `OSC_FAILSAFE_LMT` reader - Oscillator fail safe limit."]
pub type OscFailsafeLmtR = crate::FieldReader<OscFailsafeLmt>;
impl OscFailsafeLmtR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> OscFailsafeLmt {
        match self.bits {
            0 => OscFailsafeLmt::OscFailsafeLmt4k,
            1 => OscFailsafeLmt::OscFailsafeLmt64k,
            2 => OscFailsafeLmt::OscFailsafeLmt1m,
            3 => OscFailsafeLmt::OscFailsafeLmt4m,
            _ => unreachable!(),
        }
    }
    #[doc = "The limit N is 4096 (2^12) system clocks."]
    #[inline(always)]
    pub fn is_osc_failsafe_lmt_4k(&self) -> bool {
        *self == OscFailsafeLmt::OscFailsafeLmt4k
    }
    #[doc = "The limit N is 65536 (2^16) system clocks. (default)"]
    #[inline(always)]
    pub fn is_osc_failsafe_lmt_64k(&self) -> bool {
        *self == OscFailsafeLmt::OscFailsafeLmt64k
    }
    #[doc = "N is 2^20 system clocks."]
    #[inline(always)]
    pub fn is_osc_failsafe_lmt_1m(&self) -> bool {
        *self == OscFailsafeLmt::OscFailsafeLmt1m
    }
    #[doc = "N is 2^22 system clocks (full range of the counter being used)."]
    #[inline(always)]
    pub fn is_osc_failsafe_lmt_4m(&self) -> bool {
        *self == OscFailsafeLmt::OscFailsafeLmt4m
    }
}
#[doc = "Field `OSC_FAILSAFE_LMT` writer - Oscillator fail safe limit."]
pub type OscFailsafeLmtW<'a, REG> = crate::FieldWriter<'a, REG, 2, OscFailsafeLmt, crate::Safe>;
impl<'a, REG> OscFailsafeLmtW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "The limit N is 4096 (2^12) system clocks."]
    #[inline(always)]
    pub fn osc_failsafe_lmt_4k(self) -> &'a mut crate::W<REG> {
        self.variant(OscFailsafeLmt::OscFailsafeLmt4k)
    }
    #[doc = "The limit N is 65536 (2^16) system clocks. (default)"]
    #[inline(always)]
    pub fn osc_failsafe_lmt_64k(self) -> &'a mut crate::W<REG> {
        self.variant(OscFailsafeLmt::OscFailsafeLmt64k)
    }
    #[doc = "N is 2^20 system clocks."]
    #[inline(always)]
    pub fn osc_failsafe_lmt_1m(self) -> &'a mut crate::W<REG> {
        self.variant(OscFailsafeLmt::OscFailsafeLmt1m)
    }
    #[doc = "N is 2^22 system clocks (full range of the counter being used)."]
    #[inline(always)]
    pub fn osc_failsafe_lmt_4m(self) -> &'a mut crate::W<REG> {
        self.variant(OscFailsafeLmt::OscFailsafeLmt4m)
    }
}
#[doc = "Oscillator fail safe test.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OscFailsafeTest {
    #[doc = "0: No impact."]
    Disable = 0,
    #[doc = "1: Disables oscillator 2 while in dual-oscillator mode (TRNG_ENT_CTL = 01b)."]
    Enable = 1,
}
impl From<OscFailsafeTest> for bool {
    #[inline(always)]
    fn from(variant: OscFailsafeTest) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OSC_FAILSAFE_TEST` reader - Oscillator fail safe test."]
pub type OscFailsafeTestR = crate::BitReader<OscFailsafeTest>;
impl OscFailsafeTestR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> OscFailsafeTest {
        match self.bits {
            false => OscFailsafeTest::Disable,
            true => OscFailsafeTest::Enable,
        }
    }
    #[doc = "No impact."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == OscFailsafeTest::Disable
    }
    #[doc = "Disables oscillator 2 while in dual-oscillator mode (TRNG_ENT_CTL = 01b)."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == OscFailsafeTest::Enable
    }
}
#[doc = "Field `OSC_FAILSAFE_TEST` writer - Oscillator fail safe test."]
pub type OscFailsafeTestW<'a, REG> = crate::BitWriter<'a, REG, OscFailsafeTest>;
impl<'a, REG> OscFailsafeTestW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No impact."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(OscFailsafeTest::Disable)
    }
    #[doc = "Disables oscillator 2 while in dual-oscillator mode (TRNG_ENT_CTL = 01b)."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(OscFailsafeTest::Enable)
    }
}
impl R {
    #[doc = "Bits 0:1 - TRNG entropy generation control."]
    #[inline(always)]
    pub fn trng_ent_ctl(&self) -> TrngEntCtlR {
        TrngEntCtlR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 2:3 - Oscillator 2 Divide."]
    #[inline(always)]
    pub fn osc2_div(&self) -> Osc2DivR {
        Osc2DivR::new(((self.bits >> 2) & 3) as u8)
    }
    #[doc = "Bit 4 - Oscillator 2 Clock Output Enable"]
    #[inline(always)]
    pub fn osc2_out_en(&self) -> Osc2OutEnR {
        Osc2OutEnR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 9 - TRNG Oscillator 2 Frequency Count Valid"]
    #[inline(always)]
    pub fn osc2_fct_val(&self) -> Osc2FctValR {
        Osc2FctValR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bits 12:13 - Oscillator fail safe limit."]
    #[inline(always)]
    pub fn osc_failsafe_lmt(&self) -> OscFailsafeLmtR {
        OscFailsafeLmtR::new(((self.bits >> 12) & 3) as u8)
    }
    #[doc = "Bit 14 - Oscillator fail safe test."]
    #[inline(always)]
    pub fn osc_failsafe_test(&self) -> OscFailsafeTestR {
        OscFailsafeTestR::new(((self.bits >> 14) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:1 - TRNG entropy generation control."]
    #[inline(always)]
    pub fn trng_ent_ctl(&mut self) -> TrngEntCtlW<Osc2CtlSpec> {
        TrngEntCtlW::new(self, 0)
    }
    #[doc = "Bits 2:3 - Oscillator 2 Divide."]
    #[inline(always)]
    pub fn osc2_div(&mut self) -> Osc2DivW<Osc2CtlSpec> {
        Osc2DivW::new(self, 2)
    }
    #[doc = "Bit 4 - Oscillator 2 Clock Output Enable"]
    #[inline(always)]
    pub fn osc2_out_en(&mut self) -> Osc2OutEnW<Osc2CtlSpec> {
        Osc2OutEnW::new(self, 4)
    }
    #[doc = "Bits 12:13 - Oscillator fail safe limit."]
    #[inline(always)]
    pub fn osc_failsafe_lmt(&mut self) -> OscFailsafeLmtW<Osc2CtlSpec> {
        OscFailsafeLmtW::new(self, 12)
    }
    #[doc = "Bit 14 - Oscillator fail safe test."]
    #[inline(always)]
    pub fn osc_failsafe_test(&mut self) -> OscFailsafeTestW<Osc2CtlSpec> {
        OscFailsafeTestW::new(self, 14)
    }
}
#[doc = "TRNG Oscillator 2 Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`osc2_ctl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`osc2_ctl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Osc2CtlSpec;
impl crate::RegisterSpec for Osc2CtlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`osc2_ctl::R`](R) reader structure"]
impl crate::Readable for Osc2CtlSpec {}
#[doc = "`write(|w| ..)` method takes [`osc2_ctl::W`](W) writer structure"]
impl crate::Writable for Osc2CtlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets OSC2_CTL to value 0x1000"]
impl crate::Resettable for Osc2CtlSpec {
    const RESET_VALUE: u32 = 0x1000;
}

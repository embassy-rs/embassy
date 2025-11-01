#[doc = "Register `MCTL` reader"]
pub type R = crate::R<MctlSpec>;
#[doc = "Register `MCTL` writer"]
pub type W = crate::W<MctlSpec>;
#[doc = "Oscillator1 Divide\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum OscDiv {
    #[doc = "0: use ring oscillator with no divide"]
    OscDivDiv1 = 0,
    #[doc = "1: use ring oscillator divided-by-2"]
    OscDivDiv2 = 1,
    #[doc = "2: use ring oscillator divided-by-4"]
    OscDivDiv4 = 2,
    #[doc = "3: use ring oscillator divided-by-8"]
    OscDivDiv8 = 3,
}
impl From<OscDiv> for u8 {
    #[inline(always)]
    fn from(variant: OscDiv) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for OscDiv {
    type Ux = u8;
}
impl crate::IsEnum for OscDiv {}
#[doc = "Field `OSC_DIV` reader - Oscillator1 Divide"]
pub type OscDivR = crate::FieldReader<OscDiv>;
impl OscDivR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> OscDiv {
        match self.bits {
            0 => OscDiv::OscDivDiv1,
            1 => OscDiv::OscDivDiv2,
            2 => OscDiv::OscDivDiv4,
            3 => OscDiv::OscDivDiv8,
            _ => unreachable!(),
        }
    }
    #[doc = "use ring oscillator with no divide"]
    #[inline(always)]
    pub fn is_osc_div_div1(&self) -> bool {
        *self == OscDiv::OscDivDiv1
    }
    #[doc = "use ring oscillator divided-by-2"]
    #[inline(always)]
    pub fn is_osc_div_div2(&self) -> bool {
        *self == OscDiv::OscDivDiv2
    }
    #[doc = "use ring oscillator divided-by-4"]
    #[inline(always)]
    pub fn is_osc_div_div4(&self) -> bool {
        *self == OscDiv::OscDivDiv4
    }
    #[doc = "use ring oscillator divided-by-8"]
    #[inline(always)]
    pub fn is_osc_div_div8(&self) -> bool {
        *self == OscDiv::OscDivDiv8
    }
}
#[doc = "Field `OSC_DIV` writer - Oscillator1 Divide"]
pub type OscDivW<'a, REG> = crate::FieldWriter<'a, REG, 2, OscDiv, crate::Safe>;
impl<'a, REG> OscDivW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "use ring oscillator with no divide"]
    #[inline(always)]
    pub fn osc_div_div1(self) -> &'a mut crate::W<REG> {
        self.variant(OscDiv::OscDivDiv1)
    }
    #[doc = "use ring oscillator divided-by-2"]
    #[inline(always)]
    pub fn osc_div_div2(self) -> &'a mut crate::W<REG> {
        self.variant(OscDiv::OscDivDiv2)
    }
    #[doc = "use ring oscillator divided-by-4"]
    #[inline(always)]
    pub fn osc_div_div4(self) -> &'a mut crate::W<REG> {
        self.variant(OscDiv::OscDivDiv4)
    }
    #[doc = "use ring oscillator divided-by-8"]
    #[inline(always)]
    pub fn osc_div_div8(self) -> &'a mut crate::W<REG> {
        self.variant(OscDiv::OscDivDiv8)
    }
}
#[doc = "Field `DIS_SLF_TST` writer - Disable Self-Tests"]
pub type DisSlfTstW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `TRNG_ACC` reader - TRNG Access Mode"]
pub type TrngAccR = crate::BitReader;
#[doc = "Field `TRNG_ACC` writer - TRNG Access Mode"]
pub type TrngAccW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Reset Defaults\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RstDef {
    #[doc = "0: No impact."]
    Disable = 0,
    #[doc = "1: Writing a 1 to this bit clears various TRNG registers, and bits within registers, to their default state."]
    Enable = 1,
}
impl From<RstDef> for bool {
    #[inline(always)]
    fn from(variant: RstDef) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RST_DEF` writer - Reset Defaults"]
pub type RstDefW<'a, REG> = crate::BitWriter<'a, REG, RstDef>;
impl<'a, REG> RstDefW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No impact."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(RstDef::Disable)
    }
    #[doc = "Writing a 1 to this bit clears various TRNG registers, and bits within registers, to their default state."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(RstDef::Enable)
    }
}
#[doc = "Field `FCT_FAIL` reader - Frequency Count Fail"]
pub type FctFailR = crate::BitReader;
#[doc = "Frequency Count Valid\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FctVal {
    #[doc = "0: Frequency Count is not valid"]
    Disable = 0,
    #[doc = "1: Frequency Count is valid"]
    Enable = 1,
}
impl From<FctVal> for bool {
    #[inline(always)]
    fn from(variant: FctVal) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FCT_VAL` reader - Frequency Count Valid"]
pub type FctValR = crate::BitReader<FctVal>;
impl FctValR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> FctVal {
        match self.bits {
            false => FctVal::Disable,
            true => FctVal::Enable,
        }
    }
    #[doc = "Frequency Count is not valid"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == FctVal::Disable
    }
    #[doc = "Frequency Count is valid"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == FctVal::Enable
    }
}
#[doc = "Entropy Valid\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntVal {
    #[doc = "0: Entropy is not valid"]
    Disable = 0,
    #[doc = "1: Entropy is valid"]
    Enable = 1,
}
impl From<EntVal> for bool {
    #[inline(always)]
    fn from(variant: EntVal) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ENT_VAL` reader - Entropy Valid"]
pub type EntValR = crate::BitReader<EntVal>;
impl EntValR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> EntVal {
        match self.bits {
            false => EntVal::Disable,
            true => EntVal::Enable,
        }
    }
    #[doc = "Entropy is not valid"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == EntVal::Disable
    }
    #[doc = "Entropy is valid"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == EntVal::Enable
    }
}
#[doc = "Error Status\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Err {
    #[doc = "0: No error"]
    Disable = 0,
    #[doc = "1: Error detected"]
    Enable = 1,
}
impl From<Err> for bool {
    #[inline(always)]
    fn from(variant: Err) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERR` reader - Error Status"]
pub type ErrR = crate::BitReader<Err>;
impl ErrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Err {
        match self.bits {
            false => Err::Disable,
            true => Err::Enable,
        }
    }
    #[doc = "No error"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Err::Disable
    }
    #[doc = "Error detected"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Err::Enable
    }
}
#[doc = "Field `ERR` writer - Error Status"]
pub type ErrW<'a, REG> = crate::BitWriter1C<'a, REG, Err>;
impl<'a, REG> ErrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No error"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Err::Disable)
    }
    #[doc = "Error detected"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Err::Enable)
    }
}
#[doc = "TRNG is ok to stop\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TstopOk {
    #[doc = "0: TRNG is generating entropy and is not ok to stop"]
    Disable = 0,
    #[doc = "1: TRNG is not generating entropy and is ok to stop"]
    Enable = 1,
}
impl From<TstopOk> for bool {
    #[inline(always)]
    fn from(variant: TstopOk) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TSTOP_OK` reader - TRNG is ok to stop"]
pub type TstopOkR = crate::BitReader<TstopOk>;
impl TstopOkR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> TstopOk {
        match self.bits {
            false => TstopOk::Disable,
            true => TstopOk::Enable,
        }
    }
    #[doc = "TRNG is generating entropy and is not ok to stop"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == TstopOk::Disable
    }
    #[doc = "TRNG is not generating entropy and is ok to stop"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == TstopOk::Enable
    }
}
#[doc = "Oscillator 2 Failure\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Osc2Fail {
    #[doc = "0: Oscillator 2 is running."]
    Disable = 0,
    #[doc = "1: Oscillator 2 has failed (see OSC2_CTL\\[OSC_FAILSAFE_LMT\\])."]
    Enable = 1,
}
impl From<Osc2Fail> for bool {
    #[inline(always)]
    fn from(variant: Osc2Fail) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OSC2_FAIL` reader - Oscillator 2 Failure"]
pub type Osc2FailR = crate::BitReader<Osc2Fail>;
impl Osc2FailR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Osc2Fail {
        match self.bits {
            false => Osc2Fail::Disable,
            true => Osc2Fail::Enable,
        }
    }
    #[doc = "Oscillator 2 is running."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Osc2Fail::Disable
    }
    #[doc = "Oscillator 2 has failed (see OSC2_CTL\\[OSC_FAILSAFE_LMT\\])."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Osc2Fail::Enable
    }
}
#[doc = "Program Mode\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prgm {
    #[doc = "0: TRNG is in Run Mode"]
    Disable = 0,
    #[doc = "1: TRNG is in Program Mode"]
    Enable = 1,
}
impl From<Prgm> for bool {
    #[inline(always)]
    fn from(variant: Prgm) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PRGM` reader - Program Mode"]
pub type PrgmR = crate::BitReader<Prgm>;
impl PrgmR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Prgm {
        match self.bits {
            false => Prgm::Disable,
            true => Prgm::Enable,
        }
    }
    #[doc = "TRNG is in Run Mode"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Prgm::Disable
    }
    #[doc = "TRNG is in Program Mode"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Prgm::Enable
    }
}
#[doc = "Field `PRGM` writer - Program Mode"]
pub type PrgmW<'a, REG> = crate::BitWriter<'a, REG, Prgm>;
impl<'a, REG> PrgmW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "TRNG is in Run Mode"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Prgm::Disable)
    }
    #[doc = "TRNG is in Program Mode"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Prgm::Enable)
    }
}
#[doc = "Integrity Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntgErr {
    #[doc = "0: TRNG detected no internal bit error"]
    Disable = 0,
    #[doc = "1: TRNG detected internal bit error(s)"]
    Enable = 1,
}
impl From<IntgErr> for bool {
    #[inline(always)]
    fn from(variant: IntgErr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INTG_ERR` reader - Integrity Error"]
pub type IntgErrR = crate::BitReader<IntgErr>;
impl IntgErrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> IntgErr {
        match self.bits {
            false => IntgErr::Disable,
            true => IntgErr::Enable,
        }
    }
    #[doc = "TRNG detected no internal bit error"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == IntgErr::Disable
    }
    #[doc = "TRNG detected internal bit error(s)"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == IntgErr::Enable
    }
}
impl R {
    #[doc = "Bits 2:3 - Oscillator1 Divide"]
    #[inline(always)]
    pub fn osc_div(&self) -> OscDivR {
        OscDivR::new(((self.bits >> 2) & 3) as u8)
    }
    #[doc = "Bit 5 - TRNG Access Mode"]
    #[inline(always)]
    pub fn trng_acc(&self) -> TrngAccR {
        TrngAccR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 8 - Frequency Count Fail"]
    #[inline(always)]
    pub fn fct_fail(&self) -> FctFailR {
        FctFailR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Frequency Count Valid"]
    #[inline(always)]
    pub fn fct_val(&self) -> FctValR {
        FctValR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Entropy Valid"]
    #[inline(always)]
    pub fn ent_val(&self) -> EntValR {
        EntValR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 12 - Error Status"]
    #[inline(always)]
    pub fn err(&self) -> ErrR {
        ErrR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - TRNG is ok to stop"]
    #[inline(always)]
    pub fn tstop_ok(&self) -> TstopOkR {
        TstopOkR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 15 - Oscillator 2 Failure"]
    #[inline(always)]
    pub fn osc2_fail(&self) -> Osc2FailR {
        Osc2FailR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - Program Mode"]
    #[inline(always)]
    pub fn prgm(&self) -> PrgmR {
        PrgmR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 31 - Integrity Error"]
    #[inline(always)]
    pub fn intg_err(&self) -> IntgErrR {
        IntgErrR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 2:3 - Oscillator1 Divide"]
    #[inline(always)]
    pub fn osc_div(&mut self) -> OscDivW<MctlSpec> {
        OscDivW::new(self, 2)
    }
    #[doc = "Bit 4 - Disable Self-Tests"]
    #[inline(always)]
    pub fn dis_slf_tst(&mut self) -> DisSlfTstW<MctlSpec> {
        DisSlfTstW::new(self, 4)
    }
    #[doc = "Bit 5 - TRNG Access Mode"]
    #[inline(always)]
    pub fn trng_acc(&mut self) -> TrngAccW<MctlSpec> {
        TrngAccW::new(self, 5)
    }
    #[doc = "Bit 6 - Reset Defaults"]
    #[inline(always)]
    pub fn rst_def(&mut self) -> RstDefW<MctlSpec> {
        RstDefW::new(self, 6)
    }
    #[doc = "Bit 12 - Error Status"]
    #[inline(always)]
    pub fn err(&mut self) -> ErrW<MctlSpec> {
        ErrW::new(self, 12)
    }
    #[doc = "Bit 16 - Program Mode"]
    #[inline(always)]
    pub fn prgm(&mut self) -> PrgmW<MctlSpec> {
        PrgmW::new(self, 16)
    }
}
#[doc = "Miscellaneous Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mctl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mctl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MctlSpec;
impl crate::RegisterSpec for MctlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mctl::R`](R) reader structure"]
impl crate::Readable for MctlSpec {}
#[doc = "`write(|w| ..)` method takes [`mctl::W`](W) writer structure"]
impl crate::Writable for MctlSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x1000;
}
#[doc = "`reset()` method sets MCTL to value 0x0001_2000"]
impl crate::Resettable for MctlSpec {
    const RESET_VALUE: u32 = 0x0001_2000;
}

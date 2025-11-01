#[doc = "Register `TIMCTL[%s]` reader"]
pub type R = crate::R<TimctlSpec>;
#[doc = "Register `TIMCTL[%s]` writer"]
pub type W = crate::W<TimctlSpec>;
#[doc = "Timer Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Timod {
    #[doc = "0: Timer disabled"]
    Disable = 0,
    #[doc = "1: Dual 8-bit counters baud mode"]
    Dual8bitBaud = 1,
    #[doc = "2: Dual 8-bit counters PWM high mode"]
    Dual8bitPwmH = 2,
    #[doc = "3: Single 16-bit counter mode"]
    Single16bit = 3,
    #[doc = "4: Single 16-bit counter disable mode"]
    Single16bitDisable = 4,
    #[doc = "5: Dual 8-bit counters word mode"]
    Dual8bitWord = 5,
    #[doc = "6: Dual 8-bit counters PWM low mode"]
    Dual8bitPwmL = 6,
    #[doc = "7: Single 16-bit input capture mode"]
    Single16bitInCapture = 7,
}
impl From<Timod> for u8 {
    #[inline(always)]
    fn from(variant: Timod) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Timod {
    type Ux = u8;
}
impl crate::IsEnum for Timod {}
#[doc = "Field `TIMOD` reader - Timer Mode"]
pub type TimodR = crate::FieldReader<Timod>;
impl TimodR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Timod {
        match self.bits {
            0 => Timod::Disable,
            1 => Timod::Dual8bitBaud,
            2 => Timod::Dual8bitPwmH,
            3 => Timod::Single16bit,
            4 => Timod::Single16bitDisable,
            5 => Timod::Dual8bitWord,
            6 => Timod::Dual8bitPwmL,
            7 => Timod::Single16bitInCapture,
            _ => unreachable!(),
        }
    }
    #[doc = "Timer disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Timod::Disable
    }
    #[doc = "Dual 8-bit counters baud mode"]
    #[inline(always)]
    pub fn is_dual8bit_baud(&self) -> bool {
        *self == Timod::Dual8bitBaud
    }
    #[doc = "Dual 8-bit counters PWM high mode"]
    #[inline(always)]
    pub fn is_dual8bit_pwm_h(&self) -> bool {
        *self == Timod::Dual8bitPwmH
    }
    #[doc = "Single 16-bit counter mode"]
    #[inline(always)]
    pub fn is_single16bit(&self) -> bool {
        *self == Timod::Single16bit
    }
    #[doc = "Single 16-bit counter disable mode"]
    #[inline(always)]
    pub fn is_single16bit_disable(&self) -> bool {
        *self == Timod::Single16bitDisable
    }
    #[doc = "Dual 8-bit counters word mode"]
    #[inline(always)]
    pub fn is_dual8bit_word(&self) -> bool {
        *self == Timod::Dual8bitWord
    }
    #[doc = "Dual 8-bit counters PWM low mode"]
    #[inline(always)]
    pub fn is_dual8bit_pwm_l(&self) -> bool {
        *self == Timod::Dual8bitPwmL
    }
    #[doc = "Single 16-bit input capture mode"]
    #[inline(always)]
    pub fn is_single16bit_in_capture(&self) -> bool {
        *self == Timod::Single16bitInCapture
    }
}
#[doc = "Field `TIMOD` writer - Timer Mode"]
pub type TimodW<'a, REG> = crate::FieldWriter<'a, REG, 3, Timod, crate::Safe>;
impl<'a, REG> TimodW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Timer disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Timod::Disable)
    }
    #[doc = "Dual 8-bit counters baud mode"]
    #[inline(always)]
    pub fn dual8bit_baud(self) -> &'a mut crate::W<REG> {
        self.variant(Timod::Dual8bitBaud)
    }
    #[doc = "Dual 8-bit counters PWM high mode"]
    #[inline(always)]
    pub fn dual8bit_pwm_h(self) -> &'a mut crate::W<REG> {
        self.variant(Timod::Dual8bitPwmH)
    }
    #[doc = "Single 16-bit counter mode"]
    #[inline(always)]
    pub fn single16bit(self) -> &'a mut crate::W<REG> {
        self.variant(Timod::Single16bit)
    }
    #[doc = "Single 16-bit counter disable mode"]
    #[inline(always)]
    pub fn single16bit_disable(self) -> &'a mut crate::W<REG> {
        self.variant(Timod::Single16bitDisable)
    }
    #[doc = "Dual 8-bit counters word mode"]
    #[inline(always)]
    pub fn dual8bit_word(self) -> &'a mut crate::W<REG> {
        self.variant(Timod::Dual8bitWord)
    }
    #[doc = "Dual 8-bit counters PWM low mode"]
    #[inline(always)]
    pub fn dual8bit_pwm_l(self) -> &'a mut crate::W<REG> {
        self.variant(Timod::Dual8bitPwmL)
    }
    #[doc = "Single 16-bit input capture mode"]
    #[inline(always)]
    pub fn single16bit_in_capture(self) -> &'a mut crate::W<REG> {
        self.variant(Timod::Single16bitInCapture)
    }
}
#[doc = "Timer One Time Operation\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Onetim {
    #[doc = "0: Generate the timer enable event as normal"]
    NotBlocked = 0,
    #[doc = "1: Block the timer enable event unless the timer status flag is clear"]
    Blocked = 1,
}
impl From<Onetim> for bool {
    #[inline(always)]
    fn from(variant: Onetim) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ONETIM` reader - Timer One Time Operation"]
pub type OnetimR = crate::BitReader<Onetim>;
impl OnetimR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Onetim {
        match self.bits {
            false => Onetim::NotBlocked,
            true => Onetim::Blocked,
        }
    }
    #[doc = "Generate the timer enable event as normal"]
    #[inline(always)]
    pub fn is_not_blocked(&self) -> bool {
        *self == Onetim::NotBlocked
    }
    #[doc = "Block the timer enable event unless the timer status flag is clear"]
    #[inline(always)]
    pub fn is_blocked(&self) -> bool {
        *self == Onetim::Blocked
    }
}
#[doc = "Field `ONETIM` writer - Timer One Time Operation"]
pub type OnetimW<'a, REG> = crate::BitWriter<'a, REG, Onetim>;
impl<'a, REG> OnetimW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Generate the timer enable event as normal"]
    #[inline(always)]
    pub fn not_blocked(self) -> &'a mut crate::W<REG> {
        self.variant(Onetim::NotBlocked)
    }
    #[doc = "Block the timer enable event unless the timer status flag is clear"]
    #[inline(always)]
    pub fn blocked(self) -> &'a mut crate::W<REG> {
        self.variant(Onetim::Blocked)
    }
}
#[doc = "Timer Pin Input Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pinins {
    #[doc = "0: PINSEL selects timer pin input and output"]
    Pinsel = 0,
    #[doc = "1: PINSEL + 1 selects the timer pin input; timer pin output remains selected by PINSEL"]
    Pinselplus1 = 1,
}
impl From<Pinins> for bool {
    #[inline(always)]
    fn from(variant: Pinins) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PININS` reader - Timer Pin Input Select"]
pub type PininsR = crate::BitReader<Pinins>;
impl PininsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pinins {
        match self.bits {
            false => Pinins::Pinsel,
            true => Pinins::Pinselplus1,
        }
    }
    #[doc = "PINSEL selects timer pin input and output"]
    #[inline(always)]
    pub fn is_pinsel(&self) -> bool {
        *self == Pinins::Pinsel
    }
    #[doc = "PINSEL + 1 selects the timer pin input; timer pin output remains selected by PINSEL"]
    #[inline(always)]
    pub fn is_pinselplus1(&self) -> bool {
        *self == Pinins::Pinselplus1
    }
}
#[doc = "Field `PININS` writer - Timer Pin Input Select"]
pub type PininsW<'a, REG> = crate::BitWriter<'a, REG, Pinins>;
impl<'a, REG> PininsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "PINSEL selects timer pin input and output"]
    #[inline(always)]
    pub fn pinsel(self) -> &'a mut crate::W<REG> {
        self.variant(Pinins::Pinsel)
    }
    #[doc = "PINSEL + 1 selects the timer pin input; timer pin output remains selected by PINSEL"]
    #[inline(always)]
    pub fn pinselplus1(self) -> &'a mut crate::W<REG> {
        self.variant(Pinins::Pinselplus1)
    }
}
#[doc = "Timer Pin Polarity\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pinpol {
    #[doc = "0: Active high"]
    ActiveHigh = 0,
    #[doc = "1: Active low"]
    ActiveLow = 1,
}
impl From<Pinpol> for bool {
    #[inline(always)]
    fn from(variant: Pinpol) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PINPOL` reader - Timer Pin Polarity"]
pub type PinpolR = crate::BitReader<Pinpol>;
impl PinpolR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pinpol {
        match self.bits {
            false => Pinpol::ActiveHigh,
            true => Pinpol::ActiveLow,
        }
    }
    #[doc = "Active high"]
    #[inline(always)]
    pub fn is_active_high(&self) -> bool {
        *self == Pinpol::ActiveHigh
    }
    #[doc = "Active low"]
    #[inline(always)]
    pub fn is_active_low(&self) -> bool {
        *self == Pinpol::ActiveLow
    }
}
#[doc = "Field `PINPOL` writer - Timer Pin Polarity"]
pub type PinpolW<'a, REG> = crate::BitWriter<'a, REG, Pinpol>;
impl<'a, REG> PinpolW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Active high"]
    #[inline(always)]
    pub fn active_high(self) -> &'a mut crate::W<REG> {
        self.variant(Pinpol::ActiveHigh)
    }
    #[doc = "Active low"]
    #[inline(always)]
    pub fn active_low(self) -> &'a mut crate::W<REG> {
        self.variant(Pinpol::ActiveLow)
    }
}
#[doc = "Field `PINSEL` reader - Timer Pin Select"]
pub type PinselR = crate::FieldReader;
#[doc = "Field `PINSEL` writer - Timer Pin Select"]
pub type PinselW<'a, REG> = crate::FieldWriter<'a, REG, 5>;
#[doc = "Timer Pin Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pincfg {
    #[doc = "0: Timer pin output disabled"]
    Outdisable = 0,
    #[doc = "1: Timer pin open-drain or bidirectional output enable"]
    OpendBidirouten = 1,
    #[doc = "2: Timer pin bidirectional output data"]
    BidirOutdata = 2,
    #[doc = "3: Timer pin output"]
    Output = 3,
}
impl From<Pincfg> for u8 {
    #[inline(always)]
    fn from(variant: Pincfg) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pincfg {
    type Ux = u8;
}
impl crate::IsEnum for Pincfg {}
#[doc = "Field `PINCFG` reader - Timer Pin Configuration"]
pub type PincfgR = crate::FieldReader<Pincfg>;
impl PincfgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pincfg {
        match self.bits {
            0 => Pincfg::Outdisable,
            1 => Pincfg::OpendBidirouten,
            2 => Pincfg::BidirOutdata,
            3 => Pincfg::Output,
            _ => unreachable!(),
        }
    }
    #[doc = "Timer pin output disabled"]
    #[inline(always)]
    pub fn is_outdisable(&self) -> bool {
        *self == Pincfg::Outdisable
    }
    #[doc = "Timer pin open-drain or bidirectional output enable"]
    #[inline(always)]
    pub fn is_opend_bidirouten(&self) -> bool {
        *self == Pincfg::OpendBidirouten
    }
    #[doc = "Timer pin bidirectional output data"]
    #[inline(always)]
    pub fn is_bidir_outdata(&self) -> bool {
        *self == Pincfg::BidirOutdata
    }
    #[doc = "Timer pin output"]
    #[inline(always)]
    pub fn is_output(&self) -> bool {
        *self == Pincfg::Output
    }
}
#[doc = "Field `PINCFG` writer - Timer Pin Configuration"]
pub type PincfgW<'a, REG> = crate::FieldWriter<'a, REG, 2, Pincfg, crate::Safe>;
impl<'a, REG> PincfgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Timer pin output disabled"]
    #[inline(always)]
    pub fn outdisable(self) -> &'a mut crate::W<REG> {
        self.variant(Pincfg::Outdisable)
    }
    #[doc = "Timer pin open-drain or bidirectional output enable"]
    #[inline(always)]
    pub fn opend_bidirouten(self) -> &'a mut crate::W<REG> {
        self.variant(Pincfg::OpendBidirouten)
    }
    #[doc = "Timer pin bidirectional output data"]
    #[inline(always)]
    pub fn bidir_outdata(self) -> &'a mut crate::W<REG> {
        self.variant(Pincfg::BidirOutdata)
    }
    #[doc = "Timer pin output"]
    #[inline(always)]
    pub fn output(self) -> &'a mut crate::W<REG> {
        self.variant(Pincfg::Output)
    }
}
#[doc = "Trigger Source\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trgsrc {
    #[doc = "0: External"]
    ExtTrig = 0,
    #[doc = "1: Internal"]
    InternalTrig = 1,
}
impl From<Trgsrc> for bool {
    #[inline(always)]
    fn from(variant: Trgsrc) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TRGSRC` reader - Trigger Source"]
pub type TrgsrcR = crate::BitReader<Trgsrc>;
impl TrgsrcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Trgsrc {
        match self.bits {
            false => Trgsrc::ExtTrig,
            true => Trgsrc::InternalTrig,
        }
    }
    #[doc = "External"]
    #[inline(always)]
    pub fn is_ext_trig(&self) -> bool {
        *self == Trgsrc::ExtTrig
    }
    #[doc = "Internal"]
    #[inline(always)]
    pub fn is_internal_trig(&self) -> bool {
        *self == Trgsrc::InternalTrig
    }
}
#[doc = "Field `TRGSRC` writer - Trigger Source"]
pub type TrgsrcW<'a, REG> = crate::BitWriter<'a, REG, Trgsrc>;
impl<'a, REG> TrgsrcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "External"]
    #[inline(always)]
    pub fn ext_trig(self) -> &'a mut crate::W<REG> {
        self.variant(Trgsrc::ExtTrig)
    }
    #[doc = "Internal"]
    #[inline(always)]
    pub fn internal_trig(self) -> &'a mut crate::W<REG> {
        self.variant(Trgsrc::InternalTrig)
    }
}
#[doc = "Trigger Polarity\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trgpol {
    #[doc = "0: Active high"]
    ActiveHigh = 0,
    #[doc = "1: Active low"]
    ActiveLow = 1,
}
impl From<Trgpol> for bool {
    #[inline(always)]
    fn from(variant: Trgpol) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TRGPOL` reader - Trigger Polarity"]
pub type TrgpolR = crate::BitReader<Trgpol>;
impl TrgpolR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Trgpol {
        match self.bits {
            false => Trgpol::ActiveHigh,
            true => Trgpol::ActiveLow,
        }
    }
    #[doc = "Active high"]
    #[inline(always)]
    pub fn is_active_high(&self) -> bool {
        *self == Trgpol::ActiveHigh
    }
    #[doc = "Active low"]
    #[inline(always)]
    pub fn is_active_low(&self) -> bool {
        *self == Trgpol::ActiveLow
    }
}
#[doc = "Field `TRGPOL` writer - Trigger Polarity"]
pub type TrgpolW<'a, REG> = crate::BitWriter<'a, REG, Trgpol>;
impl<'a, REG> TrgpolW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Active high"]
    #[inline(always)]
    pub fn active_high(self) -> &'a mut crate::W<REG> {
        self.variant(Trgpol::ActiveHigh)
    }
    #[doc = "Active low"]
    #[inline(always)]
    pub fn active_low(self) -> &'a mut crate::W<REG> {
        self.variant(Trgpol::ActiveLow)
    }
}
#[doc = "Field `TRGSEL` reader - Trigger Select"]
pub type TrgselR = crate::FieldReader;
#[doc = "Field `TRGSEL` writer - Trigger Select"]
pub type TrgselW<'a, REG> = crate::FieldWriter<'a, REG, 6>;
impl R {
    #[doc = "Bits 0:2 - Timer Mode"]
    #[inline(always)]
    pub fn timod(&self) -> TimodR {
        TimodR::new((self.bits & 7) as u8)
    }
    #[doc = "Bit 5 - Timer One Time Operation"]
    #[inline(always)]
    pub fn onetim(&self) -> OnetimR {
        OnetimR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Timer Pin Input Select"]
    #[inline(always)]
    pub fn pinins(&self) -> PininsR {
        PininsR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Timer Pin Polarity"]
    #[inline(always)]
    pub fn pinpol(&self) -> PinpolR {
        PinpolR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bits 8:12 - Timer Pin Select"]
    #[inline(always)]
    pub fn pinsel(&self) -> PinselR {
        PinselR::new(((self.bits >> 8) & 0x1f) as u8)
    }
    #[doc = "Bits 16:17 - Timer Pin Configuration"]
    #[inline(always)]
    pub fn pincfg(&self) -> PincfgR {
        PincfgR::new(((self.bits >> 16) & 3) as u8)
    }
    #[doc = "Bit 22 - Trigger Source"]
    #[inline(always)]
    pub fn trgsrc(&self) -> TrgsrcR {
        TrgsrcR::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - Trigger Polarity"]
    #[inline(always)]
    pub fn trgpol(&self) -> TrgpolR {
        TrgpolR::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bits 24:29 - Trigger Select"]
    #[inline(always)]
    pub fn trgsel(&self) -> TrgselR {
        TrgselR::new(((self.bits >> 24) & 0x3f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:2 - Timer Mode"]
    #[inline(always)]
    pub fn timod(&mut self) -> TimodW<TimctlSpec> {
        TimodW::new(self, 0)
    }
    #[doc = "Bit 5 - Timer One Time Operation"]
    #[inline(always)]
    pub fn onetim(&mut self) -> OnetimW<TimctlSpec> {
        OnetimW::new(self, 5)
    }
    #[doc = "Bit 6 - Timer Pin Input Select"]
    #[inline(always)]
    pub fn pinins(&mut self) -> PininsW<TimctlSpec> {
        PininsW::new(self, 6)
    }
    #[doc = "Bit 7 - Timer Pin Polarity"]
    #[inline(always)]
    pub fn pinpol(&mut self) -> PinpolW<TimctlSpec> {
        PinpolW::new(self, 7)
    }
    #[doc = "Bits 8:12 - Timer Pin Select"]
    #[inline(always)]
    pub fn pinsel(&mut self) -> PinselW<TimctlSpec> {
        PinselW::new(self, 8)
    }
    #[doc = "Bits 16:17 - Timer Pin Configuration"]
    #[inline(always)]
    pub fn pincfg(&mut self) -> PincfgW<TimctlSpec> {
        PincfgW::new(self, 16)
    }
    #[doc = "Bit 22 - Trigger Source"]
    #[inline(always)]
    pub fn trgsrc(&mut self) -> TrgsrcW<TimctlSpec> {
        TrgsrcW::new(self, 22)
    }
    #[doc = "Bit 23 - Trigger Polarity"]
    #[inline(always)]
    pub fn trgpol(&mut self) -> TrgpolW<TimctlSpec> {
        TrgpolW::new(self, 23)
    }
    #[doc = "Bits 24:29 - Trigger Select"]
    #[inline(always)]
    pub fn trgsel(&mut self) -> TrgselW<TimctlSpec> {
        TrgselW::new(self, 24)
    }
}
#[doc = "Timer Control\n\nYou can [`read`](crate::Reg::read) this register and get [`timctl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`timctl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TimctlSpec;
impl crate::RegisterSpec for TimctlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`timctl::R`](R) reader structure"]
impl crate::Readable for TimctlSpec {}
#[doc = "`write(|w| ..)` method takes [`timctl::W`](W) writer structure"]
impl crate::Writable for TimctlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TIMCTL[%s] to value 0"]
impl crate::Resettable for TimctlSpec {}

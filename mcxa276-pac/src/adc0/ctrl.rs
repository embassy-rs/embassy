#[doc = "Register `CTRL` reader"]
pub type R = crate::R<CtrlSpec>;
#[doc = "Register `CTRL` writer"]
pub type W = crate::W<CtrlSpec>;
#[doc = "ADC Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Adcen {
    #[doc = "0: ADC is disabled."]
    Disabled = 0,
    #[doc = "1: ADC is enabled."]
    Enabled = 1,
}
impl From<Adcen> for bool {
    #[inline(always)]
    fn from(variant: Adcen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ADCEN` reader - ADC Enable"]
pub type AdcenR = crate::BitReader<Adcen>;
impl AdcenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Adcen {
        match self.bits {
            false => Adcen::Disabled,
            true => Adcen::Enabled,
        }
    }
    #[doc = "ADC is disabled."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Adcen::Disabled
    }
    #[doc = "ADC is enabled."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Adcen::Enabled
    }
}
#[doc = "Field `ADCEN` writer - ADC Enable"]
pub type AdcenW<'a, REG> = crate::BitWriter<'a, REG, Adcen>;
impl<'a, REG> AdcenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "ADC is disabled."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Adcen::Disabled)
    }
    #[doc = "ADC is enabled."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Adcen::Enabled)
    }
}
#[doc = "Software Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rst {
    #[doc = "0: ADC logic is not reset."]
    ReleasedFromReset = 0,
    #[doc = "1: ADC logic is reset."]
    HeldInReset = 1,
}
impl From<Rst> for bool {
    #[inline(always)]
    fn from(variant: Rst) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RST` reader - Software Reset"]
pub type RstR = crate::BitReader<Rst>;
impl RstR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rst {
        match self.bits {
            false => Rst::ReleasedFromReset,
            true => Rst::HeldInReset,
        }
    }
    #[doc = "ADC logic is not reset."]
    #[inline(always)]
    pub fn is_released_from_reset(&self) -> bool {
        *self == Rst::ReleasedFromReset
    }
    #[doc = "ADC logic is reset."]
    #[inline(always)]
    pub fn is_held_in_reset(&self) -> bool {
        *self == Rst::HeldInReset
    }
}
#[doc = "Field `RST` writer - Software Reset"]
pub type RstW<'a, REG> = crate::BitWriter<'a, REG, Rst>;
impl<'a, REG> RstW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "ADC logic is not reset."]
    #[inline(always)]
    pub fn released_from_reset(self) -> &'a mut crate::W<REG> {
        self.variant(Rst::ReleasedFromReset)
    }
    #[doc = "ADC logic is reset."]
    #[inline(always)]
    pub fn held_in_reset(self) -> &'a mut crate::W<REG> {
        self.variant(Rst::HeldInReset)
    }
}
#[doc = "Doze Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dozen {
    #[doc = "0: ADC is enabled in low power mode."]
    Enabled = 0,
    #[doc = "1: ADC is disabled in low power mode."]
    Disabled = 1,
}
impl From<Dozen> for bool {
    #[inline(always)]
    fn from(variant: Dozen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DOZEN` reader - Doze Enable"]
pub type DozenR = crate::BitReader<Dozen>;
impl DozenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dozen {
        match self.bits {
            false => Dozen::Enabled,
            true => Dozen::Disabled,
        }
    }
    #[doc = "ADC is enabled in low power mode."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Dozen::Enabled
    }
    #[doc = "ADC is disabled in low power mode."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Dozen::Disabled
    }
}
#[doc = "Field `DOZEN` writer - Doze Enable"]
pub type DozenW<'a, REG> = crate::BitWriter<'a, REG, Dozen>;
impl<'a, REG> DozenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "ADC is enabled in low power mode."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Dozen::Enabled)
    }
    #[doc = "ADC is disabled in low power mode."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Dozen::Disabled)
    }
}
#[doc = "Auto-Calibration Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalReq {
    #[doc = "0: No request for hardware calibration has been made"]
    NoCalibrationRequest = 0,
    #[doc = "1: A request for hardware calibration has been made"]
    CalibrationRequestPending = 1,
}
impl From<CalReq> for bool {
    #[inline(always)]
    fn from(variant: CalReq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CAL_REQ` reader - Auto-Calibration Request"]
pub type CalReqR = crate::BitReader<CalReq>;
impl CalReqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> CalReq {
        match self.bits {
            false => CalReq::NoCalibrationRequest,
            true => CalReq::CalibrationRequestPending,
        }
    }
    #[doc = "No request for hardware calibration has been made"]
    #[inline(always)]
    pub fn is_no_calibration_request(&self) -> bool {
        *self == CalReq::NoCalibrationRequest
    }
    #[doc = "A request for hardware calibration has been made"]
    #[inline(always)]
    pub fn is_calibration_request_pending(&self) -> bool {
        *self == CalReq::CalibrationRequestPending
    }
}
#[doc = "Field `CAL_REQ` writer - Auto-Calibration Request"]
pub type CalReqW<'a, REG> = crate::BitWriter<'a, REG, CalReq>;
impl<'a, REG> CalReqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No request for hardware calibration has been made"]
    #[inline(always)]
    pub fn no_calibration_request(self) -> &'a mut crate::W<REG> {
        self.variant(CalReq::NoCalibrationRequest)
    }
    #[doc = "A request for hardware calibration has been made"]
    #[inline(always)]
    pub fn calibration_request_pending(self) -> &'a mut crate::W<REG> {
        self.variant(CalReq::CalibrationRequestPending)
    }
}
#[doc = "Offset Calibration Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Calofs {
    #[doc = "0: No request for offset calibration has been made"]
    NoActiveOffsetCalibrationRequest = 0,
    #[doc = "1: Request for offset calibration function"]
    OffsetCalibrationRequestPending = 1,
}
impl From<Calofs> for bool {
    #[inline(always)]
    fn from(variant: Calofs) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CALOFS` reader - Offset Calibration Request"]
pub type CalofsR = crate::BitReader<Calofs>;
impl CalofsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Calofs {
        match self.bits {
            false => Calofs::NoActiveOffsetCalibrationRequest,
            true => Calofs::OffsetCalibrationRequestPending,
        }
    }
    #[doc = "No request for offset calibration has been made"]
    #[inline(always)]
    pub fn is_no_active_offset_calibration_request(&self) -> bool {
        *self == Calofs::NoActiveOffsetCalibrationRequest
    }
    #[doc = "Request for offset calibration function"]
    #[inline(always)]
    pub fn is_offset_calibration_request_pending(&self) -> bool {
        *self == Calofs::OffsetCalibrationRequestPending
    }
}
#[doc = "Field `CALOFS` writer - Offset Calibration Request"]
pub type CalofsW<'a, REG> = crate::BitWriter<'a, REG, Calofs>;
impl<'a, REG> CalofsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No request for offset calibration has been made"]
    #[inline(always)]
    pub fn no_active_offset_calibration_request(self) -> &'a mut crate::W<REG> {
        self.variant(Calofs::NoActiveOffsetCalibrationRequest)
    }
    #[doc = "Request for offset calibration function"]
    #[inline(always)]
    pub fn offset_calibration_request_pending(self) -> &'a mut crate::W<REG> {
        self.variant(Calofs::OffsetCalibrationRequestPending)
    }
}
#[doc = "High Speed Mode Trim Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Calhs {
    #[doc = "0: No request for high speed mode trim has been made"]
    NoActiveHsTrimRequest = 0,
    #[doc = "1: Request for high speed mode trim has been made"]
    HsTrimRequestPending = 1,
}
impl From<Calhs> for bool {
    #[inline(always)]
    fn from(variant: Calhs) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CALHS` reader - High Speed Mode Trim Request"]
pub type CalhsR = crate::BitReader<Calhs>;
impl CalhsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Calhs {
        match self.bits {
            false => Calhs::NoActiveHsTrimRequest,
            true => Calhs::HsTrimRequestPending,
        }
    }
    #[doc = "No request for high speed mode trim has been made"]
    #[inline(always)]
    pub fn is_no_active_hs_trim_request(&self) -> bool {
        *self == Calhs::NoActiveHsTrimRequest
    }
    #[doc = "Request for high speed mode trim has been made"]
    #[inline(always)]
    pub fn is_hs_trim_request_pending(&self) -> bool {
        *self == Calhs::HsTrimRequestPending
    }
}
#[doc = "Field `CALHS` writer - High Speed Mode Trim Request"]
pub type CalhsW<'a, REG> = crate::BitWriter<'a, REG, Calhs>;
impl<'a, REG> CalhsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No request for high speed mode trim has been made"]
    #[inline(always)]
    pub fn no_active_hs_trim_request(self) -> &'a mut crate::W<REG> {
        self.variant(Calhs::NoActiveHsTrimRequest)
    }
    #[doc = "Request for high speed mode trim has been made"]
    #[inline(always)]
    pub fn hs_trim_request_pending(self) -> &'a mut crate::W<REG> {
        self.variant(Calhs::HsTrimRequestPending)
    }
}
#[doc = "Reset FIFO 0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rstfifo0 {
    #[doc = "0: No effect."]
    NoAction = 0,
    #[doc = "1: FIFO 0 is reset."]
    TriggerReset = 1,
}
impl From<Rstfifo0> for bool {
    #[inline(always)]
    fn from(variant: Rstfifo0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RSTFIFO0` reader - Reset FIFO 0"]
pub type Rstfifo0R = crate::BitReader<Rstfifo0>;
impl Rstfifo0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rstfifo0 {
        match self.bits {
            false => Rstfifo0::NoAction,
            true => Rstfifo0::TriggerReset,
        }
    }
    #[doc = "No effect."]
    #[inline(always)]
    pub fn is_no_action(&self) -> bool {
        *self == Rstfifo0::NoAction
    }
    #[doc = "FIFO 0 is reset."]
    #[inline(always)]
    pub fn is_trigger_reset(&self) -> bool {
        *self == Rstfifo0::TriggerReset
    }
}
#[doc = "Field `RSTFIFO0` writer - Reset FIFO 0"]
pub type Rstfifo0W<'a, REG> = crate::BitWriter<'a, REG, Rstfifo0>;
impl<'a, REG> Rstfifo0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect."]
    #[inline(always)]
    pub fn no_action(self) -> &'a mut crate::W<REG> {
        self.variant(Rstfifo0::NoAction)
    }
    #[doc = "FIFO 0 is reset."]
    #[inline(always)]
    pub fn trigger_reset(self) -> &'a mut crate::W<REG> {
        self.variant(Rstfifo0::TriggerReset)
    }
}
#[doc = "Auto-Calibration Averages\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum CalAvgs {
    #[doc = "0: Single conversion."]
    NoAverage = 0,
    #[doc = "1: 2 conversions averaged."]
    Average2 = 1,
    #[doc = "2: 4 conversions averaged."]
    Average4 = 2,
    #[doc = "3: 8 conversions averaged."]
    Average8 = 3,
    #[doc = "4: 16 conversions averaged."]
    Average16 = 4,
    #[doc = "5: 32 conversions averaged."]
    Average32 = 5,
    #[doc = "6: 64 conversions averaged."]
    Average64 = 6,
    #[doc = "7: 128 conversions averaged."]
    Average128 = 7,
    #[doc = "8: 256 conversions averaged."]
    Average256 = 8,
    #[doc = "9: 512 conversions averaged."]
    Average512 = 9,
    #[doc = "10: 1024 conversions averaged."]
    Average1024 = 10,
}
impl From<CalAvgs> for u8 {
    #[inline(always)]
    fn from(variant: CalAvgs) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for CalAvgs {
    type Ux = u8;
}
impl crate::IsEnum for CalAvgs {}
#[doc = "Field `CAL_AVGS` reader - Auto-Calibration Averages"]
pub type CalAvgsR = crate::FieldReader<CalAvgs>;
impl CalAvgsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<CalAvgs> {
        match self.bits {
            0 => Some(CalAvgs::NoAverage),
            1 => Some(CalAvgs::Average2),
            2 => Some(CalAvgs::Average4),
            3 => Some(CalAvgs::Average8),
            4 => Some(CalAvgs::Average16),
            5 => Some(CalAvgs::Average32),
            6 => Some(CalAvgs::Average64),
            7 => Some(CalAvgs::Average128),
            8 => Some(CalAvgs::Average256),
            9 => Some(CalAvgs::Average512),
            10 => Some(CalAvgs::Average1024),
            _ => None,
        }
    }
    #[doc = "Single conversion."]
    #[inline(always)]
    pub fn is_no_average(&self) -> bool {
        *self == CalAvgs::NoAverage
    }
    #[doc = "2 conversions averaged."]
    #[inline(always)]
    pub fn is_average_2(&self) -> bool {
        *self == CalAvgs::Average2
    }
    #[doc = "4 conversions averaged."]
    #[inline(always)]
    pub fn is_average_4(&self) -> bool {
        *self == CalAvgs::Average4
    }
    #[doc = "8 conversions averaged."]
    #[inline(always)]
    pub fn is_average_8(&self) -> bool {
        *self == CalAvgs::Average8
    }
    #[doc = "16 conversions averaged."]
    #[inline(always)]
    pub fn is_average_16(&self) -> bool {
        *self == CalAvgs::Average16
    }
    #[doc = "32 conversions averaged."]
    #[inline(always)]
    pub fn is_average_32(&self) -> bool {
        *self == CalAvgs::Average32
    }
    #[doc = "64 conversions averaged."]
    #[inline(always)]
    pub fn is_average_64(&self) -> bool {
        *self == CalAvgs::Average64
    }
    #[doc = "128 conversions averaged."]
    #[inline(always)]
    pub fn is_average_128(&self) -> bool {
        *self == CalAvgs::Average128
    }
    #[doc = "256 conversions averaged."]
    #[inline(always)]
    pub fn is_average_256(&self) -> bool {
        *self == CalAvgs::Average256
    }
    #[doc = "512 conversions averaged."]
    #[inline(always)]
    pub fn is_average_512(&self) -> bool {
        *self == CalAvgs::Average512
    }
    #[doc = "1024 conversions averaged."]
    #[inline(always)]
    pub fn is_average_1024(&self) -> bool {
        *self == CalAvgs::Average1024
    }
}
#[doc = "Field `CAL_AVGS` writer - Auto-Calibration Averages"]
pub type CalAvgsW<'a, REG> = crate::FieldWriter<'a, REG, 4, CalAvgs>;
impl<'a, REG> CalAvgsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Single conversion."]
    #[inline(always)]
    pub fn no_average(self) -> &'a mut crate::W<REG> {
        self.variant(CalAvgs::NoAverage)
    }
    #[doc = "2 conversions averaged."]
    #[inline(always)]
    pub fn average_2(self) -> &'a mut crate::W<REG> {
        self.variant(CalAvgs::Average2)
    }
    #[doc = "4 conversions averaged."]
    #[inline(always)]
    pub fn average_4(self) -> &'a mut crate::W<REG> {
        self.variant(CalAvgs::Average4)
    }
    #[doc = "8 conversions averaged."]
    #[inline(always)]
    pub fn average_8(self) -> &'a mut crate::W<REG> {
        self.variant(CalAvgs::Average8)
    }
    #[doc = "16 conversions averaged."]
    #[inline(always)]
    pub fn average_16(self) -> &'a mut crate::W<REG> {
        self.variant(CalAvgs::Average16)
    }
    #[doc = "32 conversions averaged."]
    #[inline(always)]
    pub fn average_32(self) -> &'a mut crate::W<REG> {
        self.variant(CalAvgs::Average32)
    }
    #[doc = "64 conversions averaged."]
    #[inline(always)]
    pub fn average_64(self) -> &'a mut crate::W<REG> {
        self.variant(CalAvgs::Average64)
    }
    #[doc = "128 conversions averaged."]
    #[inline(always)]
    pub fn average_128(self) -> &'a mut crate::W<REG> {
        self.variant(CalAvgs::Average128)
    }
    #[doc = "256 conversions averaged."]
    #[inline(always)]
    pub fn average_256(self) -> &'a mut crate::W<REG> {
        self.variant(CalAvgs::Average256)
    }
    #[doc = "512 conversions averaged."]
    #[inline(always)]
    pub fn average_512(self) -> &'a mut crate::W<REG> {
        self.variant(CalAvgs::Average512)
    }
    #[doc = "1024 conversions averaged."]
    #[inline(always)]
    pub fn average_1024(self) -> &'a mut crate::W<REG> {
        self.variant(CalAvgs::Average1024)
    }
}
impl R {
    #[doc = "Bit 0 - ADC Enable"]
    #[inline(always)]
    pub fn adcen(&self) -> AdcenR {
        AdcenR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Software Reset"]
    #[inline(always)]
    pub fn rst(&self) -> RstR {
        RstR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Doze Enable"]
    #[inline(always)]
    pub fn dozen(&self) -> DozenR {
        DozenR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Auto-Calibration Request"]
    #[inline(always)]
    pub fn cal_req(&self) -> CalReqR {
        CalReqR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Offset Calibration Request"]
    #[inline(always)]
    pub fn calofs(&self) -> CalofsR {
        CalofsR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 6 - High Speed Mode Trim Request"]
    #[inline(always)]
    pub fn calhs(&self) -> CalhsR {
        CalhsR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 8 - Reset FIFO 0"]
    #[inline(always)]
    pub fn rstfifo0(&self) -> Rstfifo0R {
        Rstfifo0R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bits 16:19 - Auto-Calibration Averages"]
    #[inline(always)]
    pub fn cal_avgs(&self) -> CalAvgsR {
        CalAvgsR::new(((self.bits >> 16) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - ADC Enable"]
    #[inline(always)]
    pub fn adcen(&mut self) -> AdcenW<CtrlSpec> {
        AdcenW::new(self, 0)
    }
    #[doc = "Bit 1 - Software Reset"]
    #[inline(always)]
    pub fn rst(&mut self) -> RstW<CtrlSpec> {
        RstW::new(self, 1)
    }
    #[doc = "Bit 2 - Doze Enable"]
    #[inline(always)]
    pub fn dozen(&mut self) -> DozenW<CtrlSpec> {
        DozenW::new(self, 2)
    }
    #[doc = "Bit 3 - Auto-Calibration Request"]
    #[inline(always)]
    pub fn cal_req(&mut self) -> CalReqW<CtrlSpec> {
        CalReqW::new(self, 3)
    }
    #[doc = "Bit 4 - Offset Calibration Request"]
    #[inline(always)]
    pub fn calofs(&mut self) -> CalofsW<CtrlSpec> {
        CalofsW::new(self, 4)
    }
    #[doc = "Bit 6 - High Speed Mode Trim Request"]
    #[inline(always)]
    pub fn calhs(&mut self) -> CalhsW<CtrlSpec> {
        CalhsW::new(self, 6)
    }
    #[doc = "Bit 8 - Reset FIFO 0"]
    #[inline(always)]
    pub fn rstfifo0(&mut self) -> Rstfifo0W<CtrlSpec> {
        Rstfifo0W::new(self, 8)
    }
    #[doc = "Bits 16:19 - Auto-Calibration Averages"]
    #[inline(always)]
    pub fn cal_avgs(&mut self) -> CalAvgsW<CtrlSpec> {
        CalAvgsW::new(self, 16)
    }
}
#[doc = "Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CtrlSpec;
impl crate::RegisterSpec for CtrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ctrl::R`](R) reader structure"]
impl crate::Readable for CtrlSpec {}
#[doc = "`write(|w| ..)` method takes [`ctrl::W`](W) writer structure"]
impl crate::Writable for CtrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CTRL to value 0"]
impl crate::Resettable for CtrlSpec {}

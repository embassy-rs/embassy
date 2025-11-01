#[doc = "Register `TIMCFG[%s]` reader"]
pub type R = crate::R<TimcfgSpec>;
#[doc = "Register `TIMCFG[%s]` writer"]
pub type W = crate::W<TimcfgSpec>;
#[doc = "Timer Start\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tstart {
    #[doc = "0: Disabled"]
    Disable = 0,
    #[doc = "1: Enabled"]
    Enable = 1,
}
impl From<Tstart> for bool {
    #[inline(always)]
    fn from(variant: Tstart) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TSTART` reader - Timer Start"]
pub type TstartR = crate::BitReader<Tstart>;
impl TstartR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tstart {
        match self.bits {
            false => Tstart::Disable,
            true => Tstart::Enable,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tstart::Disable
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tstart::Enable
    }
}
#[doc = "Field `TSTART` writer - Timer Start"]
pub type TstartW<'a, REG> = crate::BitWriter<'a, REG, Tstart>;
impl<'a, REG> TstartW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tstart::Disable)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tstart::Enable)
    }
}
#[doc = "Timer Stop\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Tstop {
    #[doc = "0: Disabled"]
    StopDisable = 0,
    #[doc = "1: Enabled on timer compare"]
    EnableTmrcmp = 1,
    #[doc = "2: Enabled on timer disable"]
    EnableTmrdisable = 2,
    #[doc = "3: Enabled on timer compare and timer disable"]
    EnableTmrCmpDis = 3,
}
impl From<Tstop> for u8 {
    #[inline(always)]
    fn from(variant: Tstop) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Tstop {
    type Ux = u8;
}
impl crate::IsEnum for Tstop {}
#[doc = "Field `TSTOP` reader - Timer Stop"]
pub type TstopR = crate::FieldReader<Tstop>;
impl TstopR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tstop {
        match self.bits {
            0 => Tstop::StopDisable,
            1 => Tstop::EnableTmrcmp,
            2 => Tstop::EnableTmrdisable,
            3 => Tstop::EnableTmrCmpDis,
            _ => unreachable!(),
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_stop_disable(&self) -> bool {
        *self == Tstop::StopDisable
    }
    #[doc = "Enabled on timer compare"]
    #[inline(always)]
    pub fn is_enable_tmrcmp(&self) -> bool {
        *self == Tstop::EnableTmrcmp
    }
    #[doc = "Enabled on timer disable"]
    #[inline(always)]
    pub fn is_enable_tmrdisable(&self) -> bool {
        *self == Tstop::EnableTmrdisable
    }
    #[doc = "Enabled on timer compare and timer disable"]
    #[inline(always)]
    pub fn is_enable_tmr_cmp_dis(&self) -> bool {
        *self == Tstop::EnableTmrCmpDis
    }
}
#[doc = "Field `TSTOP` writer - Timer Stop"]
pub type TstopW<'a, REG> = crate::FieldWriter<'a, REG, 2, Tstop, crate::Safe>;
impl<'a, REG> TstopW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn stop_disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tstop::StopDisable)
    }
    #[doc = "Enabled on timer compare"]
    #[inline(always)]
    pub fn enable_tmrcmp(self) -> &'a mut crate::W<REG> {
        self.variant(Tstop::EnableTmrcmp)
    }
    #[doc = "Enabled on timer disable"]
    #[inline(always)]
    pub fn enable_tmrdisable(self) -> &'a mut crate::W<REG> {
        self.variant(Tstop::EnableTmrdisable)
    }
    #[doc = "Enabled on timer compare and timer disable"]
    #[inline(always)]
    pub fn enable_tmr_cmp_dis(self) -> &'a mut crate::W<REG> {
        self.variant(Tstop::EnableTmrCmpDis)
    }
}
#[doc = "Timer Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Timena {
    #[doc = "0: Timer always enabled"]
    Enable = 0,
    #[doc = "1: Timer enabled on timer n-1 enable"]
    TmrNminus1En = 1,
    #[doc = "2: Timer enabled on trigger high"]
    TmrTrighiEn = 2,
    #[doc = "3: Timer enabled on trigger high and pin high"]
    TmrTrigPinHiEn = 3,
    #[doc = "4: Timer enabled on pin rising edge"]
    TmrPinriseEn = 4,
    #[doc = "5: Timer enabled on pin rising edge and trigger high"]
    TmrPinriseTrighiEn = 5,
    #[doc = "6: Timer enabled on trigger rising edge"]
    TmrTrigriseEn = 6,
    #[doc = "7: Timer enabled on trigger rising or falling edge"]
    TmrTrigedgeEn = 7,
}
impl From<Timena> for u8 {
    #[inline(always)]
    fn from(variant: Timena) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Timena {
    type Ux = u8;
}
impl crate::IsEnum for Timena {}
#[doc = "Field `TIMENA` reader - Timer Enable"]
pub type TimenaR = crate::FieldReader<Timena>;
impl TimenaR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Timena {
        match self.bits {
            0 => Timena::Enable,
            1 => Timena::TmrNminus1En,
            2 => Timena::TmrTrighiEn,
            3 => Timena::TmrTrigPinHiEn,
            4 => Timena::TmrPinriseEn,
            5 => Timena::TmrPinriseTrighiEn,
            6 => Timena::TmrTrigriseEn,
            7 => Timena::TmrTrigedgeEn,
            _ => unreachable!(),
        }
    }
    #[doc = "Timer always enabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Timena::Enable
    }
    #[doc = "Timer enabled on timer n-1 enable"]
    #[inline(always)]
    pub fn is_tmr_nminus1_en(&self) -> bool {
        *self == Timena::TmrNminus1En
    }
    #[doc = "Timer enabled on trigger high"]
    #[inline(always)]
    pub fn is_tmr_trighi_en(&self) -> bool {
        *self == Timena::TmrTrighiEn
    }
    #[doc = "Timer enabled on trigger high and pin high"]
    #[inline(always)]
    pub fn is_tmr_trig_pin_hi_en(&self) -> bool {
        *self == Timena::TmrTrigPinHiEn
    }
    #[doc = "Timer enabled on pin rising edge"]
    #[inline(always)]
    pub fn is_tmr_pinrise_en(&self) -> bool {
        *self == Timena::TmrPinriseEn
    }
    #[doc = "Timer enabled on pin rising edge and trigger high"]
    #[inline(always)]
    pub fn is_tmr_pinrise_trighi_en(&self) -> bool {
        *self == Timena::TmrPinriseTrighiEn
    }
    #[doc = "Timer enabled on trigger rising edge"]
    #[inline(always)]
    pub fn is_tmr_trigrise_en(&self) -> bool {
        *self == Timena::TmrTrigriseEn
    }
    #[doc = "Timer enabled on trigger rising or falling edge"]
    #[inline(always)]
    pub fn is_tmr_trigedge_en(&self) -> bool {
        *self == Timena::TmrTrigedgeEn
    }
}
#[doc = "Field `TIMENA` writer - Timer Enable"]
pub type TimenaW<'a, REG> = crate::FieldWriter<'a, REG, 3, Timena, crate::Safe>;
impl<'a, REG> TimenaW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Timer always enabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Timena::Enable)
    }
    #[doc = "Timer enabled on timer n-1 enable"]
    #[inline(always)]
    pub fn tmr_nminus1_en(self) -> &'a mut crate::W<REG> {
        self.variant(Timena::TmrNminus1En)
    }
    #[doc = "Timer enabled on trigger high"]
    #[inline(always)]
    pub fn tmr_trighi_en(self) -> &'a mut crate::W<REG> {
        self.variant(Timena::TmrTrighiEn)
    }
    #[doc = "Timer enabled on trigger high and pin high"]
    #[inline(always)]
    pub fn tmr_trig_pin_hi_en(self) -> &'a mut crate::W<REG> {
        self.variant(Timena::TmrTrigPinHiEn)
    }
    #[doc = "Timer enabled on pin rising edge"]
    #[inline(always)]
    pub fn tmr_pinrise_en(self) -> &'a mut crate::W<REG> {
        self.variant(Timena::TmrPinriseEn)
    }
    #[doc = "Timer enabled on pin rising edge and trigger high"]
    #[inline(always)]
    pub fn tmr_pinrise_trighi_en(self) -> &'a mut crate::W<REG> {
        self.variant(Timena::TmrPinriseTrighiEn)
    }
    #[doc = "Timer enabled on trigger rising edge"]
    #[inline(always)]
    pub fn tmr_trigrise_en(self) -> &'a mut crate::W<REG> {
        self.variant(Timena::TmrTrigriseEn)
    }
    #[doc = "Timer enabled on trigger rising or falling edge"]
    #[inline(always)]
    pub fn tmr_trigedge_en(self) -> &'a mut crate::W<REG> {
        self.variant(Timena::TmrTrigedgeEn)
    }
}
#[doc = "Timer Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Timdis {
    #[doc = "0: Timer never disabled"]
    Never = 0,
    #[doc = "1: Timer disabled on timer n-1 disable"]
    TmrNminus1 = 1,
    #[doc = "2: Timer disabled on timer compare (upper 8 bits match and decrement)"]
    TmrCmp = 2,
    #[doc = "3: Timer disabled on timer compare (upper 8 bits match and decrement) and trigger low"]
    TmrCmpTriglow = 3,
    #[doc = "4: Timer disabled on pin rising or falling edge"]
    PinEdge = 4,
    #[doc = "5: Timer disabled on pin rising or falling edge provided trigger is high"]
    PinEdgeTrighi = 5,
    #[doc = "6: Timer disabled on trigger falling edge"]
    TrigFalledge = 6,
}
impl From<Timdis> for u8 {
    #[inline(always)]
    fn from(variant: Timdis) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Timdis {
    type Ux = u8;
}
impl crate::IsEnum for Timdis {}
#[doc = "Field `TIMDIS` reader - Timer Disable"]
pub type TimdisR = crate::FieldReader<Timdis>;
impl TimdisR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Timdis> {
        match self.bits {
            0 => Some(Timdis::Never),
            1 => Some(Timdis::TmrNminus1),
            2 => Some(Timdis::TmrCmp),
            3 => Some(Timdis::TmrCmpTriglow),
            4 => Some(Timdis::PinEdge),
            5 => Some(Timdis::PinEdgeTrighi),
            6 => Some(Timdis::TrigFalledge),
            _ => None,
        }
    }
    #[doc = "Timer never disabled"]
    #[inline(always)]
    pub fn is_never(&self) -> bool {
        *self == Timdis::Never
    }
    #[doc = "Timer disabled on timer n-1 disable"]
    #[inline(always)]
    pub fn is_tmr_nminus1(&self) -> bool {
        *self == Timdis::TmrNminus1
    }
    #[doc = "Timer disabled on timer compare (upper 8 bits match and decrement)"]
    #[inline(always)]
    pub fn is_tmr_cmp(&self) -> bool {
        *self == Timdis::TmrCmp
    }
    #[doc = "Timer disabled on timer compare (upper 8 bits match and decrement) and trigger low"]
    #[inline(always)]
    pub fn is_tmr_cmp_triglow(&self) -> bool {
        *self == Timdis::TmrCmpTriglow
    }
    #[doc = "Timer disabled on pin rising or falling edge"]
    #[inline(always)]
    pub fn is_pin_edge(&self) -> bool {
        *self == Timdis::PinEdge
    }
    #[doc = "Timer disabled on pin rising or falling edge provided trigger is high"]
    #[inline(always)]
    pub fn is_pin_edge_trighi(&self) -> bool {
        *self == Timdis::PinEdgeTrighi
    }
    #[doc = "Timer disabled on trigger falling edge"]
    #[inline(always)]
    pub fn is_trig_falledge(&self) -> bool {
        *self == Timdis::TrigFalledge
    }
}
#[doc = "Field `TIMDIS` writer - Timer Disable"]
pub type TimdisW<'a, REG> = crate::FieldWriter<'a, REG, 3, Timdis>;
impl<'a, REG> TimdisW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Timer never disabled"]
    #[inline(always)]
    pub fn never(self) -> &'a mut crate::W<REG> {
        self.variant(Timdis::Never)
    }
    #[doc = "Timer disabled on timer n-1 disable"]
    #[inline(always)]
    pub fn tmr_nminus1(self) -> &'a mut crate::W<REG> {
        self.variant(Timdis::TmrNminus1)
    }
    #[doc = "Timer disabled on timer compare (upper 8 bits match and decrement)"]
    #[inline(always)]
    pub fn tmr_cmp(self) -> &'a mut crate::W<REG> {
        self.variant(Timdis::TmrCmp)
    }
    #[doc = "Timer disabled on timer compare (upper 8 bits match and decrement) and trigger low"]
    #[inline(always)]
    pub fn tmr_cmp_triglow(self) -> &'a mut crate::W<REG> {
        self.variant(Timdis::TmrCmpTriglow)
    }
    #[doc = "Timer disabled on pin rising or falling edge"]
    #[inline(always)]
    pub fn pin_edge(self) -> &'a mut crate::W<REG> {
        self.variant(Timdis::PinEdge)
    }
    #[doc = "Timer disabled on pin rising or falling edge provided trigger is high"]
    #[inline(always)]
    pub fn pin_edge_trighi(self) -> &'a mut crate::W<REG> {
        self.variant(Timdis::PinEdgeTrighi)
    }
    #[doc = "Timer disabled on trigger falling edge"]
    #[inline(always)]
    pub fn trig_falledge(self) -> &'a mut crate::W<REG> {
        self.variant(Timdis::TrigFalledge)
    }
}
#[doc = "Timer Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Timrst {
    #[doc = "0: Never reset timer"]
    Never = 0,
    #[doc = "1: Timer reset on timer output high."]
    TmrOutHi = 1,
    #[doc = "2: Timer reset on timer pin equal to timer output"]
    PinEqTmrOut = 2,
    #[doc = "3: Timer reset on timer trigger equal to timer output"]
    TrigEqTmrOut = 3,
    #[doc = "4: Timer reset on timer pin rising edge"]
    PinRiseEdge = 4,
    #[doc = "6: Timer reset on trigger rising edge"]
    TrigRiseEdge = 6,
    #[doc = "7: Timer reset on trigger rising or falling edge"]
    TrigEdge = 7,
}
impl From<Timrst> for u8 {
    #[inline(always)]
    fn from(variant: Timrst) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Timrst {
    type Ux = u8;
}
impl crate::IsEnum for Timrst {}
#[doc = "Field `TIMRST` reader - Timer Reset"]
pub type TimrstR = crate::FieldReader<Timrst>;
impl TimrstR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Timrst> {
        match self.bits {
            0 => Some(Timrst::Never),
            1 => Some(Timrst::TmrOutHi),
            2 => Some(Timrst::PinEqTmrOut),
            3 => Some(Timrst::TrigEqTmrOut),
            4 => Some(Timrst::PinRiseEdge),
            6 => Some(Timrst::TrigRiseEdge),
            7 => Some(Timrst::TrigEdge),
            _ => None,
        }
    }
    #[doc = "Never reset timer"]
    #[inline(always)]
    pub fn is_never(&self) -> bool {
        *self == Timrst::Never
    }
    #[doc = "Timer reset on timer output high."]
    #[inline(always)]
    pub fn is_tmr_out_hi(&self) -> bool {
        *self == Timrst::TmrOutHi
    }
    #[doc = "Timer reset on timer pin equal to timer output"]
    #[inline(always)]
    pub fn is_pin_eq_tmr_out(&self) -> bool {
        *self == Timrst::PinEqTmrOut
    }
    #[doc = "Timer reset on timer trigger equal to timer output"]
    #[inline(always)]
    pub fn is_trig_eq_tmr_out(&self) -> bool {
        *self == Timrst::TrigEqTmrOut
    }
    #[doc = "Timer reset on timer pin rising edge"]
    #[inline(always)]
    pub fn is_pin_rise_edge(&self) -> bool {
        *self == Timrst::PinRiseEdge
    }
    #[doc = "Timer reset on trigger rising edge"]
    #[inline(always)]
    pub fn is_trig_rise_edge(&self) -> bool {
        *self == Timrst::TrigRiseEdge
    }
    #[doc = "Timer reset on trigger rising or falling edge"]
    #[inline(always)]
    pub fn is_trig_edge(&self) -> bool {
        *self == Timrst::TrigEdge
    }
}
#[doc = "Field `TIMRST` writer - Timer Reset"]
pub type TimrstW<'a, REG> = crate::FieldWriter<'a, REG, 3, Timrst>;
impl<'a, REG> TimrstW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Never reset timer"]
    #[inline(always)]
    pub fn never(self) -> &'a mut crate::W<REG> {
        self.variant(Timrst::Never)
    }
    #[doc = "Timer reset on timer output high."]
    #[inline(always)]
    pub fn tmr_out_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Timrst::TmrOutHi)
    }
    #[doc = "Timer reset on timer pin equal to timer output"]
    #[inline(always)]
    pub fn pin_eq_tmr_out(self) -> &'a mut crate::W<REG> {
        self.variant(Timrst::PinEqTmrOut)
    }
    #[doc = "Timer reset on timer trigger equal to timer output"]
    #[inline(always)]
    pub fn trig_eq_tmr_out(self) -> &'a mut crate::W<REG> {
        self.variant(Timrst::TrigEqTmrOut)
    }
    #[doc = "Timer reset on timer pin rising edge"]
    #[inline(always)]
    pub fn pin_rise_edge(self) -> &'a mut crate::W<REG> {
        self.variant(Timrst::PinRiseEdge)
    }
    #[doc = "Timer reset on trigger rising edge"]
    #[inline(always)]
    pub fn trig_rise_edge(self) -> &'a mut crate::W<REG> {
        self.variant(Timrst::TrigRiseEdge)
    }
    #[doc = "Timer reset on trigger rising or falling edge"]
    #[inline(always)]
    pub fn trig_edge(self) -> &'a mut crate::W<REG> {
        self.variant(Timrst::TrigEdge)
    }
}
#[doc = "Timer Decrement\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Timdec {
    #[doc = "0: Decrement counter on FLEXIO clock; shift clock equals timer output"]
    FlexioClkShiftclkTmrOut = 0,
    #[doc = "1: Decrement counter on trigger input (both edges); shift clock equals timer output"]
    TrigEdgeShiftclkTmrOut = 1,
    #[doc = "2: Decrement counter on pin input (both edges); shift clock equals pin input"]
    PinEdgeShiftclkTmrOut = 2,
    #[doc = "3: Decrement counter on trigger input (both edges); shift clock equals trigger input"]
    TrigEdgeShiftclkTrigIn = 3,
    #[doc = "4: Decrement counter on FLEXIO clock divided by 16; shift clock equals timer output"]
    FlexioClkDiv16ShiftclkTmrOut = 4,
    #[doc = "5: Decrement counter on FLEXIO clock divided by 256; shift clock equals timer output"]
    FlexioClkDiv256ShiftclkTmrOut = 5,
    #[doc = "6: Decrement counter on pin input (rising edge); shift clock equals pin input"]
    PinRiseShiftclkPinIn = 6,
    #[doc = "7: Decrement counter on trigger input (rising edge); shift clock equals trigger input"]
    TrigRiseShiftclkTrigIn = 7,
}
impl From<Timdec> for u8 {
    #[inline(always)]
    fn from(variant: Timdec) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Timdec {
    type Ux = u8;
}
impl crate::IsEnum for Timdec {}
#[doc = "Field `TIMDEC` reader - Timer Decrement"]
pub type TimdecR = crate::FieldReader<Timdec>;
impl TimdecR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Timdec {
        match self.bits {
            0 => Timdec::FlexioClkShiftclkTmrOut,
            1 => Timdec::TrigEdgeShiftclkTmrOut,
            2 => Timdec::PinEdgeShiftclkTmrOut,
            3 => Timdec::TrigEdgeShiftclkTrigIn,
            4 => Timdec::FlexioClkDiv16ShiftclkTmrOut,
            5 => Timdec::FlexioClkDiv256ShiftclkTmrOut,
            6 => Timdec::PinRiseShiftclkPinIn,
            7 => Timdec::TrigRiseShiftclkTrigIn,
            _ => unreachable!(),
        }
    }
    #[doc = "Decrement counter on FLEXIO clock; shift clock equals timer output"]
    #[inline(always)]
    pub fn is_flexio_clk_shiftclk_tmr_out(&self) -> bool {
        *self == Timdec::FlexioClkShiftclkTmrOut
    }
    #[doc = "Decrement counter on trigger input (both edges); shift clock equals timer output"]
    #[inline(always)]
    pub fn is_trig_edge_shiftclk_tmr_out(&self) -> bool {
        *self == Timdec::TrigEdgeShiftclkTmrOut
    }
    #[doc = "Decrement counter on pin input (both edges); shift clock equals pin input"]
    #[inline(always)]
    pub fn is_pin_edge_shiftclk_tmr_out(&self) -> bool {
        *self == Timdec::PinEdgeShiftclkTmrOut
    }
    #[doc = "Decrement counter on trigger input (both edges); shift clock equals trigger input"]
    #[inline(always)]
    pub fn is_trig_edge_shiftclk_trig_in(&self) -> bool {
        *self == Timdec::TrigEdgeShiftclkTrigIn
    }
    #[doc = "Decrement counter on FLEXIO clock divided by 16; shift clock equals timer output"]
    #[inline(always)]
    pub fn is_flexio_clk_div16_shiftclk_tmr_out(&self) -> bool {
        *self == Timdec::FlexioClkDiv16ShiftclkTmrOut
    }
    #[doc = "Decrement counter on FLEXIO clock divided by 256; shift clock equals timer output"]
    #[inline(always)]
    pub fn is_flexio_clk_div256_shiftclk_tmr_out(&self) -> bool {
        *self == Timdec::FlexioClkDiv256ShiftclkTmrOut
    }
    #[doc = "Decrement counter on pin input (rising edge); shift clock equals pin input"]
    #[inline(always)]
    pub fn is_pin_rise_shiftclk_pin_in(&self) -> bool {
        *self == Timdec::PinRiseShiftclkPinIn
    }
    #[doc = "Decrement counter on trigger input (rising edge); shift clock equals trigger input"]
    #[inline(always)]
    pub fn is_trig_rise_shiftclk_trig_in(&self) -> bool {
        *self == Timdec::TrigRiseShiftclkTrigIn
    }
}
#[doc = "Field `TIMDEC` writer - Timer Decrement"]
pub type TimdecW<'a, REG> = crate::FieldWriter<'a, REG, 3, Timdec, crate::Safe>;
impl<'a, REG> TimdecW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Decrement counter on FLEXIO clock; shift clock equals timer output"]
    #[inline(always)]
    pub fn flexio_clk_shiftclk_tmr_out(self) -> &'a mut crate::W<REG> {
        self.variant(Timdec::FlexioClkShiftclkTmrOut)
    }
    #[doc = "Decrement counter on trigger input (both edges); shift clock equals timer output"]
    #[inline(always)]
    pub fn trig_edge_shiftclk_tmr_out(self) -> &'a mut crate::W<REG> {
        self.variant(Timdec::TrigEdgeShiftclkTmrOut)
    }
    #[doc = "Decrement counter on pin input (both edges); shift clock equals pin input"]
    #[inline(always)]
    pub fn pin_edge_shiftclk_tmr_out(self) -> &'a mut crate::W<REG> {
        self.variant(Timdec::PinEdgeShiftclkTmrOut)
    }
    #[doc = "Decrement counter on trigger input (both edges); shift clock equals trigger input"]
    #[inline(always)]
    pub fn trig_edge_shiftclk_trig_in(self) -> &'a mut crate::W<REG> {
        self.variant(Timdec::TrigEdgeShiftclkTrigIn)
    }
    #[doc = "Decrement counter on FLEXIO clock divided by 16; shift clock equals timer output"]
    #[inline(always)]
    pub fn flexio_clk_div16_shiftclk_tmr_out(self) -> &'a mut crate::W<REG> {
        self.variant(Timdec::FlexioClkDiv16ShiftclkTmrOut)
    }
    #[doc = "Decrement counter on FLEXIO clock divided by 256; shift clock equals timer output"]
    #[inline(always)]
    pub fn flexio_clk_div256_shiftclk_tmr_out(self) -> &'a mut crate::W<REG> {
        self.variant(Timdec::FlexioClkDiv256ShiftclkTmrOut)
    }
    #[doc = "Decrement counter on pin input (rising edge); shift clock equals pin input"]
    #[inline(always)]
    pub fn pin_rise_shiftclk_pin_in(self) -> &'a mut crate::W<REG> {
        self.variant(Timdec::PinRiseShiftclkPinIn)
    }
    #[doc = "Decrement counter on trigger input (rising edge); shift clock equals trigger input"]
    #[inline(always)]
    pub fn trig_rise_shiftclk_trig_in(self) -> &'a mut crate::W<REG> {
        self.variant(Timdec::TrigRiseShiftclkTrigIn)
    }
}
#[doc = "Timer Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Timout {
    #[doc = "0: Logic one when enabled; not affected by timer reset"]
    One = 0,
    #[doc = "1: Logic zero when enabled; not affected by timer reset"]
    Zero = 1,
    #[doc = "2: Logic one when enabled and on timer reset"]
    OneTmrreset = 2,
    #[doc = "3: Logic zero when enabled and on timer reset"]
    ZeroTmrreset = 3,
}
impl From<Timout> for u8 {
    #[inline(always)]
    fn from(variant: Timout) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Timout {
    type Ux = u8;
}
impl crate::IsEnum for Timout {}
#[doc = "Field `TIMOUT` reader - Timer Output"]
pub type TimoutR = crate::FieldReader<Timout>;
impl TimoutR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Timout {
        match self.bits {
            0 => Timout::One,
            1 => Timout::Zero,
            2 => Timout::OneTmrreset,
            3 => Timout::ZeroTmrreset,
            _ => unreachable!(),
        }
    }
    #[doc = "Logic one when enabled; not affected by timer reset"]
    #[inline(always)]
    pub fn is_one(&self) -> bool {
        *self == Timout::One
    }
    #[doc = "Logic zero when enabled; not affected by timer reset"]
    #[inline(always)]
    pub fn is_zero(&self) -> bool {
        *self == Timout::Zero
    }
    #[doc = "Logic one when enabled and on timer reset"]
    #[inline(always)]
    pub fn is_one_tmrreset(&self) -> bool {
        *self == Timout::OneTmrreset
    }
    #[doc = "Logic zero when enabled and on timer reset"]
    #[inline(always)]
    pub fn is_zero_tmrreset(&self) -> bool {
        *self == Timout::ZeroTmrreset
    }
}
#[doc = "Field `TIMOUT` writer - Timer Output"]
pub type TimoutW<'a, REG> = crate::FieldWriter<'a, REG, 2, Timout, crate::Safe>;
impl<'a, REG> TimoutW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Logic one when enabled; not affected by timer reset"]
    #[inline(always)]
    pub fn one(self) -> &'a mut crate::W<REG> {
        self.variant(Timout::One)
    }
    #[doc = "Logic zero when enabled; not affected by timer reset"]
    #[inline(always)]
    pub fn zero(self) -> &'a mut crate::W<REG> {
        self.variant(Timout::Zero)
    }
    #[doc = "Logic one when enabled and on timer reset"]
    #[inline(always)]
    pub fn one_tmrreset(self) -> &'a mut crate::W<REG> {
        self.variant(Timout::OneTmrreset)
    }
    #[doc = "Logic zero when enabled and on timer reset"]
    #[inline(always)]
    pub fn zero_tmrreset(self) -> &'a mut crate::W<REG> {
        self.variant(Timout::ZeroTmrreset)
    }
}
impl R {
    #[doc = "Bit 1 - Timer Start"]
    #[inline(always)]
    pub fn tstart(&self) -> TstartR {
        TstartR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bits 4:5 - Timer Stop"]
    #[inline(always)]
    pub fn tstop(&self) -> TstopR {
        TstopR::new(((self.bits >> 4) & 3) as u8)
    }
    #[doc = "Bits 8:10 - Timer Enable"]
    #[inline(always)]
    pub fn timena(&self) -> TimenaR {
        TimenaR::new(((self.bits >> 8) & 7) as u8)
    }
    #[doc = "Bits 12:14 - Timer Disable"]
    #[inline(always)]
    pub fn timdis(&self) -> TimdisR {
        TimdisR::new(((self.bits >> 12) & 7) as u8)
    }
    #[doc = "Bits 16:18 - Timer Reset"]
    #[inline(always)]
    pub fn timrst(&self) -> TimrstR {
        TimrstR::new(((self.bits >> 16) & 7) as u8)
    }
    #[doc = "Bits 20:22 - Timer Decrement"]
    #[inline(always)]
    pub fn timdec(&self) -> TimdecR {
        TimdecR::new(((self.bits >> 20) & 7) as u8)
    }
    #[doc = "Bits 24:25 - Timer Output"]
    #[inline(always)]
    pub fn timout(&self) -> TimoutR {
        TimoutR::new(((self.bits >> 24) & 3) as u8)
    }
}
impl W {
    #[doc = "Bit 1 - Timer Start"]
    #[inline(always)]
    pub fn tstart(&mut self) -> TstartW<TimcfgSpec> {
        TstartW::new(self, 1)
    }
    #[doc = "Bits 4:5 - Timer Stop"]
    #[inline(always)]
    pub fn tstop(&mut self) -> TstopW<TimcfgSpec> {
        TstopW::new(self, 4)
    }
    #[doc = "Bits 8:10 - Timer Enable"]
    #[inline(always)]
    pub fn timena(&mut self) -> TimenaW<TimcfgSpec> {
        TimenaW::new(self, 8)
    }
    #[doc = "Bits 12:14 - Timer Disable"]
    #[inline(always)]
    pub fn timdis(&mut self) -> TimdisW<TimcfgSpec> {
        TimdisW::new(self, 12)
    }
    #[doc = "Bits 16:18 - Timer Reset"]
    #[inline(always)]
    pub fn timrst(&mut self) -> TimrstW<TimcfgSpec> {
        TimrstW::new(self, 16)
    }
    #[doc = "Bits 20:22 - Timer Decrement"]
    #[inline(always)]
    pub fn timdec(&mut self) -> TimdecW<TimcfgSpec> {
        TimdecW::new(self, 20)
    }
    #[doc = "Bits 24:25 - Timer Output"]
    #[inline(always)]
    pub fn timout(&mut self) -> TimoutW<TimcfgSpec> {
        TimoutW::new(self, 24)
    }
}
#[doc = "Timer Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`timcfg::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`timcfg::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TimcfgSpec;
impl crate::RegisterSpec for TimcfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`timcfg::R`](R) reader structure"]
impl crate::Readable for TimcfgSpec {}
#[doc = "`write(|w| ..)` method takes [`timcfg::W`](W) writer structure"]
impl crate::Writable for TimcfgSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TIMCFG[%s] to value 0"]
impl crate::Resettable for TimcfgSpec {}

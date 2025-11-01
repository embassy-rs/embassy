#[doc = "Register `CTRL1` reader"]
pub type R = crate::R<Ctrl1Spec>;
#[doc = "Register `CTRL1` writer"]
pub type W = crate::W<Ctrl1Spec>;
#[doc = "Field `PROPSEG` reader - Propagation Segment"]
pub type PropsegR = crate::FieldReader;
#[doc = "Field `PROPSEG` writer - Propagation Segment"]
pub type PropsegW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Listen-Only Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lom {
    #[doc = "0: Listen-Only mode is deactivated."]
    ListenOnlyModeDisabled = 0,
    #[doc = "1: FlexCAN module operates in Listen-Only mode."]
    ListenOnlyModeEnabled = 1,
}
impl From<Lom> for bool {
    #[inline(always)]
    fn from(variant: Lom) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LOM` reader - Listen-Only Mode"]
pub type LomR = crate::BitReader<Lom>;
impl LomR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lom {
        match self.bits {
            false => Lom::ListenOnlyModeDisabled,
            true => Lom::ListenOnlyModeEnabled,
        }
    }
    #[doc = "Listen-Only mode is deactivated."]
    #[inline(always)]
    pub fn is_listen_only_mode_disabled(&self) -> bool {
        *self == Lom::ListenOnlyModeDisabled
    }
    #[doc = "FlexCAN module operates in Listen-Only mode."]
    #[inline(always)]
    pub fn is_listen_only_mode_enabled(&self) -> bool {
        *self == Lom::ListenOnlyModeEnabled
    }
}
#[doc = "Field `LOM` writer - Listen-Only Mode"]
pub type LomW<'a, REG> = crate::BitWriter<'a, REG, Lom>;
impl<'a, REG> LomW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Listen-Only mode is deactivated."]
    #[inline(always)]
    pub fn listen_only_mode_disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lom::ListenOnlyModeDisabled)
    }
    #[doc = "FlexCAN module operates in Listen-Only mode."]
    #[inline(always)]
    pub fn listen_only_mode_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lom::ListenOnlyModeEnabled)
    }
}
#[doc = "Lowest Buffer Transmitted First\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lbuf {
    #[doc = "0: Buffer with highest priority is transmitted first."]
    HighestBufferFirst = 0,
    #[doc = "1: Lowest number buffer is transmitted first."]
    LowestBufferFirst = 1,
}
impl From<Lbuf> for bool {
    #[inline(always)]
    fn from(variant: Lbuf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LBUF` reader - Lowest Buffer Transmitted First"]
pub type LbufR = crate::BitReader<Lbuf>;
impl LbufR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lbuf {
        match self.bits {
            false => Lbuf::HighestBufferFirst,
            true => Lbuf::LowestBufferFirst,
        }
    }
    #[doc = "Buffer with highest priority is transmitted first."]
    #[inline(always)]
    pub fn is_highest_buffer_first(&self) -> bool {
        *self == Lbuf::HighestBufferFirst
    }
    #[doc = "Lowest number buffer is transmitted first."]
    #[inline(always)]
    pub fn is_lowest_buffer_first(&self) -> bool {
        *self == Lbuf::LowestBufferFirst
    }
}
#[doc = "Field `LBUF` writer - Lowest Buffer Transmitted First"]
pub type LbufW<'a, REG> = crate::BitWriter<'a, REG, Lbuf>;
impl<'a, REG> LbufW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Buffer with highest priority is transmitted first."]
    #[inline(always)]
    pub fn highest_buffer_first(self) -> &'a mut crate::W<REG> {
        self.variant(Lbuf::HighestBufferFirst)
    }
    #[doc = "Lowest number buffer is transmitted first."]
    #[inline(always)]
    pub fn lowest_buffer_first(self) -> &'a mut crate::W<REG> {
        self.variant(Lbuf::LowestBufferFirst)
    }
}
#[doc = "Timer Sync\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tsyn {
    #[doc = "0: Disable"]
    TimerSyncDisabled = 0,
    #[doc = "1: Enable"]
    TimerSyncEnabled = 1,
}
impl From<Tsyn> for bool {
    #[inline(always)]
    fn from(variant: Tsyn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TSYN` reader - Timer Sync"]
pub type TsynR = crate::BitReader<Tsyn>;
impl TsynR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tsyn {
        match self.bits {
            false => Tsyn::TimerSyncDisabled,
            true => Tsyn::TimerSyncEnabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_timer_sync_disabled(&self) -> bool {
        *self == Tsyn::TimerSyncDisabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_timer_sync_enabled(&self) -> bool {
        *self == Tsyn::TimerSyncEnabled
    }
}
#[doc = "Field `TSYN` writer - Timer Sync"]
pub type TsynW<'a, REG> = crate::BitWriter<'a, REG, Tsyn>;
impl<'a, REG> TsynW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn timer_sync_disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Tsyn::TimerSyncDisabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn timer_sync_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Tsyn::TimerSyncEnabled)
    }
}
#[doc = "Bus Off Recovery\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boffrec {
    #[doc = "0: Enabled"]
    AutoRecoverEnabled = 0,
    #[doc = "1: Disabled"]
    AutoRecoverDisabled = 1,
}
impl From<Boffrec> for bool {
    #[inline(always)]
    fn from(variant: Boffrec) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BOFFREC` reader - Bus Off Recovery"]
pub type BoffrecR = crate::BitReader<Boffrec>;
impl BoffrecR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Boffrec {
        match self.bits {
            false => Boffrec::AutoRecoverEnabled,
            true => Boffrec::AutoRecoverDisabled,
        }
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_auto_recover_enabled(&self) -> bool {
        *self == Boffrec::AutoRecoverEnabled
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_auto_recover_disabled(&self) -> bool {
        *self == Boffrec::AutoRecoverDisabled
    }
}
#[doc = "Field `BOFFREC` writer - Bus Off Recovery"]
pub type BoffrecW<'a, REG> = crate::BitWriter<'a, REG, Boffrec>;
impl<'a, REG> BoffrecW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn auto_recover_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Boffrec::AutoRecoverEnabled)
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn auto_recover_disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Boffrec::AutoRecoverDisabled)
    }
}
#[doc = "CAN Bit Sampling\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Smp {
    #[doc = "0: One sample is used to determine the bit value."]
    OneSample = 0,
    #[doc = "1: Three samples are used to determine the value of the received bit: the regular one (sample point) and two preceding samples. A majority rule is used."]
    ThreeSample = 1,
}
impl From<Smp> for bool {
    #[inline(always)]
    fn from(variant: Smp) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SMP` reader - CAN Bit Sampling"]
pub type SmpR = crate::BitReader<Smp>;
impl SmpR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Smp {
        match self.bits {
            false => Smp::OneSample,
            true => Smp::ThreeSample,
        }
    }
    #[doc = "One sample is used to determine the bit value."]
    #[inline(always)]
    pub fn is_one_sample(&self) -> bool {
        *self == Smp::OneSample
    }
    #[doc = "Three samples are used to determine the value of the received bit: the regular one (sample point) and two preceding samples. A majority rule is used."]
    #[inline(always)]
    pub fn is_three_sample(&self) -> bool {
        *self == Smp::ThreeSample
    }
}
#[doc = "Field `SMP` writer - CAN Bit Sampling"]
pub type SmpW<'a, REG> = crate::BitWriter<'a, REG, Smp>;
impl<'a, REG> SmpW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "One sample is used to determine the bit value."]
    #[inline(always)]
    pub fn one_sample(self) -> &'a mut crate::W<REG> {
        self.variant(Smp::OneSample)
    }
    #[doc = "Three samples are used to determine the value of the received bit: the regular one (sample point) and two preceding samples. A majority rule is used."]
    #[inline(always)]
    pub fn three_sample(self) -> &'a mut crate::W<REG> {
        self.variant(Smp::ThreeSample)
    }
}
#[doc = "RX Warning Interrupt Mask\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rwrnmsk {
    #[doc = "0: Disabled"]
    RxWarningIntDisabled = 0,
    #[doc = "1: Enabled"]
    RxWarningIntEnabled = 1,
}
impl From<Rwrnmsk> for bool {
    #[inline(always)]
    fn from(variant: Rwrnmsk) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RWRNMSK` reader - RX Warning Interrupt Mask"]
pub type RwrnmskR = crate::BitReader<Rwrnmsk>;
impl RwrnmskR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rwrnmsk {
        match self.bits {
            false => Rwrnmsk::RxWarningIntDisabled,
            true => Rwrnmsk::RxWarningIntEnabled,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_rx_warning_int_disabled(&self) -> bool {
        *self == Rwrnmsk::RxWarningIntDisabled
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_rx_warning_int_enabled(&self) -> bool {
        *self == Rwrnmsk::RxWarningIntEnabled
    }
}
#[doc = "Field `RWRNMSK` writer - RX Warning Interrupt Mask"]
pub type RwrnmskW<'a, REG> = crate::BitWriter<'a, REG, Rwrnmsk>;
impl<'a, REG> RwrnmskW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn rx_warning_int_disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rwrnmsk::RxWarningIntDisabled)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn rx_warning_int_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rwrnmsk::RxWarningIntEnabled)
    }
}
#[doc = "TX Warning Interrupt Mask\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Twrnmsk {
    #[doc = "0: Disabled"]
    TxWarningIntDisabled = 0,
    #[doc = "1: Enabled"]
    TxWarningIntEnabled = 1,
}
impl From<Twrnmsk> for bool {
    #[inline(always)]
    fn from(variant: Twrnmsk) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TWRNMSK` reader - TX Warning Interrupt Mask"]
pub type TwrnmskR = crate::BitReader<Twrnmsk>;
impl TwrnmskR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Twrnmsk {
        match self.bits {
            false => Twrnmsk::TxWarningIntDisabled,
            true => Twrnmsk::TxWarningIntEnabled,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_tx_warning_int_disabled(&self) -> bool {
        *self == Twrnmsk::TxWarningIntDisabled
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_tx_warning_int_enabled(&self) -> bool {
        *self == Twrnmsk::TxWarningIntEnabled
    }
}
#[doc = "Field `TWRNMSK` writer - TX Warning Interrupt Mask"]
pub type TwrnmskW<'a, REG> = crate::BitWriter<'a, REG, Twrnmsk>;
impl<'a, REG> TwrnmskW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn tx_warning_int_disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Twrnmsk::TxWarningIntDisabled)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn tx_warning_int_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Twrnmsk::TxWarningIntEnabled)
    }
}
#[doc = "Loopback Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lpb {
    #[doc = "0: Disabled"]
    LoopbackDisabled = 0,
    #[doc = "1: Enabled"]
    LoopbackEnabled = 1,
}
impl From<Lpb> for bool {
    #[inline(always)]
    fn from(variant: Lpb) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LPB` reader - Loopback Mode"]
pub type LpbR = crate::BitReader<Lpb>;
impl LpbR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lpb {
        match self.bits {
            false => Lpb::LoopbackDisabled,
            true => Lpb::LoopbackEnabled,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_loopback_disabled(&self) -> bool {
        *self == Lpb::LoopbackDisabled
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_loopback_enabled(&self) -> bool {
        *self == Lpb::LoopbackEnabled
    }
}
#[doc = "Field `LPB` writer - Loopback Mode"]
pub type LpbW<'a, REG> = crate::BitWriter<'a, REG, Lpb>;
impl<'a, REG> LpbW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn loopback_disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpb::LoopbackDisabled)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn loopback_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpb::LoopbackEnabled)
    }
}
#[doc = "Error Interrupt Mask\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Errmsk {
    #[doc = "0: Interrupt disabled"]
    ErrorIntDisabled = 0,
    #[doc = "1: Interrupt enabled"]
    ErrorIntEnabled = 1,
}
impl From<Errmsk> for bool {
    #[inline(always)]
    fn from(variant: Errmsk) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERRMSK` reader - Error Interrupt Mask"]
pub type ErrmskR = crate::BitReader<Errmsk>;
impl ErrmskR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Errmsk {
        match self.bits {
            false => Errmsk::ErrorIntDisabled,
            true => Errmsk::ErrorIntEnabled,
        }
    }
    #[doc = "Interrupt disabled"]
    #[inline(always)]
    pub fn is_error_int_disabled(&self) -> bool {
        *self == Errmsk::ErrorIntDisabled
    }
    #[doc = "Interrupt enabled"]
    #[inline(always)]
    pub fn is_error_int_enabled(&self) -> bool {
        *self == Errmsk::ErrorIntEnabled
    }
}
#[doc = "Field `ERRMSK` writer - Error Interrupt Mask"]
pub type ErrmskW<'a, REG> = crate::BitWriter<'a, REG, Errmsk>;
impl<'a, REG> ErrmskW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Interrupt disabled"]
    #[inline(always)]
    pub fn error_int_disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Errmsk::ErrorIntDisabled)
    }
    #[doc = "Interrupt enabled"]
    #[inline(always)]
    pub fn error_int_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Errmsk::ErrorIntEnabled)
    }
}
#[doc = "Bus Off Interrupt Mask\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boffmsk {
    #[doc = "0: Interrupt disabled"]
    BusOffIntDisabled = 0,
    #[doc = "1: Interrupt enabled"]
    BusOffIntEnabled = 1,
}
impl From<Boffmsk> for bool {
    #[inline(always)]
    fn from(variant: Boffmsk) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BOFFMSK` reader - Bus Off Interrupt Mask"]
pub type BoffmskR = crate::BitReader<Boffmsk>;
impl BoffmskR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Boffmsk {
        match self.bits {
            false => Boffmsk::BusOffIntDisabled,
            true => Boffmsk::BusOffIntEnabled,
        }
    }
    #[doc = "Interrupt disabled"]
    #[inline(always)]
    pub fn is_bus_off_int_disabled(&self) -> bool {
        *self == Boffmsk::BusOffIntDisabled
    }
    #[doc = "Interrupt enabled"]
    #[inline(always)]
    pub fn is_bus_off_int_enabled(&self) -> bool {
        *self == Boffmsk::BusOffIntEnabled
    }
}
#[doc = "Field `BOFFMSK` writer - Bus Off Interrupt Mask"]
pub type BoffmskW<'a, REG> = crate::BitWriter<'a, REG, Boffmsk>;
impl<'a, REG> BoffmskW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Interrupt disabled"]
    #[inline(always)]
    pub fn bus_off_int_disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Boffmsk::BusOffIntDisabled)
    }
    #[doc = "Interrupt enabled"]
    #[inline(always)]
    pub fn bus_off_int_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Boffmsk::BusOffIntEnabled)
    }
}
#[doc = "Field `PSEG2` reader - Phase Segment 2"]
pub type Pseg2R = crate::FieldReader;
#[doc = "Field `PSEG2` writer - Phase Segment 2"]
pub type Pseg2W<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `PSEG1` reader - Phase Segment 1"]
pub type Pseg1R = crate::FieldReader;
#[doc = "Field `PSEG1` writer - Phase Segment 1"]
pub type Pseg1W<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `RJW` reader - Resync Jump Width"]
pub type RjwR = crate::FieldReader;
#[doc = "Field `RJW` writer - Resync Jump Width"]
pub type RjwW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `PRESDIV` reader - Prescaler Division Factor"]
pub type PresdivR = crate::FieldReader;
#[doc = "Field `PRESDIV` writer - Prescaler Division Factor"]
pub type PresdivW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:2 - Propagation Segment"]
    #[inline(always)]
    pub fn propseg(&self) -> PropsegR {
        PropsegR::new((self.bits & 7) as u8)
    }
    #[doc = "Bit 3 - Listen-Only Mode"]
    #[inline(always)]
    pub fn lom(&self) -> LomR {
        LomR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Lowest Buffer Transmitted First"]
    #[inline(always)]
    pub fn lbuf(&self) -> LbufR {
        LbufR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Timer Sync"]
    #[inline(always)]
    pub fn tsyn(&self) -> TsynR {
        TsynR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Bus Off Recovery"]
    #[inline(always)]
    pub fn boffrec(&self) -> BoffrecR {
        BoffrecR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - CAN Bit Sampling"]
    #[inline(always)]
    pub fn smp(&self) -> SmpR {
        SmpR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 10 - RX Warning Interrupt Mask"]
    #[inline(always)]
    pub fn rwrnmsk(&self) -> RwrnmskR {
        RwrnmskR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - TX Warning Interrupt Mask"]
    #[inline(always)]
    pub fn twrnmsk(&self) -> TwrnmskR {
        TwrnmskR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Loopback Mode"]
    #[inline(always)]
    pub fn lpb(&self) -> LpbR {
        LpbR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 14 - Error Interrupt Mask"]
    #[inline(always)]
    pub fn errmsk(&self) -> ErrmskR {
        ErrmskR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Bus Off Interrupt Mask"]
    #[inline(always)]
    pub fn boffmsk(&self) -> BoffmskR {
        BoffmskR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bits 16:18 - Phase Segment 2"]
    #[inline(always)]
    pub fn pseg2(&self) -> Pseg2R {
        Pseg2R::new(((self.bits >> 16) & 7) as u8)
    }
    #[doc = "Bits 19:21 - Phase Segment 1"]
    #[inline(always)]
    pub fn pseg1(&self) -> Pseg1R {
        Pseg1R::new(((self.bits >> 19) & 7) as u8)
    }
    #[doc = "Bits 22:23 - Resync Jump Width"]
    #[inline(always)]
    pub fn rjw(&self) -> RjwR {
        RjwR::new(((self.bits >> 22) & 3) as u8)
    }
    #[doc = "Bits 24:31 - Prescaler Division Factor"]
    #[inline(always)]
    pub fn presdiv(&self) -> PresdivR {
        PresdivR::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:2 - Propagation Segment"]
    #[inline(always)]
    pub fn propseg(&mut self) -> PropsegW<Ctrl1Spec> {
        PropsegW::new(self, 0)
    }
    #[doc = "Bit 3 - Listen-Only Mode"]
    #[inline(always)]
    pub fn lom(&mut self) -> LomW<Ctrl1Spec> {
        LomW::new(self, 3)
    }
    #[doc = "Bit 4 - Lowest Buffer Transmitted First"]
    #[inline(always)]
    pub fn lbuf(&mut self) -> LbufW<Ctrl1Spec> {
        LbufW::new(self, 4)
    }
    #[doc = "Bit 5 - Timer Sync"]
    #[inline(always)]
    pub fn tsyn(&mut self) -> TsynW<Ctrl1Spec> {
        TsynW::new(self, 5)
    }
    #[doc = "Bit 6 - Bus Off Recovery"]
    #[inline(always)]
    pub fn boffrec(&mut self) -> BoffrecW<Ctrl1Spec> {
        BoffrecW::new(self, 6)
    }
    #[doc = "Bit 7 - CAN Bit Sampling"]
    #[inline(always)]
    pub fn smp(&mut self) -> SmpW<Ctrl1Spec> {
        SmpW::new(self, 7)
    }
    #[doc = "Bit 10 - RX Warning Interrupt Mask"]
    #[inline(always)]
    pub fn rwrnmsk(&mut self) -> RwrnmskW<Ctrl1Spec> {
        RwrnmskW::new(self, 10)
    }
    #[doc = "Bit 11 - TX Warning Interrupt Mask"]
    #[inline(always)]
    pub fn twrnmsk(&mut self) -> TwrnmskW<Ctrl1Spec> {
        TwrnmskW::new(self, 11)
    }
    #[doc = "Bit 12 - Loopback Mode"]
    #[inline(always)]
    pub fn lpb(&mut self) -> LpbW<Ctrl1Spec> {
        LpbW::new(self, 12)
    }
    #[doc = "Bit 14 - Error Interrupt Mask"]
    #[inline(always)]
    pub fn errmsk(&mut self) -> ErrmskW<Ctrl1Spec> {
        ErrmskW::new(self, 14)
    }
    #[doc = "Bit 15 - Bus Off Interrupt Mask"]
    #[inline(always)]
    pub fn boffmsk(&mut self) -> BoffmskW<Ctrl1Spec> {
        BoffmskW::new(self, 15)
    }
    #[doc = "Bits 16:18 - Phase Segment 2"]
    #[inline(always)]
    pub fn pseg2(&mut self) -> Pseg2W<Ctrl1Spec> {
        Pseg2W::new(self, 16)
    }
    #[doc = "Bits 19:21 - Phase Segment 1"]
    #[inline(always)]
    pub fn pseg1(&mut self) -> Pseg1W<Ctrl1Spec> {
        Pseg1W::new(self, 19)
    }
    #[doc = "Bits 22:23 - Resync Jump Width"]
    #[inline(always)]
    pub fn rjw(&mut self) -> RjwW<Ctrl1Spec> {
        RjwW::new(self, 22)
    }
    #[doc = "Bits 24:31 - Prescaler Division Factor"]
    #[inline(always)]
    pub fn presdiv(&mut self) -> PresdivW<Ctrl1Spec> {
        PresdivW::new(self, 24)
    }
}
#[doc = "Control 1\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ctrl1Spec;
impl crate::RegisterSpec for Ctrl1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ctrl1::R`](R) reader structure"]
impl crate::Readable for Ctrl1Spec {}
#[doc = "`write(|w| ..)` method takes [`ctrl1::W`](W) writer structure"]
impl crate::Writable for Ctrl1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CTRL1 to value 0"]
impl crate::Resettable for Ctrl1Spec {}

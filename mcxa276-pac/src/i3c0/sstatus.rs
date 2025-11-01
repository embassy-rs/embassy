#[doc = "Register `SSTATUS` reader"]
pub type R = crate::R<SstatusSpec>;
#[doc = "Register `SSTATUS` writer"]
pub type W = crate::W<SstatusSpec>;
#[doc = "Status not Stop\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stnotstop {
    #[doc = "0: In STOP condition"]
    Stopped = 0,
    #[doc = "1: Busy"]
    Busy = 1,
}
impl From<Stnotstop> for bool {
    #[inline(always)]
    fn from(variant: Stnotstop) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STNOTSTOP` reader - Status not Stop"]
pub type StnotstopR = crate::BitReader<Stnotstop>;
impl StnotstopR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Stnotstop {
        match self.bits {
            false => Stnotstop::Stopped,
            true => Stnotstop::Busy,
        }
    }
    #[doc = "In STOP condition"]
    #[inline(always)]
    pub fn is_stopped(&self) -> bool {
        *self == Stnotstop::Stopped
    }
    #[doc = "Busy"]
    #[inline(always)]
    pub fn is_busy(&self) -> bool {
        *self == Stnotstop::Busy
    }
}
#[doc = "Status Message\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stmsg {
    #[doc = "0: Idle"]
    Idle = 0,
    #[doc = "1: Busy"]
    Busy = 1,
}
impl From<Stmsg> for bool {
    #[inline(always)]
    fn from(variant: Stmsg) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STMSG` reader - Status Message"]
pub type StmsgR = crate::BitReader<Stmsg>;
impl StmsgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Stmsg {
        match self.bits {
            false => Stmsg::Idle,
            true => Stmsg::Busy,
        }
    }
    #[doc = "Idle"]
    #[inline(always)]
    pub fn is_idle(&self) -> bool {
        *self == Stmsg::Idle
    }
    #[doc = "Busy"]
    #[inline(always)]
    pub fn is_busy(&self) -> bool {
        *self == Stmsg::Busy
    }
}
#[doc = "Status Common Command Code Handler\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stccch {
    #[doc = "0: No CCC message handled"]
    Idle = 0,
    #[doc = "1: Handled automatically"]
    Busy = 1,
}
impl From<Stccch> for bool {
    #[inline(always)]
    fn from(variant: Stccch) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STCCCH` reader - Status Common Command Code Handler"]
pub type StccchR = crate::BitReader<Stccch>;
impl StccchR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Stccch {
        match self.bits {
            false => Stccch::Idle,
            true => Stccch::Busy,
        }
    }
    #[doc = "No CCC message handled"]
    #[inline(always)]
    pub fn is_idle(&self) -> bool {
        *self == Stccch::Idle
    }
    #[doc = "Handled automatically"]
    #[inline(always)]
    pub fn is_busy(&self) -> bool {
        *self == Stccch::Busy
    }
}
#[doc = "Status Request Read\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Streqrd {
    #[doc = "0: Not an SDR read"]
    Idle = 0,
    #[doc = "1: SDR read from this target or an IBI is being pushed out"]
    Busy = 1,
}
impl From<Streqrd> for bool {
    #[inline(always)]
    fn from(variant: Streqrd) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STREQRD` reader - Status Request Read"]
pub type StreqrdR = crate::BitReader<Streqrd>;
impl StreqrdR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Streqrd {
        match self.bits {
            false => Streqrd::Idle,
            true => Streqrd::Busy,
        }
    }
    #[doc = "Not an SDR read"]
    #[inline(always)]
    pub fn is_idle(&self) -> bool {
        *self == Streqrd::Idle
    }
    #[doc = "SDR read from this target or an IBI is being pushed out"]
    #[inline(always)]
    pub fn is_busy(&self) -> bool {
        *self == Streqrd::Busy
    }
}
#[doc = "Status Request Write\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Streqwr {
    #[doc = "0: Not an SDR write"]
    Idle = 0,
    #[doc = "1: SDR write data from the controller, but not in ENTDAA mode"]
    Busy = 1,
}
impl From<Streqwr> for bool {
    #[inline(always)]
    fn from(variant: Streqwr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STREQWR` reader - Status Request Write"]
pub type StreqwrR = crate::BitReader<Streqwr>;
impl StreqwrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Streqwr {
        match self.bits {
            false => Streqwr::Idle,
            true => Streqwr::Busy,
        }
    }
    #[doc = "Not an SDR write"]
    #[inline(always)]
    pub fn is_idle(&self) -> bool {
        *self == Streqwr::Idle
    }
    #[doc = "SDR write data from the controller, but not in ENTDAA mode"]
    #[inline(always)]
    pub fn is_busy(&self) -> bool {
        *self == Streqwr::Busy
    }
}
#[doc = "Status Dynamic Address Assignment\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stdaa {
    #[doc = "0: Not in ENTDAA mode"]
    NotInEntdaa = 0,
    #[doc = "1: In ENTDAA mode"]
    InEntdaa = 1,
}
impl From<Stdaa> for bool {
    #[inline(always)]
    fn from(variant: Stdaa) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STDAA` reader - Status Dynamic Address Assignment"]
pub type StdaaR = crate::BitReader<Stdaa>;
impl StdaaR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Stdaa {
        match self.bits {
            false => Stdaa::NotInEntdaa,
            true => Stdaa::InEntdaa,
        }
    }
    #[doc = "Not in ENTDAA mode"]
    #[inline(always)]
    pub fn is_not_in_entdaa(&self) -> bool {
        *self == Stdaa::NotInEntdaa
    }
    #[doc = "In ENTDAA mode"]
    #[inline(always)]
    pub fn is_in_entdaa(&self) -> bool {
        *self == Stdaa::InEntdaa
    }
}
#[doc = "Status High Data Rate\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sthdr {
    #[doc = "0: I3C bus not in HDR-DDR mode"]
    NotInHdrDdr = 0,
    #[doc = "1: I3C bus in HDR-DDR mode"]
    InHdrDdr = 1,
}
impl From<Sthdr> for bool {
    #[inline(always)]
    fn from(variant: Sthdr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STHDR` reader - Status High Data Rate"]
pub type SthdrR = crate::BitReader<Sthdr>;
impl SthdrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sthdr {
        match self.bits {
            false => Sthdr::NotInHdrDdr,
            true => Sthdr::InHdrDdr,
        }
    }
    #[doc = "I3C bus not in HDR-DDR mode"]
    #[inline(always)]
    pub fn is_not_in_hdr_ddr(&self) -> bool {
        *self == Sthdr::NotInHdrDdr
    }
    #[doc = "I3C bus in HDR-DDR mode"]
    #[inline(always)]
    pub fn is_in_hdr_ddr(&self) -> bool {
        *self == Sthdr::InHdrDdr
    }
}
#[doc = "Start Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Start {
    #[doc = "0: Not detected"]
    StartNotDetected = 0,
    #[doc = "1: Detected"]
    StartDetected = 1,
}
impl From<Start> for bool {
    #[inline(always)]
    fn from(variant: Start) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `START` reader - Start Flag"]
pub type StartR = crate::BitReader<Start>;
impl StartR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Start {
        match self.bits {
            false => Start::StartNotDetected,
            true => Start::StartDetected,
        }
    }
    #[doc = "Not detected"]
    #[inline(always)]
    pub fn is_start_not_detected(&self) -> bool {
        *self == Start::StartNotDetected
    }
    #[doc = "Detected"]
    #[inline(always)]
    pub fn is_start_detected(&self) -> bool {
        *self == Start::StartDetected
    }
}
#[doc = "Field `START` writer - Start Flag"]
pub type StartW<'a, REG> = crate::BitWriter1C<'a, REG, Start>;
impl<'a, REG> StartW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not detected"]
    #[inline(always)]
    pub fn start_not_detected(self) -> &'a mut crate::W<REG> {
        self.variant(Start::StartNotDetected)
    }
    #[doc = "Detected"]
    #[inline(always)]
    pub fn start_detected(self) -> &'a mut crate::W<REG> {
        self.variant(Start::StartDetected)
    }
}
#[doc = "Matched Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Matched {
    #[doc = "0: Header not matched"]
    NotMatched = 0,
    #[doc = "1: Header matched"]
    Matched = 1,
}
impl From<Matched> for bool {
    #[inline(always)]
    fn from(variant: Matched) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MATCHED` reader - Matched Flag"]
pub type MatchedR = crate::BitReader<Matched>;
impl MatchedR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Matched {
        match self.bits {
            false => Matched::NotMatched,
            true => Matched::Matched,
        }
    }
    #[doc = "Header not matched"]
    #[inline(always)]
    pub fn is_not_matched(&self) -> bool {
        *self == Matched::NotMatched
    }
    #[doc = "Header matched"]
    #[inline(always)]
    pub fn is_matched(&self) -> bool {
        *self == Matched::Matched
    }
}
#[doc = "Field `MATCHED` writer - Matched Flag"]
pub type MatchedW<'a, REG> = crate::BitWriter1C<'a, REG, Matched>;
impl<'a, REG> MatchedW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Header not matched"]
    #[inline(always)]
    pub fn not_matched(self) -> &'a mut crate::W<REG> {
        self.variant(Matched::NotMatched)
    }
    #[doc = "Header matched"]
    #[inline(always)]
    pub fn matched(self) -> &'a mut crate::W<REG> {
        self.variant(Matched::Matched)
    }
}
#[doc = "Stop Flag\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stop {
    #[doc = "0: No Stopped state detected"]
    NoStopDetected = 0,
    #[doc = "1: Stopped state detected"]
    StopDetected = 1,
}
impl From<Stop> for bool {
    #[inline(always)]
    fn from(variant: Stop) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STOP` reader - Stop Flag"]
pub type StopR = crate::BitReader<Stop>;
impl StopR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Stop {
        match self.bits {
            false => Stop::NoStopDetected,
            true => Stop::StopDetected,
        }
    }
    #[doc = "No Stopped state detected"]
    #[inline(always)]
    pub fn is_no_stop_detected(&self) -> bool {
        *self == Stop::NoStopDetected
    }
    #[doc = "Stopped state detected"]
    #[inline(always)]
    pub fn is_stop_detected(&self) -> bool {
        *self == Stop::StopDetected
    }
}
#[doc = "Field `STOP` writer - Stop Flag"]
pub type StopW<'a, REG> = crate::BitWriter1C<'a, REG, Stop>;
impl<'a, REG> StopW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No Stopped state detected"]
    #[inline(always)]
    pub fn no_stop_detected(self) -> &'a mut crate::W<REG> {
        self.variant(Stop::NoStopDetected)
    }
    #[doc = "Stopped state detected"]
    #[inline(always)]
    pub fn stop_detected(self) -> &'a mut crate::W<REG> {
        self.variant(Stop::StopDetected)
    }
}
#[doc = "Received Message Pending\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RxPend {
    #[doc = "0: No received message pending"]
    NoMsgPending = 0,
    #[doc = "1: Received message pending"]
    MsgPending = 1,
}
impl From<RxPend> for bool {
    #[inline(always)]
    fn from(variant: RxPend) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RX_PEND` reader - Received Message Pending"]
pub type RxPendR = crate::BitReader<RxPend>;
impl RxPendR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RxPend {
        match self.bits {
            false => RxPend::NoMsgPending,
            true => RxPend::MsgPending,
        }
    }
    #[doc = "No received message pending"]
    #[inline(always)]
    pub fn is_no_msg_pending(&self) -> bool {
        *self == RxPend::NoMsgPending
    }
    #[doc = "Received message pending"]
    #[inline(always)]
    pub fn is_msg_pending(&self) -> bool {
        *self == RxPend::MsgPending
    }
}
#[doc = "Transmit Buffer Not Full\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Txnotfull {
    #[doc = "0: Transmit buffer full"]
    Full = 0,
    #[doc = "1: Transmit buffer not full"]
    NotFull = 1,
}
impl From<Txnotfull> for bool {
    #[inline(always)]
    fn from(variant: Txnotfull) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TXNOTFULL` reader - Transmit Buffer Not Full"]
pub type TxnotfullR = crate::BitReader<Txnotfull>;
impl TxnotfullR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Txnotfull {
        match self.bits {
            false => Txnotfull::Full,
            true => Txnotfull::NotFull,
        }
    }
    #[doc = "Transmit buffer full"]
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        *self == Txnotfull::Full
    }
    #[doc = "Transmit buffer not full"]
    #[inline(always)]
    pub fn is_not_full(&self) -> bool {
        *self == Txnotfull::NotFull
    }
}
#[doc = "Dynamic Address Change Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dachg {
    #[doc = "0: No DA change detected"]
    NoChangeDetected = 0,
    #[doc = "1: DA change detected"]
    ChangeDetected = 1,
}
impl From<Dachg> for bool {
    #[inline(always)]
    fn from(variant: Dachg) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DACHG` reader - Dynamic Address Change Flag"]
pub type DachgR = crate::BitReader<Dachg>;
impl DachgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dachg {
        match self.bits {
            false => Dachg::NoChangeDetected,
            true => Dachg::ChangeDetected,
        }
    }
    #[doc = "No DA change detected"]
    #[inline(always)]
    pub fn is_no_change_detected(&self) -> bool {
        *self == Dachg::NoChangeDetected
    }
    #[doc = "DA change detected"]
    #[inline(always)]
    pub fn is_change_detected(&self) -> bool {
        *self == Dachg::ChangeDetected
    }
}
#[doc = "Field `DACHG` writer - Dynamic Address Change Flag"]
pub type DachgW<'a, REG> = crate::BitWriter1C<'a, REG, Dachg>;
impl<'a, REG> DachgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No DA change detected"]
    #[inline(always)]
    pub fn no_change_detected(self) -> &'a mut crate::W<REG> {
        self.variant(Dachg::NoChangeDetected)
    }
    #[doc = "DA change detected"]
    #[inline(always)]
    pub fn change_detected(self) -> &'a mut crate::W<REG> {
        self.variant(Dachg::ChangeDetected)
    }
}
#[doc = "Common Command Code Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ccc {
    #[doc = "0: CCC not received"]
    NoCccReceived = 0,
    #[doc = "1: CCC received"]
    CccReceived = 1,
}
impl From<Ccc> for bool {
    #[inline(always)]
    fn from(variant: Ccc) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CCC` reader - Common Command Code Flag"]
pub type CccR = crate::BitReader<Ccc>;
impl CccR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ccc {
        match self.bits {
            false => Ccc::NoCccReceived,
            true => Ccc::CccReceived,
        }
    }
    #[doc = "CCC not received"]
    #[inline(always)]
    pub fn is_no_ccc_received(&self) -> bool {
        *self == Ccc::NoCccReceived
    }
    #[doc = "CCC received"]
    #[inline(always)]
    pub fn is_ccc_received(&self) -> bool {
        *self == Ccc::CccReceived
    }
}
#[doc = "Field `CCC` writer - Common Command Code Flag"]
pub type CccW<'a, REG> = crate::BitWriter1C<'a, REG, Ccc>;
impl<'a, REG> CccW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "CCC not received"]
    #[inline(always)]
    pub fn no_ccc_received(self) -> &'a mut crate::W<REG> {
        self.variant(Ccc::NoCccReceived)
    }
    #[doc = "CCC received"]
    #[inline(always)]
    pub fn ccc_received(self) -> &'a mut crate::W<REG> {
        self.variant(Ccc::CccReceived)
    }
}
#[doc = "Field `ERRWARN` reader - Error Warning"]
pub type ErrwarnR = crate::BitReader;
#[doc = "High Data Rate Command Match Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hdrmatch {
    #[doc = "0: Did not match"]
    NoMatch = 0,
    #[doc = "1: Matched the I3C dynamic address"]
    Match = 1,
}
impl From<Hdrmatch> for bool {
    #[inline(always)]
    fn from(variant: Hdrmatch) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HDRMATCH` reader - High Data Rate Command Match Flag"]
pub type HdrmatchR = crate::BitReader<Hdrmatch>;
impl HdrmatchR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Hdrmatch {
        match self.bits {
            false => Hdrmatch::NoMatch,
            true => Hdrmatch::Match,
        }
    }
    #[doc = "Did not match"]
    #[inline(always)]
    pub fn is_no_match(&self) -> bool {
        *self == Hdrmatch::NoMatch
    }
    #[doc = "Matched the I3C dynamic address"]
    #[inline(always)]
    pub fn is_match(&self) -> bool {
        *self == Hdrmatch::Match
    }
}
#[doc = "Field `HDRMATCH` writer - High Data Rate Command Match Flag"]
pub type HdrmatchW<'a, REG> = crate::BitWriter1C<'a, REG, Hdrmatch>;
impl<'a, REG> HdrmatchW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Did not match"]
    #[inline(always)]
    pub fn no_match(self) -> &'a mut crate::W<REG> {
        self.variant(Hdrmatch::NoMatch)
    }
    #[doc = "Matched the I3C dynamic address"]
    #[inline(always)]
    pub fn match_(self) -> &'a mut crate::W<REG> {
        self.variant(Hdrmatch::Match)
    }
}
#[doc = "Common Command Code Handled Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chandled {
    #[doc = "0: CCC handling not in progress"]
    NotHandled = 0,
    #[doc = "1: CCC handling in progress"]
    Handled = 1,
}
impl From<Chandled> for bool {
    #[inline(always)]
    fn from(variant: Chandled) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CHANDLED` reader - Common Command Code Handled Flag"]
pub type ChandledR = crate::BitReader<Chandled>;
impl ChandledR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Chandled {
        match self.bits {
            false => Chandled::NotHandled,
            true => Chandled::Handled,
        }
    }
    #[doc = "CCC handling not in progress"]
    #[inline(always)]
    pub fn is_not_handled(&self) -> bool {
        *self == Chandled::NotHandled
    }
    #[doc = "CCC handling in progress"]
    #[inline(always)]
    pub fn is_handled(&self) -> bool {
        *self == Chandled::Handled
    }
}
#[doc = "Field `CHANDLED` writer - Common Command Code Handled Flag"]
pub type ChandledW<'a, REG> = crate::BitWriter1C<'a, REG, Chandled>;
impl<'a, REG> ChandledW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "CCC handling not in progress"]
    #[inline(always)]
    pub fn not_handled(self) -> &'a mut crate::W<REG> {
        self.variant(Chandled::NotHandled)
    }
    #[doc = "CCC handling in progress"]
    #[inline(always)]
    pub fn handled(self) -> &'a mut crate::W<REG> {
        self.variant(Chandled::Handled)
    }
}
#[doc = "Event Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    #[doc = "0: No event occurred"]
    NoEvent = 0,
    #[doc = "1: IBI, CR, or HJ occurred"]
    Event = 1,
}
impl From<Event> for bool {
    #[inline(always)]
    fn from(variant: Event) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EVENT` reader - Event Flag"]
pub type EventR = crate::BitReader<Event>;
impl EventR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Event {
        match self.bits {
            false => Event::NoEvent,
            true => Event::Event,
        }
    }
    #[doc = "No event occurred"]
    #[inline(always)]
    pub fn is_no_event(&self) -> bool {
        *self == Event::NoEvent
    }
    #[doc = "IBI, CR, or HJ occurred"]
    #[inline(always)]
    pub fn is_event(&self) -> bool {
        *self == Event::Event
    }
}
#[doc = "Field `EVENT` writer - Event Flag"]
pub type EventW<'a, REG> = crate::BitWriter1C<'a, REG, Event>;
impl<'a, REG> EventW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No event occurred"]
    #[inline(always)]
    pub fn no_event(self) -> &'a mut crate::W<REG> {
        self.variant(Event::NoEvent)
    }
    #[doc = "IBI, CR, or HJ occurred"]
    #[inline(always)]
    pub fn event(self) -> &'a mut crate::W<REG> {
        self.variant(Event::Event)
    }
}
#[doc = "Event Details\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Evdet {
    #[doc = "0: NONE (no event or no pending event)"]
    None = 0,
    #[doc = "1: NO_REQUEST (request is not sent yet; either there is no START condition yet, or is waiting for Bus-Available or Bus-Idle (HJ))"]
    NoRequest = 1,
    #[doc = "2: NACKed (not acknowledged, request sent and rejected); I3C tries again"]
    Nacked = 2,
    #[doc = "3: ACKed (acknowledged; request sent and accepted), so done (unless the time control data is still being sent)"]
    Acked = 3,
}
impl From<Evdet> for u8 {
    #[inline(always)]
    fn from(variant: Evdet) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Evdet {
    type Ux = u8;
}
impl crate::IsEnum for Evdet {}
#[doc = "Field `EVDET` reader - Event Details"]
pub type EvdetR = crate::FieldReader<Evdet>;
impl EvdetR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Evdet {
        match self.bits {
            0 => Evdet::None,
            1 => Evdet::NoRequest,
            2 => Evdet::Nacked,
            3 => Evdet::Acked,
            _ => unreachable!(),
        }
    }
    #[doc = "NONE (no event or no pending event)"]
    #[inline(always)]
    pub fn is_none(&self) -> bool {
        *self == Evdet::None
    }
    #[doc = "NO_REQUEST (request is not sent yet; either there is no START condition yet, or is waiting for Bus-Available or Bus-Idle (HJ))"]
    #[inline(always)]
    pub fn is_no_request(&self) -> bool {
        *self == Evdet::NoRequest
    }
    #[doc = "NACKed (not acknowledged, request sent and rejected); I3C tries again"]
    #[inline(always)]
    pub fn is_nacked(&self) -> bool {
        *self == Evdet::Nacked
    }
    #[doc = "ACKed (acknowledged; request sent and accepted), so done (unless the time control data is still being sent)"]
    #[inline(always)]
    pub fn is_acked(&self) -> bool {
        *self == Evdet::Acked
    }
}
#[doc = "In-Band Interrupts Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ibidis {
    #[doc = "0: Enabled"]
    InterruptsEnabled = 0,
    #[doc = "1: Disabled"]
    InterruptsDisabled = 1,
}
impl From<Ibidis> for bool {
    #[inline(always)]
    fn from(variant: Ibidis) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `IBIDIS` reader - In-Band Interrupts Disable"]
pub type IbidisR = crate::BitReader<Ibidis>;
impl IbidisR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ibidis {
        match self.bits {
            false => Ibidis::InterruptsEnabled,
            true => Ibidis::InterruptsDisabled,
        }
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_interrupts_enabled(&self) -> bool {
        *self == Ibidis::InterruptsEnabled
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_interrupts_disabled(&self) -> bool {
        *self == Ibidis::InterruptsDisabled
    }
}
#[doc = "Controller Requests Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mrdis {
    #[doc = "0: Enabled"]
    MrEnabled = 0,
    #[doc = "1: Disabled"]
    MrDisabled = 1,
}
impl From<Mrdis> for bool {
    #[inline(always)]
    fn from(variant: Mrdis) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MRDIS` reader - Controller Requests Disable"]
pub type MrdisR = crate::BitReader<Mrdis>;
impl MrdisR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mrdis {
        match self.bits {
            false => Mrdis::MrEnabled,
            true => Mrdis::MrDisabled,
        }
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_mr_enabled(&self) -> bool {
        *self == Mrdis::MrEnabled
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_mr_disabled(&self) -> bool {
        *self == Mrdis::MrDisabled
    }
}
#[doc = "Hot-Join Disabled\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hjdis {
    #[doc = "0: Enabled"]
    MrEnabled = 0,
    #[doc = "1: Disabled"]
    MrDisabled = 1,
}
impl From<Hjdis> for bool {
    #[inline(always)]
    fn from(variant: Hjdis) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HJDIS` reader - Hot-Join Disabled"]
pub type HjdisR = crate::BitReader<Hjdis>;
impl HjdisR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Hjdis {
        match self.bits {
            false => Hjdis::MrEnabled,
            true => Hjdis::MrDisabled,
        }
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_mr_enabled(&self) -> bool {
        *self == Hjdis::MrEnabled
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_mr_disabled(&self) -> bool {
        *self == Hjdis::MrDisabled
    }
}
#[doc = "Activity State from Common Command Codes (CCC)\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Actstate {
    #[doc = "0: NO_LATENCY (normal bus operations)"]
    NoLatency = 0,
    #[doc = "1: LATENCY_1MS (1 ms of latency)"]
    Latency1ms = 1,
    #[doc = "2: LATENCY_100MS (100 ms of latency)"]
    Latency100ms = 2,
    #[doc = "3: LATENCY_10S (10 seconds of latency)"]
    Latency10s = 3,
}
impl From<Actstate> for u8 {
    #[inline(always)]
    fn from(variant: Actstate) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Actstate {
    type Ux = u8;
}
impl crate::IsEnum for Actstate {}
#[doc = "Field `ACTSTATE` reader - Activity State from Common Command Codes (CCC)"]
pub type ActstateR = crate::FieldReader<Actstate>;
impl ActstateR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Actstate {
        match self.bits {
            0 => Actstate::NoLatency,
            1 => Actstate::Latency1ms,
            2 => Actstate::Latency100ms,
            3 => Actstate::Latency10s,
            _ => unreachable!(),
        }
    }
    #[doc = "NO_LATENCY (normal bus operations)"]
    #[inline(always)]
    pub fn is_no_latency(&self) -> bool {
        *self == Actstate::NoLatency
    }
    #[doc = "LATENCY_1MS (1 ms of latency)"]
    #[inline(always)]
    pub fn is_latency_1ms(&self) -> bool {
        *self == Actstate::Latency1ms
    }
    #[doc = "LATENCY_100MS (100 ms of latency)"]
    #[inline(always)]
    pub fn is_latency_100ms(&self) -> bool {
        *self == Actstate::Latency100ms
    }
    #[doc = "LATENCY_10S (10 seconds of latency)"]
    #[inline(always)]
    pub fn is_latency_10s(&self) -> bool {
        *self == Actstate::Latency10s
    }
}
#[doc = "Time Control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Timectrl {
    #[doc = "0: NO_TIME_CONTROL (no time control is enabled)"]
    NoTimeControl = 0,
    #[doc = "1: SYNC_MODE (Synchronous mode is enabled)"]
    Sync = 1,
    #[doc = "2: ASYNC_MODE (Asynchronous standard mode (0 or 1) is enabled)"]
    AsyncMode = 2,
    #[doc = "3: BOTHSYNCASYNC (both Synchronous and Asynchronous modes are enabled)"]
    Bothsyncasync = 3,
}
impl From<Timectrl> for u8 {
    #[inline(always)]
    fn from(variant: Timectrl) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Timectrl {
    type Ux = u8;
}
impl crate::IsEnum for Timectrl {}
#[doc = "Field `TIMECTRL` reader - Time Control"]
pub type TimectrlR = crate::FieldReader<Timectrl>;
impl TimectrlR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Timectrl {
        match self.bits {
            0 => Timectrl::NoTimeControl,
            1 => Timectrl::Sync,
            2 => Timectrl::AsyncMode,
            3 => Timectrl::Bothsyncasync,
            _ => unreachable!(),
        }
    }
    #[doc = "NO_TIME_CONTROL (no time control is enabled)"]
    #[inline(always)]
    pub fn is_no_time_control(&self) -> bool {
        *self == Timectrl::NoTimeControl
    }
    #[doc = "SYNC_MODE (Synchronous mode is enabled)"]
    #[inline(always)]
    pub fn is_sync(&self) -> bool {
        *self == Timectrl::Sync
    }
    #[doc = "ASYNC_MODE (Asynchronous standard mode (0 or 1) is enabled)"]
    #[inline(always)]
    pub fn is_async_mode(&self) -> bool {
        *self == Timectrl::AsyncMode
    }
    #[doc = "BOTHSYNCASYNC (both Synchronous and Asynchronous modes are enabled)"]
    #[inline(always)]
    pub fn is_bothsyncasync(&self) -> bool {
        *self == Timectrl::Bothsyncasync
    }
}
impl R {
    #[doc = "Bit 0 - Status not Stop"]
    #[inline(always)]
    pub fn stnotstop(&self) -> StnotstopR {
        StnotstopR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Status Message"]
    #[inline(always)]
    pub fn stmsg(&self) -> StmsgR {
        StmsgR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Status Common Command Code Handler"]
    #[inline(always)]
    pub fn stccch(&self) -> StccchR {
        StccchR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Status Request Read"]
    #[inline(always)]
    pub fn streqrd(&self) -> StreqrdR {
        StreqrdR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Status Request Write"]
    #[inline(always)]
    pub fn streqwr(&self) -> StreqwrR {
        StreqwrR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Status Dynamic Address Assignment"]
    #[inline(always)]
    pub fn stdaa(&self) -> StdaaR {
        StdaaR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Status High Data Rate"]
    #[inline(always)]
    pub fn sthdr(&self) -> SthdrR {
        SthdrR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 8 - Start Flag"]
    #[inline(always)]
    pub fn start(&self) -> StartR {
        StartR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Matched Flag"]
    #[inline(always)]
    pub fn matched(&self) -> MatchedR {
        MatchedR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Stop Flag"]
    #[inline(always)]
    pub fn stop(&self) -> StopR {
        StopR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Received Message Pending"]
    #[inline(always)]
    pub fn rx_pend(&self) -> RxPendR {
        RxPendR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Transmit Buffer Not Full"]
    #[inline(always)]
    pub fn txnotfull(&self) -> TxnotfullR {
        TxnotfullR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Dynamic Address Change Flag"]
    #[inline(always)]
    pub fn dachg(&self) -> DachgR {
        DachgR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Common Command Code Flag"]
    #[inline(always)]
    pub fn ccc(&self) -> CccR {
        CccR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Error Warning"]
    #[inline(always)]
    pub fn errwarn(&self) -> ErrwarnR {
        ErrwarnR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - High Data Rate Command Match Flag"]
    #[inline(always)]
    pub fn hdrmatch(&self) -> HdrmatchR {
        HdrmatchR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Common Command Code Handled Flag"]
    #[inline(always)]
    pub fn chandled(&self) -> ChandledR {
        ChandledR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Event Flag"]
    #[inline(always)]
    pub fn event(&self) -> EventR {
        EventR::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bits 20:21 - Event Details"]
    #[inline(always)]
    pub fn evdet(&self) -> EvdetR {
        EvdetR::new(((self.bits >> 20) & 3) as u8)
    }
    #[doc = "Bit 24 - In-Band Interrupts Disable"]
    #[inline(always)]
    pub fn ibidis(&self) -> IbidisR {
        IbidisR::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - Controller Requests Disable"]
    #[inline(always)]
    pub fn mrdis(&self) -> MrdisR {
        MrdisR::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 27 - Hot-Join Disabled"]
    #[inline(always)]
    pub fn hjdis(&self) -> HjdisR {
        HjdisR::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bits 28:29 - Activity State from Common Command Codes (CCC)"]
    #[inline(always)]
    pub fn actstate(&self) -> ActstateR {
        ActstateR::new(((self.bits >> 28) & 3) as u8)
    }
    #[doc = "Bits 30:31 - Time Control"]
    #[inline(always)]
    pub fn timectrl(&self) -> TimectrlR {
        TimectrlR::new(((self.bits >> 30) & 3) as u8)
    }
}
impl W {
    #[doc = "Bit 8 - Start Flag"]
    #[inline(always)]
    pub fn start(&mut self) -> StartW<SstatusSpec> {
        StartW::new(self, 8)
    }
    #[doc = "Bit 9 - Matched Flag"]
    #[inline(always)]
    pub fn matched(&mut self) -> MatchedW<SstatusSpec> {
        MatchedW::new(self, 9)
    }
    #[doc = "Bit 10 - Stop Flag"]
    #[inline(always)]
    pub fn stop(&mut self) -> StopW<SstatusSpec> {
        StopW::new(self, 10)
    }
    #[doc = "Bit 13 - Dynamic Address Change Flag"]
    #[inline(always)]
    pub fn dachg(&mut self) -> DachgW<SstatusSpec> {
        DachgW::new(self, 13)
    }
    #[doc = "Bit 14 - Common Command Code Flag"]
    #[inline(always)]
    pub fn ccc(&mut self) -> CccW<SstatusSpec> {
        CccW::new(self, 14)
    }
    #[doc = "Bit 16 - High Data Rate Command Match Flag"]
    #[inline(always)]
    pub fn hdrmatch(&mut self) -> HdrmatchW<SstatusSpec> {
        HdrmatchW::new(self, 16)
    }
    #[doc = "Bit 17 - Common Command Code Handled Flag"]
    #[inline(always)]
    pub fn chandled(&mut self) -> ChandledW<SstatusSpec> {
        ChandledW::new(self, 17)
    }
    #[doc = "Bit 18 - Event Flag"]
    #[inline(always)]
    pub fn event(&mut self) -> EventW<SstatusSpec> {
        EventW::new(self, 18)
    }
}
#[doc = "Target Status\n\nYou can [`read`](crate::Reg::read) this register and get [`sstatus::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sstatus::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SstatusSpec;
impl crate::RegisterSpec for SstatusSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sstatus::R`](R) reader structure"]
impl crate::Readable for SstatusSpec {}
#[doc = "`write(|w| ..)` method takes [`sstatus::W`](W) writer structure"]
impl crate::Writable for SstatusSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x0007_6700;
}
#[doc = "`reset()` method sets SSTATUS to value 0x1400"]
impl crate::Resettable for SstatusSpec {
    const RESET_VALUE: u32 = 0x1400;
}

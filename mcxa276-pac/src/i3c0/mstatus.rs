#[doc = "Register `MSTATUS` reader"]
pub type R = crate::R<MstatusSpec>;
#[doc = "Register `MSTATUS` writer"]
pub type W = crate::W<MstatusSpec>;
#[doc = "State of the Controller\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum State {
    #[doc = "0: IDLE (bus has stopped)"]
    Idle = 0,
    #[doc = "1: SLVREQ (target request)"]
    Slvreq = 1,
    #[doc = "2: MSGSDR"]
    Msgsdr = 2,
    #[doc = "3: NORMACT"]
    Normact = 3,
    #[doc = "4: MSGDDR"]
    Ddr = 4,
    #[doc = "5: DAA"]
    Daa = 5,
    #[doc = "6: IBIACK"]
    Ibiack = 6,
    #[doc = "7: IBIRCV"]
    Ibircv = 7,
}
impl From<State> for u8 {
    #[inline(always)]
    fn from(variant: State) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for State {
    type Ux = u8;
}
impl crate::IsEnum for State {}
#[doc = "Field `STATE` reader - State of the Controller"]
pub type StateR = crate::FieldReader<State>;
impl StateR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> State {
        match self.bits {
            0 => State::Idle,
            1 => State::Slvreq,
            2 => State::Msgsdr,
            3 => State::Normact,
            4 => State::Ddr,
            5 => State::Daa,
            6 => State::Ibiack,
            7 => State::Ibircv,
            _ => unreachable!(),
        }
    }
    #[doc = "IDLE (bus has stopped)"]
    #[inline(always)]
    pub fn is_idle(&self) -> bool {
        *self == State::Idle
    }
    #[doc = "SLVREQ (target request)"]
    #[inline(always)]
    pub fn is_slvreq(&self) -> bool {
        *self == State::Slvreq
    }
    #[doc = "MSGSDR"]
    #[inline(always)]
    pub fn is_msgsdr(&self) -> bool {
        *self == State::Msgsdr
    }
    #[doc = "NORMACT"]
    #[inline(always)]
    pub fn is_normact(&self) -> bool {
        *self == State::Normact
    }
    #[doc = "MSGDDR"]
    #[inline(always)]
    pub fn is_ddr(&self) -> bool {
        *self == State::Ddr
    }
    #[doc = "DAA"]
    #[inline(always)]
    pub fn is_daa(&self) -> bool {
        *self == State::Daa
    }
    #[doc = "IBIACK"]
    #[inline(always)]
    pub fn is_ibiack(&self) -> bool {
        *self == State::Ibiack
    }
    #[doc = "IBIRCV"]
    #[inline(always)]
    pub fn is_ibircv(&self) -> bool {
        *self == State::Ibircv
    }
}
#[doc = "Between\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Between {
    #[doc = "0: Inactive (for other cases)"]
    Inactive = 0,
    #[doc = "1: Active"]
    Active = 1,
}
impl From<Between> for bool {
    #[inline(always)]
    fn from(variant: Between) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BETWEEN` reader - Between"]
pub type BetweenR = crate::BitReader<Between>;
impl BetweenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Between {
        match self.bits {
            false => Between::Inactive,
            true => Between::Active,
        }
    }
    #[doc = "Inactive (for other cases)"]
    #[inline(always)]
    pub fn is_inactive(&self) -> bool {
        *self == Between::Inactive
    }
    #[doc = "Active"]
    #[inline(always)]
    pub fn is_active(&self) -> bool {
        *self == Between::Active
    }
}
#[doc = "Not Acknowledged\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nacked {
    #[doc = "0: Not NACKed"]
    NotNacked = 0,
    #[doc = "1: NACKed (not acknowledged)"]
    Nacked = 1,
}
impl From<Nacked> for bool {
    #[inline(always)]
    fn from(variant: Nacked) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NACKED` reader - Not Acknowledged"]
pub type NackedR = crate::BitReader<Nacked>;
impl NackedR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nacked {
        match self.bits {
            false => Nacked::NotNacked,
            true => Nacked::Nacked,
        }
    }
    #[doc = "Not NACKed"]
    #[inline(always)]
    pub fn is_not_nacked(&self) -> bool {
        *self == Nacked::NotNacked
    }
    #[doc = "NACKed (not acknowledged)"]
    #[inline(always)]
    pub fn is_nacked(&self) -> bool {
        *self == Nacked::Nacked
    }
}
#[doc = "In-Band Interrupt (IBI) Type\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Ibitype {
    #[doc = "0: NONE (no IBI: this status occurs when MSTATUS\\[IBIWON\\] becomes 0)"]
    None = 0,
    #[doc = "1: IBI"]
    Ibi = 1,
    #[doc = "2: CR"]
    Mr = 2,
    #[doc = "3: HJ"]
    Hj = 3,
}
impl From<Ibitype> for u8 {
    #[inline(always)]
    fn from(variant: Ibitype) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Ibitype {
    type Ux = u8;
}
impl crate::IsEnum for Ibitype {}
#[doc = "Field `IBITYPE` reader - In-Band Interrupt (IBI) Type"]
pub type IbitypeR = crate::FieldReader<Ibitype>;
impl IbitypeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ibitype {
        match self.bits {
            0 => Ibitype::None,
            1 => Ibitype::Ibi,
            2 => Ibitype::Mr,
            3 => Ibitype::Hj,
            _ => unreachable!(),
        }
    }
    #[doc = "NONE (no IBI: this status occurs when MSTATUS\\[IBIWON\\] becomes 0)"]
    #[inline(always)]
    pub fn is_none(&self) -> bool {
        *self == Ibitype::None
    }
    #[doc = "IBI"]
    #[inline(always)]
    pub fn is_ibi(&self) -> bool {
        *self == Ibitype::Ibi
    }
    #[doc = "CR"]
    #[inline(always)]
    pub fn is_mr(&self) -> bool {
        *self == Ibitype::Mr
    }
    #[doc = "HJ"]
    #[inline(always)]
    pub fn is_hj(&self) -> bool {
        *self == Ibitype::Hj
    }
}
#[doc = "Target Start Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slvstart {
    #[doc = "0: Target not requesting START"]
    NotStart = 0,
    #[doc = "1: Target requesting START"]
    Start = 1,
}
impl From<Slvstart> for bool {
    #[inline(always)]
    fn from(variant: Slvstart) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SLVSTART` reader - Target Start Flag"]
pub type SlvstartR = crate::BitReader<Slvstart>;
impl SlvstartR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Slvstart {
        match self.bits {
            false => Slvstart::NotStart,
            true => Slvstart::Start,
        }
    }
    #[doc = "Target not requesting START"]
    #[inline(always)]
    pub fn is_not_start(&self) -> bool {
        *self == Slvstart::NotStart
    }
    #[doc = "Target requesting START"]
    #[inline(always)]
    pub fn is_start(&self) -> bool {
        *self == Slvstart::Start
    }
}
#[doc = "Field `SLVSTART` writer - Target Start Flag"]
pub type SlvstartW<'a, REG> = crate::BitWriter1C<'a, REG, Slvstart>;
impl<'a, REG> SlvstartW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Target not requesting START"]
    #[inline(always)]
    pub fn not_start(self) -> &'a mut crate::W<REG> {
        self.variant(Slvstart::NotStart)
    }
    #[doc = "Target requesting START"]
    #[inline(always)]
    pub fn start(self) -> &'a mut crate::W<REG> {
        self.variant(Slvstart::Start)
    }
}
#[doc = "Controller Control Done Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mctrldone {
    #[doc = "0: Not done"]
    NotDone = 0,
    #[doc = "1: Done"]
    Done = 1,
}
impl From<Mctrldone> for bool {
    #[inline(always)]
    fn from(variant: Mctrldone) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MCTRLDONE` reader - Controller Control Done Flag"]
pub type MctrldoneR = crate::BitReader<Mctrldone>;
impl MctrldoneR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mctrldone {
        match self.bits {
            false => Mctrldone::NotDone,
            true => Mctrldone::Done,
        }
    }
    #[doc = "Not done"]
    #[inline(always)]
    pub fn is_not_done(&self) -> bool {
        *self == Mctrldone::NotDone
    }
    #[doc = "Done"]
    #[inline(always)]
    pub fn is_done(&self) -> bool {
        *self == Mctrldone::Done
    }
}
#[doc = "Field `MCTRLDONE` writer - Controller Control Done Flag"]
pub type MctrldoneW<'a, REG> = crate::BitWriter1C<'a, REG, Mctrldone>;
impl<'a, REG> MctrldoneW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not done"]
    #[inline(always)]
    pub fn not_done(self) -> &'a mut crate::W<REG> {
        self.variant(Mctrldone::NotDone)
    }
    #[doc = "Done"]
    #[inline(always)]
    pub fn done(self) -> &'a mut crate::W<REG> {
        self.variant(Mctrldone::Done)
    }
}
#[doc = "Complete Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Complete {
    #[doc = "0: Not complete"]
    NotComplete = 0,
    #[doc = "1: Complete"]
    Complete = 1,
}
impl From<Complete> for bool {
    #[inline(always)]
    fn from(variant: Complete) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `COMPLETE` reader - Complete Flag"]
pub type CompleteR = crate::BitReader<Complete>;
impl CompleteR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Complete {
        match self.bits {
            false => Complete::NotComplete,
            true => Complete::Complete,
        }
    }
    #[doc = "Not complete"]
    #[inline(always)]
    pub fn is_not_complete(&self) -> bool {
        *self == Complete::NotComplete
    }
    #[doc = "Complete"]
    #[inline(always)]
    pub fn is_complete(&self) -> bool {
        *self == Complete::Complete
    }
}
#[doc = "Field `COMPLETE` writer - Complete Flag"]
pub type CompleteW<'a, REG> = crate::BitWriter1C<'a, REG, Complete>;
impl<'a, REG> CompleteW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not complete"]
    #[inline(always)]
    pub fn not_complete(self) -> &'a mut crate::W<REG> {
        self.variant(Complete::NotComplete)
    }
    #[doc = "Complete"]
    #[inline(always)]
    pub fn complete(self) -> &'a mut crate::W<REG> {
        self.variant(Complete::Complete)
    }
}
#[doc = "RXPEND\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rxpend {
    #[doc = "0: No receive message pending"]
    Idle = 0,
    #[doc = "1: Receive message pending"]
    Pending = 1,
}
impl From<Rxpend> for bool {
    #[inline(always)]
    fn from(variant: Rxpend) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RXPEND` reader - RXPEND"]
pub type RxpendR = crate::BitReader<Rxpend>;
impl RxpendR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rxpend {
        match self.bits {
            false => Rxpend::Idle,
            true => Rxpend::Pending,
        }
    }
    #[doc = "No receive message pending"]
    #[inline(always)]
    pub fn is_idle(&self) -> bool {
        *self == Rxpend::Idle
    }
    #[doc = "Receive message pending"]
    #[inline(always)]
    pub fn is_pending(&self) -> bool {
        *self == Rxpend::Pending
    }
}
#[doc = "TX Buffer or FIFO Not Full\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Txnotfull {
    #[doc = "0: Receive buffer or FIFO full"]
    Full = 0,
    #[doc = "1: Receive buffer or FIFO not full"]
    Notfull = 1,
}
impl From<Txnotfull> for bool {
    #[inline(always)]
    fn from(variant: Txnotfull) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TXNOTFULL` reader - TX Buffer or FIFO Not Full"]
pub type TxnotfullR = crate::BitReader<Txnotfull>;
impl TxnotfullR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Txnotfull {
        match self.bits {
            false => Txnotfull::Full,
            true => Txnotfull::Notfull,
        }
    }
    #[doc = "Receive buffer or FIFO full"]
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        *self == Txnotfull::Full
    }
    #[doc = "Receive buffer or FIFO not full"]
    #[inline(always)]
    pub fn is_notfull(&self) -> bool {
        *self == Txnotfull::Notfull
    }
}
#[doc = "In-Band Interrupt (IBI) Won Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ibiwon {
    #[doc = "0: No IBI arbitration won"]
    NotWon = 0,
    #[doc = "1: IBI arbitration won"]
    Won = 1,
}
impl From<Ibiwon> for bool {
    #[inline(always)]
    fn from(variant: Ibiwon) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `IBIWON` reader - In-Band Interrupt (IBI) Won Flag"]
pub type IbiwonR = crate::BitReader<Ibiwon>;
impl IbiwonR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ibiwon {
        match self.bits {
            false => Ibiwon::NotWon,
            true => Ibiwon::Won,
        }
    }
    #[doc = "No IBI arbitration won"]
    #[inline(always)]
    pub fn is_not_won(&self) -> bool {
        *self == Ibiwon::NotWon
    }
    #[doc = "IBI arbitration won"]
    #[inline(always)]
    pub fn is_won(&self) -> bool {
        *self == Ibiwon::Won
    }
}
#[doc = "Field `IBIWON` writer - In-Band Interrupt (IBI) Won Flag"]
pub type IbiwonW<'a, REG> = crate::BitWriter1C<'a, REG, Ibiwon>;
impl<'a, REG> IbiwonW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No IBI arbitration won"]
    #[inline(always)]
    pub fn not_won(self) -> &'a mut crate::W<REG> {
        self.variant(Ibiwon::NotWon)
    }
    #[doc = "IBI arbitration won"]
    #[inline(always)]
    pub fn won(self) -> &'a mut crate::W<REG> {
        self.variant(Ibiwon::Won)
    }
}
#[doc = "Error or Warning\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Errwarn {
    #[doc = "0: No error or warning"]
    NoError = 0,
    #[doc = "1: Error or warning"]
    Error = 1,
}
impl From<Errwarn> for bool {
    #[inline(always)]
    fn from(variant: Errwarn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERRWARN` reader - Error or Warning"]
pub type ErrwarnR = crate::BitReader<Errwarn>;
impl ErrwarnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Errwarn {
        match self.bits {
            false => Errwarn::NoError,
            true => Errwarn::Error,
        }
    }
    #[doc = "No error or warning"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Errwarn::NoError
    }
    #[doc = "Error or warning"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Errwarn::Error
    }
}
#[doc = "Module is now Controller Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nowmaster {
    #[doc = "0: Not a controller"]
    NotMaster = 0,
    #[doc = "1: Controller"]
    Master = 1,
}
impl From<Nowmaster> for bool {
    #[inline(always)]
    fn from(variant: Nowmaster) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NOWMASTER` reader - Module is now Controller Flag"]
pub type NowmasterR = crate::BitReader<Nowmaster>;
impl NowmasterR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nowmaster {
        match self.bits {
            false => Nowmaster::NotMaster,
            true => Nowmaster::Master,
        }
    }
    #[doc = "Not a controller"]
    #[inline(always)]
    pub fn is_not_master(&self) -> bool {
        *self == Nowmaster::NotMaster
    }
    #[doc = "Controller"]
    #[inline(always)]
    pub fn is_master(&self) -> bool {
        *self == Nowmaster::Master
    }
}
#[doc = "Field `NOWMASTER` writer - Module is now Controller Flag"]
pub type NowmasterW<'a, REG> = crate::BitWriter1C<'a, REG, Nowmaster>;
impl<'a, REG> NowmasterW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not a controller"]
    #[inline(always)]
    pub fn not_master(self) -> &'a mut crate::W<REG> {
        self.variant(Nowmaster::NotMaster)
    }
    #[doc = "Controller"]
    #[inline(always)]
    pub fn master(self) -> &'a mut crate::W<REG> {
        self.variant(Nowmaster::Master)
    }
}
#[doc = "Field `IBIADDR` reader - IBI Address"]
pub type IbiaddrR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:2 - State of the Controller"]
    #[inline(always)]
    pub fn state(&self) -> StateR {
        StateR::new((self.bits & 7) as u8)
    }
    #[doc = "Bit 4 - Between"]
    #[inline(always)]
    pub fn between(&self) -> BetweenR {
        BetweenR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Not Acknowledged"]
    #[inline(always)]
    pub fn nacked(&self) -> NackedR {
        NackedR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bits 6:7 - In-Band Interrupt (IBI) Type"]
    #[inline(always)]
    pub fn ibitype(&self) -> IbitypeR {
        IbitypeR::new(((self.bits >> 6) & 3) as u8)
    }
    #[doc = "Bit 8 - Target Start Flag"]
    #[inline(always)]
    pub fn slvstart(&self) -> SlvstartR {
        SlvstartR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Controller Control Done Flag"]
    #[inline(always)]
    pub fn mctrldone(&self) -> MctrldoneR {
        MctrldoneR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Complete Flag"]
    #[inline(always)]
    pub fn complete(&self) -> CompleteR {
        CompleteR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - RXPEND"]
    #[inline(always)]
    pub fn rxpend(&self) -> RxpendR {
        RxpendR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - TX Buffer or FIFO Not Full"]
    #[inline(always)]
    pub fn txnotfull(&self) -> TxnotfullR {
        TxnotfullR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - In-Band Interrupt (IBI) Won Flag"]
    #[inline(always)]
    pub fn ibiwon(&self) -> IbiwonR {
        IbiwonR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 15 - Error or Warning"]
    #[inline(always)]
    pub fn errwarn(&self) -> ErrwarnR {
        ErrwarnR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 19 - Module is now Controller Flag"]
    #[inline(always)]
    pub fn nowmaster(&self) -> NowmasterR {
        NowmasterR::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bits 24:30 - IBI Address"]
    #[inline(always)]
    pub fn ibiaddr(&self) -> IbiaddrR {
        IbiaddrR::new(((self.bits >> 24) & 0x7f) as u8)
    }
}
impl W {
    #[doc = "Bit 8 - Target Start Flag"]
    #[inline(always)]
    pub fn slvstart(&mut self) -> SlvstartW<MstatusSpec> {
        SlvstartW::new(self, 8)
    }
    #[doc = "Bit 9 - Controller Control Done Flag"]
    #[inline(always)]
    pub fn mctrldone(&mut self) -> MctrldoneW<MstatusSpec> {
        MctrldoneW::new(self, 9)
    }
    #[doc = "Bit 10 - Complete Flag"]
    #[inline(always)]
    pub fn complete(&mut self) -> CompleteW<MstatusSpec> {
        CompleteW::new(self, 10)
    }
    #[doc = "Bit 13 - In-Band Interrupt (IBI) Won Flag"]
    #[inline(always)]
    pub fn ibiwon(&mut self) -> IbiwonW<MstatusSpec> {
        IbiwonW::new(self, 13)
    }
    #[doc = "Bit 19 - Module is now Controller Flag"]
    #[inline(always)]
    pub fn nowmaster(&mut self) -> NowmasterW<MstatusSpec> {
        NowmasterW::new(self, 19)
    }
}
#[doc = "Controller Status\n\nYou can [`read`](crate::Reg::read) this register and get [`mstatus::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mstatus::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MstatusSpec;
impl crate::RegisterSpec for MstatusSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mstatus::R`](R) reader structure"]
impl crate::Readable for MstatusSpec {}
#[doc = "`write(|w| ..)` method takes [`mstatus::W`](W) writer structure"]
impl crate::Writable for MstatusSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x0008_2700;
}
#[doc = "`reset()` method sets MSTATUS to value 0x1000"]
impl crate::Resettable for MstatusSpec {
    const RESET_VALUE: u32 = 0x1000;
}

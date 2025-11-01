#[doc = "Register `STAT` reader"]
pub type R = crate::R<StatSpec>;
#[doc = "Register `STAT` writer"]
pub type W = crate::W<StatSpec>;
#[doc = "LIN Break Flag Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lbkfe {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Lbkfe> for bool {
    #[inline(always)]
    fn from(variant: Lbkfe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LBKFE` reader - LIN Break Flag Enable"]
pub type LbkfeR = crate::BitReader<Lbkfe>;
impl LbkfeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lbkfe {
        match self.bits {
            false => Lbkfe::Disabled,
            true => Lbkfe::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Lbkfe::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Lbkfe::Enabled
    }
}
#[doc = "Field `LBKFE` writer - LIN Break Flag Enable"]
pub type LbkfeW<'a, REG> = crate::BitWriter<'a, REG, Lbkfe>;
impl<'a, REG> LbkfeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lbkfe::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lbkfe::Enabled)
    }
}
#[doc = "Address Mark Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ame {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Ame> for bool {
    #[inline(always)]
    fn from(variant: Ame) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `AME` reader - Address Mark Enable"]
pub type AmeR = crate::BitReader<Ame>;
impl AmeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ame {
        match self.bits {
            false => Ame::Disabled,
            true => Ame::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Ame::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Ame::Enabled
    }
}
#[doc = "Field `AME` writer - Address Mark Enable"]
pub type AmeW<'a, REG> = crate::BitWriter<'a, REG, Ame>;
impl<'a, REG> AmeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ame::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ame::Enabled)
    }
}
#[doc = "Match 2 Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ma2f {
    #[doc = "0: Not equal to MA2"]
    Nomatch = 0,
    #[doc = "1: Equal to MA2"]
    Match = 1,
}
impl From<Ma2f> for bool {
    #[inline(always)]
    fn from(variant: Ma2f) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MA2F` reader - Match 2 Flag"]
pub type Ma2fR = crate::BitReader<Ma2f>;
impl Ma2fR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ma2f {
        match self.bits {
            false => Ma2f::Nomatch,
            true => Ma2f::Match,
        }
    }
    #[doc = "Not equal to MA2"]
    #[inline(always)]
    pub fn is_nomatch(&self) -> bool {
        *self == Ma2f::Nomatch
    }
    #[doc = "Equal to MA2"]
    #[inline(always)]
    pub fn is_match(&self) -> bool {
        *self == Ma2f::Match
    }
}
#[doc = "Field `MA2F` writer - Match 2 Flag"]
pub type Ma2fW<'a, REG> = crate::BitWriter1C<'a, REG, Ma2f>;
impl<'a, REG> Ma2fW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not equal to MA2"]
    #[inline(always)]
    pub fn nomatch(self) -> &'a mut crate::W<REG> {
        self.variant(Ma2f::Nomatch)
    }
    #[doc = "Equal to MA2"]
    #[inline(always)]
    pub fn match_(self) -> &'a mut crate::W<REG> {
        self.variant(Ma2f::Match)
    }
}
#[doc = "Match 1 Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ma1f {
    #[doc = "0: Not equal to MA1"]
    Nomatch = 0,
    #[doc = "1: Equal to MA1"]
    Match = 1,
}
impl From<Ma1f> for bool {
    #[inline(always)]
    fn from(variant: Ma1f) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MA1F` reader - Match 1 Flag"]
pub type Ma1fR = crate::BitReader<Ma1f>;
impl Ma1fR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ma1f {
        match self.bits {
            false => Ma1f::Nomatch,
            true => Ma1f::Match,
        }
    }
    #[doc = "Not equal to MA1"]
    #[inline(always)]
    pub fn is_nomatch(&self) -> bool {
        *self == Ma1f::Nomatch
    }
    #[doc = "Equal to MA1"]
    #[inline(always)]
    pub fn is_match(&self) -> bool {
        *self == Ma1f::Match
    }
}
#[doc = "Field `MA1F` writer - Match 1 Flag"]
pub type Ma1fW<'a, REG> = crate::BitWriter1C<'a, REG, Ma1f>;
impl<'a, REG> Ma1fW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not equal to MA1"]
    #[inline(always)]
    pub fn nomatch(self) -> &'a mut crate::W<REG> {
        self.variant(Ma1f::Nomatch)
    }
    #[doc = "Equal to MA1"]
    #[inline(always)]
    pub fn match_(self) -> &'a mut crate::W<REG> {
        self.variant(Ma1f::Match)
    }
}
#[doc = "Parity Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pf {
    #[doc = "0: No parity error detected"]
    Noparity = 0,
    #[doc = "1: Parity error detected"]
    Parity = 1,
}
impl From<Pf> for bool {
    #[inline(always)]
    fn from(variant: Pf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PF` reader - Parity Error Flag"]
pub type PfR = crate::BitReader<Pf>;
impl PfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pf {
        match self.bits {
            false => Pf::Noparity,
            true => Pf::Parity,
        }
    }
    #[doc = "No parity error detected"]
    #[inline(always)]
    pub fn is_noparity(&self) -> bool {
        *self == Pf::Noparity
    }
    #[doc = "Parity error detected"]
    #[inline(always)]
    pub fn is_parity(&self) -> bool {
        *self == Pf::Parity
    }
}
#[doc = "Field `PF` writer - Parity Error Flag"]
pub type PfW<'a, REG> = crate::BitWriter1C<'a, REG, Pf>;
impl<'a, REG> PfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No parity error detected"]
    #[inline(always)]
    pub fn noparity(self) -> &'a mut crate::W<REG> {
        self.variant(Pf::Noparity)
    }
    #[doc = "Parity error detected"]
    #[inline(always)]
    pub fn parity(self) -> &'a mut crate::W<REG> {
        self.variant(Pf::Parity)
    }
}
#[doc = "Framing Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fe {
    #[doc = "0: No framing error detected (this does not guarantee that the framing is correct)"]
    Noerror = 0,
    #[doc = "1: Framing error detected"]
    Error = 1,
}
impl From<Fe> for bool {
    #[inline(always)]
    fn from(variant: Fe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FE` reader - Framing Error Flag"]
pub type FeR = crate::BitReader<Fe>;
impl FeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fe {
        match self.bits {
            false => Fe::Noerror,
            true => Fe::Error,
        }
    }
    #[doc = "No framing error detected (this does not guarantee that the framing is correct)"]
    #[inline(always)]
    pub fn is_noerror(&self) -> bool {
        *self == Fe::Noerror
    }
    #[doc = "Framing error detected"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Fe::Error
    }
}
#[doc = "Field `FE` writer - Framing Error Flag"]
pub type FeW<'a, REG> = crate::BitWriter1C<'a, REG, Fe>;
impl<'a, REG> FeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No framing error detected (this does not guarantee that the framing is correct)"]
    #[inline(always)]
    pub fn noerror(self) -> &'a mut crate::W<REG> {
        self.variant(Fe::Noerror)
    }
    #[doc = "Framing error detected"]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Fe::Error)
    }
}
#[doc = "Noise Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nf {
    #[doc = "0: No noise detected"]
    Nonoise = 0,
    #[doc = "1: Noise detected"]
    Noise = 1,
}
impl From<Nf> for bool {
    #[inline(always)]
    fn from(variant: Nf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NF` reader - Noise Flag"]
pub type NfR = crate::BitReader<Nf>;
impl NfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nf {
        match self.bits {
            false => Nf::Nonoise,
            true => Nf::Noise,
        }
    }
    #[doc = "No noise detected"]
    #[inline(always)]
    pub fn is_nonoise(&self) -> bool {
        *self == Nf::Nonoise
    }
    #[doc = "Noise detected"]
    #[inline(always)]
    pub fn is_noise(&self) -> bool {
        *self == Nf::Noise
    }
}
#[doc = "Field `NF` writer - Noise Flag"]
pub type NfW<'a, REG> = crate::BitWriter1C<'a, REG, Nf>;
impl<'a, REG> NfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No noise detected"]
    #[inline(always)]
    pub fn nonoise(self) -> &'a mut crate::W<REG> {
        self.variant(Nf::Nonoise)
    }
    #[doc = "Noise detected"]
    #[inline(always)]
    pub fn noise(self) -> &'a mut crate::W<REG> {
        self.variant(Nf::Noise)
    }
}
#[doc = "Receiver Overrun Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Or {
    #[doc = "0: No overrun"]
    NoOverrun = 0,
    #[doc = "1: Receive overrun (new LPUART data is lost)"]
    Overrun = 1,
}
impl From<Or> for bool {
    #[inline(always)]
    fn from(variant: Or) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OR` reader - Receiver Overrun Flag"]
pub type OrR = crate::BitReader<Or>;
impl OrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Or {
        match self.bits {
            false => Or::NoOverrun,
            true => Or::Overrun,
        }
    }
    #[doc = "No overrun"]
    #[inline(always)]
    pub fn is_no_overrun(&self) -> bool {
        *self == Or::NoOverrun
    }
    #[doc = "Receive overrun (new LPUART data is lost)"]
    #[inline(always)]
    pub fn is_overrun(&self) -> bool {
        *self == Or::Overrun
    }
}
#[doc = "Field `OR` writer - Receiver Overrun Flag"]
pub type OrW<'a, REG> = crate::BitWriter1C<'a, REG, Or>;
impl<'a, REG> OrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No overrun"]
    #[inline(always)]
    pub fn no_overrun(self) -> &'a mut crate::W<REG> {
        self.variant(Or::NoOverrun)
    }
    #[doc = "Receive overrun (new LPUART data is lost)"]
    #[inline(always)]
    pub fn overrun(self) -> &'a mut crate::W<REG> {
        self.variant(Or::Overrun)
    }
}
#[doc = "Idle Line Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Idle {
    #[doc = "0: Idle line detected"]
    Noidle = 0,
    #[doc = "1: Idle line not detected"]
    Idle = 1,
}
impl From<Idle> for bool {
    #[inline(always)]
    fn from(variant: Idle) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `IDLE` reader - Idle Line Flag"]
pub type IdleR = crate::BitReader<Idle>;
impl IdleR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Idle {
        match self.bits {
            false => Idle::Noidle,
            true => Idle::Idle,
        }
    }
    #[doc = "Idle line detected"]
    #[inline(always)]
    pub fn is_noidle(&self) -> bool {
        *self == Idle::Noidle
    }
    #[doc = "Idle line not detected"]
    #[inline(always)]
    pub fn is_idle(&self) -> bool {
        *self == Idle::Idle
    }
}
#[doc = "Field `IDLE` writer - Idle Line Flag"]
pub type IdleW<'a, REG> = crate::BitWriter1C<'a, REG, Idle>;
impl<'a, REG> IdleW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Idle line detected"]
    #[inline(always)]
    pub fn noidle(self) -> &'a mut crate::W<REG> {
        self.variant(Idle::Noidle)
    }
    #[doc = "Idle line not detected"]
    #[inline(always)]
    pub fn idle(self) -> &'a mut crate::W<REG> {
        self.variant(Idle::Idle)
    }
}
#[doc = "Receive Data Register Full Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rdrf {
    #[doc = "0: Equal to or less than watermark"]
    NoRxdata = 0,
    #[doc = "1: Greater than watermark"]
    Rxdata = 1,
}
impl From<Rdrf> for bool {
    #[inline(always)]
    fn from(variant: Rdrf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RDRF` reader - Receive Data Register Full Flag"]
pub type RdrfR = crate::BitReader<Rdrf>;
impl RdrfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rdrf {
        match self.bits {
            false => Rdrf::NoRxdata,
            true => Rdrf::Rxdata,
        }
    }
    #[doc = "Equal to or less than watermark"]
    #[inline(always)]
    pub fn is_no_rxdata(&self) -> bool {
        *self == Rdrf::NoRxdata
    }
    #[doc = "Greater than watermark"]
    #[inline(always)]
    pub fn is_rxdata(&self) -> bool {
        *self == Rdrf::Rxdata
    }
}
#[doc = "Transmission Complete Flag\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tc {
    #[doc = "0: Transmitter active"]
    Active = 0,
    #[doc = "1: Transmitter idle"]
    Complete = 1,
}
impl From<Tc> for bool {
    #[inline(always)]
    fn from(variant: Tc) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TC` reader - Transmission Complete Flag"]
pub type TcR = crate::BitReader<Tc>;
impl TcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tc {
        match self.bits {
            false => Tc::Active,
            true => Tc::Complete,
        }
    }
    #[doc = "Transmitter active"]
    #[inline(always)]
    pub fn is_active(&self) -> bool {
        *self == Tc::Active
    }
    #[doc = "Transmitter idle"]
    #[inline(always)]
    pub fn is_complete(&self) -> bool {
        *self == Tc::Complete
    }
}
#[doc = "Transmit Data Register Empty Flag\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tdre {
    #[doc = "0: Greater than watermark"]
    Txdata = 0,
    #[doc = "1: Equal to or less than watermark"]
    NoTxdata = 1,
}
impl From<Tdre> for bool {
    #[inline(always)]
    fn from(variant: Tdre) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TDRE` reader - Transmit Data Register Empty Flag"]
pub type TdreR = crate::BitReader<Tdre>;
impl TdreR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tdre {
        match self.bits {
            false => Tdre::Txdata,
            true => Tdre::NoTxdata,
        }
    }
    #[doc = "Greater than watermark"]
    #[inline(always)]
    pub fn is_txdata(&self) -> bool {
        *self == Tdre::Txdata
    }
    #[doc = "Equal to or less than watermark"]
    #[inline(always)]
    pub fn is_no_txdata(&self) -> bool {
        *self == Tdre::NoTxdata
    }
}
#[doc = "Receiver Active Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Raf {
    #[doc = "0: Idle, waiting for a start bit"]
    Idle = 0,
    #[doc = "1: Receiver active (RXD pin input not idle)"]
    Active = 1,
}
impl From<Raf> for bool {
    #[inline(always)]
    fn from(variant: Raf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RAF` reader - Receiver Active Flag"]
pub type RafR = crate::BitReader<Raf>;
impl RafR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Raf {
        match self.bits {
            false => Raf::Idle,
            true => Raf::Active,
        }
    }
    #[doc = "Idle, waiting for a start bit"]
    #[inline(always)]
    pub fn is_idle(&self) -> bool {
        *self == Raf::Idle
    }
    #[doc = "Receiver active (RXD pin input not idle)"]
    #[inline(always)]
    pub fn is_active(&self) -> bool {
        *self == Raf::Active
    }
}
#[doc = "LIN Break Detection Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lbkde {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Lbkde> for bool {
    #[inline(always)]
    fn from(variant: Lbkde) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LBKDE` reader - LIN Break Detection Enable"]
pub type LbkdeR = crate::BitReader<Lbkde>;
impl LbkdeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lbkde {
        match self.bits {
            false => Lbkde::Disabled,
            true => Lbkde::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Lbkde::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Lbkde::Enabled
    }
}
#[doc = "Field `LBKDE` writer - LIN Break Detection Enable"]
pub type LbkdeW<'a, REG> = crate::BitWriter<'a, REG, Lbkde>;
impl<'a, REG> LbkdeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lbkde::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lbkde::Enabled)
    }
}
#[doc = "Break Character Generation Length\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Brk13 {
    #[doc = "0: 9 to 13 bit times"]
    Short = 0,
    #[doc = "1: 12 to 15 bit times"]
    Long = 1,
}
impl From<Brk13> for bool {
    #[inline(always)]
    fn from(variant: Brk13) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BRK13` reader - Break Character Generation Length"]
pub type Brk13R = crate::BitReader<Brk13>;
impl Brk13R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Brk13 {
        match self.bits {
            false => Brk13::Short,
            true => Brk13::Long,
        }
    }
    #[doc = "9 to 13 bit times"]
    #[inline(always)]
    pub fn is_short(&self) -> bool {
        *self == Brk13::Short
    }
    #[doc = "12 to 15 bit times"]
    #[inline(always)]
    pub fn is_long(&self) -> bool {
        *self == Brk13::Long
    }
}
#[doc = "Field `BRK13` writer - Break Character Generation Length"]
pub type Brk13W<'a, REG> = crate::BitWriter<'a, REG, Brk13>;
impl<'a, REG> Brk13W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "9 to 13 bit times"]
    #[inline(always)]
    pub fn short(self) -> &'a mut crate::W<REG> {
        self.variant(Brk13::Short)
    }
    #[doc = "12 to 15 bit times"]
    #[inline(always)]
    pub fn long(self) -> &'a mut crate::W<REG> {
        self.variant(Brk13::Long)
    }
}
#[doc = "Receive Wake Up Idle Detect\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rwuid {
    #[doc = "0: STAT\\[IDLE\\] does not become 1"]
    IdleNotset = 0,
    #[doc = "1: STAT\\[IDLE\\] becomes 1"]
    IdleSet = 1,
}
impl From<Rwuid> for bool {
    #[inline(always)]
    fn from(variant: Rwuid) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RWUID` reader - Receive Wake Up Idle Detect"]
pub type RwuidR = crate::BitReader<Rwuid>;
impl RwuidR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rwuid {
        match self.bits {
            false => Rwuid::IdleNotset,
            true => Rwuid::IdleSet,
        }
    }
    #[doc = "STAT\\[IDLE\\] does not become 1"]
    #[inline(always)]
    pub fn is_idle_notset(&self) -> bool {
        *self == Rwuid::IdleNotset
    }
    #[doc = "STAT\\[IDLE\\] becomes 1"]
    #[inline(always)]
    pub fn is_idle_set(&self) -> bool {
        *self == Rwuid::IdleSet
    }
}
#[doc = "Field `RWUID` writer - Receive Wake Up Idle Detect"]
pub type RwuidW<'a, REG> = crate::BitWriter<'a, REG, Rwuid>;
impl<'a, REG> RwuidW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "STAT\\[IDLE\\] does not become 1"]
    #[inline(always)]
    pub fn idle_notset(self) -> &'a mut crate::W<REG> {
        self.variant(Rwuid::IdleNotset)
    }
    #[doc = "STAT\\[IDLE\\] becomes 1"]
    #[inline(always)]
    pub fn idle_set(self) -> &'a mut crate::W<REG> {
        self.variant(Rwuid::IdleSet)
    }
}
#[doc = "Receive Data Inversion\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rxinv {
    #[doc = "0: Inverted"]
    NotInverted = 0,
    #[doc = "1: Not inverted"]
    Inverted = 1,
}
impl From<Rxinv> for bool {
    #[inline(always)]
    fn from(variant: Rxinv) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RXINV` reader - Receive Data Inversion"]
pub type RxinvR = crate::BitReader<Rxinv>;
impl RxinvR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rxinv {
        match self.bits {
            false => Rxinv::NotInverted,
            true => Rxinv::Inverted,
        }
    }
    #[doc = "Inverted"]
    #[inline(always)]
    pub fn is_not_inverted(&self) -> bool {
        *self == Rxinv::NotInverted
    }
    #[doc = "Not inverted"]
    #[inline(always)]
    pub fn is_inverted(&self) -> bool {
        *self == Rxinv::Inverted
    }
}
#[doc = "Field `RXINV` writer - Receive Data Inversion"]
pub type RxinvW<'a, REG> = crate::BitWriter<'a, REG, Rxinv>;
impl<'a, REG> RxinvW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Inverted"]
    #[inline(always)]
    pub fn not_inverted(self) -> &'a mut crate::W<REG> {
        self.variant(Rxinv::NotInverted)
    }
    #[doc = "Not inverted"]
    #[inline(always)]
    pub fn inverted(self) -> &'a mut crate::W<REG> {
        self.variant(Rxinv::Inverted)
    }
}
#[doc = "MSB First\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Msbf {
    #[doc = "0: LSB"]
    LsbFirst = 0,
    #[doc = "1: MSB"]
    MsbFirst = 1,
}
impl From<Msbf> for bool {
    #[inline(always)]
    fn from(variant: Msbf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MSBF` reader - MSB First"]
pub type MsbfR = crate::BitReader<Msbf>;
impl MsbfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Msbf {
        match self.bits {
            false => Msbf::LsbFirst,
            true => Msbf::MsbFirst,
        }
    }
    #[doc = "LSB"]
    #[inline(always)]
    pub fn is_lsb_first(&self) -> bool {
        *self == Msbf::LsbFirst
    }
    #[doc = "MSB"]
    #[inline(always)]
    pub fn is_msb_first(&self) -> bool {
        *self == Msbf::MsbFirst
    }
}
#[doc = "Field `MSBF` writer - MSB First"]
pub type MsbfW<'a, REG> = crate::BitWriter<'a, REG, Msbf>;
impl<'a, REG> MsbfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "LSB"]
    #[inline(always)]
    pub fn lsb_first(self) -> &'a mut crate::W<REG> {
        self.variant(Msbf::LsbFirst)
    }
    #[doc = "MSB"]
    #[inline(always)]
    pub fn msb_first(self) -> &'a mut crate::W<REG> {
        self.variant(Msbf::MsbFirst)
    }
}
#[doc = "RXD Pin Active Edge Interrupt Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rxedgif {
    #[doc = "0: Not occurred"]
    NoEdge = 0,
    #[doc = "1: Occurred"]
    Edge = 1,
}
impl From<Rxedgif> for bool {
    #[inline(always)]
    fn from(variant: Rxedgif) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RXEDGIF` reader - RXD Pin Active Edge Interrupt Flag"]
pub type RxedgifR = crate::BitReader<Rxedgif>;
impl RxedgifR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rxedgif {
        match self.bits {
            false => Rxedgif::NoEdge,
            true => Rxedgif::Edge,
        }
    }
    #[doc = "Not occurred"]
    #[inline(always)]
    pub fn is_no_edge(&self) -> bool {
        *self == Rxedgif::NoEdge
    }
    #[doc = "Occurred"]
    #[inline(always)]
    pub fn is_edge(&self) -> bool {
        *self == Rxedgif::Edge
    }
}
#[doc = "Field `RXEDGIF` writer - RXD Pin Active Edge Interrupt Flag"]
pub type RxedgifW<'a, REG> = crate::BitWriter1C<'a, REG, Rxedgif>;
impl<'a, REG> RxedgifW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not occurred"]
    #[inline(always)]
    pub fn no_edge(self) -> &'a mut crate::W<REG> {
        self.variant(Rxedgif::NoEdge)
    }
    #[doc = "Occurred"]
    #[inline(always)]
    pub fn edge(self) -> &'a mut crate::W<REG> {
        self.variant(Rxedgif::Edge)
    }
}
#[doc = "LIN Break Detect Interrupt Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lbkdif {
    #[doc = "0: Not detected"]
    NotDetected = 0,
    #[doc = "1: Detected"]
    Detected = 1,
}
impl From<Lbkdif> for bool {
    #[inline(always)]
    fn from(variant: Lbkdif) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LBKDIF` reader - LIN Break Detect Interrupt Flag"]
pub type LbkdifR = crate::BitReader<Lbkdif>;
impl LbkdifR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lbkdif {
        match self.bits {
            false => Lbkdif::NotDetected,
            true => Lbkdif::Detected,
        }
    }
    #[doc = "Not detected"]
    #[inline(always)]
    pub fn is_not_detected(&self) -> bool {
        *self == Lbkdif::NotDetected
    }
    #[doc = "Detected"]
    #[inline(always)]
    pub fn is_detected(&self) -> bool {
        *self == Lbkdif::Detected
    }
}
#[doc = "Field `LBKDIF` writer - LIN Break Detect Interrupt Flag"]
pub type LbkdifW<'a, REG> = crate::BitWriter1C<'a, REG, Lbkdif>;
impl<'a, REG> LbkdifW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not detected"]
    #[inline(always)]
    pub fn not_detected(self) -> &'a mut crate::W<REG> {
        self.variant(Lbkdif::NotDetected)
    }
    #[doc = "Detected"]
    #[inline(always)]
    pub fn detected(self) -> &'a mut crate::W<REG> {
        self.variant(Lbkdif::Detected)
    }
}
impl R {
    #[doc = "Bit 0 - LIN Break Flag Enable"]
    #[inline(always)]
    pub fn lbkfe(&self) -> LbkfeR {
        LbkfeR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Address Mark Enable"]
    #[inline(always)]
    pub fn ame(&self) -> AmeR {
        AmeR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 14 - Match 2 Flag"]
    #[inline(always)]
    pub fn ma2f(&self) -> Ma2fR {
        Ma2fR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Match 1 Flag"]
    #[inline(always)]
    pub fn ma1f(&self) -> Ma1fR {
        Ma1fR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - Parity Error Flag"]
    #[inline(always)]
    pub fn pf(&self) -> PfR {
        PfR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Framing Error Flag"]
    #[inline(always)]
    pub fn fe(&self) -> FeR {
        FeR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Noise Flag"]
    #[inline(always)]
    pub fn nf(&self) -> NfR {
        NfR::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - Receiver Overrun Flag"]
    #[inline(always)]
    pub fn or(&self) -> OrR {
        OrR::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - Idle Line Flag"]
    #[inline(always)]
    pub fn idle(&self) -> IdleR {
        IdleR::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - Receive Data Register Full Flag"]
    #[inline(always)]
    pub fn rdrf(&self) -> RdrfR {
        RdrfR::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - Transmission Complete Flag"]
    #[inline(always)]
    pub fn tc(&self) -> TcR {
        TcR::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - Transmit Data Register Empty Flag"]
    #[inline(always)]
    pub fn tdre(&self) -> TdreR {
        TdreR::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bit 24 - Receiver Active Flag"]
    #[inline(always)]
    pub fn raf(&self) -> RafR {
        RafR::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - LIN Break Detection Enable"]
    #[inline(always)]
    pub fn lbkde(&self) -> LbkdeR {
        LbkdeR::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - Break Character Generation Length"]
    #[inline(always)]
    pub fn brk13(&self) -> Brk13R {
        Brk13R::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - Receive Wake Up Idle Detect"]
    #[inline(always)]
    pub fn rwuid(&self) -> RwuidR {
        RwuidR::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bit 28 - Receive Data Inversion"]
    #[inline(always)]
    pub fn rxinv(&self) -> RxinvR {
        RxinvR::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 29 - MSB First"]
    #[inline(always)]
    pub fn msbf(&self) -> MsbfR {
        MsbfR::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - RXD Pin Active Edge Interrupt Flag"]
    #[inline(always)]
    pub fn rxedgif(&self) -> RxedgifR {
        RxedgifR::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - LIN Break Detect Interrupt Flag"]
    #[inline(always)]
    pub fn lbkdif(&self) -> LbkdifR {
        LbkdifR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - LIN Break Flag Enable"]
    #[inline(always)]
    pub fn lbkfe(&mut self) -> LbkfeW<StatSpec> {
        LbkfeW::new(self, 0)
    }
    #[doc = "Bit 1 - Address Mark Enable"]
    #[inline(always)]
    pub fn ame(&mut self) -> AmeW<StatSpec> {
        AmeW::new(self, 1)
    }
    #[doc = "Bit 14 - Match 2 Flag"]
    #[inline(always)]
    pub fn ma2f(&mut self) -> Ma2fW<StatSpec> {
        Ma2fW::new(self, 14)
    }
    #[doc = "Bit 15 - Match 1 Flag"]
    #[inline(always)]
    pub fn ma1f(&mut self) -> Ma1fW<StatSpec> {
        Ma1fW::new(self, 15)
    }
    #[doc = "Bit 16 - Parity Error Flag"]
    #[inline(always)]
    pub fn pf(&mut self) -> PfW<StatSpec> {
        PfW::new(self, 16)
    }
    #[doc = "Bit 17 - Framing Error Flag"]
    #[inline(always)]
    pub fn fe(&mut self) -> FeW<StatSpec> {
        FeW::new(self, 17)
    }
    #[doc = "Bit 18 - Noise Flag"]
    #[inline(always)]
    pub fn nf(&mut self) -> NfW<StatSpec> {
        NfW::new(self, 18)
    }
    #[doc = "Bit 19 - Receiver Overrun Flag"]
    #[inline(always)]
    pub fn or(&mut self) -> OrW<StatSpec> {
        OrW::new(self, 19)
    }
    #[doc = "Bit 20 - Idle Line Flag"]
    #[inline(always)]
    pub fn idle(&mut self) -> IdleW<StatSpec> {
        IdleW::new(self, 20)
    }
    #[doc = "Bit 25 - LIN Break Detection Enable"]
    #[inline(always)]
    pub fn lbkde(&mut self) -> LbkdeW<StatSpec> {
        LbkdeW::new(self, 25)
    }
    #[doc = "Bit 26 - Break Character Generation Length"]
    #[inline(always)]
    pub fn brk13(&mut self) -> Brk13W<StatSpec> {
        Brk13W::new(self, 26)
    }
    #[doc = "Bit 27 - Receive Wake Up Idle Detect"]
    #[inline(always)]
    pub fn rwuid(&mut self) -> RwuidW<StatSpec> {
        RwuidW::new(self, 27)
    }
    #[doc = "Bit 28 - Receive Data Inversion"]
    #[inline(always)]
    pub fn rxinv(&mut self) -> RxinvW<StatSpec> {
        RxinvW::new(self, 28)
    }
    #[doc = "Bit 29 - MSB First"]
    #[inline(always)]
    pub fn msbf(&mut self) -> MsbfW<StatSpec> {
        MsbfW::new(self, 29)
    }
    #[doc = "Bit 30 - RXD Pin Active Edge Interrupt Flag"]
    #[inline(always)]
    pub fn rxedgif(&mut self) -> RxedgifW<StatSpec> {
        RxedgifW::new(self, 30)
    }
    #[doc = "Bit 31 - LIN Break Detect Interrupt Flag"]
    #[inline(always)]
    pub fn lbkdif(&mut self) -> LbkdifW<StatSpec> {
        LbkdifW::new(self, 31)
    }
}
#[doc = "Status\n\nYou can [`read`](crate::Reg::read) this register and get [`stat::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`stat::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct StatSpec;
impl crate::RegisterSpec for StatSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`stat::R`](R) reader structure"]
impl crate::Readable for StatSpec {}
#[doc = "`write(|w| ..)` method takes [`stat::W`](W) writer structure"]
impl crate::Writable for StatSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0xc01f_c000;
}
#[doc = "`reset()` method sets STAT to value 0x00c0_0000"]
impl crate::Resettable for StatSpec {
    const RESET_VALUE: u32 = 0x00c0_0000;
}

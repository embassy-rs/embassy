#[doc = "Register `CTRL` reader"]
pub type R = crate::R<CtrlSpec>;
#[doc = "Register `CTRL` writer"]
pub type W = crate::W<CtrlSpec>;
#[doc = "Parity Type\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pt {
    #[doc = "0: Even parity"]
    Even = 0,
    #[doc = "1: Odd parity"]
    Odd = 1,
}
impl From<Pt> for bool {
    #[inline(always)]
    fn from(variant: Pt) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PT` reader - Parity Type"]
pub type PtR = crate::BitReader<Pt>;
impl PtR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pt {
        match self.bits {
            false => Pt::Even,
            true => Pt::Odd,
        }
    }
    #[doc = "Even parity"]
    #[inline(always)]
    pub fn is_even(&self) -> bool {
        *self == Pt::Even
    }
    #[doc = "Odd parity"]
    #[inline(always)]
    pub fn is_odd(&self) -> bool {
        *self == Pt::Odd
    }
}
#[doc = "Field `PT` writer - Parity Type"]
pub type PtW<'a, REG> = crate::BitWriter<'a, REG, Pt>;
impl<'a, REG> PtW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Even parity"]
    #[inline(always)]
    pub fn even(self) -> &'a mut crate::W<REG> {
        self.variant(Pt::Even)
    }
    #[doc = "Odd parity"]
    #[inline(always)]
    pub fn odd(self) -> &'a mut crate::W<REG> {
        self.variant(Pt::Odd)
    }
}
#[doc = "Parity Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pe {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Pe> for bool {
    #[inline(always)]
    fn from(variant: Pe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PE` reader - Parity Enable"]
pub type PeR = crate::BitReader<Pe>;
impl PeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pe {
        match self.bits {
            false => Pe::Disabled,
            true => Pe::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Pe::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Pe::Enabled
    }
}
#[doc = "Field `PE` writer - Parity Enable"]
pub type PeW<'a, REG> = crate::BitWriter<'a, REG, Pe>;
impl<'a, REG> PeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Pe::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Pe::Enabled)
    }
}
#[doc = "Idle Line Type Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ilt {
    #[doc = "0: After the start bit"]
    FromStart = 0,
    #[doc = "1: After the stop bit"]
    FromStop = 1,
}
impl From<Ilt> for bool {
    #[inline(always)]
    fn from(variant: Ilt) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ILT` reader - Idle Line Type Select"]
pub type IltR = crate::BitReader<Ilt>;
impl IltR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ilt {
        match self.bits {
            false => Ilt::FromStart,
            true => Ilt::FromStop,
        }
    }
    #[doc = "After the start bit"]
    #[inline(always)]
    pub fn is_from_start(&self) -> bool {
        *self == Ilt::FromStart
    }
    #[doc = "After the stop bit"]
    #[inline(always)]
    pub fn is_from_stop(&self) -> bool {
        *self == Ilt::FromStop
    }
}
#[doc = "Field `ILT` writer - Idle Line Type Select"]
pub type IltW<'a, REG> = crate::BitWriter<'a, REG, Ilt>;
impl<'a, REG> IltW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "After the start bit"]
    #[inline(always)]
    pub fn from_start(self) -> &'a mut crate::W<REG> {
        self.variant(Ilt::FromStart)
    }
    #[doc = "After the stop bit"]
    #[inline(always)]
    pub fn from_stop(self) -> &'a mut crate::W<REG> {
        self.variant(Ilt::FromStop)
    }
}
#[doc = "Receiver Wake-Up Method Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wake {
    #[doc = "0: Idle"]
    Idle = 0,
    #[doc = "1: Mark"]
    Mark = 1,
}
impl From<Wake> for bool {
    #[inline(always)]
    fn from(variant: Wake) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WAKE` reader - Receiver Wake-Up Method Select"]
pub type WakeR = crate::BitReader<Wake>;
impl WakeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wake {
        match self.bits {
            false => Wake::Idle,
            true => Wake::Mark,
        }
    }
    #[doc = "Idle"]
    #[inline(always)]
    pub fn is_idle(&self) -> bool {
        *self == Wake::Idle
    }
    #[doc = "Mark"]
    #[inline(always)]
    pub fn is_mark(&self) -> bool {
        *self == Wake::Mark
    }
}
#[doc = "Field `WAKE` writer - Receiver Wake-Up Method Select"]
pub type WakeW<'a, REG> = crate::BitWriter<'a, REG, Wake>;
impl<'a, REG> WakeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Idle"]
    #[inline(always)]
    pub fn idle(self) -> &'a mut crate::W<REG> {
        self.variant(Wake::Idle)
    }
    #[doc = "Mark"]
    #[inline(always)]
    pub fn mark(self) -> &'a mut crate::W<REG> {
        self.variant(Wake::Mark)
    }
}
#[doc = "9-Bit Or 8-Bit Mode Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum M {
    #[doc = "0: 8-bit"]
    Data8 = 0,
    #[doc = "1: 9-bit"]
    Data9 = 1,
}
impl From<M> for bool {
    #[inline(always)]
    fn from(variant: M) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `M` reader - 9-Bit Or 8-Bit Mode Select"]
pub type MR = crate::BitReader<M>;
impl MR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> M {
        match self.bits {
            false => M::Data8,
            true => M::Data9,
        }
    }
    #[doc = "8-bit"]
    #[inline(always)]
    pub fn is_data8(&self) -> bool {
        *self == M::Data8
    }
    #[doc = "9-bit"]
    #[inline(always)]
    pub fn is_data9(&self) -> bool {
        *self == M::Data9
    }
}
#[doc = "Field `M` writer - 9-Bit Or 8-Bit Mode Select"]
pub type MW<'a, REG> = crate::BitWriter<'a, REG, M>;
impl<'a, REG> MW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "8-bit"]
    #[inline(always)]
    pub fn data8(self) -> &'a mut crate::W<REG> {
        self.variant(M::Data8)
    }
    #[doc = "9-bit"]
    #[inline(always)]
    pub fn data9(self) -> &'a mut crate::W<REG> {
        self.variant(M::Data9)
    }
}
#[doc = "Receiver Source Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rsrc {
    #[doc = "0: Internal Loopback mode"]
    NoEffect = 0,
    #[doc = "1: Single-wire mode"]
    Onewire = 1,
}
impl From<Rsrc> for bool {
    #[inline(always)]
    fn from(variant: Rsrc) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RSRC` reader - Receiver Source Select"]
pub type RsrcR = crate::BitReader<Rsrc>;
impl RsrcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rsrc {
        match self.bits {
            false => Rsrc::NoEffect,
            true => Rsrc::Onewire,
        }
    }
    #[doc = "Internal Loopback mode"]
    #[inline(always)]
    pub fn is_no_effect(&self) -> bool {
        *self == Rsrc::NoEffect
    }
    #[doc = "Single-wire mode"]
    #[inline(always)]
    pub fn is_onewire(&self) -> bool {
        *self == Rsrc::Onewire
    }
}
#[doc = "Field `RSRC` writer - Receiver Source Select"]
pub type RsrcW<'a, REG> = crate::BitWriter<'a, REG, Rsrc>;
impl<'a, REG> RsrcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Internal Loopback mode"]
    #[inline(always)]
    pub fn no_effect(self) -> &'a mut crate::W<REG> {
        self.variant(Rsrc::NoEffect)
    }
    #[doc = "Single-wire mode"]
    #[inline(always)]
    pub fn onewire(self) -> &'a mut crate::W<REG> {
        self.variant(Rsrc::Onewire)
    }
}
#[doc = "Doze Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dozeen {
    #[doc = "0: Enable"]
    Enabled = 0,
    #[doc = "1: Disable"]
    Disabled = 1,
}
impl From<Dozeen> for bool {
    #[inline(always)]
    fn from(variant: Dozeen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DOZEEN` reader - Doze Mode"]
pub type DozeenR = crate::BitReader<Dozeen>;
impl DozeenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dozeen {
        match self.bits {
            false => Dozeen::Enabled,
            true => Dozeen::Disabled,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Dozeen::Enabled
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Dozeen::Disabled
    }
}
#[doc = "Field `DOZEEN` writer - Doze Mode"]
pub type DozeenW<'a, REG> = crate::BitWriter<'a, REG, Dozeen>;
impl<'a, REG> DozeenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Dozeen::Enabled)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Dozeen::Disabled)
    }
}
#[doc = "Loop Mode Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Loops {
    #[doc = "0: Normal operation: RXD and TXD use separate pins"]
    Noffect = 0,
    #[doc = "1: Loop mode or Single-Wire mode"]
    Loopback = 1,
}
impl From<Loops> for bool {
    #[inline(always)]
    fn from(variant: Loops) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LOOPS` reader - Loop Mode Select"]
pub type LoopsR = crate::BitReader<Loops>;
impl LoopsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Loops {
        match self.bits {
            false => Loops::Noffect,
            true => Loops::Loopback,
        }
    }
    #[doc = "Normal operation: RXD and TXD use separate pins"]
    #[inline(always)]
    pub fn is_noffect(&self) -> bool {
        *self == Loops::Noffect
    }
    #[doc = "Loop mode or Single-Wire mode"]
    #[inline(always)]
    pub fn is_loopback(&self) -> bool {
        *self == Loops::Loopback
    }
}
#[doc = "Field `LOOPS` writer - Loop Mode Select"]
pub type LoopsW<'a, REG> = crate::BitWriter<'a, REG, Loops>;
impl<'a, REG> LoopsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Normal operation: RXD and TXD use separate pins"]
    #[inline(always)]
    pub fn noffect(self) -> &'a mut crate::W<REG> {
        self.variant(Loops::Noffect)
    }
    #[doc = "Loop mode or Single-Wire mode"]
    #[inline(always)]
    pub fn loopback(self) -> &'a mut crate::W<REG> {
        self.variant(Loops::Loopback)
    }
}
#[doc = "Idle Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Idlecfg {
    #[doc = "0: 1"]
    Idle1 = 0,
    #[doc = "1: 2"]
    Idle2 = 1,
    #[doc = "2: 4"]
    Idle4 = 2,
    #[doc = "3: 8"]
    Idle8 = 3,
    #[doc = "4: 16"]
    Idle16 = 4,
    #[doc = "5: 32"]
    Idle32 = 5,
    #[doc = "6: 64"]
    Idle64 = 6,
    #[doc = "7: 128"]
    Idle128 = 7,
}
impl From<Idlecfg> for u8 {
    #[inline(always)]
    fn from(variant: Idlecfg) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Idlecfg {
    type Ux = u8;
}
impl crate::IsEnum for Idlecfg {}
#[doc = "Field `IDLECFG` reader - Idle Configuration"]
pub type IdlecfgR = crate::FieldReader<Idlecfg>;
impl IdlecfgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Idlecfg {
        match self.bits {
            0 => Idlecfg::Idle1,
            1 => Idlecfg::Idle2,
            2 => Idlecfg::Idle4,
            3 => Idlecfg::Idle8,
            4 => Idlecfg::Idle16,
            5 => Idlecfg::Idle32,
            6 => Idlecfg::Idle64,
            7 => Idlecfg::Idle128,
            _ => unreachable!(),
        }
    }
    #[doc = "1"]
    #[inline(always)]
    pub fn is_idle_1(&self) -> bool {
        *self == Idlecfg::Idle1
    }
    #[doc = "2"]
    #[inline(always)]
    pub fn is_idle_2(&self) -> bool {
        *self == Idlecfg::Idle2
    }
    #[doc = "4"]
    #[inline(always)]
    pub fn is_idle_4(&self) -> bool {
        *self == Idlecfg::Idle4
    }
    #[doc = "8"]
    #[inline(always)]
    pub fn is_idle_8(&self) -> bool {
        *self == Idlecfg::Idle8
    }
    #[doc = "16"]
    #[inline(always)]
    pub fn is_idle_16(&self) -> bool {
        *self == Idlecfg::Idle16
    }
    #[doc = "32"]
    #[inline(always)]
    pub fn is_idle_32(&self) -> bool {
        *self == Idlecfg::Idle32
    }
    #[doc = "64"]
    #[inline(always)]
    pub fn is_idle_64(&self) -> bool {
        *self == Idlecfg::Idle64
    }
    #[doc = "128"]
    #[inline(always)]
    pub fn is_idle_128(&self) -> bool {
        *self == Idlecfg::Idle128
    }
}
#[doc = "Field `IDLECFG` writer - Idle Configuration"]
pub type IdlecfgW<'a, REG> = crate::FieldWriter<'a, REG, 3, Idlecfg, crate::Safe>;
impl<'a, REG> IdlecfgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "1"]
    #[inline(always)]
    pub fn idle_1(self) -> &'a mut crate::W<REG> {
        self.variant(Idlecfg::Idle1)
    }
    #[doc = "2"]
    #[inline(always)]
    pub fn idle_2(self) -> &'a mut crate::W<REG> {
        self.variant(Idlecfg::Idle2)
    }
    #[doc = "4"]
    #[inline(always)]
    pub fn idle_4(self) -> &'a mut crate::W<REG> {
        self.variant(Idlecfg::Idle4)
    }
    #[doc = "8"]
    #[inline(always)]
    pub fn idle_8(self) -> &'a mut crate::W<REG> {
        self.variant(Idlecfg::Idle8)
    }
    #[doc = "16"]
    #[inline(always)]
    pub fn idle_16(self) -> &'a mut crate::W<REG> {
        self.variant(Idlecfg::Idle16)
    }
    #[doc = "32"]
    #[inline(always)]
    pub fn idle_32(self) -> &'a mut crate::W<REG> {
        self.variant(Idlecfg::Idle32)
    }
    #[doc = "64"]
    #[inline(always)]
    pub fn idle_64(self) -> &'a mut crate::W<REG> {
        self.variant(Idlecfg::Idle64)
    }
    #[doc = "128"]
    #[inline(always)]
    pub fn idle_128(self) -> &'a mut crate::W<REG> {
        self.variant(Idlecfg::Idle128)
    }
}
#[doc = "7-Bit Mode Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum M7 {
    #[doc = "0: 8-bit to 10-bit"]
    NoEffect = 0,
    #[doc = "1: 7-bit"]
    Data7 = 1,
}
impl From<M7> for bool {
    #[inline(always)]
    fn from(variant: M7) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `M7` reader - 7-Bit Mode Select"]
pub type M7R = crate::BitReader<M7>;
impl M7R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> M7 {
        match self.bits {
            false => M7::NoEffect,
            true => M7::Data7,
        }
    }
    #[doc = "8-bit to 10-bit"]
    #[inline(always)]
    pub fn is_no_effect(&self) -> bool {
        *self == M7::NoEffect
    }
    #[doc = "7-bit"]
    #[inline(always)]
    pub fn is_data7(&self) -> bool {
        *self == M7::Data7
    }
}
#[doc = "Field `M7` writer - 7-Bit Mode Select"]
pub type M7W<'a, REG> = crate::BitWriter<'a, REG, M7>;
impl<'a, REG> M7W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "8-bit to 10-bit"]
    #[inline(always)]
    pub fn no_effect(self) -> &'a mut crate::W<REG> {
        self.variant(M7::NoEffect)
    }
    #[doc = "7-bit"]
    #[inline(always)]
    pub fn data7(self) -> &'a mut crate::W<REG> {
        self.variant(M7::Data7)
    }
}
#[doc = "TXD and RXD Pin Swap\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Swap {
    #[doc = "0: Use the standard way"]
    Standard = 0,
    #[doc = "1: Swap"]
    Swap = 1,
}
impl From<Swap> for bool {
    #[inline(always)]
    fn from(variant: Swap) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SWAP` reader - TXD and RXD Pin Swap"]
pub type SwapR = crate::BitReader<Swap>;
impl SwapR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Swap {
        match self.bits {
            false => Swap::Standard,
            true => Swap::Swap,
        }
    }
    #[doc = "Use the standard way"]
    #[inline(always)]
    pub fn is_standard(&self) -> bool {
        *self == Swap::Standard
    }
    #[doc = "Swap"]
    #[inline(always)]
    pub fn is_swap(&self) -> bool {
        *self == Swap::Swap
    }
}
#[doc = "Field `SWAP` writer - TXD and RXD Pin Swap"]
pub type SwapW<'a, REG> = crate::BitWriter<'a, REG, Swap>;
impl<'a, REG> SwapW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Use the standard way"]
    #[inline(always)]
    pub fn standard(self) -> &'a mut crate::W<REG> {
        self.variant(Swap::Standard)
    }
    #[doc = "Swap"]
    #[inline(always)]
    pub fn swap(self) -> &'a mut crate::W<REG> {
        self.variant(Swap::Swap)
    }
}
#[doc = "Match 2 (MA2F) Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ma2ie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Ma2ie> for bool {
    #[inline(always)]
    fn from(variant: Ma2ie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MA2IE` reader - Match 2 (MA2F) Interrupt Enable"]
pub type Ma2ieR = crate::BitReader<Ma2ie>;
impl Ma2ieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ma2ie {
        match self.bits {
            false => Ma2ie::Disabled,
            true => Ma2ie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Ma2ie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Ma2ie::Enabled
    }
}
#[doc = "Field `MA2IE` writer - Match 2 (MA2F) Interrupt Enable"]
pub type Ma2ieW<'a, REG> = crate::BitWriter<'a, REG, Ma2ie>;
impl<'a, REG> Ma2ieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ma2ie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ma2ie::Enabled)
    }
}
#[doc = "Match 1 (MA1F) Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ma1ie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Ma1ie> for bool {
    #[inline(always)]
    fn from(variant: Ma1ie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MA1IE` reader - Match 1 (MA1F) Interrupt Enable"]
pub type Ma1ieR = crate::BitReader<Ma1ie>;
impl Ma1ieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ma1ie {
        match self.bits {
            false => Ma1ie::Disabled,
            true => Ma1ie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Ma1ie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Ma1ie::Enabled
    }
}
#[doc = "Field `MA1IE` writer - Match 1 (MA1F) Interrupt Enable"]
pub type Ma1ieW<'a, REG> = crate::BitWriter<'a, REG, Ma1ie>;
impl<'a, REG> Ma1ieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ma1ie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ma1ie::Enabled)
    }
}
#[doc = "Send Break\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sbk {
    #[doc = "0: Normal transmitter operation"]
    NoEffect = 0,
    #[doc = "1: Queue break character(s) to be sent"]
    TxBreak = 1,
}
impl From<Sbk> for bool {
    #[inline(always)]
    fn from(variant: Sbk) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SBK` reader - Send Break"]
pub type SbkR = crate::BitReader<Sbk>;
impl SbkR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sbk {
        match self.bits {
            false => Sbk::NoEffect,
            true => Sbk::TxBreak,
        }
    }
    #[doc = "Normal transmitter operation"]
    #[inline(always)]
    pub fn is_no_effect(&self) -> bool {
        *self == Sbk::NoEffect
    }
    #[doc = "Queue break character(s) to be sent"]
    #[inline(always)]
    pub fn is_tx_break(&self) -> bool {
        *self == Sbk::TxBreak
    }
}
#[doc = "Field `SBK` writer - Send Break"]
pub type SbkW<'a, REG> = crate::BitWriter<'a, REG, Sbk>;
impl<'a, REG> SbkW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Normal transmitter operation"]
    #[inline(always)]
    pub fn no_effect(self) -> &'a mut crate::W<REG> {
        self.variant(Sbk::NoEffect)
    }
    #[doc = "Queue break character(s) to be sent"]
    #[inline(always)]
    pub fn tx_break(self) -> &'a mut crate::W<REG> {
        self.variant(Sbk::TxBreak)
    }
}
#[doc = "Receiver Wake-Up Control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rwu {
    #[doc = "0: Normal receiver operation"]
    NoEffect = 0,
    #[doc = "1: LPUART receiver in standby, waiting for a wake-up condition"]
    RxWakeup = 1,
}
impl From<Rwu> for bool {
    #[inline(always)]
    fn from(variant: Rwu) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RWU` reader - Receiver Wake-Up Control"]
pub type RwuR = crate::BitReader<Rwu>;
impl RwuR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rwu {
        match self.bits {
            false => Rwu::NoEffect,
            true => Rwu::RxWakeup,
        }
    }
    #[doc = "Normal receiver operation"]
    #[inline(always)]
    pub fn is_no_effect(&self) -> bool {
        *self == Rwu::NoEffect
    }
    #[doc = "LPUART receiver in standby, waiting for a wake-up condition"]
    #[inline(always)]
    pub fn is_rx_wakeup(&self) -> bool {
        *self == Rwu::RxWakeup
    }
}
#[doc = "Field `RWU` writer - Receiver Wake-Up Control"]
pub type RwuW<'a, REG> = crate::BitWriter<'a, REG, Rwu>;
impl<'a, REG> RwuW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Normal receiver operation"]
    #[inline(always)]
    pub fn no_effect(self) -> &'a mut crate::W<REG> {
        self.variant(Rwu::NoEffect)
    }
    #[doc = "LPUART receiver in standby, waiting for a wake-up condition"]
    #[inline(always)]
    pub fn rx_wakeup(self) -> &'a mut crate::W<REG> {
        self.variant(Rwu::RxWakeup)
    }
}
#[doc = "Receiver Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Re {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Re> for bool {
    #[inline(always)]
    fn from(variant: Re) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RE` reader - Receiver Enable"]
pub type ReR = crate::BitReader<Re>;
impl ReR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Re {
        match self.bits {
            false => Re::Disabled,
            true => Re::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Re::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Re::Enabled
    }
}
#[doc = "Field `RE` writer - Receiver Enable"]
pub type ReW<'a, REG> = crate::BitWriter<'a, REG, Re>;
impl<'a, REG> ReW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Re::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Re::Enabled)
    }
}
#[doc = "Transmitter Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Te {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Te> for bool {
    #[inline(always)]
    fn from(variant: Te) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TE` reader - Transmitter Enable"]
pub type TeR = crate::BitReader<Te>;
impl TeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Te {
        match self.bits {
            false => Te::Disabled,
            true => Te::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Te::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Te::Enabled
    }
}
#[doc = "Field `TE` writer - Transmitter Enable"]
pub type TeW<'a, REG> = crate::BitWriter<'a, REG, Te>;
impl<'a, REG> TeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Te::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Te::Enabled)
    }
}
#[doc = "Idle Line Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ilie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Ilie> for bool {
    #[inline(always)]
    fn from(variant: Ilie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ILIE` reader - Idle Line Interrupt Enable"]
pub type IlieR = crate::BitReader<Ilie>;
impl IlieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ilie {
        match self.bits {
            false => Ilie::Disabled,
            true => Ilie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Ilie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Ilie::Enabled
    }
}
#[doc = "Field `ILIE` writer - Idle Line Interrupt Enable"]
pub type IlieW<'a, REG> = crate::BitWriter<'a, REG, Ilie>;
impl<'a, REG> IlieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ilie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ilie::Enabled)
    }
}
#[doc = "Receiver Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Rie> for bool {
    #[inline(always)]
    fn from(variant: Rie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RIE` reader - Receiver Interrupt Enable"]
pub type RieR = crate::BitReader<Rie>;
impl RieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rie {
        match self.bits {
            false => Rie::Disabled,
            true => Rie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Rie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Rie::Enabled
    }
}
#[doc = "Field `RIE` writer - Receiver Interrupt Enable"]
pub type RieW<'a, REG> = crate::BitWriter<'a, REG, Rie>;
impl<'a, REG> RieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rie::Enabled)
    }
}
#[doc = "Transmission Complete Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tcie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Tcie> for bool {
    #[inline(always)]
    fn from(variant: Tcie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TCIE` reader - Transmission Complete Interrupt Enable"]
pub type TcieR = crate::BitReader<Tcie>;
impl TcieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tcie {
        match self.bits {
            false => Tcie::Disabled,
            true => Tcie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Tcie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Tcie::Enabled
    }
}
#[doc = "Field `TCIE` writer - Transmission Complete Interrupt Enable"]
pub type TcieW<'a, REG> = crate::BitWriter<'a, REG, Tcie>;
impl<'a, REG> TcieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Tcie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Tcie::Enabled)
    }
}
#[doc = "Transmit Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Tie> for bool {
    #[inline(always)]
    fn from(variant: Tie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIE` reader - Transmit Interrupt Enable"]
pub type TieR = crate::BitReader<Tie>;
impl TieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tie {
        match self.bits {
            false => Tie::Disabled,
            true => Tie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Tie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Tie::Enabled
    }
}
#[doc = "Field `TIE` writer - Transmit Interrupt Enable"]
pub type TieW<'a, REG> = crate::BitWriter<'a, REG, Tie>;
impl<'a, REG> TieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Tie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Tie::Enabled)
    }
}
#[doc = "Parity Error Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Peie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Peie> for bool {
    #[inline(always)]
    fn from(variant: Peie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PEIE` reader - Parity Error Interrupt Enable"]
pub type PeieR = crate::BitReader<Peie>;
impl PeieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Peie {
        match self.bits {
            false => Peie::Disabled,
            true => Peie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Peie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Peie::Enabled
    }
}
#[doc = "Field `PEIE` writer - Parity Error Interrupt Enable"]
pub type PeieW<'a, REG> = crate::BitWriter<'a, REG, Peie>;
impl<'a, REG> PeieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Peie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Peie::Enabled)
    }
}
#[doc = "Framing Error Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Feie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Feie> for bool {
    #[inline(always)]
    fn from(variant: Feie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FEIE` reader - Framing Error Interrupt Enable"]
pub type FeieR = crate::BitReader<Feie>;
impl FeieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Feie {
        match self.bits {
            false => Feie::Disabled,
            true => Feie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Feie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Feie::Enabled
    }
}
#[doc = "Field `FEIE` writer - Framing Error Interrupt Enable"]
pub type FeieW<'a, REG> = crate::BitWriter<'a, REG, Feie>;
impl<'a, REG> FeieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Feie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Feie::Enabled)
    }
}
#[doc = "Noise Error Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Neie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Neie> for bool {
    #[inline(always)]
    fn from(variant: Neie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NEIE` reader - Noise Error Interrupt Enable"]
pub type NeieR = crate::BitReader<Neie>;
impl NeieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Neie {
        match self.bits {
            false => Neie::Disabled,
            true => Neie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Neie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Neie::Enabled
    }
}
#[doc = "Field `NEIE` writer - Noise Error Interrupt Enable"]
pub type NeieW<'a, REG> = crate::BitWriter<'a, REG, Neie>;
impl<'a, REG> NeieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Neie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Neie::Enabled)
    }
}
#[doc = "Overrun Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Orie> for bool {
    #[inline(always)]
    fn from(variant: Orie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ORIE` reader - Overrun Interrupt Enable"]
pub type OrieR = crate::BitReader<Orie>;
impl OrieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Orie {
        match self.bits {
            false => Orie::Disabled,
            true => Orie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Orie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Orie::Enabled
    }
}
#[doc = "Field `ORIE` writer - Overrun Interrupt Enable"]
pub type OrieW<'a, REG> = crate::BitWriter<'a, REG, Orie>;
impl<'a, REG> OrieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Orie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Orie::Enabled)
    }
}
#[doc = "Transmit Data Inversion\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Txinv {
    #[doc = "0: Not inverted"]
    NotInverted = 0,
    #[doc = "1: Inverted"]
    Inverted = 1,
}
impl From<Txinv> for bool {
    #[inline(always)]
    fn from(variant: Txinv) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TXINV` reader - Transmit Data Inversion"]
pub type TxinvR = crate::BitReader<Txinv>;
impl TxinvR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Txinv {
        match self.bits {
            false => Txinv::NotInverted,
            true => Txinv::Inverted,
        }
    }
    #[doc = "Not inverted"]
    #[inline(always)]
    pub fn is_not_inverted(&self) -> bool {
        *self == Txinv::NotInverted
    }
    #[doc = "Inverted"]
    #[inline(always)]
    pub fn is_inverted(&self) -> bool {
        *self == Txinv::Inverted
    }
}
#[doc = "Field `TXINV` writer - Transmit Data Inversion"]
pub type TxinvW<'a, REG> = crate::BitWriter<'a, REG, Txinv>;
impl<'a, REG> TxinvW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not inverted"]
    #[inline(always)]
    pub fn not_inverted(self) -> &'a mut crate::W<REG> {
        self.variant(Txinv::NotInverted)
    }
    #[doc = "Inverted"]
    #[inline(always)]
    pub fn inverted(self) -> &'a mut crate::W<REG> {
        self.variant(Txinv::Inverted)
    }
}
#[doc = "TXD Pin Direction in Single-Wire Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Txdir {
    #[doc = "0: Input"]
    TxInput = 0,
    #[doc = "1: Output"]
    TxOutput = 1,
}
impl From<Txdir> for bool {
    #[inline(always)]
    fn from(variant: Txdir) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TXDIR` reader - TXD Pin Direction in Single-Wire Mode"]
pub type TxdirR = crate::BitReader<Txdir>;
impl TxdirR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Txdir {
        match self.bits {
            false => Txdir::TxInput,
            true => Txdir::TxOutput,
        }
    }
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_tx_input(&self) -> bool {
        *self == Txdir::TxInput
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn is_tx_output(&self) -> bool {
        *self == Txdir::TxOutput
    }
}
#[doc = "Field `TXDIR` writer - TXD Pin Direction in Single-Wire Mode"]
pub type TxdirW<'a, REG> = crate::BitWriter<'a, REG, Txdir>;
impl<'a, REG> TxdirW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input"]
    #[inline(always)]
    pub fn tx_input(self) -> &'a mut crate::W<REG> {
        self.variant(Txdir::TxInput)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn tx_output(self) -> &'a mut crate::W<REG> {
        self.variant(Txdir::TxOutput)
    }
}
#[doc = "Field `R9T8` reader - Receive Bit 9 Transmit Bit 8"]
pub type R9t8R = crate::BitReader;
#[doc = "Field `R9T8` writer - Receive Bit 9 Transmit Bit 8"]
pub type R9t8W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `R8T9` reader - Receive Bit 8 Transmit Bit 9"]
pub type R8t9R = crate::BitReader;
#[doc = "Field `R8T9` writer - Receive Bit 8 Transmit Bit 9"]
pub type R8t9W<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bit 0 - Parity Type"]
    #[inline(always)]
    pub fn pt(&self) -> PtR {
        PtR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Parity Enable"]
    #[inline(always)]
    pub fn pe(&self) -> PeR {
        PeR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Idle Line Type Select"]
    #[inline(always)]
    pub fn ilt(&self) -> IltR {
        IltR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Receiver Wake-Up Method Select"]
    #[inline(always)]
    pub fn wake(&self) -> WakeR {
        WakeR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - 9-Bit Or 8-Bit Mode Select"]
    #[inline(always)]
    pub fn m(&self) -> MR {
        MR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Receiver Source Select"]
    #[inline(always)]
    pub fn rsrc(&self) -> RsrcR {
        RsrcR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Doze Mode"]
    #[inline(always)]
    pub fn dozeen(&self) -> DozeenR {
        DozeenR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Loop Mode Select"]
    #[inline(always)]
    pub fn loops(&self) -> LoopsR {
        LoopsR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bits 8:10 - Idle Configuration"]
    #[inline(always)]
    pub fn idlecfg(&self) -> IdlecfgR {
        IdlecfgR::new(((self.bits >> 8) & 7) as u8)
    }
    #[doc = "Bit 11 - 7-Bit Mode Select"]
    #[inline(always)]
    pub fn m7(&self) -> M7R {
        M7R::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - TXD and RXD Pin Swap"]
    #[inline(always)]
    pub fn swap(&self) -> SwapR {
        SwapR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 14 - Match 2 (MA2F) Interrupt Enable"]
    #[inline(always)]
    pub fn ma2ie(&self) -> Ma2ieR {
        Ma2ieR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Match 1 (MA1F) Interrupt Enable"]
    #[inline(always)]
    pub fn ma1ie(&self) -> Ma1ieR {
        Ma1ieR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - Send Break"]
    #[inline(always)]
    pub fn sbk(&self) -> SbkR {
        SbkR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Receiver Wake-Up Control"]
    #[inline(always)]
    pub fn rwu(&self) -> RwuR {
        RwuR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Receiver Enable"]
    #[inline(always)]
    pub fn re(&self) -> ReR {
        ReR::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - Transmitter Enable"]
    #[inline(always)]
    pub fn te(&self) -> TeR {
        TeR::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - Idle Line Interrupt Enable"]
    #[inline(always)]
    pub fn ilie(&self) -> IlieR {
        IlieR::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - Receiver Interrupt Enable"]
    #[inline(always)]
    pub fn rie(&self) -> RieR {
        RieR::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - Transmission Complete Interrupt Enable"]
    #[inline(always)]
    pub fn tcie(&self) -> TcieR {
        TcieR::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - Transmit Interrupt Enable"]
    #[inline(always)]
    pub fn tie(&self) -> TieR {
        TieR::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bit 24 - Parity Error Interrupt Enable"]
    #[inline(always)]
    pub fn peie(&self) -> PeieR {
        PeieR::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - Framing Error Interrupt Enable"]
    #[inline(always)]
    pub fn feie(&self) -> FeieR {
        FeieR::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - Noise Error Interrupt Enable"]
    #[inline(always)]
    pub fn neie(&self) -> NeieR {
        NeieR::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - Overrun Interrupt Enable"]
    #[inline(always)]
    pub fn orie(&self) -> OrieR {
        OrieR::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bit 28 - Transmit Data Inversion"]
    #[inline(always)]
    pub fn txinv(&self) -> TxinvR {
        TxinvR::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 29 - TXD Pin Direction in Single-Wire Mode"]
    #[inline(always)]
    pub fn txdir(&self) -> TxdirR {
        TxdirR::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - Receive Bit 9 Transmit Bit 8"]
    #[inline(always)]
    pub fn r9t8(&self) -> R9t8R {
        R9t8R::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Receive Bit 8 Transmit Bit 9"]
    #[inline(always)]
    pub fn r8t9(&self) -> R8t9R {
        R8t9R::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Parity Type"]
    #[inline(always)]
    pub fn pt(&mut self) -> PtW<CtrlSpec> {
        PtW::new(self, 0)
    }
    #[doc = "Bit 1 - Parity Enable"]
    #[inline(always)]
    pub fn pe(&mut self) -> PeW<CtrlSpec> {
        PeW::new(self, 1)
    }
    #[doc = "Bit 2 - Idle Line Type Select"]
    #[inline(always)]
    pub fn ilt(&mut self) -> IltW<CtrlSpec> {
        IltW::new(self, 2)
    }
    #[doc = "Bit 3 - Receiver Wake-Up Method Select"]
    #[inline(always)]
    pub fn wake(&mut self) -> WakeW<CtrlSpec> {
        WakeW::new(self, 3)
    }
    #[doc = "Bit 4 - 9-Bit Or 8-Bit Mode Select"]
    #[inline(always)]
    pub fn m(&mut self) -> MW<CtrlSpec> {
        MW::new(self, 4)
    }
    #[doc = "Bit 5 - Receiver Source Select"]
    #[inline(always)]
    pub fn rsrc(&mut self) -> RsrcW<CtrlSpec> {
        RsrcW::new(self, 5)
    }
    #[doc = "Bit 6 - Doze Mode"]
    #[inline(always)]
    pub fn dozeen(&mut self) -> DozeenW<CtrlSpec> {
        DozeenW::new(self, 6)
    }
    #[doc = "Bit 7 - Loop Mode Select"]
    #[inline(always)]
    pub fn loops(&mut self) -> LoopsW<CtrlSpec> {
        LoopsW::new(self, 7)
    }
    #[doc = "Bits 8:10 - Idle Configuration"]
    #[inline(always)]
    pub fn idlecfg(&mut self) -> IdlecfgW<CtrlSpec> {
        IdlecfgW::new(self, 8)
    }
    #[doc = "Bit 11 - 7-Bit Mode Select"]
    #[inline(always)]
    pub fn m7(&mut self) -> M7W<CtrlSpec> {
        M7W::new(self, 11)
    }
    #[doc = "Bit 12 - TXD and RXD Pin Swap"]
    #[inline(always)]
    pub fn swap(&mut self) -> SwapW<CtrlSpec> {
        SwapW::new(self, 12)
    }
    #[doc = "Bit 14 - Match 2 (MA2F) Interrupt Enable"]
    #[inline(always)]
    pub fn ma2ie(&mut self) -> Ma2ieW<CtrlSpec> {
        Ma2ieW::new(self, 14)
    }
    #[doc = "Bit 15 - Match 1 (MA1F) Interrupt Enable"]
    #[inline(always)]
    pub fn ma1ie(&mut self) -> Ma1ieW<CtrlSpec> {
        Ma1ieW::new(self, 15)
    }
    #[doc = "Bit 16 - Send Break"]
    #[inline(always)]
    pub fn sbk(&mut self) -> SbkW<CtrlSpec> {
        SbkW::new(self, 16)
    }
    #[doc = "Bit 17 - Receiver Wake-Up Control"]
    #[inline(always)]
    pub fn rwu(&mut self) -> RwuW<CtrlSpec> {
        RwuW::new(self, 17)
    }
    #[doc = "Bit 18 - Receiver Enable"]
    #[inline(always)]
    pub fn re(&mut self) -> ReW<CtrlSpec> {
        ReW::new(self, 18)
    }
    #[doc = "Bit 19 - Transmitter Enable"]
    #[inline(always)]
    pub fn te(&mut self) -> TeW<CtrlSpec> {
        TeW::new(self, 19)
    }
    #[doc = "Bit 20 - Idle Line Interrupt Enable"]
    #[inline(always)]
    pub fn ilie(&mut self) -> IlieW<CtrlSpec> {
        IlieW::new(self, 20)
    }
    #[doc = "Bit 21 - Receiver Interrupt Enable"]
    #[inline(always)]
    pub fn rie(&mut self) -> RieW<CtrlSpec> {
        RieW::new(self, 21)
    }
    #[doc = "Bit 22 - Transmission Complete Interrupt Enable"]
    #[inline(always)]
    pub fn tcie(&mut self) -> TcieW<CtrlSpec> {
        TcieW::new(self, 22)
    }
    #[doc = "Bit 23 - Transmit Interrupt Enable"]
    #[inline(always)]
    pub fn tie(&mut self) -> TieW<CtrlSpec> {
        TieW::new(self, 23)
    }
    #[doc = "Bit 24 - Parity Error Interrupt Enable"]
    #[inline(always)]
    pub fn peie(&mut self) -> PeieW<CtrlSpec> {
        PeieW::new(self, 24)
    }
    #[doc = "Bit 25 - Framing Error Interrupt Enable"]
    #[inline(always)]
    pub fn feie(&mut self) -> FeieW<CtrlSpec> {
        FeieW::new(self, 25)
    }
    #[doc = "Bit 26 - Noise Error Interrupt Enable"]
    #[inline(always)]
    pub fn neie(&mut self) -> NeieW<CtrlSpec> {
        NeieW::new(self, 26)
    }
    #[doc = "Bit 27 - Overrun Interrupt Enable"]
    #[inline(always)]
    pub fn orie(&mut self) -> OrieW<CtrlSpec> {
        OrieW::new(self, 27)
    }
    #[doc = "Bit 28 - Transmit Data Inversion"]
    #[inline(always)]
    pub fn txinv(&mut self) -> TxinvW<CtrlSpec> {
        TxinvW::new(self, 28)
    }
    #[doc = "Bit 29 - TXD Pin Direction in Single-Wire Mode"]
    #[inline(always)]
    pub fn txdir(&mut self) -> TxdirW<CtrlSpec> {
        TxdirW::new(self, 29)
    }
    #[doc = "Bit 30 - Receive Bit 9 Transmit Bit 8"]
    #[inline(always)]
    pub fn r9t8(&mut self) -> R9t8W<CtrlSpec> {
        R9t8W::new(self, 30)
    }
    #[doc = "Bit 31 - Receive Bit 8 Transmit Bit 9"]
    #[inline(always)]
    pub fn r8t9(&mut self) -> R8t9W<CtrlSpec> {
        R8t9W::new(self, 31)
    }
}
#[doc = "Control\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
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

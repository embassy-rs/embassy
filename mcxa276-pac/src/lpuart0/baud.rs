#[doc = "Register `BAUD` reader"]
pub type R = crate::R<BaudSpec>;
#[doc = "Register `BAUD` writer"]
pub type W = crate::W<BaudSpec>;
#[doc = "Field `SBR` reader - Baud Rate Modulo Divisor"]
pub type SbrR = crate::FieldReader<u16>;
#[doc = "Field `SBR` writer - Baud Rate Modulo Divisor"]
pub type SbrW<'a, REG> = crate::FieldWriter<'a, REG, 13, u16>;
#[doc = "Stop Bit Number Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sbns {
    #[doc = "0: One stop bit"]
    One = 0,
    #[doc = "1: Two stop bits"]
    Two = 1,
}
impl From<Sbns> for bool {
    #[inline(always)]
    fn from(variant: Sbns) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SBNS` reader - Stop Bit Number Select"]
pub type SbnsR = crate::BitReader<Sbns>;
impl SbnsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sbns {
        match self.bits {
            false => Sbns::One,
            true => Sbns::Two,
        }
    }
    #[doc = "One stop bit"]
    #[inline(always)]
    pub fn is_one(&self) -> bool {
        *self == Sbns::One
    }
    #[doc = "Two stop bits"]
    #[inline(always)]
    pub fn is_two(&self) -> bool {
        *self == Sbns::Two
    }
}
#[doc = "Field `SBNS` writer - Stop Bit Number Select"]
pub type SbnsW<'a, REG> = crate::BitWriter<'a, REG, Sbns>;
impl<'a, REG> SbnsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "One stop bit"]
    #[inline(always)]
    pub fn one(self) -> &'a mut crate::W<REG> {
        self.variant(Sbns::One)
    }
    #[doc = "Two stop bits"]
    #[inline(always)]
    pub fn two(self) -> &'a mut crate::W<REG> {
        self.variant(Sbns::Two)
    }
}
#[doc = "RX Input Active Edge Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rxedgie {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Rxedgie> for bool {
    #[inline(always)]
    fn from(variant: Rxedgie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RXEDGIE` reader - RX Input Active Edge Interrupt Enable"]
pub type RxedgieR = crate::BitReader<Rxedgie>;
impl RxedgieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rxedgie {
        match self.bits {
            false => Rxedgie::Disable,
            true => Rxedgie::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Rxedgie::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Rxedgie::Enable
    }
}
#[doc = "Field `RXEDGIE` writer - RX Input Active Edge Interrupt Enable"]
pub type RxedgieW<'a, REG> = crate::BitWriter<'a, REG, Rxedgie>;
impl<'a, REG> RxedgieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Rxedgie::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Rxedgie::Enable)
    }
}
#[doc = "LIN Break Detect Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lbkdie {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Lbkdie> for bool {
    #[inline(always)]
    fn from(variant: Lbkdie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LBKDIE` reader - LIN Break Detect Interrupt Enable"]
pub type LbkdieR = crate::BitReader<Lbkdie>;
impl LbkdieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lbkdie {
        match self.bits {
            false => Lbkdie::Disable,
            true => Lbkdie::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Lbkdie::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Lbkdie::Enable
    }
}
#[doc = "Field `LBKDIE` writer - LIN Break Detect Interrupt Enable"]
pub type LbkdieW<'a, REG> = crate::BitWriter<'a, REG, Lbkdie>;
impl<'a, REG> LbkdieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Lbkdie::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Lbkdie::Enable)
    }
}
#[doc = "Resynchronization Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resyncdis {
    #[doc = "0: Enable"]
    Resync = 0,
    #[doc = "1: Disable"]
    NoResync = 1,
}
impl From<Resyncdis> for bool {
    #[inline(always)]
    fn from(variant: Resyncdis) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RESYNCDIS` reader - Resynchronization Disable"]
pub type ResyncdisR = crate::BitReader<Resyncdis>;
impl ResyncdisR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Resyncdis {
        match self.bits {
            false => Resyncdis::Resync,
            true => Resyncdis::NoResync,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_resync(&self) -> bool {
        *self == Resyncdis::Resync
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_no_resync(&self) -> bool {
        *self == Resyncdis::NoResync
    }
}
#[doc = "Field `RESYNCDIS` writer - Resynchronization Disable"]
pub type ResyncdisW<'a, REG> = crate::BitWriter<'a, REG, Resyncdis>;
impl<'a, REG> ResyncdisW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn resync(self) -> &'a mut crate::W<REG> {
        self.variant(Resyncdis::Resync)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn no_resync(self) -> &'a mut crate::W<REG> {
        self.variant(Resyncdis::NoResync)
    }
}
#[doc = "Both Edge Sampling\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bothedge {
    #[doc = "0: Rising edge"]
    Disabled = 0,
    #[doc = "1: Both rising and falling edges"]
    Enabled = 1,
}
impl From<Bothedge> for bool {
    #[inline(always)]
    fn from(variant: Bothedge) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BOTHEDGE` reader - Both Edge Sampling"]
pub type BothedgeR = crate::BitReader<Bothedge>;
impl BothedgeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Bothedge {
        match self.bits {
            false => Bothedge::Disabled,
            true => Bothedge::Enabled,
        }
    }
    #[doc = "Rising edge"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Bothedge::Disabled
    }
    #[doc = "Both rising and falling edges"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Bothedge::Enabled
    }
}
#[doc = "Field `BOTHEDGE` writer - Both Edge Sampling"]
pub type BothedgeW<'a, REG> = crate::BitWriter<'a, REG, Bothedge>;
impl<'a, REG> BothedgeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Rising edge"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Bothedge::Disabled)
    }
    #[doc = "Both rising and falling edges"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Bothedge::Enabled)
    }
}
#[doc = "Match Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Matcfg {
    #[doc = "0: Address match wake-up"]
    AddrMatch = 0,
    #[doc = "1: Idle match wake-up"]
    IdleMatch = 1,
    #[doc = "2: Match on and match off"]
    OnoffMatch = 2,
    #[doc = "3: Enables RWU on data match and match on or off for the transmitter CTS input"]
    RwuMatch = 3,
}
impl From<Matcfg> for u8 {
    #[inline(always)]
    fn from(variant: Matcfg) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Matcfg {
    type Ux = u8;
}
impl crate::IsEnum for Matcfg {}
#[doc = "Field `MATCFG` reader - Match Configuration"]
pub type MatcfgR = crate::FieldReader<Matcfg>;
impl MatcfgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Matcfg {
        match self.bits {
            0 => Matcfg::AddrMatch,
            1 => Matcfg::IdleMatch,
            2 => Matcfg::OnoffMatch,
            3 => Matcfg::RwuMatch,
            _ => unreachable!(),
        }
    }
    #[doc = "Address match wake-up"]
    #[inline(always)]
    pub fn is_addr_match(&self) -> bool {
        *self == Matcfg::AddrMatch
    }
    #[doc = "Idle match wake-up"]
    #[inline(always)]
    pub fn is_idle_match(&self) -> bool {
        *self == Matcfg::IdleMatch
    }
    #[doc = "Match on and match off"]
    #[inline(always)]
    pub fn is_onoff_match(&self) -> bool {
        *self == Matcfg::OnoffMatch
    }
    #[doc = "Enables RWU on data match and match on or off for the transmitter CTS input"]
    #[inline(always)]
    pub fn is_rwu_match(&self) -> bool {
        *self == Matcfg::RwuMatch
    }
}
#[doc = "Field `MATCFG` writer - Match Configuration"]
pub type MatcfgW<'a, REG> = crate::FieldWriter<'a, REG, 2, Matcfg, crate::Safe>;
impl<'a, REG> MatcfgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Address match wake-up"]
    #[inline(always)]
    pub fn addr_match(self) -> &'a mut crate::W<REG> {
        self.variant(Matcfg::AddrMatch)
    }
    #[doc = "Idle match wake-up"]
    #[inline(always)]
    pub fn idle_match(self) -> &'a mut crate::W<REG> {
        self.variant(Matcfg::IdleMatch)
    }
    #[doc = "Match on and match off"]
    #[inline(always)]
    pub fn onoff_match(self) -> &'a mut crate::W<REG> {
        self.variant(Matcfg::OnoffMatch)
    }
    #[doc = "Enables RWU on data match and match on or off for the transmitter CTS input"]
    #[inline(always)]
    pub fn rwu_match(self) -> &'a mut crate::W<REG> {
        self.variant(Matcfg::RwuMatch)
    }
}
#[doc = "Receiver Idle DMA Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ridmae {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Ridmae> for bool {
    #[inline(always)]
    fn from(variant: Ridmae) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RIDMAE` reader - Receiver Idle DMA Enable"]
pub type RidmaeR = crate::BitReader<Ridmae>;
impl RidmaeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ridmae {
        match self.bits {
            false => Ridmae::Disabled,
            true => Ridmae::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Ridmae::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Ridmae::Enabled
    }
}
#[doc = "Field `RIDMAE` writer - Receiver Idle DMA Enable"]
pub type RidmaeW<'a, REG> = crate::BitWriter<'a, REG, Ridmae>;
impl<'a, REG> RidmaeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ridmae::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ridmae::Enabled)
    }
}
#[doc = "Receiver Full DMA Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rdmae {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Rdmae> for bool {
    #[inline(always)]
    fn from(variant: Rdmae) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RDMAE` reader - Receiver Full DMA Enable"]
pub type RdmaeR = crate::BitReader<Rdmae>;
impl RdmaeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rdmae {
        match self.bits {
            false => Rdmae::Disabled,
            true => Rdmae::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Rdmae::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Rdmae::Enabled
    }
}
#[doc = "Field `RDMAE` writer - Receiver Full DMA Enable"]
pub type RdmaeW<'a, REG> = crate::BitWriter<'a, REG, Rdmae>;
impl<'a, REG> RdmaeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rdmae::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rdmae::Enabled)
    }
}
#[doc = "Transmitter DMA Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tdmae {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Tdmae> for bool {
    #[inline(always)]
    fn from(variant: Tdmae) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TDMAE` reader - Transmitter DMA Enable"]
pub type TdmaeR = crate::BitReader<Tdmae>;
impl TdmaeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tdmae {
        match self.bits {
            false => Tdmae::Disabled,
            true => Tdmae::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Tdmae::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Tdmae::Enabled
    }
}
#[doc = "Field `TDMAE` writer - Transmitter DMA Enable"]
pub type TdmaeW<'a, REG> = crate::BitWriter<'a, REG, Tdmae>;
impl<'a, REG> TdmaeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Tdmae::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Tdmae::Enabled)
    }
}
#[doc = "Oversampling Ratio\n\nValue on reset: 15"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Osr {
    #[doc = "0: Results in an OSR of 16"]
    Default = 0,
    #[doc = "3: Results in an OSR of 4 (requires BAUD\\[BOTHEDGE\\] to be 1)"]
    Osr4 = 3,
    #[doc = "4: Results in an OSR of 5 (requires BAUD\\[BOTHEDGE\\] to be 1)"]
    Osr5 = 4,
    #[doc = "5: Results in an OSR of 6 (requires BAUD\\[BOTHEDGE\\] to be 1)"]
    Osr6 = 5,
    #[doc = "6: Results in an OSR of 7 (requires BAUD\\[BOTHEDGE\\] to be 1)"]
    Osr7 = 6,
    #[doc = "7: Results in an OSR of 8"]
    Osr8 = 7,
    #[doc = "8: Results in an OSR of 9"]
    Osr9 = 8,
    #[doc = "9: Results in an OSR of 10"]
    Osr10 = 9,
    #[doc = "10: Results in an OSR of 11"]
    Osr11 = 10,
    #[doc = "11: Results in an OSR of 12"]
    Osr12 = 11,
    #[doc = "12: Results in an OSR of 13"]
    Osr13 = 12,
    #[doc = "13: Results in an OSR of 14"]
    Osr14 = 13,
    #[doc = "14: Results in an OSR of 15"]
    Osr15 = 14,
    #[doc = "15: Results in an OSR of 16"]
    Osr16 = 15,
    #[doc = "16: Results in an OSR of 17"]
    Osr17 = 16,
    #[doc = "17: Results in an OSR of 18"]
    Osr18 = 17,
    #[doc = "18: Results in an OSR of 19"]
    Osr19 = 18,
    #[doc = "19: Results in an OSR of 20"]
    Osr20 = 19,
    #[doc = "20: Results in an OSR of 21"]
    Osr21 = 20,
    #[doc = "21: Results in an OSR of 22"]
    Osr22 = 21,
    #[doc = "22: Results in an OSR of 23"]
    Osr23 = 22,
    #[doc = "23: Results in an OSR of 24"]
    Osr24 = 23,
    #[doc = "24: Results in an OSR of 25"]
    Osr25 = 24,
    #[doc = "25: Results in an OSR of 26"]
    Osr26 = 25,
    #[doc = "26: Results in an OSR of 27"]
    Osr27 = 26,
    #[doc = "27: Results in an OSR of 28"]
    Osr28 = 27,
    #[doc = "28: Results in an OSR of 29"]
    Osr29 = 28,
    #[doc = "29: Results in an OSR of 30"]
    Osr30 = 29,
    #[doc = "30: Results in an OSR of 31"]
    Osr31 = 30,
    #[doc = "31: Results in an OSR of 32"]
    Osr32 = 31,
}
impl From<Osr> for u8 {
    #[inline(always)]
    fn from(variant: Osr) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Osr {
    type Ux = u8;
}
impl crate::IsEnum for Osr {}
#[doc = "Field `OSR` reader - Oversampling Ratio"]
pub type OsrR = crate::FieldReader<Osr>;
impl OsrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Osr> {
        match self.bits {
            0 => Some(Osr::Default),
            3 => Some(Osr::Osr4),
            4 => Some(Osr::Osr5),
            5 => Some(Osr::Osr6),
            6 => Some(Osr::Osr7),
            7 => Some(Osr::Osr8),
            8 => Some(Osr::Osr9),
            9 => Some(Osr::Osr10),
            10 => Some(Osr::Osr11),
            11 => Some(Osr::Osr12),
            12 => Some(Osr::Osr13),
            13 => Some(Osr::Osr14),
            14 => Some(Osr::Osr15),
            15 => Some(Osr::Osr16),
            16 => Some(Osr::Osr17),
            17 => Some(Osr::Osr18),
            18 => Some(Osr::Osr19),
            19 => Some(Osr::Osr20),
            20 => Some(Osr::Osr21),
            21 => Some(Osr::Osr22),
            22 => Some(Osr::Osr23),
            23 => Some(Osr::Osr24),
            24 => Some(Osr::Osr25),
            25 => Some(Osr::Osr26),
            26 => Some(Osr::Osr27),
            27 => Some(Osr::Osr28),
            28 => Some(Osr::Osr29),
            29 => Some(Osr::Osr30),
            30 => Some(Osr::Osr31),
            31 => Some(Osr::Osr32),
            _ => None,
        }
    }
    #[doc = "Results in an OSR of 16"]
    #[inline(always)]
    pub fn is_default(&self) -> bool {
        *self == Osr::Default
    }
    #[doc = "Results in an OSR of 4 (requires BAUD\\[BOTHEDGE\\] to be 1)"]
    #[inline(always)]
    pub fn is_osr_4(&self) -> bool {
        *self == Osr::Osr4
    }
    #[doc = "Results in an OSR of 5 (requires BAUD\\[BOTHEDGE\\] to be 1)"]
    #[inline(always)]
    pub fn is_osr_5(&self) -> bool {
        *self == Osr::Osr5
    }
    #[doc = "Results in an OSR of 6 (requires BAUD\\[BOTHEDGE\\] to be 1)"]
    #[inline(always)]
    pub fn is_osr_6(&self) -> bool {
        *self == Osr::Osr6
    }
    #[doc = "Results in an OSR of 7 (requires BAUD\\[BOTHEDGE\\] to be 1)"]
    #[inline(always)]
    pub fn is_osr_7(&self) -> bool {
        *self == Osr::Osr7
    }
    #[doc = "Results in an OSR of 8"]
    #[inline(always)]
    pub fn is_osr_8(&self) -> bool {
        *self == Osr::Osr8
    }
    #[doc = "Results in an OSR of 9"]
    #[inline(always)]
    pub fn is_osr_9(&self) -> bool {
        *self == Osr::Osr9
    }
    #[doc = "Results in an OSR of 10"]
    #[inline(always)]
    pub fn is_osr_10(&self) -> bool {
        *self == Osr::Osr10
    }
    #[doc = "Results in an OSR of 11"]
    #[inline(always)]
    pub fn is_osr_11(&self) -> bool {
        *self == Osr::Osr11
    }
    #[doc = "Results in an OSR of 12"]
    #[inline(always)]
    pub fn is_osr_12(&self) -> bool {
        *self == Osr::Osr12
    }
    #[doc = "Results in an OSR of 13"]
    #[inline(always)]
    pub fn is_osr_13(&self) -> bool {
        *self == Osr::Osr13
    }
    #[doc = "Results in an OSR of 14"]
    #[inline(always)]
    pub fn is_osr_14(&self) -> bool {
        *self == Osr::Osr14
    }
    #[doc = "Results in an OSR of 15"]
    #[inline(always)]
    pub fn is_osr_15(&self) -> bool {
        *self == Osr::Osr15
    }
    #[doc = "Results in an OSR of 16"]
    #[inline(always)]
    pub fn is_osr_16(&self) -> bool {
        *self == Osr::Osr16
    }
    #[doc = "Results in an OSR of 17"]
    #[inline(always)]
    pub fn is_osr_17(&self) -> bool {
        *self == Osr::Osr17
    }
    #[doc = "Results in an OSR of 18"]
    #[inline(always)]
    pub fn is_osr_18(&self) -> bool {
        *self == Osr::Osr18
    }
    #[doc = "Results in an OSR of 19"]
    #[inline(always)]
    pub fn is_osr_19(&self) -> bool {
        *self == Osr::Osr19
    }
    #[doc = "Results in an OSR of 20"]
    #[inline(always)]
    pub fn is_osr_20(&self) -> bool {
        *self == Osr::Osr20
    }
    #[doc = "Results in an OSR of 21"]
    #[inline(always)]
    pub fn is_osr_21(&self) -> bool {
        *self == Osr::Osr21
    }
    #[doc = "Results in an OSR of 22"]
    #[inline(always)]
    pub fn is_osr_22(&self) -> bool {
        *self == Osr::Osr22
    }
    #[doc = "Results in an OSR of 23"]
    #[inline(always)]
    pub fn is_osr_23(&self) -> bool {
        *self == Osr::Osr23
    }
    #[doc = "Results in an OSR of 24"]
    #[inline(always)]
    pub fn is_osr_24(&self) -> bool {
        *self == Osr::Osr24
    }
    #[doc = "Results in an OSR of 25"]
    #[inline(always)]
    pub fn is_osr_25(&self) -> bool {
        *self == Osr::Osr25
    }
    #[doc = "Results in an OSR of 26"]
    #[inline(always)]
    pub fn is_osr_26(&self) -> bool {
        *self == Osr::Osr26
    }
    #[doc = "Results in an OSR of 27"]
    #[inline(always)]
    pub fn is_osr_27(&self) -> bool {
        *self == Osr::Osr27
    }
    #[doc = "Results in an OSR of 28"]
    #[inline(always)]
    pub fn is_osr_28(&self) -> bool {
        *self == Osr::Osr28
    }
    #[doc = "Results in an OSR of 29"]
    #[inline(always)]
    pub fn is_osr_29(&self) -> bool {
        *self == Osr::Osr29
    }
    #[doc = "Results in an OSR of 30"]
    #[inline(always)]
    pub fn is_osr_30(&self) -> bool {
        *self == Osr::Osr30
    }
    #[doc = "Results in an OSR of 31"]
    #[inline(always)]
    pub fn is_osr_31(&self) -> bool {
        *self == Osr::Osr31
    }
    #[doc = "Results in an OSR of 32"]
    #[inline(always)]
    pub fn is_osr_32(&self) -> bool {
        *self == Osr::Osr32
    }
}
#[doc = "Field `OSR` writer - Oversampling Ratio"]
pub type OsrW<'a, REG> = crate::FieldWriter<'a, REG, 5, Osr>;
impl<'a, REG> OsrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Results in an OSR of 16"]
    #[inline(always)]
    pub fn default(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Default)
    }
    #[doc = "Results in an OSR of 4 (requires BAUD\\[BOTHEDGE\\] to be 1)"]
    #[inline(always)]
    pub fn osr_4(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr4)
    }
    #[doc = "Results in an OSR of 5 (requires BAUD\\[BOTHEDGE\\] to be 1)"]
    #[inline(always)]
    pub fn osr_5(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr5)
    }
    #[doc = "Results in an OSR of 6 (requires BAUD\\[BOTHEDGE\\] to be 1)"]
    #[inline(always)]
    pub fn osr_6(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr6)
    }
    #[doc = "Results in an OSR of 7 (requires BAUD\\[BOTHEDGE\\] to be 1)"]
    #[inline(always)]
    pub fn osr_7(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr7)
    }
    #[doc = "Results in an OSR of 8"]
    #[inline(always)]
    pub fn osr_8(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr8)
    }
    #[doc = "Results in an OSR of 9"]
    #[inline(always)]
    pub fn osr_9(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr9)
    }
    #[doc = "Results in an OSR of 10"]
    #[inline(always)]
    pub fn osr_10(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr10)
    }
    #[doc = "Results in an OSR of 11"]
    #[inline(always)]
    pub fn osr_11(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr11)
    }
    #[doc = "Results in an OSR of 12"]
    #[inline(always)]
    pub fn osr_12(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr12)
    }
    #[doc = "Results in an OSR of 13"]
    #[inline(always)]
    pub fn osr_13(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr13)
    }
    #[doc = "Results in an OSR of 14"]
    #[inline(always)]
    pub fn osr_14(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr14)
    }
    #[doc = "Results in an OSR of 15"]
    #[inline(always)]
    pub fn osr_15(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr15)
    }
    #[doc = "Results in an OSR of 16"]
    #[inline(always)]
    pub fn osr_16(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr16)
    }
    #[doc = "Results in an OSR of 17"]
    #[inline(always)]
    pub fn osr_17(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr17)
    }
    #[doc = "Results in an OSR of 18"]
    #[inline(always)]
    pub fn osr_18(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr18)
    }
    #[doc = "Results in an OSR of 19"]
    #[inline(always)]
    pub fn osr_19(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr19)
    }
    #[doc = "Results in an OSR of 20"]
    #[inline(always)]
    pub fn osr_20(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr20)
    }
    #[doc = "Results in an OSR of 21"]
    #[inline(always)]
    pub fn osr_21(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr21)
    }
    #[doc = "Results in an OSR of 22"]
    #[inline(always)]
    pub fn osr_22(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr22)
    }
    #[doc = "Results in an OSR of 23"]
    #[inline(always)]
    pub fn osr_23(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr23)
    }
    #[doc = "Results in an OSR of 24"]
    #[inline(always)]
    pub fn osr_24(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr24)
    }
    #[doc = "Results in an OSR of 25"]
    #[inline(always)]
    pub fn osr_25(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr25)
    }
    #[doc = "Results in an OSR of 26"]
    #[inline(always)]
    pub fn osr_26(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr26)
    }
    #[doc = "Results in an OSR of 27"]
    #[inline(always)]
    pub fn osr_27(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr27)
    }
    #[doc = "Results in an OSR of 28"]
    #[inline(always)]
    pub fn osr_28(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr28)
    }
    #[doc = "Results in an OSR of 29"]
    #[inline(always)]
    pub fn osr_29(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr29)
    }
    #[doc = "Results in an OSR of 30"]
    #[inline(always)]
    pub fn osr_30(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr30)
    }
    #[doc = "Results in an OSR of 31"]
    #[inline(always)]
    pub fn osr_31(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr31)
    }
    #[doc = "Results in an OSR of 32"]
    #[inline(always)]
    pub fn osr_32(self) -> &'a mut crate::W<REG> {
        self.variant(Osr::Osr32)
    }
}
#[doc = "10-Bit Mode Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum M10 {
    #[doc = "0: Receiver and transmitter use 7-bit to 9-bit data characters"]
    Disabled = 0,
    #[doc = "1: Receiver and transmitter use 10-bit data characters"]
    Enabled = 1,
}
impl From<M10> for bool {
    #[inline(always)]
    fn from(variant: M10) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `M10` reader - 10-Bit Mode Select"]
pub type M10R = crate::BitReader<M10>;
impl M10R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> M10 {
        match self.bits {
            false => M10::Disabled,
            true => M10::Enabled,
        }
    }
    #[doc = "Receiver and transmitter use 7-bit to 9-bit data characters"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == M10::Disabled
    }
    #[doc = "Receiver and transmitter use 10-bit data characters"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == M10::Enabled
    }
}
#[doc = "Field `M10` writer - 10-Bit Mode Select"]
pub type M10W<'a, REG> = crate::BitWriter<'a, REG, M10>;
impl<'a, REG> M10W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Receiver and transmitter use 7-bit to 9-bit data characters"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(M10::Disabled)
    }
    #[doc = "Receiver and transmitter use 10-bit data characters"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(M10::Enabled)
    }
}
#[doc = "Match Address Mode Enable 2\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Maen2 {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Maen2> for bool {
    #[inline(always)]
    fn from(variant: Maen2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MAEN2` reader - Match Address Mode Enable 2"]
pub type Maen2R = crate::BitReader<Maen2>;
impl Maen2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Maen2 {
        match self.bits {
            false => Maen2::Disabled,
            true => Maen2::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Maen2::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Maen2::Enabled
    }
}
#[doc = "Field `MAEN2` writer - Match Address Mode Enable 2"]
pub type Maen2W<'a, REG> = crate::BitWriter<'a, REG, Maen2>;
impl<'a, REG> Maen2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Maen2::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Maen2::Enabled)
    }
}
#[doc = "Match Address Mode Enable 1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Maen1 {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Maen1> for bool {
    #[inline(always)]
    fn from(variant: Maen1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MAEN1` reader - Match Address Mode Enable 1"]
pub type Maen1R = crate::BitReader<Maen1>;
impl Maen1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Maen1 {
        match self.bits {
            false => Maen1::Disabled,
            true => Maen1::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Maen1::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Maen1::Enabled
    }
}
#[doc = "Field `MAEN1` writer - Match Address Mode Enable 1"]
pub type Maen1W<'a, REG> = crate::BitWriter<'a, REG, Maen1>;
impl<'a, REG> Maen1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Maen1::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Maen1::Enabled)
    }
}
impl R {
    #[doc = "Bits 0:12 - Baud Rate Modulo Divisor"]
    #[inline(always)]
    pub fn sbr(&self) -> SbrR {
        SbrR::new((self.bits & 0x1fff) as u16)
    }
    #[doc = "Bit 13 - Stop Bit Number Select"]
    #[inline(always)]
    pub fn sbns(&self) -> SbnsR {
        SbnsR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - RX Input Active Edge Interrupt Enable"]
    #[inline(always)]
    pub fn rxedgie(&self) -> RxedgieR {
        RxedgieR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - LIN Break Detect Interrupt Enable"]
    #[inline(always)]
    pub fn lbkdie(&self) -> LbkdieR {
        LbkdieR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - Resynchronization Disable"]
    #[inline(always)]
    pub fn resyncdis(&self) -> ResyncdisR {
        ResyncdisR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Both Edge Sampling"]
    #[inline(always)]
    pub fn bothedge(&self) -> BothedgeR {
        BothedgeR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bits 18:19 - Match Configuration"]
    #[inline(always)]
    pub fn matcfg(&self) -> MatcfgR {
        MatcfgR::new(((self.bits >> 18) & 3) as u8)
    }
    #[doc = "Bit 20 - Receiver Idle DMA Enable"]
    #[inline(always)]
    pub fn ridmae(&self) -> RidmaeR {
        RidmaeR::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - Receiver Full DMA Enable"]
    #[inline(always)]
    pub fn rdmae(&self) -> RdmaeR {
        RdmaeR::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 23 - Transmitter DMA Enable"]
    #[inline(always)]
    pub fn tdmae(&self) -> TdmaeR {
        TdmaeR::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bits 24:28 - Oversampling Ratio"]
    #[inline(always)]
    pub fn osr(&self) -> OsrR {
        OsrR::new(((self.bits >> 24) & 0x1f) as u8)
    }
    #[doc = "Bit 29 - 10-Bit Mode Select"]
    #[inline(always)]
    pub fn m10(&self) -> M10R {
        M10R::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - Match Address Mode Enable 2"]
    #[inline(always)]
    pub fn maen2(&self) -> Maen2R {
        Maen2R::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Match Address Mode Enable 1"]
    #[inline(always)]
    pub fn maen1(&self) -> Maen1R {
        Maen1R::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:12 - Baud Rate Modulo Divisor"]
    #[inline(always)]
    pub fn sbr(&mut self) -> SbrW<BaudSpec> {
        SbrW::new(self, 0)
    }
    #[doc = "Bit 13 - Stop Bit Number Select"]
    #[inline(always)]
    pub fn sbns(&mut self) -> SbnsW<BaudSpec> {
        SbnsW::new(self, 13)
    }
    #[doc = "Bit 14 - RX Input Active Edge Interrupt Enable"]
    #[inline(always)]
    pub fn rxedgie(&mut self) -> RxedgieW<BaudSpec> {
        RxedgieW::new(self, 14)
    }
    #[doc = "Bit 15 - LIN Break Detect Interrupt Enable"]
    #[inline(always)]
    pub fn lbkdie(&mut self) -> LbkdieW<BaudSpec> {
        LbkdieW::new(self, 15)
    }
    #[doc = "Bit 16 - Resynchronization Disable"]
    #[inline(always)]
    pub fn resyncdis(&mut self) -> ResyncdisW<BaudSpec> {
        ResyncdisW::new(self, 16)
    }
    #[doc = "Bit 17 - Both Edge Sampling"]
    #[inline(always)]
    pub fn bothedge(&mut self) -> BothedgeW<BaudSpec> {
        BothedgeW::new(self, 17)
    }
    #[doc = "Bits 18:19 - Match Configuration"]
    #[inline(always)]
    pub fn matcfg(&mut self) -> MatcfgW<BaudSpec> {
        MatcfgW::new(self, 18)
    }
    #[doc = "Bit 20 - Receiver Idle DMA Enable"]
    #[inline(always)]
    pub fn ridmae(&mut self) -> RidmaeW<BaudSpec> {
        RidmaeW::new(self, 20)
    }
    #[doc = "Bit 21 - Receiver Full DMA Enable"]
    #[inline(always)]
    pub fn rdmae(&mut self) -> RdmaeW<BaudSpec> {
        RdmaeW::new(self, 21)
    }
    #[doc = "Bit 23 - Transmitter DMA Enable"]
    #[inline(always)]
    pub fn tdmae(&mut self) -> TdmaeW<BaudSpec> {
        TdmaeW::new(self, 23)
    }
    #[doc = "Bits 24:28 - Oversampling Ratio"]
    #[inline(always)]
    pub fn osr(&mut self) -> OsrW<BaudSpec> {
        OsrW::new(self, 24)
    }
    #[doc = "Bit 29 - 10-Bit Mode Select"]
    #[inline(always)]
    pub fn m10(&mut self) -> M10W<BaudSpec> {
        M10W::new(self, 29)
    }
    #[doc = "Bit 30 - Match Address Mode Enable 2"]
    #[inline(always)]
    pub fn maen2(&mut self) -> Maen2W<BaudSpec> {
        Maen2W::new(self, 30)
    }
    #[doc = "Bit 31 - Match Address Mode Enable 1"]
    #[inline(always)]
    pub fn maen1(&mut self) -> Maen1W<BaudSpec> {
        Maen1W::new(self, 31)
    }
}
#[doc = "Baud Rate\n\nYou can [`read`](crate::Reg::read) this register and get [`baud::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`baud::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct BaudSpec;
impl crate::RegisterSpec for BaudSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`baud::R`](R) reader structure"]
impl crate::Readable for BaudSpec {}
#[doc = "`write(|w| ..)` method takes [`baud::W`](W) writer structure"]
impl crate::Writable for BaudSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets BAUD to value 0x0f00_0004"]
impl crate::Resettable for BaudSpec {
    const RESET_VALUE: u32 = 0x0f00_0004;
}

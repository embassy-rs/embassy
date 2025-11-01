#[doc = "Register `TCR` reader"]
pub type R = crate::R<TcrSpec>;
#[doc = "Register `TCR` writer"]
pub type W = crate::W<TcrSpec>;
#[doc = "Field `FRAMESZ` reader - Frame Size"]
pub type FrameszR = crate::FieldReader<u16>;
#[doc = "Field `FRAMESZ` writer - Frame Size"]
pub type FrameszW<'a, REG> = crate::FieldWriter<'a, REG, 12, u16>;
#[doc = "Transfer Width\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Width {
    #[doc = "0: 1-bit transfer"]
    Onebit = 0,
    #[doc = "1: 2-bit transfer"]
    Twobit = 1,
    #[doc = "2: 4-bit transfer"]
    Fourbit = 2,
}
impl From<Width> for u8 {
    #[inline(always)]
    fn from(variant: Width) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Width {
    type Ux = u8;
}
impl crate::IsEnum for Width {}
#[doc = "Field `WIDTH` reader - Transfer Width"]
pub type WidthR = crate::FieldReader<Width>;
impl WidthR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Width> {
        match self.bits {
            0 => Some(Width::Onebit),
            1 => Some(Width::Twobit),
            2 => Some(Width::Fourbit),
            _ => None,
        }
    }
    #[doc = "1-bit transfer"]
    #[inline(always)]
    pub fn is_onebit(&self) -> bool {
        *self == Width::Onebit
    }
    #[doc = "2-bit transfer"]
    #[inline(always)]
    pub fn is_twobit(&self) -> bool {
        *self == Width::Twobit
    }
    #[doc = "4-bit transfer"]
    #[inline(always)]
    pub fn is_fourbit(&self) -> bool {
        *self == Width::Fourbit
    }
}
#[doc = "Field `WIDTH` writer - Transfer Width"]
pub type WidthW<'a, REG> = crate::FieldWriter<'a, REG, 2, Width>;
impl<'a, REG> WidthW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "1-bit transfer"]
    #[inline(always)]
    pub fn onebit(self) -> &'a mut crate::W<REG> {
        self.variant(Width::Onebit)
    }
    #[doc = "2-bit transfer"]
    #[inline(always)]
    pub fn twobit(self) -> &'a mut crate::W<REG> {
        self.variant(Width::Twobit)
    }
    #[doc = "4-bit transfer"]
    #[inline(always)]
    pub fn fourbit(self) -> &'a mut crate::W<REG> {
        self.variant(Width::Fourbit)
    }
}
#[doc = "Transmit Data Mask\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Txmsk {
    #[doc = "0: Normal transfer"]
    Normal = 0,
    #[doc = "1: Mask transmit data"]
    Mask = 1,
}
impl From<Txmsk> for bool {
    #[inline(always)]
    fn from(variant: Txmsk) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TXMSK` reader - Transmit Data Mask"]
pub type TxmskR = crate::BitReader<Txmsk>;
impl TxmskR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Txmsk {
        match self.bits {
            false => Txmsk::Normal,
            true => Txmsk::Mask,
        }
    }
    #[doc = "Normal transfer"]
    #[inline(always)]
    pub fn is_normal(&self) -> bool {
        *self == Txmsk::Normal
    }
    #[doc = "Mask transmit data"]
    #[inline(always)]
    pub fn is_mask(&self) -> bool {
        *self == Txmsk::Mask
    }
}
#[doc = "Field `TXMSK` writer - Transmit Data Mask"]
pub type TxmskW<'a, REG> = crate::BitWriter<'a, REG, Txmsk>;
impl<'a, REG> TxmskW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Normal transfer"]
    #[inline(always)]
    pub fn normal(self) -> &'a mut crate::W<REG> {
        self.variant(Txmsk::Normal)
    }
    #[doc = "Mask transmit data"]
    #[inline(always)]
    pub fn mask(self) -> &'a mut crate::W<REG> {
        self.variant(Txmsk::Mask)
    }
}
#[doc = "Receive Data Mask\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rxmsk {
    #[doc = "0: Normal transfer"]
    Normal = 0,
    #[doc = "1: Mask receive data"]
    Mask = 1,
}
impl From<Rxmsk> for bool {
    #[inline(always)]
    fn from(variant: Rxmsk) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RXMSK` reader - Receive Data Mask"]
pub type RxmskR = crate::BitReader<Rxmsk>;
impl RxmskR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rxmsk {
        match self.bits {
            false => Rxmsk::Normal,
            true => Rxmsk::Mask,
        }
    }
    #[doc = "Normal transfer"]
    #[inline(always)]
    pub fn is_normal(&self) -> bool {
        *self == Rxmsk::Normal
    }
    #[doc = "Mask receive data"]
    #[inline(always)]
    pub fn is_mask(&self) -> bool {
        *self == Rxmsk::Mask
    }
}
#[doc = "Field `RXMSK` writer - Receive Data Mask"]
pub type RxmskW<'a, REG> = crate::BitWriter<'a, REG, Rxmsk>;
impl<'a, REG> RxmskW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Normal transfer"]
    #[inline(always)]
    pub fn normal(self) -> &'a mut crate::W<REG> {
        self.variant(Rxmsk::Normal)
    }
    #[doc = "Mask receive data"]
    #[inline(always)]
    pub fn mask(self) -> &'a mut crate::W<REG> {
        self.variant(Rxmsk::Mask)
    }
}
#[doc = "Continuing Command\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Contc {
    #[doc = "0: Command word for start of new transfer"]
    Start = 0,
    #[doc = "1: Command word for continuing transfer"]
    Continue = 1,
}
impl From<Contc> for bool {
    #[inline(always)]
    fn from(variant: Contc) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CONTC` reader - Continuing Command"]
pub type ContcR = crate::BitReader<Contc>;
impl ContcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Contc {
        match self.bits {
            false => Contc::Start,
            true => Contc::Continue,
        }
    }
    #[doc = "Command word for start of new transfer"]
    #[inline(always)]
    pub fn is_start(&self) -> bool {
        *self == Contc::Start
    }
    #[doc = "Command word for continuing transfer"]
    #[inline(always)]
    pub fn is_continue(&self) -> bool {
        *self == Contc::Continue
    }
}
#[doc = "Field `CONTC` writer - Continuing Command"]
pub type ContcW<'a, REG> = crate::BitWriter<'a, REG, Contc>;
impl<'a, REG> ContcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Command word for start of new transfer"]
    #[inline(always)]
    pub fn start(self) -> &'a mut crate::W<REG> {
        self.variant(Contc::Start)
    }
    #[doc = "Command word for continuing transfer"]
    #[inline(always)]
    pub fn continue_(self) -> &'a mut crate::W<REG> {
        self.variant(Contc::Continue)
    }
}
#[doc = "Continuous Transfer\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cont {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Cont> for bool {
    #[inline(always)]
    fn from(variant: Cont) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CONT` reader - Continuous Transfer"]
pub type ContR = crate::BitReader<Cont>;
impl ContR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cont {
        match self.bits {
            false => Cont::Disabled,
            true => Cont::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Cont::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Cont::Enabled
    }
}
#[doc = "Field `CONT` writer - Continuous Transfer"]
pub type ContW<'a, REG> = crate::BitWriter<'a, REG, Cont>;
impl<'a, REG> ContW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Cont::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Cont::Enabled)
    }
}
#[doc = "Byte Swap\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bysw {
    #[doc = "0: Disable byte swap"]
    Disabled = 0,
    #[doc = "1: Enable byte swap"]
    Enabled = 1,
}
impl From<Bysw> for bool {
    #[inline(always)]
    fn from(variant: Bysw) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BYSW` reader - Byte Swap"]
pub type ByswR = crate::BitReader<Bysw>;
impl ByswR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Bysw {
        match self.bits {
            false => Bysw::Disabled,
            true => Bysw::Enabled,
        }
    }
    #[doc = "Disable byte swap"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Bysw::Disabled
    }
    #[doc = "Enable byte swap"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Bysw::Enabled
    }
}
#[doc = "Field `BYSW` writer - Byte Swap"]
pub type ByswW<'a, REG> = crate::BitWriter<'a, REG, Bysw>;
impl<'a, REG> ByswW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable byte swap"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Bysw::Disabled)
    }
    #[doc = "Enable byte swap"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Bysw::Enabled)
    }
}
#[doc = "LSB First\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lsbf {
    #[doc = "0: MSB first"]
    MsbFirst = 0,
    #[doc = "1: LSB first"]
    LsbFirst = 1,
}
impl From<Lsbf> for bool {
    #[inline(always)]
    fn from(variant: Lsbf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LSBF` reader - LSB First"]
pub type LsbfR = crate::BitReader<Lsbf>;
impl LsbfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lsbf {
        match self.bits {
            false => Lsbf::MsbFirst,
            true => Lsbf::LsbFirst,
        }
    }
    #[doc = "MSB first"]
    #[inline(always)]
    pub fn is_msb_first(&self) -> bool {
        *self == Lsbf::MsbFirst
    }
    #[doc = "LSB first"]
    #[inline(always)]
    pub fn is_lsb_first(&self) -> bool {
        *self == Lsbf::LsbFirst
    }
}
#[doc = "Field `LSBF` writer - LSB First"]
pub type LsbfW<'a, REG> = crate::BitWriter<'a, REG, Lsbf>;
impl<'a, REG> LsbfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "MSB first"]
    #[inline(always)]
    pub fn msb_first(self) -> &'a mut crate::W<REG> {
        self.variant(Lsbf::MsbFirst)
    }
    #[doc = "LSB first"]
    #[inline(always)]
    pub fn lsb_first(self) -> &'a mut crate::W<REG> {
        self.variant(Lsbf::LsbFirst)
    }
}
#[doc = "Peripheral Chip Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pcs {
    #[doc = "0: Transfer using PCS\\[0\\]"]
    TxPcs0 = 0,
    #[doc = "1: Transfer using PCS\\[1\\]"]
    TxPcs1 = 1,
    #[doc = "2: Transfer using PCS\\[2\\]"]
    TxPcs2 = 2,
    #[doc = "3: Transfer using PCS\\[3\\]"]
    TxPcs3 = 3,
}
impl From<Pcs> for u8 {
    #[inline(always)]
    fn from(variant: Pcs) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pcs {
    type Ux = u8;
}
impl crate::IsEnum for Pcs {}
#[doc = "Field `PCS` reader - Peripheral Chip Select"]
pub type PcsR = crate::FieldReader<Pcs>;
impl PcsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pcs {
        match self.bits {
            0 => Pcs::TxPcs0,
            1 => Pcs::TxPcs1,
            2 => Pcs::TxPcs2,
            3 => Pcs::TxPcs3,
            _ => unreachable!(),
        }
    }
    #[doc = "Transfer using PCS\\[0\\]"]
    #[inline(always)]
    pub fn is_tx_pcs0(&self) -> bool {
        *self == Pcs::TxPcs0
    }
    #[doc = "Transfer using PCS\\[1\\]"]
    #[inline(always)]
    pub fn is_tx_pcs1(&self) -> bool {
        *self == Pcs::TxPcs1
    }
    #[doc = "Transfer using PCS\\[2\\]"]
    #[inline(always)]
    pub fn is_tx_pcs2(&self) -> bool {
        *self == Pcs::TxPcs2
    }
    #[doc = "Transfer using PCS\\[3\\]"]
    #[inline(always)]
    pub fn is_tx_pcs3(&self) -> bool {
        *self == Pcs::TxPcs3
    }
}
#[doc = "Field `PCS` writer - Peripheral Chip Select"]
pub type PcsW<'a, REG> = crate::FieldWriter<'a, REG, 2, Pcs, crate::Safe>;
impl<'a, REG> PcsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Transfer using PCS\\[0\\]"]
    #[inline(always)]
    pub fn tx_pcs0(self) -> &'a mut crate::W<REG> {
        self.variant(Pcs::TxPcs0)
    }
    #[doc = "Transfer using PCS\\[1\\]"]
    #[inline(always)]
    pub fn tx_pcs1(self) -> &'a mut crate::W<REG> {
        self.variant(Pcs::TxPcs1)
    }
    #[doc = "Transfer using PCS\\[2\\]"]
    #[inline(always)]
    pub fn tx_pcs2(self) -> &'a mut crate::W<REG> {
        self.variant(Pcs::TxPcs2)
    }
    #[doc = "Transfer using PCS\\[3\\]"]
    #[inline(always)]
    pub fn tx_pcs3(self) -> &'a mut crate::W<REG> {
        self.variant(Pcs::TxPcs3)
    }
}
#[doc = "Prescaler Value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Prescale {
    #[doc = "0: Divide by 1"]
    Divideby1 = 0,
    #[doc = "1: Divide by 2"]
    Divideby2 = 1,
    #[doc = "2: Divide by 4"]
    Divideby4 = 2,
    #[doc = "3: Divide by 8"]
    Divideby8 = 3,
    #[doc = "4: Divide by 16"]
    Divideby16 = 4,
    #[doc = "5: Divide by 32"]
    Divideby32 = 5,
    #[doc = "6: Divide by 64"]
    Divideby64 = 6,
    #[doc = "7: Divide by 128"]
    Divideby128 = 7,
}
impl From<Prescale> for u8 {
    #[inline(always)]
    fn from(variant: Prescale) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Prescale {
    type Ux = u8;
}
impl crate::IsEnum for Prescale {}
#[doc = "Field `PRESCALE` reader - Prescaler Value"]
pub type PrescaleR = crate::FieldReader<Prescale>;
impl PrescaleR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Prescale {
        match self.bits {
            0 => Prescale::Divideby1,
            1 => Prescale::Divideby2,
            2 => Prescale::Divideby4,
            3 => Prescale::Divideby8,
            4 => Prescale::Divideby16,
            5 => Prescale::Divideby32,
            6 => Prescale::Divideby64,
            7 => Prescale::Divideby128,
            _ => unreachable!(),
        }
    }
    #[doc = "Divide by 1"]
    #[inline(always)]
    pub fn is_divideby1(&self) -> bool {
        *self == Prescale::Divideby1
    }
    #[doc = "Divide by 2"]
    #[inline(always)]
    pub fn is_divideby2(&self) -> bool {
        *self == Prescale::Divideby2
    }
    #[doc = "Divide by 4"]
    #[inline(always)]
    pub fn is_divideby4(&self) -> bool {
        *self == Prescale::Divideby4
    }
    #[doc = "Divide by 8"]
    #[inline(always)]
    pub fn is_divideby8(&self) -> bool {
        *self == Prescale::Divideby8
    }
    #[doc = "Divide by 16"]
    #[inline(always)]
    pub fn is_divideby16(&self) -> bool {
        *self == Prescale::Divideby16
    }
    #[doc = "Divide by 32"]
    #[inline(always)]
    pub fn is_divideby32(&self) -> bool {
        *self == Prescale::Divideby32
    }
    #[doc = "Divide by 64"]
    #[inline(always)]
    pub fn is_divideby64(&self) -> bool {
        *self == Prescale::Divideby64
    }
    #[doc = "Divide by 128"]
    #[inline(always)]
    pub fn is_divideby128(&self) -> bool {
        *self == Prescale::Divideby128
    }
}
#[doc = "Field `PRESCALE` writer - Prescaler Value"]
pub type PrescaleW<'a, REG> = crate::FieldWriter<'a, REG, 3, Prescale, crate::Safe>;
impl<'a, REG> PrescaleW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Divide by 1"]
    #[inline(always)]
    pub fn divideby1(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::Divideby1)
    }
    #[doc = "Divide by 2"]
    #[inline(always)]
    pub fn divideby2(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::Divideby2)
    }
    #[doc = "Divide by 4"]
    #[inline(always)]
    pub fn divideby4(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::Divideby4)
    }
    #[doc = "Divide by 8"]
    #[inline(always)]
    pub fn divideby8(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::Divideby8)
    }
    #[doc = "Divide by 16"]
    #[inline(always)]
    pub fn divideby16(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::Divideby16)
    }
    #[doc = "Divide by 32"]
    #[inline(always)]
    pub fn divideby32(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::Divideby32)
    }
    #[doc = "Divide by 64"]
    #[inline(always)]
    pub fn divideby64(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::Divideby64)
    }
    #[doc = "Divide by 128"]
    #[inline(always)]
    pub fn divideby128(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::Divideby128)
    }
}
#[doc = "Clock Phase\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cpha {
    #[doc = "0: Captured"]
    Captured = 0,
    #[doc = "1: Changed"]
    Changed = 1,
}
impl From<Cpha> for bool {
    #[inline(always)]
    fn from(variant: Cpha) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CPHA` reader - Clock Phase"]
pub type CphaR = crate::BitReader<Cpha>;
impl CphaR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cpha {
        match self.bits {
            false => Cpha::Captured,
            true => Cpha::Changed,
        }
    }
    #[doc = "Captured"]
    #[inline(always)]
    pub fn is_captured(&self) -> bool {
        *self == Cpha::Captured
    }
    #[doc = "Changed"]
    #[inline(always)]
    pub fn is_changed(&self) -> bool {
        *self == Cpha::Changed
    }
}
#[doc = "Field `CPHA` writer - Clock Phase"]
pub type CphaW<'a, REG> = crate::BitWriter<'a, REG, Cpha>;
impl<'a, REG> CphaW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Captured"]
    #[inline(always)]
    pub fn captured(self) -> &'a mut crate::W<REG> {
        self.variant(Cpha::Captured)
    }
    #[doc = "Changed"]
    #[inline(always)]
    pub fn changed(self) -> &'a mut crate::W<REG> {
        self.variant(Cpha::Changed)
    }
}
#[doc = "Clock Polarity\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cpol {
    #[doc = "0: Inactive low"]
    InactiveLow = 0,
    #[doc = "1: Inactive high"]
    InactiveHigh = 1,
}
impl From<Cpol> for bool {
    #[inline(always)]
    fn from(variant: Cpol) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CPOL` reader - Clock Polarity"]
pub type CpolR = crate::BitReader<Cpol>;
impl CpolR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cpol {
        match self.bits {
            false => Cpol::InactiveLow,
            true => Cpol::InactiveHigh,
        }
    }
    #[doc = "Inactive low"]
    #[inline(always)]
    pub fn is_inactive_low(&self) -> bool {
        *self == Cpol::InactiveLow
    }
    #[doc = "Inactive high"]
    #[inline(always)]
    pub fn is_inactive_high(&self) -> bool {
        *self == Cpol::InactiveHigh
    }
}
#[doc = "Field `CPOL` writer - Clock Polarity"]
pub type CpolW<'a, REG> = crate::BitWriter<'a, REG, Cpol>;
impl<'a, REG> CpolW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Inactive low"]
    #[inline(always)]
    pub fn inactive_low(self) -> &'a mut crate::W<REG> {
        self.variant(Cpol::InactiveLow)
    }
    #[doc = "Inactive high"]
    #[inline(always)]
    pub fn inactive_high(self) -> &'a mut crate::W<REG> {
        self.variant(Cpol::InactiveHigh)
    }
}
impl R {
    #[doc = "Bits 0:11 - Frame Size"]
    #[inline(always)]
    pub fn framesz(&self) -> FrameszR {
        FrameszR::new((self.bits & 0x0fff) as u16)
    }
    #[doc = "Bits 16:17 - Transfer Width"]
    #[inline(always)]
    pub fn width(&self) -> WidthR {
        WidthR::new(((self.bits >> 16) & 3) as u8)
    }
    #[doc = "Bit 18 - Transmit Data Mask"]
    #[inline(always)]
    pub fn txmsk(&self) -> TxmskR {
        TxmskR::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - Receive Data Mask"]
    #[inline(always)]
    pub fn rxmsk(&self) -> RxmskR {
        RxmskR::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - Continuing Command"]
    #[inline(always)]
    pub fn contc(&self) -> ContcR {
        ContcR::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - Continuous Transfer"]
    #[inline(always)]
    pub fn cont(&self) -> ContR {
        ContR::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - Byte Swap"]
    #[inline(always)]
    pub fn bysw(&self) -> ByswR {
        ByswR::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - LSB First"]
    #[inline(always)]
    pub fn lsbf(&self) -> LsbfR {
        LsbfR::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bits 24:25 - Peripheral Chip Select"]
    #[inline(always)]
    pub fn pcs(&self) -> PcsR {
        PcsR::new(((self.bits >> 24) & 3) as u8)
    }
    #[doc = "Bits 27:29 - Prescaler Value"]
    #[inline(always)]
    pub fn prescale(&self) -> PrescaleR {
        PrescaleR::new(((self.bits >> 27) & 7) as u8)
    }
    #[doc = "Bit 30 - Clock Phase"]
    #[inline(always)]
    pub fn cpha(&self) -> CphaR {
        CphaR::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Clock Polarity"]
    #[inline(always)]
    pub fn cpol(&self) -> CpolR {
        CpolR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:11 - Frame Size"]
    #[inline(always)]
    pub fn framesz(&mut self) -> FrameszW<TcrSpec> {
        FrameszW::new(self, 0)
    }
    #[doc = "Bits 16:17 - Transfer Width"]
    #[inline(always)]
    pub fn width(&mut self) -> WidthW<TcrSpec> {
        WidthW::new(self, 16)
    }
    #[doc = "Bit 18 - Transmit Data Mask"]
    #[inline(always)]
    pub fn txmsk(&mut self) -> TxmskW<TcrSpec> {
        TxmskW::new(self, 18)
    }
    #[doc = "Bit 19 - Receive Data Mask"]
    #[inline(always)]
    pub fn rxmsk(&mut self) -> RxmskW<TcrSpec> {
        RxmskW::new(self, 19)
    }
    #[doc = "Bit 20 - Continuing Command"]
    #[inline(always)]
    pub fn contc(&mut self) -> ContcW<TcrSpec> {
        ContcW::new(self, 20)
    }
    #[doc = "Bit 21 - Continuous Transfer"]
    #[inline(always)]
    pub fn cont(&mut self) -> ContW<TcrSpec> {
        ContW::new(self, 21)
    }
    #[doc = "Bit 22 - Byte Swap"]
    #[inline(always)]
    pub fn bysw(&mut self) -> ByswW<TcrSpec> {
        ByswW::new(self, 22)
    }
    #[doc = "Bit 23 - LSB First"]
    #[inline(always)]
    pub fn lsbf(&mut self) -> LsbfW<TcrSpec> {
        LsbfW::new(self, 23)
    }
    #[doc = "Bits 24:25 - Peripheral Chip Select"]
    #[inline(always)]
    pub fn pcs(&mut self) -> PcsW<TcrSpec> {
        PcsW::new(self, 24)
    }
    #[doc = "Bits 27:29 - Prescaler Value"]
    #[inline(always)]
    pub fn prescale(&mut self) -> PrescaleW<TcrSpec> {
        PrescaleW::new(self, 27)
    }
    #[doc = "Bit 30 - Clock Phase"]
    #[inline(always)]
    pub fn cpha(&mut self) -> CphaW<TcrSpec> {
        CphaW::new(self, 30)
    }
    #[doc = "Bit 31 - Clock Polarity"]
    #[inline(always)]
    pub fn cpol(&mut self) -> CpolW<TcrSpec> {
        CpolW::new(self, 31)
    }
}
#[doc = "Transmit Command\n\nYou can [`read`](crate::Reg::read) this register and get [`tcr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tcr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TcrSpec;
impl crate::RegisterSpec for TcrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`tcr::R`](R) reader structure"]
impl crate::Readable for TcrSpec {}
#[doc = "`write(|w| ..)` method takes [`tcr::W`](W) writer structure"]
impl crate::Writable for TcrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TCR to value 0x1f"]
impl crate::Resettable for TcrSpec {
    const RESET_VALUE: u32 = 0x1f;
}

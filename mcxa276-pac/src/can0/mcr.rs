#[doc = "Register `MCR` reader"]
pub type R = crate::R<McrSpec>;
#[doc = "Register `MCR` writer"]
pub type W = crate::W<McrSpec>;
#[doc = "Field `MAXMB` reader - Number of the Last Message Buffer"]
pub type MaxmbR = crate::FieldReader;
#[doc = "Field `MAXMB` writer - Number of the Last Message Buffer"]
pub type MaxmbW<'a, REG> = crate::FieldWriter<'a, REG, 7>;
#[doc = "ID Acceptance Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Idam {
    #[doc = "0: Format A: One full ID (standard and extended) per ID filter table element."]
    OneFullId = 0,
    #[doc = "1: Format B: Two full standard IDs or two partial 14-bit (standard and extended) IDs per ID filter table element."]
    TwoFullId = 1,
    #[doc = "2: Format C: Four partial 8-bit standard IDs per ID filter table element."]
    FourPartialId = 2,
    #[doc = "3: Format D: All frames rejected."]
    AllFramesRejected = 3,
}
impl From<Idam> for u8 {
    #[inline(always)]
    fn from(variant: Idam) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Idam {
    type Ux = u8;
}
impl crate::IsEnum for Idam {}
#[doc = "Field `IDAM` reader - ID Acceptance Mode"]
pub type IdamR = crate::FieldReader<Idam>;
impl IdamR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Idam {
        match self.bits {
            0 => Idam::OneFullId,
            1 => Idam::TwoFullId,
            2 => Idam::FourPartialId,
            3 => Idam::AllFramesRejected,
            _ => unreachable!(),
        }
    }
    #[doc = "Format A: One full ID (standard and extended) per ID filter table element."]
    #[inline(always)]
    pub fn is_one_full_id(&self) -> bool {
        *self == Idam::OneFullId
    }
    #[doc = "Format B: Two full standard IDs or two partial 14-bit (standard and extended) IDs per ID filter table element."]
    #[inline(always)]
    pub fn is_two_full_id(&self) -> bool {
        *self == Idam::TwoFullId
    }
    #[doc = "Format C: Four partial 8-bit standard IDs per ID filter table element."]
    #[inline(always)]
    pub fn is_four_partial_id(&self) -> bool {
        *self == Idam::FourPartialId
    }
    #[doc = "Format D: All frames rejected."]
    #[inline(always)]
    pub fn is_all_frames_rejected(&self) -> bool {
        *self == Idam::AllFramesRejected
    }
}
#[doc = "Field `IDAM` writer - ID Acceptance Mode"]
pub type IdamW<'a, REG> = crate::FieldWriter<'a, REG, 2, Idam, crate::Safe>;
impl<'a, REG> IdamW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Format A: One full ID (standard and extended) per ID filter table element."]
    #[inline(always)]
    pub fn one_full_id(self) -> &'a mut crate::W<REG> {
        self.variant(Idam::OneFullId)
    }
    #[doc = "Format B: Two full standard IDs or two partial 14-bit (standard and extended) IDs per ID filter table element."]
    #[inline(always)]
    pub fn two_full_id(self) -> &'a mut crate::W<REG> {
        self.variant(Idam::TwoFullId)
    }
    #[doc = "Format C: Four partial 8-bit standard IDs per ID filter table element."]
    #[inline(always)]
    pub fn four_partial_id(self) -> &'a mut crate::W<REG> {
        self.variant(Idam::FourPartialId)
    }
    #[doc = "Format D: All frames rejected."]
    #[inline(always)]
    pub fn all_frames_rejected(self) -> &'a mut crate::W<REG> {
        self.variant(Idam::AllFramesRejected)
    }
}
#[doc = "CAN FD Operation Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fden {
    #[doc = "0: Disable"]
    CanFdDisabled = 0,
    #[doc = "1: Enable"]
    CanFdEnabled = 1,
}
impl From<Fden> for bool {
    #[inline(always)]
    fn from(variant: Fden) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FDEN` reader - CAN FD Operation Enable"]
pub type FdenR = crate::BitReader<Fden>;
impl FdenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fden {
        match self.bits {
            false => Fden::CanFdDisabled,
            true => Fden::CanFdEnabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_can_fd_disabled(&self) -> bool {
        *self == Fden::CanFdDisabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_can_fd_enabled(&self) -> bool {
        *self == Fden::CanFdEnabled
    }
}
#[doc = "Field `FDEN` writer - CAN FD Operation Enable"]
pub type FdenW<'a, REG> = crate::BitWriter<'a, REG, Fden>;
impl<'a, REG> FdenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn can_fd_disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Fden::CanFdDisabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn can_fd_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Fden::CanFdEnabled)
    }
}
#[doc = "Abort Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aen {
    #[doc = "0: Disabled"]
    AbortDisabled = 0,
    #[doc = "1: Enabled"]
    AbortEnabled = 1,
}
impl From<Aen> for bool {
    #[inline(always)]
    fn from(variant: Aen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `AEN` reader - Abort Enable"]
pub type AenR = crate::BitReader<Aen>;
impl AenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Aen {
        match self.bits {
            false => Aen::AbortDisabled,
            true => Aen::AbortEnabled,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_abort_disabled(&self) -> bool {
        *self == Aen::AbortDisabled
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_abort_enabled(&self) -> bool {
        *self == Aen::AbortEnabled
    }
}
#[doc = "Field `AEN` writer - Abort Enable"]
pub type AenW<'a, REG> = crate::BitWriter<'a, REG, Aen>;
impl<'a, REG> AenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn abort_disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Aen::AbortDisabled)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn abort_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Aen::AbortEnabled)
    }
}
#[doc = "Local Priority Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lprioen {
    #[doc = "0: Disable"]
    LocalPriorityDisabled = 0,
    #[doc = "1: Enable"]
    LocalPriorityEnabled = 1,
}
impl From<Lprioen> for bool {
    #[inline(always)]
    fn from(variant: Lprioen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LPRIOEN` reader - Local Priority Enable"]
pub type LprioenR = crate::BitReader<Lprioen>;
impl LprioenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lprioen {
        match self.bits {
            false => Lprioen::LocalPriorityDisabled,
            true => Lprioen::LocalPriorityEnabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_local_priority_disabled(&self) -> bool {
        *self == Lprioen::LocalPriorityDisabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_local_priority_enabled(&self) -> bool {
        *self == Lprioen::LocalPriorityEnabled
    }
}
#[doc = "Field `LPRIOEN` writer - Local Priority Enable"]
pub type LprioenW<'a, REG> = crate::BitWriter<'a, REG, Lprioen>;
impl<'a, REG> LprioenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn local_priority_disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lprioen::LocalPriorityDisabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn local_priority_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lprioen::LocalPriorityEnabled)
    }
}
#[doc = "Pretended Networking Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PnetEn {
    #[doc = "0: Disable"]
    PnDisabled = 0,
    #[doc = "1: Enable"]
    PnEnabled = 1,
}
impl From<PnetEn> for bool {
    #[inline(always)]
    fn from(variant: PnetEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PNET_EN` reader - Pretended Networking Enable"]
pub type PnetEnR = crate::BitReader<PnetEn>;
impl PnetEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> PnetEn {
        match self.bits {
            false => PnetEn::PnDisabled,
            true => PnetEn::PnEnabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_pn_disabled(&self) -> bool {
        *self == PnetEn::PnDisabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_pn_enabled(&self) -> bool {
        *self == PnetEn::PnEnabled
    }
}
#[doc = "Field `PNET_EN` writer - Pretended Networking Enable"]
pub type PnetEnW<'a, REG> = crate::BitWriter<'a, REG, PnetEn>;
impl<'a, REG> PnetEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn pn_disabled(self) -> &'a mut crate::W<REG> {
        self.variant(PnetEn::PnDisabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn pn_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(PnetEn::PnEnabled)
    }
}
#[doc = "DMA Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dma {
    #[doc = "0: Disable"]
    Id1 = 0,
    #[doc = "1: Enable"]
    Id2 = 1,
}
impl From<Dma> for bool {
    #[inline(always)]
    fn from(variant: Dma) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DMA` reader - DMA Enable"]
pub type DmaR = crate::BitReader<Dma>;
impl DmaR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dma {
        match self.bits {
            false => Dma::Id1,
            true => Dma::Id2,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_id1(&self) -> bool {
        *self == Dma::Id1
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_id2(&self) -> bool {
        *self == Dma::Id2
    }
}
#[doc = "Field `DMA` writer - DMA Enable"]
pub type DmaW<'a, REG> = crate::BitWriter<'a, REG, Dma>;
impl<'a, REG> DmaW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn id1(self) -> &'a mut crate::W<REG> {
        self.variant(Dma::Id1)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn id2(self) -> &'a mut crate::W<REG> {
        self.variant(Dma::Id2)
    }
}
#[doc = "Individual RX Masking and Queue Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Irmq {
    #[doc = "0: Disable"]
    IndividualRxMaskingDisabled = 0,
    #[doc = "1: Enable"]
    IndividualRxMaskingEnabled = 1,
}
impl From<Irmq> for bool {
    #[inline(always)]
    fn from(variant: Irmq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `IRMQ` reader - Individual RX Masking and Queue Enable"]
pub type IrmqR = crate::BitReader<Irmq>;
impl IrmqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Irmq {
        match self.bits {
            false => Irmq::IndividualRxMaskingDisabled,
            true => Irmq::IndividualRxMaskingEnabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_individual_rx_masking_disabled(&self) -> bool {
        *self == Irmq::IndividualRxMaskingDisabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_individual_rx_masking_enabled(&self) -> bool {
        *self == Irmq::IndividualRxMaskingEnabled
    }
}
#[doc = "Field `IRMQ` writer - Individual RX Masking and Queue Enable"]
pub type IrmqW<'a, REG> = crate::BitWriter<'a, REG, Irmq>;
impl<'a, REG> IrmqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn individual_rx_masking_disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Irmq::IndividualRxMaskingDisabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn individual_rx_masking_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Irmq::IndividualRxMaskingEnabled)
    }
}
#[doc = "Self-Reception Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Srxdis {
    #[doc = "0: Enable"]
    SelfReceptionEnabled = 0,
    #[doc = "1: Disable"]
    SelfReceptionDisabled = 1,
}
impl From<Srxdis> for bool {
    #[inline(always)]
    fn from(variant: Srxdis) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SRXDIS` reader - Self-Reception Disable"]
pub type SrxdisR = crate::BitReader<Srxdis>;
impl SrxdisR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Srxdis {
        match self.bits {
            false => Srxdis::SelfReceptionEnabled,
            true => Srxdis::SelfReceptionDisabled,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_self_reception_enabled(&self) -> bool {
        *self == Srxdis::SelfReceptionEnabled
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_self_reception_disabled(&self) -> bool {
        *self == Srxdis::SelfReceptionDisabled
    }
}
#[doc = "Field `SRXDIS` writer - Self-Reception Disable"]
pub type SrxdisW<'a, REG> = crate::BitWriter<'a, REG, Srxdis>;
impl<'a, REG> SrxdisW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn self_reception_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Srxdis::SelfReceptionEnabled)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn self_reception_disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Srxdis::SelfReceptionDisabled)
    }
}
#[doc = "Doze Mode Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Doze {
    #[doc = "0: Disable"]
    LowPowerDozeDisabled = 0,
    #[doc = "1: Enable"]
    LowPowerDozeEnabled = 1,
}
impl From<Doze> for bool {
    #[inline(always)]
    fn from(variant: Doze) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DOZE` reader - Doze Mode Enable"]
pub type DozeR = crate::BitReader<Doze>;
impl DozeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Doze {
        match self.bits {
            false => Doze::LowPowerDozeDisabled,
            true => Doze::LowPowerDozeEnabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_low_power_doze_disabled(&self) -> bool {
        *self == Doze::LowPowerDozeDisabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_low_power_doze_enabled(&self) -> bool {
        *self == Doze::LowPowerDozeEnabled
    }
}
#[doc = "Field `DOZE` writer - Doze Mode Enable"]
pub type DozeW<'a, REG> = crate::BitWriter<'a, REG, Doze>;
impl<'a, REG> DozeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn low_power_doze_disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Doze::LowPowerDozeDisabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn low_power_doze_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Doze::LowPowerDozeEnabled)
    }
}
#[doc = "Wake-Up Source\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Waksrc {
    #[doc = "0: No filter applied"]
    UnfilteredRxInput = 0,
    #[doc = "1: Filter applied"]
    FilteredRxInput = 1,
}
impl From<Waksrc> for bool {
    #[inline(always)]
    fn from(variant: Waksrc) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WAKSRC` reader - Wake-Up Source"]
pub type WaksrcR = crate::BitReader<Waksrc>;
impl WaksrcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Waksrc {
        match self.bits {
            false => Waksrc::UnfilteredRxInput,
            true => Waksrc::FilteredRxInput,
        }
    }
    #[doc = "No filter applied"]
    #[inline(always)]
    pub fn is_unfiltered_rx_input(&self) -> bool {
        *self == Waksrc::UnfilteredRxInput
    }
    #[doc = "Filter applied"]
    #[inline(always)]
    pub fn is_filtered_rx_input(&self) -> bool {
        *self == Waksrc::FilteredRxInput
    }
}
#[doc = "Field `WAKSRC` writer - Wake-Up Source"]
pub type WaksrcW<'a, REG> = crate::BitWriter<'a, REG, Waksrc>;
impl<'a, REG> WaksrcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No filter applied"]
    #[inline(always)]
    pub fn unfiltered_rx_input(self) -> &'a mut crate::W<REG> {
        self.variant(Waksrc::UnfilteredRxInput)
    }
    #[doc = "Filter applied"]
    #[inline(always)]
    pub fn filtered_rx_input(self) -> &'a mut crate::W<REG> {
        self.variant(Waksrc::FilteredRxInput)
    }
}
#[doc = "Low-Power Mode Acknowledge\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lpmack {
    #[doc = "0: Not in a low-power mode"]
    LowPowerNo = 0,
    #[doc = "1: In a low-power mode"]
    LowPowerYes = 1,
}
impl From<Lpmack> for bool {
    #[inline(always)]
    fn from(variant: Lpmack) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LPMACK` reader - Low-Power Mode Acknowledge"]
pub type LpmackR = crate::BitReader<Lpmack>;
impl LpmackR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lpmack {
        match self.bits {
            false => Lpmack::LowPowerNo,
            true => Lpmack::LowPowerYes,
        }
    }
    #[doc = "Not in a low-power mode"]
    #[inline(always)]
    pub fn is_low_power_no(&self) -> bool {
        *self == Lpmack::LowPowerNo
    }
    #[doc = "In a low-power mode"]
    #[inline(always)]
    pub fn is_low_power_yes(&self) -> bool {
        *self == Lpmack::LowPowerYes
    }
}
#[doc = "Warning Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wrnen {
    #[doc = "0: Disable"]
    TwrnintRwrnintInactive = 0,
    #[doc = "1: Enable"]
    TwrnintRwrnintActive = 1,
}
impl From<Wrnen> for bool {
    #[inline(always)]
    fn from(variant: Wrnen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WRNEN` reader - Warning Interrupt Enable"]
pub type WrnenR = crate::BitReader<Wrnen>;
impl WrnenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wrnen {
        match self.bits {
            false => Wrnen::TwrnintRwrnintInactive,
            true => Wrnen::TwrnintRwrnintActive,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_twrnint_rwrnint_inactive(&self) -> bool {
        *self == Wrnen::TwrnintRwrnintInactive
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_twrnint_rwrnint_active(&self) -> bool {
        *self == Wrnen::TwrnintRwrnintActive
    }
}
#[doc = "Field `WRNEN` writer - Warning Interrupt Enable"]
pub type WrnenW<'a, REG> = crate::BitWriter<'a, REG, Wrnen>;
impl<'a, REG> WrnenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn twrnint_rwrnint_inactive(self) -> &'a mut crate::W<REG> {
        self.variant(Wrnen::TwrnintRwrnintInactive)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn twrnint_rwrnint_active(self) -> &'a mut crate::W<REG> {
        self.variant(Wrnen::TwrnintRwrnintActive)
    }
}
#[doc = "Self Wake-up\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slfwak {
    #[doc = "0: Disable"]
    SelfWakeupDisabled = 0,
    #[doc = "1: Enable"]
    SelfWakeupEnabled = 1,
}
impl From<Slfwak> for bool {
    #[inline(always)]
    fn from(variant: Slfwak) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SLFWAK` reader - Self Wake-up"]
pub type SlfwakR = crate::BitReader<Slfwak>;
impl SlfwakR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Slfwak {
        match self.bits {
            false => Slfwak::SelfWakeupDisabled,
            true => Slfwak::SelfWakeupEnabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_self_wakeup_disabled(&self) -> bool {
        *self == Slfwak::SelfWakeupDisabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_self_wakeup_enabled(&self) -> bool {
        *self == Slfwak::SelfWakeupEnabled
    }
}
#[doc = "Field `SLFWAK` writer - Self Wake-up"]
pub type SlfwakW<'a, REG> = crate::BitWriter<'a, REG, Slfwak>;
impl<'a, REG> SlfwakW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn self_wakeup_disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Slfwak::SelfWakeupDisabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn self_wakeup_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Slfwak::SelfWakeupEnabled)
    }
}
#[doc = "Supervisor Mode\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Supv {
    #[doc = "0: User mode"]
    Id1 = 0,
    #[doc = "1: Supervisor mode"]
    Id2 = 1,
}
impl From<Supv> for bool {
    #[inline(always)]
    fn from(variant: Supv) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SUPV` reader - Supervisor Mode"]
pub type SupvR = crate::BitReader<Supv>;
impl SupvR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Supv {
        match self.bits {
            false => Supv::Id1,
            true => Supv::Id2,
        }
    }
    #[doc = "User mode"]
    #[inline(always)]
    pub fn is_id1(&self) -> bool {
        *self == Supv::Id1
    }
    #[doc = "Supervisor mode"]
    #[inline(always)]
    pub fn is_id2(&self) -> bool {
        *self == Supv::Id2
    }
}
#[doc = "Field `SUPV` writer - Supervisor Mode"]
pub type SupvW<'a, REG> = crate::BitWriter<'a, REG, Supv>;
impl<'a, REG> SupvW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "User mode"]
    #[inline(always)]
    pub fn id1(self) -> &'a mut crate::W<REG> {
        self.variant(Supv::Id1)
    }
    #[doc = "Supervisor mode"]
    #[inline(always)]
    pub fn id2(self) -> &'a mut crate::W<REG> {
        self.variant(Supv::Id2)
    }
}
#[doc = "Freeze Mode Acknowledge\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Frzack {
    #[doc = "0: Not in Freeze mode, prescaler running."]
    FreezeModeNo = 0,
    #[doc = "1: In Freeze mode, prescaler stopped."]
    FreezeModeYes = 1,
}
impl From<Frzack> for bool {
    #[inline(always)]
    fn from(variant: Frzack) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FRZACK` reader - Freeze Mode Acknowledge"]
pub type FrzackR = crate::BitReader<Frzack>;
impl FrzackR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Frzack {
        match self.bits {
            false => Frzack::FreezeModeNo,
            true => Frzack::FreezeModeYes,
        }
    }
    #[doc = "Not in Freeze mode, prescaler running."]
    #[inline(always)]
    pub fn is_freeze_mode_no(&self) -> bool {
        *self == Frzack::FreezeModeNo
    }
    #[doc = "In Freeze mode, prescaler stopped."]
    #[inline(always)]
    pub fn is_freeze_mode_yes(&self) -> bool {
        *self == Frzack::FreezeModeYes
    }
}
#[doc = "Soft Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Softrst {
    #[doc = "0: No reset"]
    SoftrstNoResetRequest = 0,
    #[doc = "1: Soft reset affects reset registers"]
    SoftrstResetRegisters = 1,
}
impl From<Softrst> for bool {
    #[inline(always)]
    fn from(variant: Softrst) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SOFTRST` reader - Soft Reset"]
pub type SoftrstR = crate::BitReader<Softrst>;
impl SoftrstR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Softrst {
        match self.bits {
            false => Softrst::SoftrstNoResetRequest,
            true => Softrst::SoftrstResetRegisters,
        }
    }
    #[doc = "No reset"]
    #[inline(always)]
    pub fn is_softrst_no_reset_request(&self) -> bool {
        *self == Softrst::SoftrstNoResetRequest
    }
    #[doc = "Soft reset affects reset registers"]
    #[inline(always)]
    pub fn is_softrst_reset_registers(&self) -> bool {
        *self == Softrst::SoftrstResetRegisters
    }
}
#[doc = "Field `SOFTRST` writer - Soft Reset"]
pub type SoftrstW<'a, REG> = crate::BitWriter<'a, REG, Softrst>;
impl<'a, REG> SoftrstW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No reset"]
    #[inline(always)]
    pub fn softrst_no_reset_request(self) -> &'a mut crate::W<REG> {
        self.variant(Softrst::SoftrstNoResetRequest)
    }
    #[doc = "Soft reset affects reset registers"]
    #[inline(always)]
    pub fn softrst_reset_registers(self) -> &'a mut crate::W<REG> {
        self.variant(Softrst::SoftrstResetRegisters)
    }
}
#[doc = "Wake-up Interrupt Mask\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wakmsk {
    #[doc = "0: Disabled"]
    WakeupInterruptDisabled = 0,
    #[doc = "1: Enabled"]
    WakeupInterruptEnabled = 1,
}
impl From<Wakmsk> for bool {
    #[inline(always)]
    fn from(variant: Wakmsk) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WAKMSK` reader - Wake-up Interrupt Mask"]
pub type WakmskR = crate::BitReader<Wakmsk>;
impl WakmskR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wakmsk {
        match self.bits {
            false => Wakmsk::WakeupInterruptDisabled,
            true => Wakmsk::WakeupInterruptEnabled,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_wakeup_interrupt_disabled(&self) -> bool {
        *self == Wakmsk::WakeupInterruptDisabled
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_wakeup_interrupt_enabled(&self) -> bool {
        *self == Wakmsk::WakeupInterruptEnabled
    }
}
#[doc = "Field `WAKMSK` writer - Wake-up Interrupt Mask"]
pub type WakmskW<'a, REG> = crate::BitWriter<'a, REG, Wakmsk>;
impl<'a, REG> WakmskW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn wakeup_interrupt_disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Wakmsk::WakeupInterruptDisabled)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn wakeup_interrupt_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Wakmsk::WakeupInterruptEnabled)
    }
}
#[doc = "FlexCAN Not Ready\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Notrdy {
    #[doc = "0: FlexCAN is in Normal mode, Listen-Only mode, or Loopback mode."]
    Id1 = 0,
    #[doc = "1: FlexCAN is in Disable mode, Doze mode, Stop mode, or Freeze mode."]
    Id2 = 1,
}
impl From<Notrdy> for bool {
    #[inline(always)]
    fn from(variant: Notrdy) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NOTRDY` reader - FlexCAN Not Ready"]
pub type NotrdyR = crate::BitReader<Notrdy>;
impl NotrdyR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Notrdy {
        match self.bits {
            false => Notrdy::Id1,
            true => Notrdy::Id2,
        }
    }
    #[doc = "FlexCAN is in Normal mode, Listen-Only mode, or Loopback mode."]
    #[inline(always)]
    pub fn is_id1(&self) -> bool {
        *self == Notrdy::Id1
    }
    #[doc = "FlexCAN is in Disable mode, Doze mode, Stop mode, or Freeze mode."]
    #[inline(always)]
    pub fn is_id2(&self) -> bool {
        *self == Notrdy::Id2
    }
}
#[doc = "Halt FlexCAN\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Halt {
    #[doc = "0: No request"]
    HaltDisable = 0,
    #[doc = "1: Enter Freeze mode, if MCR\\[FRZ\\] = 1."]
    HaltEnable = 1,
}
impl From<Halt> for bool {
    #[inline(always)]
    fn from(variant: Halt) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HALT` reader - Halt FlexCAN"]
pub type HaltR = crate::BitReader<Halt>;
impl HaltR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Halt {
        match self.bits {
            false => Halt::HaltDisable,
            true => Halt::HaltEnable,
        }
    }
    #[doc = "No request"]
    #[inline(always)]
    pub fn is_halt_disable(&self) -> bool {
        *self == Halt::HaltDisable
    }
    #[doc = "Enter Freeze mode, if MCR\\[FRZ\\] = 1."]
    #[inline(always)]
    pub fn is_halt_enable(&self) -> bool {
        *self == Halt::HaltEnable
    }
}
#[doc = "Field `HALT` writer - Halt FlexCAN"]
pub type HaltW<'a, REG> = crate::BitWriter<'a, REG, Halt>;
impl<'a, REG> HaltW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No request"]
    #[inline(always)]
    pub fn halt_disable(self) -> &'a mut crate::W<REG> {
        self.variant(Halt::HaltDisable)
    }
    #[doc = "Enter Freeze mode, if MCR\\[FRZ\\] = 1."]
    #[inline(always)]
    pub fn halt_enable(self) -> &'a mut crate::W<REG> {
        self.variant(Halt::HaltEnable)
    }
}
#[doc = "Legacy RX FIFO Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rfen {
    #[doc = "0: Disable"]
    Id1 = 0,
    #[doc = "1: Enable"]
    Id2 = 1,
}
impl From<Rfen> for bool {
    #[inline(always)]
    fn from(variant: Rfen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RFEN` reader - Legacy RX FIFO Enable"]
pub type RfenR = crate::BitReader<Rfen>;
impl RfenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rfen {
        match self.bits {
            false => Rfen::Id1,
            true => Rfen::Id2,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_id1(&self) -> bool {
        *self == Rfen::Id1
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_id2(&self) -> bool {
        *self == Rfen::Id2
    }
}
#[doc = "Field `RFEN` writer - Legacy RX FIFO Enable"]
pub type RfenW<'a, REG> = crate::BitWriter<'a, REG, Rfen>;
impl<'a, REG> RfenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn id1(self) -> &'a mut crate::W<REG> {
        self.variant(Rfen::Id1)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn id2(self) -> &'a mut crate::W<REG> {
        self.variant(Rfen::Id2)
    }
}
#[doc = "Freeze Enable\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Frz {
    #[doc = "0: Disable"]
    FreezeModeDisabled = 0,
    #[doc = "1: Enable"]
    FreezeModeEnabled = 1,
}
impl From<Frz> for bool {
    #[inline(always)]
    fn from(variant: Frz) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FRZ` reader - Freeze Enable"]
pub type FrzR = crate::BitReader<Frz>;
impl FrzR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Frz {
        match self.bits {
            false => Frz::FreezeModeDisabled,
            true => Frz::FreezeModeEnabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_freeze_mode_disabled(&self) -> bool {
        *self == Frz::FreezeModeDisabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_freeze_mode_enabled(&self) -> bool {
        *self == Frz::FreezeModeEnabled
    }
}
#[doc = "Field `FRZ` writer - Freeze Enable"]
pub type FrzW<'a, REG> = crate::BitWriter<'a, REG, Frz>;
impl<'a, REG> FrzW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn freeze_mode_disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Frz::FreezeModeDisabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn freeze_mode_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Frz::FreezeModeEnabled)
    }
}
#[doc = "Module Disable\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mdis {
    #[doc = "0: Enable"]
    FlexcanEnabled = 0,
    #[doc = "1: Disable"]
    FlexcanDisabled = 1,
}
impl From<Mdis> for bool {
    #[inline(always)]
    fn from(variant: Mdis) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MDIS` reader - Module Disable"]
pub type MdisR = crate::BitReader<Mdis>;
impl MdisR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mdis {
        match self.bits {
            false => Mdis::FlexcanEnabled,
            true => Mdis::FlexcanDisabled,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_flexcan_enabled(&self) -> bool {
        *self == Mdis::FlexcanEnabled
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_flexcan_disabled(&self) -> bool {
        *self == Mdis::FlexcanDisabled
    }
}
#[doc = "Field `MDIS` writer - Module Disable"]
pub type MdisW<'a, REG> = crate::BitWriter<'a, REG, Mdis>;
impl<'a, REG> MdisW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn flexcan_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Mdis::FlexcanEnabled)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn flexcan_disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Mdis::FlexcanDisabled)
    }
}
impl R {
    #[doc = "Bits 0:6 - Number of the Last Message Buffer"]
    #[inline(always)]
    pub fn maxmb(&self) -> MaxmbR {
        MaxmbR::new((self.bits & 0x7f) as u8)
    }
    #[doc = "Bits 8:9 - ID Acceptance Mode"]
    #[inline(always)]
    pub fn idam(&self) -> IdamR {
        IdamR::new(((self.bits >> 8) & 3) as u8)
    }
    #[doc = "Bit 11 - CAN FD Operation Enable"]
    #[inline(always)]
    pub fn fden(&self) -> FdenR {
        FdenR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Abort Enable"]
    #[inline(always)]
    pub fn aen(&self) -> AenR {
        AenR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Local Priority Enable"]
    #[inline(always)]
    pub fn lprioen(&self) -> LprioenR {
        LprioenR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Pretended Networking Enable"]
    #[inline(always)]
    pub fn pnet_en(&self) -> PnetEnR {
        PnetEnR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - DMA Enable"]
    #[inline(always)]
    pub fn dma(&self) -> DmaR {
        DmaR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - Individual RX Masking and Queue Enable"]
    #[inline(always)]
    pub fn irmq(&self) -> IrmqR {
        IrmqR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Self-Reception Disable"]
    #[inline(always)]
    pub fn srxdis(&self) -> SrxdisR {
        SrxdisR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Doze Mode Enable"]
    #[inline(always)]
    pub fn doze(&self) -> DozeR {
        DozeR::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - Wake-Up Source"]
    #[inline(always)]
    pub fn waksrc(&self) -> WaksrcR {
        WaksrcR::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - Low-Power Mode Acknowledge"]
    #[inline(always)]
    pub fn lpmack(&self) -> LpmackR {
        LpmackR::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - Warning Interrupt Enable"]
    #[inline(always)]
    pub fn wrnen(&self) -> WrnenR {
        WrnenR::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - Self Wake-up"]
    #[inline(always)]
    pub fn slfwak(&self) -> SlfwakR {
        SlfwakR::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - Supervisor Mode"]
    #[inline(always)]
    pub fn supv(&self) -> SupvR {
        SupvR::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bit 24 - Freeze Mode Acknowledge"]
    #[inline(always)]
    pub fn frzack(&self) -> FrzackR {
        FrzackR::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - Soft Reset"]
    #[inline(always)]
    pub fn softrst(&self) -> SoftrstR {
        SoftrstR::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - Wake-up Interrupt Mask"]
    #[inline(always)]
    pub fn wakmsk(&self) -> WakmskR {
        WakmskR::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - FlexCAN Not Ready"]
    #[inline(always)]
    pub fn notrdy(&self) -> NotrdyR {
        NotrdyR::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bit 28 - Halt FlexCAN"]
    #[inline(always)]
    pub fn halt(&self) -> HaltR {
        HaltR::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 29 - Legacy RX FIFO Enable"]
    #[inline(always)]
    pub fn rfen(&self) -> RfenR {
        RfenR::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - Freeze Enable"]
    #[inline(always)]
    pub fn frz(&self) -> FrzR {
        FrzR::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Module Disable"]
    #[inline(always)]
    pub fn mdis(&self) -> MdisR {
        MdisR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:6 - Number of the Last Message Buffer"]
    #[inline(always)]
    pub fn maxmb(&mut self) -> MaxmbW<McrSpec> {
        MaxmbW::new(self, 0)
    }
    #[doc = "Bits 8:9 - ID Acceptance Mode"]
    #[inline(always)]
    pub fn idam(&mut self) -> IdamW<McrSpec> {
        IdamW::new(self, 8)
    }
    #[doc = "Bit 11 - CAN FD Operation Enable"]
    #[inline(always)]
    pub fn fden(&mut self) -> FdenW<McrSpec> {
        FdenW::new(self, 11)
    }
    #[doc = "Bit 12 - Abort Enable"]
    #[inline(always)]
    pub fn aen(&mut self) -> AenW<McrSpec> {
        AenW::new(self, 12)
    }
    #[doc = "Bit 13 - Local Priority Enable"]
    #[inline(always)]
    pub fn lprioen(&mut self) -> LprioenW<McrSpec> {
        LprioenW::new(self, 13)
    }
    #[doc = "Bit 14 - Pretended Networking Enable"]
    #[inline(always)]
    pub fn pnet_en(&mut self) -> PnetEnW<McrSpec> {
        PnetEnW::new(self, 14)
    }
    #[doc = "Bit 15 - DMA Enable"]
    #[inline(always)]
    pub fn dma(&mut self) -> DmaW<McrSpec> {
        DmaW::new(self, 15)
    }
    #[doc = "Bit 16 - Individual RX Masking and Queue Enable"]
    #[inline(always)]
    pub fn irmq(&mut self) -> IrmqW<McrSpec> {
        IrmqW::new(self, 16)
    }
    #[doc = "Bit 17 - Self-Reception Disable"]
    #[inline(always)]
    pub fn srxdis(&mut self) -> SrxdisW<McrSpec> {
        SrxdisW::new(self, 17)
    }
    #[doc = "Bit 18 - Doze Mode Enable"]
    #[inline(always)]
    pub fn doze(&mut self) -> DozeW<McrSpec> {
        DozeW::new(self, 18)
    }
    #[doc = "Bit 19 - Wake-Up Source"]
    #[inline(always)]
    pub fn waksrc(&mut self) -> WaksrcW<McrSpec> {
        WaksrcW::new(self, 19)
    }
    #[doc = "Bit 21 - Warning Interrupt Enable"]
    #[inline(always)]
    pub fn wrnen(&mut self) -> WrnenW<McrSpec> {
        WrnenW::new(self, 21)
    }
    #[doc = "Bit 22 - Self Wake-up"]
    #[inline(always)]
    pub fn slfwak(&mut self) -> SlfwakW<McrSpec> {
        SlfwakW::new(self, 22)
    }
    #[doc = "Bit 23 - Supervisor Mode"]
    #[inline(always)]
    pub fn supv(&mut self) -> SupvW<McrSpec> {
        SupvW::new(self, 23)
    }
    #[doc = "Bit 25 - Soft Reset"]
    #[inline(always)]
    pub fn softrst(&mut self) -> SoftrstW<McrSpec> {
        SoftrstW::new(self, 25)
    }
    #[doc = "Bit 26 - Wake-up Interrupt Mask"]
    #[inline(always)]
    pub fn wakmsk(&mut self) -> WakmskW<McrSpec> {
        WakmskW::new(self, 26)
    }
    #[doc = "Bit 28 - Halt FlexCAN"]
    #[inline(always)]
    pub fn halt(&mut self) -> HaltW<McrSpec> {
        HaltW::new(self, 28)
    }
    #[doc = "Bit 29 - Legacy RX FIFO Enable"]
    #[inline(always)]
    pub fn rfen(&mut self) -> RfenW<McrSpec> {
        RfenW::new(self, 29)
    }
    #[doc = "Bit 30 - Freeze Enable"]
    #[inline(always)]
    pub fn frz(&mut self) -> FrzW<McrSpec> {
        FrzW::new(self, 30)
    }
    #[doc = "Bit 31 - Module Disable"]
    #[inline(always)]
    pub fn mdis(&mut self) -> MdisW<McrSpec> {
        MdisW::new(self, 31)
    }
}
#[doc = "Module Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`mcr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mcr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct McrSpec;
impl crate::RegisterSpec for McrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mcr::R`](R) reader structure"]
impl crate::Readable for McrSpec {}
#[doc = "`write(|w| ..)` method takes [`mcr::W`](W) writer structure"]
impl crate::Writable for McrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MCR to value 0xd890_000f"]
impl crate::Resettable for McrSpec {
    const RESET_VALUE: u32 = 0xd890_000f;
}

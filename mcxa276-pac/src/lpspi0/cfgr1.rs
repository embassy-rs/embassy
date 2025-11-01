#[doc = "Register `CFGR1` reader"]
pub type R = crate::R<Cfgr1Spec>;
#[doc = "Register `CFGR1` writer"]
pub type W = crate::W<Cfgr1Spec>;
#[doc = "Controller Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Master {
    #[doc = "0: Peripheral mode"]
    SlaveMode = 0,
    #[doc = "1: Controller mode"]
    MasterMode = 1,
}
impl From<Master> for bool {
    #[inline(always)]
    fn from(variant: Master) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MASTER` reader - Controller Mode"]
pub type MasterR = crate::BitReader<Master>;
impl MasterR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Master {
        match self.bits {
            false => Master::SlaveMode,
            true => Master::MasterMode,
        }
    }
    #[doc = "Peripheral mode"]
    #[inline(always)]
    pub fn is_slave_mode(&self) -> bool {
        *self == Master::SlaveMode
    }
    #[doc = "Controller mode"]
    #[inline(always)]
    pub fn is_master_mode(&self) -> bool {
        *self == Master::MasterMode
    }
}
#[doc = "Field `MASTER` writer - Controller Mode"]
pub type MasterW<'a, REG> = crate::BitWriter<'a, REG, Master>;
impl<'a, REG> MasterW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral mode"]
    #[inline(always)]
    pub fn slave_mode(self) -> &'a mut crate::W<REG> {
        self.variant(Master::SlaveMode)
    }
    #[doc = "Controller mode"]
    #[inline(always)]
    pub fn master_mode(self) -> &'a mut crate::W<REG> {
        self.variant(Master::MasterMode)
    }
}
#[doc = "Sample Point\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sample {
    #[doc = "0: SCK edge"]
    OnSckEdge = 0,
    #[doc = "1: Delayed SCK edge"]
    OnDelayedSckEdge = 1,
}
impl From<Sample> for bool {
    #[inline(always)]
    fn from(variant: Sample) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SAMPLE` reader - Sample Point"]
pub type SampleR = crate::BitReader<Sample>;
impl SampleR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sample {
        match self.bits {
            false => Sample::OnSckEdge,
            true => Sample::OnDelayedSckEdge,
        }
    }
    #[doc = "SCK edge"]
    #[inline(always)]
    pub fn is_on_sck_edge(&self) -> bool {
        *self == Sample::OnSckEdge
    }
    #[doc = "Delayed SCK edge"]
    #[inline(always)]
    pub fn is_on_delayed_sck_edge(&self) -> bool {
        *self == Sample::OnDelayedSckEdge
    }
}
#[doc = "Field `SAMPLE` writer - Sample Point"]
pub type SampleW<'a, REG> = crate::BitWriter<'a, REG, Sample>;
impl<'a, REG> SampleW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "SCK edge"]
    #[inline(always)]
    pub fn on_sck_edge(self) -> &'a mut crate::W<REG> {
        self.variant(Sample::OnSckEdge)
    }
    #[doc = "Delayed SCK edge"]
    #[inline(always)]
    pub fn on_delayed_sck_edge(self) -> &'a mut crate::W<REG> {
        self.variant(Sample::OnDelayedSckEdge)
    }
}
#[doc = "Automatic PCS\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Autopcs {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Autopcs> for bool {
    #[inline(always)]
    fn from(variant: Autopcs) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `AUTOPCS` reader - Automatic PCS"]
pub type AutopcsR = crate::BitReader<Autopcs>;
impl AutopcsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Autopcs {
        match self.bits {
            false => Autopcs::Disabled,
            true => Autopcs::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Autopcs::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Autopcs::Enabled
    }
}
#[doc = "Field `AUTOPCS` writer - Automatic PCS"]
pub type AutopcsW<'a, REG> = crate::BitWriter<'a, REG, Autopcs>;
impl<'a, REG> AutopcsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Autopcs::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Autopcs::Enabled)
    }
}
#[doc = "No Stall\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nostall {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Nostall> for bool {
    #[inline(always)]
    fn from(variant: Nostall) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NOSTALL` reader - No Stall"]
pub type NostallR = crate::BitReader<Nostall>;
impl NostallR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nostall {
        match self.bits {
            false => Nostall::Disable,
            true => Nostall::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Nostall::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Nostall::Enable
    }
}
#[doc = "Field `NOSTALL` writer - No Stall"]
pub type NostallW<'a, REG> = crate::BitWriter<'a, REG, Nostall>;
impl<'a, REG> NostallW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Nostall::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Nostall::Enable)
    }
}
#[doc = "Partial Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Partial {
    #[doc = "0: Discard"]
    Discarded = 0,
    #[doc = "1: Store"]
    Stored = 1,
}
impl From<Partial> for bool {
    #[inline(always)]
    fn from(variant: Partial) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PARTIAL` reader - Partial Enable"]
pub type PartialR = crate::BitReader<Partial>;
impl PartialR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Partial {
        match self.bits {
            false => Partial::Discarded,
            true => Partial::Stored,
        }
    }
    #[doc = "Discard"]
    #[inline(always)]
    pub fn is_discarded(&self) -> bool {
        *self == Partial::Discarded
    }
    #[doc = "Store"]
    #[inline(always)]
    pub fn is_stored(&self) -> bool {
        *self == Partial::Stored
    }
}
#[doc = "Field `PARTIAL` writer - Partial Enable"]
pub type PartialW<'a, REG> = crate::BitWriter<'a, REG, Partial>;
impl<'a, REG> PartialW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Discard"]
    #[inline(always)]
    pub fn discarded(self) -> &'a mut crate::W<REG> {
        self.variant(Partial::Discarded)
    }
    #[doc = "Store"]
    #[inline(always)]
    pub fn stored(self) -> &'a mut crate::W<REG> {
        self.variant(Partial::Stored)
    }
}
#[doc = "Peripheral Chip Select Polarity\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pcspol {
    #[doc = "0: Active low"]
    Discarded = 0,
    #[doc = "1: Active high"]
    Stored = 1,
}
impl From<Pcspol> for u8 {
    #[inline(always)]
    fn from(variant: Pcspol) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pcspol {
    type Ux = u8;
}
impl crate::IsEnum for Pcspol {}
#[doc = "Field `PCSPOL` reader - Peripheral Chip Select Polarity"]
pub type PcspolR = crate::FieldReader<Pcspol>;
impl PcspolR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Pcspol> {
        match self.bits {
            0 => Some(Pcspol::Discarded),
            1 => Some(Pcspol::Stored),
            _ => None,
        }
    }
    #[doc = "Active low"]
    #[inline(always)]
    pub fn is_discarded(&self) -> bool {
        *self == Pcspol::Discarded
    }
    #[doc = "Active high"]
    #[inline(always)]
    pub fn is_stored(&self) -> bool {
        *self == Pcspol::Stored
    }
}
#[doc = "Field `PCSPOL` writer - Peripheral Chip Select Polarity"]
pub type PcspolW<'a, REG> = crate::FieldWriter<'a, REG, 4, Pcspol>;
impl<'a, REG> PcspolW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Active low"]
    #[inline(always)]
    pub fn discarded(self) -> &'a mut crate::W<REG> {
        self.variant(Pcspol::Discarded)
    }
    #[doc = "Active high"]
    #[inline(always)]
    pub fn stored(self) -> &'a mut crate::W<REG> {
        self.variant(Pcspol::Stored)
    }
}
#[doc = "Match Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Matcfg {
    #[doc = "0: Match is disabled"]
    Disabled = 0,
    #[doc = "2: Match first data word with compare word"]
    EnabledFirstdatamatch = 2,
    #[doc = "3: Match any data word with compare word"]
    EnabledAnydatamatch = 3,
    #[doc = "4: Sequential match, first data word"]
    EnabledDatamatch100 = 4,
    #[doc = "5: Sequential match, any data word"]
    EnabledDatamatch101 = 5,
    #[doc = "6: Match first data word (masked) with compare word (masked)"]
    EnabledDatamatch110 = 6,
    #[doc = "7: Match any data word (masked) with compare word (masked)"]
    EnabledDatamatch111 = 7,
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
    pub const fn variant(&self) -> Option<Matcfg> {
        match self.bits {
            0 => Some(Matcfg::Disabled),
            2 => Some(Matcfg::EnabledFirstdatamatch),
            3 => Some(Matcfg::EnabledAnydatamatch),
            4 => Some(Matcfg::EnabledDatamatch100),
            5 => Some(Matcfg::EnabledDatamatch101),
            6 => Some(Matcfg::EnabledDatamatch110),
            7 => Some(Matcfg::EnabledDatamatch111),
            _ => None,
        }
    }
    #[doc = "Match is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Matcfg::Disabled
    }
    #[doc = "Match first data word with compare word"]
    #[inline(always)]
    pub fn is_enabled_firstdatamatch(&self) -> bool {
        *self == Matcfg::EnabledFirstdatamatch
    }
    #[doc = "Match any data word with compare word"]
    #[inline(always)]
    pub fn is_enabled_anydatamatch(&self) -> bool {
        *self == Matcfg::EnabledAnydatamatch
    }
    #[doc = "Sequential match, first data word"]
    #[inline(always)]
    pub fn is_enabled_datamatch_100(&self) -> bool {
        *self == Matcfg::EnabledDatamatch100
    }
    #[doc = "Sequential match, any data word"]
    #[inline(always)]
    pub fn is_enabled_datamatch_101(&self) -> bool {
        *self == Matcfg::EnabledDatamatch101
    }
    #[doc = "Match first data word (masked) with compare word (masked)"]
    #[inline(always)]
    pub fn is_enabled_datamatch_110(&self) -> bool {
        *self == Matcfg::EnabledDatamatch110
    }
    #[doc = "Match any data word (masked) with compare word (masked)"]
    #[inline(always)]
    pub fn is_enabled_datamatch_111(&self) -> bool {
        *self == Matcfg::EnabledDatamatch111
    }
}
#[doc = "Field `MATCFG` writer - Match Configuration"]
pub type MatcfgW<'a, REG> = crate::FieldWriter<'a, REG, 3, Matcfg>;
impl<'a, REG> MatcfgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Match is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Matcfg::Disabled)
    }
    #[doc = "Match first data word with compare word"]
    #[inline(always)]
    pub fn enabled_firstdatamatch(self) -> &'a mut crate::W<REG> {
        self.variant(Matcfg::EnabledFirstdatamatch)
    }
    #[doc = "Match any data word with compare word"]
    #[inline(always)]
    pub fn enabled_anydatamatch(self) -> &'a mut crate::W<REG> {
        self.variant(Matcfg::EnabledAnydatamatch)
    }
    #[doc = "Sequential match, first data word"]
    #[inline(always)]
    pub fn enabled_datamatch_100(self) -> &'a mut crate::W<REG> {
        self.variant(Matcfg::EnabledDatamatch100)
    }
    #[doc = "Sequential match, any data word"]
    #[inline(always)]
    pub fn enabled_datamatch_101(self) -> &'a mut crate::W<REG> {
        self.variant(Matcfg::EnabledDatamatch101)
    }
    #[doc = "Match first data word (masked) with compare word (masked)"]
    #[inline(always)]
    pub fn enabled_datamatch_110(self) -> &'a mut crate::W<REG> {
        self.variant(Matcfg::EnabledDatamatch110)
    }
    #[doc = "Match any data word (masked) with compare word (masked)"]
    #[inline(always)]
    pub fn enabled_datamatch_111(self) -> &'a mut crate::W<REG> {
        self.variant(Matcfg::EnabledDatamatch111)
    }
}
#[doc = "Pin Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pincfg {
    #[doc = "0: SIN is used for input data; SOUT is used for output data"]
    SinInSoutOut = 0,
    #[doc = "1: SIN is used for both input and output data; only half-duplex serial transfers are supported"]
    SinBothInOut = 1,
    #[doc = "2: SOUT is used for both input and output data; only half-duplex serial transfers are supported"]
    SoutBothInOut = 2,
    #[doc = "3: SOUT is used for input data; SIN is used for output data"]
    SoutInSinOut = 3,
}
impl From<Pincfg> for u8 {
    #[inline(always)]
    fn from(variant: Pincfg) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pincfg {
    type Ux = u8;
}
impl crate::IsEnum for Pincfg {}
#[doc = "Field `PINCFG` reader - Pin Configuration"]
pub type PincfgR = crate::FieldReader<Pincfg>;
impl PincfgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pincfg {
        match self.bits {
            0 => Pincfg::SinInSoutOut,
            1 => Pincfg::SinBothInOut,
            2 => Pincfg::SoutBothInOut,
            3 => Pincfg::SoutInSinOut,
            _ => unreachable!(),
        }
    }
    #[doc = "SIN is used for input data; SOUT is used for output data"]
    #[inline(always)]
    pub fn is_sin_in_sout_out(&self) -> bool {
        *self == Pincfg::SinInSoutOut
    }
    #[doc = "SIN is used for both input and output data; only half-duplex serial transfers are supported"]
    #[inline(always)]
    pub fn is_sin_both_in_out(&self) -> bool {
        *self == Pincfg::SinBothInOut
    }
    #[doc = "SOUT is used for both input and output data; only half-duplex serial transfers are supported"]
    #[inline(always)]
    pub fn is_sout_both_in_out(&self) -> bool {
        *self == Pincfg::SoutBothInOut
    }
    #[doc = "SOUT is used for input data; SIN is used for output data"]
    #[inline(always)]
    pub fn is_sout_in_sin_out(&self) -> bool {
        *self == Pincfg::SoutInSinOut
    }
}
#[doc = "Field `PINCFG` writer - Pin Configuration"]
pub type PincfgW<'a, REG> = crate::FieldWriter<'a, REG, 2, Pincfg, crate::Safe>;
impl<'a, REG> PincfgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "SIN is used for input data; SOUT is used for output data"]
    #[inline(always)]
    pub fn sin_in_sout_out(self) -> &'a mut crate::W<REG> {
        self.variant(Pincfg::SinInSoutOut)
    }
    #[doc = "SIN is used for both input and output data; only half-duplex serial transfers are supported"]
    #[inline(always)]
    pub fn sin_both_in_out(self) -> &'a mut crate::W<REG> {
        self.variant(Pincfg::SinBothInOut)
    }
    #[doc = "SOUT is used for both input and output data; only half-duplex serial transfers are supported"]
    #[inline(always)]
    pub fn sout_both_in_out(self) -> &'a mut crate::W<REG> {
        self.variant(Pincfg::SoutBothInOut)
    }
    #[doc = "SOUT is used for input data; SIN is used for output data"]
    #[inline(always)]
    pub fn sout_in_sin_out(self) -> &'a mut crate::W<REG> {
        self.variant(Pincfg::SoutInSinOut)
    }
}
#[doc = "Output Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcfg {
    #[doc = "0: Retain last value"]
    RetainLastvalue = 0,
    #[doc = "1: 3-stated"]
    Tristated = 1,
}
impl From<Outcfg> for bool {
    #[inline(always)]
    fn from(variant: Outcfg) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OUTCFG` reader - Output Configuration"]
pub type OutcfgR = crate::BitReader<Outcfg>;
impl OutcfgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Outcfg {
        match self.bits {
            false => Outcfg::RetainLastvalue,
            true => Outcfg::Tristated,
        }
    }
    #[doc = "Retain last value"]
    #[inline(always)]
    pub fn is_retain_lastvalue(&self) -> bool {
        *self == Outcfg::RetainLastvalue
    }
    #[doc = "3-stated"]
    #[inline(always)]
    pub fn is_tristated(&self) -> bool {
        *self == Outcfg::Tristated
    }
}
#[doc = "Field `OUTCFG` writer - Output Configuration"]
pub type OutcfgW<'a, REG> = crate::BitWriter<'a, REG, Outcfg>;
impl<'a, REG> OutcfgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Retain last value"]
    #[inline(always)]
    pub fn retain_lastvalue(self) -> &'a mut crate::W<REG> {
        self.variant(Outcfg::RetainLastvalue)
    }
    #[doc = "3-stated"]
    #[inline(always)]
    pub fn tristated(self) -> &'a mut crate::W<REG> {
        self.variant(Outcfg::Tristated)
    }
}
#[doc = "Peripheral Chip Select Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pcscfg {
    #[doc = "0: PCS\\[3:2\\] configured for chip select function"]
    ChipSelect = 0,
    #[doc = "1: PCS\\[3:2\\] configured for half-duplex 4-bit transfers (PCS\\[3:2\\] = DATA\\[3:2\\])"]
    Halfduplex4bit = 1,
}
impl From<Pcscfg> for bool {
    #[inline(always)]
    fn from(variant: Pcscfg) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PCSCFG` reader - Peripheral Chip Select Configuration"]
pub type PcscfgR = crate::BitReader<Pcscfg>;
impl PcscfgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pcscfg {
        match self.bits {
            false => Pcscfg::ChipSelect,
            true => Pcscfg::Halfduplex4bit,
        }
    }
    #[doc = "PCS\\[3:2\\] configured for chip select function"]
    #[inline(always)]
    pub fn is_chip_select(&self) -> bool {
        *self == Pcscfg::ChipSelect
    }
    #[doc = "PCS\\[3:2\\] configured for half-duplex 4-bit transfers (PCS\\[3:2\\] = DATA\\[3:2\\])"]
    #[inline(always)]
    pub fn is_halfduplex4bit(&self) -> bool {
        *self == Pcscfg::Halfduplex4bit
    }
}
#[doc = "Field `PCSCFG` writer - Peripheral Chip Select Configuration"]
pub type PcscfgW<'a, REG> = crate::BitWriter<'a, REG, Pcscfg>;
impl<'a, REG> PcscfgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "PCS\\[3:2\\] configured for chip select function"]
    #[inline(always)]
    pub fn chip_select(self) -> &'a mut crate::W<REG> {
        self.variant(Pcscfg::ChipSelect)
    }
    #[doc = "PCS\\[3:2\\] configured for half-duplex 4-bit transfers (PCS\\[3:2\\] = DATA\\[3:2\\])"]
    #[inline(always)]
    pub fn halfduplex4bit(self) -> &'a mut crate::W<REG> {
        self.variant(Pcscfg::Halfduplex4bit)
    }
}
impl R {
    #[doc = "Bit 0 - Controller Mode"]
    #[inline(always)]
    pub fn master(&self) -> MasterR {
        MasterR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Sample Point"]
    #[inline(always)]
    pub fn sample(&self) -> SampleR {
        SampleR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Automatic PCS"]
    #[inline(always)]
    pub fn autopcs(&self) -> AutopcsR {
        AutopcsR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - No Stall"]
    #[inline(always)]
    pub fn nostall(&self) -> NostallR {
        NostallR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Partial Enable"]
    #[inline(always)]
    pub fn partial(&self) -> PartialR {
        PartialR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bits 8:11 - Peripheral Chip Select Polarity"]
    #[inline(always)]
    pub fn pcspol(&self) -> PcspolR {
        PcspolR::new(((self.bits >> 8) & 0x0f) as u8)
    }
    #[doc = "Bits 16:18 - Match Configuration"]
    #[inline(always)]
    pub fn matcfg(&self) -> MatcfgR {
        MatcfgR::new(((self.bits >> 16) & 7) as u8)
    }
    #[doc = "Bits 24:25 - Pin Configuration"]
    #[inline(always)]
    pub fn pincfg(&self) -> PincfgR {
        PincfgR::new(((self.bits >> 24) & 3) as u8)
    }
    #[doc = "Bit 26 - Output Configuration"]
    #[inline(always)]
    pub fn outcfg(&self) -> OutcfgR {
        OutcfgR::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - Peripheral Chip Select Configuration"]
    #[inline(always)]
    pub fn pcscfg(&self) -> PcscfgR {
        PcscfgR::new(((self.bits >> 27) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Controller Mode"]
    #[inline(always)]
    pub fn master(&mut self) -> MasterW<Cfgr1Spec> {
        MasterW::new(self, 0)
    }
    #[doc = "Bit 1 - Sample Point"]
    #[inline(always)]
    pub fn sample(&mut self) -> SampleW<Cfgr1Spec> {
        SampleW::new(self, 1)
    }
    #[doc = "Bit 2 - Automatic PCS"]
    #[inline(always)]
    pub fn autopcs(&mut self) -> AutopcsW<Cfgr1Spec> {
        AutopcsW::new(self, 2)
    }
    #[doc = "Bit 3 - No Stall"]
    #[inline(always)]
    pub fn nostall(&mut self) -> NostallW<Cfgr1Spec> {
        NostallW::new(self, 3)
    }
    #[doc = "Bit 4 - Partial Enable"]
    #[inline(always)]
    pub fn partial(&mut self) -> PartialW<Cfgr1Spec> {
        PartialW::new(self, 4)
    }
    #[doc = "Bits 8:11 - Peripheral Chip Select Polarity"]
    #[inline(always)]
    pub fn pcspol(&mut self) -> PcspolW<Cfgr1Spec> {
        PcspolW::new(self, 8)
    }
    #[doc = "Bits 16:18 - Match Configuration"]
    #[inline(always)]
    pub fn matcfg(&mut self) -> MatcfgW<Cfgr1Spec> {
        MatcfgW::new(self, 16)
    }
    #[doc = "Bits 24:25 - Pin Configuration"]
    #[inline(always)]
    pub fn pincfg(&mut self) -> PincfgW<Cfgr1Spec> {
        PincfgW::new(self, 24)
    }
    #[doc = "Bit 26 - Output Configuration"]
    #[inline(always)]
    pub fn outcfg(&mut self) -> OutcfgW<Cfgr1Spec> {
        OutcfgW::new(self, 26)
    }
    #[doc = "Bit 27 - Peripheral Chip Select Configuration"]
    #[inline(always)]
    pub fn pcscfg(&mut self) -> PcscfgW<Cfgr1Spec> {
        PcscfgW::new(self, 27)
    }
}
#[doc = "Configuration 1\n\nYou can [`read`](crate::Reg::read) this register and get [`cfgr1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cfgr1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Cfgr1Spec;
impl crate::RegisterSpec for Cfgr1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`cfgr1::R`](R) reader structure"]
impl crate::Readable for Cfgr1Spec {}
#[doc = "`write(|w| ..)` method takes [`cfgr1::W`](W) writer structure"]
impl crate::Writable for Cfgr1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CFGR1 to value 0"]
impl crate::Resettable for Cfgr1Spec {}

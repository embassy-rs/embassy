#[doc = "Register `TCD_CSR` reader"]
pub type R = crate::R<TcdCsrSpec>;
#[doc = "Register `TCD_CSR` writer"]
pub type W = crate::W<TcdCsrSpec>;
#[doc = "Channel Start\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Start {
    #[doc = "0: Channel not explicitly started"]
    ChannelNotStarted = 0,
    #[doc = "1: Channel explicitly started via a software-initiated service request"]
    ChannelStarted = 1,
}
impl From<Start> for bool {
    #[inline(always)]
    fn from(variant: Start) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `START` reader - Channel Start"]
pub type StartR = crate::BitReader<Start>;
impl StartR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Start {
        match self.bits {
            false => Start::ChannelNotStarted,
            true => Start::ChannelStarted,
        }
    }
    #[doc = "Channel not explicitly started"]
    #[inline(always)]
    pub fn is_channel_not_started(&self) -> bool {
        *self == Start::ChannelNotStarted
    }
    #[doc = "Channel explicitly started via a software-initiated service request"]
    #[inline(always)]
    pub fn is_channel_started(&self) -> bool {
        *self == Start::ChannelStarted
    }
}
#[doc = "Field `START` writer - Channel Start"]
pub type StartW<'a, REG> = crate::BitWriter<'a, REG, Start>;
impl<'a, REG> StartW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Channel not explicitly started"]
    #[inline(always)]
    pub fn channel_not_started(self) -> &'a mut crate::W<REG> {
        self.variant(Start::ChannelNotStarted)
    }
    #[doc = "Channel explicitly started via a software-initiated service request"]
    #[inline(always)]
    pub fn channel_started(self) -> &'a mut crate::W<REG> {
        self.variant(Start::ChannelStarted)
    }
}
#[doc = "Enable Interrupt If Major count complete\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Intmajor {
    #[doc = "0: End-of-major loop interrupt disabled"]
    Disable = 0,
    #[doc = "1: End-of-major loop interrupt enabled"]
    Enable = 1,
}
impl From<Intmajor> for bool {
    #[inline(always)]
    fn from(variant: Intmajor) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INTMAJOR` reader - Enable Interrupt If Major count complete"]
pub type IntmajorR = crate::BitReader<Intmajor>;
impl IntmajorR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Intmajor {
        match self.bits {
            false => Intmajor::Disable,
            true => Intmajor::Enable,
        }
    }
    #[doc = "End-of-major loop interrupt disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Intmajor::Disable
    }
    #[doc = "End-of-major loop interrupt enabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Intmajor::Enable
    }
}
#[doc = "Field `INTMAJOR` writer - Enable Interrupt If Major count complete"]
pub type IntmajorW<'a, REG> = crate::BitWriter<'a, REG, Intmajor>;
impl<'a, REG> IntmajorW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "End-of-major loop interrupt disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Intmajor::Disable)
    }
    #[doc = "End-of-major loop interrupt enabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Intmajor::Enable)
    }
}
#[doc = "Enable Interrupt If Major Counter Half-complete\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inthalf {
    #[doc = "0: Halfway point interrupt disabled"]
    Disable = 0,
    #[doc = "1: Halfway point interrupt enabled"]
    Enable = 1,
}
impl From<Inthalf> for bool {
    #[inline(always)]
    fn from(variant: Inthalf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INTHALF` reader - Enable Interrupt If Major Counter Half-complete"]
pub type InthalfR = crate::BitReader<Inthalf>;
impl InthalfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Inthalf {
        match self.bits {
            false => Inthalf::Disable,
            true => Inthalf::Enable,
        }
    }
    #[doc = "Halfway point interrupt disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Inthalf::Disable
    }
    #[doc = "Halfway point interrupt enabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Inthalf::Enable
    }
}
#[doc = "Field `INTHALF` writer - Enable Interrupt If Major Counter Half-complete"]
pub type InthalfW<'a, REG> = crate::BitWriter<'a, REG, Inthalf>;
impl<'a, REG> InthalfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Halfway point interrupt disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Inthalf::Disable)
    }
    #[doc = "Halfway point interrupt enabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Inthalf::Enable)
    }
}
#[doc = "Disable Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dreq {
    #[doc = "0: No operation"]
    ChannelNotAffected = 0,
    #[doc = "1: Clear the ERQ field to 0 upon major loop completion, thus disabling hardware service requests"]
    ErqFieldClear = 1,
}
impl From<Dreq> for bool {
    #[inline(always)]
    fn from(variant: Dreq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DREQ` reader - Disable Request"]
pub type DreqR = crate::BitReader<Dreq>;
impl DreqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dreq {
        match self.bits {
            false => Dreq::ChannelNotAffected,
            true => Dreq::ErqFieldClear,
        }
    }
    #[doc = "No operation"]
    #[inline(always)]
    pub fn is_channel_not_affected(&self) -> bool {
        *self == Dreq::ChannelNotAffected
    }
    #[doc = "Clear the ERQ field to 0 upon major loop completion, thus disabling hardware service requests"]
    #[inline(always)]
    pub fn is_erq_field_clear(&self) -> bool {
        *self == Dreq::ErqFieldClear
    }
}
#[doc = "Field `DREQ` writer - Disable Request"]
pub type DreqW<'a, REG> = crate::BitWriter<'a, REG, Dreq>;
impl<'a, REG> DreqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No operation"]
    #[inline(always)]
    pub fn channel_not_affected(self) -> &'a mut crate::W<REG> {
        self.variant(Dreq::ChannelNotAffected)
    }
    #[doc = "Clear the ERQ field to 0 upon major loop completion, thus disabling hardware service requests"]
    #[inline(always)]
    pub fn erq_field_clear(self) -> &'a mut crate::W<REG> {
        self.variant(Dreq::ErqFieldClear)
    }
}
#[doc = "Enable Scatter/Gather Processing\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Esg {
    #[doc = "0: Current channel's TCD is normal format"]
    NormalFormat = 0,
    #[doc = "1: Current channel's TCD specifies scatter/gather format."]
    ScatterGatherFormat = 1,
}
impl From<Esg> for bool {
    #[inline(always)]
    fn from(variant: Esg) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ESG` reader - Enable Scatter/Gather Processing"]
pub type EsgR = crate::BitReader<Esg>;
impl EsgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Esg {
        match self.bits {
            false => Esg::NormalFormat,
            true => Esg::ScatterGatherFormat,
        }
    }
    #[doc = "Current channel's TCD is normal format"]
    #[inline(always)]
    pub fn is_normal_format(&self) -> bool {
        *self == Esg::NormalFormat
    }
    #[doc = "Current channel's TCD specifies scatter/gather format."]
    #[inline(always)]
    pub fn is_scatter_gather_format(&self) -> bool {
        *self == Esg::ScatterGatherFormat
    }
}
#[doc = "Field `ESG` writer - Enable Scatter/Gather Processing"]
pub type EsgW<'a, REG> = crate::BitWriter<'a, REG, Esg>;
impl<'a, REG> EsgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Current channel's TCD is normal format"]
    #[inline(always)]
    pub fn normal_format(self) -> &'a mut crate::W<REG> {
        self.variant(Esg::NormalFormat)
    }
    #[doc = "Current channel's TCD specifies scatter/gather format."]
    #[inline(always)]
    pub fn scatter_gather_format(self) -> &'a mut crate::W<REG> {
        self.variant(Esg::ScatterGatherFormat)
    }
}
#[doc = "Enable Link When Major Loop Complete\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Majorelink {
    #[doc = "0: Channel-to-channel linking disabled"]
    Disable = 0,
    #[doc = "1: Channel-to-channel linking enabled"]
    Enable = 1,
}
impl From<Majorelink> for bool {
    #[inline(always)]
    fn from(variant: Majorelink) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MAJORELINK` reader - Enable Link When Major Loop Complete"]
pub type MajorelinkR = crate::BitReader<Majorelink>;
impl MajorelinkR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Majorelink {
        match self.bits {
            false => Majorelink::Disable,
            true => Majorelink::Enable,
        }
    }
    #[doc = "Channel-to-channel linking disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Majorelink::Disable
    }
    #[doc = "Channel-to-channel linking enabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Majorelink::Enable
    }
}
#[doc = "Field `MAJORELINK` writer - Enable Link When Major Loop Complete"]
pub type MajorelinkW<'a, REG> = crate::BitWriter<'a, REG, Majorelink>;
impl<'a, REG> MajorelinkW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Channel-to-channel linking disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Majorelink::Disable)
    }
    #[doc = "Channel-to-channel linking enabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Majorelink::Enable)
    }
}
#[doc = "Enable End-Of-Packet Processing\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Eeop {
    #[doc = "0: End-of-packet operation disabled"]
    Disable = 0,
    #[doc = "1: End-of-packet hardware input signal enabled"]
    Enable = 1,
}
impl From<Eeop> for bool {
    #[inline(always)]
    fn from(variant: Eeop) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EEOP` reader - Enable End-Of-Packet Processing"]
pub type EeopR = crate::BitReader<Eeop>;
impl EeopR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Eeop {
        match self.bits {
            false => Eeop::Disable,
            true => Eeop::Enable,
        }
    }
    #[doc = "End-of-packet operation disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Eeop::Disable
    }
    #[doc = "End-of-packet hardware input signal enabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Eeop::Enable
    }
}
#[doc = "Field `EEOP` writer - Enable End-Of-Packet Processing"]
pub type EeopW<'a, REG> = crate::BitWriter<'a, REG, Eeop>;
impl<'a, REG> EeopW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "End-of-packet operation disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Eeop::Disable)
    }
    #[doc = "End-of-packet hardware input signal enabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Eeop::Enable)
    }
}
#[doc = "Enable Store Destination Address\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Esda {
    #[doc = "0: Ability to store destination address to system memory disabled"]
    Disable = 0,
    #[doc = "1: Ability to store destination address to system memory enabled"]
    Enable = 1,
}
impl From<Esda> for bool {
    #[inline(always)]
    fn from(variant: Esda) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ESDA` reader - Enable Store Destination Address"]
pub type EsdaR = crate::BitReader<Esda>;
impl EsdaR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Esda {
        match self.bits {
            false => Esda::Disable,
            true => Esda::Enable,
        }
    }
    #[doc = "Ability to store destination address to system memory disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Esda::Disable
    }
    #[doc = "Ability to store destination address to system memory enabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Esda::Enable
    }
}
#[doc = "Field `ESDA` writer - Enable Store Destination Address"]
pub type EsdaW<'a, REG> = crate::BitWriter<'a, REG, Esda>;
impl<'a, REG> EsdaW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Ability to store destination address to system memory disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Esda::Disable)
    }
    #[doc = "Ability to store destination address to system memory enabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Esda::Enable)
    }
}
#[doc = "Field `MAJORLINKCH` reader - Major Loop Link Channel Number"]
pub type MajorlinkchR = crate::FieldReader;
#[doc = "Field `MAJORLINKCH` writer - Major Loop Link Channel Number"]
pub type MajorlinkchW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Bandwidth Control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Bwc {
    #[doc = "0: No eDMA engine stalls"]
    NoStall = 0,
    #[doc = "2: eDMA engine stalls for 4 cycles after each R/W"]
    EngineStallsFour = 2,
    #[doc = "3: eDMA engine stalls for 8 cycles after each R/W"]
    EngineStallsEight = 3,
}
impl From<Bwc> for u8 {
    #[inline(always)]
    fn from(variant: Bwc) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Bwc {
    type Ux = u8;
}
impl crate::IsEnum for Bwc {}
#[doc = "Field `BWC` reader - Bandwidth Control"]
pub type BwcR = crate::FieldReader<Bwc>;
impl BwcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Bwc> {
        match self.bits {
            0 => Some(Bwc::NoStall),
            2 => Some(Bwc::EngineStallsFour),
            3 => Some(Bwc::EngineStallsEight),
            _ => None,
        }
    }
    #[doc = "No eDMA engine stalls"]
    #[inline(always)]
    pub fn is_no_stall(&self) -> bool {
        *self == Bwc::NoStall
    }
    #[doc = "eDMA engine stalls for 4 cycles after each R/W"]
    #[inline(always)]
    pub fn is_engine_stalls_four(&self) -> bool {
        *self == Bwc::EngineStallsFour
    }
    #[doc = "eDMA engine stalls for 8 cycles after each R/W"]
    #[inline(always)]
    pub fn is_engine_stalls_eight(&self) -> bool {
        *self == Bwc::EngineStallsEight
    }
}
#[doc = "Field `BWC` writer - Bandwidth Control"]
pub type BwcW<'a, REG> = crate::FieldWriter<'a, REG, 2, Bwc>;
impl<'a, REG> BwcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "No eDMA engine stalls"]
    #[inline(always)]
    pub fn no_stall(self) -> &'a mut crate::W<REG> {
        self.variant(Bwc::NoStall)
    }
    #[doc = "eDMA engine stalls for 4 cycles after each R/W"]
    #[inline(always)]
    pub fn engine_stalls_four(self) -> &'a mut crate::W<REG> {
        self.variant(Bwc::EngineStallsFour)
    }
    #[doc = "eDMA engine stalls for 8 cycles after each R/W"]
    #[inline(always)]
    pub fn engine_stalls_eight(self) -> &'a mut crate::W<REG> {
        self.variant(Bwc::EngineStallsEight)
    }
}
impl R {
    #[doc = "Bit 0 - Channel Start"]
    #[inline(always)]
    pub fn start(&self) -> StartR {
        StartR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Enable Interrupt If Major count complete"]
    #[inline(always)]
    pub fn intmajor(&self) -> IntmajorR {
        IntmajorR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Enable Interrupt If Major Counter Half-complete"]
    #[inline(always)]
    pub fn inthalf(&self) -> InthalfR {
        InthalfR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Disable Request"]
    #[inline(always)]
    pub fn dreq(&self) -> DreqR {
        DreqR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Enable Scatter/Gather Processing"]
    #[inline(always)]
    pub fn esg(&self) -> EsgR {
        EsgR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Enable Link When Major Loop Complete"]
    #[inline(always)]
    pub fn majorelink(&self) -> MajorelinkR {
        MajorelinkR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Enable End-Of-Packet Processing"]
    #[inline(always)]
    pub fn eeop(&self) -> EeopR {
        EeopR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Enable Store Destination Address"]
    #[inline(always)]
    pub fn esda(&self) -> EsdaR {
        EsdaR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bits 8:10 - Major Loop Link Channel Number"]
    #[inline(always)]
    pub fn majorlinkch(&self) -> MajorlinkchR {
        MajorlinkchR::new(((self.bits >> 8) & 7) as u8)
    }
    #[doc = "Bits 14:15 - Bandwidth Control"]
    #[inline(always)]
    pub fn bwc(&self) -> BwcR {
        BwcR::new(((self.bits >> 14) & 3) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - Channel Start"]
    #[inline(always)]
    pub fn start(&mut self) -> StartW<TcdCsrSpec> {
        StartW::new(self, 0)
    }
    #[doc = "Bit 1 - Enable Interrupt If Major count complete"]
    #[inline(always)]
    pub fn intmajor(&mut self) -> IntmajorW<TcdCsrSpec> {
        IntmajorW::new(self, 1)
    }
    #[doc = "Bit 2 - Enable Interrupt If Major Counter Half-complete"]
    #[inline(always)]
    pub fn inthalf(&mut self) -> InthalfW<TcdCsrSpec> {
        InthalfW::new(self, 2)
    }
    #[doc = "Bit 3 - Disable Request"]
    #[inline(always)]
    pub fn dreq(&mut self) -> DreqW<TcdCsrSpec> {
        DreqW::new(self, 3)
    }
    #[doc = "Bit 4 - Enable Scatter/Gather Processing"]
    #[inline(always)]
    pub fn esg(&mut self) -> EsgW<TcdCsrSpec> {
        EsgW::new(self, 4)
    }
    #[doc = "Bit 5 - Enable Link When Major Loop Complete"]
    #[inline(always)]
    pub fn majorelink(&mut self) -> MajorelinkW<TcdCsrSpec> {
        MajorelinkW::new(self, 5)
    }
    #[doc = "Bit 6 - Enable End-Of-Packet Processing"]
    #[inline(always)]
    pub fn eeop(&mut self) -> EeopW<TcdCsrSpec> {
        EeopW::new(self, 6)
    }
    #[doc = "Bit 7 - Enable Store Destination Address"]
    #[inline(always)]
    pub fn esda(&mut self) -> EsdaW<TcdCsrSpec> {
        EsdaW::new(self, 7)
    }
    #[doc = "Bits 8:10 - Major Loop Link Channel Number"]
    #[inline(always)]
    pub fn majorlinkch(&mut self) -> MajorlinkchW<TcdCsrSpec> {
        MajorlinkchW::new(self, 8)
    }
    #[doc = "Bits 14:15 - Bandwidth Control"]
    #[inline(always)]
    pub fn bwc(&mut self) -> BwcW<TcdCsrSpec> {
        BwcW::new(self, 14)
    }
}
#[doc = "TCD Control and Status\n\nYou can [`read`](crate::Reg::read) this register and get [`tcd_csr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tcd_csr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TcdCsrSpec;
impl crate::RegisterSpec for TcdCsrSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`tcd_csr::R`](R) reader structure"]
impl crate::Readable for TcdCsrSpec {}
#[doc = "`write(|w| ..)` method takes [`tcd_csr::W`](W) writer structure"]
impl crate::Writable for TcdCsrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TCD_CSR to value 0"]
impl crate::Resettable for TcdCsrSpec {}

#[doc = "Register `CTRL2` reader"]
pub type R = crate::R<Ctrl2Spec>;
#[doc = "Register `CTRL2` writer"]
pub type W = crate::W<Ctrl2Spec>;
#[doc = "Payload Byte and Bit Order Selection\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pes {
    #[doc = "0: Big-endian"]
    BigEnd = 0,
    #[doc = "1: Little-endian"]
    LittleEnd = 1,
}
impl From<Pes> for bool {
    #[inline(always)]
    fn from(variant: Pes) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PES` reader - Payload Byte and Bit Order Selection"]
pub type PesR = crate::BitReader<Pes>;
impl PesR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pes {
        match self.bits {
            false => Pes::BigEnd,
            true => Pes::LittleEnd,
        }
    }
    #[doc = "Big-endian"]
    #[inline(always)]
    pub fn is_big_end(&self) -> bool {
        *self == Pes::BigEnd
    }
    #[doc = "Little-endian"]
    #[inline(always)]
    pub fn is_little_end(&self) -> bool {
        *self == Pes::LittleEnd
    }
}
#[doc = "Field `PES` writer - Payload Byte and Bit Order Selection"]
pub type PesW<'a, REG> = crate::BitWriter<'a, REG, Pes>;
impl<'a, REG> PesW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Big-endian"]
    #[inline(always)]
    pub fn big_end(self) -> &'a mut crate::W<REG> {
        self.variant(Pes::BigEnd)
    }
    #[doc = "Little-endian"]
    #[inline(always)]
    pub fn little_end(self) -> &'a mut crate::W<REG> {
        self.variant(Pes::LittleEnd)
    }
}
#[doc = "ACK Suppression Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Asd {
    #[doc = "0: Enabled"]
    Enable = 0,
    #[doc = "1: Disabled"]
    Disable = 1,
}
impl From<Asd> for bool {
    #[inline(always)]
    fn from(variant: Asd) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ASD` reader - ACK Suppression Disable"]
pub type AsdR = crate::BitReader<Asd>;
impl AsdR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Asd {
        match self.bits {
            false => Asd::Enable,
            true => Asd::Disable,
        }
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Asd::Enable
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Asd::Disable
    }
}
#[doc = "Field `ASD` writer - ACK Suppression Disable"]
pub type AsdW<'a, REG> = crate::BitWriter<'a, REG, Asd>;
impl<'a, REG> AsdW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Asd::Enable)
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Asd::Disable)
    }
}
#[doc = "Edge Filter Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edfltdis {
    #[doc = "0: Enabled"]
    Enable = 0,
    #[doc = "1: Disabled"]
    Disable = 1,
}
impl From<Edfltdis> for bool {
    #[inline(always)]
    fn from(variant: Edfltdis) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EDFLTDIS` reader - Edge Filter Disable"]
pub type EdfltdisR = crate::BitReader<Edfltdis>;
impl EdfltdisR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Edfltdis {
        match self.bits {
            false => Edfltdis::Enable,
            true => Edfltdis::Disable,
        }
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Edfltdis::Enable
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Edfltdis::Disable
    }
}
#[doc = "Field `EDFLTDIS` writer - Edge Filter Disable"]
pub type EdfltdisW<'a, REG> = crate::BitWriter<'a, REG, Edfltdis>;
impl<'a, REG> EdfltdisW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Edfltdis::Enable)
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Edfltdis::Disable)
    }
}
#[doc = "ISO CAN FD Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Isocanfden {
    #[doc = "0: Disable"]
    NonIso = 0,
    #[doc = "1: Enable"]
    Iso = 1,
}
impl From<Isocanfden> for bool {
    #[inline(always)]
    fn from(variant: Isocanfden) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ISOCANFDEN` reader - ISO CAN FD Enable"]
pub type IsocanfdenR = crate::BitReader<Isocanfden>;
impl IsocanfdenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Isocanfden {
        match self.bits {
            false => Isocanfden::NonIso,
            true => Isocanfden::Iso,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_non_iso(&self) -> bool {
        *self == Isocanfden::NonIso
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_iso(&self) -> bool {
        *self == Isocanfden::Iso
    }
}
#[doc = "Field `ISOCANFDEN` writer - ISO CAN FD Enable"]
pub type IsocanfdenW<'a, REG> = crate::BitWriter<'a, REG, Isocanfden>;
impl<'a, REG> IsocanfdenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn non_iso(self) -> &'a mut crate::W<REG> {
        self.variant(Isocanfden::NonIso)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn iso(self) -> &'a mut crate::W<REG> {
        self.variant(Isocanfden::Iso)
    }
}
#[doc = "Bit Timing Expansion Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bte {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Bte> for bool {
    #[inline(always)]
    fn from(variant: Bte) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BTE` reader - Bit Timing Expansion Enable"]
pub type BteR = crate::BitReader<Bte>;
impl BteR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Bte {
        match self.bits {
            false => Bte::Disable,
            true => Bte::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Bte::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Bte::Enable
    }
}
#[doc = "Field `BTE` writer - Bit Timing Expansion Enable"]
pub type BteW<'a, REG> = crate::BitWriter<'a, REG, Bte>;
impl<'a, REG> BteW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Bte::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Bte::Enable)
    }
}
#[doc = "Protocol Exception Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prexcen {
    #[doc = "0: Disabled"]
    Disable = 0,
    #[doc = "1: Enabled"]
    Enable = 1,
}
impl From<Prexcen> for bool {
    #[inline(always)]
    fn from(variant: Prexcen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PREXCEN` reader - Protocol Exception Enable"]
pub type PrexcenR = crate::BitReader<Prexcen>;
impl PrexcenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Prexcen {
        match self.bits {
            false => Prexcen::Disable,
            true => Prexcen::Enable,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Prexcen::Disable
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Prexcen::Enable
    }
}
#[doc = "Field `PREXCEN` writer - Protocol Exception Enable"]
pub type PrexcenW<'a, REG> = crate::BitWriter<'a, REG, Prexcen>;
impl<'a, REG> PrexcenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Prexcen::Disable)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Prexcen::Enable)
    }
}
#[doc = "Entire Frame Arbitration Field Comparison Enable for RX Message Buffers\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Eacen {
    #[doc = "0: Disable"]
    RtrCompareNo = 0,
    #[doc = "1: Enable"]
    RtrCompareYes = 1,
}
impl From<Eacen> for bool {
    #[inline(always)]
    fn from(variant: Eacen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EACEN` reader - Entire Frame Arbitration Field Comparison Enable for RX Message Buffers"]
pub type EacenR = crate::BitReader<Eacen>;
impl EacenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Eacen {
        match self.bits {
            false => Eacen::RtrCompareNo,
            true => Eacen::RtrCompareYes,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_rtr_compare_no(&self) -> bool {
        *self == Eacen::RtrCompareNo
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_rtr_compare_yes(&self) -> bool {
        *self == Eacen::RtrCompareYes
    }
}
#[doc = "Field `EACEN` writer - Entire Frame Arbitration Field Comparison Enable for RX Message Buffers"]
pub type EacenW<'a, REG> = crate::BitWriter<'a, REG, Eacen>;
impl<'a, REG> EacenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn rtr_compare_no(self) -> &'a mut crate::W<REG> {
        self.variant(Eacen::RtrCompareNo)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn rtr_compare_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Eacen::RtrCompareYes)
    }
}
#[doc = "Remote Request Storing\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rrs {
    #[doc = "0: Generated"]
    RemoteResponseFrameNotGenerated = 0,
    #[doc = "1: Stored"]
    RemoteResponseFrameGenerated = 1,
}
impl From<Rrs> for bool {
    #[inline(always)]
    fn from(variant: Rrs) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RRS` reader - Remote Request Storing"]
pub type RrsR = crate::BitReader<Rrs>;
impl RrsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rrs {
        match self.bits {
            false => Rrs::RemoteResponseFrameNotGenerated,
            true => Rrs::RemoteResponseFrameGenerated,
        }
    }
    #[doc = "Generated"]
    #[inline(always)]
    pub fn is_remote_response_frame_not_generated(&self) -> bool {
        *self == Rrs::RemoteResponseFrameNotGenerated
    }
    #[doc = "Stored"]
    #[inline(always)]
    pub fn is_remote_response_frame_generated(&self) -> bool {
        *self == Rrs::RemoteResponseFrameGenerated
    }
}
#[doc = "Field `RRS` writer - Remote Request Storing"]
pub type RrsW<'a, REG> = crate::BitWriter<'a, REG, Rrs>;
impl<'a, REG> RrsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Generated"]
    #[inline(always)]
    pub fn remote_response_frame_not_generated(self) -> &'a mut crate::W<REG> {
        self.variant(Rrs::RemoteResponseFrameNotGenerated)
    }
    #[doc = "Stored"]
    #[inline(always)]
    pub fn remote_response_frame_generated(self) -> &'a mut crate::W<REG> {
        self.variant(Rrs::RemoteResponseFrameGenerated)
    }
}
#[doc = "Message Buffers Reception Priority\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mrp {
    #[doc = "0: Matching starts from Legacy RX FIFO or Enhanced RX FIFO and continues on message buffers."]
    Id1 = 0,
    #[doc = "1: Matching starts from message buffers and continues on Legacy RX FIFO or Enhanced RX FIFO."]
    Id3 = 1,
}
impl From<Mrp> for bool {
    #[inline(always)]
    fn from(variant: Mrp) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MRP` reader - Message Buffers Reception Priority"]
pub type MrpR = crate::BitReader<Mrp>;
impl MrpR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mrp {
        match self.bits {
            false => Mrp::Id1,
            true => Mrp::Id3,
        }
    }
    #[doc = "Matching starts from Legacy RX FIFO or Enhanced RX FIFO and continues on message buffers."]
    #[inline(always)]
    pub fn is_id1(&self) -> bool {
        *self == Mrp::Id1
    }
    #[doc = "Matching starts from message buffers and continues on Legacy RX FIFO or Enhanced RX FIFO."]
    #[inline(always)]
    pub fn is_id3(&self) -> bool {
        *self == Mrp::Id3
    }
}
#[doc = "Field `MRP` writer - Message Buffers Reception Priority"]
pub type MrpW<'a, REG> = crate::BitWriter<'a, REG, Mrp>;
impl<'a, REG> MrpW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Matching starts from Legacy RX FIFO or Enhanced RX FIFO and continues on message buffers."]
    #[inline(always)]
    pub fn id1(self) -> &'a mut crate::W<REG> {
        self.variant(Mrp::Id1)
    }
    #[doc = "Matching starts from message buffers and continues on Legacy RX FIFO or Enhanced RX FIFO."]
    #[inline(always)]
    pub fn id3(self) -> &'a mut crate::W<REG> {
        self.variant(Mrp::Id3)
    }
}
#[doc = "Field `TASD` reader - Transmission Arbitration Start Delay"]
pub type TasdR = crate::FieldReader;
#[doc = "Field `TASD` writer - Transmission Arbitration Start Delay"]
pub type TasdW<'a, REG> = crate::FieldWriter<'a, REG, 5>;
#[doc = "Field `RFFN` reader - Number of Legacy Receive FIFO Filters"]
pub type RffnR = crate::FieldReader;
#[doc = "Field `RFFN` writer - Number of Legacy Receive FIFO Filters"]
pub type RffnW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Bus Off Done Interrupt Mask\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boffdonemsk {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Boffdonemsk> for bool {
    #[inline(always)]
    fn from(variant: Boffdonemsk) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BOFFDONEMSK` reader - Bus Off Done Interrupt Mask"]
pub type BoffdonemskR = crate::BitReader<Boffdonemsk>;
impl BoffdonemskR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Boffdonemsk {
        match self.bits {
            false => Boffdonemsk::Disable,
            true => Boffdonemsk::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Boffdonemsk::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Boffdonemsk::Enable
    }
}
#[doc = "Field `BOFFDONEMSK` writer - Bus Off Done Interrupt Mask"]
pub type BoffdonemskW<'a, REG> = crate::BitWriter<'a, REG, Boffdonemsk>;
impl<'a, REG> BoffdonemskW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Boffdonemsk::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Boffdonemsk::Enable)
    }
}
#[doc = "Error Interrupt Mask for Errors Detected in the Data Phase of Fast CAN FD Frames\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrmskFast {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<ErrmskFast> for bool {
    #[inline(always)]
    fn from(variant: ErrmskFast) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERRMSK_FAST` reader - Error Interrupt Mask for Errors Detected in the Data Phase of Fast CAN FD Frames"]
pub type ErrmskFastR = crate::BitReader<ErrmskFast>;
impl ErrmskFastR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> ErrmskFast {
        match self.bits {
            false => ErrmskFast::Disable,
            true => ErrmskFast::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == ErrmskFast::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == ErrmskFast::Enable
    }
}
#[doc = "Field `ERRMSK_FAST` writer - Error Interrupt Mask for Errors Detected in the Data Phase of Fast CAN FD Frames"]
pub type ErrmskFastW<'a, REG> = crate::BitWriter<'a, REG, ErrmskFast>;
impl<'a, REG> ErrmskFastW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(ErrmskFast::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(ErrmskFast::Enable)
    }
}
impl R {
    #[doc = "Bit 0 - Payload Byte and Bit Order Selection"]
    #[inline(always)]
    pub fn pes(&self) -> PesR {
        PesR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - ACK Suppression Disable"]
    #[inline(always)]
    pub fn asd(&self) -> AsdR {
        AsdR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 11 - Edge Filter Disable"]
    #[inline(always)]
    pub fn edfltdis(&self) -> EdfltdisR {
        EdfltdisR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - ISO CAN FD Enable"]
    #[inline(always)]
    pub fn isocanfden(&self) -> IsocanfdenR {
        IsocanfdenR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Bit Timing Expansion Enable"]
    #[inline(always)]
    pub fn bte(&self) -> BteR {
        BteR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Protocol Exception Enable"]
    #[inline(always)]
    pub fn prexcen(&self) -> PrexcenR {
        PrexcenR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 16 - Entire Frame Arbitration Field Comparison Enable for RX Message Buffers"]
    #[inline(always)]
    pub fn eacen(&self) -> EacenR {
        EacenR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Remote Request Storing"]
    #[inline(always)]
    pub fn rrs(&self) -> RrsR {
        RrsR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Message Buffers Reception Priority"]
    #[inline(always)]
    pub fn mrp(&self) -> MrpR {
        MrpR::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bits 19:23 - Transmission Arbitration Start Delay"]
    #[inline(always)]
    pub fn tasd(&self) -> TasdR {
        TasdR::new(((self.bits >> 19) & 0x1f) as u8)
    }
    #[doc = "Bits 24:27 - Number of Legacy Receive FIFO Filters"]
    #[inline(always)]
    pub fn rffn(&self) -> RffnR {
        RffnR::new(((self.bits >> 24) & 0x0f) as u8)
    }
    #[doc = "Bit 30 - Bus Off Done Interrupt Mask"]
    #[inline(always)]
    pub fn boffdonemsk(&self) -> BoffdonemskR {
        BoffdonemskR::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Error Interrupt Mask for Errors Detected in the Data Phase of Fast CAN FD Frames"]
    #[inline(always)]
    pub fn errmsk_fast(&self) -> ErrmskFastR {
        ErrmskFastR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Payload Byte and Bit Order Selection"]
    #[inline(always)]
    pub fn pes(&mut self) -> PesW<Ctrl2Spec> {
        PesW::new(self, 0)
    }
    #[doc = "Bit 1 - ACK Suppression Disable"]
    #[inline(always)]
    pub fn asd(&mut self) -> AsdW<Ctrl2Spec> {
        AsdW::new(self, 1)
    }
    #[doc = "Bit 11 - Edge Filter Disable"]
    #[inline(always)]
    pub fn edfltdis(&mut self) -> EdfltdisW<Ctrl2Spec> {
        EdfltdisW::new(self, 11)
    }
    #[doc = "Bit 12 - ISO CAN FD Enable"]
    #[inline(always)]
    pub fn isocanfden(&mut self) -> IsocanfdenW<Ctrl2Spec> {
        IsocanfdenW::new(self, 12)
    }
    #[doc = "Bit 13 - Bit Timing Expansion Enable"]
    #[inline(always)]
    pub fn bte(&mut self) -> BteW<Ctrl2Spec> {
        BteW::new(self, 13)
    }
    #[doc = "Bit 14 - Protocol Exception Enable"]
    #[inline(always)]
    pub fn prexcen(&mut self) -> PrexcenW<Ctrl2Spec> {
        PrexcenW::new(self, 14)
    }
    #[doc = "Bit 16 - Entire Frame Arbitration Field Comparison Enable for RX Message Buffers"]
    #[inline(always)]
    pub fn eacen(&mut self) -> EacenW<Ctrl2Spec> {
        EacenW::new(self, 16)
    }
    #[doc = "Bit 17 - Remote Request Storing"]
    #[inline(always)]
    pub fn rrs(&mut self) -> RrsW<Ctrl2Spec> {
        RrsW::new(self, 17)
    }
    #[doc = "Bit 18 - Message Buffers Reception Priority"]
    #[inline(always)]
    pub fn mrp(&mut self) -> MrpW<Ctrl2Spec> {
        MrpW::new(self, 18)
    }
    #[doc = "Bits 19:23 - Transmission Arbitration Start Delay"]
    #[inline(always)]
    pub fn tasd(&mut self) -> TasdW<Ctrl2Spec> {
        TasdW::new(self, 19)
    }
    #[doc = "Bits 24:27 - Number of Legacy Receive FIFO Filters"]
    #[inline(always)]
    pub fn rffn(&mut self) -> RffnW<Ctrl2Spec> {
        RffnW::new(self, 24)
    }
    #[doc = "Bit 30 - Bus Off Done Interrupt Mask"]
    #[inline(always)]
    pub fn boffdonemsk(&mut self) -> BoffdonemskW<Ctrl2Spec> {
        BoffdonemskW::new(self, 30)
    }
    #[doc = "Bit 31 - Error Interrupt Mask for Errors Detected in the Data Phase of Fast CAN FD Frames"]
    #[inline(always)]
    pub fn errmsk_fast(&mut self) -> ErrmskFastW<Ctrl2Spec> {
        ErrmskFastW::new(self, 31)
    }
}
#[doc = "Control 2\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl2::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl2::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ctrl2Spec;
impl crate::RegisterSpec for Ctrl2Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ctrl2::R`](R) reader structure"]
impl crate::Readable for Ctrl2Spec {}
#[doc = "`write(|w| ..)` method takes [`ctrl2::W`](W) writer structure"]
impl crate::Writable for Ctrl2Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CTRL2 to value 0x00a0_0000"]
impl crate::Resettable for Ctrl2Spec {
    const RESET_VALUE: u32 = 0x00a0_0000;
}

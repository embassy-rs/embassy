#[doc = "Register `SCAPABILITIES` reader"]
pub type R = crate::R<ScapabilitiesSpec>;
#[doc = "ID 48b Handler\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Idena {
    #[doc = "0: Application"]
    Application = 0,
    #[doc = "1: Hardware"]
    Hw = 1,
    #[doc = "2: Hardware, but the I3C module instance handles ID 48b"]
    HwBut = 2,
    #[doc = "3: A part number register (PARTNO)"]
    Partno = 3,
}
impl From<Idena> for u8 {
    #[inline(always)]
    fn from(variant: Idena) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Idena {
    type Ux = u8;
}
impl crate::IsEnum for Idena {}
#[doc = "Field `IDENA` reader - ID 48b Handler"]
pub type IdenaR = crate::FieldReader<Idena>;
impl IdenaR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Idena {
        match self.bits {
            0 => Idena::Application,
            1 => Idena::Hw,
            2 => Idena::HwBut,
            3 => Idena::Partno,
            _ => unreachable!(),
        }
    }
    #[doc = "Application"]
    #[inline(always)]
    pub fn is_application(&self) -> bool {
        *self == Idena::Application
    }
    #[doc = "Hardware"]
    #[inline(always)]
    pub fn is_hw(&self) -> bool {
        *self == Idena::Hw
    }
    #[doc = "Hardware, but the I3C module instance handles ID 48b"]
    #[inline(always)]
    pub fn is_hw_but(&self) -> bool {
        *self == Idena::HwBut
    }
    #[doc = "A part number register (PARTNO)"]
    #[inline(always)]
    pub fn is_partno(&self) -> bool {
        *self == Idena::Partno
    }
}
#[doc = "ID Register\n\nValue on reset: 12"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Idreg {
    #[doc = "0: All ID register features disabled"]
    AllDisabled = 0,
    #[doc = "1: ID Instance is a register; used if there is no PARTNO register"]
    IdInstance = 1,
}
impl From<Idreg> for u8 {
    #[inline(always)]
    fn from(variant: Idreg) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Idreg {
    type Ux = u8;
}
impl crate::IsEnum for Idreg {}
#[doc = "Field `IDREG` reader - ID Register"]
pub type IdregR = crate::FieldReader<Idreg>;
impl IdregR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Idreg> {
        match self.bits {
            0 => Some(Idreg::AllDisabled),
            1 => Some(Idreg::IdInstance),
            _ => None,
        }
    }
    #[doc = "All ID register features disabled"]
    #[inline(always)]
    pub fn is_all_disabled(&self) -> bool {
        *self == Idreg::AllDisabled
    }
    #[doc = "ID Instance is a register; used if there is no PARTNO register"]
    #[inline(always)]
    pub fn is_id_instance(&self) -> bool {
        *self == Idreg::IdInstance
    }
}
#[doc = "High Data Rate Support\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Hdrsupp {
    #[doc = "0: No HDR modes supported"]
    NoHdr = 0,
    #[doc = "1: DDR mode supported"]
    Ddr = 1,
}
impl From<Hdrsupp> for u8 {
    #[inline(always)]
    fn from(variant: Hdrsupp) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Hdrsupp {
    type Ux = u8;
}
impl crate::IsEnum for Hdrsupp {}
#[doc = "Field `HDRSUPP` reader - High Data Rate Support"]
pub type HdrsuppR = crate::FieldReader<Hdrsupp>;
impl HdrsuppR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Hdrsupp> {
        match self.bits {
            0 => Some(Hdrsupp::NoHdr),
            1 => Some(Hdrsupp::Ddr),
            _ => None,
        }
    }
    #[doc = "No HDR modes supported"]
    #[inline(always)]
    pub fn is_no_hdr(&self) -> bool {
        *self == Hdrsupp::NoHdr
    }
    #[doc = "DDR mode supported"]
    #[inline(always)]
    pub fn is_ddr(&self) -> bool {
        *self == Hdrsupp::Ddr
    }
}
#[doc = "Controller\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Master {
    #[doc = "0: Not supported"]
    Masternotsupported = 0,
    #[doc = "1: Supported"]
    Mastersupported = 1,
}
impl From<Master> for bool {
    #[inline(always)]
    fn from(variant: Master) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MASTER` reader - Controller"]
pub type MasterR = crate::BitReader<Master>;
impl MasterR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Master {
        match self.bits {
            false => Master::Masternotsupported,
            true => Master::Mastersupported,
        }
    }
    #[doc = "Not supported"]
    #[inline(always)]
    pub fn is_masternotsupported(&self) -> bool {
        *self == Master::Masternotsupported
    }
    #[doc = "Supported"]
    #[inline(always)]
    pub fn is_mastersupported(&self) -> bool {
        *self == Master::Mastersupported
    }
}
#[doc = "Static Address\n\nValue on reset: 3"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Saddr {
    #[doc = "0: No static address"]
    NoStatic = 0,
    #[doc = "1: Static address is fixed in hardware"]
    Static = 1,
    #[doc = "2: Hardware controls the static address dynamically (for example, from the pin strap)"]
    HwControl = 2,
    #[doc = "3: SCONFIG register supplies the static address"]
    Config = 3,
}
impl From<Saddr> for u8 {
    #[inline(always)]
    fn from(variant: Saddr) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Saddr {
    type Ux = u8;
}
impl crate::IsEnum for Saddr {}
#[doc = "Field `SADDR` reader - Static Address"]
pub type SaddrR = crate::FieldReader<Saddr>;
impl SaddrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Saddr {
        match self.bits {
            0 => Saddr::NoStatic,
            1 => Saddr::Static,
            2 => Saddr::HwControl,
            3 => Saddr::Config,
            _ => unreachable!(),
        }
    }
    #[doc = "No static address"]
    #[inline(always)]
    pub fn is_no_static(&self) -> bool {
        *self == Saddr::NoStatic
    }
    #[doc = "Static address is fixed in hardware"]
    #[inline(always)]
    pub fn is_static(&self) -> bool {
        *self == Saddr::Static
    }
    #[doc = "Hardware controls the static address dynamically (for example, from the pin strap)"]
    #[inline(always)]
    pub fn is_hw_control(&self) -> bool {
        *self == Saddr::HwControl
    }
    #[doc = "SCONFIG register supplies the static address"]
    #[inline(always)]
    pub fn is_config(&self) -> bool {
        *self == Saddr::Config
    }
}
#[doc = "Common Command Codes Handling\n\nValue on reset: 15"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Ccchandle {
    #[doc = "0: All handling features disabled"]
    AllDisabled = 0,
    #[doc = "1: The I3C module manages events, activities, status, HDR, and if enabled for it, ID and static-address-related items"]
    BlockHandle = 1,
}
impl From<Ccchandle> for u8 {
    #[inline(always)]
    fn from(variant: Ccchandle) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Ccchandle {
    type Ux = u8;
}
impl crate::IsEnum for Ccchandle {}
#[doc = "Field `CCCHANDLE` reader - Common Command Codes Handling"]
pub type CcchandleR = crate::FieldReader<Ccchandle>;
impl CcchandleR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Ccchandle> {
        match self.bits {
            0 => Some(Ccchandle::AllDisabled),
            1 => Some(Ccchandle::BlockHandle),
            _ => None,
        }
    }
    #[doc = "All handling features disabled"]
    #[inline(always)]
    pub fn is_all_disabled(&self) -> bool {
        *self == Ccchandle::AllDisabled
    }
    #[doc = "The I3C module manages events, activities, status, HDR, and if enabled for it, ID and static-address-related items"]
    #[inline(always)]
    pub fn is_block_handle(&self) -> bool {
        *self == Ccchandle::BlockHandle
    }
}
#[doc = "In-Band Interrupts, Controller Requests, Hot-Join Events\n\nValue on reset: 31"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum IbiMrHj {
    #[doc = "0: Application cannot generate IBI, CR, or HJ"]
    AllDisabled = 0,
    #[doc = "1: Application can generate an IBI"]
    Ibi = 1,
}
impl From<IbiMrHj> for u8 {
    #[inline(always)]
    fn from(variant: IbiMrHj) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for IbiMrHj {
    type Ux = u8;
}
impl crate::IsEnum for IbiMrHj {}
#[doc = "Field `IBI_MR_HJ` reader - In-Band Interrupts, Controller Requests, Hot-Join Events"]
pub type IbiMrHjR = crate::FieldReader<IbiMrHj>;
impl IbiMrHjR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<IbiMrHj> {
        match self.bits {
            0 => Some(IbiMrHj::AllDisabled),
            1 => Some(IbiMrHj::Ibi),
            _ => None,
        }
    }
    #[doc = "Application cannot generate IBI, CR, or HJ"]
    #[inline(always)]
    pub fn is_all_disabled(&self) -> bool {
        *self == IbiMrHj::AllDisabled
    }
    #[doc = "Application can generate an IBI"]
    #[inline(always)]
    pub fn is_ibi(&self) -> bool {
        *self == IbiMrHj::Ibi
    }
}
#[doc = "Time Control\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Timectrl {
    #[doc = "0: No time control supported"]
    NoTimeControlType = 0,
    #[doc = "1: At least one time-control type supported"]
    Atleast1TimeControl = 1,
}
impl From<Timectrl> for bool {
    #[inline(always)]
    fn from(variant: Timectrl) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIMECTRL` reader - Time Control"]
pub type TimectrlR = crate::BitReader<Timectrl>;
impl TimectrlR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Timectrl {
        match self.bits {
            false => Timectrl::NoTimeControlType,
            true => Timectrl::Atleast1TimeControl,
        }
    }
    #[doc = "No time control supported"]
    #[inline(always)]
    pub fn is_no_time_control_type(&self) -> bool {
        *self == Timectrl::NoTimeControlType
    }
    #[doc = "At least one time-control type supported"]
    #[inline(always)]
    pub fn is_atleast1_time_control(&self) -> bool {
        *self == Timectrl::Atleast1TimeControl
    }
}
#[doc = "External FIFO\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Extfifo {
    #[doc = "0: No external FIFO available"]
    NoExtFifo = 0,
    #[doc = "1: Standard available or free external FIFO"]
    StdExtFifo = 1,
    #[doc = "2: Request track external FIFO"]
    RequestExtFifo = 2,
}
impl From<Extfifo> for u8 {
    #[inline(always)]
    fn from(variant: Extfifo) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Extfifo {
    type Ux = u8;
}
impl crate::IsEnum for Extfifo {}
#[doc = "Field `EXTFIFO` reader - External FIFO"]
pub type ExtfifoR = crate::FieldReader<Extfifo>;
impl ExtfifoR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Extfifo> {
        match self.bits {
            0 => Some(Extfifo::NoExtFifo),
            1 => Some(Extfifo::StdExtFifo),
            2 => Some(Extfifo::RequestExtFifo),
            _ => None,
        }
    }
    #[doc = "No external FIFO available"]
    #[inline(always)]
    pub fn is_no_ext_fifo(&self) -> bool {
        *self == Extfifo::NoExtFifo
    }
    #[doc = "Standard available or free external FIFO"]
    #[inline(always)]
    pub fn is_std_ext_fifo(&self) -> bool {
        *self == Extfifo::StdExtFifo
    }
    #[doc = "Request track external FIFO"]
    #[inline(always)]
    pub fn is_request_ext_fifo(&self) -> bool {
        *self == Extfifo::RequestExtFifo
    }
}
#[doc = "FIFO Transmit\n\nValue on reset: 2"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Fifotx {
    #[doc = "0: Two"]
    Fifo2byte = 0,
    #[doc = "1: Four"]
    Fifo4byte = 1,
    #[doc = "2: Eight"]
    Fifo8byte = 2,
    #[doc = "3: 16 or larger"]
    Fifo16byte = 3,
}
impl From<Fifotx> for u8 {
    #[inline(always)]
    fn from(variant: Fifotx) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Fifotx {
    type Ux = u8;
}
impl crate::IsEnum for Fifotx {}
#[doc = "Field `FIFOTX` reader - FIFO Transmit"]
pub type FifotxR = crate::FieldReader<Fifotx>;
impl FifotxR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fifotx {
        match self.bits {
            0 => Fifotx::Fifo2byte,
            1 => Fifotx::Fifo4byte,
            2 => Fifotx::Fifo8byte,
            3 => Fifotx::Fifo16byte,
            _ => unreachable!(),
        }
    }
    #[doc = "Two"]
    #[inline(always)]
    pub fn is_fifo_2byte(&self) -> bool {
        *self == Fifotx::Fifo2byte
    }
    #[doc = "Four"]
    #[inline(always)]
    pub fn is_fifo_4byte(&self) -> bool {
        *self == Fifotx::Fifo4byte
    }
    #[doc = "Eight"]
    #[inline(always)]
    pub fn is_fifo_8byte(&self) -> bool {
        *self == Fifotx::Fifo8byte
    }
    #[doc = "16 or larger"]
    #[inline(always)]
    pub fn is_fifo_16byte(&self) -> bool {
        *self == Fifotx::Fifo16byte
    }
}
#[doc = "FIFO Receive\n\nValue on reset: 2"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Fiforx {
    #[doc = "0: Two or three"]
    Fifo2byte = 0,
    #[doc = "1: Four"]
    Fifo4byte = 1,
    #[doc = "2: Eight"]
    Fifo8byte = 2,
    #[doc = "3: 16 or larger"]
    Fifo16byte = 3,
}
impl From<Fiforx> for u8 {
    #[inline(always)]
    fn from(variant: Fiforx) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Fiforx {
    type Ux = u8;
}
impl crate::IsEnum for Fiforx {}
#[doc = "Field `FIFORX` reader - FIFO Receive"]
pub type FiforxR = crate::FieldReader<Fiforx>;
impl FiforxR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fiforx {
        match self.bits {
            0 => Fiforx::Fifo2byte,
            1 => Fiforx::Fifo4byte,
            2 => Fiforx::Fifo8byte,
            3 => Fiforx::Fifo16byte,
            _ => unreachable!(),
        }
    }
    #[doc = "Two or three"]
    #[inline(always)]
    pub fn is_fifo_2byte(&self) -> bool {
        *self == Fiforx::Fifo2byte
    }
    #[doc = "Four"]
    #[inline(always)]
    pub fn is_fifo_4byte(&self) -> bool {
        *self == Fiforx::Fifo4byte
    }
    #[doc = "Eight"]
    #[inline(always)]
    pub fn is_fifo_8byte(&self) -> bool {
        *self == Fiforx::Fifo8byte
    }
    #[doc = "16 or larger"]
    #[inline(always)]
    pub fn is_fifo_16byte(&self) -> bool {
        *self == Fiforx::Fifo16byte
    }
}
#[doc = "Interrupts\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Int {
    #[doc = "0: Not supported"]
    Interruptsno = 0,
    #[doc = "1: Supported"]
    Interruptsyes = 1,
}
impl From<Int> for bool {
    #[inline(always)]
    fn from(variant: Int) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT` reader - Interrupts"]
pub type IntR = crate::BitReader<Int>;
impl IntR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Int {
        match self.bits {
            false => Int::Interruptsno,
            true => Int::Interruptsyes,
        }
    }
    #[doc = "Not supported"]
    #[inline(always)]
    pub fn is_interruptsno(&self) -> bool {
        *self == Int::Interruptsno
    }
    #[doc = "Supported"]
    #[inline(always)]
    pub fn is_interruptsyes(&self) -> bool {
        *self == Int::Interruptsyes
    }
}
#[doc = "Direct Memory Access\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dma {
    #[doc = "0: Not supported"]
    Dmano = 0,
    #[doc = "1: Supported"]
    Dmayes = 1,
}
impl From<Dma> for bool {
    #[inline(always)]
    fn from(variant: Dma) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DMA` reader - Direct Memory Access"]
pub type DmaR = crate::BitReader<Dma>;
impl DmaR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dma {
        match self.bits {
            false => Dma::Dmano,
            true => Dma::Dmayes,
        }
    }
    #[doc = "Not supported"]
    #[inline(always)]
    pub fn is_dmano(&self) -> bool {
        *self == Dma::Dmano
    }
    #[doc = "Supported"]
    #[inline(always)]
    pub fn is_dmayes(&self) -> bool {
        *self == Dma::Dmayes
    }
}
impl R {
    #[doc = "Bits 0:1 - ID 48b Handler"]
    #[inline(always)]
    pub fn idena(&self) -> IdenaR {
        IdenaR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 2:5 - ID Register"]
    #[inline(always)]
    pub fn idreg(&self) -> IdregR {
        IdregR::new(((self.bits >> 2) & 0x0f) as u8)
    }
    #[doc = "Bits 6:7 - High Data Rate Support"]
    #[inline(always)]
    pub fn hdrsupp(&self) -> HdrsuppR {
        HdrsuppR::new(((self.bits >> 6) & 3) as u8)
    }
    #[doc = "Bit 9 - Controller"]
    #[inline(always)]
    pub fn master(&self) -> MasterR {
        MasterR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bits 10:11 - Static Address"]
    #[inline(always)]
    pub fn saddr(&self) -> SaddrR {
        SaddrR::new(((self.bits >> 10) & 3) as u8)
    }
    #[doc = "Bits 12:15 - Common Command Codes Handling"]
    #[inline(always)]
    pub fn ccchandle(&self) -> CcchandleR {
        CcchandleR::new(((self.bits >> 12) & 0x0f) as u8)
    }
    #[doc = "Bits 16:20 - In-Band Interrupts, Controller Requests, Hot-Join Events"]
    #[inline(always)]
    pub fn ibi_mr_hj(&self) -> IbiMrHjR {
        IbiMrHjR::new(((self.bits >> 16) & 0x1f) as u8)
    }
    #[doc = "Bit 21 - Time Control"]
    #[inline(always)]
    pub fn timectrl(&self) -> TimectrlR {
        TimectrlR::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bits 23:25 - External FIFO"]
    #[inline(always)]
    pub fn extfifo(&self) -> ExtfifoR {
        ExtfifoR::new(((self.bits >> 23) & 7) as u8)
    }
    #[doc = "Bits 26:27 - FIFO Transmit"]
    #[inline(always)]
    pub fn fifotx(&self) -> FifotxR {
        FifotxR::new(((self.bits >> 26) & 3) as u8)
    }
    #[doc = "Bits 28:29 - FIFO Receive"]
    #[inline(always)]
    pub fn fiforx(&self) -> FiforxR {
        FiforxR::new(((self.bits >> 28) & 3) as u8)
    }
    #[doc = "Bit 30 - Interrupts"]
    #[inline(always)]
    pub fn int(&self) -> IntR {
        IntR::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Direct Memory Access"]
    #[inline(always)]
    pub fn dma(&self) -> DmaR {
        DmaR::new(((self.bits >> 31) & 1) != 0)
    }
}
#[doc = "Target Capabilities\n\nYou can [`read`](crate::Reg::read) this register and get [`scapabilities::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ScapabilitiesSpec;
impl crate::RegisterSpec for ScapabilitiesSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`scapabilities::R`](R) reader structure"]
impl crate::Readable for ScapabilitiesSpec {}
#[doc = "`reset()` method sets SCAPABILITIES to value 0xe83f_fe70"]
impl crate::Resettable for ScapabilitiesSpec {
    const RESET_VALUE: u32 = 0xe83f_fe70;
}

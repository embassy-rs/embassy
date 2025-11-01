#[doc = "Register `FSTAT` reader"]
pub type R = crate::R<FstatSpec>;
#[doc = "Register `FSTAT` writer"]
pub type W = crate::W<FstatSpec>;
#[doc = "Command Fail Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fail {
    #[doc = "0: Error not detected"]
    Fail0 = 0,
    #[doc = "1: Error detected"]
    Fail1 = 1,
}
impl From<Fail> for bool {
    #[inline(always)]
    fn from(variant: Fail) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FAIL` reader - Command Fail Flag"]
pub type FailR = crate::BitReader<Fail>;
impl FailR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fail {
        match self.bits {
            false => Fail::Fail0,
            true => Fail::Fail1,
        }
    }
    #[doc = "Error not detected"]
    #[inline(always)]
    pub fn is_fail0(&self) -> bool {
        *self == Fail::Fail0
    }
    #[doc = "Error detected"]
    #[inline(always)]
    pub fn is_fail1(&self) -> bool {
        *self == Fail::Fail1
    }
}
#[doc = "Command Abort Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmdabt {
    #[doc = "0: No command abort detected"]
    Cmdabt0 = 0,
    #[doc = "1: Command abort detected"]
    Cmdabt1 = 1,
}
impl From<Cmdabt> for bool {
    #[inline(always)]
    fn from(variant: Cmdabt) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CMDABT` reader - Command Abort Flag"]
pub type CmdabtR = crate::BitReader<Cmdabt>;
impl CmdabtR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cmdabt {
        match self.bits {
            false => Cmdabt::Cmdabt0,
            true => Cmdabt::Cmdabt1,
        }
    }
    #[doc = "No command abort detected"]
    #[inline(always)]
    pub fn is_cmdabt0(&self) -> bool {
        *self == Cmdabt::Cmdabt0
    }
    #[doc = "Command abort detected"]
    #[inline(always)]
    pub fn is_cmdabt1(&self) -> bool {
        *self == Cmdabt::Cmdabt1
    }
}
#[doc = "Field `CMDABT` writer - Command Abort Flag"]
pub type CmdabtW<'a, REG> = crate::BitWriter1C<'a, REG, Cmdabt>;
impl<'a, REG> CmdabtW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No command abort detected"]
    #[inline(always)]
    pub fn cmdabt0(self) -> &'a mut crate::W<REG> {
        self.variant(Cmdabt::Cmdabt0)
    }
    #[doc = "Command abort detected"]
    #[inline(always)]
    pub fn cmdabt1(self) -> &'a mut crate::W<REG> {
        self.variant(Cmdabt::Cmdabt1)
    }
}
#[doc = "Command Protection Violation Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pviol {
    #[doc = "0: No protection violation detected"]
    Pviol0 = 0,
    #[doc = "1: Protection violation detected"]
    Pviol1 = 1,
}
impl From<Pviol> for bool {
    #[inline(always)]
    fn from(variant: Pviol) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PVIOL` reader - Command Protection Violation Flag"]
pub type PviolR = crate::BitReader<Pviol>;
impl PviolR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pviol {
        match self.bits {
            false => Pviol::Pviol0,
            true => Pviol::Pviol1,
        }
    }
    #[doc = "No protection violation detected"]
    #[inline(always)]
    pub fn is_pviol0(&self) -> bool {
        *self == Pviol::Pviol0
    }
    #[doc = "Protection violation detected"]
    #[inline(always)]
    pub fn is_pviol1(&self) -> bool {
        *self == Pviol::Pviol1
    }
}
#[doc = "Field `PVIOL` writer - Command Protection Violation Flag"]
pub type PviolW<'a, REG> = crate::BitWriter1C<'a, REG, Pviol>;
impl<'a, REG> PviolW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No protection violation detected"]
    #[inline(always)]
    pub fn pviol0(self) -> &'a mut crate::W<REG> {
        self.variant(Pviol::Pviol0)
    }
    #[doc = "Protection violation detected"]
    #[inline(always)]
    pub fn pviol1(self) -> &'a mut crate::W<REG> {
        self.variant(Pviol::Pviol1)
    }
}
#[doc = "Command Access Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Accerr {
    #[doc = "0: No access error detected"]
    Accerr0 = 0,
    #[doc = "1: Access error detected"]
    Accerr1 = 1,
}
impl From<Accerr> for bool {
    #[inline(always)]
    fn from(variant: Accerr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ACCERR` reader - Command Access Error Flag"]
pub type AccerrR = crate::BitReader<Accerr>;
impl AccerrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Accerr {
        match self.bits {
            false => Accerr::Accerr0,
            true => Accerr::Accerr1,
        }
    }
    #[doc = "No access error detected"]
    #[inline(always)]
    pub fn is_accerr0(&self) -> bool {
        *self == Accerr::Accerr0
    }
    #[doc = "Access error detected"]
    #[inline(always)]
    pub fn is_accerr1(&self) -> bool {
        *self == Accerr::Accerr1
    }
}
#[doc = "Field `ACCERR` writer - Command Access Error Flag"]
pub type AccerrW<'a, REG> = crate::BitWriter1C<'a, REG, Accerr>;
impl<'a, REG> AccerrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No access error detected"]
    #[inline(always)]
    pub fn accerr0(self) -> &'a mut crate::W<REG> {
        self.variant(Accerr::Accerr0)
    }
    #[doc = "Access error detected"]
    #[inline(always)]
    pub fn accerr1(self) -> &'a mut crate::W<REG> {
        self.variant(Accerr::Accerr1)
    }
}
#[doc = "Command Write Sequence Abort Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cwsabt {
    #[doc = "0: Command write sequence not aborted"]
    Cwsabt0 = 0,
    #[doc = "1: Command write sequence aborted"]
    Cwsabt1 = 1,
}
impl From<Cwsabt> for bool {
    #[inline(always)]
    fn from(variant: Cwsabt) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CWSABT` reader - Command Write Sequence Abort Flag"]
pub type CwsabtR = crate::BitReader<Cwsabt>;
impl CwsabtR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cwsabt {
        match self.bits {
            false => Cwsabt::Cwsabt0,
            true => Cwsabt::Cwsabt1,
        }
    }
    #[doc = "Command write sequence not aborted"]
    #[inline(always)]
    pub fn is_cwsabt0(&self) -> bool {
        *self == Cwsabt::Cwsabt0
    }
    #[doc = "Command write sequence aborted"]
    #[inline(always)]
    pub fn is_cwsabt1(&self) -> bool {
        *self == Cwsabt::Cwsabt1
    }
}
#[doc = "Field `CWSABT` writer - Command Write Sequence Abort Flag"]
pub type CwsabtW<'a, REG> = crate::BitWriter1C<'a, REG, Cwsabt>;
impl<'a, REG> CwsabtW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Command write sequence not aborted"]
    #[inline(always)]
    pub fn cwsabt0(self) -> &'a mut crate::W<REG> {
        self.variant(Cwsabt::Cwsabt0)
    }
    #[doc = "Command write sequence aborted"]
    #[inline(always)]
    pub fn cwsabt1(self) -> &'a mut crate::W<REG> {
        self.variant(Cwsabt::Cwsabt1)
    }
}
#[doc = "Command Complete Interrupt Flag\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ccif {
    #[doc = "0: Flash command, initialization, or power mode recovery in progress"]
    Ccif0 = 0,
    #[doc = "1: Flash command, initialization, or power mode recovery has completed"]
    Ccif1 = 1,
}
impl From<Ccif> for bool {
    #[inline(always)]
    fn from(variant: Ccif) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CCIF` reader - Command Complete Interrupt Flag"]
pub type CcifR = crate::BitReader<Ccif>;
impl CcifR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ccif {
        match self.bits {
            false => Ccif::Ccif0,
            true => Ccif::Ccif1,
        }
    }
    #[doc = "Flash command, initialization, or power mode recovery in progress"]
    #[inline(always)]
    pub fn is_ccif0(&self) -> bool {
        *self == Ccif::Ccif0
    }
    #[doc = "Flash command, initialization, or power mode recovery has completed"]
    #[inline(always)]
    pub fn is_ccif1(&self) -> bool {
        *self == Ccif::Ccif1
    }
}
#[doc = "Field `CCIF` writer - Command Complete Interrupt Flag"]
pub type CcifW<'a, REG> = crate::BitWriter1C<'a, REG, Ccif>;
impl<'a, REG> CcifW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Flash command, initialization, or power mode recovery in progress"]
    #[inline(always)]
    pub fn ccif0(self) -> &'a mut crate::W<REG> {
        self.variant(Ccif::Ccif0)
    }
    #[doc = "Flash command, initialization, or power mode recovery has completed"]
    #[inline(always)]
    pub fn ccif1(self) -> &'a mut crate::W<REG> {
        self.variant(Ccif::Ccif1)
    }
}
#[doc = "Command protection level\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Cmdprt {
    #[doc = "0: Secure, normal access"]
    Cmdprt00 = 0,
    #[doc = "1: Secure, privileged access"]
    Cmdprt01 = 1,
    #[doc = "2: Nonsecure, normal access"]
    Cmdprt10 = 2,
    #[doc = "3: Nonsecure, privileged access"]
    Cmdprt11 = 3,
}
impl From<Cmdprt> for u8 {
    #[inline(always)]
    fn from(variant: Cmdprt) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Cmdprt {
    type Ux = u8;
}
impl crate::IsEnum for Cmdprt {}
#[doc = "Field `CMDPRT` reader - Command protection level"]
pub type CmdprtR = crate::FieldReader<Cmdprt>;
impl CmdprtR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cmdprt {
        match self.bits {
            0 => Cmdprt::Cmdprt00,
            1 => Cmdprt::Cmdprt01,
            2 => Cmdprt::Cmdprt10,
            3 => Cmdprt::Cmdprt11,
            _ => unreachable!(),
        }
    }
    #[doc = "Secure, normal access"]
    #[inline(always)]
    pub fn is_cmdprt00(&self) -> bool {
        *self == Cmdprt::Cmdprt00
    }
    #[doc = "Secure, privileged access"]
    #[inline(always)]
    pub fn is_cmdprt01(&self) -> bool {
        *self == Cmdprt::Cmdprt01
    }
    #[doc = "Nonsecure, normal access"]
    #[inline(always)]
    pub fn is_cmdprt10(&self) -> bool {
        *self == Cmdprt::Cmdprt10
    }
    #[doc = "Nonsecure, privileged access"]
    #[inline(always)]
    pub fn is_cmdprt11(&self) -> bool {
        *self == Cmdprt::Cmdprt11
    }
}
#[doc = "Command protection status flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmdp {
    #[doc = "0: Command protection level and domain ID are stale"]
    Cmdp0 = 0,
    #[doc = "1: Command protection level (CMDPRT) and domain ID (CMDDID) are set"]
    Cmdp1 = 1,
}
impl From<Cmdp> for bool {
    #[inline(always)]
    fn from(variant: Cmdp) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CMDP` reader - Command protection status flag"]
pub type CmdpR = crate::BitReader<Cmdp>;
impl CmdpR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cmdp {
        match self.bits {
            false => Cmdp::Cmdp0,
            true => Cmdp::Cmdp1,
        }
    }
    #[doc = "Command protection level and domain ID are stale"]
    #[inline(always)]
    pub fn is_cmdp0(&self) -> bool {
        *self == Cmdp::Cmdp0
    }
    #[doc = "Command protection level (CMDPRT) and domain ID (CMDDID) are set"]
    #[inline(always)]
    pub fn is_cmdp1(&self) -> bool {
        *self == Cmdp::Cmdp1
    }
}
#[doc = "Field `CMDDID` reader - Command domain ID"]
pub type CmddidR = crate::FieldReader;
#[doc = "Double Bit Fault Detect Interrupt Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dfdif {
    #[doc = "0: Double bit fault not detected during a valid flash read access"]
    Dfdif0 = 0,
    #[doc = "1: Double bit fault detected (or FCTRL\\[FDFD\\] is set) during a valid flash read access"]
    Dfdif1 = 1,
}
impl From<Dfdif> for bool {
    #[inline(always)]
    fn from(variant: Dfdif) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DFDIF` reader - Double Bit Fault Detect Interrupt Flag"]
pub type DfdifR = crate::BitReader<Dfdif>;
impl DfdifR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dfdif {
        match self.bits {
            false => Dfdif::Dfdif0,
            true => Dfdif::Dfdif1,
        }
    }
    #[doc = "Double bit fault not detected during a valid flash read access"]
    #[inline(always)]
    pub fn is_dfdif0(&self) -> bool {
        *self == Dfdif::Dfdif0
    }
    #[doc = "Double bit fault detected (or FCTRL\\[FDFD\\] is set) during a valid flash read access"]
    #[inline(always)]
    pub fn is_dfdif1(&self) -> bool {
        *self == Dfdif::Dfdif1
    }
}
#[doc = "Field `DFDIF` writer - Double Bit Fault Detect Interrupt Flag"]
pub type DfdifW<'a, REG> = crate::BitWriter1C<'a, REG, Dfdif>;
impl<'a, REG> DfdifW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Double bit fault not detected during a valid flash read access"]
    #[inline(always)]
    pub fn dfdif0(self) -> &'a mut crate::W<REG> {
        self.variant(Dfdif::Dfdif0)
    }
    #[doc = "Double bit fault detected (or FCTRL\\[FDFD\\] is set) during a valid flash read access"]
    #[inline(always)]
    pub fn dfdif1(self) -> &'a mut crate::W<REG> {
        self.variant(Dfdif::Dfdif1)
    }
}
#[doc = "Salvage Used for Erase operation\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SalvUsed {
    #[doc = "0: Salvage not used during last operation"]
    SalvUsed0 = 0,
    #[doc = "1: Salvage used during the last erase operation"]
    SalvUsed1 = 1,
}
impl From<SalvUsed> for bool {
    #[inline(always)]
    fn from(variant: SalvUsed) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SALV_USED` reader - Salvage Used for Erase operation"]
pub type SalvUsedR = crate::BitReader<SalvUsed>;
impl SalvUsedR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> SalvUsed {
        match self.bits {
            false => SalvUsed::SalvUsed0,
            true => SalvUsed::SalvUsed1,
        }
    }
    #[doc = "Salvage not used during last operation"]
    #[inline(always)]
    pub fn is_salv_used0(&self) -> bool {
        *self == SalvUsed::SalvUsed0
    }
    #[doc = "Salvage used during the last erase operation"]
    #[inline(always)]
    pub fn is_salv_used1(&self) -> bool {
        *self == SalvUsed::SalvUsed1
    }
}
#[doc = "Program-Erase Write Enable Control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pewen {
    #[doc = "0: Writes are not enabled"]
    Pewen00 = 0,
    #[doc = "1: Writes are enabled for one flash or IFR phrase (phrase programming, sector erase)"]
    Pewen01 = 1,
    #[doc = "2: Writes are enabled for one flash or IFR page (page programming)"]
    Pewen10 = 2,
}
impl From<Pewen> for u8 {
    #[inline(always)]
    fn from(variant: Pewen) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pewen {
    type Ux = u8;
}
impl crate::IsEnum for Pewen {}
#[doc = "Field `PEWEN` reader - Program-Erase Write Enable Control"]
pub type PewenR = crate::FieldReader<Pewen>;
impl PewenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Pewen> {
        match self.bits {
            0 => Some(Pewen::Pewen00),
            1 => Some(Pewen::Pewen01),
            2 => Some(Pewen::Pewen10),
            _ => None,
        }
    }
    #[doc = "Writes are not enabled"]
    #[inline(always)]
    pub fn is_pewen00(&self) -> bool {
        *self == Pewen::Pewen00
    }
    #[doc = "Writes are enabled for one flash or IFR phrase (phrase programming, sector erase)"]
    #[inline(always)]
    pub fn is_pewen01(&self) -> bool {
        *self == Pewen::Pewen01
    }
    #[doc = "Writes are enabled for one flash or IFR page (page programming)"]
    #[inline(always)]
    pub fn is_pewen10(&self) -> bool {
        *self == Pewen::Pewen10
    }
}
#[doc = "Program-Erase Ready Control/Status Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Perdy {
    #[doc = "0: Program or sector erase command operation not stalled"]
    Perdy0 = 0,
    #[doc = "1: Program or sector erase command operation ready to execute"]
    Perdy1 = 1,
}
impl From<Perdy> for bool {
    #[inline(always)]
    fn from(variant: Perdy) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PERDY` reader - Program-Erase Ready Control/Status Flag"]
pub type PerdyR = crate::BitReader<Perdy>;
impl PerdyR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Perdy {
        match self.bits {
            false => Perdy::Perdy0,
            true => Perdy::Perdy1,
        }
    }
    #[doc = "Program or sector erase command operation not stalled"]
    #[inline(always)]
    pub fn is_perdy0(&self) -> bool {
        *self == Perdy::Perdy0
    }
    #[doc = "Program or sector erase command operation ready to execute"]
    #[inline(always)]
    pub fn is_perdy1(&self) -> bool {
        *self == Perdy::Perdy1
    }
}
#[doc = "Field `PERDY` writer - Program-Erase Ready Control/Status Flag"]
pub type PerdyW<'a, REG> = crate::BitWriter1C<'a, REG, Perdy>;
impl<'a, REG> PerdyW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Program or sector erase command operation not stalled"]
    #[inline(always)]
    pub fn perdy0(self) -> &'a mut crate::W<REG> {
        self.variant(Perdy::Perdy0)
    }
    #[doc = "Program or sector erase command operation ready to execute"]
    #[inline(always)]
    pub fn perdy1(self) -> &'a mut crate::W<REG> {
        self.variant(Perdy::Perdy1)
    }
}
impl R {
    #[doc = "Bit 0 - Command Fail Flag"]
    #[inline(always)]
    pub fn fail(&self) -> FailR {
        FailR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 2 - Command Abort Flag"]
    #[inline(always)]
    pub fn cmdabt(&self) -> CmdabtR {
        CmdabtR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 4 - Command Protection Violation Flag"]
    #[inline(always)]
    pub fn pviol(&self) -> PviolR {
        PviolR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Command Access Error Flag"]
    #[inline(always)]
    pub fn accerr(&self) -> AccerrR {
        AccerrR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Command Write Sequence Abort Flag"]
    #[inline(always)]
    pub fn cwsabt(&self) -> CwsabtR {
        CwsabtR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Command Complete Interrupt Flag"]
    #[inline(always)]
    pub fn ccif(&self) -> CcifR {
        CcifR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bits 8:9 - Command protection level"]
    #[inline(always)]
    pub fn cmdprt(&self) -> CmdprtR {
        CmdprtR::new(((self.bits >> 8) & 3) as u8)
    }
    #[doc = "Bit 11 - Command protection status flag"]
    #[inline(always)]
    pub fn cmdp(&self) -> CmdpR {
        CmdpR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bits 12:15 - Command domain ID"]
    #[inline(always)]
    pub fn cmddid(&self) -> CmddidR {
        CmddidR::new(((self.bits >> 12) & 0x0f) as u8)
    }
    #[doc = "Bit 16 - Double Bit Fault Detect Interrupt Flag"]
    #[inline(always)]
    pub fn dfdif(&self) -> DfdifR {
        DfdifR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Salvage Used for Erase operation"]
    #[inline(always)]
    pub fn salv_used(&self) -> SalvUsedR {
        SalvUsedR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bits 24:25 - Program-Erase Write Enable Control"]
    #[inline(always)]
    pub fn pewen(&self) -> PewenR {
        PewenR::new(((self.bits >> 24) & 3) as u8)
    }
    #[doc = "Bit 31 - Program-Erase Ready Control/Status Flag"]
    #[inline(always)]
    pub fn perdy(&self) -> PerdyR {
        PerdyR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 2 - Command Abort Flag"]
    #[inline(always)]
    pub fn cmdabt(&mut self) -> CmdabtW<FstatSpec> {
        CmdabtW::new(self, 2)
    }
    #[doc = "Bit 4 - Command Protection Violation Flag"]
    #[inline(always)]
    pub fn pviol(&mut self) -> PviolW<FstatSpec> {
        PviolW::new(self, 4)
    }
    #[doc = "Bit 5 - Command Access Error Flag"]
    #[inline(always)]
    pub fn accerr(&mut self) -> AccerrW<FstatSpec> {
        AccerrW::new(self, 5)
    }
    #[doc = "Bit 6 - Command Write Sequence Abort Flag"]
    #[inline(always)]
    pub fn cwsabt(&mut self) -> CwsabtW<FstatSpec> {
        CwsabtW::new(self, 6)
    }
    #[doc = "Bit 7 - Command Complete Interrupt Flag"]
    #[inline(always)]
    pub fn ccif(&mut self) -> CcifW<FstatSpec> {
        CcifW::new(self, 7)
    }
    #[doc = "Bit 16 - Double Bit Fault Detect Interrupt Flag"]
    #[inline(always)]
    pub fn dfdif(&mut self) -> DfdifW<FstatSpec> {
        DfdifW::new(self, 16)
    }
    #[doc = "Bit 31 - Program-Erase Ready Control/Status Flag"]
    #[inline(always)]
    pub fn perdy(&mut self) -> PerdyW<FstatSpec> {
        PerdyW::new(self, 31)
    }
}
#[doc = "Flash Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`fstat::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fstat::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FstatSpec;
impl crate::RegisterSpec for FstatSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`fstat::R`](R) reader structure"]
impl crate::Readable for FstatSpec {}
#[doc = "`write(|w| ..)` method takes [`fstat::W`](W) writer structure"]
impl crate::Writable for FstatSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x8001_00f4;
}
#[doc = "`reset()` method sets FSTAT to value 0x80"]
impl crate::Resettable for FstatSpec {
    const RESET_VALUE: u32 = 0x80;
}

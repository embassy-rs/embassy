#[doc = "Register `MSR` reader"]
pub type R = crate::R<MsrSpec>;
#[doc = "Register `MSR` writer"]
pub type W = crate::W<MsrSpec>;
#[doc = "Transmit Data Flag\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tdf {
    #[doc = "0: Transmit data not requested"]
    Disabled = 0,
    #[doc = "1: Transmit data requested"]
    Enabled = 1,
}
impl From<Tdf> for bool {
    #[inline(always)]
    fn from(variant: Tdf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TDF` reader - Transmit Data Flag"]
pub type TdfR = crate::BitReader<Tdf>;
impl TdfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tdf {
        match self.bits {
            false => Tdf::Disabled,
            true => Tdf::Enabled,
        }
    }
    #[doc = "Transmit data not requested"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Tdf::Disabled
    }
    #[doc = "Transmit data requested"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Tdf::Enabled
    }
}
#[doc = "Receive Data Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rdf {
    #[doc = "0: Receive data not ready"]
    Disabled = 0,
    #[doc = "1: Receive data ready"]
    Enabled = 1,
}
impl From<Rdf> for bool {
    #[inline(always)]
    fn from(variant: Rdf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RDF` reader - Receive Data Flag"]
pub type RdfR = crate::BitReader<Rdf>;
impl RdfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rdf {
        match self.bits {
            false => Rdf::Disabled,
            true => Rdf::Enabled,
        }
    }
    #[doc = "Receive data not ready"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Rdf::Disabled
    }
    #[doc = "Receive data ready"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Rdf::Enabled
    }
}
#[doc = "End Packet Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Epf {
    #[doc = "0: No Stop or repeated Start generated"]
    IntNo = 0,
    #[doc = "1: Stop or repeated Start generated"]
    IntYes = 1,
}
impl From<Epf> for bool {
    #[inline(always)]
    fn from(variant: Epf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EPF` reader - End Packet Flag"]
pub type EpfR = crate::BitReader<Epf>;
impl EpfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Epf {
        match self.bits {
            false => Epf::IntNo,
            true => Epf::IntYes,
        }
    }
    #[doc = "No Stop or repeated Start generated"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Epf::IntNo
    }
    #[doc = "Stop or repeated Start generated"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Epf::IntYes
    }
}
#[doc = "Field `EPF` writer - End Packet Flag"]
pub type EpfW<'a, REG> = crate::BitWriter1C<'a, REG, Epf>;
impl<'a, REG> EpfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No Stop or repeated Start generated"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Epf::IntNo)
    }
    #[doc = "Stop or repeated Start generated"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Epf::IntYes)
    }
}
#[doc = "Stop Detect Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sdf {
    #[doc = "0: No Stop condition generated"]
    IntNo = 0,
    #[doc = "1: Stop condition generated"]
    IntYes = 1,
}
impl From<Sdf> for bool {
    #[inline(always)]
    fn from(variant: Sdf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SDF` reader - Stop Detect Flag"]
pub type SdfR = crate::BitReader<Sdf>;
impl SdfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sdf {
        match self.bits {
            false => Sdf::IntNo,
            true => Sdf::IntYes,
        }
    }
    #[doc = "No Stop condition generated"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Sdf::IntNo
    }
    #[doc = "Stop condition generated"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Sdf::IntYes
    }
}
#[doc = "Field `SDF` writer - Stop Detect Flag"]
pub type SdfW<'a, REG> = crate::BitWriter1C<'a, REG, Sdf>;
impl<'a, REG> SdfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No Stop condition generated"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Sdf::IntNo)
    }
    #[doc = "Stop condition generated"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Sdf::IntYes)
    }
}
#[doc = "NACK Detect Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ndf {
    #[doc = "0: No unexpected NACK detected"]
    IntNo = 0,
    #[doc = "1: Unexpected NACK detected"]
    IntYes = 1,
}
impl From<Ndf> for bool {
    #[inline(always)]
    fn from(variant: Ndf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NDF` reader - NACK Detect Flag"]
pub type NdfR = crate::BitReader<Ndf>;
impl NdfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ndf {
        match self.bits {
            false => Ndf::IntNo,
            true => Ndf::IntYes,
        }
    }
    #[doc = "No unexpected NACK detected"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Ndf::IntNo
    }
    #[doc = "Unexpected NACK detected"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Ndf::IntYes
    }
}
#[doc = "Field `NDF` writer - NACK Detect Flag"]
pub type NdfW<'a, REG> = crate::BitWriter1C<'a, REG, Ndf>;
impl<'a, REG> NdfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No unexpected NACK detected"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Ndf::IntNo)
    }
    #[doc = "Unexpected NACK detected"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Ndf::IntYes)
    }
}
#[doc = "Arbitration Lost Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alf {
    #[doc = "0: Controller did not lose arbitration"]
    IntNo = 0,
    #[doc = "1: Controller lost arbitration"]
    IntYes = 1,
}
impl From<Alf> for bool {
    #[inline(always)]
    fn from(variant: Alf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ALF` reader - Arbitration Lost Flag"]
pub type AlfR = crate::BitReader<Alf>;
impl AlfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Alf {
        match self.bits {
            false => Alf::IntNo,
            true => Alf::IntYes,
        }
    }
    #[doc = "Controller did not lose arbitration"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Alf::IntNo
    }
    #[doc = "Controller lost arbitration"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Alf::IntYes
    }
}
#[doc = "Field `ALF` writer - Arbitration Lost Flag"]
pub type AlfW<'a, REG> = crate::BitWriter1C<'a, REG, Alf>;
impl<'a, REG> AlfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Controller did not lose arbitration"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Alf::IntNo)
    }
    #[doc = "Controller lost arbitration"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Alf::IntYes)
    }
}
#[doc = "FIFO Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fef {
    #[doc = "0: No FIFO error"]
    IntNo = 0,
    #[doc = "1: FIFO error"]
    IntYes = 1,
}
impl From<Fef> for bool {
    #[inline(always)]
    fn from(variant: Fef) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FEF` reader - FIFO Error Flag"]
pub type FefR = crate::BitReader<Fef>;
impl FefR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fef {
        match self.bits {
            false => Fef::IntNo,
            true => Fef::IntYes,
        }
    }
    #[doc = "No FIFO error"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Fef::IntNo
    }
    #[doc = "FIFO error"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Fef::IntYes
    }
}
#[doc = "Field `FEF` writer - FIFO Error Flag"]
pub type FefW<'a, REG> = crate::BitWriter1C<'a, REG, Fef>;
impl<'a, REG> FefW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No FIFO error"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Fef::IntNo)
    }
    #[doc = "FIFO error"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Fef::IntYes)
    }
}
#[doc = "Pin Low Timeout Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pltf {
    #[doc = "0: Pin low timeout did not occur"]
    IntNo = 0,
    #[doc = "1: Pin low timeout occurred"]
    IntYes = 1,
}
impl From<Pltf> for bool {
    #[inline(always)]
    fn from(variant: Pltf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PLTF` reader - Pin Low Timeout Flag"]
pub type PltfR = crate::BitReader<Pltf>;
impl PltfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pltf {
        match self.bits {
            false => Pltf::IntNo,
            true => Pltf::IntYes,
        }
    }
    #[doc = "Pin low timeout did not occur"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Pltf::IntNo
    }
    #[doc = "Pin low timeout occurred"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Pltf::IntYes
    }
}
#[doc = "Field `PLTF` writer - Pin Low Timeout Flag"]
pub type PltfW<'a, REG> = crate::BitWriter1C<'a, REG, Pltf>;
impl<'a, REG> PltfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin low timeout did not occur"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Pltf::IntNo)
    }
    #[doc = "Pin low timeout occurred"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Pltf::IntYes)
    }
}
#[doc = "Data Match Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dmf {
    #[doc = "0: Matching data not received"]
    IntNo = 0,
    #[doc = "1: Matching data received"]
    IntYes = 1,
}
impl From<Dmf> for bool {
    #[inline(always)]
    fn from(variant: Dmf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DMF` reader - Data Match Flag"]
pub type DmfR = crate::BitReader<Dmf>;
impl DmfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dmf {
        match self.bits {
            false => Dmf::IntNo,
            true => Dmf::IntYes,
        }
    }
    #[doc = "Matching data not received"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Dmf::IntNo
    }
    #[doc = "Matching data received"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Dmf::IntYes
    }
}
#[doc = "Field `DMF` writer - Data Match Flag"]
pub type DmfW<'a, REG> = crate::BitWriter1C<'a, REG, Dmf>;
impl<'a, REG> DmfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Matching data not received"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Dmf::IntNo)
    }
    #[doc = "Matching data received"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Dmf::IntYes)
    }
}
#[doc = "Start Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stf {
    #[doc = "0: Start condition not detected"]
    IntNo = 0,
    #[doc = "1: Start condition detected"]
    IntYes = 1,
}
impl From<Stf> for bool {
    #[inline(always)]
    fn from(variant: Stf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STF` reader - Start Flag"]
pub type StfR = crate::BitReader<Stf>;
impl StfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Stf {
        match self.bits {
            false => Stf::IntNo,
            true => Stf::IntYes,
        }
    }
    #[doc = "Start condition not detected"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Stf::IntNo
    }
    #[doc = "Start condition detected"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Stf::IntYes
    }
}
#[doc = "Field `STF` writer - Start Flag"]
pub type StfW<'a, REG> = crate::BitWriter1C<'a, REG, Stf>;
impl<'a, REG> StfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Start condition not detected"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Stf::IntNo)
    }
    #[doc = "Start condition detected"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Stf::IntYes)
    }
}
#[doc = "Controller Busy Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mbf {
    #[doc = "0: Idle"]
    Idle = 0,
    #[doc = "1: Busy"]
    Busy = 1,
}
impl From<Mbf> for bool {
    #[inline(always)]
    fn from(variant: Mbf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MBF` reader - Controller Busy Flag"]
pub type MbfR = crate::BitReader<Mbf>;
impl MbfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mbf {
        match self.bits {
            false => Mbf::Idle,
            true => Mbf::Busy,
        }
    }
    #[doc = "Idle"]
    #[inline(always)]
    pub fn is_idle(&self) -> bool {
        *self == Mbf::Idle
    }
    #[doc = "Busy"]
    #[inline(always)]
    pub fn is_busy(&self) -> bool {
        *self == Mbf::Busy
    }
}
#[doc = "Bus Busy Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bbf {
    #[doc = "0: Idle"]
    Idle = 0,
    #[doc = "1: Busy"]
    Busy = 1,
}
impl From<Bbf> for bool {
    #[inline(always)]
    fn from(variant: Bbf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BBF` reader - Bus Busy Flag"]
pub type BbfR = crate::BitReader<Bbf>;
impl BbfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Bbf {
        match self.bits {
            false => Bbf::Idle,
            true => Bbf::Busy,
        }
    }
    #[doc = "Idle"]
    #[inline(always)]
    pub fn is_idle(&self) -> bool {
        *self == Bbf::Idle
    }
    #[doc = "Busy"]
    #[inline(always)]
    pub fn is_busy(&self) -> bool {
        *self == Bbf::Busy
    }
}
impl R {
    #[doc = "Bit 0 - Transmit Data Flag"]
    #[inline(always)]
    pub fn tdf(&self) -> TdfR {
        TdfR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Receive Data Flag"]
    #[inline(always)]
    pub fn rdf(&self) -> RdfR {
        RdfR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 8 - End Packet Flag"]
    #[inline(always)]
    pub fn epf(&self) -> EpfR {
        EpfR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Stop Detect Flag"]
    #[inline(always)]
    pub fn sdf(&self) -> SdfR {
        SdfR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - NACK Detect Flag"]
    #[inline(always)]
    pub fn ndf(&self) -> NdfR {
        NdfR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Arbitration Lost Flag"]
    #[inline(always)]
    pub fn alf(&self) -> AlfR {
        AlfR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - FIFO Error Flag"]
    #[inline(always)]
    pub fn fef(&self) -> FefR {
        FefR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Pin Low Timeout Flag"]
    #[inline(always)]
    pub fn pltf(&self) -> PltfR {
        PltfR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Data Match Flag"]
    #[inline(always)]
    pub fn dmf(&self) -> DmfR {
        DmfR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Start Flag"]
    #[inline(always)]
    pub fn stf(&self) -> StfR {
        StfR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 24 - Controller Busy Flag"]
    #[inline(always)]
    pub fn mbf(&self) -> MbfR {
        MbfR::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - Bus Busy Flag"]
    #[inline(always)]
    pub fn bbf(&self) -> BbfR {
        BbfR::new(((self.bits >> 25) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 8 - End Packet Flag"]
    #[inline(always)]
    pub fn epf(&mut self) -> EpfW<MsrSpec> {
        EpfW::new(self, 8)
    }
    #[doc = "Bit 9 - Stop Detect Flag"]
    #[inline(always)]
    pub fn sdf(&mut self) -> SdfW<MsrSpec> {
        SdfW::new(self, 9)
    }
    #[doc = "Bit 10 - NACK Detect Flag"]
    #[inline(always)]
    pub fn ndf(&mut self) -> NdfW<MsrSpec> {
        NdfW::new(self, 10)
    }
    #[doc = "Bit 11 - Arbitration Lost Flag"]
    #[inline(always)]
    pub fn alf(&mut self) -> AlfW<MsrSpec> {
        AlfW::new(self, 11)
    }
    #[doc = "Bit 12 - FIFO Error Flag"]
    #[inline(always)]
    pub fn fef(&mut self) -> FefW<MsrSpec> {
        FefW::new(self, 12)
    }
    #[doc = "Bit 13 - Pin Low Timeout Flag"]
    #[inline(always)]
    pub fn pltf(&mut self) -> PltfW<MsrSpec> {
        PltfW::new(self, 13)
    }
    #[doc = "Bit 14 - Data Match Flag"]
    #[inline(always)]
    pub fn dmf(&mut self) -> DmfW<MsrSpec> {
        DmfW::new(self, 14)
    }
    #[doc = "Bit 15 - Start Flag"]
    #[inline(always)]
    pub fn stf(&mut self) -> StfW<MsrSpec> {
        StfW::new(self, 15)
    }
}
#[doc = "Controller Status\n\nYou can [`read`](crate::Reg::read) this register and get [`msr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`msr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MsrSpec;
impl crate::RegisterSpec for MsrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`msr::R`](R) reader structure"]
impl crate::Readable for MsrSpec {}
#[doc = "`write(|w| ..)` method takes [`msr::W`](W) writer structure"]
impl crate::Writable for MsrSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0xff00;
}
#[doc = "`reset()` method sets MSR to value 0x01"]
impl crate::Resettable for MsrSpec {
    const RESET_VALUE: u32 = 0x01;
}

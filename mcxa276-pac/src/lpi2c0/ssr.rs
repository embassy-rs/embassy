#[doc = "Register `SSR` reader"]
pub type R = crate::R<SsrSpec>;
#[doc = "Register `SSR` writer"]
pub type W = crate::W<SsrSpec>;
#[doc = "Transmit Data Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tdf {
    #[doc = "0: Transmit data not requested"]
    NoFlag = 0,
    #[doc = "1: Transmit data is requested"]
    Flag = 1,
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
            false => Tdf::NoFlag,
            true => Tdf::Flag,
        }
    }
    #[doc = "Transmit data not requested"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Tdf::NoFlag
    }
    #[doc = "Transmit data is requested"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Tdf::Flag
    }
}
#[doc = "Receive Data Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rdf {
    #[doc = "0: Not ready"]
    NotReady = 0,
    #[doc = "1: Ready"]
    Ready = 1,
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
            false => Rdf::NotReady,
            true => Rdf::Ready,
        }
    }
    #[doc = "Not ready"]
    #[inline(always)]
    pub fn is_not_ready(&self) -> bool {
        *self == Rdf::NotReady
    }
    #[doc = "Ready"]
    #[inline(always)]
    pub fn is_ready(&self) -> bool {
        *self == Rdf::Ready
    }
}
#[doc = "Address Valid Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Avf {
    #[doc = "0: Not valid"]
    NotValid = 0,
    #[doc = "1: Valid"]
    Valid = 1,
}
impl From<Avf> for bool {
    #[inline(always)]
    fn from(variant: Avf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `AVF` reader - Address Valid Flag"]
pub type AvfR = crate::BitReader<Avf>;
impl AvfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Avf {
        match self.bits {
            false => Avf::NotValid,
            true => Avf::Valid,
        }
    }
    #[doc = "Not valid"]
    #[inline(always)]
    pub fn is_not_valid(&self) -> bool {
        *self == Avf::NotValid
    }
    #[doc = "Valid"]
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        *self == Avf::Valid
    }
}
#[doc = "Transmit ACK Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Taf {
    #[doc = "0: Not required"]
    NotRequired = 0,
    #[doc = "1: Required"]
    Required = 1,
}
impl From<Taf> for bool {
    #[inline(always)]
    fn from(variant: Taf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TAF` reader - Transmit ACK Flag"]
pub type TafR = crate::BitReader<Taf>;
impl TafR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Taf {
        match self.bits {
            false => Taf::NotRequired,
            true => Taf::Required,
        }
    }
    #[doc = "Not required"]
    #[inline(always)]
    pub fn is_not_required(&self) -> bool {
        *self == Taf::NotRequired
    }
    #[doc = "Required"]
    #[inline(always)]
    pub fn is_required(&self) -> bool {
        *self == Taf::Required
    }
}
#[doc = "Repeated Start Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rsf {
    #[doc = "0: No repeated Start detected"]
    IntNo = 0,
    #[doc = "1: Repeated Start detected"]
    IntYes = 1,
}
impl From<Rsf> for bool {
    #[inline(always)]
    fn from(variant: Rsf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RSF` reader - Repeated Start Flag"]
pub type RsfR = crate::BitReader<Rsf>;
impl RsfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rsf {
        match self.bits {
            false => Rsf::IntNo,
            true => Rsf::IntYes,
        }
    }
    #[doc = "No repeated Start detected"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Rsf::IntNo
    }
    #[doc = "Repeated Start detected"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Rsf::IntYes
    }
}
#[doc = "Field `RSF` writer - Repeated Start Flag"]
pub type RsfW<'a, REG> = crate::BitWriter1C<'a, REG, Rsf>;
impl<'a, REG> RsfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No repeated Start detected"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Rsf::IntNo)
    }
    #[doc = "Repeated Start detected"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Rsf::IntYes)
    }
}
#[doc = "Stop Detect Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sdf {
    #[doc = "0: No Stop detected"]
    IntNo = 0,
    #[doc = "1: Stop detected"]
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
    #[doc = "No Stop detected"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Sdf::IntNo
    }
    #[doc = "Stop detected"]
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
    #[doc = "No Stop detected"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Sdf::IntNo)
    }
    #[doc = "Stop detected"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Sdf::IntYes)
    }
}
#[doc = "Bit Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bef {
    #[doc = "0: No bit error occurred"]
    IntNo = 0,
    #[doc = "1: Bit error occurred"]
    IntYes = 1,
}
impl From<Bef> for bool {
    #[inline(always)]
    fn from(variant: Bef) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BEF` reader - Bit Error Flag"]
pub type BefR = crate::BitReader<Bef>;
impl BefR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Bef {
        match self.bits {
            false => Bef::IntNo,
            true => Bef::IntYes,
        }
    }
    #[doc = "No bit error occurred"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Bef::IntNo
    }
    #[doc = "Bit error occurred"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Bef::IntYes
    }
}
#[doc = "Field `BEF` writer - Bit Error Flag"]
pub type BefW<'a, REG> = crate::BitWriter1C<'a, REG, Bef>;
impl<'a, REG> BefW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No bit error occurred"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Bef::IntNo)
    }
    #[doc = "Bit error occurred"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Bef::IntYes)
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
#[doc = "Address Match 0 Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Am0f {
    #[doc = "0: ADDR0 matching address not received"]
    NoFlag = 0,
    #[doc = "1: ADDR0 matching address received"]
    Flag = 1,
}
impl From<Am0f> for bool {
    #[inline(always)]
    fn from(variant: Am0f) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `AM0F` reader - Address Match 0 Flag"]
pub type Am0fR = crate::BitReader<Am0f>;
impl Am0fR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Am0f {
        match self.bits {
            false => Am0f::NoFlag,
            true => Am0f::Flag,
        }
    }
    #[doc = "ADDR0 matching address not received"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Am0f::NoFlag
    }
    #[doc = "ADDR0 matching address received"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Am0f::Flag
    }
}
#[doc = "Address Match 1 Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Am1f {
    #[doc = "0: Matching address not received"]
    NoFlag = 0,
    #[doc = "1: Matching address received"]
    Flag = 1,
}
impl From<Am1f> for bool {
    #[inline(always)]
    fn from(variant: Am1f) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `AM1F` reader - Address Match 1 Flag"]
pub type Am1fR = crate::BitReader<Am1f>;
impl Am1fR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Am1f {
        match self.bits {
            false => Am1f::NoFlag,
            true => Am1f::Flag,
        }
    }
    #[doc = "Matching address not received"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Am1f::NoFlag
    }
    #[doc = "Matching address received"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Am1f::Flag
    }
}
#[doc = "General Call Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gcf {
    #[doc = "0: General call address disabled or not detected"]
    NoFlag = 0,
    #[doc = "1: General call address detected"]
    Flag = 1,
}
impl From<Gcf> for bool {
    #[inline(always)]
    fn from(variant: Gcf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GCF` reader - General Call Flag"]
pub type GcfR = crate::BitReader<Gcf>;
impl GcfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gcf {
        match self.bits {
            false => Gcf::NoFlag,
            true => Gcf::Flag,
        }
    }
    #[doc = "General call address disabled or not detected"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Gcf::NoFlag
    }
    #[doc = "General call address detected"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Gcf::Flag
    }
}
#[doc = "SMBus Alert Response Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sarf {
    #[doc = "0: Disabled or not detected"]
    NoFlag = 0,
    #[doc = "1: Enabled and detected"]
    Flag = 1,
}
impl From<Sarf> for bool {
    #[inline(always)]
    fn from(variant: Sarf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SARF` reader - SMBus Alert Response Flag"]
pub type SarfR = crate::BitReader<Sarf>;
impl SarfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sarf {
        match self.bits {
            false => Sarf::NoFlag,
            true => Sarf::Flag,
        }
    }
    #[doc = "Disabled or not detected"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Sarf::NoFlag
    }
    #[doc = "Enabled and detected"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Sarf::Flag
    }
}
#[doc = "Target Busy Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sbf {
    #[doc = "0: Idle"]
    Idle = 0,
    #[doc = "1: Busy"]
    Busy = 1,
}
impl From<Sbf> for bool {
    #[inline(always)]
    fn from(variant: Sbf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SBF` reader - Target Busy Flag"]
pub type SbfR = crate::BitReader<Sbf>;
impl SbfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sbf {
        match self.bits {
            false => Sbf::Idle,
            true => Sbf::Busy,
        }
    }
    #[doc = "Idle"]
    #[inline(always)]
    pub fn is_idle(&self) -> bool {
        *self == Sbf::Idle
    }
    #[doc = "Busy"]
    #[inline(always)]
    pub fn is_busy(&self) -> bool {
        *self == Sbf::Busy
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
    #[doc = "Bit 2 - Address Valid Flag"]
    #[inline(always)]
    pub fn avf(&self) -> AvfR {
        AvfR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Transmit ACK Flag"]
    #[inline(always)]
    pub fn taf(&self) -> TafR {
        TafR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 8 - Repeated Start Flag"]
    #[inline(always)]
    pub fn rsf(&self) -> RsfR {
        RsfR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Stop Detect Flag"]
    #[inline(always)]
    pub fn sdf(&self) -> SdfR {
        SdfR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Bit Error Flag"]
    #[inline(always)]
    pub fn bef(&self) -> BefR {
        BefR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - FIFO Error Flag"]
    #[inline(always)]
    pub fn fef(&self) -> FefR {
        FefR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Address Match 0 Flag"]
    #[inline(always)]
    pub fn am0f(&self) -> Am0fR {
        Am0fR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Address Match 1 Flag"]
    #[inline(always)]
    pub fn am1f(&self) -> Am1fR {
        Am1fR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - General Call Flag"]
    #[inline(always)]
    pub fn gcf(&self) -> GcfR {
        GcfR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - SMBus Alert Response Flag"]
    #[inline(always)]
    pub fn sarf(&self) -> SarfR {
        SarfR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 24 - Target Busy Flag"]
    #[inline(always)]
    pub fn sbf(&self) -> SbfR {
        SbfR::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - Bus Busy Flag"]
    #[inline(always)]
    pub fn bbf(&self) -> BbfR {
        BbfR::new(((self.bits >> 25) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 8 - Repeated Start Flag"]
    #[inline(always)]
    pub fn rsf(&mut self) -> RsfW<SsrSpec> {
        RsfW::new(self, 8)
    }
    #[doc = "Bit 9 - Stop Detect Flag"]
    #[inline(always)]
    pub fn sdf(&mut self) -> SdfW<SsrSpec> {
        SdfW::new(self, 9)
    }
    #[doc = "Bit 10 - Bit Error Flag"]
    #[inline(always)]
    pub fn bef(&mut self) -> BefW<SsrSpec> {
        BefW::new(self, 10)
    }
    #[doc = "Bit 11 - FIFO Error Flag"]
    #[inline(always)]
    pub fn fef(&mut self) -> FefW<SsrSpec> {
        FefW::new(self, 11)
    }
}
#[doc = "Target Status\n\nYou can [`read`](crate::Reg::read) this register and get [`ssr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ssr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SsrSpec;
impl crate::RegisterSpec for SsrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ssr::R`](R) reader structure"]
impl crate::Readable for SsrSpec {}
#[doc = "`write(|w| ..)` method takes [`ssr::W`](W) writer structure"]
impl crate::Writable for SsrSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x0f00;
}
#[doc = "`reset()` method sets SSR to value 0"]
impl crate::Resettable for SsrSpec {}

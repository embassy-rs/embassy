#[doc = "Register `CTRL1_PN` reader"]
pub type R = crate::R<Ctrl1PnSpec>;
#[doc = "Register `CTRL1_PN` writer"]
pub type W = crate::W<Ctrl1PnSpec>;
#[doc = "Filtering Combination Selection\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Fcs {
    #[doc = "0: Message ID filtering only"]
    IdFiltering = 0,
    #[doc = "1: Message ID filtering and payload filtering"]
    IdPayloadFiltering = 1,
    #[doc = "2: Message ID filtering occurring a specified number of times"]
    IdFilteringNumber = 2,
    #[doc = "3: Message ID filtering and payload filtering a specified number of times"]
    IdPayloadFilteringNumber = 3,
}
impl From<Fcs> for u8 {
    #[inline(always)]
    fn from(variant: Fcs) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Fcs {
    type Ux = u8;
}
impl crate::IsEnum for Fcs {}
#[doc = "Field `FCS` reader - Filtering Combination Selection"]
pub type FcsR = crate::FieldReader<Fcs>;
impl FcsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fcs {
        match self.bits {
            0 => Fcs::IdFiltering,
            1 => Fcs::IdPayloadFiltering,
            2 => Fcs::IdFilteringNumber,
            3 => Fcs::IdPayloadFilteringNumber,
            _ => unreachable!(),
        }
    }
    #[doc = "Message ID filtering only"]
    #[inline(always)]
    pub fn is_id_filtering(&self) -> bool {
        *self == Fcs::IdFiltering
    }
    #[doc = "Message ID filtering and payload filtering"]
    #[inline(always)]
    pub fn is_id_payload_filtering(&self) -> bool {
        *self == Fcs::IdPayloadFiltering
    }
    #[doc = "Message ID filtering occurring a specified number of times"]
    #[inline(always)]
    pub fn is_id_filtering_number(&self) -> bool {
        *self == Fcs::IdFilteringNumber
    }
    #[doc = "Message ID filtering and payload filtering a specified number of times"]
    #[inline(always)]
    pub fn is_id_payload_filtering_number(&self) -> bool {
        *self == Fcs::IdPayloadFilteringNumber
    }
}
#[doc = "Field `FCS` writer - Filtering Combination Selection"]
pub type FcsW<'a, REG> = crate::FieldWriter<'a, REG, 2, Fcs, crate::Safe>;
impl<'a, REG> FcsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Message ID filtering only"]
    #[inline(always)]
    pub fn id_filtering(self) -> &'a mut crate::W<REG> {
        self.variant(Fcs::IdFiltering)
    }
    #[doc = "Message ID filtering and payload filtering"]
    #[inline(always)]
    pub fn id_payload_filtering(self) -> &'a mut crate::W<REG> {
        self.variant(Fcs::IdPayloadFiltering)
    }
    #[doc = "Message ID filtering occurring a specified number of times"]
    #[inline(always)]
    pub fn id_filtering_number(self) -> &'a mut crate::W<REG> {
        self.variant(Fcs::IdFilteringNumber)
    }
    #[doc = "Message ID filtering and payload filtering a specified number of times"]
    #[inline(always)]
    pub fn id_payload_filtering_number(self) -> &'a mut crate::W<REG> {
        self.variant(Fcs::IdPayloadFilteringNumber)
    }
}
#[doc = "ID Filtering Selection\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Idfs {
    #[doc = "0: Match ID contents to an exact target value"]
    MatchExact = 0,
    #[doc = "1: Match an ID value greater than or equal to a specified target value"]
    MatchGte = 1,
    #[doc = "2: Match an ID value smaller than or equal to a specified target value"]
    MatchLte = 2,
    #[doc = "3: Match an ID value within a range of values, inclusive"]
    MatchRange = 3,
}
impl From<Idfs> for u8 {
    #[inline(always)]
    fn from(variant: Idfs) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Idfs {
    type Ux = u8;
}
impl crate::IsEnum for Idfs {}
#[doc = "Field `IDFS` reader - ID Filtering Selection"]
pub type IdfsR = crate::FieldReader<Idfs>;
impl IdfsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Idfs {
        match self.bits {
            0 => Idfs::MatchExact,
            1 => Idfs::MatchGte,
            2 => Idfs::MatchLte,
            3 => Idfs::MatchRange,
            _ => unreachable!(),
        }
    }
    #[doc = "Match ID contents to an exact target value"]
    #[inline(always)]
    pub fn is_match_exact(&self) -> bool {
        *self == Idfs::MatchExact
    }
    #[doc = "Match an ID value greater than or equal to a specified target value"]
    #[inline(always)]
    pub fn is_match_gte(&self) -> bool {
        *self == Idfs::MatchGte
    }
    #[doc = "Match an ID value smaller than or equal to a specified target value"]
    #[inline(always)]
    pub fn is_match_lte(&self) -> bool {
        *self == Idfs::MatchLte
    }
    #[doc = "Match an ID value within a range of values, inclusive"]
    #[inline(always)]
    pub fn is_match_range(&self) -> bool {
        *self == Idfs::MatchRange
    }
}
#[doc = "Field `IDFS` writer - ID Filtering Selection"]
pub type IdfsW<'a, REG> = crate::FieldWriter<'a, REG, 2, Idfs, crate::Safe>;
impl<'a, REG> IdfsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Match ID contents to an exact target value"]
    #[inline(always)]
    pub fn match_exact(self) -> &'a mut crate::W<REG> {
        self.variant(Idfs::MatchExact)
    }
    #[doc = "Match an ID value greater than or equal to a specified target value"]
    #[inline(always)]
    pub fn match_gte(self) -> &'a mut crate::W<REG> {
        self.variant(Idfs::MatchGte)
    }
    #[doc = "Match an ID value smaller than or equal to a specified target value"]
    #[inline(always)]
    pub fn match_lte(self) -> &'a mut crate::W<REG> {
        self.variant(Idfs::MatchLte)
    }
    #[doc = "Match an ID value within a range of values, inclusive"]
    #[inline(always)]
    pub fn match_range(self) -> &'a mut crate::W<REG> {
        self.variant(Idfs::MatchRange)
    }
}
#[doc = "Payload Filtering Selection\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Plfs {
    #[doc = "0: Match payload contents to an exact target value"]
    MatchExact = 0,
    #[doc = "1: Match a payload value greater than or equal to a specified target value"]
    MatchGte = 1,
    #[doc = "2: Match a payload value smaller than or equal to a specified target value"]
    MatchLte = 2,
    #[doc = "3: Match upon a payload value within a range of values, inclusive"]
    MatchRange = 3,
}
impl From<Plfs> for u8 {
    #[inline(always)]
    fn from(variant: Plfs) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Plfs {
    type Ux = u8;
}
impl crate::IsEnum for Plfs {}
#[doc = "Field `PLFS` reader - Payload Filtering Selection"]
pub type PlfsR = crate::FieldReader<Plfs>;
impl PlfsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Plfs {
        match self.bits {
            0 => Plfs::MatchExact,
            1 => Plfs::MatchGte,
            2 => Plfs::MatchLte,
            3 => Plfs::MatchRange,
            _ => unreachable!(),
        }
    }
    #[doc = "Match payload contents to an exact target value"]
    #[inline(always)]
    pub fn is_match_exact(&self) -> bool {
        *self == Plfs::MatchExact
    }
    #[doc = "Match a payload value greater than or equal to a specified target value"]
    #[inline(always)]
    pub fn is_match_gte(&self) -> bool {
        *self == Plfs::MatchGte
    }
    #[doc = "Match a payload value smaller than or equal to a specified target value"]
    #[inline(always)]
    pub fn is_match_lte(&self) -> bool {
        *self == Plfs::MatchLte
    }
    #[doc = "Match upon a payload value within a range of values, inclusive"]
    #[inline(always)]
    pub fn is_match_range(&self) -> bool {
        *self == Plfs::MatchRange
    }
}
#[doc = "Field `PLFS` writer - Payload Filtering Selection"]
pub type PlfsW<'a, REG> = crate::FieldWriter<'a, REG, 2, Plfs, crate::Safe>;
impl<'a, REG> PlfsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Match payload contents to an exact target value"]
    #[inline(always)]
    pub fn match_exact(self) -> &'a mut crate::W<REG> {
        self.variant(Plfs::MatchExact)
    }
    #[doc = "Match a payload value greater than or equal to a specified target value"]
    #[inline(always)]
    pub fn match_gte(self) -> &'a mut crate::W<REG> {
        self.variant(Plfs::MatchGte)
    }
    #[doc = "Match a payload value smaller than or equal to a specified target value"]
    #[inline(always)]
    pub fn match_lte(self) -> &'a mut crate::W<REG> {
        self.variant(Plfs::MatchLte)
    }
    #[doc = "Match upon a payload value within a range of values, inclusive"]
    #[inline(always)]
    pub fn match_range(self) -> &'a mut crate::W<REG> {
        self.variant(Plfs::MatchRange)
    }
}
#[doc = "Number of Messages Matching the Same Filtering Criteria\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Nmatch {
    #[doc = "1: Once"]
    Match1 = 1,
    #[doc = "2: Twice"]
    Match2 = 2,
    #[doc = "255: 255 times"]
    Match255 = 255,
}
impl From<Nmatch> for u8 {
    #[inline(always)]
    fn from(variant: Nmatch) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Nmatch {
    type Ux = u8;
}
impl crate::IsEnum for Nmatch {}
#[doc = "Field `NMATCH` reader - Number of Messages Matching the Same Filtering Criteria"]
pub type NmatchR = crate::FieldReader<Nmatch>;
impl NmatchR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Nmatch> {
        match self.bits {
            1 => Some(Nmatch::Match1),
            2 => Some(Nmatch::Match2),
            255 => Some(Nmatch::Match255),
            _ => None,
        }
    }
    #[doc = "Once"]
    #[inline(always)]
    pub fn is_match_1(&self) -> bool {
        *self == Nmatch::Match1
    }
    #[doc = "Twice"]
    #[inline(always)]
    pub fn is_match_2(&self) -> bool {
        *self == Nmatch::Match2
    }
    #[doc = "255 times"]
    #[inline(always)]
    pub fn is_match_255(&self) -> bool {
        *self == Nmatch::Match255
    }
}
#[doc = "Field `NMATCH` writer - Number of Messages Matching the Same Filtering Criteria"]
pub type NmatchW<'a, REG> = crate::FieldWriter<'a, REG, 8, Nmatch>;
impl<'a, REG> NmatchW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Once"]
    #[inline(always)]
    pub fn match_1(self) -> &'a mut crate::W<REG> {
        self.variant(Nmatch::Match1)
    }
    #[doc = "Twice"]
    #[inline(always)]
    pub fn match_2(self) -> &'a mut crate::W<REG> {
        self.variant(Nmatch::Match2)
    }
    #[doc = "255 times"]
    #[inline(always)]
    pub fn match_255(self) -> &'a mut crate::W<REG> {
        self.variant(Nmatch::Match255)
    }
}
#[doc = "Wake-up by Matching Flag Mask\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WumfMsk {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<WumfMsk> for bool {
    #[inline(always)]
    fn from(variant: WumfMsk) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUMF_MSK` reader - Wake-up by Matching Flag Mask"]
pub type WumfMskR = crate::BitReader<WumfMsk>;
impl WumfMskR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> WumfMsk {
        match self.bits {
            false => WumfMsk::Disable,
            true => WumfMsk::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == WumfMsk::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == WumfMsk::Enable
    }
}
#[doc = "Field `WUMF_MSK` writer - Wake-up by Matching Flag Mask"]
pub type WumfMskW<'a, REG> = crate::BitWriter<'a, REG, WumfMsk>;
impl<'a, REG> WumfMskW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(WumfMsk::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(WumfMsk::Enable)
    }
}
#[doc = "Wake-up by Timeout Flag Mask\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WtofMsk {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<WtofMsk> for bool {
    #[inline(always)]
    fn from(variant: WtofMsk) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WTOF_MSK` reader - Wake-up by Timeout Flag Mask"]
pub type WtofMskR = crate::BitReader<WtofMsk>;
impl WtofMskR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> WtofMsk {
        match self.bits {
            false => WtofMsk::Disable,
            true => WtofMsk::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == WtofMsk::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == WtofMsk::Enable
    }
}
#[doc = "Field `WTOF_MSK` writer - Wake-up by Timeout Flag Mask"]
pub type WtofMskW<'a, REG> = crate::BitWriter<'a, REG, WtofMsk>;
impl<'a, REG> WtofMskW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(WtofMsk::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(WtofMsk::Enable)
    }
}
impl R {
    #[doc = "Bits 0:1 - Filtering Combination Selection"]
    #[inline(always)]
    pub fn fcs(&self) -> FcsR {
        FcsR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 2:3 - ID Filtering Selection"]
    #[inline(always)]
    pub fn idfs(&self) -> IdfsR {
        IdfsR::new(((self.bits >> 2) & 3) as u8)
    }
    #[doc = "Bits 4:5 - Payload Filtering Selection"]
    #[inline(always)]
    pub fn plfs(&self) -> PlfsR {
        PlfsR::new(((self.bits >> 4) & 3) as u8)
    }
    #[doc = "Bits 8:15 - Number of Messages Matching the Same Filtering Criteria"]
    #[inline(always)]
    pub fn nmatch(&self) -> NmatchR {
        NmatchR::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bit 16 - Wake-up by Matching Flag Mask"]
    #[inline(always)]
    pub fn wumf_msk(&self) -> WumfMskR {
        WumfMskR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Wake-up by Timeout Flag Mask"]
    #[inline(always)]
    pub fn wtof_msk(&self) -> WtofMskR {
        WtofMskR::new(((self.bits >> 17) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:1 - Filtering Combination Selection"]
    #[inline(always)]
    pub fn fcs(&mut self) -> FcsW<Ctrl1PnSpec> {
        FcsW::new(self, 0)
    }
    #[doc = "Bits 2:3 - ID Filtering Selection"]
    #[inline(always)]
    pub fn idfs(&mut self) -> IdfsW<Ctrl1PnSpec> {
        IdfsW::new(self, 2)
    }
    #[doc = "Bits 4:5 - Payload Filtering Selection"]
    #[inline(always)]
    pub fn plfs(&mut self) -> PlfsW<Ctrl1PnSpec> {
        PlfsW::new(self, 4)
    }
    #[doc = "Bits 8:15 - Number of Messages Matching the Same Filtering Criteria"]
    #[inline(always)]
    pub fn nmatch(&mut self) -> NmatchW<Ctrl1PnSpec> {
        NmatchW::new(self, 8)
    }
    #[doc = "Bit 16 - Wake-up by Matching Flag Mask"]
    #[inline(always)]
    pub fn wumf_msk(&mut self) -> WumfMskW<Ctrl1PnSpec> {
        WumfMskW::new(self, 16)
    }
    #[doc = "Bit 17 - Wake-up by Timeout Flag Mask"]
    #[inline(always)]
    pub fn wtof_msk(&mut self) -> WtofMskW<Ctrl1PnSpec> {
        WtofMskW::new(self, 17)
    }
}
#[doc = "Pretended Networking Control 1\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl1_pn::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl1_pn::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ctrl1PnSpec;
impl crate::RegisterSpec for Ctrl1PnSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ctrl1_pn::R`](R) reader structure"]
impl crate::Readable for Ctrl1PnSpec {}
#[doc = "`write(|w| ..)` method takes [`ctrl1_pn::W`](W) writer structure"]
impl crate::Writable for Ctrl1PnSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CTRL1_PN to value 0x0100"]
impl crate::Resettable for Ctrl1PnSpec {
    const RESET_VALUE: u32 = 0x0100;
}

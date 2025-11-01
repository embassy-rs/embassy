#[doc = "Register `STALL_OL_DIS` reader"]
pub type R = crate::R<StallOlDisSpec>;
#[doc = "Register `STALL_OL_DIS` writer"]
pub type W = crate::W<StallOlDisSpec>;
#[doc = "Disable Endpoint 0 OUT Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallODis0 {
    #[doc = "0: Enable"]
    EnEp0OutStall = 0,
    #[doc = "1: Disable"]
    DisEp0OutStall = 1,
}
impl From<StallODis0> for bool {
    #[inline(always)]
    fn from(variant: StallODis0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_O_DIS0` reader - Disable Endpoint 0 OUT Direction"]
pub type StallODis0R = crate::BitReader<StallODis0>;
impl StallODis0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallODis0 {
        match self.bits {
            false => StallODis0::EnEp0OutStall,
            true => StallODis0::DisEp0OutStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep0_out_stall(&self) -> bool {
        *self == StallODis0::EnEp0OutStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep0_out_stall(&self) -> bool {
        *self == StallODis0::DisEp0OutStall
    }
}
#[doc = "Field `STALL_O_DIS0` writer - Disable Endpoint 0 OUT Direction"]
pub type StallODis0W<'a, REG> = crate::BitWriter<'a, REG, StallODis0>;
impl<'a, REG> StallODis0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep0_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis0::EnEp0OutStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep0_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis0::DisEp0OutStall)
    }
}
#[doc = "Disable Endpoint 1 OUT Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallODis1 {
    #[doc = "0: Enable"]
    EnEp1OutStall = 0,
    #[doc = "1: Disable"]
    DisEp1OutStall = 1,
}
impl From<StallODis1> for bool {
    #[inline(always)]
    fn from(variant: StallODis1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_O_DIS1` reader - Disable Endpoint 1 OUT Direction"]
pub type StallODis1R = crate::BitReader<StallODis1>;
impl StallODis1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallODis1 {
        match self.bits {
            false => StallODis1::EnEp1OutStall,
            true => StallODis1::DisEp1OutStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep1_out_stall(&self) -> bool {
        *self == StallODis1::EnEp1OutStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep1_out_stall(&self) -> bool {
        *self == StallODis1::DisEp1OutStall
    }
}
#[doc = "Field `STALL_O_DIS1` writer - Disable Endpoint 1 OUT Direction"]
pub type StallODis1W<'a, REG> = crate::BitWriter<'a, REG, StallODis1>;
impl<'a, REG> StallODis1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep1_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis1::EnEp1OutStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep1_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis1::DisEp1OutStall)
    }
}
#[doc = "Disable Endpoint 2 OUT Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallODis2 {
    #[doc = "0: Enable"]
    EnEp2OutStall = 0,
    #[doc = "1: Disable"]
    DisEp2OutStall = 1,
}
impl From<StallODis2> for bool {
    #[inline(always)]
    fn from(variant: StallODis2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_O_DIS2` reader - Disable Endpoint 2 OUT Direction"]
pub type StallODis2R = crate::BitReader<StallODis2>;
impl StallODis2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallODis2 {
        match self.bits {
            false => StallODis2::EnEp2OutStall,
            true => StallODis2::DisEp2OutStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep2_out_stall(&self) -> bool {
        *self == StallODis2::EnEp2OutStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep2_out_stall(&self) -> bool {
        *self == StallODis2::DisEp2OutStall
    }
}
#[doc = "Field `STALL_O_DIS2` writer - Disable Endpoint 2 OUT Direction"]
pub type StallODis2W<'a, REG> = crate::BitWriter<'a, REG, StallODis2>;
impl<'a, REG> StallODis2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep2_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis2::EnEp2OutStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep2_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis2::DisEp2OutStall)
    }
}
#[doc = "Disable Endpoint 3 OUT Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallODis3 {
    #[doc = "0: Enable"]
    EnEp3OutStall = 0,
    #[doc = "1: Disable"]
    DisEp3OutStall = 1,
}
impl From<StallODis3> for bool {
    #[inline(always)]
    fn from(variant: StallODis3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_O_DIS3` reader - Disable Endpoint 3 OUT Direction"]
pub type StallODis3R = crate::BitReader<StallODis3>;
impl StallODis3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallODis3 {
        match self.bits {
            false => StallODis3::EnEp3OutStall,
            true => StallODis3::DisEp3OutStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep3_out_stall(&self) -> bool {
        *self == StallODis3::EnEp3OutStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep3_out_stall(&self) -> bool {
        *self == StallODis3::DisEp3OutStall
    }
}
#[doc = "Field `STALL_O_DIS3` writer - Disable Endpoint 3 OUT Direction"]
pub type StallODis3W<'a, REG> = crate::BitWriter<'a, REG, StallODis3>;
impl<'a, REG> StallODis3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep3_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis3::EnEp3OutStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep3_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis3::DisEp3OutStall)
    }
}
#[doc = "Disable Endpoint 4 OUT Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallODis4 {
    #[doc = "0: Enable"]
    EnEp4OutStall = 0,
    #[doc = "1: Disable"]
    DisEp4OutStall = 1,
}
impl From<StallODis4> for bool {
    #[inline(always)]
    fn from(variant: StallODis4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_O_DIS4` reader - Disable Endpoint 4 OUT Direction"]
pub type StallODis4R = crate::BitReader<StallODis4>;
impl StallODis4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallODis4 {
        match self.bits {
            false => StallODis4::EnEp4OutStall,
            true => StallODis4::DisEp4OutStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep4_out_stall(&self) -> bool {
        *self == StallODis4::EnEp4OutStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep4_out_stall(&self) -> bool {
        *self == StallODis4::DisEp4OutStall
    }
}
#[doc = "Field `STALL_O_DIS4` writer - Disable Endpoint 4 OUT Direction"]
pub type StallODis4W<'a, REG> = crate::BitWriter<'a, REG, StallODis4>;
impl<'a, REG> StallODis4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep4_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis4::EnEp4OutStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep4_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis4::DisEp4OutStall)
    }
}
#[doc = "Disable Endpoint 5 OUT Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallODis5 {
    #[doc = "0: Enable"]
    EnEp5OutStall = 0,
    #[doc = "1: Disable"]
    DisEp5OutStall = 1,
}
impl From<StallODis5> for bool {
    #[inline(always)]
    fn from(variant: StallODis5) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_O_DIS5` reader - Disable Endpoint 5 OUT Direction"]
pub type StallODis5R = crate::BitReader<StallODis5>;
impl StallODis5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallODis5 {
        match self.bits {
            false => StallODis5::EnEp5OutStall,
            true => StallODis5::DisEp5OutStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep5_out_stall(&self) -> bool {
        *self == StallODis5::EnEp5OutStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep5_out_stall(&self) -> bool {
        *self == StallODis5::DisEp5OutStall
    }
}
#[doc = "Field `STALL_O_DIS5` writer - Disable Endpoint 5 OUT Direction"]
pub type StallODis5W<'a, REG> = crate::BitWriter<'a, REG, StallODis5>;
impl<'a, REG> StallODis5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep5_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis5::EnEp5OutStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep5_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis5::DisEp5OutStall)
    }
}
#[doc = "Disable Endpoint 6 OUT Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallODis6 {
    #[doc = "0: Enable"]
    EnEp6OutStall = 0,
    #[doc = "1: Disable"]
    DisEp6OutStall = 1,
}
impl From<StallODis6> for bool {
    #[inline(always)]
    fn from(variant: StallODis6) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_O_DIS6` reader - Disable Endpoint 6 OUT Direction"]
pub type StallODis6R = crate::BitReader<StallODis6>;
impl StallODis6R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallODis6 {
        match self.bits {
            false => StallODis6::EnEp6OutStall,
            true => StallODis6::DisEp6OutStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep6_out_stall(&self) -> bool {
        *self == StallODis6::EnEp6OutStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep6_out_stall(&self) -> bool {
        *self == StallODis6::DisEp6OutStall
    }
}
#[doc = "Field `STALL_O_DIS6` writer - Disable Endpoint 6 OUT Direction"]
pub type StallODis6W<'a, REG> = crate::BitWriter<'a, REG, StallODis6>;
impl<'a, REG> StallODis6W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep6_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis6::EnEp6OutStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep6_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis6::DisEp6OutStall)
    }
}
#[doc = "Disable Endpoint 7 OUT Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallODis7 {
    #[doc = "0: Enable"]
    EnEp7OutStall = 0,
    #[doc = "1: Disable"]
    DisEp7OutStall = 1,
}
impl From<StallODis7> for bool {
    #[inline(always)]
    fn from(variant: StallODis7) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_O_DIS7` reader - Disable Endpoint 7 OUT Direction"]
pub type StallODis7R = crate::BitReader<StallODis7>;
impl StallODis7R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallODis7 {
        match self.bits {
            false => StallODis7::EnEp7OutStall,
            true => StallODis7::DisEp7OutStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep7_out_stall(&self) -> bool {
        *self == StallODis7::EnEp7OutStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep7_out_stall(&self) -> bool {
        *self == StallODis7::DisEp7OutStall
    }
}
#[doc = "Field `STALL_O_DIS7` writer - Disable Endpoint 7 OUT Direction"]
pub type StallODis7W<'a, REG> = crate::BitWriter<'a, REG, StallODis7>;
impl<'a, REG> StallODis7W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep7_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis7::EnEp7OutStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep7_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis7::DisEp7OutStall)
    }
}
impl R {
    #[doc = "Bit 0 - Disable Endpoint 0 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis0(&self) -> StallODis0R {
        StallODis0R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Disable Endpoint 1 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis1(&self) -> StallODis1R {
        StallODis1R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Disable Endpoint 2 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis2(&self) -> StallODis2R {
        StallODis2R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Disable Endpoint 3 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis3(&self) -> StallODis3R {
        StallODis3R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Disable Endpoint 4 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis4(&self) -> StallODis4R {
        StallODis4R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Disable Endpoint 5 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis5(&self) -> StallODis5R {
        StallODis5R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Disable Endpoint 6 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis6(&self) -> StallODis6R {
        StallODis6R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Disable Endpoint 7 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis7(&self) -> StallODis7R {
        StallODis7R::new(((self.bits >> 7) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Disable Endpoint 0 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis0(&mut self) -> StallODis0W<StallOlDisSpec> {
        StallODis0W::new(self, 0)
    }
    #[doc = "Bit 1 - Disable Endpoint 1 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis1(&mut self) -> StallODis1W<StallOlDisSpec> {
        StallODis1W::new(self, 1)
    }
    #[doc = "Bit 2 - Disable Endpoint 2 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis2(&mut self) -> StallODis2W<StallOlDisSpec> {
        StallODis2W::new(self, 2)
    }
    #[doc = "Bit 3 - Disable Endpoint 3 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis3(&mut self) -> StallODis3W<StallOlDisSpec> {
        StallODis3W::new(self, 3)
    }
    #[doc = "Bit 4 - Disable Endpoint 4 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis4(&mut self) -> StallODis4W<StallOlDisSpec> {
        StallODis4W::new(self, 4)
    }
    #[doc = "Bit 5 - Disable Endpoint 5 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis5(&mut self) -> StallODis5W<StallOlDisSpec> {
        StallODis5W::new(self, 5)
    }
    #[doc = "Bit 6 - Disable Endpoint 6 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis6(&mut self) -> StallODis6W<StallOlDisSpec> {
        StallODis6W::new(self, 6)
    }
    #[doc = "Bit 7 - Disable Endpoint 7 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis7(&mut self) -> StallODis7W<StallOlDisSpec> {
        StallODis7W::new(self, 7)
    }
}
#[doc = "Peripheral Mode Stall Disable for Endpoints 7 to 0 in OUT Direction\n\nYou can [`read`](crate::Reg::read) this register and get [`stall_ol_dis::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`stall_ol_dis::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct StallOlDisSpec;
impl crate::RegisterSpec for StallOlDisSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`stall_ol_dis::R`](R) reader structure"]
impl crate::Readable for StallOlDisSpec {}
#[doc = "`write(|w| ..)` method takes [`stall_ol_dis::W`](W) writer structure"]
impl crate::Writable for StallOlDisSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets STALL_OL_DIS to value 0"]
impl crate::Resettable for StallOlDisSpec {}

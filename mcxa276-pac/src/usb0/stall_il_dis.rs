#[doc = "Register `STALL_IL_DIS` reader"]
pub type R = crate::R<StallIlDisSpec>;
#[doc = "Register `STALL_IL_DIS` writer"]
pub type W = crate::W<StallIlDisSpec>;
#[doc = "Disable Endpoint 0 IN Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallIDis0 {
    #[doc = "0: Enable"]
    EnEp0InStall = 0,
    #[doc = "1: Disable"]
    DisEp0InStall = 1,
}
impl From<StallIDis0> for bool {
    #[inline(always)]
    fn from(variant: StallIDis0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_I_DIS0` reader - Disable Endpoint 0 IN Direction"]
pub type StallIDis0R = crate::BitReader<StallIDis0>;
impl StallIDis0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallIDis0 {
        match self.bits {
            false => StallIDis0::EnEp0InStall,
            true => StallIDis0::DisEp0InStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep0_in_stall(&self) -> bool {
        *self == StallIDis0::EnEp0InStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep0_in_stall(&self) -> bool {
        *self == StallIDis0::DisEp0InStall
    }
}
#[doc = "Field `STALL_I_DIS0` writer - Disable Endpoint 0 IN Direction"]
pub type StallIDis0W<'a, REG> = crate::BitWriter<'a, REG, StallIDis0>;
impl<'a, REG> StallIDis0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep0_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis0::EnEp0InStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep0_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis0::DisEp0InStall)
    }
}
#[doc = "Disable Endpoint 1 IN Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallIDis1 {
    #[doc = "0: Enable"]
    EnEp1InStall = 0,
    #[doc = "1: Disable"]
    DisEp1InStall = 1,
}
impl From<StallIDis1> for bool {
    #[inline(always)]
    fn from(variant: StallIDis1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_I_DIS1` reader - Disable Endpoint 1 IN Direction"]
pub type StallIDis1R = crate::BitReader<StallIDis1>;
impl StallIDis1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallIDis1 {
        match self.bits {
            false => StallIDis1::EnEp1InStall,
            true => StallIDis1::DisEp1InStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep1_in_stall(&self) -> bool {
        *self == StallIDis1::EnEp1InStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep1_in_stall(&self) -> bool {
        *self == StallIDis1::DisEp1InStall
    }
}
#[doc = "Field `STALL_I_DIS1` writer - Disable Endpoint 1 IN Direction"]
pub type StallIDis1W<'a, REG> = crate::BitWriter<'a, REG, StallIDis1>;
impl<'a, REG> StallIDis1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep1_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis1::EnEp1InStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep1_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis1::DisEp1InStall)
    }
}
#[doc = "Disable Endpoint 2 IN Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallIDis2 {
    #[doc = "0: Enable"]
    EnEp2InStall = 0,
    #[doc = "1: Disable"]
    DisEp2InStall = 1,
}
impl From<StallIDis2> for bool {
    #[inline(always)]
    fn from(variant: StallIDis2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_I_DIS2` reader - Disable Endpoint 2 IN Direction"]
pub type StallIDis2R = crate::BitReader<StallIDis2>;
impl StallIDis2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallIDis2 {
        match self.bits {
            false => StallIDis2::EnEp2InStall,
            true => StallIDis2::DisEp2InStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep2_in_stall(&self) -> bool {
        *self == StallIDis2::EnEp2InStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep2_in_stall(&self) -> bool {
        *self == StallIDis2::DisEp2InStall
    }
}
#[doc = "Field `STALL_I_DIS2` writer - Disable Endpoint 2 IN Direction"]
pub type StallIDis2W<'a, REG> = crate::BitWriter<'a, REG, StallIDis2>;
impl<'a, REG> StallIDis2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep2_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis2::EnEp2InStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep2_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis2::DisEp2InStall)
    }
}
#[doc = "Disable Endpoint 3 IN Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallIDis3 {
    #[doc = "0: Enable"]
    EnEp3InStall = 0,
    #[doc = "1: Disable"]
    DisEp3InStall = 1,
}
impl From<StallIDis3> for bool {
    #[inline(always)]
    fn from(variant: StallIDis3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_I_DIS3` reader - Disable Endpoint 3 IN Direction"]
pub type StallIDis3R = crate::BitReader<StallIDis3>;
impl StallIDis3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallIDis3 {
        match self.bits {
            false => StallIDis3::EnEp3InStall,
            true => StallIDis3::DisEp3InStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep3_in_stall(&self) -> bool {
        *self == StallIDis3::EnEp3InStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep3_in_stall(&self) -> bool {
        *self == StallIDis3::DisEp3InStall
    }
}
#[doc = "Field `STALL_I_DIS3` writer - Disable Endpoint 3 IN Direction"]
pub type StallIDis3W<'a, REG> = crate::BitWriter<'a, REG, StallIDis3>;
impl<'a, REG> StallIDis3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep3_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis3::EnEp3InStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep3_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis3::DisEp3InStall)
    }
}
#[doc = "Disable Endpoint 4 IN Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallIDis4 {
    #[doc = "0: Enable"]
    EnEp4InStall = 0,
    #[doc = "1: Disable"]
    DisEp4InStall = 1,
}
impl From<StallIDis4> for bool {
    #[inline(always)]
    fn from(variant: StallIDis4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_I_DIS4` reader - Disable Endpoint 4 IN Direction"]
pub type StallIDis4R = crate::BitReader<StallIDis4>;
impl StallIDis4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallIDis4 {
        match self.bits {
            false => StallIDis4::EnEp4InStall,
            true => StallIDis4::DisEp4InStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep4_in_stall(&self) -> bool {
        *self == StallIDis4::EnEp4InStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep4_in_stall(&self) -> bool {
        *self == StallIDis4::DisEp4InStall
    }
}
#[doc = "Field `STALL_I_DIS4` writer - Disable Endpoint 4 IN Direction"]
pub type StallIDis4W<'a, REG> = crate::BitWriter<'a, REG, StallIDis4>;
impl<'a, REG> StallIDis4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep4_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis4::EnEp4InStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep4_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis4::DisEp4InStall)
    }
}
#[doc = "Disable Endpoint 5 IN Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallIDis5 {
    #[doc = "0: Enable"]
    EnEp5InStall = 0,
    #[doc = "1: Disable"]
    DisEp5InStall = 1,
}
impl From<StallIDis5> for bool {
    #[inline(always)]
    fn from(variant: StallIDis5) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_I_DIS5` reader - Disable Endpoint 5 IN Direction"]
pub type StallIDis5R = crate::BitReader<StallIDis5>;
impl StallIDis5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallIDis5 {
        match self.bits {
            false => StallIDis5::EnEp5InStall,
            true => StallIDis5::DisEp5InStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep5_in_stall(&self) -> bool {
        *self == StallIDis5::EnEp5InStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep5_in_stall(&self) -> bool {
        *self == StallIDis5::DisEp5InStall
    }
}
#[doc = "Field `STALL_I_DIS5` writer - Disable Endpoint 5 IN Direction"]
pub type StallIDis5W<'a, REG> = crate::BitWriter<'a, REG, StallIDis5>;
impl<'a, REG> StallIDis5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep5_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis5::EnEp5InStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep5_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis5::DisEp5InStall)
    }
}
#[doc = "Disable Endpoint 6 IN Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallIDis6 {
    #[doc = "0: Enable"]
    EnEp6InStall = 0,
    #[doc = "1: Disable"]
    DisEp6InStall = 1,
}
impl From<StallIDis6> for bool {
    #[inline(always)]
    fn from(variant: StallIDis6) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_I_DIS6` reader - Disable Endpoint 6 IN Direction"]
pub type StallIDis6R = crate::BitReader<StallIDis6>;
impl StallIDis6R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallIDis6 {
        match self.bits {
            false => StallIDis6::EnEp6InStall,
            true => StallIDis6::DisEp6InStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep6_in_stall(&self) -> bool {
        *self == StallIDis6::EnEp6InStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep6_in_stall(&self) -> bool {
        *self == StallIDis6::DisEp6InStall
    }
}
#[doc = "Field `STALL_I_DIS6` writer - Disable Endpoint 6 IN Direction"]
pub type StallIDis6W<'a, REG> = crate::BitWriter<'a, REG, StallIDis6>;
impl<'a, REG> StallIDis6W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep6_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis6::EnEp6InStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep6_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis6::DisEp6InStall)
    }
}
#[doc = "Disable Endpoint 7 IN Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallIDis7 {
    #[doc = "0: Enable"]
    EnEp7InStall = 0,
    #[doc = "1: Disable"]
    DisEp7InStall = 1,
}
impl From<StallIDis7> for bool {
    #[inline(always)]
    fn from(variant: StallIDis7) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_I_DIS7` reader - Disable Endpoint 7 IN Direction"]
pub type StallIDis7R = crate::BitReader<StallIDis7>;
impl StallIDis7R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallIDis7 {
        match self.bits {
            false => StallIDis7::EnEp7InStall,
            true => StallIDis7::DisEp7InStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep7_in_stall(&self) -> bool {
        *self == StallIDis7::EnEp7InStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep7_in_stall(&self) -> bool {
        *self == StallIDis7::DisEp7InStall
    }
}
#[doc = "Field `STALL_I_DIS7` writer - Disable Endpoint 7 IN Direction"]
pub type StallIDis7W<'a, REG> = crate::BitWriter<'a, REG, StallIDis7>;
impl<'a, REG> StallIDis7W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep7_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis7::EnEp7InStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep7_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis7::DisEp7InStall)
    }
}
impl R {
    #[doc = "Bit 0 - Disable Endpoint 0 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis0(&self) -> StallIDis0R {
        StallIDis0R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Disable Endpoint 1 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis1(&self) -> StallIDis1R {
        StallIDis1R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Disable Endpoint 2 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis2(&self) -> StallIDis2R {
        StallIDis2R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Disable Endpoint 3 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis3(&self) -> StallIDis3R {
        StallIDis3R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Disable Endpoint 4 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis4(&self) -> StallIDis4R {
        StallIDis4R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Disable Endpoint 5 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis5(&self) -> StallIDis5R {
        StallIDis5R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Disable Endpoint 6 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis6(&self) -> StallIDis6R {
        StallIDis6R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Disable Endpoint 7 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis7(&self) -> StallIDis7R {
        StallIDis7R::new(((self.bits >> 7) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Disable Endpoint 0 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis0(&mut self) -> StallIDis0W<StallIlDisSpec> {
        StallIDis0W::new(self, 0)
    }
    #[doc = "Bit 1 - Disable Endpoint 1 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis1(&mut self) -> StallIDis1W<StallIlDisSpec> {
        StallIDis1W::new(self, 1)
    }
    #[doc = "Bit 2 - Disable Endpoint 2 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis2(&mut self) -> StallIDis2W<StallIlDisSpec> {
        StallIDis2W::new(self, 2)
    }
    #[doc = "Bit 3 - Disable Endpoint 3 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis3(&mut self) -> StallIDis3W<StallIlDisSpec> {
        StallIDis3W::new(self, 3)
    }
    #[doc = "Bit 4 - Disable Endpoint 4 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis4(&mut self) -> StallIDis4W<StallIlDisSpec> {
        StallIDis4W::new(self, 4)
    }
    #[doc = "Bit 5 - Disable Endpoint 5 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis5(&mut self) -> StallIDis5W<StallIlDisSpec> {
        StallIDis5W::new(self, 5)
    }
    #[doc = "Bit 6 - Disable Endpoint 6 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis6(&mut self) -> StallIDis6W<StallIlDisSpec> {
        StallIDis6W::new(self, 6)
    }
    #[doc = "Bit 7 - Disable Endpoint 7 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis7(&mut self) -> StallIDis7W<StallIlDisSpec> {
        StallIDis7W::new(self, 7)
    }
}
#[doc = "Peripheral Mode Stall Disable for Endpoints 7 to 0 in IN Direction\n\nYou can [`read`](crate::Reg::read) this register and get [`stall_il_dis::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`stall_il_dis::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct StallIlDisSpec;
impl crate::RegisterSpec for StallIlDisSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`stall_il_dis::R`](R) reader structure"]
impl crate::Readable for StallIlDisSpec {}
#[doc = "`write(|w| ..)` method takes [`stall_il_dis::W`](W) writer structure"]
impl crate::Writable for StallIlDisSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets STALL_IL_DIS to value 0"]
impl crate::Resettable for StallIlDisSpec {}

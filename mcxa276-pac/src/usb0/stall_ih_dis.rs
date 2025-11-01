#[doc = "Register `STALL_IH_DIS` reader"]
pub type R = crate::R<StallIhDisSpec>;
#[doc = "Register `STALL_IH_DIS` writer"]
pub type W = crate::W<StallIhDisSpec>;
#[doc = "Disable Endpoint 8 IN Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallIDis8 {
    #[doc = "0: Enable"]
    EnEp8InStall = 0,
    #[doc = "1: Disable"]
    DisEp8InStall = 1,
}
impl From<StallIDis8> for bool {
    #[inline(always)]
    fn from(variant: StallIDis8) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_I_DIS8` reader - Disable Endpoint 8 IN Direction"]
pub type StallIDis8R = crate::BitReader<StallIDis8>;
impl StallIDis8R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallIDis8 {
        match self.bits {
            false => StallIDis8::EnEp8InStall,
            true => StallIDis8::DisEp8InStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep8_in_stall(&self) -> bool {
        *self == StallIDis8::EnEp8InStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep8_in_stall(&self) -> bool {
        *self == StallIDis8::DisEp8InStall
    }
}
#[doc = "Field `STALL_I_DIS8` writer - Disable Endpoint 8 IN Direction"]
pub type StallIDis8W<'a, REG> = crate::BitWriter<'a, REG, StallIDis8>;
impl<'a, REG> StallIDis8W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep8_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis8::EnEp8InStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep8_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis8::DisEp8InStall)
    }
}
#[doc = "Disable Endpoint 9 IN Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallIDis9 {
    #[doc = "0: Enable"]
    EnEp9InStall = 0,
    #[doc = "1: Disable"]
    DisEp9InStall = 1,
}
impl From<StallIDis9> for bool {
    #[inline(always)]
    fn from(variant: StallIDis9) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_I_DIS9` reader - Disable Endpoint 9 IN Direction"]
pub type StallIDis9R = crate::BitReader<StallIDis9>;
impl StallIDis9R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallIDis9 {
        match self.bits {
            false => StallIDis9::EnEp9InStall,
            true => StallIDis9::DisEp9InStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep9_in_stall(&self) -> bool {
        *self == StallIDis9::EnEp9InStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep9_in_stall(&self) -> bool {
        *self == StallIDis9::DisEp9InStall
    }
}
#[doc = "Field `STALL_I_DIS9` writer - Disable Endpoint 9 IN Direction"]
pub type StallIDis9W<'a, REG> = crate::BitWriter<'a, REG, StallIDis9>;
impl<'a, REG> StallIDis9W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep9_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis9::EnEp9InStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep9_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis9::DisEp9InStall)
    }
}
#[doc = "Disable Endpoint 10 IN Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallIDis10 {
    #[doc = "0: Enable"]
    EnEp10InStall = 0,
    #[doc = "1: Disable"]
    DisEp10InStall = 1,
}
impl From<StallIDis10> for bool {
    #[inline(always)]
    fn from(variant: StallIDis10) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_I_DIS10` reader - Disable Endpoint 10 IN Direction"]
pub type StallIDis10R = crate::BitReader<StallIDis10>;
impl StallIDis10R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallIDis10 {
        match self.bits {
            false => StallIDis10::EnEp10InStall,
            true => StallIDis10::DisEp10InStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep10_in_stall(&self) -> bool {
        *self == StallIDis10::EnEp10InStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep10_in_stall(&self) -> bool {
        *self == StallIDis10::DisEp10InStall
    }
}
#[doc = "Field `STALL_I_DIS10` writer - Disable Endpoint 10 IN Direction"]
pub type StallIDis10W<'a, REG> = crate::BitWriter<'a, REG, StallIDis10>;
impl<'a, REG> StallIDis10W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep10_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis10::EnEp10InStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep10_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis10::DisEp10InStall)
    }
}
#[doc = "Disable Endpoint 11 IN Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallIDis11 {
    #[doc = "0: Enable"]
    EnEp11InStall = 0,
    #[doc = "1: Disable"]
    DisEp11InStall = 1,
}
impl From<StallIDis11> for bool {
    #[inline(always)]
    fn from(variant: StallIDis11) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_I_DIS11` reader - Disable Endpoint 11 IN Direction"]
pub type StallIDis11R = crate::BitReader<StallIDis11>;
impl StallIDis11R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallIDis11 {
        match self.bits {
            false => StallIDis11::EnEp11InStall,
            true => StallIDis11::DisEp11InStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep11_in_stall(&self) -> bool {
        *self == StallIDis11::EnEp11InStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep11_in_stall(&self) -> bool {
        *self == StallIDis11::DisEp11InStall
    }
}
#[doc = "Field `STALL_I_DIS11` writer - Disable Endpoint 11 IN Direction"]
pub type StallIDis11W<'a, REG> = crate::BitWriter<'a, REG, StallIDis11>;
impl<'a, REG> StallIDis11W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep11_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis11::EnEp11InStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep11_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis11::DisEp11InStall)
    }
}
#[doc = "Disable Endpoint 12 IN Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallIDis12 {
    #[doc = "0: Enable"]
    EnEp12InStall = 0,
    #[doc = "1: Disable"]
    DisEp12InStall = 1,
}
impl From<StallIDis12> for bool {
    #[inline(always)]
    fn from(variant: StallIDis12) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_I_DIS12` reader - Disable Endpoint 12 IN Direction"]
pub type StallIDis12R = crate::BitReader<StallIDis12>;
impl StallIDis12R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallIDis12 {
        match self.bits {
            false => StallIDis12::EnEp12InStall,
            true => StallIDis12::DisEp12InStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep12_in_stall(&self) -> bool {
        *self == StallIDis12::EnEp12InStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep12_in_stall(&self) -> bool {
        *self == StallIDis12::DisEp12InStall
    }
}
#[doc = "Field `STALL_I_DIS12` writer - Disable Endpoint 12 IN Direction"]
pub type StallIDis12W<'a, REG> = crate::BitWriter<'a, REG, StallIDis12>;
impl<'a, REG> StallIDis12W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep12_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis12::EnEp12InStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep12_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis12::DisEp12InStall)
    }
}
#[doc = "Disable Endpoint 13 IN Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallIDis13 {
    #[doc = "0: Enable"]
    EnEp13InStall = 0,
    #[doc = "1: Disable"]
    DisEp13InStall = 1,
}
impl From<StallIDis13> for bool {
    #[inline(always)]
    fn from(variant: StallIDis13) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_I_DIS13` reader - Disable Endpoint 13 IN Direction"]
pub type StallIDis13R = crate::BitReader<StallIDis13>;
impl StallIDis13R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallIDis13 {
        match self.bits {
            false => StallIDis13::EnEp13InStall,
            true => StallIDis13::DisEp13InStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep13_in_stall(&self) -> bool {
        *self == StallIDis13::EnEp13InStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep13_in_stall(&self) -> bool {
        *self == StallIDis13::DisEp13InStall
    }
}
#[doc = "Field `STALL_I_DIS13` writer - Disable Endpoint 13 IN Direction"]
pub type StallIDis13W<'a, REG> = crate::BitWriter<'a, REG, StallIDis13>;
impl<'a, REG> StallIDis13W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep13_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis13::EnEp13InStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep13_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis13::DisEp13InStall)
    }
}
#[doc = "Disable Endpoint 14 IN Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallIDis14 {
    #[doc = "0: Enable"]
    EnEp14InStall = 0,
    #[doc = "1: Disable"]
    DisEp14InStall = 1,
}
impl From<StallIDis14> for bool {
    #[inline(always)]
    fn from(variant: StallIDis14) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_I_DIS14` reader - Disable Endpoint 14 IN Direction"]
pub type StallIDis14R = crate::BitReader<StallIDis14>;
impl StallIDis14R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallIDis14 {
        match self.bits {
            false => StallIDis14::EnEp14InStall,
            true => StallIDis14::DisEp14InStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep14_in_stall(&self) -> bool {
        *self == StallIDis14::EnEp14InStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep14_in_stall(&self) -> bool {
        *self == StallIDis14::DisEp14InStall
    }
}
#[doc = "Field `STALL_I_DIS14` writer - Disable Endpoint 14 IN Direction"]
pub type StallIDis14W<'a, REG> = crate::BitWriter<'a, REG, StallIDis14>;
impl<'a, REG> StallIDis14W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep14_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis14::EnEp14InStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep14_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis14::DisEp14InStall)
    }
}
#[doc = "Disable Endpoint 15 IN Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallIDis15 {
    #[doc = "0: Enable"]
    EnEp15InStall = 0,
    #[doc = "1: Disable"]
    DisEp15InStall = 1,
}
impl From<StallIDis15> for bool {
    #[inline(always)]
    fn from(variant: StallIDis15) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_I_DIS15` reader - Disable Endpoint 15 IN Direction"]
pub type StallIDis15R = crate::BitReader<StallIDis15>;
impl StallIDis15R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallIDis15 {
        match self.bits {
            false => StallIDis15::EnEp15InStall,
            true => StallIDis15::DisEp15InStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep15_in_stall(&self) -> bool {
        *self == StallIDis15::EnEp15InStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep15_in_stall(&self) -> bool {
        *self == StallIDis15::DisEp15InStall
    }
}
#[doc = "Field `STALL_I_DIS15` writer - Disable Endpoint 15 IN Direction"]
pub type StallIDis15W<'a, REG> = crate::BitWriter<'a, REG, StallIDis15>;
impl<'a, REG> StallIDis15W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep15_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis15::EnEp15InStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep15_in_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallIDis15::DisEp15InStall)
    }
}
impl R {
    #[doc = "Bit 0 - Disable Endpoint 8 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis8(&self) -> StallIDis8R {
        StallIDis8R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Disable Endpoint 9 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis9(&self) -> StallIDis9R {
        StallIDis9R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Disable Endpoint 10 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis10(&self) -> StallIDis10R {
        StallIDis10R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Disable Endpoint 11 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis11(&self) -> StallIDis11R {
        StallIDis11R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Disable Endpoint 12 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis12(&self) -> StallIDis12R {
        StallIDis12R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Disable Endpoint 13 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis13(&self) -> StallIDis13R {
        StallIDis13R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Disable Endpoint 14 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis14(&self) -> StallIDis14R {
        StallIDis14R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Disable Endpoint 15 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis15(&self) -> StallIDis15R {
        StallIDis15R::new(((self.bits >> 7) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Disable Endpoint 8 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis8(&mut self) -> StallIDis8W<StallIhDisSpec> {
        StallIDis8W::new(self, 0)
    }
    #[doc = "Bit 1 - Disable Endpoint 9 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis9(&mut self) -> StallIDis9W<StallIhDisSpec> {
        StallIDis9W::new(self, 1)
    }
    #[doc = "Bit 2 - Disable Endpoint 10 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis10(&mut self) -> StallIDis10W<StallIhDisSpec> {
        StallIDis10W::new(self, 2)
    }
    #[doc = "Bit 3 - Disable Endpoint 11 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis11(&mut self) -> StallIDis11W<StallIhDisSpec> {
        StallIDis11W::new(self, 3)
    }
    #[doc = "Bit 4 - Disable Endpoint 12 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis12(&mut self) -> StallIDis12W<StallIhDisSpec> {
        StallIDis12W::new(self, 4)
    }
    #[doc = "Bit 5 - Disable Endpoint 13 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis13(&mut self) -> StallIDis13W<StallIhDisSpec> {
        StallIDis13W::new(self, 5)
    }
    #[doc = "Bit 6 - Disable Endpoint 14 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis14(&mut self) -> StallIDis14W<StallIhDisSpec> {
        StallIDis14W::new(self, 6)
    }
    #[doc = "Bit 7 - Disable Endpoint 15 IN Direction"]
    #[inline(always)]
    pub fn stall_i_dis15(&mut self) -> StallIDis15W<StallIhDisSpec> {
        StallIDis15W::new(self, 7)
    }
}
#[doc = "Peripheral Mode Stall Disable for Endpoints 15 to 8 in IN Direction\n\nYou can [`read`](crate::Reg::read) this register and get [`stall_ih_dis::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`stall_ih_dis::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct StallIhDisSpec;
impl crate::RegisterSpec for StallIhDisSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`stall_ih_dis::R`](R) reader structure"]
impl crate::Readable for StallIhDisSpec {}
#[doc = "`write(|w| ..)` method takes [`stall_ih_dis::W`](W) writer structure"]
impl crate::Writable for StallIhDisSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets STALL_IH_DIS to value 0"]
impl crate::Resettable for StallIhDisSpec {}

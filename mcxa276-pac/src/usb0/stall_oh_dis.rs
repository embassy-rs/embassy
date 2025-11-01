#[doc = "Register `STALL_OH_DIS` reader"]
pub type R = crate::R<StallOhDisSpec>;
#[doc = "Register `STALL_OH_DIS` writer"]
pub type W = crate::W<StallOhDisSpec>;
#[doc = "Disable Endpoint 8 OUT Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallODis8 {
    #[doc = "0: Enable"]
    EnEp8OutStall = 0,
    #[doc = "1: Disable"]
    DisEp8OutStall = 1,
}
impl From<StallODis8> for bool {
    #[inline(always)]
    fn from(variant: StallODis8) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_O_DIS8` reader - Disable Endpoint 8 OUT Direction"]
pub type StallODis8R = crate::BitReader<StallODis8>;
impl StallODis8R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallODis8 {
        match self.bits {
            false => StallODis8::EnEp8OutStall,
            true => StallODis8::DisEp8OutStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep8_out_stall(&self) -> bool {
        *self == StallODis8::EnEp8OutStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep8_out_stall(&self) -> bool {
        *self == StallODis8::DisEp8OutStall
    }
}
#[doc = "Field `STALL_O_DIS8` writer - Disable Endpoint 8 OUT Direction"]
pub type StallODis8W<'a, REG> = crate::BitWriter<'a, REG, StallODis8>;
impl<'a, REG> StallODis8W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep8_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis8::EnEp8OutStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep8_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis8::DisEp8OutStall)
    }
}
#[doc = "Disable Endpoint 9 OUT Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallODis9 {
    #[doc = "0: Enable"]
    EnEp9OutStall = 0,
    #[doc = "1: Disable"]
    DisEp9OutStall = 1,
}
impl From<StallODis9> for bool {
    #[inline(always)]
    fn from(variant: StallODis9) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_O_DIS9` reader - Disable Endpoint 9 OUT Direction"]
pub type StallODis9R = crate::BitReader<StallODis9>;
impl StallODis9R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallODis9 {
        match self.bits {
            false => StallODis9::EnEp9OutStall,
            true => StallODis9::DisEp9OutStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep9_out_stall(&self) -> bool {
        *self == StallODis9::EnEp9OutStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep9_out_stall(&self) -> bool {
        *self == StallODis9::DisEp9OutStall
    }
}
#[doc = "Field `STALL_O_DIS9` writer - Disable Endpoint 9 OUT Direction"]
pub type StallODis9W<'a, REG> = crate::BitWriter<'a, REG, StallODis9>;
impl<'a, REG> StallODis9W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep9_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis9::EnEp9OutStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep9_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis9::DisEp9OutStall)
    }
}
#[doc = "Disable Endpoint 10 OUT Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallODis10 {
    #[doc = "0: Enable"]
    EnEp10OutStall = 0,
    #[doc = "1: Disable"]
    DisEp10OutStall = 1,
}
impl From<StallODis10> for bool {
    #[inline(always)]
    fn from(variant: StallODis10) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_O_DIS10` reader - Disable Endpoint 10 OUT Direction"]
pub type StallODis10R = crate::BitReader<StallODis10>;
impl StallODis10R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallODis10 {
        match self.bits {
            false => StallODis10::EnEp10OutStall,
            true => StallODis10::DisEp10OutStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep10_out_stall(&self) -> bool {
        *self == StallODis10::EnEp10OutStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep10_out_stall(&self) -> bool {
        *self == StallODis10::DisEp10OutStall
    }
}
#[doc = "Field `STALL_O_DIS10` writer - Disable Endpoint 10 OUT Direction"]
pub type StallODis10W<'a, REG> = crate::BitWriter<'a, REG, StallODis10>;
impl<'a, REG> StallODis10W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep10_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis10::EnEp10OutStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep10_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis10::DisEp10OutStall)
    }
}
#[doc = "Disable Endpoint 11 OUT Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallODis11 {
    #[doc = "0: Enable"]
    EnEp11OutStall = 0,
    #[doc = "1: Disable"]
    DisEp11OutStall = 1,
}
impl From<StallODis11> for bool {
    #[inline(always)]
    fn from(variant: StallODis11) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_O_DIS11` reader - Disable Endpoint 11 OUT Direction"]
pub type StallODis11R = crate::BitReader<StallODis11>;
impl StallODis11R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallODis11 {
        match self.bits {
            false => StallODis11::EnEp11OutStall,
            true => StallODis11::DisEp11OutStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep11_out_stall(&self) -> bool {
        *self == StallODis11::EnEp11OutStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep11_out_stall(&self) -> bool {
        *self == StallODis11::DisEp11OutStall
    }
}
#[doc = "Field `STALL_O_DIS11` writer - Disable Endpoint 11 OUT Direction"]
pub type StallODis11W<'a, REG> = crate::BitWriter<'a, REG, StallODis11>;
impl<'a, REG> StallODis11W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep11_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis11::EnEp11OutStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep11_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis11::DisEp11OutStall)
    }
}
#[doc = "Disable endpoint 12 OUT direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallODis12 {
    #[doc = "0: Enable"]
    EnEp12OutStall = 0,
    #[doc = "1: Disable"]
    DisEp12OutStall = 1,
}
impl From<StallODis12> for bool {
    #[inline(always)]
    fn from(variant: StallODis12) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_O_DIS12` reader - Disable endpoint 12 OUT direction"]
pub type StallODis12R = crate::BitReader<StallODis12>;
impl StallODis12R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallODis12 {
        match self.bits {
            false => StallODis12::EnEp12OutStall,
            true => StallODis12::DisEp12OutStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep12_out_stall(&self) -> bool {
        *self == StallODis12::EnEp12OutStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep12_out_stall(&self) -> bool {
        *self == StallODis12::DisEp12OutStall
    }
}
#[doc = "Field `STALL_O_DIS12` writer - Disable endpoint 12 OUT direction"]
pub type StallODis12W<'a, REG> = crate::BitWriter<'a, REG, StallODis12>;
impl<'a, REG> StallODis12W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep12_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis12::EnEp12OutStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep12_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis12::DisEp12OutStall)
    }
}
#[doc = "Disable Endpoint 13 OUT Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallODis13 {
    #[doc = "0: Enable"]
    EnEp13OutStall = 0,
    #[doc = "1: Disable"]
    DisEp13OutStall = 1,
}
impl From<StallODis13> for bool {
    #[inline(always)]
    fn from(variant: StallODis13) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_O_DIS13` reader - Disable Endpoint 13 OUT Direction"]
pub type StallODis13R = crate::BitReader<StallODis13>;
impl StallODis13R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallODis13 {
        match self.bits {
            false => StallODis13::EnEp13OutStall,
            true => StallODis13::DisEp13OutStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep13_out_stall(&self) -> bool {
        *self == StallODis13::EnEp13OutStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep13_out_stall(&self) -> bool {
        *self == StallODis13::DisEp13OutStall
    }
}
#[doc = "Field `STALL_O_DIS13` writer - Disable Endpoint 13 OUT Direction"]
pub type StallODis13W<'a, REG> = crate::BitWriter<'a, REG, StallODis13>;
impl<'a, REG> StallODis13W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep13_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis13::EnEp13OutStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep13_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis13::DisEp13OutStall)
    }
}
#[doc = "Disable Endpoint 14 OUT Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallODis14 {
    #[doc = "0: Enable"]
    EnEp14OutStall = 0,
    #[doc = "1: Disable"]
    DisEp14OutStall = 1,
}
impl From<StallODis14> for bool {
    #[inline(always)]
    fn from(variant: StallODis14) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_O_DIS14` reader - Disable Endpoint 14 OUT Direction"]
pub type StallODis14R = crate::BitReader<StallODis14>;
impl StallODis14R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallODis14 {
        match self.bits {
            false => StallODis14::EnEp14OutStall,
            true => StallODis14::DisEp14OutStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep14_out_stall(&self) -> bool {
        *self == StallODis14::EnEp14OutStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep14_out_stall(&self) -> bool {
        *self == StallODis14::DisEp14OutStall
    }
}
#[doc = "Field `STALL_O_DIS14` writer - Disable Endpoint 14 OUT Direction"]
pub type StallODis14W<'a, REG> = crate::BitWriter<'a, REG, StallODis14>;
impl<'a, REG> StallODis14W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep14_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis14::EnEp14OutStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep14_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis14::DisEp14OutStall)
    }
}
#[doc = "Disable Endpoint 15 OUT Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallODis15 {
    #[doc = "0: Enable"]
    EnEp15OutStall = 0,
    #[doc = "1: Disable"]
    DisEp15OutStall = 1,
}
impl From<StallODis15> for bool {
    #[inline(always)]
    fn from(variant: StallODis15) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL_O_DIS15` reader - Disable Endpoint 15 OUT Direction"]
pub type StallODis15R = crate::BitReader<StallODis15>;
impl StallODis15R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StallODis15 {
        match self.bits {
            false => StallODis15::EnEp15OutStall,
            true => StallODis15::DisEp15OutStall,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_ep15_out_stall(&self) -> bool {
        *self == StallODis15::EnEp15OutStall
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_ep15_out_stall(&self) -> bool {
        *self == StallODis15::DisEp15OutStall
    }
}
#[doc = "Field `STALL_O_DIS15` writer - Disable Endpoint 15 OUT Direction"]
pub type StallODis15W<'a, REG> = crate::BitWriter<'a, REG, StallODis15>;
impl<'a, REG> StallODis15W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_ep15_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis15::EnEp15OutStall)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_ep15_out_stall(self) -> &'a mut crate::W<REG> {
        self.variant(StallODis15::DisEp15OutStall)
    }
}
impl R {
    #[doc = "Bit 0 - Disable Endpoint 8 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis8(&self) -> StallODis8R {
        StallODis8R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Disable Endpoint 9 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis9(&self) -> StallODis9R {
        StallODis9R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Disable Endpoint 10 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis10(&self) -> StallODis10R {
        StallODis10R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Disable Endpoint 11 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis11(&self) -> StallODis11R {
        StallODis11R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Disable endpoint 12 OUT direction"]
    #[inline(always)]
    pub fn stall_o_dis12(&self) -> StallODis12R {
        StallODis12R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Disable Endpoint 13 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis13(&self) -> StallODis13R {
        StallODis13R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Disable Endpoint 14 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis14(&self) -> StallODis14R {
        StallODis14R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Disable Endpoint 15 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis15(&self) -> StallODis15R {
        StallODis15R::new(((self.bits >> 7) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Disable Endpoint 8 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis8(&mut self) -> StallODis8W<StallOhDisSpec> {
        StallODis8W::new(self, 0)
    }
    #[doc = "Bit 1 - Disable Endpoint 9 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis9(&mut self) -> StallODis9W<StallOhDisSpec> {
        StallODis9W::new(self, 1)
    }
    #[doc = "Bit 2 - Disable Endpoint 10 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis10(&mut self) -> StallODis10W<StallOhDisSpec> {
        StallODis10W::new(self, 2)
    }
    #[doc = "Bit 3 - Disable Endpoint 11 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis11(&mut self) -> StallODis11W<StallOhDisSpec> {
        StallODis11W::new(self, 3)
    }
    #[doc = "Bit 4 - Disable endpoint 12 OUT direction"]
    #[inline(always)]
    pub fn stall_o_dis12(&mut self) -> StallODis12W<StallOhDisSpec> {
        StallODis12W::new(self, 4)
    }
    #[doc = "Bit 5 - Disable Endpoint 13 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis13(&mut self) -> StallODis13W<StallOhDisSpec> {
        StallODis13W::new(self, 5)
    }
    #[doc = "Bit 6 - Disable Endpoint 14 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis14(&mut self) -> StallODis14W<StallOhDisSpec> {
        StallODis14W::new(self, 6)
    }
    #[doc = "Bit 7 - Disable Endpoint 15 OUT Direction"]
    #[inline(always)]
    pub fn stall_o_dis15(&mut self) -> StallODis15W<StallOhDisSpec> {
        StallODis15W::new(self, 7)
    }
}
#[doc = "Peripheral Mode Stall Disable for Endpoints 15 to 8 in OUT Direction\n\nYou can [`read`](crate::Reg::read) this register and get [`stall_oh_dis::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`stall_oh_dis::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct StallOhDisSpec;
impl crate::RegisterSpec for StallOhDisSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`stall_oh_dis::R`](R) reader structure"]
impl crate::Readable for StallOhDisSpec {}
#[doc = "`write(|w| ..)` method takes [`stall_oh_dis::W`](W) writer structure"]
impl crate::Writable for StallOhDisSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets STALL_OH_DIS to value 0"]
impl crate::Resettable for StallOhDisSpec {}

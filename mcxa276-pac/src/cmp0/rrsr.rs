#[doc = "Register `RRSR` reader"]
pub type R = crate::R<RrsrSpec>;
#[doc = "Register `RRSR` writer"]
pub type W = crate::W<RrsrSpec>;
#[doc = "Channel 0 Input Changed Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RrCh0f {
    #[doc = "0: No different"]
    NotDifferent = 0,
    #[doc = "1: Different"]
    Different = 1,
}
impl From<RrCh0f> for bool {
    #[inline(always)]
    fn from(variant: RrCh0f) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RR_CH0F` reader - Channel 0 Input Changed Flag"]
pub type RrCh0fR = crate::BitReader<RrCh0f>;
impl RrCh0fR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RrCh0f {
        match self.bits {
            false => RrCh0f::NotDifferent,
            true => RrCh0f::Different,
        }
    }
    #[doc = "No different"]
    #[inline(always)]
    pub fn is_not_different(&self) -> bool {
        *self == RrCh0f::NotDifferent
    }
    #[doc = "Different"]
    #[inline(always)]
    pub fn is_different(&self) -> bool {
        *self == RrCh0f::Different
    }
}
#[doc = "Field `RR_CH0F` writer - Channel 0 Input Changed Flag"]
pub type RrCh0fW<'a, REG> = crate::BitWriter1C<'a, REG, RrCh0f>;
impl<'a, REG> RrCh0fW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No different"]
    #[inline(always)]
    pub fn not_different(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh0f::NotDifferent)
    }
    #[doc = "Different"]
    #[inline(always)]
    pub fn different(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh0f::Different)
    }
}
#[doc = "Channel 1 Input Changed Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RrCh1f {
    #[doc = "0: No different"]
    NotDifferent = 0,
    #[doc = "1: Different"]
    Different = 1,
}
impl From<RrCh1f> for bool {
    #[inline(always)]
    fn from(variant: RrCh1f) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RR_CH1F` reader - Channel 1 Input Changed Flag"]
pub type RrCh1fR = crate::BitReader<RrCh1f>;
impl RrCh1fR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RrCh1f {
        match self.bits {
            false => RrCh1f::NotDifferent,
            true => RrCh1f::Different,
        }
    }
    #[doc = "No different"]
    #[inline(always)]
    pub fn is_not_different(&self) -> bool {
        *self == RrCh1f::NotDifferent
    }
    #[doc = "Different"]
    #[inline(always)]
    pub fn is_different(&self) -> bool {
        *self == RrCh1f::Different
    }
}
#[doc = "Field `RR_CH1F` writer - Channel 1 Input Changed Flag"]
pub type RrCh1fW<'a, REG> = crate::BitWriter1C<'a, REG, RrCh1f>;
impl<'a, REG> RrCh1fW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No different"]
    #[inline(always)]
    pub fn not_different(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh1f::NotDifferent)
    }
    #[doc = "Different"]
    #[inline(always)]
    pub fn different(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh1f::Different)
    }
}
#[doc = "Channel 2 Input Changed Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RrCh2f {
    #[doc = "0: No different"]
    NotDifferent = 0,
    #[doc = "1: Different"]
    Different = 1,
}
impl From<RrCh2f> for bool {
    #[inline(always)]
    fn from(variant: RrCh2f) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RR_CH2F` reader - Channel 2 Input Changed Flag"]
pub type RrCh2fR = crate::BitReader<RrCh2f>;
impl RrCh2fR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RrCh2f {
        match self.bits {
            false => RrCh2f::NotDifferent,
            true => RrCh2f::Different,
        }
    }
    #[doc = "No different"]
    #[inline(always)]
    pub fn is_not_different(&self) -> bool {
        *self == RrCh2f::NotDifferent
    }
    #[doc = "Different"]
    #[inline(always)]
    pub fn is_different(&self) -> bool {
        *self == RrCh2f::Different
    }
}
#[doc = "Field `RR_CH2F` writer - Channel 2 Input Changed Flag"]
pub type RrCh2fW<'a, REG> = crate::BitWriter1C<'a, REG, RrCh2f>;
impl<'a, REG> RrCh2fW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No different"]
    #[inline(always)]
    pub fn not_different(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh2f::NotDifferent)
    }
    #[doc = "Different"]
    #[inline(always)]
    pub fn different(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh2f::Different)
    }
}
#[doc = "Channel 3 Input Changed Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RrCh3f {
    #[doc = "0: No different"]
    NotDifferent = 0,
    #[doc = "1: Different"]
    Different = 1,
}
impl From<RrCh3f> for bool {
    #[inline(always)]
    fn from(variant: RrCh3f) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RR_CH3F` reader - Channel 3 Input Changed Flag"]
pub type RrCh3fR = crate::BitReader<RrCh3f>;
impl RrCh3fR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RrCh3f {
        match self.bits {
            false => RrCh3f::NotDifferent,
            true => RrCh3f::Different,
        }
    }
    #[doc = "No different"]
    #[inline(always)]
    pub fn is_not_different(&self) -> bool {
        *self == RrCh3f::NotDifferent
    }
    #[doc = "Different"]
    #[inline(always)]
    pub fn is_different(&self) -> bool {
        *self == RrCh3f::Different
    }
}
#[doc = "Field `RR_CH3F` writer - Channel 3 Input Changed Flag"]
pub type RrCh3fW<'a, REG> = crate::BitWriter1C<'a, REG, RrCh3f>;
impl<'a, REG> RrCh3fW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No different"]
    #[inline(always)]
    pub fn not_different(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh3f::NotDifferent)
    }
    #[doc = "Different"]
    #[inline(always)]
    pub fn different(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh3f::Different)
    }
}
#[doc = "Channel 4 Input Changed Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RrCh4f {
    #[doc = "0: No different"]
    NotDifferent = 0,
    #[doc = "1: Different"]
    Different = 1,
}
impl From<RrCh4f> for bool {
    #[inline(always)]
    fn from(variant: RrCh4f) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RR_CH4F` reader - Channel 4 Input Changed Flag"]
pub type RrCh4fR = crate::BitReader<RrCh4f>;
impl RrCh4fR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RrCh4f {
        match self.bits {
            false => RrCh4f::NotDifferent,
            true => RrCh4f::Different,
        }
    }
    #[doc = "No different"]
    #[inline(always)]
    pub fn is_not_different(&self) -> bool {
        *self == RrCh4f::NotDifferent
    }
    #[doc = "Different"]
    #[inline(always)]
    pub fn is_different(&self) -> bool {
        *self == RrCh4f::Different
    }
}
#[doc = "Field `RR_CH4F` writer - Channel 4 Input Changed Flag"]
pub type RrCh4fW<'a, REG> = crate::BitWriter1C<'a, REG, RrCh4f>;
impl<'a, REG> RrCh4fW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No different"]
    #[inline(always)]
    pub fn not_different(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh4f::NotDifferent)
    }
    #[doc = "Different"]
    #[inline(always)]
    pub fn different(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh4f::Different)
    }
}
#[doc = "Channel 5 Input Changed Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RrCh5f {
    #[doc = "0: No different"]
    NotDifferent = 0,
    #[doc = "1: Different"]
    Different = 1,
}
impl From<RrCh5f> for bool {
    #[inline(always)]
    fn from(variant: RrCh5f) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RR_CH5F` reader - Channel 5 Input Changed Flag"]
pub type RrCh5fR = crate::BitReader<RrCh5f>;
impl RrCh5fR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RrCh5f {
        match self.bits {
            false => RrCh5f::NotDifferent,
            true => RrCh5f::Different,
        }
    }
    #[doc = "No different"]
    #[inline(always)]
    pub fn is_not_different(&self) -> bool {
        *self == RrCh5f::NotDifferent
    }
    #[doc = "Different"]
    #[inline(always)]
    pub fn is_different(&self) -> bool {
        *self == RrCh5f::Different
    }
}
#[doc = "Field `RR_CH5F` writer - Channel 5 Input Changed Flag"]
pub type RrCh5fW<'a, REG> = crate::BitWriter1C<'a, REG, RrCh5f>;
impl<'a, REG> RrCh5fW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No different"]
    #[inline(always)]
    pub fn not_different(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh5f::NotDifferent)
    }
    #[doc = "Different"]
    #[inline(always)]
    pub fn different(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh5f::Different)
    }
}
#[doc = "Channel 6 Input Changed Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RrCh6f {
    #[doc = "0: No different"]
    NotDifferent = 0,
    #[doc = "1: Different"]
    Different = 1,
}
impl From<RrCh6f> for bool {
    #[inline(always)]
    fn from(variant: RrCh6f) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RR_CH6F` reader - Channel 6 Input Changed Flag"]
pub type RrCh6fR = crate::BitReader<RrCh6f>;
impl RrCh6fR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RrCh6f {
        match self.bits {
            false => RrCh6f::NotDifferent,
            true => RrCh6f::Different,
        }
    }
    #[doc = "No different"]
    #[inline(always)]
    pub fn is_not_different(&self) -> bool {
        *self == RrCh6f::NotDifferent
    }
    #[doc = "Different"]
    #[inline(always)]
    pub fn is_different(&self) -> bool {
        *self == RrCh6f::Different
    }
}
#[doc = "Field `RR_CH6F` writer - Channel 6 Input Changed Flag"]
pub type RrCh6fW<'a, REG> = crate::BitWriter1C<'a, REG, RrCh6f>;
impl<'a, REG> RrCh6fW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No different"]
    #[inline(always)]
    pub fn not_different(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh6f::NotDifferent)
    }
    #[doc = "Different"]
    #[inline(always)]
    pub fn different(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh6f::Different)
    }
}
#[doc = "Channel 7 Input Changed Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RrCh7f {
    #[doc = "0: No different"]
    NotDifferent = 0,
    #[doc = "1: Different"]
    Different = 1,
}
impl From<RrCh7f> for bool {
    #[inline(always)]
    fn from(variant: RrCh7f) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RR_CH7F` reader - Channel 7 Input Changed Flag"]
pub type RrCh7fR = crate::BitReader<RrCh7f>;
impl RrCh7fR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RrCh7f {
        match self.bits {
            false => RrCh7f::NotDifferent,
            true => RrCh7f::Different,
        }
    }
    #[doc = "No different"]
    #[inline(always)]
    pub fn is_not_different(&self) -> bool {
        *self == RrCh7f::NotDifferent
    }
    #[doc = "Different"]
    #[inline(always)]
    pub fn is_different(&self) -> bool {
        *self == RrCh7f::Different
    }
}
#[doc = "Field `RR_CH7F` writer - Channel 7 Input Changed Flag"]
pub type RrCh7fW<'a, REG> = crate::BitWriter1C<'a, REG, RrCh7f>;
impl<'a, REG> RrCh7fW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No different"]
    #[inline(always)]
    pub fn not_different(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh7f::NotDifferent)
    }
    #[doc = "Different"]
    #[inline(always)]
    pub fn different(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh7f::Different)
    }
}
impl R {
    #[doc = "Bit 0 - Channel 0 Input Changed Flag"]
    #[inline(always)]
    pub fn rr_ch0f(&self) -> RrCh0fR {
        RrCh0fR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Channel 1 Input Changed Flag"]
    #[inline(always)]
    pub fn rr_ch1f(&self) -> RrCh1fR {
        RrCh1fR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Channel 2 Input Changed Flag"]
    #[inline(always)]
    pub fn rr_ch2f(&self) -> RrCh2fR {
        RrCh2fR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Channel 3 Input Changed Flag"]
    #[inline(always)]
    pub fn rr_ch3f(&self) -> RrCh3fR {
        RrCh3fR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Channel 4 Input Changed Flag"]
    #[inline(always)]
    pub fn rr_ch4f(&self) -> RrCh4fR {
        RrCh4fR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Channel 5 Input Changed Flag"]
    #[inline(always)]
    pub fn rr_ch5f(&self) -> RrCh5fR {
        RrCh5fR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Channel 6 Input Changed Flag"]
    #[inline(always)]
    pub fn rr_ch6f(&self) -> RrCh6fR {
        RrCh6fR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Channel 7 Input Changed Flag"]
    #[inline(always)]
    pub fn rr_ch7f(&self) -> RrCh7fR {
        RrCh7fR::new(((self.bits >> 7) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Channel 0 Input Changed Flag"]
    #[inline(always)]
    pub fn rr_ch0f(&mut self) -> RrCh0fW<RrsrSpec> {
        RrCh0fW::new(self, 0)
    }
    #[doc = "Bit 1 - Channel 1 Input Changed Flag"]
    #[inline(always)]
    pub fn rr_ch1f(&mut self) -> RrCh1fW<RrsrSpec> {
        RrCh1fW::new(self, 1)
    }
    #[doc = "Bit 2 - Channel 2 Input Changed Flag"]
    #[inline(always)]
    pub fn rr_ch2f(&mut self) -> RrCh2fW<RrsrSpec> {
        RrCh2fW::new(self, 2)
    }
    #[doc = "Bit 3 - Channel 3 Input Changed Flag"]
    #[inline(always)]
    pub fn rr_ch3f(&mut self) -> RrCh3fW<RrsrSpec> {
        RrCh3fW::new(self, 3)
    }
    #[doc = "Bit 4 - Channel 4 Input Changed Flag"]
    #[inline(always)]
    pub fn rr_ch4f(&mut self) -> RrCh4fW<RrsrSpec> {
        RrCh4fW::new(self, 4)
    }
    #[doc = "Bit 5 - Channel 5 Input Changed Flag"]
    #[inline(always)]
    pub fn rr_ch5f(&mut self) -> RrCh5fW<RrsrSpec> {
        RrCh5fW::new(self, 5)
    }
    #[doc = "Bit 6 - Channel 6 Input Changed Flag"]
    #[inline(always)]
    pub fn rr_ch6f(&mut self) -> RrCh6fW<RrsrSpec> {
        RrCh6fW::new(self, 6)
    }
    #[doc = "Bit 7 - Channel 7 Input Changed Flag"]
    #[inline(always)]
    pub fn rr_ch7f(&mut self) -> RrCh7fW<RrsrSpec> {
        RrCh7fW::new(self, 7)
    }
}
#[doc = "Round Robin Status\n\nYou can [`read`](crate::Reg::read) this register and get [`rrsr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rrsr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RrsrSpec;
impl crate::RegisterSpec for RrsrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`rrsr::R`](R) reader structure"]
impl crate::Readable for RrsrSpec {}
#[doc = "`write(|w| ..)` method takes [`rrsr::W`](W) writer structure"]
impl crate::Writable for RrsrSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0xff;
}
#[doc = "`reset()` method sets RRSR to value 0"]
impl crate::Resettable for RrsrSpec {}

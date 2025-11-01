#[doc = "Register `SM0STS` reader"]
pub type R = crate::R<Sm0stsSpec>;
#[doc = "Register `SM0STS` writer"]
pub type W = crate::W<Sm0stsSpec>;
#[doc = "Compare Flags\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Cmpf {
    #[doc = "0: No compare event has occurred for a particular VALx value."]
    NoEvent = 0,
    #[doc = "1: A compare event has occurred for a particular VALx value."]
    Event = 1,
}
impl From<Cmpf> for u8 {
    #[inline(always)]
    fn from(variant: Cmpf) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Cmpf {
    type Ux = u8;
}
impl crate::IsEnum for Cmpf {}
#[doc = "Field `CMPF` reader - Compare Flags"]
pub type CmpfR = crate::FieldReader<Cmpf>;
impl CmpfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Cmpf> {
        match self.bits {
            0 => Some(Cmpf::NoEvent),
            1 => Some(Cmpf::Event),
            _ => None,
        }
    }
    #[doc = "No compare event has occurred for a particular VALx value."]
    #[inline(always)]
    pub fn is_no_event(&self) -> bool {
        *self == Cmpf::NoEvent
    }
    #[doc = "A compare event has occurred for a particular VALx value."]
    #[inline(always)]
    pub fn is_event(&self) -> bool {
        *self == Cmpf::Event
    }
}
#[doc = "Field `CMPF` writer - Compare Flags"]
pub type CmpfW<'a, REG> = crate::FieldWriter<'a, REG, 6, Cmpf>;
impl<'a, REG> CmpfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "No compare event has occurred for a particular VALx value."]
    #[inline(always)]
    pub fn no_event(self) -> &'a mut crate::W<REG> {
        self.variant(Cmpf::NoEvent)
    }
    #[doc = "A compare event has occurred for a particular VALx value."]
    #[inline(always)]
    pub fn event(self) -> &'a mut crate::W<REG> {
        self.variant(Cmpf::Event)
    }
}
#[doc = "Field `CFX0` reader - Capture Flag X0"]
pub type Cfx0R = crate::BitReader;
#[doc = "Field `CFX0` writer - Capture Flag X0"]
pub type Cfx0W<'a, REG> = crate::BitWriter1C<'a, REG>;
#[doc = "Field `CFX1` reader - Capture Flag X1"]
pub type Cfx1R = crate::BitReader;
#[doc = "Field `CFX1` writer - Capture Flag X1"]
pub type Cfx1W<'a, REG> = crate::BitWriter1C<'a, REG>;
#[doc = "Reload Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rf {
    #[doc = "0: No new reload cycle since last STS\\[RF\\] clearing"]
    NoFlag = 0,
    #[doc = "1: New reload cycle since last STS\\[RF\\] clearing"]
    Flag = 1,
}
impl From<Rf> for bool {
    #[inline(always)]
    fn from(variant: Rf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RF` reader - Reload Flag"]
pub type RfR = crate::BitReader<Rf>;
impl RfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rf {
        match self.bits {
            false => Rf::NoFlag,
            true => Rf::Flag,
        }
    }
    #[doc = "No new reload cycle since last STS\\[RF\\] clearing"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Rf::NoFlag
    }
    #[doc = "New reload cycle since last STS\\[RF\\] clearing"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Rf::Flag
    }
}
#[doc = "Field `RF` writer - Reload Flag"]
pub type RfW<'a, REG> = crate::BitWriter1C<'a, REG, Rf>;
impl<'a, REG> RfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No new reload cycle since last STS\\[RF\\] clearing"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Rf::NoFlag)
    }
    #[doc = "New reload cycle since last STS\\[RF\\] clearing"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Rf::Flag)
    }
}
#[doc = "Reload Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ref {
    #[doc = "0: No reload error occurred."]
    NoFlag = 0,
    #[doc = "1: Reload signal occurred with non-coherent data and MCTRL\\[LDOK\\] = 0."]
    Flag = 1,
}
impl From<Ref> for bool {
    #[inline(always)]
    fn from(variant: Ref) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `REF` reader - Reload Error Flag"]
pub type RefR = crate::BitReader<Ref>;
impl RefR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ref {
        match self.bits {
            false => Ref::NoFlag,
            true => Ref::Flag,
        }
    }
    #[doc = "No reload error occurred."]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Ref::NoFlag
    }
    #[doc = "Reload signal occurred with non-coherent data and MCTRL\\[LDOK\\] = 0."]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Ref::Flag
    }
}
#[doc = "Field `REF` writer - Reload Error Flag"]
pub type RefW<'a, REG> = crate::BitWriter1C<'a, REG, Ref>;
impl<'a, REG> RefW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No reload error occurred."]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Ref::NoFlag)
    }
    #[doc = "Reload signal occurred with non-coherent data and MCTRL\\[LDOK\\] = 0."]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Ref::Flag)
    }
}
#[doc = "Registers Updated Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ruf {
    #[doc = "0: No register update has occurred since last reload."]
    NoFlag = 0,
    #[doc = "1: At least one of the double buffered registers has been updated since the last reload."]
    Flag = 1,
}
impl From<Ruf> for bool {
    #[inline(always)]
    fn from(variant: Ruf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RUF` reader - Registers Updated Flag"]
pub type RufR = crate::BitReader<Ruf>;
impl RufR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ruf {
        match self.bits {
            false => Ruf::NoFlag,
            true => Ruf::Flag,
        }
    }
    #[doc = "No register update has occurred since last reload."]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Ruf::NoFlag
    }
    #[doc = "At least one of the double buffered registers has been updated since the last reload."]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Ruf::Flag
    }
}
impl R {
    #[doc = "Bits 0:5 - Compare Flags"]
    #[inline(always)]
    pub fn cmpf(&self) -> CmpfR {
        CmpfR::new((self.bits & 0x3f) as u8)
    }
    #[doc = "Bit 6 - Capture Flag X0"]
    #[inline(always)]
    pub fn cfx0(&self) -> Cfx0R {
        Cfx0R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Capture Flag X1"]
    #[inline(always)]
    pub fn cfx1(&self) -> Cfx1R {
        Cfx1R::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 12 - Reload Flag"]
    #[inline(always)]
    pub fn rf(&self) -> RfR {
        RfR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Reload Error Flag"]
    #[inline(always)]
    pub fn ref_(&self) -> RefR {
        RefR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Registers Updated Flag"]
    #[inline(always)]
    pub fn ruf(&self) -> RufR {
        RufR::new(((self.bits >> 14) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:5 - Compare Flags"]
    #[inline(always)]
    pub fn cmpf(&mut self) -> CmpfW<Sm0stsSpec> {
        CmpfW::new(self, 0)
    }
    #[doc = "Bit 6 - Capture Flag X0"]
    #[inline(always)]
    pub fn cfx0(&mut self) -> Cfx0W<Sm0stsSpec> {
        Cfx0W::new(self, 6)
    }
    #[doc = "Bit 7 - Capture Flag X1"]
    #[inline(always)]
    pub fn cfx1(&mut self) -> Cfx1W<Sm0stsSpec> {
        Cfx1W::new(self, 7)
    }
    #[doc = "Bit 12 - Reload Flag"]
    #[inline(always)]
    pub fn rf(&mut self) -> RfW<Sm0stsSpec> {
        RfW::new(self, 12)
    }
    #[doc = "Bit 13 - Reload Error Flag"]
    #[inline(always)]
    pub fn ref_(&mut self) -> RefW<Sm0stsSpec> {
        RefW::new(self, 13)
    }
}
#[doc = "Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0sts::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm0sts::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm0stsSpec;
impl crate::RegisterSpec for Sm0stsSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm0sts::R`](R) reader structure"]
impl crate::Readable for Sm0stsSpec {}
#[doc = "`write(|w| ..)` method takes [`sm0sts::W`](W) writer structure"]
impl crate::Writable for Sm0stsSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u16 = 0x30ff;
}
#[doc = "`reset()` method sets SM0STS to value 0"]
impl crate::Resettable for Sm0stsSpec {}

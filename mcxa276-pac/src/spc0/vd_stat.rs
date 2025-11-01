#[doc = "Register `VD_STAT` reader"]
pub type R = crate::R<VdStatSpec>;
#[doc = "Register `VD_STAT` writer"]
pub type W = crate::W<VdStatSpec>;
#[doc = "Core Low-Voltage Detect Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorevddLvdf {
    #[doc = "0: Event not detected"]
    EventNo = 0,
    #[doc = "1: Event detected"]
    EventYes = 1,
}
impl From<CorevddLvdf> for bool {
    #[inline(always)]
    fn from(variant: CorevddLvdf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `COREVDD_LVDF` reader - Core Low-Voltage Detect Flag"]
pub type CorevddLvdfR = crate::BitReader<CorevddLvdf>;
impl CorevddLvdfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> CorevddLvdf {
        match self.bits {
            false => CorevddLvdf::EventNo,
            true => CorevddLvdf::EventYes,
        }
    }
    #[doc = "Event not detected"]
    #[inline(always)]
    pub fn is_event_no(&self) -> bool {
        *self == CorevddLvdf::EventNo
    }
    #[doc = "Event detected"]
    #[inline(always)]
    pub fn is_event_yes(&self) -> bool {
        *self == CorevddLvdf::EventYes
    }
}
#[doc = "Field `COREVDD_LVDF` writer - Core Low-Voltage Detect Flag"]
pub type CorevddLvdfW<'a, REG> = crate::BitWriter1C<'a, REG, CorevddLvdf>;
impl<'a, REG> CorevddLvdfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Event not detected"]
    #[inline(always)]
    pub fn event_no(self) -> &'a mut crate::W<REG> {
        self.variant(CorevddLvdf::EventNo)
    }
    #[doc = "Event detected"]
    #[inline(always)]
    pub fn event_yes(self) -> &'a mut crate::W<REG> {
        self.variant(CorevddLvdf::EventYes)
    }
}
#[doc = "System Low-Voltage Detect Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SysvddLvdf {
    #[doc = "0: Event not detected"]
    EventNo = 0,
    #[doc = "1: Event detected"]
    EventYes = 1,
}
impl From<SysvddLvdf> for bool {
    #[inline(always)]
    fn from(variant: SysvddLvdf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SYSVDD_LVDF` reader - System Low-Voltage Detect Flag"]
pub type SysvddLvdfR = crate::BitReader<SysvddLvdf>;
impl SysvddLvdfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> SysvddLvdf {
        match self.bits {
            false => SysvddLvdf::EventNo,
            true => SysvddLvdf::EventYes,
        }
    }
    #[doc = "Event not detected"]
    #[inline(always)]
    pub fn is_event_no(&self) -> bool {
        *self == SysvddLvdf::EventNo
    }
    #[doc = "Event detected"]
    #[inline(always)]
    pub fn is_event_yes(&self) -> bool {
        *self == SysvddLvdf::EventYes
    }
}
#[doc = "Field `SYSVDD_LVDF` writer - System Low-Voltage Detect Flag"]
pub type SysvddLvdfW<'a, REG> = crate::BitWriter1C<'a, REG, SysvddLvdf>;
impl<'a, REG> SysvddLvdfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Event not detected"]
    #[inline(always)]
    pub fn event_no(self) -> &'a mut crate::W<REG> {
        self.variant(SysvddLvdf::EventNo)
    }
    #[doc = "Event detected"]
    #[inline(always)]
    pub fn event_yes(self) -> &'a mut crate::W<REG> {
        self.variant(SysvddLvdf::EventYes)
    }
}
#[doc = "System HVD Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SysvddHvdf {
    #[doc = "0: Event not detected"]
    EventNo = 0,
    #[doc = "1: Event detected"]
    EventYes = 1,
}
impl From<SysvddHvdf> for bool {
    #[inline(always)]
    fn from(variant: SysvddHvdf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SYSVDD_HVDF` reader - System HVD Flag"]
pub type SysvddHvdfR = crate::BitReader<SysvddHvdf>;
impl SysvddHvdfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> SysvddHvdf {
        match self.bits {
            false => SysvddHvdf::EventNo,
            true => SysvddHvdf::EventYes,
        }
    }
    #[doc = "Event not detected"]
    #[inline(always)]
    pub fn is_event_no(&self) -> bool {
        *self == SysvddHvdf::EventNo
    }
    #[doc = "Event detected"]
    #[inline(always)]
    pub fn is_event_yes(&self) -> bool {
        *self == SysvddHvdf::EventYes
    }
}
#[doc = "Field `SYSVDD_HVDF` writer - System HVD Flag"]
pub type SysvddHvdfW<'a, REG> = crate::BitWriter1C<'a, REG, SysvddHvdf>;
impl<'a, REG> SysvddHvdfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Event not detected"]
    #[inline(always)]
    pub fn event_no(self) -> &'a mut crate::W<REG> {
        self.variant(SysvddHvdf::EventNo)
    }
    #[doc = "Event detected"]
    #[inline(always)]
    pub fn event_yes(self) -> &'a mut crate::W<REG> {
        self.variant(SysvddHvdf::EventYes)
    }
}
impl R {
    #[doc = "Bit 0 - Core Low-Voltage Detect Flag"]
    #[inline(always)]
    pub fn corevdd_lvdf(&self) -> CorevddLvdfR {
        CorevddLvdfR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - System Low-Voltage Detect Flag"]
    #[inline(always)]
    pub fn sysvdd_lvdf(&self) -> SysvddLvdfR {
        SysvddLvdfR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 5 - System HVD Flag"]
    #[inline(always)]
    pub fn sysvdd_hvdf(&self) -> SysvddHvdfR {
        SysvddHvdfR::new(((self.bits >> 5) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Core Low-Voltage Detect Flag"]
    #[inline(always)]
    pub fn corevdd_lvdf(&mut self) -> CorevddLvdfW<VdStatSpec> {
        CorevddLvdfW::new(self, 0)
    }
    #[doc = "Bit 1 - System Low-Voltage Detect Flag"]
    #[inline(always)]
    pub fn sysvdd_lvdf(&mut self) -> SysvddLvdfW<VdStatSpec> {
        SysvddLvdfW::new(self, 1)
    }
    #[doc = "Bit 5 - System HVD Flag"]
    #[inline(always)]
    pub fn sysvdd_hvdf(&mut self) -> SysvddHvdfW<VdStatSpec> {
        SysvddHvdfW::new(self, 5)
    }
}
#[doc = "Voltage Detect Status\n\nYou can [`read`](crate::Reg::read) this register and get [`vd_stat::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`vd_stat::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct VdStatSpec;
impl crate::RegisterSpec for VdStatSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`vd_stat::R`](R) reader structure"]
impl crate::Readable for VdStatSpec {}
#[doc = "`write(|w| ..)` method takes [`vd_stat::W`](W) writer structure"]
impl crate::Writable for VdStatSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x23;
}
#[doc = "`reset()` method sets VD_STAT to value 0"]
impl crate::Resettable for VdStatSpec {}

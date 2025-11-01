#[doc = "Register `WU_MTC` reader"]
pub type R = crate::R<WuMtcSpec>;
#[doc = "Register `WU_MTC` writer"]
pub type W = crate::W<WuMtcSpec>;
#[doc = "Field `MCOUNTER` reader - Number of Matches in Pretended Networking"]
pub type McounterR = crate::FieldReader;
#[doc = "Wake-up by Match Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wumf {
    #[doc = "0: No event detected"]
    NoMatch = 0,
    #[doc = "1: Event detected"]
    Match = 1,
}
impl From<Wumf> for bool {
    #[inline(always)]
    fn from(variant: Wumf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WUMF` reader - Wake-up by Match Flag"]
pub type WumfR = crate::BitReader<Wumf>;
impl WumfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wumf {
        match self.bits {
            false => Wumf::NoMatch,
            true => Wumf::Match,
        }
    }
    #[doc = "No event detected"]
    #[inline(always)]
    pub fn is_no_match(&self) -> bool {
        *self == Wumf::NoMatch
    }
    #[doc = "Event detected"]
    #[inline(always)]
    pub fn is_match(&self) -> bool {
        *self == Wumf::Match
    }
}
#[doc = "Field `WUMF` writer - Wake-up by Match Flag"]
pub type WumfW<'a, REG> = crate::BitWriter1C<'a, REG, Wumf>;
impl<'a, REG> WumfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No event detected"]
    #[inline(always)]
    pub fn no_match(self) -> &'a mut crate::W<REG> {
        self.variant(Wumf::NoMatch)
    }
    #[doc = "Event detected"]
    #[inline(always)]
    pub fn match_(self) -> &'a mut crate::W<REG> {
        self.variant(Wumf::Match)
    }
}
#[doc = "Wake-up by Timeout Flag Bit\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wtof {
    #[doc = "0: No event detected"]
    NoWakeup = 0,
    #[doc = "1: Event detected"]
    Wakeup = 1,
}
impl From<Wtof> for bool {
    #[inline(always)]
    fn from(variant: Wtof) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WTOF` reader - Wake-up by Timeout Flag Bit"]
pub type WtofR = crate::BitReader<Wtof>;
impl WtofR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wtof {
        match self.bits {
            false => Wtof::NoWakeup,
            true => Wtof::Wakeup,
        }
    }
    #[doc = "No event detected"]
    #[inline(always)]
    pub fn is_no_wakeup(&self) -> bool {
        *self == Wtof::NoWakeup
    }
    #[doc = "Event detected"]
    #[inline(always)]
    pub fn is_wakeup(&self) -> bool {
        *self == Wtof::Wakeup
    }
}
#[doc = "Field `WTOF` writer - Wake-up by Timeout Flag Bit"]
pub type WtofW<'a, REG> = crate::BitWriter1C<'a, REG, Wtof>;
impl<'a, REG> WtofW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No event detected"]
    #[inline(always)]
    pub fn no_wakeup(self) -> &'a mut crate::W<REG> {
        self.variant(Wtof::NoWakeup)
    }
    #[doc = "Event detected"]
    #[inline(always)]
    pub fn wakeup(self) -> &'a mut crate::W<REG> {
        self.variant(Wtof::Wakeup)
    }
}
impl R {
    #[doc = "Bits 8:15 - Number of Matches in Pretended Networking"]
    #[inline(always)]
    pub fn mcounter(&self) -> McounterR {
        McounterR::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bit 16 - Wake-up by Match Flag"]
    #[inline(always)]
    pub fn wumf(&self) -> WumfR {
        WumfR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Wake-up by Timeout Flag Bit"]
    #[inline(always)]
    pub fn wtof(&self) -> WtofR {
        WtofR::new(((self.bits >> 17) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 16 - Wake-up by Match Flag"]
    #[inline(always)]
    pub fn wumf(&mut self) -> WumfW<WuMtcSpec> {
        WumfW::new(self, 16)
    }
    #[doc = "Bit 17 - Wake-up by Timeout Flag Bit"]
    #[inline(always)]
    pub fn wtof(&mut self) -> WtofW<WuMtcSpec> {
        WtofW::new(self, 17)
    }
}
#[doc = "Pretended Networking Wake-Up Match\n\nYou can [`read`](crate::Reg::read) this register and get [`wu_mtc::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`wu_mtc::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct WuMtcSpec;
impl crate::RegisterSpec for WuMtcSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`wu_mtc::R`](R) reader structure"]
impl crate::Readable for WuMtcSpec {}
#[doc = "`write(|w| ..)` method takes [`wu_mtc::W`](W) writer structure"]
impl crate::Writable for WuMtcSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x0003_0000;
}
#[doc = "`reset()` method sets WU_MTC to value 0"]
impl crate::Resettable for WuMtcSpec {}

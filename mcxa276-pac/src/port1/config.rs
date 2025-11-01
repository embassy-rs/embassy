#[doc = "Register `CONFIG` reader"]
pub type R = crate::R<ConfigSpec>;
#[doc = "Register `CONFIG` writer"]
pub type W = crate::W<ConfigSpec>;
#[doc = "Port Voltage Range\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Range {
    #[doc = "0: 1.71 V-3.6 V"]
    Range0 = 0,
    #[doc = "1: 2.70 V-3.6 V"]
    Range1 = 1,
}
impl From<Range> for bool {
    #[inline(always)]
    fn from(variant: Range) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RANGE` reader - Port Voltage Range"]
pub type RangeR = crate::BitReader<Range>;
impl RangeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Range {
        match self.bits {
            false => Range::Range0,
            true => Range::Range1,
        }
    }
    #[doc = "1.71 V-3.6 V"]
    #[inline(always)]
    pub fn is_range0(&self) -> bool {
        *self == Range::Range0
    }
    #[doc = "2.70 V-3.6 V"]
    #[inline(always)]
    pub fn is_range1(&self) -> bool {
        *self == Range::Range1
    }
}
#[doc = "Field `RANGE` writer - Port Voltage Range"]
pub type RangeW<'a, REG> = crate::BitWriter<'a, REG, Range>;
impl<'a, REG> RangeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "1.71 V-3.6 V"]
    #[inline(always)]
    pub fn range0(self) -> &'a mut crate::W<REG> {
        self.variant(Range::Range0)
    }
    #[doc = "2.70 V-3.6 V"]
    #[inline(always)]
    pub fn range1(self) -> &'a mut crate::W<REG> {
        self.variant(Range::Range1)
    }
}
impl R {
    #[doc = "Bit 0 - Port Voltage Range"]
    #[inline(always)]
    pub fn range(&self) -> RangeR {
        RangeR::new((self.bits & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Port Voltage Range"]
    #[inline(always)]
    pub fn range(&mut self) -> RangeW<ConfigSpec> {
        RangeW::new(self, 0)
    }
}
#[doc = "Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`config::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`config::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ConfigSpec;
impl crate::RegisterSpec for ConfigSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`config::R`](R) reader structure"]
impl crate::Readable for ConfigSpec {}
#[doc = "`write(|w| ..)` method takes [`config::W`](W) writer structure"]
impl crate::Writable for ConfigSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CONFIG to value 0"]
impl crate::Resettable for ConfigSpec {}

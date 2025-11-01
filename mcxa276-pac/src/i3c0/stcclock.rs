#[doc = "Register `STCCLOCK` reader"]
pub type R = crate::R<StcclockSpec>;
#[doc = "Register `STCCLOCK` writer"]
pub type W = crate::W<StcclockSpec>;
#[doc = "Field `ACCURACY` reader - Clock Accuracy"]
pub type AccuracyR = crate::FieldReader;
#[doc = "Field `ACCURACY` writer - Clock Accuracy"]
pub type AccuracyW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `FREQ` reader - Clock Frequency"]
pub type FreqR = crate::FieldReader;
#[doc = "Field `FREQ` writer - Clock Frequency"]
pub type FreqW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Clock Accuracy"]
    #[inline(always)]
    pub fn accuracy(&self) -> AccuracyR {
        AccuracyR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Clock Frequency"]
    #[inline(always)]
    pub fn freq(&self) -> FreqR {
        FreqR::new(((self.bits >> 8) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Clock Accuracy"]
    #[inline(always)]
    pub fn accuracy(&mut self) -> AccuracyW<StcclockSpec> {
        AccuracyW::new(self, 0)
    }
    #[doc = "Bits 8:15 - Clock Frequency"]
    #[inline(always)]
    pub fn freq(&mut self) -> FreqW<StcclockSpec> {
        FreqW::new(self, 8)
    }
}
#[doc = "Target Time Control Clock\n\nYou can [`read`](crate::Reg::read) this register and get [`stcclock::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`stcclock::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct StcclockSpec;
impl crate::RegisterSpec for StcclockSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`stcclock::R`](R) reader structure"]
impl crate::Readable for StcclockSpec {}
#[doc = "`write(|w| ..)` method takes [`stcclock::W`](W) writer structure"]
impl crate::Writable for StcclockSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets STCCLOCK to value 0x3014"]
impl crate::Resettable for StcclockSpec {
    const RESET_VALUE: u32 = 0x3014;
}

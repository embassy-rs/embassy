#[doc = "Register `FDCBT` reader"]
pub type R = crate::R<FdcbtSpec>;
#[doc = "Register `FDCBT` writer"]
pub type W = crate::W<FdcbtSpec>;
#[doc = "Field `FPSEG2` reader - Fast Phase Segment 2"]
pub type Fpseg2R = crate::FieldReader;
#[doc = "Field `FPSEG2` writer - Fast Phase Segment 2"]
pub type Fpseg2W<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `FPSEG1` reader - Fast Phase Segment 1"]
pub type Fpseg1R = crate::FieldReader;
#[doc = "Field `FPSEG1` writer - Fast Phase Segment 1"]
pub type Fpseg1W<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `FPROPSEG` reader - Fast Propagation Segment"]
pub type FpropsegR = crate::FieldReader;
#[doc = "Field `FPROPSEG` writer - Fast Propagation Segment"]
pub type FpropsegW<'a, REG> = crate::FieldWriter<'a, REG, 5>;
#[doc = "Field `FRJW` reader - Fast Resync Jump Width"]
pub type FrjwR = crate::FieldReader;
#[doc = "Field `FRJW` writer - Fast Resync Jump Width"]
pub type FrjwW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `FPRESDIV` reader - Fast Prescaler Division Factor"]
pub type FpresdivR = crate::FieldReader<u16>;
#[doc = "Field `FPRESDIV` writer - Fast Prescaler Division Factor"]
pub type FpresdivW<'a, REG> = crate::FieldWriter<'a, REG, 10, u16>;
impl R {
    #[doc = "Bits 0:2 - Fast Phase Segment 2"]
    #[inline(always)]
    pub fn fpseg2(&self) -> Fpseg2R {
        Fpseg2R::new((self.bits & 7) as u8)
    }
    #[doc = "Bits 5:7 - Fast Phase Segment 1"]
    #[inline(always)]
    pub fn fpseg1(&self) -> Fpseg1R {
        Fpseg1R::new(((self.bits >> 5) & 7) as u8)
    }
    #[doc = "Bits 10:14 - Fast Propagation Segment"]
    #[inline(always)]
    pub fn fpropseg(&self) -> FpropsegR {
        FpropsegR::new(((self.bits >> 10) & 0x1f) as u8)
    }
    #[doc = "Bits 16:18 - Fast Resync Jump Width"]
    #[inline(always)]
    pub fn frjw(&self) -> FrjwR {
        FrjwR::new(((self.bits >> 16) & 7) as u8)
    }
    #[doc = "Bits 20:29 - Fast Prescaler Division Factor"]
    #[inline(always)]
    pub fn fpresdiv(&self) -> FpresdivR {
        FpresdivR::new(((self.bits >> 20) & 0x03ff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:2 - Fast Phase Segment 2"]
    #[inline(always)]
    pub fn fpseg2(&mut self) -> Fpseg2W<FdcbtSpec> {
        Fpseg2W::new(self, 0)
    }
    #[doc = "Bits 5:7 - Fast Phase Segment 1"]
    #[inline(always)]
    pub fn fpseg1(&mut self) -> Fpseg1W<FdcbtSpec> {
        Fpseg1W::new(self, 5)
    }
    #[doc = "Bits 10:14 - Fast Propagation Segment"]
    #[inline(always)]
    pub fn fpropseg(&mut self) -> FpropsegW<FdcbtSpec> {
        FpropsegW::new(self, 10)
    }
    #[doc = "Bits 16:18 - Fast Resync Jump Width"]
    #[inline(always)]
    pub fn frjw(&mut self) -> FrjwW<FdcbtSpec> {
        FrjwW::new(self, 16)
    }
    #[doc = "Bits 20:29 - Fast Prescaler Division Factor"]
    #[inline(always)]
    pub fn fpresdiv(&mut self) -> FpresdivW<FdcbtSpec> {
        FpresdivW::new(self, 20)
    }
}
#[doc = "CAN FD Bit Timing\n\nYou can [`read`](crate::Reg::read) this register and get [`fdcbt::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fdcbt::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FdcbtSpec;
impl crate::RegisterSpec for FdcbtSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`fdcbt::R`](R) reader structure"]
impl crate::Readable for FdcbtSpec {}
#[doc = "`write(|w| ..)` method takes [`fdcbt::W`](W) writer structure"]
impl crate::Writable for FdcbtSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FDCBT to value 0"]
impl crate::Resettable for FdcbtSpec {}

#[doc = "Register `EDCBT` reader"]
pub type R = crate::R<EdcbtSpec>;
#[doc = "Register `EDCBT` writer"]
pub type W = crate::W<EdcbtSpec>;
#[doc = "Field `DTSEG1` reader - Data Phase Segment 1"]
pub type Dtseg1R = crate::FieldReader;
#[doc = "Field `DTSEG1` writer - Data Phase Segment 1"]
pub type Dtseg1W<'a, REG> = crate::FieldWriter<'a, REG, 5>;
#[doc = "Field `DTSEG2` reader - Data Phase Time Segment 2"]
pub type Dtseg2R = crate::FieldReader;
#[doc = "Field `DTSEG2` writer - Data Phase Time Segment 2"]
pub type Dtseg2W<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `DRJW` reader - Data Phase Resynchronization Jump Width"]
pub type DrjwR = crate::FieldReader;
#[doc = "Field `DRJW` writer - Data Phase Resynchronization Jump Width"]
pub type DrjwW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:4 - Data Phase Segment 1"]
    #[inline(always)]
    pub fn dtseg1(&self) -> Dtseg1R {
        Dtseg1R::new((self.bits & 0x1f) as u8)
    }
    #[doc = "Bits 12:15 - Data Phase Time Segment 2"]
    #[inline(always)]
    pub fn dtseg2(&self) -> Dtseg2R {
        Dtseg2R::new(((self.bits >> 12) & 0x0f) as u8)
    }
    #[doc = "Bits 22:25 - Data Phase Resynchronization Jump Width"]
    #[inline(always)]
    pub fn drjw(&self) -> DrjwR {
        DrjwR::new(((self.bits >> 22) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:4 - Data Phase Segment 1"]
    #[inline(always)]
    pub fn dtseg1(&mut self) -> Dtseg1W<EdcbtSpec> {
        Dtseg1W::new(self, 0)
    }
    #[doc = "Bits 12:15 - Data Phase Time Segment 2"]
    #[inline(always)]
    pub fn dtseg2(&mut self) -> Dtseg2W<EdcbtSpec> {
        Dtseg2W::new(self, 12)
    }
    #[doc = "Bits 22:25 - Data Phase Resynchronization Jump Width"]
    #[inline(always)]
    pub fn drjw(&mut self) -> DrjwW<EdcbtSpec> {
        DrjwW::new(self, 22)
    }
}
#[doc = "Enhanced Data Phase CAN Bit Timing\n\nYou can [`read`](crate::Reg::read) this register and get [`edcbt::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`edcbt::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct EdcbtSpec;
impl crate::RegisterSpec for EdcbtSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`edcbt::R`](R) reader structure"]
impl crate::Readable for EdcbtSpec {}
#[doc = "`write(|w| ..)` method takes [`edcbt::W`](W) writer structure"]
impl crate::Writable for EdcbtSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets EDCBT to value 0"]
impl crate::Resettable for EdcbtSpec {}

#[doc = "Register `MWDATAHE` writer"]
pub type W = crate::W<MwdataheSpec>;
#[doc = "Field `DATA0` writer - Data Byte 0"]
pub type Data0W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA1` writer - Data Byte 1"]
pub type Data1W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl W {
    #[doc = "Bits 0:7 - Data Byte 0"]
    #[inline(always)]
    pub fn data0(&mut self) -> Data0W<MwdataheSpec> {
        Data0W::new(self, 0)
    }
    #[doc = "Bits 8:15 - Data Byte 1"]
    #[inline(always)]
    pub fn data1(&mut self) -> Data1W<MwdataheSpec> {
        Data1W::new(self, 8)
    }
}
#[doc = "Controller Write Data Halfword End\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mwdatahe::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MwdataheSpec;
impl crate::RegisterSpec for MwdataheSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`mwdatahe::W`](W) writer structure"]
impl crate::Writable for MwdataheSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MWDATAHE to value 0"]
impl crate::Resettable for MwdataheSpec {}

#[doc = "Register `SWDATAHE` writer"]
pub type W = crate::W<SwdataheSpec>;
#[doc = "Field `DATA0` writer - Data 0"]
pub type Data0W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA1` writer - Data 1"]
pub type Data1W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl W {
    #[doc = "Bits 0:7 - Data 0"]
    #[inline(always)]
    pub fn data0(&mut self) -> Data0W<SwdataheSpec> {
        Data0W::new(self, 0)
    }
    #[doc = "Bits 8:15 - Data 1"]
    #[inline(always)]
    pub fn data1(&mut self) -> Data1W<SwdataheSpec> {
        Data1W::new(self, 8)
    }
}
#[doc = "Target Write Data Halfword End\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`swdatahe::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SwdataheSpec;
impl crate::RegisterSpec for SwdataheSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`swdatahe::W`](W) writer structure"]
impl crate::Writable for SwdataheSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SWDATAHE to value 0"]
impl crate::Resettable for SwdataheSpec {}

#[doc = "Register `TCD_SLAST_SDA` reader"]
pub type R = crate::R<TcdSlastSdaSpec>;
#[doc = "Register `TCD_SLAST_SDA` writer"]
pub type W = crate::W<TcdSlastSdaSpec>;
#[doc = "Field `SLAST_SDA` reader - Last Source Address Adjustment / Store DADDR Address"]
pub type SlastSdaR = crate::FieldReader<u32>;
#[doc = "Field `SLAST_SDA` writer - Last Source Address Adjustment / Store DADDR Address"]
pub type SlastSdaW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Last Source Address Adjustment / Store DADDR Address"]
    #[inline(always)]
    pub fn slast_sda(&self) -> SlastSdaR {
        SlastSdaR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Last Source Address Adjustment / Store DADDR Address"]
    #[inline(always)]
    pub fn slast_sda(&mut self) -> SlastSdaW<TcdSlastSdaSpec> {
        SlastSdaW::new(self, 0)
    }
}
#[doc = "TCD Last Source Address Adjustment / Store DADDR Address\n\nYou can [`read`](crate::Reg::read) this register and get [`tcd_slast_sda::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tcd_slast_sda::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TcdSlastSdaSpec;
impl crate::RegisterSpec for TcdSlastSdaSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`tcd_slast_sda::R`](R) reader structure"]
impl crate::Readable for TcdSlastSdaSpec {}
#[doc = "`write(|w| ..)` method takes [`tcd_slast_sda::W`](W) writer structure"]
impl crate::Writable for TcdSlastSdaSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TCD_SLAST_SDA to value 0"]
impl crate::Resettable for TcdSlastSdaSpec {}

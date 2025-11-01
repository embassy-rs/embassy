#[doc = "Register `TCD_DLAST_SGA` reader"]
pub type R = crate::R<TcdDlastSgaSpec>;
#[doc = "Register `TCD_DLAST_SGA` writer"]
pub type W = crate::W<TcdDlastSgaSpec>;
#[doc = "Field `DLAST_SGA` reader - Last Destination Address Adjustment / Scatter Gather Address"]
pub type DlastSgaR = crate::FieldReader<u32>;
#[doc = "Field `DLAST_SGA` writer - Last Destination Address Adjustment / Scatter Gather Address"]
pub type DlastSgaW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Last Destination Address Adjustment / Scatter Gather Address"]
    #[inline(always)]
    pub fn dlast_sga(&self) -> DlastSgaR {
        DlastSgaR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Last Destination Address Adjustment / Scatter Gather Address"]
    #[inline(always)]
    pub fn dlast_sga(&mut self) -> DlastSgaW<TcdDlastSgaSpec> {
        DlastSgaW::new(self, 0)
    }
}
#[doc = "TCD Last Destination Address Adjustment / Scatter Gather Address\n\nYou can [`read`](crate::Reg::read) this register and get [`tcd_dlast_sga::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tcd_dlast_sga::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TcdDlastSgaSpec;
impl crate::RegisterSpec for TcdDlastSgaSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`tcd_dlast_sga::R`](R) reader structure"]
impl crate::Readable for TcdDlastSgaSpec {}
#[doc = "`write(|w| ..)` method takes [`tcd_dlast_sga::W`](W) writer structure"]
impl crate::Writable for TcdDlastSgaSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TCD_DLAST_SGA to value 0"]
impl crate::Resettable for TcdDlastSgaSpec {}

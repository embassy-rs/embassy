#[doc = "Register `MWMSG_SDR_DATA` writer"]
pub type W = crate::W<DataMwmsgSdrDataSpec>;
#[doc = "Field `DATA16B` writer - Data"]
pub type Data16bW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl W {
    #[doc = "Bits 0:15 - Data"]
    #[inline(always)]
    pub fn data16b(&mut self) -> Data16bW<DataMwmsgSdrDataSpec> {
        Data16bW::new(self, 0)
    }
}
#[doc = "Controller Write Message Data in SDR mode\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`data_mwmsg_sdr_data::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct DataMwmsgSdrDataSpec;
impl crate::RegisterSpec for DataMwmsgSdrDataSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`data_mwmsg_sdr_data::W`](W) writer structure"]
impl crate::Writable for DataMwmsgSdrDataSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MWMSG_SDR_DATA to value 0"]
impl crate::Resettable for DataMwmsgSdrDataSpec {}

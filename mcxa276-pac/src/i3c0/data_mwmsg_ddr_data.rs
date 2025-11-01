#[doc = "Register `MWMSG_DDR_DATA` writer"]
pub type W = crate::W<DataMwmsgDdrDataSpec>;
#[doc = "Field `DATA16B` writer - Data"]
pub type Data16bW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl W {
    #[doc = "Bits 0:15 - Data"]
    #[inline(always)]
    pub fn data16b(&mut self) -> Data16bW<DataMwmsgDdrDataSpec> {
        Data16bW::new(self, 0)
    }
}
#[doc = "Controller Write Message Data in DDR mode\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`data_mwmsg_ddr_data::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct DataMwmsgDdrDataSpec;
impl crate::RegisterSpec for DataMwmsgDdrDataSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`data_mwmsg_ddr_data::W`](W) writer structure"]
impl crate::Writable for DataMwmsgDdrDataSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MWMSG_DDR_DATA to value 0"]
impl crate::Resettable for DataMwmsgDdrDataSpec {}

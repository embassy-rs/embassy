#[doc = "Register `CH_MUX` reader"]
pub type R = crate::R<ChMuxSpec>;
#[doc = "Register `CH_MUX` writer"]
pub type W = crate::W<ChMuxSpec>;
#[doc = "Field `SRC` reader - Service Request Source"]
pub type SrcR = crate::FieldReader;
#[doc = "Field `SRC` writer - Service Request Source"]
pub type SrcW<'a, REG> = crate::FieldWriter<'a, REG, 7>;
impl R {
    #[doc = "Bits 0:6 - Service Request Source"]
    #[inline(always)]
    pub fn src(&self) -> SrcR {
        SrcR::new((self.bits & 0x7f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:6 - Service Request Source"]
    #[inline(always)]
    pub fn src(&mut self) -> SrcW<ChMuxSpec> {
        SrcW::new(self, 0)
    }
}
#[doc = "Channel Multiplexor Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`ch_mux::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ch_mux::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ChMuxSpec;
impl crate::RegisterSpec for ChMuxSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ch_mux::R`](R) reader structure"]
impl crate::Readable for ChMuxSpec {}
#[doc = "`write(|w| ..)` method takes [`ch_mux::W`](W) writer structure"]
impl crate::Writable for ChMuxSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CH_MUX to value 0"]
impl crate::Resettable for ChMuxSpec {}

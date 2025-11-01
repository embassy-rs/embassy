#[doc = "Register `CH_GRPRI[%s]` reader"]
pub type R = crate::R<ChGrpriSpec>;
#[doc = "Register `CH_GRPRI[%s]` writer"]
pub type W = crate::W<ChGrpriSpec>;
#[doc = "Field `GRPRI` reader - Arbitration Group For Channel n"]
pub type GrpriR = crate::FieldReader;
#[doc = "Field `GRPRI` writer - Arbitration Group For Channel n"]
pub type GrpriW<'a, REG> = crate::FieldWriter<'a, REG, 5>;
impl R {
    #[doc = "Bits 0:4 - Arbitration Group For Channel n"]
    #[inline(always)]
    pub fn grpri(&self) -> GrpriR {
        GrpriR::new((self.bits & 0x1f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:4 - Arbitration Group For Channel n"]
    #[inline(always)]
    pub fn grpri(&mut self) -> GrpriW<ChGrpriSpec> {
        GrpriW::new(self, 0)
    }
}
#[doc = "Channel Arbitration Group\n\nYou can [`read`](crate::Reg::read) this register and get [`ch_grpri::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ch_grpri::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ChGrpriSpec;
impl crate::RegisterSpec for ChGrpriSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ch_grpri::R`](R) reader structure"]
impl crate::Readable for ChGrpriSpec {}
#[doc = "`write(|w| ..)` method takes [`ch_grpri::W`](W) writer structure"]
impl crate::Writable for ChGrpriSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CH_GRPRI[%s] to value 0"]
impl crate::Resettable for ChGrpriSpec {}

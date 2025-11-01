#[doc = "Register `ACTIVE_CFG1` reader"]
pub type R = crate::R<ActiveCfg1Spec>;
#[doc = "Register `ACTIVE_CFG1` writer"]
pub type W = crate::W<ActiveCfg1Spec>;
#[doc = "Field `SOC_CNTRL` reader - Active Config Chip Control"]
pub type SocCntrlR = crate::FieldReader<u32>;
#[doc = "Field `SOC_CNTRL` writer - Active Config Chip Control"]
pub type SocCntrlW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Active Config Chip Control"]
    #[inline(always)]
    pub fn soc_cntrl(&self) -> SocCntrlR {
        SocCntrlR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Active Config Chip Control"]
    #[inline(always)]
    pub fn soc_cntrl(&mut self) -> SocCntrlW<ActiveCfg1Spec> {
        SocCntrlW::new(self, 0)
    }
}
#[doc = "Active Power Mode Configuration 1\n\nYou can [`read`](crate::Reg::read) this register and get [`active_cfg1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`active_cfg1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ActiveCfg1Spec;
impl crate::RegisterSpec for ActiveCfg1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`active_cfg1::R`](R) reader structure"]
impl crate::Readable for ActiveCfg1Spec {}
#[doc = "`write(|w| ..)` method takes [`active_cfg1::W`](W) writer structure"]
impl crate::Writable for ActiveCfg1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets ACTIVE_CFG1 to value 0x02"]
impl crate::Resettable for ActiveCfg1Spec {
    const RESET_VALUE: u32 = 0x02;
}

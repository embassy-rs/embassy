#[doc = "Register `SPLLLOCK_CNFG` reader"]
pub type R = crate::R<SplllockCnfgSpec>;
#[doc = "Register `SPLLLOCK_CNFG` writer"]
pub type W = crate::W<SplllockCnfgSpec>;
#[doc = "Field `LOCK_TIME` reader - Configures the number of reference clocks to count before SPLL is considered locked."]
pub type LockTimeR = crate::FieldReader<u32>;
#[doc = "Field `LOCK_TIME` writer - Configures the number of reference clocks to count before SPLL is considered locked."]
pub type LockTimeW<'a, REG> = crate::FieldWriter<'a, REG, 17, u32>;
impl R {
    #[doc = "Bits 0:16 - Configures the number of reference clocks to count before SPLL is considered locked."]
    #[inline(always)]
    pub fn lock_time(&self) -> LockTimeR {
        LockTimeR::new(self.bits & 0x0001_ffff)
    }
}
impl W {
    #[doc = "Bits 0:16 - Configures the number of reference clocks to count before SPLL is considered locked."]
    #[inline(always)]
    pub fn lock_time(&mut self) -> LockTimeW<SplllockCnfgSpec> {
        LockTimeW::new(self, 0)
    }
}
#[doc = "SPLL LOCK Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`splllock_cnfg::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`splllock_cnfg::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SplllockCnfgSpec;
impl crate::RegisterSpec for SplllockCnfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`splllock_cnfg::R`](R) reader structure"]
impl crate::Readable for SplllockCnfgSpec {}
#[doc = "`write(|w| ..)` method takes [`splllock_cnfg::W`](W) writer structure"]
impl crate::Writable for SplllockCnfgSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SPLLLOCK_CNFG to value 0x4f4c"]
impl crate::Resettable for SplllockCnfgSpec {
    const RESET_VALUE: u32 = 0x4f4c;
}

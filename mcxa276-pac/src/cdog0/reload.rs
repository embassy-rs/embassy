#[doc = "Register `RELOAD` reader"]
pub type R = crate::R<ReloadSpec>;
#[doc = "Register `RELOAD` writer"]
pub type W = crate::W<ReloadSpec>;
#[doc = "Field `RLOAD` reader - Instruction Timer reload value"]
pub type RloadR = crate::FieldReader<u32>;
#[doc = "Field `RLOAD` writer - Instruction Timer reload value"]
pub type RloadW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Instruction Timer reload value"]
    #[inline(always)]
    pub fn rload(&self) -> RloadR {
        RloadR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Instruction Timer reload value"]
    #[inline(always)]
    pub fn rload(&mut self) -> RloadW<ReloadSpec> {
        RloadW::new(self, 0)
    }
}
#[doc = "Instruction Timer Reload Register\n\nYou can [`read`](crate::Reg::read) this register and get [`reload::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`reload::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ReloadSpec;
impl crate::RegisterSpec for ReloadSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`reload::R`](R) reader structure"]
impl crate::Readable for ReloadSpec {}
#[doc = "`write(|w| ..)` method takes [`reload::W`](W) writer structure"]
impl crate::Writable for ReloadSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RELOAD to value 0xffff_ffff"]
impl crate::Resettable for ReloadSpec {
    const RESET_VALUE: u32 = 0xffff_ffff;
}

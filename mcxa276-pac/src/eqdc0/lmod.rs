#[doc = "Register `LMOD` reader"]
pub type R = crate::R<LmodSpec>;
#[doc = "Register `LMOD` writer"]
pub type W = crate::W<LmodSpec>;
#[doc = "Field `MOD` reader - MOD"]
pub type ModR = crate::FieldReader<u16>;
#[doc = "Field `MOD` writer - MOD"]
pub type ModW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - MOD"]
    #[inline(always)]
    pub fn mod_(&self) -> ModR {
        ModR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:15 - MOD"]
    #[inline(always)]
    pub fn mod_(&mut self) -> ModW<LmodSpec> {
        ModW::new(self, 0)
    }
}
#[doc = "Lower Modulus Register\n\nYou can [`read`](crate::Reg::read) this register and get [`lmod::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lmod::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct LmodSpec;
impl crate::RegisterSpec for LmodSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`lmod::R`](R) reader structure"]
impl crate::Readable for LmodSpec {}
#[doc = "`write(|w| ..)` method takes [`lmod::W`](W) writer structure"]
impl crate::Writable for LmodSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets LMOD to value 0"]
impl crate::Resettable for LmodSpec {}

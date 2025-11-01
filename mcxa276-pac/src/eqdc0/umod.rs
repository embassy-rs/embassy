#[doc = "Register `UMOD` reader"]
pub type R = crate::R<UmodSpec>;
#[doc = "Register `UMOD` writer"]
pub type W = crate::W<UmodSpec>;
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
    pub fn mod_(&mut self) -> ModW<UmodSpec> {
        ModW::new(self, 0)
    }
}
#[doc = "Upper Modulus Register\n\nYou can [`read`](crate::Reg::read) this register and get [`umod::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`umod::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct UmodSpec;
impl crate::RegisterSpec for UmodSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`umod::R`](R) reader structure"]
impl crate::Readable for UmodSpec {}
#[doc = "`write(|w| ..)` method takes [`umod::W`](W) writer structure"]
impl crate::Writable for UmodSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets UMOD to value 0"]
impl crate::Resettable for UmodSpec {}

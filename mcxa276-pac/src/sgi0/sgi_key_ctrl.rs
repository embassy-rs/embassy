#[doc = "Register `sgi_key_ctrl` reader"]
pub type R = crate::R<SgiKeyCtrlSpec>;
#[doc = "Register `sgi_key_ctrl` writer"]
pub type W = crate::W<SgiKeyCtrlSpec>;
#[doc = "Field `key_wo` reader - SGI Key control register(1-bit per KEY SFR)"]
pub type KeyWoR = crate::FieldReader<u32>;
#[doc = "Field `key_wo` writer - SGI Key control register(1-bit per KEY SFR)"]
pub type KeyWoW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - SGI Key control register(1-bit per KEY SFR)"]
    #[inline(always)]
    pub fn key_wo(&self) -> KeyWoR {
        KeyWoR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - SGI Key control register(1-bit per KEY SFR)"]
    #[inline(always)]
    pub fn key_wo(&mut self) -> KeyWoW<SgiKeyCtrlSpec> {
        KeyWoW::new(self, 0)
    }
}
#[doc = "SGI Key Control SFR\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key_ctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key_ctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKeyCtrlSpec;
impl crate::RegisterSpec for SgiKeyCtrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key_ctrl::R`](R) reader structure"]
impl crate::Readable for SgiKeyCtrlSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key_ctrl::W`](W) writer structure"]
impl crate::Writable for SgiKeyCtrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key_ctrl to value 0"]
impl crate::Resettable for SgiKeyCtrlSpec {}

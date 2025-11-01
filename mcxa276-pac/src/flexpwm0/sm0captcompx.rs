#[doc = "Register `SM0CAPTCOMPX` reader"]
pub type R = crate::R<Sm0captcompxSpec>;
#[doc = "Register `SM0CAPTCOMPX` writer"]
pub type W = crate::W<Sm0captcompxSpec>;
#[doc = "Field `EDGCMPX` reader - Edge Compare X"]
pub type EdgcmpxR = crate::FieldReader;
#[doc = "Field `EDGCMPX` writer - Edge Compare X"]
pub type EdgcmpxW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `EDGCNTX` reader - Edge Counter X"]
pub type EdgcntxR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:7 - Edge Compare X"]
    #[inline(always)]
    pub fn edgcmpx(&self) -> EdgcmpxR {
        EdgcmpxR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Edge Counter X"]
    #[inline(always)]
    pub fn edgcntx(&self) -> EdgcntxR {
        EdgcntxR::new(((self.bits >> 8) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Edge Compare X"]
    #[inline(always)]
    pub fn edgcmpx(&mut self) -> EdgcmpxW<Sm0captcompxSpec> {
        EdgcmpxW::new(self, 0)
    }
}
#[doc = "Capture Compare X Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0captcompx::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm0captcompx::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm0captcompxSpec;
impl crate::RegisterSpec for Sm0captcompxSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm0captcompx::R`](R) reader structure"]
impl crate::Readable for Sm0captcompxSpec {}
#[doc = "`write(|w| ..)` method takes [`sm0captcompx::W`](W) writer structure"]
impl crate::Writable for Sm0captcompxSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SM0CAPTCOMPX to value 0"]
impl crate::Resettable for Sm0captcompxSpec {}

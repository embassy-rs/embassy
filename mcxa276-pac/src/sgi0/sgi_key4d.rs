#[doc = "Register `sgi_key4d` reader"]
pub type R = crate::R<SgiKey4dSpec>;
#[doc = "Register `sgi_key4d` writer"]
pub type W = crate::W<SgiKey4dSpec>;
#[doc = "Field `key4d` reader - Input Key register"]
pub type Key4dR = crate::FieldReader<u32>;
#[doc = "Field `key4d` writer - Input Key register"]
pub type Key4dW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key4d(&self) -> Key4dR {
        Key4dR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key4d(&mut self) -> Key4dW<SgiKey4dSpec> {
        Key4dW::new(self, 0)
    }
}
#[doc = "Input Key register 4 - Word-0\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key4d::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key4d::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey4dSpec;
impl crate::RegisterSpec for SgiKey4dSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key4d::R`](R) reader structure"]
impl crate::Readable for SgiKey4dSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key4d::W`](W) writer structure"]
impl crate::Writable for SgiKey4dSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key4d to value 0"]
impl crate::Resettable for SgiKey4dSpec {}

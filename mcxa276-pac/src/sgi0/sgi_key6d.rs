#[doc = "Register `sgi_key6d` reader"]
pub type R = crate::R<SgiKey6dSpec>;
#[doc = "Register `sgi_key6d` writer"]
pub type W = crate::W<SgiKey6dSpec>;
#[doc = "Field `key6d` reader - Input Key register"]
pub type Key6dR = crate::FieldReader<u32>;
#[doc = "Field `key6d` writer - Input Key register"]
pub type Key6dW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key6d(&self) -> Key6dR {
        Key6dR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key6d(&mut self) -> Key6dW<SgiKey6dSpec> {
        Key6dW::new(self, 0)
    }
}
#[doc = "Input Key register 6 - Word-0\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key6d::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key6d::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey6dSpec;
impl crate::RegisterSpec for SgiKey6dSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key6d::R`](R) reader structure"]
impl crate::Readable for SgiKey6dSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key6d::W`](W) writer structure"]
impl crate::Writable for SgiKey6dSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key6d to value 0"]
impl crate::Resettable for SgiKey6dSpec {}

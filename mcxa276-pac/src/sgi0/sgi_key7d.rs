#[doc = "Register `sgi_key7d` reader"]
pub type R = crate::R<SgiKey7dSpec>;
#[doc = "Register `sgi_key7d` writer"]
pub type W = crate::W<SgiKey7dSpec>;
#[doc = "Field `key7d` reader - Input Key register"]
pub type Key7dR = crate::FieldReader<u32>;
#[doc = "Field `key7d` writer - Input Key register"]
pub type Key7dW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key7d(&self) -> Key7dR {
        Key7dR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key7d(&mut self) -> Key7dW<SgiKey7dSpec> {
        Key7dW::new(self, 0)
    }
}
#[doc = "Input Key register 7 - Word-0\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key7d::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key7d::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey7dSpec;
impl crate::RegisterSpec for SgiKey7dSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key7d::R`](R) reader structure"]
impl crate::Readable for SgiKey7dSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key7d::W`](W) writer structure"]
impl crate::Writable for SgiKey7dSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key7d to value 0"]
impl crate::Resettable for SgiKey7dSpec {}

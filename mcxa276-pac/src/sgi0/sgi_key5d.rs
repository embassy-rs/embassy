#[doc = "Register `sgi_key5d` reader"]
pub type R = crate::R<SgiKey5dSpec>;
#[doc = "Register `sgi_key5d` writer"]
pub type W = crate::W<SgiKey5dSpec>;
#[doc = "Field `key5d` reader - Input Key register"]
pub type Key5dR = crate::FieldReader<u32>;
#[doc = "Field `key5d` writer - Input Key register"]
pub type Key5dW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key5d(&self) -> Key5dR {
        Key5dR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key5d(&mut self) -> Key5dW<SgiKey5dSpec> {
        Key5dW::new(self, 0)
    }
}
#[doc = "Input Key register 5 - Word-0\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key5d::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key5d::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey5dSpec;
impl crate::RegisterSpec for SgiKey5dSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key5d::R`](R) reader structure"]
impl crate::Readable for SgiKey5dSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key5d::W`](W) writer structure"]
impl crate::Writable for SgiKey5dSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key5d to value 0"]
impl crate::Resettable for SgiKey5dSpec {}

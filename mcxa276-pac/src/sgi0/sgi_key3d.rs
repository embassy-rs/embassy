#[doc = "Register `sgi_key3d` reader"]
pub type R = crate::R<SgiKey3dSpec>;
#[doc = "Register `sgi_key3d` writer"]
pub type W = crate::W<SgiKey3dSpec>;
#[doc = "Field `key3d` reader - Input Key register"]
pub type Key3dR = crate::FieldReader<u32>;
#[doc = "Field `key3d` writer - Input Key register"]
pub type Key3dW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key3d(&self) -> Key3dR {
        Key3dR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key3d(&mut self) -> Key3dW<SgiKey3dSpec> {
        Key3dW::new(self, 0)
    }
}
#[doc = "Input Key register 3 - Word-0\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key3d::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key3d::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey3dSpec;
impl crate::RegisterSpec for SgiKey3dSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key3d::R`](R) reader structure"]
impl crate::Readable for SgiKey3dSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key3d::W`](W) writer structure"]
impl crate::Writable for SgiKey3dSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key3d to value 0"]
impl crate::Resettable for SgiKey3dSpec {}

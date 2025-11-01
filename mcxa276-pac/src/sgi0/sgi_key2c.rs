#[doc = "Register `sgi_key2c` reader"]
pub type R = crate::R<SgiKey2cSpec>;
#[doc = "Register `sgi_key2c` writer"]
pub type W = crate::W<SgiKey2cSpec>;
#[doc = "Field `key2c` reader - Input Key register"]
pub type Key2cR = crate::FieldReader<u32>;
#[doc = "Field `key2c` writer - Input Key register"]
pub type Key2cW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key2c(&self) -> Key2cR {
        Key2cR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Key register"]
    #[inline(always)]
    pub fn key2c(&mut self) -> Key2cW<SgiKey2cSpec> {
        Key2cW::new(self, 0)
    }
}
#[doc = "Input Key register 2 - Word-1\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key2c::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key2c::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKey2cSpec;
impl crate::RegisterSpec for SgiKey2cSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key2c::R`](R) reader structure"]
impl crate::Readable for SgiKey2cSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_key2c::W`](W) writer structure"]
impl crate::Writable for SgiKey2cSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_key2c to value 0"]
impl crate::Resettable for SgiKey2cSpec {}

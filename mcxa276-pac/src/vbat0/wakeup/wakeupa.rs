#[doc = "Register `WAKEUPA` reader"]
pub type R = crate::R<WakeupaSpec>;
#[doc = "Register `WAKEUPA` writer"]
pub type W = crate::W<WakeupaSpec>;
#[doc = "Field `REG` reader - Register"]
pub type RegR = crate::FieldReader<u32>;
#[doc = "Field `REG` writer - Register"]
pub type RegW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Register"]
    #[inline(always)]
    pub fn reg(&self) -> RegR {
        RegR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Register"]
    #[inline(always)]
    pub fn reg(&mut self) -> RegW<WakeupaSpec> {
        RegW::new(self, 0)
    }
}
#[doc = "Wakeup 0 Register A\n\nYou can [`read`](crate::Reg::read) this register and get [`wakeupa::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`wakeupa::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct WakeupaSpec;
impl crate::RegisterSpec for WakeupaSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`wakeupa::R`](R) reader structure"]
impl crate::Readable for WakeupaSpec {}
#[doc = "`write(|w| ..)` method takes [`wakeupa::W`](W) writer structure"]
impl crate::Writable for WakeupaSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets WAKEUPA to value 0"]
impl crate::Resettable for WakeupaSpec {}

#[doc = "Register `WINDOW` reader"]
pub type R = crate::R<WindowSpec>;
#[doc = "Register `WINDOW` writer"]
pub type W = crate::W<WindowSpec>;
#[doc = "Field `WINDOW` reader - Watchdog Window Value"]
pub type WindowR = crate::FieldReader<u32>;
#[doc = "Field `WINDOW` writer - Watchdog Window Value"]
pub type WindowW<'a, REG> = crate::FieldWriter<'a, REG, 24, u32>;
impl R {
    #[doc = "Bits 0:23 - Watchdog Window Value"]
    #[inline(always)]
    pub fn window(&self) -> WindowR {
        WindowR::new(self.bits & 0x00ff_ffff)
    }
}
impl W {
    #[doc = "Bits 0:23 - Watchdog Window Value"]
    #[inline(always)]
    pub fn window(&mut self) -> WindowW<WindowSpec> {
        WindowW::new(self, 0)
    }
}
#[doc = "Window Compare Value\n\nYou can [`read`](crate::Reg::read) this register and get [`window::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`window::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct WindowSpec;
impl crate::RegisterSpec for WindowSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`window::R`](R) reader structure"]
impl crate::Readable for WindowSpec {}
#[doc = "`write(|w| ..)` method takes [`window::W`](W) writer structure"]
impl crate::Writable for WindowSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets WINDOW to value 0x00ff_ffff"]
impl crate::Resettable for WindowSpec {
    const RESET_VALUE: u32 = 0x00ff_ffff;
}

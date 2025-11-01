#[doc = "Register `SCR5L` reader"]
pub type R = crate::R<Scr5lScr5lSpec>;
#[doc = "Register `SCR5L` writer"]
pub type W = crate::W<Scr5lScr5lSpec>;
#[doc = "Field `RUN5_MAX` reader - Run Length 5 Maximum Limit"]
pub type Run5MaxR = crate::FieldReader<u16>;
#[doc = "Field `RUN5_MAX` writer - Run Length 5 Maximum Limit"]
pub type Run5MaxW<'a, REG> = crate::FieldWriter<'a, REG, 11, u16>;
#[doc = "Field `RUN5_RNG` reader - Run Length 5 Range"]
pub type Run5RngR = crate::FieldReader<u16>;
#[doc = "Field `RUN5_RNG` writer - Run Length 5 Range"]
pub type Run5RngW<'a, REG> = crate::FieldWriter<'a, REG, 11, u16>;
impl R {
    #[doc = "Bits 0:10 - Run Length 5 Maximum Limit"]
    #[inline(always)]
    pub fn run5_max(&self) -> Run5MaxR {
        Run5MaxR::new((self.bits & 0x07ff) as u16)
    }
    #[doc = "Bits 16:26 - Run Length 5 Range"]
    #[inline(always)]
    pub fn run5_rng(&self) -> Run5RngR {
        Run5RngR::new(((self.bits >> 16) & 0x07ff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:10 - Run Length 5 Maximum Limit"]
    #[inline(always)]
    pub fn run5_max(&mut self) -> Run5MaxW<Scr5lScr5lSpec> {
        Run5MaxW::new(self, 0)
    }
    #[doc = "Bits 16:26 - Run Length 5 Range"]
    #[inline(always)]
    pub fn run5_rng(&mut self) -> Run5RngW<Scr5lScr5lSpec> {
        Run5RngW::new(self, 16)
    }
}
#[doc = "Statistical Check Run Length 5 Limit Register\n\nYou can [`read`](crate::Reg::read) this register and get [`scr5l_scr5l::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`scr5l_scr5l::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Scr5lScr5lSpec;
impl crate::RegisterSpec for Scr5lScr5lSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`scr5l_scr5l::R`](R) reader structure"]
impl crate::Readable for Scr5lScr5lSpec {}
#[doc = "`write(|w| ..)` method takes [`scr5l_scr5l::W`](W) writer structure"]
impl crate::Writable for Scr5lScr5lSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SCR5L to value 0x0013_0012"]
impl crate::Resettable for Scr5lScr5lSpec {
    const RESET_VALUE: u32 = 0x0013_0012;
}

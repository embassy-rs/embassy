#[doc = "Register `SCR4L` reader"]
pub type R = crate::R<Scr4lScr4lSpec>;
#[doc = "Register `SCR4L` writer"]
pub type W = crate::W<Scr4lScr4lSpec>;
#[doc = "Field `RUN4_MAX` reader - Run Length 4 Maximum Limit"]
pub type Run4MaxR = crate::FieldReader<u16>;
#[doc = "Field `RUN4_MAX` writer - Run Length 4 Maximum Limit"]
pub type Run4MaxW<'a, REG> = crate::FieldWriter<'a, REG, 12, u16>;
#[doc = "Field `RUN4_RNG` reader - Run Length 4 Range"]
pub type Run4RngR = crate::FieldReader<u16>;
#[doc = "Field `RUN4_RNG` writer - Run Length 4 Range"]
pub type Run4RngW<'a, REG> = crate::FieldWriter<'a, REG, 12, u16>;
impl R {
    #[doc = "Bits 0:11 - Run Length 4 Maximum Limit"]
    #[inline(always)]
    pub fn run4_max(&self) -> Run4MaxR {
        Run4MaxR::new((self.bits & 0x0fff) as u16)
    }
    #[doc = "Bits 16:27 - Run Length 4 Range"]
    #[inline(always)]
    pub fn run4_rng(&self) -> Run4RngR {
        Run4RngR::new(((self.bits >> 16) & 0x0fff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:11 - Run Length 4 Maximum Limit"]
    #[inline(always)]
    pub fn run4_max(&mut self) -> Run4MaxW<Scr4lScr4lSpec> {
        Run4MaxW::new(self, 0)
    }
    #[doc = "Bits 16:27 - Run Length 4 Range"]
    #[inline(always)]
    pub fn run4_rng(&mut self) -> Run4RngW<Scr4lScr4lSpec> {
        Run4RngW::new(self, 16)
    }
}
#[doc = "Statistical Check Run Length 4 Limit Register\n\nYou can [`read`](crate::Reg::read) this register and get [`scr4l_scr4l::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`scr4l_scr4l::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Scr4lScr4lSpec;
impl crate::RegisterSpec for Scr4lScr4lSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`scr4l_scr4l::R`](R) reader structure"]
impl crate::Readable for Scr4lScr4lSpec {}
#[doc = "`write(|w| ..)` method takes [`scr4l_scr4l::W`](W) writer structure"]
impl crate::Writable for Scr4lScr4lSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SCR4L to value 0x001b_001a"]
impl crate::Resettable for Scr4lScr4lSpec {
    const RESET_VALUE: u32 = 0x001b_001a;
}

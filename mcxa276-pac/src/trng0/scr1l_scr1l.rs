#[doc = "Register `SCR1L` reader"]
pub type R = crate::R<Scr1lScr1lSpec>;
#[doc = "Register `SCR1L` writer"]
pub type W = crate::W<Scr1lScr1lSpec>;
#[doc = "Field `RUN1_MAX` reader - Run Length 1 Maximum Limit"]
pub type Run1MaxR = crate::FieldReader<u16>;
#[doc = "Field `RUN1_MAX` writer - Run Length 1 Maximum Limit"]
pub type Run1MaxW<'a, REG> = crate::FieldWriter<'a, REG, 15, u16>;
#[doc = "Field `RUN1_RNG` reader - Run Length 1 Range"]
pub type Run1RngR = crate::FieldReader<u16>;
#[doc = "Field `RUN1_RNG` writer - Run Length 1 Range"]
pub type Run1RngW<'a, REG> = crate::FieldWriter<'a, REG, 15, u16>;
impl R {
    #[doc = "Bits 0:14 - Run Length 1 Maximum Limit"]
    #[inline(always)]
    pub fn run1_max(&self) -> Run1MaxR {
        Run1MaxR::new((self.bits & 0x7fff) as u16)
    }
    #[doc = "Bits 16:30 - Run Length 1 Range"]
    #[inline(always)]
    pub fn run1_rng(&self) -> Run1RngR {
        Run1RngR::new(((self.bits >> 16) & 0x7fff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:14 - Run Length 1 Maximum Limit"]
    #[inline(always)]
    pub fn run1_max(&mut self) -> Run1MaxW<Scr1lScr1lSpec> {
        Run1MaxW::new(self, 0)
    }
    #[doc = "Bits 16:30 - Run Length 1 Range"]
    #[inline(always)]
    pub fn run1_rng(&mut self) -> Run1RngW<Scr1lScr1lSpec> {
        Run1RngW::new(self, 16)
    }
}
#[doc = "Statistical Check Run Length 1 Limit Register\n\nYou can [`read`](crate::Reg::read) this register and get [`scr1l_scr1l::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`scr1l_scr1l::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Scr1lScr1lSpec;
impl crate::RegisterSpec for Scr1lScr1lSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`scr1l_scr1l::R`](R) reader structure"]
impl crate::Readable for Scr1lScr1lSpec {}
#[doc = "`write(|w| ..)` method takes [`scr1l_scr1l::W`](W) writer structure"]
impl crate::Writable for Scr1lScr1lSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SCR1L to value 0x004f_006b"]
impl crate::Resettable for Scr1lScr1lSpec {
    const RESET_VALUE: u32 = 0x004f_006b;
}

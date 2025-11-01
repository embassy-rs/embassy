#[doc = "Register `SCR3L` reader"]
pub type R = crate::R<Scr3lScr3lSpec>;
#[doc = "Register `SCR3L` writer"]
pub type W = crate::W<Scr3lScr3lSpec>;
#[doc = "Field `RUN3_MAX` reader - Run Length 3 Maximum Limit"]
pub type Run3MaxR = crate::FieldReader<u16>;
#[doc = "Field `RUN3_MAX` writer - Run Length 3 Maximum Limit"]
pub type Run3MaxW<'a, REG> = crate::FieldWriter<'a, REG, 13, u16>;
#[doc = "Field `RUN3_RNG` reader - Run Length 3 Range"]
pub type Run3RngR = crate::FieldReader<u16>;
#[doc = "Field `RUN3_RNG` writer - Run Length 3 Range"]
pub type Run3RngW<'a, REG> = crate::FieldWriter<'a, REG, 13, u16>;
impl R {
    #[doc = "Bits 0:12 - Run Length 3 Maximum Limit"]
    #[inline(always)]
    pub fn run3_max(&self) -> Run3MaxR {
        Run3MaxR::new((self.bits & 0x1fff) as u16)
    }
    #[doc = "Bits 16:28 - Run Length 3 Range"]
    #[inline(always)]
    pub fn run3_rng(&self) -> Run3RngR {
        Run3RngR::new(((self.bits >> 16) & 0x1fff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:12 - Run Length 3 Maximum Limit"]
    #[inline(always)]
    pub fn run3_max(&mut self) -> Run3MaxW<Scr3lScr3lSpec> {
        Run3MaxW::new(self, 0)
    }
    #[doc = "Bits 16:28 - Run Length 3 Range"]
    #[inline(always)]
    pub fn run3_rng(&mut self) -> Run3RngW<Scr3lScr3lSpec> {
        Run3RngW::new(self, 16)
    }
}
#[doc = "Statistical Check Run Length 3 Limit Register\n\nYou can [`read`](crate::Reg::read) this register and get [`scr3l_scr3l::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`scr3l_scr3l::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Scr3lScr3lSpec;
impl crate::RegisterSpec for Scr3lScr3lSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`scr3l_scr3l::R`](R) reader structure"]
impl crate::Readable for Scr3lScr3lSpec {}
#[doc = "`write(|w| ..)` method takes [`scr3l_scr3l::W`](W) writer structure"]
impl crate::Writable for Scr3lScr3lSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SCR3L to value 0x002d_0037"]
impl crate::Resettable for Scr3lScr3lSpec {
    const RESET_VALUE: u32 = 0x002d_0037;
}

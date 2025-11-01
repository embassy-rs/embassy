#[doc = "Register `LCD_WFTO[%s]` reader"]
pub type R = crate::R<LcdWftoSpec>;
#[doc = "Register `LCD_WFTO[%s]` writer"]
pub type W = crate::W<LcdWftoSpec>;
#[doc = "Field `WF0` reader - Waveform Pin 0"]
pub type Wf0R = crate::FieldReader;
#[doc = "Field `WF0` writer - Waveform Pin 0"]
pub type Wf0W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `WF1` reader - Waveform Pin 1"]
pub type Wf1R = crate::FieldReader;
#[doc = "Field `WF1` writer - Waveform Pin 1"]
pub type Wf1W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `WF2` reader - Waveform Pin 2"]
pub type Wf2R = crate::FieldReader;
#[doc = "Field `WF2` writer - Waveform Pin 2"]
pub type Wf2W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `WF3` reader - Waveform Pin 3"]
pub type Wf3R = crate::FieldReader;
#[doc = "Field `WF3` writer - Waveform Pin 3"]
pub type Wf3W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Waveform Pin 0"]
    #[inline(always)]
    pub fn wf0(&self) -> Wf0R {
        Wf0R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Waveform Pin 1"]
    #[inline(always)]
    pub fn wf1(&self) -> Wf1R {
        Wf1R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Waveform Pin 2"]
    #[inline(always)]
    pub fn wf2(&self) -> Wf2R {
        Wf2R::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - Waveform Pin 3"]
    #[inline(always)]
    pub fn wf3(&self) -> Wf3R {
        Wf3R::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Waveform Pin 0"]
    #[inline(always)]
    pub fn wf0(&mut self) -> Wf0W<LcdWftoSpec> {
        Wf0W::new(self, 0)
    }
    #[doc = "Bits 8:15 - Waveform Pin 1"]
    #[inline(always)]
    pub fn wf1(&mut self) -> Wf1W<LcdWftoSpec> {
        Wf1W::new(self, 8)
    }
    #[doc = "Bits 16:23 - Waveform Pin 2"]
    #[inline(always)]
    pub fn wf2(&mut self) -> Wf2W<LcdWftoSpec> {
        Wf2W::new(self, 16)
    }
    #[doc = "Bits 24:31 - Waveform Pin 3"]
    #[inline(always)]
    pub fn wf3(&mut self) -> Wf3W<LcdWftoSpec> {
        Wf3W::new(self, 24)
    }
}
#[doc = "LCD Waveform i * 4 + 3 to i * 4 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`lcd_wfto::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lcd_wfto::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct LcdWftoSpec;
impl crate::RegisterSpec for LcdWftoSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`lcd_wfto::R`](R) reader structure"]
impl crate::Readable for LcdWftoSpec {}
#[doc = "`write(|w| ..)` method takes [`lcd_wfto::W`](W) writer structure"]
impl crate::Writable for LcdWftoSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets LCD_WFTO[%s] to value 0"]
impl crate::Resettable for LcdWftoSpec {}

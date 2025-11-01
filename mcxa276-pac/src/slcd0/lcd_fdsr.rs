#[doc = "Register `LCD_FDSR` reader"]
pub type R = crate::R<LcdFdsrSpec>;
#[doc = "Register `LCD_FDSR` writer"]
pub type W = crate::W<LcdFdsrSpec>;
#[doc = "Field `FDCNT` reader - Fault Detect Counter"]
pub type FdcntR = crate::FieldReader;
#[doc = "Fault Detection Complete Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fdcf {
    #[doc = "0: Fault detection is not completed."]
    NotCompleted = 0,
    #[doc = "1: Fault detection is completed."]
    Completed = 1,
}
impl From<Fdcf> for bool {
    #[inline(always)]
    fn from(variant: Fdcf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FDCF` reader - Fault Detection Complete Flag"]
pub type FdcfR = crate::BitReader<Fdcf>;
impl FdcfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fdcf {
        match self.bits {
            false => Fdcf::NotCompleted,
            true => Fdcf::Completed,
        }
    }
    #[doc = "Fault detection is not completed."]
    #[inline(always)]
    pub fn is_not_completed(&self) -> bool {
        *self == Fdcf::NotCompleted
    }
    #[doc = "Fault detection is completed."]
    #[inline(always)]
    pub fn is_completed(&self) -> bool {
        *self == Fdcf::Completed
    }
}
#[doc = "Field `FDCF` writer - Fault Detection Complete Flag"]
pub type FdcfW<'a, REG> = crate::BitWriter1C<'a, REG, Fdcf>;
impl<'a, REG> FdcfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Fault detection is not completed."]
    #[inline(always)]
    pub fn not_completed(self) -> &'a mut crate::W<REG> {
        self.variant(Fdcf::NotCompleted)
    }
    #[doc = "Fault detection is completed."]
    #[inline(always)]
    pub fn completed(self) -> &'a mut crate::W<REG> {
        self.variant(Fdcf::Completed)
    }
}
impl R {
    #[doc = "Bits 0:7 - Fault Detect Counter"]
    #[inline(always)]
    pub fn fdcnt(&self) -> FdcntR {
        FdcntR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bit 15 - Fault Detection Complete Flag"]
    #[inline(always)]
    pub fn fdcf(&self) -> FdcfR {
        FdcfR::new(((self.bits >> 15) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 15 - Fault Detection Complete Flag"]
    #[inline(always)]
    pub fn fdcf(&mut self) -> FdcfW<LcdFdsrSpec> {
        FdcfW::new(self, 15)
    }
}
#[doc = "LCD Fault Detect Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`lcd_fdsr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lcd_fdsr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct LcdFdsrSpec;
impl crate::RegisterSpec for LcdFdsrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`lcd_fdsr::R`](R) reader structure"]
impl crate::Readable for LcdFdsrSpec {}
#[doc = "`write(|w| ..)` method takes [`lcd_fdsr::W`](W) writer structure"]
impl crate::Writable for LcdFdsrSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x8000;
}
#[doc = "`reset()` method sets LCD_FDSR to value 0"]
impl crate::Resettable for LcdFdsrSpec {}

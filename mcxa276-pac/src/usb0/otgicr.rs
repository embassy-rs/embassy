#[doc = "Register `OTGICR` reader"]
pub type R = crate::R<OtgicrSpec>;
#[doc = "Register `OTGICR` writer"]
pub type W = crate::W<OtgicrSpec>;
#[doc = "Line State Change Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Linestateen {
    #[doc = "0: Disable"]
    DisLinestInt = 0,
    #[doc = "1: Enable"]
    EnLinestInt = 1,
}
impl From<Linestateen> for bool {
    #[inline(always)]
    fn from(variant: Linestateen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LINESTATEEN` reader - Line State Change Interrupt Enable"]
pub type LinestateenR = crate::BitReader<Linestateen>;
impl LinestateenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Linestateen {
        match self.bits {
            false => Linestateen::DisLinestInt,
            true => Linestateen::EnLinestInt,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_linest_int(&self) -> bool {
        *self == Linestateen::DisLinestInt
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_linest_int(&self) -> bool {
        *self == Linestateen::EnLinestInt
    }
}
#[doc = "Field `LINESTATEEN` writer - Line State Change Interrupt Enable"]
pub type LinestateenW<'a, REG> = crate::BitWriter<'a, REG, Linestateen>;
impl<'a, REG> LinestateenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_linest_int(self) -> &'a mut crate::W<REG> {
        self.variant(Linestateen::DisLinestInt)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_linest_int(self) -> &'a mut crate::W<REG> {
        self.variant(Linestateen::EnLinestInt)
    }
}
#[doc = "1-Millisecond Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Onemsecen {
    #[doc = "0: Disable"]
    DisTimerInt = 0,
    #[doc = "1: Enable"]
    EnTimerInt = 1,
}
impl From<Onemsecen> for bool {
    #[inline(always)]
    fn from(variant: Onemsecen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ONEMSECEN` reader - 1-Millisecond Interrupt Enable"]
pub type OnemsecenR = crate::BitReader<Onemsecen>;
impl OnemsecenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Onemsecen {
        match self.bits {
            false => Onemsecen::DisTimerInt,
            true => Onemsecen::EnTimerInt,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_timer_int(&self) -> bool {
        *self == Onemsecen::DisTimerInt
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_timer_int(&self) -> bool {
        *self == Onemsecen::EnTimerInt
    }
}
#[doc = "Field `ONEMSECEN` writer - 1-Millisecond Interrupt Enable"]
pub type OnemsecenW<'a, REG> = crate::BitWriter<'a, REG, Onemsecen>;
impl<'a, REG> OnemsecenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_timer_int(self) -> &'a mut crate::W<REG> {
        self.variant(Onemsecen::DisTimerInt)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_timer_int(self) -> &'a mut crate::W<REG> {
        self.variant(Onemsecen::EnTimerInt)
    }
}
impl R {
    #[doc = "Bit 5 - Line State Change Interrupt Enable"]
    #[inline(always)]
    pub fn linestateen(&self) -> LinestateenR {
        LinestateenR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - 1-Millisecond Interrupt Enable"]
    #[inline(always)]
    pub fn onemsecen(&self) -> OnemsecenR {
        OnemsecenR::new(((self.bits >> 6) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 5 - Line State Change Interrupt Enable"]
    #[inline(always)]
    pub fn linestateen(&mut self) -> LinestateenW<OtgicrSpec> {
        LinestateenW::new(self, 5)
    }
    #[doc = "Bit 6 - 1-Millisecond Interrupt Enable"]
    #[inline(always)]
    pub fn onemsecen(&mut self) -> OnemsecenW<OtgicrSpec> {
        OnemsecenW::new(self, 6)
    }
}
#[doc = "OTG Interrupt Control\n\nYou can [`read`](crate::Reg::read) this register and get [`otgicr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`otgicr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct OtgicrSpec;
impl crate::RegisterSpec for OtgicrSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`otgicr::R`](R) reader structure"]
impl crate::Readable for OtgicrSpec {}
#[doc = "`write(|w| ..)` method takes [`otgicr::W`](W) writer structure"]
impl crate::Writable for OtgicrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets OTGICR to value 0"]
impl crate::Resettable for OtgicrSpec {}

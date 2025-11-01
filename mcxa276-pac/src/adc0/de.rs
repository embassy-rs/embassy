#[doc = "Register `DE` reader"]
pub type R = crate::R<DeSpec>;
#[doc = "Register `DE` writer"]
pub type W = crate::W<DeSpec>;
#[doc = "FIFO 0 Watermark DMA Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fwmde0 {
    #[doc = "0: DMA request disabled."]
    Disabled = 0,
    #[doc = "1: DMA request enabled."]
    Enabled = 1,
}
impl From<Fwmde0> for bool {
    #[inline(always)]
    fn from(variant: Fwmde0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FWMDE0` reader - FIFO 0 Watermark DMA Enable"]
pub type Fwmde0R = crate::BitReader<Fwmde0>;
impl Fwmde0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fwmde0 {
        match self.bits {
            false => Fwmde0::Disabled,
            true => Fwmde0::Enabled,
        }
    }
    #[doc = "DMA request disabled."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Fwmde0::Disabled
    }
    #[doc = "DMA request enabled."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Fwmde0::Enabled
    }
}
#[doc = "Field `FWMDE0` writer - FIFO 0 Watermark DMA Enable"]
pub type Fwmde0W<'a, REG> = crate::BitWriter<'a, REG, Fwmde0>;
impl<'a, REG> Fwmde0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "DMA request disabled."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Fwmde0::Disabled)
    }
    #[doc = "DMA request enabled."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Fwmde0::Enabled)
    }
}
impl R {
    #[doc = "Bit 0 - FIFO 0 Watermark DMA Enable"]
    #[inline(always)]
    pub fn fwmde0(&self) -> Fwmde0R {
        Fwmde0R::new((self.bits & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - FIFO 0 Watermark DMA Enable"]
    #[inline(always)]
    pub fn fwmde0(&mut self) -> Fwmde0W<DeSpec> {
        Fwmde0W::new(self, 0)
    }
}
#[doc = "DMA Enable Register\n\nYou can [`read`](crate::Reg::read) this register and get [`de::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`de::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct DeSpec;
impl crate::RegisterSpec for DeSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`de::R`](R) reader structure"]
impl crate::Readable for DeSpec {}
#[doc = "`write(|w| ..)` method takes [`de::W`](W) writer structure"]
impl crate::Writable for DeSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets DE to value 0"]
impl crate::Resettable for DeSpec {}

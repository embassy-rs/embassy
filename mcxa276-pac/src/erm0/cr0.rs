#[doc = "Register `CR0` reader"]
pub type R = crate::R<Cr0Spec>;
#[doc = "Register `CR0` writer"]
pub type W = crate::W<Cr0Spec>;
#[doc = "ENCIE1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Encie1 {
    #[doc = "0: Interrupt notification of Memory 1 non-correctable error events is disabled."]
    Disable = 0,
    #[doc = "1: Interrupt notification of Memory 1 non-correctable error events is enabled."]
    Enable = 1,
}
impl From<Encie1> for bool {
    #[inline(always)]
    fn from(variant: Encie1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ENCIE1` reader - ENCIE1"]
pub type Encie1R = crate::BitReader<Encie1>;
impl Encie1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Encie1 {
        match self.bits {
            false => Encie1::Disable,
            true => Encie1::Enable,
        }
    }
    #[doc = "Interrupt notification of Memory 1 non-correctable error events is disabled."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Encie1::Disable
    }
    #[doc = "Interrupt notification of Memory 1 non-correctable error events is enabled."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Encie1::Enable
    }
}
#[doc = "Field `ENCIE1` writer - ENCIE1"]
pub type Encie1W<'a, REG> = crate::BitWriter<'a, REG, Encie1>;
impl<'a, REG> Encie1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Interrupt notification of Memory 1 non-correctable error events is disabled."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Encie1::Disable)
    }
    #[doc = "Interrupt notification of Memory 1 non-correctable error events is enabled."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Encie1::Enable)
    }
}
#[doc = "ESCIE1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Escie1 {
    #[doc = "0: Interrupt notification of Memory 1 single-bit correction events is disabled."]
    Disable = 0,
    #[doc = "1: Interrupt notification of Memory 1 single-bit correction events is enabled."]
    Enable = 1,
}
impl From<Escie1> for bool {
    #[inline(always)]
    fn from(variant: Escie1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ESCIE1` reader - ESCIE1"]
pub type Escie1R = crate::BitReader<Escie1>;
impl Escie1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Escie1 {
        match self.bits {
            false => Escie1::Disable,
            true => Escie1::Enable,
        }
    }
    #[doc = "Interrupt notification of Memory 1 single-bit correction events is disabled."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Escie1::Disable
    }
    #[doc = "Interrupt notification of Memory 1 single-bit correction events is enabled."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Escie1::Enable
    }
}
#[doc = "Field `ESCIE1` writer - ESCIE1"]
pub type Escie1W<'a, REG> = crate::BitWriter<'a, REG, Escie1>;
impl<'a, REG> Escie1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Interrupt notification of Memory 1 single-bit correction events is disabled."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Escie1::Disable)
    }
    #[doc = "Interrupt notification of Memory 1 single-bit correction events is enabled."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Escie1::Enable)
    }
}
#[doc = "ENCIE0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Encie0 {
    #[doc = "0: Interrupt notification of Memory 0 non-correctable error events is disabled."]
    Disable = 0,
    #[doc = "1: Interrupt notification of Memory 0 non-correctable error events is enabled."]
    Enable = 1,
}
impl From<Encie0> for bool {
    #[inline(always)]
    fn from(variant: Encie0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ENCIE0` reader - ENCIE0"]
pub type Encie0R = crate::BitReader<Encie0>;
impl Encie0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Encie0 {
        match self.bits {
            false => Encie0::Disable,
            true => Encie0::Enable,
        }
    }
    #[doc = "Interrupt notification of Memory 0 non-correctable error events is disabled."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Encie0::Disable
    }
    #[doc = "Interrupt notification of Memory 0 non-correctable error events is enabled."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Encie0::Enable
    }
}
#[doc = "Field `ENCIE0` writer - ENCIE0"]
pub type Encie0W<'a, REG> = crate::BitWriter<'a, REG, Encie0>;
impl<'a, REG> Encie0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Interrupt notification of Memory 0 non-correctable error events is disabled."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Encie0::Disable)
    }
    #[doc = "Interrupt notification of Memory 0 non-correctable error events is enabled."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Encie0::Enable)
    }
}
#[doc = "ESCIE0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Escie0 {
    #[doc = "0: Interrupt notification of Memory 0 single-bit correction events is disabled."]
    Disable = 0,
    #[doc = "1: Interrupt notification of Memory 0 single-bit correction events is enabled."]
    Enable = 1,
}
impl From<Escie0> for bool {
    #[inline(always)]
    fn from(variant: Escie0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ESCIE0` reader - ESCIE0"]
pub type Escie0R = crate::BitReader<Escie0>;
impl Escie0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Escie0 {
        match self.bits {
            false => Escie0::Disable,
            true => Escie0::Enable,
        }
    }
    #[doc = "Interrupt notification of Memory 0 single-bit correction events is disabled."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Escie0::Disable
    }
    #[doc = "Interrupt notification of Memory 0 single-bit correction events is enabled."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Escie0::Enable
    }
}
#[doc = "Field `ESCIE0` writer - ESCIE0"]
pub type Escie0W<'a, REG> = crate::BitWriter<'a, REG, Escie0>;
impl<'a, REG> Escie0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Interrupt notification of Memory 0 single-bit correction events is disabled."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Escie0::Disable)
    }
    #[doc = "Interrupt notification of Memory 0 single-bit correction events is enabled."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Escie0::Enable)
    }
}
impl R {
    #[doc = "Bit 26 - ENCIE1"]
    #[inline(always)]
    pub fn encie1(&self) -> Encie1R {
        Encie1R::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - ESCIE1"]
    #[inline(always)]
    pub fn escie1(&self) -> Escie1R {
        Escie1R::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bit 30 - ENCIE0"]
    #[inline(always)]
    pub fn encie0(&self) -> Encie0R {
        Encie0R::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - ESCIE0"]
    #[inline(always)]
    pub fn escie0(&self) -> Escie0R {
        Escie0R::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 26 - ENCIE1"]
    #[inline(always)]
    pub fn encie1(&mut self) -> Encie1W<Cr0Spec> {
        Encie1W::new(self, 26)
    }
    #[doc = "Bit 27 - ESCIE1"]
    #[inline(always)]
    pub fn escie1(&mut self) -> Escie1W<Cr0Spec> {
        Escie1W::new(self, 27)
    }
    #[doc = "Bit 30 - ENCIE0"]
    #[inline(always)]
    pub fn encie0(&mut self) -> Encie0W<Cr0Spec> {
        Encie0W::new(self, 30)
    }
    #[doc = "Bit 31 - ESCIE0"]
    #[inline(always)]
    pub fn escie0(&mut self) -> Escie0W<Cr0Spec> {
        Escie0W::new(self, 31)
    }
}
#[doc = "ERM Configuration Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`cr0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cr0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Cr0Spec;
impl crate::RegisterSpec for Cr0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`cr0::R`](R) reader structure"]
impl crate::Readable for Cr0Spec {}
#[doc = "`write(|w| ..)` method takes [`cr0::W`](W) writer structure"]
impl crate::Writable for Cr0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CR0 to value 0"]
impl crate::Resettable for Cr0Spec {}

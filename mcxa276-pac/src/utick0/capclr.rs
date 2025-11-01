#[doc = "Register `CAPCLR` writer"]
pub type W = crate::W<CapclrSpec>;
#[doc = "Clear Capture 0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capclr0 {
    #[doc = "0: Does nothing"]
    Capclr0nothing = 0,
    #[doc = "1: Clears the CAP0 register value"]
    Capclr0cleared = 1,
}
impl From<Capclr0> for bool {
    #[inline(always)]
    fn from(variant: Capclr0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CAPCLR0` writer - Clear Capture 0"]
pub type Capclr0W<'a, REG> = crate::BitWriter<'a, REG, Capclr0>;
impl<'a, REG> Capclr0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does nothing"]
    #[inline(always)]
    pub fn capclr0nothing(self) -> &'a mut crate::W<REG> {
        self.variant(Capclr0::Capclr0nothing)
    }
    #[doc = "Clears the CAP0 register value"]
    #[inline(always)]
    pub fn capclr0cleared(self) -> &'a mut crate::W<REG> {
        self.variant(Capclr0::Capclr0cleared)
    }
}
#[doc = "Clear Capture 1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capclr1 {
    #[doc = "0: Does nothing"]
    Capclr1nothing = 0,
    #[doc = "1: Clears the CAP1 register value"]
    Capclr1cleared = 1,
}
impl From<Capclr1> for bool {
    #[inline(always)]
    fn from(variant: Capclr1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CAPCLR1` writer - Clear Capture 1"]
pub type Capclr1W<'a, REG> = crate::BitWriter<'a, REG, Capclr1>;
impl<'a, REG> Capclr1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does nothing"]
    #[inline(always)]
    pub fn capclr1nothing(self) -> &'a mut crate::W<REG> {
        self.variant(Capclr1::Capclr1nothing)
    }
    #[doc = "Clears the CAP1 register value"]
    #[inline(always)]
    pub fn capclr1cleared(self) -> &'a mut crate::W<REG> {
        self.variant(Capclr1::Capclr1cleared)
    }
}
#[doc = "Clear Capture 2\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capclr2 {
    #[doc = "0: Does nothing"]
    Capclr2nothing = 0,
    #[doc = "1: Clears the CAP2 register value"]
    Capclr2cleared = 1,
}
impl From<Capclr2> for bool {
    #[inline(always)]
    fn from(variant: Capclr2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CAPCLR2` writer - Clear Capture 2"]
pub type Capclr2W<'a, REG> = crate::BitWriter<'a, REG, Capclr2>;
impl<'a, REG> Capclr2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does nothing"]
    #[inline(always)]
    pub fn capclr2nothing(self) -> &'a mut crate::W<REG> {
        self.variant(Capclr2::Capclr2nothing)
    }
    #[doc = "Clears the CAP2 register value"]
    #[inline(always)]
    pub fn capclr2cleared(self) -> &'a mut crate::W<REG> {
        self.variant(Capclr2::Capclr2cleared)
    }
}
#[doc = "Clear Capture 3\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capclr3 {
    #[doc = "0: Does nothing"]
    Capclr3nothing = 0,
    #[doc = "1: Clears the CAP3 register value"]
    Capclr3cleared = 1,
}
impl From<Capclr3> for bool {
    #[inline(always)]
    fn from(variant: Capclr3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CAPCLR3` writer - Clear Capture 3"]
pub type Capclr3W<'a, REG> = crate::BitWriter<'a, REG, Capclr3>;
impl<'a, REG> Capclr3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does nothing"]
    #[inline(always)]
    pub fn capclr3nothing(self) -> &'a mut crate::W<REG> {
        self.variant(Capclr3::Capclr3nothing)
    }
    #[doc = "Clears the CAP3 register value"]
    #[inline(always)]
    pub fn capclr3cleared(self) -> &'a mut crate::W<REG> {
        self.variant(Capclr3::Capclr3cleared)
    }
}
impl W {
    #[doc = "Bit 0 - Clear Capture 0"]
    #[inline(always)]
    pub fn capclr0(&mut self) -> Capclr0W<CapclrSpec> {
        Capclr0W::new(self, 0)
    }
    #[doc = "Bit 1 - Clear Capture 1"]
    #[inline(always)]
    pub fn capclr1(&mut self) -> Capclr1W<CapclrSpec> {
        Capclr1W::new(self, 1)
    }
    #[doc = "Bit 2 - Clear Capture 2"]
    #[inline(always)]
    pub fn capclr2(&mut self) -> Capclr2W<CapclrSpec> {
        Capclr2W::new(self, 2)
    }
    #[doc = "Bit 3 - Clear Capture 3"]
    #[inline(always)]
    pub fn capclr3(&mut self) -> Capclr3W<CapclrSpec> {
        Capclr3W::new(self, 3)
    }
}
#[doc = "Capture Clear\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`capclr::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CapclrSpec;
impl crate::RegisterSpec for CapclrSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`capclr::W`](W) writer structure"]
impl crate::Writable for CapclrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CAPCLR to value 0"]
impl crate::Resettable for CapclrSpec {}

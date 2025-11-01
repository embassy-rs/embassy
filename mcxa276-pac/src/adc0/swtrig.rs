#[doc = "Register `SWTRIG` reader"]
pub type R = crate::R<SwtrigSpec>;
#[doc = "Register `SWTRIG` writer"]
pub type W = crate::W<SwtrigSpec>;
#[doc = "Software Trigger 0 Event\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Swt0 {
    #[doc = "0: No trigger 0 event generated."]
    NoTrigger = 0,
    #[doc = "1: Trigger 0 event generated."]
    InitiateTrigger0 = 1,
}
impl From<Swt0> for bool {
    #[inline(always)]
    fn from(variant: Swt0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SWT0` reader - Software Trigger 0 Event"]
pub type Swt0R = crate::BitReader<Swt0>;
impl Swt0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Swt0 {
        match self.bits {
            false => Swt0::NoTrigger,
            true => Swt0::InitiateTrigger0,
        }
    }
    #[doc = "No trigger 0 event generated."]
    #[inline(always)]
    pub fn is_no_trigger(&self) -> bool {
        *self == Swt0::NoTrigger
    }
    #[doc = "Trigger 0 event generated."]
    #[inline(always)]
    pub fn is_initiate_trigger_0(&self) -> bool {
        *self == Swt0::InitiateTrigger0
    }
}
#[doc = "Field `SWT0` writer - Software Trigger 0 Event"]
pub type Swt0W<'a, REG> = crate::BitWriter<'a, REG, Swt0>;
impl<'a, REG> Swt0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No trigger 0 event generated."]
    #[inline(always)]
    pub fn no_trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Swt0::NoTrigger)
    }
    #[doc = "Trigger 0 event generated."]
    #[inline(always)]
    pub fn initiate_trigger_0(self) -> &'a mut crate::W<REG> {
        self.variant(Swt0::InitiateTrigger0)
    }
}
#[doc = "Software Trigger 1 Event\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Swt1 {
    #[doc = "0: No trigger 1 event generated."]
    NoTrigger = 0,
    #[doc = "1: Trigger 1 event generated."]
    InitiateTrigger1 = 1,
}
impl From<Swt1> for bool {
    #[inline(always)]
    fn from(variant: Swt1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SWT1` reader - Software Trigger 1 Event"]
pub type Swt1R = crate::BitReader<Swt1>;
impl Swt1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Swt1 {
        match self.bits {
            false => Swt1::NoTrigger,
            true => Swt1::InitiateTrigger1,
        }
    }
    #[doc = "No trigger 1 event generated."]
    #[inline(always)]
    pub fn is_no_trigger(&self) -> bool {
        *self == Swt1::NoTrigger
    }
    #[doc = "Trigger 1 event generated."]
    #[inline(always)]
    pub fn is_initiate_trigger_1(&self) -> bool {
        *self == Swt1::InitiateTrigger1
    }
}
#[doc = "Field `SWT1` writer - Software Trigger 1 Event"]
pub type Swt1W<'a, REG> = crate::BitWriter<'a, REG, Swt1>;
impl<'a, REG> Swt1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No trigger 1 event generated."]
    #[inline(always)]
    pub fn no_trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Swt1::NoTrigger)
    }
    #[doc = "Trigger 1 event generated."]
    #[inline(always)]
    pub fn initiate_trigger_1(self) -> &'a mut crate::W<REG> {
        self.variant(Swt1::InitiateTrigger1)
    }
}
#[doc = "Software Trigger 2 Event\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Swt2 {
    #[doc = "0: No trigger 2 event generated."]
    NoTrigger = 0,
    #[doc = "1: Trigger 2 event generated."]
    InitiateTrigger2 = 1,
}
impl From<Swt2> for bool {
    #[inline(always)]
    fn from(variant: Swt2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SWT2` reader - Software Trigger 2 Event"]
pub type Swt2R = crate::BitReader<Swt2>;
impl Swt2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Swt2 {
        match self.bits {
            false => Swt2::NoTrigger,
            true => Swt2::InitiateTrigger2,
        }
    }
    #[doc = "No trigger 2 event generated."]
    #[inline(always)]
    pub fn is_no_trigger(&self) -> bool {
        *self == Swt2::NoTrigger
    }
    #[doc = "Trigger 2 event generated."]
    #[inline(always)]
    pub fn is_initiate_trigger_2(&self) -> bool {
        *self == Swt2::InitiateTrigger2
    }
}
#[doc = "Field `SWT2` writer - Software Trigger 2 Event"]
pub type Swt2W<'a, REG> = crate::BitWriter<'a, REG, Swt2>;
impl<'a, REG> Swt2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No trigger 2 event generated."]
    #[inline(always)]
    pub fn no_trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Swt2::NoTrigger)
    }
    #[doc = "Trigger 2 event generated."]
    #[inline(always)]
    pub fn initiate_trigger_2(self) -> &'a mut crate::W<REG> {
        self.variant(Swt2::InitiateTrigger2)
    }
}
#[doc = "Software Trigger 3 Event\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Swt3 {
    #[doc = "0: No trigger 3 event generated."]
    NoTrigger = 0,
    #[doc = "1: Trigger 3 event generated."]
    InitiateTrigger3 = 1,
}
impl From<Swt3> for bool {
    #[inline(always)]
    fn from(variant: Swt3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SWT3` reader - Software Trigger 3 Event"]
pub type Swt3R = crate::BitReader<Swt3>;
impl Swt3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Swt3 {
        match self.bits {
            false => Swt3::NoTrigger,
            true => Swt3::InitiateTrigger3,
        }
    }
    #[doc = "No trigger 3 event generated."]
    #[inline(always)]
    pub fn is_no_trigger(&self) -> bool {
        *self == Swt3::NoTrigger
    }
    #[doc = "Trigger 3 event generated."]
    #[inline(always)]
    pub fn is_initiate_trigger_3(&self) -> bool {
        *self == Swt3::InitiateTrigger3
    }
}
#[doc = "Field `SWT3` writer - Software Trigger 3 Event"]
pub type Swt3W<'a, REG> = crate::BitWriter<'a, REG, Swt3>;
impl<'a, REG> Swt3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No trigger 3 event generated."]
    #[inline(always)]
    pub fn no_trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Swt3::NoTrigger)
    }
    #[doc = "Trigger 3 event generated."]
    #[inline(always)]
    pub fn initiate_trigger_3(self) -> &'a mut crate::W<REG> {
        self.variant(Swt3::InitiateTrigger3)
    }
}
impl R {
    #[doc = "Bit 0 - Software Trigger 0 Event"]
    #[inline(always)]
    pub fn swt0(&self) -> Swt0R {
        Swt0R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Software Trigger 1 Event"]
    #[inline(always)]
    pub fn swt1(&self) -> Swt1R {
        Swt1R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Software Trigger 2 Event"]
    #[inline(always)]
    pub fn swt2(&self) -> Swt2R {
        Swt2R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Software Trigger 3 Event"]
    #[inline(always)]
    pub fn swt3(&self) -> Swt3R {
        Swt3R::new(((self.bits >> 3) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Software Trigger 0 Event"]
    #[inline(always)]
    pub fn swt0(&mut self) -> Swt0W<SwtrigSpec> {
        Swt0W::new(self, 0)
    }
    #[doc = "Bit 1 - Software Trigger 1 Event"]
    #[inline(always)]
    pub fn swt1(&mut self) -> Swt1W<SwtrigSpec> {
        Swt1W::new(self, 1)
    }
    #[doc = "Bit 2 - Software Trigger 2 Event"]
    #[inline(always)]
    pub fn swt2(&mut self) -> Swt2W<SwtrigSpec> {
        Swt2W::new(self, 2)
    }
    #[doc = "Bit 3 - Software Trigger 3 Event"]
    #[inline(always)]
    pub fn swt3(&mut self) -> Swt3W<SwtrigSpec> {
        Swt3W::new(self, 3)
    }
}
#[doc = "Software Trigger Register\n\nYou can [`read`](crate::Reg::read) this register and get [`swtrig::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`swtrig::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SwtrigSpec;
impl crate::RegisterSpec for SwtrigSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`swtrig::R`](R) reader structure"]
impl crate::Readable for SwtrigSpec {}
#[doc = "`write(|w| ..)` method takes [`swtrig::W`](W) writer structure"]
impl crate::Writable for SwtrigSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SWTRIG to value 0"]
impl crate::Resettable for SwtrigSpec {}

#[doc = "Register `CPUSTAT` reader"]
pub type R = crate::R<CpustatSpec>;
#[doc = "CPU0 sleeping state\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cpu0sleeping {
    #[doc = "0: CPU is not sleeping"]
    Awake = 0,
    #[doc = "1: CPU is sleeping"]
    Sleeping = 1,
}
impl From<Cpu0sleeping> for bool {
    #[inline(always)]
    fn from(variant: Cpu0sleeping) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CPU0SLEEPING` reader - CPU0 sleeping state"]
pub type Cpu0sleepingR = crate::BitReader<Cpu0sleeping>;
impl Cpu0sleepingR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cpu0sleeping {
        match self.bits {
            false => Cpu0sleeping::Awake,
            true => Cpu0sleeping::Sleeping,
        }
    }
    #[doc = "CPU is not sleeping"]
    #[inline(always)]
    pub fn is_awake(&self) -> bool {
        *self == Cpu0sleeping::Awake
    }
    #[doc = "CPU is sleeping"]
    #[inline(always)]
    pub fn is_sleeping(&self) -> bool {
        *self == Cpu0sleeping::Sleeping
    }
}
#[doc = "CPU0 lockup state\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cpu0lockup {
    #[doc = "0: CPU is not in lockup"]
    Awake = 0,
    #[doc = "1: CPU is in lockup"]
    Sleeping = 1,
}
impl From<Cpu0lockup> for bool {
    #[inline(always)]
    fn from(variant: Cpu0lockup) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CPU0LOCKUP` reader - CPU0 lockup state"]
pub type Cpu0lockupR = crate::BitReader<Cpu0lockup>;
impl Cpu0lockupR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cpu0lockup {
        match self.bits {
            false => Cpu0lockup::Awake,
            true => Cpu0lockup::Sleeping,
        }
    }
    #[doc = "CPU is not in lockup"]
    #[inline(always)]
    pub fn is_awake(&self) -> bool {
        *self == Cpu0lockup::Awake
    }
    #[doc = "CPU is in lockup"]
    #[inline(always)]
    pub fn is_sleeping(&self) -> bool {
        *self == Cpu0lockup::Sleeping
    }
}
impl R {
    #[doc = "Bit 0 - CPU0 sleeping state"]
    #[inline(always)]
    pub fn cpu0sleeping(&self) -> Cpu0sleepingR {
        Cpu0sleepingR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 2 - CPU0 lockup state"]
    #[inline(always)]
    pub fn cpu0lockup(&self) -> Cpu0lockupR {
        Cpu0lockupR::new(((self.bits >> 2) & 1) != 0)
    }
}
#[doc = "CPU Status\n\nYou can [`read`](crate::Reg::read) this register and get [`cpustat::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CpustatSpec;
impl crate::RegisterSpec for CpustatSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`cpustat::R`](R) reader structure"]
impl crate::Readable for CpustatSpec {}
#[doc = "`reset()` method sets CPUSTAT to value 0"]
impl crate::Resettable for CpustatSpec {}

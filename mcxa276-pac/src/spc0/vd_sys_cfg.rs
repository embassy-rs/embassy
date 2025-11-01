#[doc = "Register `VD_SYS_CFG` reader"]
pub type R = crate::R<VdSysCfgSpec>;
#[doc = "Register `VD_SYS_CFG` writer"]
pub type W = crate::W<VdSysCfgSpec>;
#[doc = "System LVD Reset Enable\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lvdre {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Lvdre> for bool {
    #[inline(always)]
    fn from(variant: Lvdre) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LVDRE` reader - System LVD Reset Enable"]
pub type LvdreR = crate::BitReader<Lvdre>;
impl LvdreR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lvdre {
        match self.bits {
            false => Lvdre::Disable,
            true => Lvdre::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Lvdre::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Lvdre::Enable
    }
}
#[doc = "Field `LVDRE` writer - System LVD Reset Enable"]
pub type LvdreW<'a, REG> = crate::BitWriter<'a, REG, Lvdre>;
impl<'a, REG> LvdreW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Lvdre::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Lvdre::Enable)
    }
}
#[doc = "System LVD Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lvdie {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Lvdie> for bool {
    #[inline(always)]
    fn from(variant: Lvdie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LVDIE` reader - System LVD Interrupt Enable"]
pub type LvdieR = crate::BitReader<Lvdie>;
impl LvdieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lvdie {
        match self.bits {
            false => Lvdie::Disable,
            true => Lvdie::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Lvdie::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Lvdie::Enable
    }
}
#[doc = "Field `LVDIE` writer - System LVD Interrupt Enable"]
pub type LvdieW<'a, REG> = crate::BitWriter<'a, REG, Lvdie>;
impl<'a, REG> LvdieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Lvdie::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Lvdie::Enable)
    }
}
#[doc = "System HVD Reset Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hvdre {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Hvdre> for bool {
    #[inline(always)]
    fn from(variant: Hvdre) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HVDRE` reader - System HVD Reset Enable"]
pub type HvdreR = crate::BitReader<Hvdre>;
impl HvdreR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Hvdre {
        match self.bits {
            false => Hvdre::Disable,
            true => Hvdre::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Hvdre::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Hvdre::Enable
    }
}
#[doc = "Field `HVDRE` writer - System HVD Reset Enable"]
pub type HvdreW<'a, REG> = crate::BitWriter<'a, REG, Hvdre>;
impl<'a, REG> HvdreW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Hvdre::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Hvdre::Enable)
    }
}
#[doc = "System HVD Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hvdie {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Hvdie> for bool {
    #[inline(always)]
    fn from(variant: Hvdie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HVDIE` reader - System HVD Interrupt Enable"]
pub type HvdieR = crate::BitReader<Hvdie>;
impl HvdieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Hvdie {
        match self.bits {
            false => Hvdie::Disable,
            true => Hvdie::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Hvdie::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Hvdie::Enable
    }
}
#[doc = "Field `HVDIE` writer - System HVD Interrupt Enable"]
pub type HvdieW<'a, REG> = crate::BitWriter<'a, REG, Hvdie>;
impl<'a, REG> HvdieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Hvdie::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Hvdie::Enable)
    }
}
#[doc = "System Voltage Detect Reset Enable Lock\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lock {
    #[doc = "0: Allow"]
    Allow = 0,
    #[doc = "1: Deny"]
    Deny = 1,
}
impl From<Lock> for bool {
    #[inline(always)]
    fn from(variant: Lock) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LOCK` reader - System Voltage Detect Reset Enable Lock"]
pub type LockR = crate::BitReader<Lock>;
impl LockR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lock {
        match self.bits {
            false => Lock::Allow,
            true => Lock::Deny,
        }
    }
    #[doc = "Allow"]
    #[inline(always)]
    pub fn is_allow(&self) -> bool {
        *self == Lock::Allow
    }
    #[doc = "Deny"]
    #[inline(always)]
    pub fn is_deny(&self) -> bool {
        *self == Lock::Deny
    }
}
#[doc = "Field `LOCK` writer - System Voltage Detect Reset Enable Lock"]
pub type LockW<'a, REG> = crate::BitWriter<'a, REG, Lock>;
impl<'a, REG> LockW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Allow"]
    #[inline(always)]
    pub fn allow(self) -> &'a mut crate::W<REG> {
        self.variant(Lock::Allow)
    }
    #[doc = "Deny"]
    #[inline(always)]
    pub fn deny(self) -> &'a mut crate::W<REG> {
        self.variant(Lock::Deny)
    }
}
impl R {
    #[doc = "Bit 0 - System LVD Reset Enable"]
    #[inline(always)]
    pub fn lvdre(&self) -> LvdreR {
        LvdreR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - System LVD Interrupt Enable"]
    #[inline(always)]
    pub fn lvdie(&self) -> LvdieR {
        LvdieR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - System HVD Reset Enable"]
    #[inline(always)]
    pub fn hvdre(&self) -> HvdreR {
        HvdreR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - System HVD Interrupt Enable"]
    #[inline(always)]
    pub fn hvdie(&self) -> HvdieR {
        HvdieR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 16 - System Voltage Detect Reset Enable Lock"]
    #[inline(always)]
    pub fn lock(&self) -> LockR {
        LockR::new(((self.bits >> 16) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - System LVD Reset Enable"]
    #[inline(always)]
    pub fn lvdre(&mut self) -> LvdreW<VdSysCfgSpec> {
        LvdreW::new(self, 0)
    }
    #[doc = "Bit 1 - System LVD Interrupt Enable"]
    #[inline(always)]
    pub fn lvdie(&mut self) -> LvdieW<VdSysCfgSpec> {
        LvdieW::new(self, 1)
    }
    #[doc = "Bit 2 - System HVD Reset Enable"]
    #[inline(always)]
    pub fn hvdre(&mut self) -> HvdreW<VdSysCfgSpec> {
        HvdreW::new(self, 2)
    }
    #[doc = "Bit 3 - System HVD Interrupt Enable"]
    #[inline(always)]
    pub fn hvdie(&mut self) -> HvdieW<VdSysCfgSpec> {
        HvdieW::new(self, 3)
    }
    #[doc = "Bit 16 - System Voltage Detect Reset Enable Lock"]
    #[inline(always)]
    pub fn lock(&mut self) -> LockW<VdSysCfgSpec> {
        LockW::new(self, 16)
    }
}
#[doc = "System Voltage Detect Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`vd_sys_cfg::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`vd_sys_cfg::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct VdSysCfgSpec;
impl crate::RegisterSpec for VdSysCfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`vd_sys_cfg::R`](R) reader structure"]
impl crate::Readable for VdSysCfgSpec {}
#[doc = "`write(|w| ..)` method takes [`vd_sys_cfg::W`](W) writer structure"]
impl crate::Writable for VdSysCfgSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets VD_SYS_CFG to value 0x01"]
impl crate::Resettable for VdSysCfgSpec {
    const RESET_VALUE: u32 = 0x01;
}

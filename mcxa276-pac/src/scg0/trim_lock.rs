#[doc = "Register `TRIM_LOCK` reader"]
pub type R = crate::R<TrimLockSpec>;
#[doc = "Register `TRIM_LOCK` writer"]
pub type W = crate::W<TrimLockSpec>;
#[doc = "TRIM_UNLOCK\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrimUnlock {
    #[doc = "0: SCG Trim Registers locked and not writable."]
    Locked = 0,
    #[doc = "1: SCG Trim registers unlocked and writable."]
    NotLocked = 1,
}
impl From<TrimUnlock> for bool {
    #[inline(always)]
    fn from(variant: TrimUnlock) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TRIM_UNLOCK` reader - TRIM_UNLOCK"]
pub type TrimUnlockR = crate::BitReader<TrimUnlock>;
impl TrimUnlockR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> TrimUnlock {
        match self.bits {
            false => TrimUnlock::Locked,
            true => TrimUnlock::NotLocked,
        }
    }
    #[doc = "SCG Trim Registers locked and not writable."]
    #[inline(always)]
    pub fn is_locked(&self) -> bool {
        *self == TrimUnlock::Locked
    }
    #[doc = "SCG Trim registers unlocked and writable."]
    #[inline(always)]
    pub fn is_not_locked(&self) -> bool {
        *self == TrimUnlock::NotLocked
    }
}
#[doc = "Field `TRIM_UNLOCK` writer - TRIM_UNLOCK"]
pub type TrimUnlockW<'a, REG> = crate::BitWriter<'a, REG, TrimUnlock>;
impl<'a, REG> TrimUnlockW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "SCG Trim Registers locked and not writable."]
    #[inline(always)]
    pub fn locked(self) -> &'a mut crate::W<REG> {
        self.variant(TrimUnlock::Locked)
    }
    #[doc = "SCG Trim registers unlocked and writable."]
    #[inline(always)]
    pub fn not_locked(self) -> &'a mut crate::W<REG> {
        self.variant(TrimUnlock::NotLocked)
    }
}
#[doc = "IFR_DISABLE\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfrDisable {
    #[doc = "0: IFR write access to SCG trim registers not disabled. The SCG Trim registers are reprogrammed with the IFR values after any system reset."]
    Enabled = 0,
    #[doc = "1: IFR write access to SCG trim registers during system reset is blocked."]
    Disabled = 1,
}
impl From<IfrDisable> for bool {
    #[inline(always)]
    fn from(variant: IfrDisable) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `IFR_DISABLE` reader - IFR_DISABLE"]
pub type IfrDisableR = crate::BitReader<IfrDisable>;
impl IfrDisableR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> IfrDisable {
        match self.bits {
            false => IfrDisable::Enabled,
            true => IfrDisable::Disabled,
        }
    }
    #[doc = "IFR write access to SCG trim registers not disabled. The SCG Trim registers are reprogrammed with the IFR values after any system reset."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == IfrDisable::Enabled
    }
    #[doc = "IFR write access to SCG trim registers during system reset is blocked."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == IfrDisable::Disabled
    }
}
#[doc = "Field `IFR_DISABLE` writer - IFR_DISABLE"]
pub type IfrDisableW<'a, REG> = crate::BitWriter<'a, REG, IfrDisable>;
impl<'a, REG> IfrDisableW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "IFR write access to SCG trim registers not disabled. The SCG Trim registers are reprogrammed with the IFR values after any system reset."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(IfrDisable::Enabled)
    }
    #[doc = "IFR write access to SCG trim registers during system reset is blocked."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(IfrDisable::Disabled)
    }
}
#[doc = "Field `TRIM_LOCK_KEY` reader - TRIM_LOCK_KEY"]
pub type TrimLockKeyR = crate::FieldReader<u16>;
#[doc = "Field `TRIM_LOCK_KEY` writer - TRIM_LOCK_KEY"]
pub type TrimLockKeyW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bit 0 - TRIM_UNLOCK"]
    #[inline(always)]
    pub fn trim_unlock(&self) -> TrimUnlockR {
        TrimUnlockR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - IFR_DISABLE"]
    #[inline(always)]
    pub fn ifr_disable(&self) -> IfrDisableR {
        IfrDisableR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bits 16:31 - TRIM_LOCK_KEY"]
    #[inline(always)]
    pub fn trim_lock_key(&self) -> TrimLockKeyR {
        TrimLockKeyR::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bit 0 - TRIM_UNLOCK"]
    #[inline(always)]
    pub fn trim_unlock(&mut self) -> TrimUnlockW<TrimLockSpec> {
        TrimUnlockW::new(self, 0)
    }
    #[doc = "Bit 1 - IFR_DISABLE"]
    #[inline(always)]
    pub fn ifr_disable(&mut self) -> IfrDisableW<TrimLockSpec> {
        IfrDisableW::new(self, 1)
    }
    #[doc = "Bits 16:31 - TRIM_LOCK_KEY"]
    #[inline(always)]
    pub fn trim_lock_key(&mut self) -> TrimLockKeyW<TrimLockSpec> {
        TrimLockKeyW::new(self, 16)
    }
}
#[doc = "Trim Lock register\n\nYou can [`read`](crate::Reg::read) this register and get [`trim_lock::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`trim_lock::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TrimLockSpec;
impl crate::RegisterSpec for TrimLockSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`trim_lock::R`](R) reader structure"]
impl crate::Readable for TrimLockSpec {}
#[doc = "`write(|w| ..)` method takes [`trim_lock::W`](W) writer structure"]
impl crate::Writable for TrimLockSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TRIM_LOCK to value 0"]
impl crate::Resettable for TrimLockSpec {}

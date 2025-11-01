#[doc = "Register `STATUS` reader"]
pub type R = crate::R<StatusSpec>;
#[doc = "Interrupt Status.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntStatus {
    #[doc = "0: No effect"]
    Disable = 0,
    #[doc = "1: Triggers interrupt"]
    Enable = 1,
}
impl From<IntStatus> for bool {
    #[inline(always)]
    fn from(variant: IntStatus) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT_STATUS` reader - Interrupt Status."]
pub type IntStatusR = crate::BitReader<IntStatus>;
impl IntStatusR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> IntStatus {
        match self.bits {
            false => IntStatus::Disable,
            true => IntStatus::Enable,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == IntStatus::Disable
    }
    #[doc = "Triggers interrupt"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == IntStatus::Enable
    }
}
#[doc = "Provides the current lock status of indexes.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockStatus {
    #[doc = "0: Current read index is not locked"]
    Lock0 = 0,
    #[doc = "1: Current read index is locked"]
    Lock1 = 1,
}
impl From<LockStatus> for bool {
    #[inline(always)]
    fn from(variant: LockStatus) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LOCK_STATUS` reader - Provides the current lock status of indexes."]
pub type LockStatusR = crate::BitReader<LockStatus>;
impl LockStatusR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> LockStatus {
        match self.bits {
            false => LockStatus::Lock0,
            true => LockStatus::Lock1,
        }
    }
    #[doc = "Current read index is not locked"]
    #[inline(always)]
    pub fn is_lock0(&self) -> bool {
        *self == LockStatus::Lock0
    }
    #[doc = "Current read index is locked"]
    #[inline(always)]
    pub fn is_lock1(&self) -> bool {
        *self == LockStatus::Lock1
    }
}
#[doc = "Status of the Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ErrorStatus {
    #[doc = "0: No error"]
    Stat0 = 0,
    #[doc = "1: FSM error has occurred"]
    Stat1 = 1,
    #[doc = "2: Write index out of the bound (OOB) error"]
    Stat2 = 2,
    #[doc = "3: Write index OOB and FSM error"]
    Stat3 = 3,
    #[doc = "4: Read index OOB error"]
    Stat4 = 4,
    #[doc = "6: Write index and read index OOB error"]
    Stat5 = 6,
    #[doc = "7: Read index OOB, write index OOB, and FSM error"]
    Stat6 = 7,
}
impl From<ErrorStatus> for u8 {
    #[inline(always)]
    fn from(variant: ErrorStatus) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for ErrorStatus {
    type Ux = u8;
}
impl crate::IsEnum for ErrorStatus {}
#[doc = "Field `ERROR_STATUS` reader - Status of the Error"]
pub type ErrorStatusR = crate::FieldReader<ErrorStatus>;
impl ErrorStatusR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<ErrorStatus> {
        match self.bits {
            0 => Some(ErrorStatus::Stat0),
            1 => Some(ErrorStatus::Stat1),
            2 => Some(ErrorStatus::Stat2),
            3 => Some(ErrorStatus::Stat3),
            4 => Some(ErrorStatus::Stat4),
            6 => Some(ErrorStatus::Stat5),
            7 => Some(ErrorStatus::Stat6),
            _ => None,
        }
    }
    #[doc = "No error"]
    #[inline(always)]
    pub fn is_stat0(&self) -> bool {
        *self == ErrorStatus::Stat0
    }
    #[doc = "FSM error has occurred"]
    #[inline(always)]
    pub fn is_stat1(&self) -> bool {
        *self == ErrorStatus::Stat1
    }
    #[doc = "Write index out of the bound (OOB) error"]
    #[inline(always)]
    pub fn is_stat2(&self) -> bool {
        *self == ErrorStatus::Stat2
    }
    #[doc = "Write index OOB and FSM error"]
    #[inline(always)]
    pub fn is_stat3(&self) -> bool {
        *self == ErrorStatus::Stat3
    }
    #[doc = "Read index OOB error"]
    #[inline(always)]
    pub fn is_stat4(&self) -> bool {
        *self == ErrorStatus::Stat4
    }
    #[doc = "Write index and read index OOB error"]
    #[inline(always)]
    pub fn is_stat5(&self) -> bool {
        *self == ErrorStatus::Stat5
    }
    #[doc = "Read index OOB, write index OOB, and FSM error"]
    #[inline(always)]
    pub fn is_stat6(&self) -> bool {
        *self == ErrorStatus::Stat6
    }
}
#[doc = "Field `RESERVED18` reader - Reserved for Future Use"]
pub type Reserved18R = crate::FieldReader<u16>;
#[doc = "Field `FSM_STATE` reader - Status of FSM"]
pub type FsmStateR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bit 0 - Interrupt Status."]
    #[inline(always)]
    pub fn int_status(&self) -> IntStatusR {
        IntStatusR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Provides the current lock status of indexes."]
    #[inline(always)]
    pub fn lock_status(&self) -> LockStatusR {
        LockStatusR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bits 2:4 - Status of the Error"]
    #[inline(always)]
    pub fn error_status(&self) -> ErrorStatusR {
        ErrorStatusR::new(((self.bits >> 2) & 7) as u8)
    }
    #[doc = "Bits 5:18 - Reserved for Future Use"]
    #[inline(always)]
    pub fn reserved18(&self) -> Reserved18R {
        Reserved18R::new(((self.bits >> 5) & 0x3fff) as u16)
    }
    #[doc = "Bits 19:31 - Status of FSM"]
    #[inline(always)]
    pub fn fsm_state(&self) -> FsmStateR {
        FsmStateR::new(((self.bits >> 19) & 0x1fff) as u16)
    }
}
#[doc = "Status\n\nYou can [`read`](crate::Reg::read) this register and get [`status::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct StatusSpec;
impl crate::RegisterSpec for StatusSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`status::R`](R) reader structure"]
impl crate::Readable for StatusSpec {}
#[doc = "`reset()` method sets STATUS to value 0x00b0_0000"]
impl crate::Resettable for StatusSpec {
    const RESET_VALUE: u32 = 0x00b0_0000;
}

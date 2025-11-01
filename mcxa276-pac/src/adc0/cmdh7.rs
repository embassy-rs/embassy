#[doc = "Register `CMDH7` reader"]
pub type R = crate::R<Cmdh7Spec>;
#[doc = "Register `CMDH7` writer"]
pub type W = crate::W<Cmdh7Spec>;
#[doc = "Compare Function Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Cmpen {
    #[doc = "0: Compare disabled."]
    DisabledAlwaysStoreResult = 0,
    #[doc = "2: Compare enabled. Store on true."]
    CompareResultStoreIfTrue = 2,
    #[doc = "3: Compare enabled. Repeat channel acquisition (sample/convert/compare) until true."]
    CompareResultKeepConvertingUntilTrueStoreIfTrue = 3,
}
impl From<Cmpen> for u8 {
    #[inline(always)]
    fn from(variant: Cmpen) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Cmpen {
    type Ux = u8;
}
impl crate::IsEnum for Cmpen {}
#[doc = "Field `CMPEN` reader - Compare Function Enable"]
pub type CmpenR = crate::FieldReader<Cmpen>;
impl CmpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Cmpen> {
        match self.bits {
            0 => Some(Cmpen::DisabledAlwaysStoreResult),
            2 => Some(Cmpen::CompareResultStoreIfTrue),
            3 => Some(Cmpen::CompareResultKeepConvertingUntilTrueStoreIfTrue),
            _ => None,
        }
    }
    #[doc = "Compare disabled."]
    #[inline(always)]
    pub fn is_disabled_always_store_result(&self) -> bool {
        *self == Cmpen::DisabledAlwaysStoreResult
    }
    #[doc = "Compare enabled. Store on true."]
    #[inline(always)]
    pub fn is_compare_result_store_if_true(&self) -> bool {
        *self == Cmpen::CompareResultStoreIfTrue
    }
    #[doc = "Compare enabled. Repeat channel acquisition (sample/convert/compare) until true."]
    #[inline(always)]
    pub fn is_compare_result_keep_converting_until_true_store_if_true(&self) -> bool {
        *self == Cmpen::CompareResultKeepConvertingUntilTrueStoreIfTrue
    }
}
#[doc = "Field `CMPEN` writer - Compare Function Enable"]
pub type CmpenW<'a, REG> = crate::FieldWriter<'a, REG, 2, Cmpen>;
impl<'a, REG> CmpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Compare disabled."]
    #[inline(always)]
    pub fn disabled_always_store_result(self) -> &'a mut crate::W<REG> {
        self.variant(Cmpen::DisabledAlwaysStoreResult)
    }
    #[doc = "Compare enabled. Store on true."]
    #[inline(always)]
    pub fn compare_result_store_if_true(self) -> &'a mut crate::W<REG> {
        self.variant(Cmpen::CompareResultStoreIfTrue)
    }
    #[doc = "Compare enabled. Repeat channel acquisition (sample/convert/compare) until true."]
    #[inline(always)]
    pub fn compare_result_keep_converting_until_true_store_if_true(self) -> &'a mut crate::W<REG> {
        self.variant(Cmpen::CompareResultKeepConvertingUntilTrueStoreIfTrue)
    }
}
#[doc = "Wait for Trigger Assertion before Execution.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaitTrig {
    #[doc = "0: This command will be automatically executed."]
    Disabled = 0,
    #[doc = "1: The active trigger must be asserted again before executing this command."]
    Enabled = 1,
}
impl From<WaitTrig> for bool {
    #[inline(always)]
    fn from(variant: WaitTrig) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WAIT_TRIG` reader - Wait for Trigger Assertion before Execution."]
pub type WaitTrigR = crate::BitReader<WaitTrig>;
impl WaitTrigR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> WaitTrig {
        match self.bits {
            false => WaitTrig::Disabled,
            true => WaitTrig::Enabled,
        }
    }
    #[doc = "This command will be automatically executed."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == WaitTrig::Disabled
    }
    #[doc = "The active trigger must be asserted again before executing this command."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == WaitTrig::Enabled
    }
}
#[doc = "Field `WAIT_TRIG` writer - Wait for Trigger Assertion before Execution."]
pub type WaitTrigW<'a, REG> = crate::BitWriter<'a, REG, WaitTrig>;
impl<'a, REG> WaitTrigW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "This command will be automatically executed."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(WaitTrig::Disabled)
    }
    #[doc = "The active trigger must be asserted again before executing this command."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(WaitTrig::Enabled)
    }
}
#[doc = "Loop with Increment\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lwi {
    #[doc = "0: Auto channel increment disabled"]
    Disabled = 0,
    #[doc = "1: Auto channel increment enabled"]
    Enabled = 1,
}
impl From<Lwi> for bool {
    #[inline(always)]
    fn from(variant: Lwi) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LWI` reader - Loop with Increment"]
pub type LwiR = crate::BitReader<Lwi>;
impl LwiR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lwi {
        match self.bits {
            false => Lwi::Disabled,
            true => Lwi::Enabled,
        }
    }
    #[doc = "Auto channel increment disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Lwi::Disabled
    }
    #[doc = "Auto channel increment enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Lwi::Enabled
    }
}
#[doc = "Field `LWI` writer - Loop with Increment"]
pub type LwiW<'a, REG> = crate::BitWriter<'a, REG, Lwi>;
impl<'a, REG> LwiW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Auto channel increment disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lwi::Disabled)
    }
    #[doc = "Auto channel increment enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lwi::Enabled)
    }
}
#[doc = "Sample Time Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Sts {
    #[doc = "0: Minimum sample time of 3.5 ADCK cycles."]
    Sample3p5 = 0,
    #[doc = "1: 3.5 + 21 ADCK cycles; 5.5 ADCK cycles total sample time."]
    Sample5p5 = 1,
    #[doc = "2: 3.5 + 22 ADCK cycles; 7.5 ADCK cycles total sample time."]
    Sample7p5 = 2,
    #[doc = "3: 3.5 + 23 ADCK cycles; 11.5 ADCK cycles total sample time."]
    Sample11p5 = 3,
    #[doc = "4: 3.5 + 24 ADCK cycles; 19.5 ADCK cycles total sample time."]
    Sample19p5 = 4,
    #[doc = "5: 3.5 + 25 ADCK cycles; 35.5 ADCK cycles total sample time."]
    Sample35p5 = 5,
    #[doc = "6: 3.5 + 26 ADCK cycles; 67.5 ADCK cycles total sample time."]
    Sample67p5 = 6,
    #[doc = "7: 3.5 + 27 ADCK cycles; 131.5 ADCK cycles total sample time."]
    Sample131p5 = 7,
}
impl From<Sts> for u8 {
    #[inline(always)]
    fn from(variant: Sts) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Sts {
    type Ux = u8;
}
impl crate::IsEnum for Sts {}
#[doc = "Field `STS` reader - Sample Time Select"]
pub type StsR = crate::FieldReader<Sts>;
impl StsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sts {
        match self.bits {
            0 => Sts::Sample3p5,
            1 => Sts::Sample5p5,
            2 => Sts::Sample7p5,
            3 => Sts::Sample11p5,
            4 => Sts::Sample19p5,
            5 => Sts::Sample35p5,
            6 => Sts::Sample67p5,
            7 => Sts::Sample131p5,
            _ => unreachable!(),
        }
    }
    #[doc = "Minimum sample time of 3.5 ADCK cycles."]
    #[inline(always)]
    pub fn is_sample_3p5(&self) -> bool {
        *self == Sts::Sample3p5
    }
    #[doc = "3.5 + 21 ADCK cycles; 5.5 ADCK cycles total sample time."]
    #[inline(always)]
    pub fn is_sample_5p5(&self) -> bool {
        *self == Sts::Sample5p5
    }
    #[doc = "3.5 + 22 ADCK cycles; 7.5 ADCK cycles total sample time."]
    #[inline(always)]
    pub fn is_sample_7p5(&self) -> bool {
        *self == Sts::Sample7p5
    }
    #[doc = "3.5 + 23 ADCK cycles; 11.5 ADCK cycles total sample time."]
    #[inline(always)]
    pub fn is_sample_11p5(&self) -> bool {
        *self == Sts::Sample11p5
    }
    #[doc = "3.5 + 24 ADCK cycles; 19.5 ADCK cycles total sample time."]
    #[inline(always)]
    pub fn is_sample_19p5(&self) -> bool {
        *self == Sts::Sample19p5
    }
    #[doc = "3.5 + 25 ADCK cycles; 35.5 ADCK cycles total sample time."]
    #[inline(always)]
    pub fn is_sample_35p5(&self) -> bool {
        *self == Sts::Sample35p5
    }
    #[doc = "3.5 + 26 ADCK cycles; 67.5 ADCK cycles total sample time."]
    #[inline(always)]
    pub fn is_sample_67p5(&self) -> bool {
        *self == Sts::Sample67p5
    }
    #[doc = "3.5 + 27 ADCK cycles; 131.5 ADCK cycles total sample time."]
    #[inline(always)]
    pub fn is_sample_131p5(&self) -> bool {
        *self == Sts::Sample131p5
    }
}
#[doc = "Field `STS` writer - Sample Time Select"]
pub type StsW<'a, REG> = crate::FieldWriter<'a, REG, 3, Sts, crate::Safe>;
impl<'a, REG> StsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Minimum sample time of 3.5 ADCK cycles."]
    #[inline(always)]
    pub fn sample_3p5(self) -> &'a mut crate::W<REG> {
        self.variant(Sts::Sample3p5)
    }
    #[doc = "3.5 + 21 ADCK cycles; 5.5 ADCK cycles total sample time."]
    #[inline(always)]
    pub fn sample_5p5(self) -> &'a mut crate::W<REG> {
        self.variant(Sts::Sample5p5)
    }
    #[doc = "3.5 + 22 ADCK cycles; 7.5 ADCK cycles total sample time."]
    #[inline(always)]
    pub fn sample_7p5(self) -> &'a mut crate::W<REG> {
        self.variant(Sts::Sample7p5)
    }
    #[doc = "3.5 + 23 ADCK cycles; 11.5 ADCK cycles total sample time."]
    #[inline(always)]
    pub fn sample_11p5(self) -> &'a mut crate::W<REG> {
        self.variant(Sts::Sample11p5)
    }
    #[doc = "3.5 + 24 ADCK cycles; 19.5 ADCK cycles total sample time."]
    #[inline(always)]
    pub fn sample_19p5(self) -> &'a mut crate::W<REG> {
        self.variant(Sts::Sample19p5)
    }
    #[doc = "3.5 + 25 ADCK cycles; 35.5 ADCK cycles total sample time."]
    #[inline(always)]
    pub fn sample_35p5(self) -> &'a mut crate::W<REG> {
        self.variant(Sts::Sample35p5)
    }
    #[doc = "3.5 + 26 ADCK cycles; 67.5 ADCK cycles total sample time."]
    #[inline(always)]
    pub fn sample_67p5(self) -> &'a mut crate::W<REG> {
        self.variant(Sts::Sample67p5)
    }
    #[doc = "3.5 + 27 ADCK cycles; 131.5 ADCK cycles total sample time."]
    #[inline(always)]
    pub fn sample_131p5(self) -> &'a mut crate::W<REG> {
        self.variant(Sts::Sample131p5)
    }
}
#[doc = "Hardware Average Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Avgs {
    #[doc = "0: Single conversion."]
    NoAverage = 0,
    #[doc = "1: 2 conversions averaged."]
    Average2 = 1,
    #[doc = "2: 4 conversions averaged."]
    Average4 = 2,
    #[doc = "3: 8 conversions averaged."]
    Average8 = 3,
    #[doc = "4: 16 conversions averaged."]
    Average16 = 4,
    #[doc = "5: 32 conversions averaged."]
    Average32 = 5,
    #[doc = "6: 64 conversions averaged."]
    Average64 = 6,
    #[doc = "7: 128 conversions averaged."]
    Average128 = 7,
    #[doc = "8: 256 conversions averaged."]
    Average256 = 8,
    #[doc = "9: 512 conversions averaged."]
    Average512 = 9,
    #[doc = "10: 1024 conversions averaged."]
    Average1024 = 10,
}
impl From<Avgs> for u8 {
    #[inline(always)]
    fn from(variant: Avgs) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Avgs {
    type Ux = u8;
}
impl crate::IsEnum for Avgs {}
#[doc = "Field `AVGS` reader - Hardware Average Select"]
pub type AvgsR = crate::FieldReader<Avgs>;
impl AvgsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Avgs> {
        match self.bits {
            0 => Some(Avgs::NoAverage),
            1 => Some(Avgs::Average2),
            2 => Some(Avgs::Average4),
            3 => Some(Avgs::Average8),
            4 => Some(Avgs::Average16),
            5 => Some(Avgs::Average32),
            6 => Some(Avgs::Average64),
            7 => Some(Avgs::Average128),
            8 => Some(Avgs::Average256),
            9 => Some(Avgs::Average512),
            10 => Some(Avgs::Average1024),
            _ => None,
        }
    }
    #[doc = "Single conversion."]
    #[inline(always)]
    pub fn is_no_average(&self) -> bool {
        *self == Avgs::NoAverage
    }
    #[doc = "2 conversions averaged."]
    #[inline(always)]
    pub fn is_average_2(&self) -> bool {
        *self == Avgs::Average2
    }
    #[doc = "4 conversions averaged."]
    #[inline(always)]
    pub fn is_average_4(&self) -> bool {
        *self == Avgs::Average4
    }
    #[doc = "8 conversions averaged."]
    #[inline(always)]
    pub fn is_average_8(&self) -> bool {
        *self == Avgs::Average8
    }
    #[doc = "16 conversions averaged."]
    #[inline(always)]
    pub fn is_average_16(&self) -> bool {
        *self == Avgs::Average16
    }
    #[doc = "32 conversions averaged."]
    #[inline(always)]
    pub fn is_average_32(&self) -> bool {
        *self == Avgs::Average32
    }
    #[doc = "64 conversions averaged."]
    #[inline(always)]
    pub fn is_average_64(&self) -> bool {
        *self == Avgs::Average64
    }
    #[doc = "128 conversions averaged."]
    #[inline(always)]
    pub fn is_average_128(&self) -> bool {
        *self == Avgs::Average128
    }
    #[doc = "256 conversions averaged."]
    #[inline(always)]
    pub fn is_average_256(&self) -> bool {
        *self == Avgs::Average256
    }
    #[doc = "512 conversions averaged."]
    #[inline(always)]
    pub fn is_average_512(&self) -> bool {
        *self == Avgs::Average512
    }
    #[doc = "1024 conversions averaged."]
    #[inline(always)]
    pub fn is_average_1024(&self) -> bool {
        *self == Avgs::Average1024
    }
}
#[doc = "Field `AVGS` writer - Hardware Average Select"]
pub type AvgsW<'a, REG> = crate::FieldWriter<'a, REG, 4, Avgs>;
impl<'a, REG> AvgsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Single conversion."]
    #[inline(always)]
    pub fn no_average(self) -> &'a mut crate::W<REG> {
        self.variant(Avgs::NoAverage)
    }
    #[doc = "2 conversions averaged."]
    #[inline(always)]
    pub fn average_2(self) -> &'a mut crate::W<REG> {
        self.variant(Avgs::Average2)
    }
    #[doc = "4 conversions averaged."]
    #[inline(always)]
    pub fn average_4(self) -> &'a mut crate::W<REG> {
        self.variant(Avgs::Average4)
    }
    #[doc = "8 conversions averaged."]
    #[inline(always)]
    pub fn average_8(self) -> &'a mut crate::W<REG> {
        self.variant(Avgs::Average8)
    }
    #[doc = "16 conversions averaged."]
    #[inline(always)]
    pub fn average_16(self) -> &'a mut crate::W<REG> {
        self.variant(Avgs::Average16)
    }
    #[doc = "32 conversions averaged."]
    #[inline(always)]
    pub fn average_32(self) -> &'a mut crate::W<REG> {
        self.variant(Avgs::Average32)
    }
    #[doc = "64 conversions averaged."]
    #[inline(always)]
    pub fn average_64(self) -> &'a mut crate::W<REG> {
        self.variant(Avgs::Average64)
    }
    #[doc = "128 conversions averaged."]
    #[inline(always)]
    pub fn average_128(self) -> &'a mut crate::W<REG> {
        self.variant(Avgs::Average128)
    }
    #[doc = "256 conversions averaged."]
    #[inline(always)]
    pub fn average_256(self) -> &'a mut crate::W<REG> {
        self.variant(Avgs::Average256)
    }
    #[doc = "512 conversions averaged."]
    #[inline(always)]
    pub fn average_512(self) -> &'a mut crate::W<REG> {
        self.variant(Avgs::Average512)
    }
    #[doc = "1024 conversions averaged."]
    #[inline(always)]
    pub fn average_1024(self) -> &'a mut crate::W<REG> {
        self.variant(Avgs::Average1024)
    }
}
#[doc = "Loop Count Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Loop {
    #[doc = "0: Looping not enabled. Command executes 1 time."]
    CmdExec1x = 0,
    #[doc = "1: Loop 1 time. Command executes 2 times."]
    CmdExec2x = 1,
    #[doc = "2: Loop 2 times. Command executes 3 times."]
    CmdExec3x = 2,
    #[doc = "3: Loop corresponding number of times. Command executes LOOP+1 times."]
    CmdExecutesCorrespondingTimes3 = 3,
    #[doc = "4: Loop corresponding number of times. Command executes LOOP+1 times."]
    CmdExecutesCorrespondingTimes4 = 4,
    #[doc = "5: Loop corresponding number of times. Command executes LOOP+1 times."]
    CmdExecutesCorrespondingTimes5 = 5,
    #[doc = "6: Loop corresponding number of times. Command executes LOOP+1 times."]
    CmdExecutesCorrespondingTimes6 = 6,
    #[doc = "7: Loop corresponding number of times. Command executes LOOP+1 times."]
    CmdExecutesCorrespondingTimes7 = 7,
    #[doc = "8: Loop corresponding number of times. Command executes LOOP+1 times."]
    CmdExecutesCorrespondingTimes8 = 8,
    #[doc = "9: Loop corresponding number of times. Command executes LOOP+1 times."]
    CmdExecutesCorrespondingTimes9 = 9,
    #[doc = "15: Loop 15 times. Command executes 16 times."]
    CmdExec15x = 15,
}
impl From<Loop> for u8 {
    #[inline(always)]
    fn from(variant: Loop) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Loop {
    type Ux = u8;
}
impl crate::IsEnum for Loop {}
#[doc = "Field `LOOP` reader - Loop Count Select"]
pub type LoopR = crate::FieldReader<Loop>;
impl LoopR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Loop> {
        match self.bits {
            0 => Some(Loop::CmdExec1x),
            1 => Some(Loop::CmdExec2x),
            2 => Some(Loop::CmdExec3x),
            3 => Some(Loop::CmdExecutesCorrespondingTimes3),
            4 => Some(Loop::CmdExecutesCorrespondingTimes4),
            5 => Some(Loop::CmdExecutesCorrespondingTimes5),
            6 => Some(Loop::CmdExecutesCorrespondingTimes6),
            7 => Some(Loop::CmdExecutesCorrespondingTimes7),
            8 => Some(Loop::CmdExecutesCorrespondingTimes8),
            9 => Some(Loop::CmdExecutesCorrespondingTimes9),
            15 => Some(Loop::CmdExec15x),
            _ => None,
        }
    }
    #[doc = "Looping not enabled. Command executes 1 time."]
    #[inline(always)]
    pub fn is_cmd_exec_1x(&self) -> bool {
        *self == Loop::CmdExec1x
    }
    #[doc = "Loop 1 time. Command executes 2 times."]
    #[inline(always)]
    pub fn is_cmd_exec_2x(&self) -> bool {
        *self == Loop::CmdExec2x
    }
    #[doc = "Loop 2 times. Command executes 3 times."]
    #[inline(always)]
    pub fn is_cmd_exec_3x(&self) -> bool {
        *self == Loop::CmdExec3x
    }
    #[doc = "Loop corresponding number of times. Command executes LOOP+1 times."]
    #[inline(always)]
    pub fn is_cmd_executes_corresponding_times_3(&self) -> bool {
        *self == Loop::CmdExecutesCorrespondingTimes3
    }
    #[doc = "Loop corresponding number of times. Command executes LOOP+1 times."]
    #[inline(always)]
    pub fn is_cmd_executes_corresponding_times_4(&self) -> bool {
        *self == Loop::CmdExecutesCorrespondingTimes4
    }
    #[doc = "Loop corresponding number of times. Command executes LOOP+1 times."]
    #[inline(always)]
    pub fn is_cmd_executes_corresponding_times_5(&self) -> bool {
        *self == Loop::CmdExecutesCorrespondingTimes5
    }
    #[doc = "Loop corresponding number of times. Command executes LOOP+1 times."]
    #[inline(always)]
    pub fn is_cmd_executes_corresponding_times_6(&self) -> bool {
        *self == Loop::CmdExecutesCorrespondingTimes6
    }
    #[doc = "Loop corresponding number of times. Command executes LOOP+1 times."]
    #[inline(always)]
    pub fn is_cmd_executes_corresponding_times_7(&self) -> bool {
        *self == Loop::CmdExecutesCorrespondingTimes7
    }
    #[doc = "Loop corresponding number of times. Command executes LOOP+1 times."]
    #[inline(always)]
    pub fn is_cmd_executes_corresponding_times_8(&self) -> bool {
        *self == Loop::CmdExecutesCorrespondingTimes8
    }
    #[doc = "Loop corresponding number of times. Command executes LOOP+1 times."]
    #[inline(always)]
    pub fn is_cmd_executes_corresponding_times_9(&self) -> bool {
        *self == Loop::CmdExecutesCorrespondingTimes9
    }
    #[doc = "Loop 15 times. Command executes 16 times."]
    #[inline(always)]
    pub fn is_cmd_exec_15x(&self) -> bool {
        *self == Loop::CmdExec15x
    }
}
#[doc = "Field `LOOP` writer - Loop Count Select"]
pub type LoopW<'a, REG> = crate::FieldWriter<'a, REG, 4, Loop>;
impl<'a, REG> LoopW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Looping not enabled. Command executes 1 time."]
    #[inline(always)]
    pub fn cmd_exec_1x(self) -> &'a mut crate::W<REG> {
        self.variant(Loop::CmdExec1x)
    }
    #[doc = "Loop 1 time. Command executes 2 times."]
    #[inline(always)]
    pub fn cmd_exec_2x(self) -> &'a mut crate::W<REG> {
        self.variant(Loop::CmdExec2x)
    }
    #[doc = "Loop 2 times. Command executes 3 times."]
    #[inline(always)]
    pub fn cmd_exec_3x(self) -> &'a mut crate::W<REG> {
        self.variant(Loop::CmdExec3x)
    }
    #[doc = "Loop corresponding number of times. Command executes LOOP+1 times."]
    #[inline(always)]
    pub fn cmd_executes_corresponding_times_3(self) -> &'a mut crate::W<REG> {
        self.variant(Loop::CmdExecutesCorrespondingTimes3)
    }
    #[doc = "Loop corresponding number of times. Command executes LOOP+1 times."]
    #[inline(always)]
    pub fn cmd_executes_corresponding_times_4(self) -> &'a mut crate::W<REG> {
        self.variant(Loop::CmdExecutesCorrespondingTimes4)
    }
    #[doc = "Loop corresponding number of times. Command executes LOOP+1 times."]
    #[inline(always)]
    pub fn cmd_executes_corresponding_times_5(self) -> &'a mut crate::W<REG> {
        self.variant(Loop::CmdExecutesCorrespondingTimes5)
    }
    #[doc = "Loop corresponding number of times. Command executes LOOP+1 times."]
    #[inline(always)]
    pub fn cmd_executes_corresponding_times_6(self) -> &'a mut crate::W<REG> {
        self.variant(Loop::CmdExecutesCorrespondingTimes6)
    }
    #[doc = "Loop corresponding number of times. Command executes LOOP+1 times."]
    #[inline(always)]
    pub fn cmd_executes_corresponding_times_7(self) -> &'a mut crate::W<REG> {
        self.variant(Loop::CmdExecutesCorrespondingTimes7)
    }
    #[doc = "Loop corresponding number of times. Command executes LOOP+1 times."]
    #[inline(always)]
    pub fn cmd_executes_corresponding_times_8(self) -> &'a mut crate::W<REG> {
        self.variant(Loop::CmdExecutesCorrespondingTimes8)
    }
    #[doc = "Loop corresponding number of times. Command executes LOOP+1 times."]
    #[inline(always)]
    pub fn cmd_executes_corresponding_times_9(self) -> &'a mut crate::W<REG> {
        self.variant(Loop::CmdExecutesCorrespondingTimes9)
    }
    #[doc = "Loop 15 times. Command executes 16 times."]
    #[inline(always)]
    pub fn cmd_exec_15x(self) -> &'a mut crate::W<REG> {
        self.variant(Loop::CmdExec15x)
    }
}
#[doc = "Next Command Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Next {
    #[doc = "0: No next command defined. Terminate conversions at completion of current command. If lower priority trigger pending, begin command associated with lower priority trigger."]
    NoNextCmdTerminateOnFinish = 0,
    #[doc = "1: Select CMD1 command buffer register as next command."]
    DoCmd1Next = 1,
    #[doc = "2: Select corresponding CMD command buffer register as next command"]
    DoCorrespondingCmdNext2 = 2,
    #[doc = "3: Select corresponding CMD command buffer register as next command"]
    DoCorrespondingCmdNext3 = 3,
    #[doc = "4: Select corresponding CMD command buffer register as next command"]
    DoCorrespondingCmdNext4 = 4,
    #[doc = "5: Select corresponding CMD command buffer register as next command"]
    DoCorrespondingCmdNext5 = 5,
    #[doc = "6: Select corresponding CMD command buffer register as next command"]
    DoCorrespondingCmdNext6 = 6,
    #[doc = "7: Select CMD7 command buffer register as next command."]
    DoCmd7Next = 7,
}
impl From<Next> for u8 {
    #[inline(always)]
    fn from(variant: Next) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Next {
    type Ux = u8;
}
impl crate::IsEnum for Next {}
#[doc = "Field `NEXT` reader - Next Command Select"]
pub type NextR = crate::FieldReader<Next>;
impl NextR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Next {
        match self.bits {
            0 => Next::NoNextCmdTerminateOnFinish,
            1 => Next::DoCmd1Next,
            2 => Next::DoCorrespondingCmdNext2,
            3 => Next::DoCorrespondingCmdNext3,
            4 => Next::DoCorrespondingCmdNext4,
            5 => Next::DoCorrespondingCmdNext5,
            6 => Next::DoCorrespondingCmdNext6,
            7 => Next::DoCmd7Next,
            _ => unreachable!(),
        }
    }
    #[doc = "No next command defined. Terminate conversions at completion of current command. If lower priority trigger pending, begin command associated with lower priority trigger."]
    #[inline(always)]
    pub fn is_no_next_cmd_terminate_on_finish(&self) -> bool {
        *self == Next::NoNextCmdTerminateOnFinish
    }
    #[doc = "Select CMD1 command buffer register as next command."]
    #[inline(always)]
    pub fn is_do_cmd1_next(&self) -> bool {
        *self == Next::DoCmd1Next
    }
    #[doc = "Select corresponding CMD command buffer register as next command"]
    #[inline(always)]
    pub fn is_do_corresponding_cmd_next_2(&self) -> bool {
        *self == Next::DoCorrespondingCmdNext2
    }
    #[doc = "Select corresponding CMD command buffer register as next command"]
    #[inline(always)]
    pub fn is_do_corresponding_cmd_next_3(&self) -> bool {
        *self == Next::DoCorrespondingCmdNext3
    }
    #[doc = "Select corresponding CMD command buffer register as next command"]
    #[inline(always)]
    pub fn is_do_corresponding_cmd_next_4(&self) -> bool {
        *self == Next::DoCorrespondingCmdNext4
    }
    #[doc = "Select corresponding CMD command buffer register as next command"]
    #[inline(always)]
    pub fn is_do_corresponding_cmd_next_5(&self) -> bool {
        *self == Next::DoCorrespondingCmdNext5
    }
    #[doc = "Select corresponding CMD command buffer register as next command"]
    #[inline(always)]
    pub fn is_do_corresponding_cmd_next_6(&self) -> bool {
        *self == Next::DoCorrespondingCmdNext6
    }
    #[doc = "Select CMD7 command buffer register as next command."]
    #[inline(always)]
    pub fn is_do_cmd7_next(&self) -> bool {
        *self == Next::DoCmd7Next
    }
}
#[doc = "Field `NEXT` writer - Next Command Select"]
pub type NextW<'a, REG> = crate::FieldWriter<'a, REG, 3, Next, crate::Safe>;
impl<'a, REG> NextW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "No next command defined. Terminate conversions at completion of current command. If lower priority trigger pending, begin command associated with lower priority trigger."]
    #[inline(always)]
    pub fn no_next_cmd_terminate_on_finish(self) -> &'a mut crate::W<REG> {
        self.variant(Next::NoNextCmdTerminateOnFinish)
    }
    #[doc = "Select CMD1 command buffer register as next command."]
    #[inline(always)]
    pub fn do_cmd1_next(self) -> &'a mut crate::W<REG> {
        self.variant(Next::DoCmd1Next)
    }
    #[doc = "Select corresponding CMD command buffer register as next command"]
    #[inline(always)]
    pub fn do_corresponding_cmd_next_2(self) -> &'a mut crate::W<REG> {
        self.variant(Next::DoCorrespondingCmdNext2)
    }
    #[doc = "Select corresponding CMD command buffer register as next command"]
    #[inline(always)]
    pub fn do_corresponding_cmd_next_3(self) -> &'a mut crate::W<REG> {
        self.variant(Next::DoCorrespondingCmdNext3)
    }
    #[doc = "Select corresponding CMD command buffer register as next command"]
    #[inline(always)]
    pub fn do_corresponding_cmd_next_4(self) -> &'a mut crate::W<REG> {
        self.variant(Next::DoCorrespondingCmdNext4)
    }
    #[doc = "Select corresponding CMD command buffer register as next command"]
    #[inline(always)]
    pub fn do_corresponding_cmd_next_5(self) -> &'a mut crate::W<REG> {
        self.variant(Next::DoCorrespondingCmdNext5)
    }
    #[doc = "Select corresponding CMD command buffer register as next command"]
    #[inline(always)]
    pub fn do_corresponding_cmd_next_6(self) -> &'a mut crate::W<REG> {
        self.variant(Next::DoCorrespondingCmdNext6)
    }
    #[doc = "Select CMD7 command buffer register as next command."]
    #[inline(always)]
    pub fn do_cmd7_next(self) -> &'a mut crate::W<REG> {
        self.variant(Next::DoCmd7Next)
    }
}
impl R {
    #[doc = "Bits 0:1 - Compare Function Enable"]
    #[inline(always)]
    pub fn cmpen(&self) -> CmpenR {
        CmpenR::new((self.bits & 3) as u8)
    }
    #[doc = "Bit 2 - Wait for Trigger Assertion before Execution."]
    #[inline(always)]
    pub fn wait_trig(&self) -> WaitTrigR {
        WaitTrigR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 7 - Loop with Increment"]
    #[inline(always)]
    pub fn lwi(&self) -> LwiR {
        LwiR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bits 8:10 - Sample Time Select"]
    #[inline(always)]
    pub fn sts(&self) -> StsR {
        StsR::new(((self.bits >> 8) & 7) as u8)
    }
    #[doc = "Bits 12:15 - Hardware Average Select"]
    #[inline(always)]
    pub fn avgs(&self) -> AvgsR {
        AvgsR::new(((self.bits >> 12) & 0x0f) as u8)
    }
    #[doc = "Bits 16:19 - Loop Count Select"]
    #[inline(always)]
    pub fn loop_(&self) -> LoopR {
        LoopR::new(((self.bits >> 16) & 0x0f) as u8)
    }
    #[doc = "Bits 24:26 - Next Command Select"]
    #[inline(always)]
    pub fn next(&self) -> NextR {
        NextR::new(((self.bits >> 24) & 7) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - Compare Function Enable"]
    #[inline(always)]
    pub fn cmpen(&mut self) -> CmpenW<Cmdh7Spec> {
        CmpenW::new(self, 0)
    }
    #[doc = "Bit 2 - Wait for Trigger Assertion before Execution."]
    #[inline(always)]
    pub fn wait_trig(&mut self) -> WaitTrigW<Cmdh7Spec> {
        WaitTrigW::new(self, 2)
    }
    #[doc = "Bit 7 - Loop with Increment"]
    #[inline(always)]
    pub fn lwi(&mut self) -> LwiW<Cmdh7Spec> {
        LwiW::new(self, 7)
    }
    #[doc = "Bits 8:10 - Sample Time Select"]
    #[inline(always)]
    pub fn sts(&mut self) -> StsW<Cmdh7Spec> {
        StsW::new(self, 8)
    }
    #[doc = "Bits 12:15 - Hardware Average Select"]
    #[inline(always)]
    pub fn avgs(&mut self) -> AvgsW<Cmdh7Spec> {
        AvgsW::new(self, 12)
    }
    #[doc = "Bits 16:19 - Loop Count Select"]
    #[inline(always)]
    pub fn loop_(&mut self) -> LoopW<Cmdh7Spec> {
        LoopW::new(self, 16)
    }
    #[doc = "Bits 24:26 - Next Command Select"]
    #[inline(always)]
    pub fn next(&mut self) -> NextW<Cmdh7Spec> {
        NextW::new(self, 24)
    }
}
#[doc = "Command High Buffer Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cmdh7::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmdh7::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Cmdh7Spec;
impl crate::RegisterSpec for Cmdh7Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`cmdh7::R`](R) reader structure"]
impl crate::Readable for Cmdh7Spec {}
#[doc = "`write(|w| ..)` method takes [`cmdh7::W`](W) writer structure"]
impl crate::Writable for Cmdh7Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CMDH7 to value 0"]
impl crate::Resettable for Cmdh7Spec {}

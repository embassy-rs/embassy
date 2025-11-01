#[doc = "Register `RESFIFO0` reader"]
pub type R = crate::R<Resfifo0Spec>;
#[doc = "Field `D` reader - Data Result"]
pub type DR = crate::FieldReader<u16>;
#[doc = "Trigger Source\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Tsrc {
    #[doc = "0: Trigger source 0 initiated this conversion."]
    Trigger0 = 0,
    #[doc = "1: Trigger source 1 initiated this conversion."]
    Trigger1 = 1,
    #[doc = "2: Trigger source 2 initiated this conversion."]
    Trigger2 = 2,
    #[doc = "3: Trigger source 3 initiated this conversion."]
    Trigger3 = 3,
}
impl From<Tsrc> for u8 {
    #[inline(always)]
    fn from(variant: Tsrc) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Tsrc {
    type Ux = u8;
}
impl crate::IsEnum for Tsrc {}
#[doc = "Field `TSRC` reader - Trigger Source"]
pub type TsrcR = crate::FieldReader<Tsrc>;
impl TsrcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tsrc {
        match self.bits {
            0 => Tsrc::Trigger0,
            1 => Tsrc::Trigger1,
            2 => Tsrc::Trigger2,
            3 => Tsrc::Trigger3,
            _ => unreachable!(),
        }
    }
    #[doc = "Trigger source 0 initiated this conversion."]
    #[inline(always)]
    pub fn is_trigger_0(&self) -> bool {
        *self == Tsrc::Trigger0
    }
    #[doc = "Trigger source 1 initiated this conversion."]
    #[inline(always)]
    pub fn is_trigger_1(&self) -> bool {
        *self == Tsrc::Trigger1
    }
    #[doc = "Trigger source 2 initiated this conversion."]
    #[inline(always)]
    pub fn is_trigger_2(&self) -> bool {
        *self == Tsrc::Trigger2
    }
    #[doc = "Trigger source 3 initiated this conversion."]
    #[inline(always)]
    pub fn is_trigger_3(&self) -> bool {
        *self == Tsrc::Trigger3
    }
}
#[doc = "Loop Count Value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Loopcnt {
    #[doc = "0: Result is from initial conversion in command."]
    Result1 = 0,
    #[doc = "1: Result is from second conversion in command."]
    Result2 = 1,
    #[doc = "2: Result is from LOOPCNT+1 conversion in command."]
    CorrespondingResult2 = 2,
    #[doc = "3: Result is from LOOPCNT+1 conversion in command."]
    CorrespondingResult3 = 3,
    #[doc = "4: Result is from LOOPCNT+1 conversion in command."]
    CorrespondingResult4 = 4,
    #[doc = "5: Result is from LOOPCNT+1 conversion in command."]
    CorrespondingResult5 = 5,
    #[doc = "6: Result is from LOOPCNT+1 conversion in command."]
    CorrespondingResult6 = 6,
    #[doc = "7: Result is from LOOPCNT+1 conversion in command."]
    CorrespondingResult7 = 7,
    #[doc = "8: Result is from LOOPCNT+1 conversion in command."]
    CorrespondingResult8 = 8,
    #[doc = "9: Result is from LOOPCNT+1 conversion in command."]
    CorrespondingResult9 = 9,
    #[doc = "15: Result is from 16th conversion in command."]
    Result16 = 15,
}
impl From<Loopcnt> for u8 {
    #[inline(always)]
    fn from(variant: Loopcnt) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Loopcnt {
    type Ux = u8;
}
impl crate::IsEnum for Loopcnt {}
#[doc = "Field `LOOPCNT` reader - Loop Count Value"]
pub type LoopcntR = crate::FieldReader<Loopcnt>;
impl LoopcntR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Loopcnt> {
        match self.bits {
            0 => Some(Loopcnt::Result1),
            1 => Some(Loopcnt::Result2),
            2 => Some(Loopcnt::CorrespondingResult2),
            3 => Some(Loopcnt::CorrespondingResult3),
            4 => Some(Loopcnt::CorrespondingResult4),
            5 => Some(Loopcnt::CorrespondingResult5),
            6 => Some(Loopcnt::CorrespondingResult6),
            7 => Some(Loopcnt::CorrespondingResult7),
            8 => Some(Loopcnt::CorrespondingResult8),
            9 => Some(Loopcnt::CorrespondingResult9),
            15 => Some(Loopcnt::Result16),
            _ => None,
        }
    }
    #[doc = "Result is from initial conversion in command."]
    #[inline(always)]
    pub fn is_result_1(&self) -> bool {
        *self == Loopcnt::Result1
    }
    #[doc = "Result is from second conversion in command."]
    #[inline(always)]
    pub fn is_result_2(&self) -> bool {
        *self == Loopcnt::Result2
    }
    #[doc = "Result is from LOOPCNT+1 conversion in command."]
    #[inline(always)]
    pub fn is_corresponding_result_2(&self) -> bool {
        *self == Loopcnt::CorrespondingResult2
    }
    #[doc = "Result is from LOOPCNT+1 conversion in command."]
    #[inline(always)]
    pub fn is_corresponding_result_3(&self) -> bool {
        *self == Loopcnt::CorrespondingResult3
    }
    #[doc = "Result is from LOOPCNT+1 conversion in command."]
    #[inline(always)]
    pub fn is_corresponding_result_4(&self) -> bool {
        *self == Loopcnt::CorrespondingResult4
    }
    #[doc = "Result is from LOOPCNT+1 conversion in command."]
    #[inline(always)]
    pub fn is_corresponding_result_5(&self) -> bool {
        *self == Loopcnt::CorrespondingResult5
    }
    #[doc = "Result is from LOOPCNT+1 conversion in command."]
    #[inline(always)]
    pub fn is_corresponding_result_6(&self) -> bool {
        *self == Loopcnt::CorrespondingResult6
    }
    #[doc = "Result is from LOOPCNT+1 conversion in command."]
    #[inline(always)]
    pub fn is_corresponding_result_7(&self) -> bool {
        *self == Loopcnt::CorrespondingResult7
    }
    #[doc = "Result is from LOOPCNT+1 conversion in command."]
    #[inline(always)]
    pub fn is_corresponding_result_8(&self) -> bool {
        *self == Loopcnt::CorrespondingResult8
    }
    #[doc = "Result is from LOOPCNT+1 conversion in command."]
    #[inline(always)]
    pub fn is_corresponding_result_9(&self) -> bool {
        *self == Loopcnt::CorrespondingResult9
    }
    #[doc = "Result is from 16th conversion in command."]
    #[inline(always)]
    pub fn is_result_16(&self) -> bool {
        *self == Loopcnt::Result16
    }
}
#[doc = "Command Buffer Source\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Cmdsrc {
    #[doc = "0: Not a valid value CMDSRC value for a dataword in RESFIFO. 0x0 is only found in initial FIFO state prior to an ADC conversion result dataword being stored to a RESFIFO buffer."]
    NotValid = 0,
    #[doc = "1: CMD1 buffer used as control settings for this conversion."]
    Cmd1 = 1,
    #[doc = "2: Corresponding command buffer used as control settings for this conversion."]
    CorrespondingCmd2 = 2,
    #[doc = "3: Corresponding command buffer used as control settings for this conversion."]
    CorrespondingCmd3 = 3,
    #[doc = "4: Corresponding command buffer used as control settings for this conversion."]
    CorrespondingCmd4 = 4,
    #[doc = "5: Corresponding command buffer used as control settings for this conversion."]
    CorrespondingCmd5 = 5,
    #[doc = "6: Corresponding command buffer used as control settings for this conversion."]
    CorrespondingCmd6 = 6,
    #[doc = "7: CMD7 buffer used as control settings for this conversion."]
    Cmd7 = 7,
}
impl From<Cmdsrc> for u8 {
    #[inline(always)]
    fn from(variant: Cmdsrc) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Cmdsrc {
    type Ux = u8;
}
impl crate::IsEnum for Cmdsrc {}
#[doc = "Field `CMDSRC` reader - Command Buffer Source"]
pub type CmdsrcR = crate::FieldReader<Cmdsrc>;
impl CmdsrcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cmdsrc {
        match self.bits {
            0 => Cmdsrc::NotValid,
            1 => Cmdsrc::Cmd1,
            2 => Cmdsrc::CorrespondingCmd2,
            3 => Cmdsrc::CorrespondingCmd3,
            4 => Cmdsrc::CorrespondingCmd4,
            5 => Cmdsrc::CorrespondingCmd5,
            6 => Cmdsrc::CorrespondingCmd6,
            7 => Cmdsrc::Cmd7,
            _ => unreachable!(),
        }
    }
    #[doc = "Not a valid value CMDSRC value for a dataword in RESFIFO. 0x0 is only found in initial FIFO state prior to an ADC conversion result dataword being stored to a RESFIFO buffer."]
    #[inline(always)]
    pub fn is_not_valid(&self) -> bool {
        *self == Cmdsrc::NotValid
    }
    #[doc = "CMD1 buffer used as control settings for this conversion."]
    #[inline(always)]
    pub fn is_cmd1(&self) -> bool {
        *self == Cmdsrc::Cmd1
    }
    #[doc = "Corresponding command buffer used as control settings for this conversion."]
    #[inline(always)]
    pub fn is_corresponding_cmd_2(&self) -> bool {
        *self == Cmdsrc::CorrespondingCmd2
    }
    #[doc = "Corresponding command buffer used as control settings for this conversion."]
    #[inline(always)]
    pub fn is_corresponding_cmd_3(&self) -> bool {
        *self == Cmdsrc::CorrespondingCmd3
    }
    #[doc = "Corresponding command buffer used as control settings for this conversion."]
    #[inline(always)]
    pub fn is_corresponding_cmd_4(&self) -> bool {
        *self == Cmdsrc::CorrespondingCmd4
    }
    #[doc = "Corresponding command buffer used as control settings for this conversion."]
    #[inline(always)]
    pub fn is_corresponding_cmd_5(&self) -> bool {
        *self == Cmdsrc::CorrespondingCmd5
    }
    #[doc = "Corresponding command buffer used as control settings for this conversion."]
    #[inline(always)]
    pub fn is_corresponding_cmd_6(&self) -> bool {
        *self == Cmdsrc::CorrespondingCmd6
    }
    #[doc = "CMD7 buffer used as control settings for this conversion."]
    #[inline(always)]
    pub fn is_cmd7(&self) -> bool {
        *self == Cmdsrc::Cmd7
    }
}
#[doc = "FIFO Entry is Valid\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Valid {
    #[doc = "0: FIFO is empty. Discard any read from RESFIFO."]
    NotValid = 0,
    #[doc = "1: FIFO record read from RESFIFO is valid."]
    Valid = 1,
}
impl From<Valid> for bool {
    #[inline(always)]
    fn from(variant: Valid) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `VALID` reader - FIFO Entry is Valid"]
pub type ValidR = crate::BitReader<Valid>;
impl ValidR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Valid {
        match self.bits {
            false => Valid::NotValid,
            true => Valid::Valid,
        }
    }
    #[doc = "FIFO is empty. Discard any read from RESFIFO."]
    #[inline(always)]
    pub fn is_not_valid(&self) -> bool {
        *self == Valid::NotValid
    }
    #[doc = "FIFO record read from RESFIFO is valid."]
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        *self == Valid::Valid
    }
}
impl R {
    #[doc = "Bits 0:15 - Data Result"]
    #[inline(always)]
    pub fn d(&self) -> DR {
        DR::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bits 16:17 - Trigger Source"]
    #[inline(always)]
    pub fn tsrc(&self) -> TsrcR {
        TsrcR::new(((self.bits >> 16) & 3) as u8)
    }
    #[doc = "Bits 20:23 - Loop Count Value"]
    #[inline(always)]
    pub fn loopcnt(&self) -> LoopcntR {
        LoopcntR::new(((self.bits >> 20) & 0x0f) as u8)
    }
    #[doc = "Bits 24:26 - Command Buffer Source"]
    #[inline(always)]
    pub fn cmdsrc(&self) -> CmdsrcR {
        CmdsrcR::new(((self.bits >> 24) & 7) as u8)
    }
    #[doc = "Bit 31 - FIFO Entry is Valid"]
    #[inline(always)]
    pub fn valid(&self) -> ValidR {
        ValidR::new(((self.bits >> 31) & 1) != 0)
    }
}
#[doc = "Data Result FIFO Register\n\nYou can [`read`](crate::Reg::read) this register and get [`resfifo0::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Resfifo0Spec;
impl crate::RegisterSpec for Resfifo0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`resfifo0::R`](R) reader structure"]
impl crate::Readable for Resfifo0Spec {}
#[doc = "`reset()` method sets RESFIFO0 to value 0"]
impl crate::Resettable for Resfifo0Spec {}

#[doc = "Register `VERID` reader"]
pub type R = crate::R<VeridSpec>;
#[doc = "Resolution\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res {
    #[doc = "0: Up to 12-bit single ended resolution supported (and 13-bit differential resolution if VERID\\[DIFFEN\\] = 1b)."]
    Max13Bit = 0,
    #[doc = "1: Up to 16-bit single ended resolution supported (and 16-bit differential resolution if VERID\\[DIFFEN\\] = 1b)."]
    Max16Bit = 1,
}
impl From<Res> for bool {
    #[inline(always)]
    fn from(variant: Res) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES` reader - Resolution"]
pub type ResR = crate::BitReader<Res>;
impl ResR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res {
        match self.bits {
            false => Res::Max13Bit,
            true => Res::Max16Bit,
        }
    }
    #[doc = "Up to 12-bit single ended resolution supported (and 13-bit differential resolution if VERID\\[DIFFEN\\] = 1b)."]
    #[inline(always)]
    pub fn is_max_13_bit(&self) -> bool {
        *self == Res::Max13Bit
    }
    #[doc = "Up to 16-bit single ended resolution supported (and 16-bit differential resolution if VERID\\[DIFFEN\\] = 1b)."]
    #[inline(always)]
    pub fn is_max_16_bit(&self) -> bool {
        *self == Res::Max16Bit
    }
}
#[doc = "Differential Supported\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Diffen {
    #[doc = "0: Differential operation not supported."]
    DifferentialNotSupported = 0,
    #[doc = "1: Differential operation supported."]
    DifferentialSupported = 1,
}
impl From<Diffen> for bool {
    #[inline(always)]
    fn from(variant: Diffen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DIFFEN` reader - Differential Supported"]
pub type DiffenR = crate::BitReader<Diffen>;
impl DiffenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Diffen {
        match self.bits {
            false => Diffen::DifferentialNotSupported,
            true => Diffen::DifferentialSupported,
        }
    }
    #[doc = "Differential operation not supported."]
    #[inline(always)]
    pub fn is_differential_not_supported(&self) -> bool {
        *self == Diffen::DifferentialNotSupported
    }
    #[doc = "Differential operation supported."]
    #[inline(always)]
    pub fn is_differential_supported(&self) -> bool {
        *self == Diffen::DifferentialSupported
    }
}
#[doc = "Multi Vref Implemented\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mvi {
    #[doc = "0: Single voltage reference high (VREFH) input supported."]
    MultipleRefNotSupported = 0,
    #[doc = "1: Multiple voltage reference high (VREFH) inputs supported."]
    MultipleRefSupported = 1,
}
impl From<Mvi> for bool {
    #[inline(always)]
    fn from(variant: Mvi) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MVI` reader - Multi Vref Implemented"]
pub type MviR = crate::BitReader<Mvi>;
impl MviR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mvi {
        match self.bits {
            false => Mvi::MultipleRefNotSupported,
            true => Mvi::MultipleRefSupported,
        }
    }
    #[doc = "Single voltage reference high (VREFH) input supported."]
    #[inline(always)]
    pub fn is_multiple_ref_not_supported(&self) -> bool {
        *self == Mvi::MultipleRefNotSupported
    }
    #[doc = "Multiple voltage reference high (VREFH) inputs supported."]
    #[inline(always)]
    pub fn is_multiple_ref_supported(&self) -> bool {
        *self == Mvi::MultipleRefSupported
    }
}
#[doc = "Channel Scale Width\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Csw {
    #[doc = "0: Channel scaling not supported."]
    CscaleNotSupported = 0,
    #[doc = "1: Channel scaling supported. 1-bit CSCALE control field."]
    BitWidth1 = 1,
    #[doc = "6: Channel scaling supported. 6-bit CSCALE control field."]
    BitWidth6 = 6,
}
impl From<Csw> for u8 {
    #[inline(always)]
    fn from(variant: Csw) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Csw {
    type Ux = u8;
}
impl crate::IsEnum for Csw {}
#[doc = "Field `CSW` reader - Channel Scale Width"]
pub type CswR = crate::FieldReader<Csw>;
impl CswR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Csw> {
        match self.bits {
            0 => Some(Csw::CscaleNotSupported),
            1 => Some(Csw::BitWidth1),
            6 => Some(Csw::BitWidth6),
            _ => None,
        }
    }
    #[doc = "Channel scaling not supported."]
    #[inline(always)]
    pub fn is_cscale_not_supported(&self) -> bool {
        *self == Csw::CscaleNotSupported
    }
    #[doc = "Channel scaling supported. 1-bit CSCALE control field."]
    #[inline(always)]
    pub fn is_bit_width_1(&self) -> bool {
        *self == Csw::BitWidth1
    }
    #[doc = "Channel scaling supported. 6-bit CSCALE control field."]
    #[inline(always)]
    pub fn is_bit_width_6(&self) -> bool {
        *self == Csw::BitWidth6
    }
}
#[doc = "Voltage Reference 1 Range Control Bit Implemented\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vr1rngi {
    #[doc = "0: Range control not required. CFG\\[VREF1RNG\\] is not implemented."]
    Ref1FixedVoltageRange = 0,
    #[doc = "1: Range control required. CFG\\[VREF1RNG\\] is implemented."]
    Ref1SelectableVoltageRange = 1,
}
impl From<Vr1rngi> for bool {
    #[inline(always)]
    fn from(variant: Vr1rngi) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `VR1RNGI` reader - Voltage Reference 1 Range Control Bit Implemented"]
pub type Vr1rngiR = crate::BitReader<Vr1rngi>;
impl Vr1rngiR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Vr1rngi {
        match self.bits {
            false => Vr1rngi::Ref1FixedVoltageRange,
            true => Vr1rngi::Ref1SelectableVoltageRange,
        }
    }
    #[doc = "Range control not required. CFG\\[VREF1RNG\\] is not implemented."]
    #[inline(always)]
    pub fn is_ref1_fixed_voltage_range(&self) -> bool {
        *self == Vr1rngi::Ref1FixedVoltageRange
    }
    #[doc = "Range control required. CFG\\[VREF1RNG\\] is implemented."]
    #[inline(always)]
    pub fn is_ref1_selectable_voltage_range(&self) -> bool {
        *self == Vr1rngi::Ref1SelectableVoltageRange
    }
}
#[doc = "Internal ADC Clock Implemented\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Iadcki {
    #[doc = "0: Internal clock source not implemented."]
    InternalClkNotAvailable = 0,
    #[doc = "1: Internal clock source (and CFG\\[ADCKEN\\]) implemented."]
    InternalClkAvailable = 1,
}
impl From<Iadcki> for bool {
    #[inline(always)]
    fn from(variant: Iadcki) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `IADCKI` reader - Internal ADC Clock Implemented"]
pub type IadckiR = crate::BitReader<Iadcki>;
impl IadckiR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Iadcki {
        match self.bits {
            false => Iadcki::InternalClkNotAvailable,
            true => Iadcki::InternalClkAvailable,
        }
    }
    #[doc = "Internal clock source not implemented."]
    #[inline(always)]
    pub fn is_internal_clk_not_available(&self) -> bool {
        *self == Iadcki::InternalClkNotAvailable
    }
    #[doc = "Internal clock source (and CFG\\[ADCKEN\\]) implemented."]
    #[inline(always)]
    pub fn is_internal_clk_available(&self) -> bool {
        *self == Iadcki::InternalClkAvailable
    }
}
#[doc = "Calibration Function Implemented\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Calofsi {
    #[doc = "0: Calibration Not Implemented."]
    CalFunctionNotAvailable = 0,
    #[doc = "1: Calibration Implemented."]
    CalFunctionAvailable = 1,
}
impl From<Calofsi> for bool {
    #[inline(always)]
    fn from(variant: Calofsi) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CALOFSI` reader - Calibration Function Implemented"]
pub type CalofsiR = crate::BitReader<Calofsi>;
impl CalofsiR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Calofsi {
        match self.bits {
            false => Calofsi::CalFunctionNotAvailable,
            true => Calofsi::CalFunctionAvailable,
        }
    }
    #[doc = "Calibration Not Implemented."]
    #[inline(always)]
    pub fn is_cal_function_not_available(&self) -> bool {
        *self == Calofsi::CalFunctionNotAvailable
    }
    #[doc = "Calibration Implemented."]
    #[inline(always)]
    pub fn is_cal_function_available(&self) -> bool {
        *self == Calofsi::CalFunctionAvailable
    }
}
#[doc = "Number of Single Ended Outputs Supported\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumSec {
    #[doc = "0: This design supports one single ended conversion at a time."]
    SingleConvertor = 0,
    #[doc = "1: This design supports two simultaneous single ended conversions."]
    DualConvertor = 1,
}
impl From<NumSec> for bool {
    #[inline(always)]
    fn from(variant: NumSec) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NUM_SEC` reader - Number of Single Ended Outputs Supported"]
pub type NumSecR = crate::BitReader<NumSec>;
impl NumSecR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> NumSec {
        match self.bits {
            false => NumSec::SingleConvertor,
            true => NumSec::DualConvertor,
        }
    }
    #[doc = "This design supports one single ended conversion at a time."]
    #[inline(always)]
    pub fn is_single_convertor(&self) -> bool {
        *self == NumSec::SingleConvertor
    }
    #[doc = "This design supports two simultaneous single ended conversions."]
    #[inline(always)]
    pub fn is_dual_convertor(&self) -> bool {
        *self == NumSec::DualConvertor
    }
}
#[doc = "Number of FIFOs\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum NumFifo {
    #[doc = "0: N/A"]
    NoFifoImplemented = 0,
    #[doc = "1: This design supports one result FIFO."]
    Cnt1 = 1,
    #[doc = "2: This design supports two result FIFOs."]
    Cnt2 = 2,
    #[doc = "3: This design supports three result FIFOs."]
    Cnt3 = 3,
    #[doc = "4: This design supports four result FIFOs."]
    Cnt4 = 4,
}
impl From<NumFifo> for u8 {
    #[inline(always)]
    fn from(variant: NumFifo) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for NumFifo {
    type Ux = u8;
}
impl crate::IsEnum for NumFifo {}
#[doc = "Field `NUM_FIFO` reader - Number of FIFOs"]
pub type NumFifoR = crate::FieldReader<NumFifo>;
impl NumFifoR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<NumFifo> {
        match self.bits {
            0 => Some(NumFifo::NoFifoImplemented),
            1 => Some(NumFifo::Cnt1),
            2 => Some(NumFifo::Cnt2),
            3 => Some(NumFifo::Cnt3),
            4 => Some(NumFifo::Cnt4),
            _ => None,
        }
    }
    #[doc = "N/A"]
    #[inline(always)]
    pub fn is_no_fifo_implemented(&self) -> bool {
        *self == NumFifo::NoFifoImplemented
    }
    #[doc = "This design supports one result FIFO."]
    #[inline(always)]
    pub fn is_cnt_1(&self) -> bool {
        *self == NumFifo::Cnt1
    }
    #[doc = "This design supports two result FIFOs."]
    #[inline(always)]
    pub fn is_cnt_2(&self) -> bool {
        *self == NumFifo::Cnt2
    }
    #[doc = "This design supports three result FIFOs."]
    #[inline(always)]
    pub fn is_cnt_3(&self) -> bool {
        *self == NumFifo::Cnt3
    }
    #[doc = "This design supports four result FIFOs."]
    #[inline(always)]
    pub fn is_cnt_4(&self) -> bool {
        *self == NumFifo::Cnt4
    }
}
#[doc = "Field `MINOR` reader - Minor Version Number"]
pub type MinorR = crate::FieldReader;
#[doc = "Field `MAJOR` reader - Major Version Number"]
pub type MajorR = crate::FieldReader;
impl R {
    #[doc = "Bit 0 - Resolution"]
    #[inline(always)]
    pub fn res(&self) -> ResR {
        ResR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Differential Supported"]
    #[inline(always)]
    pub fn diffen(&self) -> DiffenR {
        DiffenR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 3 - Multi Vref Implemented"]
    #[inline(always)]
    pub fn mvi(&self) -> MviR {
        MviR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bits 4:6 - Channel Scale Width"]
    #[inline(always)]
    pub fn csw(&self) -> CswR {
        CswR::new(((self.bits >> 4) & 7) as u8)
    }
    #[doc = "Bit 8 - Voltage Reference 1 Range Control Bit Implemented"]
    #[inline(always)]
    pub fn vr1rngi(&self) -> Vr1rngiR {
        Vr1rngiR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Internal ADC Clock Implemented"]
    #[inline(always)]
    pub fn iadcki(&self) -> IadckiR {
        IadckiR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Calibration Function Implemented"]
    #[inline(always)]
    pub fn calofsi(&self) -> CalofsiR {
        CalofsiR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Number of Single Ended Outputs Supported"]
    #[inline(always)]
    pub fn num_sec(&self) -> NumSecR {
        NumSecR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bits 12:14 - Number of FIFOs"]
    #[inline(always)]
    pub fn num_fifo(&self) -> NumFifoR {
        NumFifoR::new(((self.bits >> 12) & 7) as u8)
    }
    #[doc = "Bits 16:23 - Minor Version Number"]
    #[inline(always)]
    pub fn minor(&self) -> MinorR {
        MinorR::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - Major Version Number"]
    #[inline(always)]
    pub fn major(&self) -> MajorR {
        MajorR::new(((self.bits >> 24) & 0xff) as u8)
    }
}
#[doc = "Version ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`verid::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct VeridSpec;
impl crate::RegisterSpec for VeridSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`verid::R`](R) reader structure"]
impl crate::Readable for VeridSpec {}
#[doc = "`reset()` method sets VERID to value 0x0200_1409"]
impl crate::Resettable for VeridSpec {
    const RESET_VALUE: u32 = 0x0200_1409;
}

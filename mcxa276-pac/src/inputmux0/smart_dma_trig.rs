#[doc = "Register `SmartDMA_TRIG[%s]` reader"]
pub type R = crate::R<SmartDmaTrigSpec>;
#[doc = "Register `SmartDMA_TRIG[%s]` writer"]
pub type W = crate::W<SmartDmaTrigSpec>;
#[doc = "Input number for SmartDMA.\n\nValue on reset: 127"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Inp {
    #[doc = "1: GPIO P0_16 input is selected"]
    Val1 = 1,
    #[doc = "2: GPIO P0_17 input is selected"]
    Val2 = 2,
    #[doc = "3: GPIO P1_8 input is selected"]
    Val3 = 3,
    #[doc = "4: GPIO P1_9 input is selected"]
    Val4 = 4,
    #[doc = "5: GPIO P1_10 input is selected"]
    Val5 = 5,
    #[doc = "6: GPIO P1_11 input is selected"]
    Val6 = 6,
    #[doc = "7: GPIO P1_12 input is selected"]
    Val7 = 7,
    #[doc = "8: GPIO P1_13 input is selected"]
    Val8 = 8,
    #[doc = "9: GPIO P2_0 input is selected"]
    Val9 = 9,
    #[doc = "10: GPIO P2_1 input is selected"]
    Val10 = 10,
    #[doc = "11: GPIO P2_2 input is selected"]
    Val11 = 11,
    #[doc = "12: GPIO P2_3 input is selected"]
    Val12 = 12,
    #[doc = "13: GPIO P2_6 input is selected"]
    Val13 = 13,
    #[doc = "14: GPIO P3_8 input is selected"]
    Val14 = 14,
    #[doc = "15: GPIO P3_9 input is selected"]
    Val15 = 15,
    #[doc = "16: GPIO P3_10 input is selected"]
    Val16 = 16,
    #[doc = "17: GPIO P3_11 input is selected"]
    Val17 = 17,
    #[doc = "18: GPIO P3_12 input is seclected"]
    Val18 = 18,
    #[doc = "19: GPIO0 Pin Event Trig input is selected"]
    Val19 = 19,
    #[doc = "20: GPIO1 Pin Event Trig input is selected"]
    Val20 = 20,
    #[doc = "21: GPIO2 Pin Event Trig input is selected"]
    Val21 = 21,
    #[doc = "22: GPIO3 Pin Event Trig input is selected"]
    Val22 = 22,
    #[doc = "23: GPIO4 Pin Event Trig input is selected"]
    Val23 = 23,
    #[doc = "24: ARM_TXEV input is selected"]
    Val24 = 24,
    #[doc = "25: AOI0_OUT0 input is selected"]
    Val25 = 25,
    #[doc = "26: AOI1_OUT1 input is selected"]
    Val26 = 26,
    #[doc = "27: DMA_IRQ input is selected"]
    Val27 = 27,
    #[doc = "28: MAU_IRQ input is selected"]
    Val28 = 28,
    #[doc = "29: WUU_IRQ input is selected"]
    Val29 = 29,
    #[doc = "30: CTimer0_MAT2 input is selected"]
    Val30 = 30,
    #[doc = "31: CTimer0_MAT3 input is selected"]
    Val31 = 31,
    #[doc = "32: CTimer1_MAT2 input is selected"]
    Val32 = 32,
    #[doc = "33: CTimer1_MAT3 input is selected"]
    Val33 = 33,
    #[doc = "34: CTimer2_MAT2 input is selected"]
    Val34 = 34,
    #[doc = "35: CTimer2_MAT3 input is selected"]
    Val35 = 35,
    #[doc = "36: CTimer3_MAT2 input is selected"]
    Val36 = 36,
    #[doc = "37: CTimer3_MAT3 input is selected"]
    Val37 = 37,
    #[doc = "38: CTimer4_MAT2 input is selected"]
    Val38 = 38,
    #[doc = "39: CTimer4_MAT3 input is selected"]
    Val39 = 39,
    #[doc = "40: OSTIMER_IRQ input is selected"]
    Val40 = 40,
    #[doc = "41: PWM0_IRQ input is selected"]
    Val41 = 41,
    #[doc = "42: PWM1_IRQ input is selected"]
    Val42 = 42,
    #[doc = "43: QDC0_IRQ input is selected"]
    Val43 = 43,
    #[doc = "44: QDC1_IRQ input is selected"]
    Val44 = 44,
    #[doc = "45: RTC_Alarm_IRQ input is selected"]
    Val45 = 45,
    #[doc = "46: RTC_1Hz_IRQ input is selected"]
    Val46 = 46,
    #[doc = "47: uTICK_IRQ input is selected"]
    Val47 = 47,
    #[doc = "48: WDT_IRQ input is selected"]
    Val48 = 48,
    #[doc = "49: Wakeup_Timer_IRQ input is selected"]
    Val49 = 49,
    #[doc = "50: CAN0_IRQ input is selected"]
    Val50 = 50,
    #[doc = "51: CAN1_IRQ input is selected"]
    Val51 = 51,
    #[doc = "52: FlexIO_IRQ input is selected"]
    Val52 = 52,
    #[doc = "53: FlexIO_Shifer0_DMA_Req input is selected"]
    Val53 = 53,
    #[doc = "54: FlexIO_Shifer1_DMA_Req input is selected"]
    Val54 = 54,
    #[doc = "55: FlexIO_Shifer2_DMA_Req input is selected"]
    Val55 = 55,
    #[doc = "56: FlexIO_Shifer3_DMA_Req input is selected"]
    Val56 = 56,
    #[doc = "57: I3C0_IRQ input is selected"]
    Val57 = 57,
    #[doc = "58: LPI2C0_IRQ input is selected"]
    Val58 = 58,
    #[doc = "59: LPI2C1_IRQ input is selected"]
    Val59 = 59,
    #[doc = "60: LPSPI0_IRQ input is selected"]
    Val60 = 60,
    #[doc = "61: LPSPI1_IRQ input is selected"]
    Val61 = 61,
    #[doc = "62: LPUART0_IRQ input is selected"]
    Val62 = 62,
    #[doc = "63: LPUART1_IRQ input is selected"]
    Val63 = 63,
    #[doc = "64: LPUART2_IRQ input is selected"]
    Val64 = 64,
    #[doc = "65: LPUART3_IRQ input is selected"]
    Val65 = 65,
    #[doc = "66: USB0_SOF input is selected"]
    Val66 = 66,
    #[doc = "68: ADC0_IRQ input is selected"]
    Val68 = 68,
    #[doc = "69: ADC1_IRQ input is selected"]
    Val69 = 69,
    #[doc = "70: ADC2_IRQ input is selected"]
    Val70 = 70,
    #[doc = "71: ADC3_IRQ input is selected"]
    Val71 = 71,
    #[doc = "72: CMP0_IRQ input is selected"]
    Val72 = 72,
    #[doc = "73: CMP1_IRQ input is selected"]
    Val73 = 73,
    #[doc = "74: CMP2_IRQ input is selected"]
    Val74 = 74,
    #[doc = "75: CMP0_OUT input is selected"]
    Val75 = 75,
    #[doc = "76: CMP1_OUT input is selected"]
    Val76 = 76,
    #[doc = "77: CMP2_OUT input is selected"]
    Val77 = 77,
    #[doc = "78: DAC0_IRQ input is selected"]
    Val78 = 78,
    #[doc = "79: SLCD_IRQ input is selected"]
    Val79 = 79,
}
impl From<Inp> for u8 {
    #[inline(always)]
    fn from(variant: Inp) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Inp {
    type Ux = u8;
}
impl crate::IsEnum for Inp {}
#[doc = "Field `INP` reader - Input number for SmartDMA."]
pub type InpR = crate::FieldReader<Inp>;
impl InpR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Inp> {
        match self.bits {
            1 => Some(Inp::Val1),
            2 => Some(Inp::Val2),
            3 => Some(Inp::Val3),
            4 => Some(Inp::Val4),
            5 => Some(Inp::Val5),
            6 => Some(Inp::Val6),
            7 => Some(Inp::Val7),
            8 => Some(Inp::Val8),
            9 => Some(Inp::Val9),
            10 => Some(Inp::Val10),
            11 => Some(Inp::Val11),
            12 => Some(Inp::Val12),
            13 => Some(Inp::Val13),
            14 => Some(Inp::Val14),
            15 => Some(Inp::Val15),
            16 => Some(Inp::Val16),
            17 => Some(Inp::Val17),
            18 => Some(Inp::Val18),
            19 => Some(Inp::Val19),
            20 => Some(Inp::Val20),
            21 => Some(Inp::Val21),
            22 => Some(Inp::Val22),
            23 => Some(Inp::Val23),
            24 => Some(Inp::Val24),
            25 => Some(Inp::Val25),
            26 => Some(Inp::Val26),
            27 => Some(Inp::Val27),
            28 => Some(Inp::Val28),
            29 => Some(Inp::Val29),
            30 => Some(Inp::Val30),
            31 => Some(Inp::Val31),
            32 => Some(Inp::Val32),
            33 => Some(Inp::Val33),
            34 => Some(Inp::Val34),
            35 => Some(Inp::Val35),
            36 => Some(Inp::Val36),
            37 => Some(Inp::Val37),
            38 => Some(Inp::Val38),
            39 => Some(Inp::Val39),
            40 => Some(Inp::Val40),
            41 => Some(Inp::Val41),
            42 => Some(Inp::Val42),
            43 => Some(Inp::Val43),
            44 => Some(Inp::Val44),
            45 => Some(Inp::Val45),
            46 => Some(Inp::Val46),
            47 => Some(Inp::Val47),
            48 => Some(Inp::Val48),
            49 => Some(Inp::Val49),
            50 => Some(Inp::Val50),
            51 => Some(Inp::Val51),
            52 => Some(Inp::Val52),
            53 => Some(Inp::Val53),
            54 => Some(Inp::Val54),
            55 => Some(Inp::Val55),
            56 => Some(Inp::Val56),
            57 => Some(Inp::Val57),
            58 => Some(Inp::Val58),
            59 => Some(Inp::Val59),
            60 => Some(Inp::Val60),
            61 => Some(Inp::Val61),
            62 => Some(Inp::Val62),
            63 => Some(Inp::Val63),
            64 => Some(Inp::Val64),
            65 => Some(Inp::Val65),
            66 => Some(Inp::Val66),
            68 => Some(Inp::Val68),
            69 => Some(Inp::Val69),
            70 => Some(Inp::Val70),
            71 => Some(Inp::Val71),
            72 => Some(Inp::Val72),
            73 => Some(Inp::Val73),
            74 => Some(Inp::Val74),
            75 => Some(Inp::Val75),
            76 => Some(Inp::Val76),
            77 => Some(Inp::Val77),
            78 => Some(Inp::Val78),
            79 => Some(Inp::Val79),
            _ => None,
        }
    }
    #[doc = "GPIO P0_16 input is selected"]
    #[inline(always)]
    pub fn is_val1(&self) -> bool {
        *self == Inp::Val1
    }
    #[doc = "GPIO P0_17 input is selected"]
    #[inline(always)]
    pub fn is_val2(&self) -> bool {
        *self == Inp::Val2
    }
    #[doc = "GPIO P1_8 input is selected"]
    #[inline(always)]
    pub fn is_val3(&self) -> bool {
        *self == Inp::Val3
    }
    #[doc = "GPIO P1_9 input is selected"]
    #[inline(always)]
    pub fn is_val4(&self) -> bool {
        *self == Inp::Val4
    }
    #[doc = "GPIO P1_10 input is selected"]
    #[inline(always)]
    pub fn is_val5(&self) -> bool {
        *self == Inp::Val5
    }
    #[doc = "GPIO P1_11 input is selected"]
    #[inline(always)]
    pub fn is_val6(&self) -> bool {
        *self == Inp::Val6
    }
    #[doc = "GPIO P1_12 input is selected"]
    #[inline(always)]
    pub fn is_val7(&self) -> bool {
        *self == Inp::Val7
    }
    #[doc = "GPIO P1_13 input is selected"]
    #[inline(always)]
    pub fn is_val8(&self) -> bool {
        *self == Inp::Val8
    }
    #[doc = "GPIO P2_0 input is selected"]
    #[inline(always)]
    pub fn is_val9(&self) -> bool {
        *self == Inp::Val9
    }
    #[doc = "GPIO P2_1 input is selected"]
    #[inline(always)]
    pub fn is_val10(&self) -> bool {
        *self == Inp::Val10
    }
    #[doc = "GPIO P2_2 input is selected"]
    #[inline(always)]
    pub fn is_val11(&self) -> bool {
        *self == Inp::Val11
    }
    #[doc = "GPIO P2_3 input is selected"]
    #[inline(always)]
    pub fn is_val12(&self) -> bool {
        *self == Inp::Val12
    }
    #[doc = "GPIO P2_6 input is selected"]
    #[inline(always)]
    pub fn is_val13(&self) -> bool {
        *self == Inp::Val13
    }
    #[doc = "GPIO P3_8 input is selected"]
    #[inline(always)]
    pub fn is_val14(&self) -> bool {
        *self == Inp::Val14
    }
    #[doc = "GPIO P3_9 input is selected"]
    #[inline(always)]
    pub fn is_val15(&self) -> bool {
        *self == Inp::Val15
    }
    #[doc = "GPIO P3_10 input is selected"]
    #[inline(always)]
    pub fn is_val16(&self) -> bool {
        *self == Inp::Val16
    }
    #[doc = "GPIO P3_11 input is selected"]
    #[inline(always)]
    pub fn is_val17(&self) -> bool {
        *self == Inp::Val17
    }
    #[doc = "GPIO P3_12 input is seclected"]
    #[inline(always)]
    pub fn is_val18(&self) -> bool {
        *self == Inp::Val18
    }
    #[doc = "GPIO0 Pin Event Trig input is selected"]
    #[inline(always)]
    pub fn is_val19(&self) -> bool {
        *self == Inp::Val19
    }
    #[doc = "GPIO1 Pin Event Trig input is selected"]
    #[inline(always)]
    pub fn is_val20(&self) -> bool {
        *self == Inp::Val20
    }
    #[doc = "GPIO2 Pin Event Trig input is selected"]
    #[inline(always)]
    pub fn is_val21(&self) -> bool {
        *self == Inp::Val21
    }
    #[doc = "GPIO3 Pin Event Trig input is selected"]
    #[inline(always)]
    pub fn is_val22(&self) -> bool {
        *self == Inp::Val22
    }
    #[doc = "GPIO4 Pin Event Trig input is selected"]
    #[inline(always)]
    pub fn is_val23(&self) -> bool {
        *self == Inp::Val23
    }
    #[doc = "ARM_TXEV input is selected"]
    #[inline(always)]
    pub fn is_val24(&self) -> bool {
        *self == Inp::Val24
    }
    #[doc = "AOI0_OUT0 input is selected"]
    #[inline(always)]
    pub fn is_val25(&self) -> bool {
        *self == Inp::Val25
    }
    #[doc = "AOI1_OUT1 input is selected"]
    #[inline(always)]
    pub fn is_val26(&self) -> bool {
        *self == Inp::Val26
    }
    #[doc = "DMA_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val27(&self) -> bool {
        *self == Inp::Val27
    }
    #[doc = "MAU_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val28(&self) -> bool {
        *self == Inp::Val28
    }
    #[doc = "WUU_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val29(&self) -> bool {
        *self == Inp::Val29
    }
    #[doc = "CTimer0_MAT2 input is selected"]
    #[inline(always)]
    pub fn is_val30(&self) -> bool {
        *self == Inp::Val30
    }
    #[doc = "CTimer0_MAT3 input is selected"]
    #[inline(always)]
    pub fn is_val31(&self) -> bool {
        *self == Inp::Val31
    }
    #[doc = "CTimer1_MAT2 input is selected"]
    #[inline(always)]
    pub fn is_val32(&self) -> bool {
        *self == Inp::Val32
    }
    #[doc = "CTimer1_MAT3 input is selected"]
    #[inline(always)]
    pub fn is_val33(&self) -> bool {
        *self == Inp::Val33
    }
    #[doc = "CTimer2_MAT2 input is selected"]
    #[inline(always)]
    pub fn is_val34(&self) -> bool {
        *self == Inp::Val34
    }
    #[doc = "CTimer2_MAT3 input is selected"]
    #[inline(always)]
    pub fn is_val35(&self) -> bool {
        *self == Inp::Val35
    }
    #[doc = "CTimer3_MAT2 input is selected"]
    #[inline(always)]
    pub fn is_val36(&self) -> bool {
        *self == Inp::Val36
    }
    #[doc = "CTimer3_MAT3 input is selected"]
    #[inline(always)]
    pub fn is_val37(&self) -> bool {
        *self == Inp::Val37
    }
    #[doc = "CTimer4_MAT2 input is selected"]
    #[inline(always)]
    pub fn is_val38(&self) -> bool {
        *self == Inp::Val38
    }
    #[doc = "CTimer4_MAT3 input is selected"]
    #[inline(always)]
    pub fn is_val39(&self) -> bool {
        *self == Inp::Val39
    }
    #[doc = "OSTIMER_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val40(&self) -> bool {
        *self == Inp::Val40
    }
    #[doc = "PWM0_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val41(&self) -> bool {
        *self == Inp::Val41
    }
    #[doc = "PWM1_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val42(&self) -> bool {
        *self == Inp::Val42
    }
    #[doc = "QDC0_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val43(&self) -> bool {
        *self == Inp::Val43
    }
    #[doc = "QDC1_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val44(&self) -> bool {
        *self == Inp::Val44
    }
    #[doc = "RTC_Alarm_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val45(&self) -> bool {
        *self == Inp::Val45
    }
    #[doc = "RTC_1Hz_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val46(&self) -> bool {
        *self == Inp::Val46
    }
    #[doc = "uTICK_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val47(&self) -> bool {
        *self == Inp::Val47
    }
    #[doc = "WDT_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val48(&self) -> bool {
        *self == Inp::Val48
    }
    #[doc = "Wakeup_Timer_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val49(&self) -> bool {
        *self == Inp::Val49
    }
    #[doc = "CAN0_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val50(&self) -> bool {
        *self == Inp::Val50
    }
    #[doc = "CAN1_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val51(&self) -> bool {
        *self == Inp::Val51
    }
    #[doc = "FlexIO_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val52(&self) -> bool {
        *self == Inp::Val52
    }
    #[doc = "FlexIO_Shifer0_DMA_Req input is selected"]
    #[inline(always)]
    pub fn is_val53(&self) -> bool {
        *self == Inp::Val53
    }
    #[doc = "FlexIO_Shifer1_DMA_Req input is selected"]
    #[inline(always)]
    pub fn is_val54(&self) -> bool {
        *self == Inp::Val54
    }
    #[doc = "FlexIO_Shifer2_DMA_Req input is selected"]
    #[inline(always)]
    pub fn is_val55(&self) -> bool {
        *self == Inp::Val55
    }
    #[doc = "FlexIO_Shifer3_DMA_Req input is selected"]
    #[inline(always)]
    pub fn is_val56(&self) -> bool {
        *self == Inp::Val56
    }
    #[doc = "I3C0_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val57(&self) -> bool {
        *self == Inp::Val57
    }
    #[doc = "LPI2C0_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val58(&self) -> bool {
        *self == Inp::Val58
    }
    #[doc = "LPI2C1_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val59(&self) -> bool {
        *self == Inp::Val59
    }
    #[doc = "LPSPI0_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val60(&self) -> bool {
        *self == Inp::Val60
    }
    #[doc = "LPSPI1_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val61(&self) -> bool {
        *self == Inp::Val61
    }
    #[doc = "LPUART0_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val62(&self) -> bool {
        *self == Inp::Val62
    }
    #[doc = "LPUART1_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val63(&self) -> bool {
        *self == Inp::Val63
    }
    #[doc = "LPUART2_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val64(&self) -> bool {
        *self == Inp::Val64
    }
    #[doc = "LPUART3_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val65(&self) -> bool {
        *self == Inp::Val65
    }
    #[doc = "USB0_SOF input is selected"]
    #[inline(always)]
    pub fn is_val66(&self) -> bool {
        *self == Inp::Val66
    }
    #[doc = "ADC0_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val68(&self) -> bool {
        *self == Inp::Val68
    }
    #[doc = "ADC1_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val69(&self) -> bool {
        *self == Inp::Val69
    }
    #[doc = "ADC2_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val70(&self) -> bool {
        *self == Inp::Val70
    }
    #[doc = "ADC3_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val71(&self) -> bool {
        *self == Inp::Val71
    }
    #[doc = "CMP0_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val72(&self) -> bool {
        *self == Inp::Val72
    }
    #[doc = "CMP1_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val73(&self) -> bool {
        *self == Inp::Val73
    }
    #[doc = "CMP2_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val74(&self) -> bool {
        *self == Inp::Val74
    }
    #[doc = "CMP0_OUT input is selected"]
    #[inline(always)]
    pub fn is_val75(&self) -> bool {
        *self == Inp::Val75
    }
    #[doc = "CMP1_OUT input is selected"]
    #[inline(always)]
    pub fn is_val76(&self) -> bool {
        *self == Inp::Val76
    }
    #[doc = "CMP2_OUT input is selected"]
    #[inline(always)]
    pub fn is_val77(&self) -> bool {
        *self == Inp::Val77
    }
    #[doc = "DAC0_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val78(&self) -> bool {
        *self == Inp::Val78
    }
    #[doc = "SLCD_IRQ input is selected"]
    #[inline(always)]
    pub fn is_val79(&self) -> bool {
        *self == Inp::Val79
    }
}
#[doc = "Field `INP` writer - Input number for SmartDMA."]
pub type InpW<'a, REG> = crate::FieldWriter<'a, REG, 7, Inp>;
impl<'a, REG> InpW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "GPIO P0_16 input is selected"]
    #[inline(always)]
    pub fn val1(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val1)
    }
    #[doc = "GPIO P0_17 input is selected"]
    #[inline(always)]
    pub fn val2(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val2)
    }
    #[doc = "GPIO P1_8 input is selected"]
    #[inline(always)]
    pub fn val3(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val3)
    }
    #[doc = "GPIO P1_9 input is selected"]
    #[inline(always)]
    pub fn val4(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val4)
    }
    #[doc = "GPIO P1_10 input is selected"]
    #[inline(always)]
    pub fn val5(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val5)
    }
    #[doc = "GPIO P1_11 input is selected"]
    #[inline(always)]
    pub fn val6(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val6)
    }
    #[doc = "GPIO P1_12 input is selected"]
    #[inline(always)]
    pub fn val7(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val7)
    }
    #[doc = "GPIO P1_13 input is selected"]
    #[inline(always)]
    pub fn val8(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val8)
    }
    #[doc = "GPIO P2_0 input is selected"]
    #[inline(always)]
    pub fn val9(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val9)
    }
    #[doc = "GPIO P2_1 input is selected"]
    #[inline(always)]
    pub fn val10(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val10)
    }
    #[doc = "GPIO P2_2 input is selected"]
    #[inline(always)]
    pub fn val11(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val11)
    }
    #[doc = "GPIO P2_3 input is selected"]
    #[inline(always)]
    pub fn val12(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val12)
    }
    #[doc = "GPIO P2_6 input is selected"]
    #[inline(always)]
    pub fn val13(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val13)
    }
    #[doc = "GPIO P3_8 input is selected"]
    #[inline(always)]
    pub fn val14(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val14)
    }
    #[doc = "GPIO P3_9 input is selected"]
    #[inline(always)]
    pub fn val15(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val15)
    }
    #[doc = "GPIO P3_10 input is selected"]
    #[inline(always)]
    pub fn val16(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val16)
    }
    #[doc = "GPIO P3_11 input is selected"]
    #[inline(always)]
    pub fn val17(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val17)
    }
    #[doc = "GPIO P3_12 input is seclected"]
    #[inline(always)]
    pub fn val18(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val18)
    }
    #[doc = "GPIO0 Pin Event Trig input is selected"]
    #[inline(always)]
    pub fn val19(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val19)
    }
    #[doc = "GPIO1 Pin Event Trig input is selected"]
    #[inline(always)]
    pub fn val20(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val20)
    }
    #[doc = "GPIO2 Pin Event Trig input is selected"]
    #[inline(always)]
    pub fn val21(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val21)
    }
    #[doc = "GPIO3 Pin Event Trig input is selected"]
    #[inline(always)]
    pub fn val22(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val22)
    }
    #[doc = "GPIO4 Pin Event Trig input is selected"]
    #[inline(always)]
    pub fn val23(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val23)
    }
    #[doc = "ARM_TXEV input is selected"]
    #[inline(always)]
    pub fn val24(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val24)
    }
    #[doc = "AOI0_OUT0 input is selected"]
    #[inline(always)]
    pub fn val25(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val25)
    }
    #[doc = "AOI1_OUT1 input is selected"]
    #[inline(always)]
    pub fn val26(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val26)
    }
    #[doc = "DMA_IRQ input is selected"]
    #[inline(always)]
    pub fn val27(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val27)
    }
    #[doc = "MAU_IRQ input is selected"]
    #[inline(always)]
    pub fn val28(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val28)
    }
    #[doc = "WUU_IRQ input is selected"]
    #[inline(always)]
    pub fn val29(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val29)
    }
    #[doc = "CTimer0_MAT2 input is selected"]
    #[inline(always)]
    pub fn val30(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val30)
    }
    #[doc = "CTimer0_MAT3 input is selected"]
    #[inline(always)]
    pub fn val31(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val31)
    }
    #[doc = "CTimer1_MAT2 input is selected"]
    #[inline(always)]
    pub fn val32(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val32)
    }
    #[doc = "CTimer1_MAT3 input is selected"]
    #[inline(always)]
    pub fn val33(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val33)
    }
    #[doc = "CTimer2_MAT2 input is selected"]
    #[inline(always)]
    pub fn val34(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val34)
    }
    #[doc = "CTimer2_MAT3 input is selected"]
    #[inline(always)]
    pub fn val35(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val35)
    }
    #[doc = "CTimer3_MAT2 input is selected"]
    #[inline(always)]
    pub fn val36(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val36)
    }
    #[doc = "CTimer3_MAT3 input is selected"]
    #[inline(always)]
    pub fn val37(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val37)
    }
    #[doc = "CTimer4_MAT2 input is selected"]
    #[inline(always)]
    pub fn val38(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val38)
    }
    #[doc = "CTimer4_MAT3 input is selected"]
    #[inline(always)]
    pub fn val39(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val39)
    }
    #[doc = "OSTIMER_IRQ input is selected"]
    #[inline(always)]
    pub fn val40(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val40)
    }
    #[doc = "PWM0_IRQ input is selected"]
    #[inline(always)]
    pub fn val41(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val41)
    }
    #[doc = "PWM1_IRQ input is selected"]
    #[inline(always)]
    pub fn val42(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val42)
    }
    #[doc = "QDC0_IRQ input is selected"]
    #[inline(always)]
    pub fn val43(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val43)
    }
    #[doc = "QDC1_IRQ input is selected"]
    #[inline(always)]
    pub fn val44(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val44)
    }
    #[doc = "RTC_Alarm_IRQ input is selected"]
    #[inline(always)]
    pub fn val45(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val45)
    }
    #[doc = "RTC_1Hz_IRQ input is selected"]
    #[inline(always)]
    pub fn val46(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val46)
    }
    #[doc = "uTICK_IRQ input is selected"]
    #[inline(always)]
    pub fn val47(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val47)
    }
    #[doc = "WDT_IRQ input is selected"]
    #[inline(always)]
    pub fn val48(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val48)
    }
    #[doc = "Wakeup_Timer_IRQ input is selected"]
    #[inline(always)]
    pub fn val49(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val49)
    }
    #[doc = "CAN0_IRQ input is selected"]
    #[inline(always)]
    pub fn val50(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val50)
    }
    #[doc = "CAN1_IRQ input is selected"]
    #[inline(always)]
    pub fn val51(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val51)
    }
    #[doc = "FlexIO_IRQ input is selected"]
    #[inline(always)]
    pub fn val52(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val52)
    }
    #[doc = "FlexIO_Shifer0_DMA_Req input is selected"]
    #[inline(always)]
    pub fn val53(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val53)
    }
    #[doc = "FlexIO_Shifer1_DMA_Req input is selected"]
    #[inline(always)]
    pub fn val54(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val54)
    }
    #[doc = "FlexIO_Shifer2_DMA_Req input is selected"]
    #[inline(always)]
    pub fn val55(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val55)
    }
    #[doc = "FlexIO_Shifer3_DMA_Req input is selected"]
    #[inline(always)]
    pub fn val56(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val56)
    }
    #[doc = "I3C0_IRQ input is selected"]
    #[inline(always)]
    pub fn val57(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val57)
    }
    #[doc = "LPI2C0_IRQ input is selected"]
    #[inline(always)]
    pub fn val58(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val58)
    }
    #[doc = "LPI2C1_IRQ input is selected"]
    #[inline(always)]
    pub fn val59(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val59)
    }
    #[doc = "LPSPI0_IRQ input is selected"]
    #[inline(always)]
    pub fn val60(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val60)
    }
    #[doc = "LPSPI1_IRQ input is selected"]
    #[inline(always)]
    pub fn val61(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val61)
    }
    #[doc = "LPUART0_IRQ input is selected"]
    #[inline(always)]
    pub fn val62(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val62)
    }
    #[doc = "LPUART1_IRQ input is selected"]
    #[inline(always)]
    pub fn val63(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val63)
    }
    #[doc = "LPUART2_IRQ input is selected"]
    #[inline(always)]
    pub fn val64(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val64)
    }
    #[doc = "LPUART3_IRQ input is selected"]
    #[inline(always)]
    pub fn val65(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val65)
    }
    #[doc = "USB0_SOF input is selected"]
    #[inline(always)]
    pub fn val66(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val66)
    }
    #[doc = "ADC0_IRQ input is selected"]
    #[inline(always)]
    pub fn val68(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val68)
    }
    #[doc = "ADC1_IRQ input is selected"]
    #[inline(always)]
    pub fn val69(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val69)
    }
    #[doc = "ADC2_IRQ input is selected"]
    #[inline(always)]
    pub fn val70(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val70)
    }
    #[doc = "ADC3_IRQ input is selected"]
    #[inline(always)]
    pub fn val71(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val71)
    }
    #[doc = "CMP0_IRQ input is selected"]
    #[inline(always)]
    pub fn val72(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val72)
    }
    #[doc = "CMP1_IRQ input is selected"]
    #[inline(always)]
    pub fn val73(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val73)
    }
    #[doc = "CMP2_IRQ input is selected"]
    #[inline(always)]
    pub fn val74(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val74)
    }
    #[doc = "CMP0_OUT input is selected"]
    #[inline(always)]
    pub fn val75(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val75)
    }
    #[doc = "CMP1_OUT input is selected"]
    #[inline(always)]
    pub fn val76(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val76)
    }
    #[doc = "CMP2_OUT input is selected"]
    #[inline(always)]
    pub fn val77(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val77)
    }
    #[doc = "DAC0_IRQ input is selected"]
    #[inline(always)]
    pub fn val78(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val78)
    }
    #[doc = "SLCD_IRQ input is selected"]
    #[inline(always)]
    pub fn val79(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val79)
    }
}
impl R {
    #[doc = "Bits 0:6 - Input number for SmartDMA."]
    #[inline(always)]
    pub fn inp(&self) -> InpR {
        InpR::new((self.bits & 0x7f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:6 - Input number for SmartDMA."]
    #[inline(always)]
    pub fn inp(&mut self) -> InpW<SmartDmaTrigSpec> {
        InpW::new(self, 0)
    }
}
#[doc = "SmartDMA Trigger Input Connections\n\nYou can [`read`](crate::Reg::read) this register and get [`smart_dma_trig::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`smart_dma_trig::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SmartDmaTrigSpec;
impl crate::RegisterSpec for SmartDmaTrigSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`smart_dma_trig::R`](R) reader structure"]
impl crate::Readable for SmartDmaTrigSpec {}
#[doc = "`write(|w| ..)` method takes [`smart_dma_trig::W`](W) writer structure"]
impl crate::Writable for SmartDmaTrigSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SmartDMA_TRIG[%s] to value 0x7f"]
impl crate::Resettable for SmartDmaTrigSpec {
    const RESET_VALUE: u32 = 0x7f;
}

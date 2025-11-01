#[doc = "Register `TIMER3TRIG` reader"]
pub type R = crate::R<Timer3trigSpec>;
#[doc = "Register `TIMER3TRIG` writer"]
pub type W = crate::W<Timer3trigSpec>;
#[doc = "Input number for CTIMER3\n\nValue on reset: 127"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Inp {
    #[doc = "1: CT_INP0 input is selected"]
    Val1 = 1,
    #[doc = "2: CT_INP1 input is selected"]
    Val2 = 2,
    #[doc = "3: CT_INP2 input is selected"]
    Val3 = 3,
    #[doc = "4: CT_INP3 input is selected"]
    Val4 = 4,
    #[doc = "5: CT_INP4 input is selected"]
    Val5 = 5,
    #[doc = "6: CT_INP5 input is selected"]
    Val6 = 6,
    #[doc = "7: CT_INP6 input is selected"]
    Val7 = 7,
    #[doc = "8: CT_INP7 input is selected"]
    Val8 = 8,
    #[doc = "9: CT_INP8 input is selected"]
    Val9 = 9,
    #[doc = "10: CT_INP9 input is selected"]
    Val10 = 10,
    #[doc = "11: CT_INP10 input is selected"]
    Val11 = 11,
    #[doc = "12: CT_INP11 input is selected"]
    Val12 = 12,
    #[doc = "13: CT_INP12 input is selected"]
    Val13 = 13,
    #[doc = "14: CT_INP13 input is selected"]
    Val14 = 14,
    #[doc = "15: CT_INP14 input is selected"]
    Val15 = 15,
    #[doc = "16: CT_INP15 input is selected"]
    Val16 = 16,
    #[doc = "17: CT_INP16 input is selected"]
    Val17 = 17,
    #[doc = "18: CT_INP17 input is selected"]
    Val18 = 18,
    #[doc = "19: CT_INP18 input is selected"]
    Val19 = 19,
    #[doc = "20: CT_INP19 input is selected"]
    Val20 = 20,
    #[doc = "21: USB0 usb0 start of frame input is selected"]
    Val21 = 21,
    #[doc = "22: AOI0_OUT0 input is selected"]
    Val22 = 22,
    #[doc = "23: AOI0_OUT1 input is selected"]
    Val23 = 23,
    #[doc = "24: AOI0_OUT2 input is selected"]
    Val24 = 24,
    #[doc = "25: AOI0_OUT3 input is selected"]
    Val25 = 25,
    #[doc = "26: ADC0_tcomp\\[0\\]"]
    Val26 = 26,
    #[doc = "27: ADC0_tcomp\\[1\\]"]
    Val27 = 27,
    #[doc = "28: ADC0_tcomp\\[2\\]"]
    Val28 = 28,
    #[doc = "29: ADC0_tcomp\\[3\\] input is selected"]
    Val29 = 29,
    #[doc = "30: CMP0_OUT is selected"]
    Val30 = 30,
    #[doc = "31: CMP1_OUT is selected"]
    Val31 = 31,
    #[doc = "32: CMP2_OUT is selected"]
    Val32 = 32,
    #[doc = "33: CTimer0_MAT1 input is selected"]
    Val33 = 33,
    #[doc = "34: CTimer0_MAT2 input is selected"]
    Val34 = 34,
    #[doc = "35: CTimer0_MAT3 input is selected"]
    Val35 = 35,
    #[doc = "36: CTimer1_MAT1 input is selected"]
    Val36 = 36,
    #[doc = "37: CTimer1_MAT2 input is selected"]
    Val37 = 37,
    #[doc = "38: CTimer1_MAT3 input is selected"]
    Val38 = 38,
    #[doc = "39: QDC0_CMP_FLAG0 is selected"]
    Val39 = 39,
    #[doc = "40: QDC0_CMP_FLAG1 input is selected"]
    Val40 = 40,
    #[doc = "41: QDC0_CMP_FLAG2 input is selected"]
    Val41 = 41,
    #[doc = "42: QDC0_CMP_FLAG3 input is selected"]
    Val42 = 42,
    #[doc = "43: QDC0_POS_MATCH0 input is selected"]
    Val43 = 43,
    #[doc = "44: PWM0_SM0_MUX_TRIG0 input is selected"]
    Val44 = 44,
    #[doc = "45: PWM0_SM1_MUX_TRIG0 input is selected"]
    Val45 = 45,
    #[doc = "46: PWM0_SM2_MUX_TRIG0 input is selected"]
    Val46 = 46,
    #[doc = "47: PWM0_SM3_MUX_TRIG0 input is selected"]
    Val47 = 47,
    #[doc = "48: LPI2C0 Master End of Packet input is selected"]
    Val48 = 48,
    #[doc = "49: LPI2C0 Slave End of Packet input is selected"]
    Val49 = 49,
    #[doc = "50: LPI2C1 Master End of Packet input is selected"]
    Val50 = 50,
    #[doc = "51: LPI2C1 Slave End of Packet input is selected"]
    Val51 = 51,
    #[doc = "52: LPSPI0 End of Frame input is selected"]
    Val52 = 52,
    #[doc = "53: LPSPI0 Received Data Word input is selected"]
    Val53 = 53,
    #[doc = "54: LPSPI1 End of Frame input is selected"]
    Val54 = 54,
    #[doc = "55: LPSPI1 Received Data Word input is selected"]
    Val55 = 55,
    #[doc = "56: LPUART0 Received Data Word input is selected"]
    Val56 = 56,
    #[doc = "57: LPUART0 Transmitted Data Word input is selected"]
    Val57 = 57,
    #[doc = "58: LPUART0 Receive Line Idle input is selected"]
    Val58 = 58,
    #[doc = "59: LPUART1 Received Data Word input is selected"]
    Val59 = 59,
    #[doc = "60: LPUART1 Transmitted Data Word input is selected"]
    Val60 = 60,
    #[doc = "61: LPUART1 Receive Line Idle input is selected"]
    Val61 = 61,
    #[doc = "62: LPUART2 Received Data Word input is selected"]
    Val62 = 62,
    #[doc = "63: LPUART2 Transmitted Data Word input is selected"]
    Val63 = 63,
    #[doc = "64: LPUART2 Receive Line Idle input is selected"]
    Val64 = 64,
    #[doc = "65: LPUART3 Received Data Word input is selected"]
    Val65 = 65,
    #[doc = "66: LPUART3 Transmitted Data Word input is selected"]
    Val66 = 66,
    #[doc = "67: LPUART3 Receive Line Idle input is selected"]
    Val67 = 67,
    #[doc = "68: LPUART4 Received Data Word input is selected"]
    Val68 = 68,
    #[doc = "69: LPUART4 Transmitted Data Word input is selected"]
    Val69 = 69,
    #[doc = "70: LPUART4 Receive Line Idle input is selected"]
    Val70 = 70,
    #[doc = "71: AOI1_OUT0 input is selected"]
    Val71 = 71,
    #[doc = "72: AOI1_OUT1 input is selected"]
    Val72 = 72,
    #[doc = "73: AOI1_OUT2 input is selected"]
    Val73 = 73,
    #[doc = "74: AOI1_OUT3 input is selected"]
    Val74 = 74,
    #[doc = "75: ADC1_tcomp\\[0\\] input is selected"]
    Val75 = 75,
    #[doc = "76: ADC1_tcomp\\[1\\] input is selected"]
    Val76 = 76,
    #[doc = "77: ADC1_tcomp\\[2\\] input is selected"]
    Val77 = 77,
    #[doc = "78: ADC1_tcomp\\[3\\] input is selected"]
    Val78 = 78,
    #[doc = "79: CTimer2_MAT1 input is selected"]
    Val79 = 79,
    #[doc = "80: CTimer2_MAT2 input is selected"]
    Val80 = 80,
    #[doc = "81: CTimer2_MAT3 input is selected"]
    Val81 = 81,
    #[doc = "82: CTimer4_MAT1 input is selected"]
    Val82 = 82,
    #[doc = "83: CTimer4_MAT2 input is selected"]
    Val83 = 83,
    #[doc = "84: CTimer4_MAT3 input is selected"]
    Val84 = 84,
    #[doc = "85: QDC1_CMP_FLAG0 input is selected"]
    Val85 = 85,
    #[doc = "86: QDC1_CMP_FLAG1 input is selected"]
    Val86 = 86,
    #[doc = "87: QDC1_CMP_FLAG2 input is selected"]
    Val87 = 87,
    #[doc = "88: QDC1_CMP_FLAG3 input is selected"]
    Val88 = 88,
    #[doc = "89: QDC1_POS_MATCH0 input is selected"]
    Val89 = 89,
    #[doc = "90: PWM1_SM0_MUX_TRIG0 input is selected"]
    Val90 = 90,
    #[doc = "91: PWM1_SM1_MUX_TRIG0 input is selected"]
    Val91 = 91,
    #[doc = "92: PWM1_SM2_MUX_TRIG0 input is selected"]
    Val92 = 92,
    #[doc = "93: PWM1_SM2_MUX_TRIG0 input is selected"]
    Val93 = 93,
    #[doc = "94: LPI2C2 Master End of Packet input is selected"]
    Val94 = 94,
    #[doc = "95: LPI2C2 Slave End of Packet input is selected"]
    Val95 = 95,
    #[doc = "96: LPI2C3 Master End of Packet input is selected"]
    Val96 = 96,
    #[doc = "97: LPI2C3 Slave End of Packet input is selected"]
    Val97 = 97,
    #[doc = "98: LPUART5 Received Data Word input is selected"]
    Val98 = 98,
    #[doc = "99: LPUART5 Transmitted Data Word input is selected"]
    Val99 = 99,
    #[doc = "100: LPUART5 Receive Line Idle input is selected"]
    Val100 = 100,
    #[doc = "105: ADC2_tcomp\\[0\\] input is selected"]
    Val105 = 105,
    #[doc = "106: ADC2_tcomp\\[1\\] input is selected"]
    Val106 = 106,
    #[doc = "107: ADC2_tcomp\\[2\\] input is selected"]
    Val107 = 107,
    #[doc = "108: ADC2_tcomp\\[3\\] input is selected"]
    Val108 = 108,
    #[doc = "109: ADC3_tcomp\\[0\\] input is selected"]
    Val109 = 109,
    #[doc = "110: ADC3_tcomp\\[1\\] input is selected"]
    Val110 = 110,
    #[doc = "111: ADC3_tcomp\\[2\\] input is selected"]
    Val111 = 111,
    #[doc = "112: ADC3_tcomp\\[3\\] input is selected"]
    Val112 = 112,
    #[doc = "113: TRIG_IN0 input is selected"]
    Val113 = 113,
    #[doc = "114: TRIG_IN1 input is selected"]
    Val114 = 114,
    #[doc = "115: TRIG_IN2 input is selected"]
    Val115 = 115,
    #[doc = "116: TRIG_IN3 input is selected"]
    Val116 = 116,
    #[doc = "117: TRIG_IN4 input is selected"]
    Val117 = 117,
    #[doc = "118: TRIG_IN5 input is selected"]
    Val118 = 118,
    #[doc = "119: TRIG_IN6 input is selected"]
    Val119 = 119,
    #[doc = "120: TRIG_IN7 input is selected"]
    Val120 = 120,
    #[doc = "121: TRIG_IN8 input is selected"]
    Val121 = 121,
    #[doc = "122: TRIG_IN9 input is selected"]
    Val122 = 122,
    #[doc = "123: TRIG_IN10 input is selected"]
    Val123 = 123,
    #[doc = "124: TRIG_IN11 input is selected"]
    Val124 = 124,
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
#[doc = "Field `INP` reader - Input number for CTIMER3"]
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
            67 => Some(Inp::Val67),
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
            80 => Some(Inp::Val80),
            81 => Some(Inp::Val81),
            82 => Some(Inp::Val82),
            83 => Some(Inp::Val83),
            84 => Some(Inp::Val84),
            85 => Some(Inp::Val85),
            86 => Some(Inp::Val86),
            87 => Some(Inp::Val87),
            88 => Some(Inp::Val88),
            89 => Some(Inp::Val89),
            90 => Some(Inp::Val90),
            91 => Some(Inp::Val91),
            92 => Some(Inp::Val92),
            93 => Some(Inp::Val93),
            94 => Some(Inp::Val94),
            95 => Some(Inp::Val95),
            96 => Some(Inp::Val96),
            97 => Some(Inp::Val97),
            98 => Some(Inp::Val98),
            99 => Some(Inp::Val99),
            100 => Some(Inp::Val100),
            105 => Some(Inp::Val105),
            106 => Some(Inp::Val106),
            107 => Some(Inp::Val107),
            108 => Some(Inp::Val108),
            109 => Some(Inp::Val109),
            110 => Some(Inp::Val110),
            111 => Some(Inp::Val111),
            112 => Some(Inp::Val112),
            113 => Some(Inp::Val113),
            114 => Some(Inp::Val114),
            115 => Some(Inp::Val115),
            116 => Some(Inp::Val116),
            117 => Some(Inp::Val117),
            118 => Some(Inp::Val118),
            119 => Some(Inp::Val119),
            120 => Some(Inp::Val120),
            121 => Some(Inp::Val121),
            122 => Some(Inp::Val122),
            123 => Some(Inp::Val123),
            124 => Some(Inp::Val124),
            _ => None,
        }
    }
    #[doc = "CT_INP0 input is selected"]
    #[inline(always)]
    pub fn is_val1(&self) -> bool {
        *self == Inp::Val1
    }
    #[doc = "CT_INP1 input is selected"]
    #[inline(always)]
    pub fn is_val2(&self) -> bool {
        *self == Inp::Val2
    }
    #[doc = "CT_INP2 input is selected"]
    #[inline(always)]
    pub fn is_val3(&self) -> bool {
        *self == Inp::Val3
    }
    #[doc = "CT_INP3 input is selected"]
    #[inline(always)]
    pub fn is_val4(&self) -> bool {
        *self == Inp::Val4
    }
    #[doc = "CT_INP4 input is selected"]
    #[inline(always)]
    pub fn is_val5(&self) -> bool {
        *self == Inp::Val5
    }
    #[doc = "CT_INP5 input is selected"]
    #[inline(always)]
    pub fn is_val6(&self) -> bool {
        *self == Inp::Val6
    }
    #[doc = "CT_INP6 input is selected"]
    #[inline(always)]
    pub fn is_val7(&self) -> bool {
        *self == Inp::Val7
    }
    #[doc = "CT_INP7 input is selected"]
    #[inline(always)]
    pub fn is_val8(&self) -> bool {
        *self == Inp::Val8
    }
    #[doc = "CT_INP8 input is selected"]
    #[inline(always)]
    pub fn is_val9(&self) -> bool {
        *self == Inp::Val9
    }
    #[doc = "CT_INP9 input is selected"]
    #[inline(always)]
    pub fn is_val10(&self) -> bool {
        *self == Inp::Val10
    }
    #[doc = "CT_INP10 input is selected"]
    #[inline(always)]
    pub fn is_val11(&self) -> bool {
        *self == Inp::Val11
    }
    #[doc = "CT_INP11 input is selected"]
    #[inline(always)]
    pub fn is_val12(&self) -> bool {
        *self == Inp::Val12
    }
    #[doc = "CT_INP12 input is selected"]
    #[inline(always)]
    pub fn is_val13(&self) -> bool {
        *self == Inp::Val13
    }
    #[doc = "CT_INP13 input is selected"]
    #[inline(always)]
    pub fn is_val14(&self) -> bool {
        *self == Inp::Val14
    }
    #[doc = "CT_INP14 input is selected"]
    #[inline(always)]
    pub fn is_val15(&self) -> bool {
        *self == Inp::Val15
    }
    #[doc = "CT_INP15 input is selected"]
    #[inline(always)]
    pub fn is_val16(&self) -> bool {
        *self == Inp::Val16
    }
    #[doc = "CT_INP16 input is selected"]
    #[inline(always)]
    pub fn is_val17(&self) -> bool {
        *self == Inp::Val17
    }
    #[doc = "CT_INP17 input is selected"]
    #[inline(always)]
    pub fn is_val18(&self) -> bool {
        *self == Inp::Val18
    }
    #[doc = "CT_INP18 input is selected"]
    #[inline(always)]
    pub fn is_val19(&self) -> bool {
        *self == Inp::Val19
    }
    #[doc = "CT_INP19 input is selected"]
    #[inline(always)]
    pub fn is_val20(&self) -> bool {
        *self == Inp::Val20
    }
    #[doc = "USB0 usb0 start of frame input is selected"]
    #[inline(always)]
    pub fn is_val21(&self) -> bool {
        *self == Inp::Val21
    }
    #[doc = "AOI0_OUT0 input is selected"]
    #[inline(always)]
    pub fn is_val22(&self) -> bool {
        *self == Inp::Val22
    }
    #[doc = "AOI0_OUT1 input is selected"]
    #[inline(always)]
    pub fn is_val23(&self) -> bool {
        *self == Inp::Val23
    }
    #[doc = "AOI0_OUT2 input is selected"]
    #[inline(always)]
    pub fn is_val24(&self) -> bool {
        *self == Inp::Val24
    }
    #[doc = "AOI0_OUT3 input is selected"]
    #[inline(always)]
    pub fn is_val25(&self) -> bool {
        *self == Inp::Val25
    }
    #[doc = "ADC0_tcomp\\[0\\]"]
    #[inline(always)]
    pub fn is_val26(&self) -> bool {
        *self == Inp::Val26
    }
    #[doc = "ADC0_tcomp\\[1\\]"]
    #[inline(always)]
    pub fn is_val27(&self) -> bool {
        *self == Inp::Val27
    }
    #[doc = "ADC0_tcomp\\[2\\]"]
    #[inline(always)]
    pub fn is_val28(&self) -> bool {
        *self == Inp::Val28
    }
    #[doc = "ADC0_tcomp\\[3\\] input is selected"]
    #[inline(always)]
    pub fn is_val29(&self) -> bool {
        *self == Inp::Val29
    }
    #[doc = "CMP0_OUT is selected"]
    #[inline(always)]
    pub fn is_val30(&self) -> bool {
        *self == Inp::Val30
    }
    #[doc = "CMP1_OUT is selected"]
    #[inline(always)]
    pub fn is_val31(&self) -> bool {
        *self == Inp::Val31
    }
    #[doc = "CMP2_OUT is selected"]
    #[inline(always)]
    pub fn is_val32(&self) -> bool {
        *self == Inp::Val32
    }
    #[doc = "CTimer0_MAT1 input is selected"]
    #[inline(always)]
    pub fn is_val33(&self) -> bool {
        *self == Inp::Val33
    }
    #[doc = "CTimer0_MAT2 input is selected"]
    #[inline(always)]
    pub fn is_val34(&self) -> bool {
        *self == Inp::Val34
    }
    #[doc = "CTimer0_MAT3 input is selected"]
    #[inline(always)]
    pub fn is_val35(&self) -> bool {
        *self == Inp::Val35
    }
    #[doc = "CTimer1_MAT1 input is selected"]
    #[inline(always)]
    pub fn is_val36(&self) -> bool {
        *self == Inp::Val36
    }
    #[doc = "CTimer1_MAT2 input is selected"]
    #[inline(always)]
    pub fn is_val37(&self) -> bool {
        *self == Inp::Val37
    }
    #[doc = "CTimer1_MAT3 input is selected"]
    #[inline(always)]
    pub fn is_val38(&self) -> bool {
        *self == Inp::Val38
    }
    #[doc = "QDC0_CMP_FLAG0 is selected"]
    #[inline(always)]
    pub fn is_val39(&self) -> bool {
        *self == Inp::Val39
    }
    #[doc = "QDC0_CMP_FLAG1 input is selected"]
    #[inline(always)]
    pub fn is_val40(&self) -> bool {
        *self == Inp::Val40
    }
    #[doc = "QDC0_CMP_FLAG2 input is selected"]
    #[inline(always)]
    pub fn is_val41(&self) -> bool {
        *self == Inp::Val41
    }
    #[doc = "QDC0_CMP_FLAG3 input is selected"]
    #[inline(always)]
    pub fn is_val42(&self) -> bool {
        *self == Inp::Val42
    }
    #[doc = "QDC0_POS_MATCH0 input is selected"]
    #[inline(always)]
    pub fn is_val43(&self) -> bool {
        *self == Inp::Val43
    }
    #[doc = "PWM0_SM0_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn is_val44(&self) -> bool {
        *self == Inp::Val44
    }
    #[doc = "PWM0_SM1_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn is_val45(&self) -> bool {
        *self == Inp::Val45
    }
    #[doc = "PWM0_SM2_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn is_val46(&self) -> bool {
        *self == Inp::Val46
    }
    #[doc = "PWM0_SM3_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn is_val47(&self) -> bool {
        *self == Inp::Val47
    }
    #[doc = "LPI2C0 Master End of Packet input is selected"]
    #[inline(always)]
    pub fn is_val48(&self) -> bool {
        *self == Inp::Val48
    }
    #[doc = "LPI2C0 Slave End of Packet input is selected"]
    #[inline(always)]
    pub fn is_val49(&self) -> bool {
        *self == Inp::Val49
    }
    #[doc = "LPI2C1 Master End of Packet input is selected"]
    #[inline(always)]
    pub fn is_val50(&self) -> bool {
        *self == Inp::Val50
    }
    #[doc = "LPI2C1 Slave End of Packet input is selected"]
    #[inline(always)]
    pub fn is_val51(&self) -> bool {
        *self == Inp::Val51
    }
    #[doc = "LPSPI0 End of Frame input is selected"]
    #[inline(always)]
    pub fn is_val52(&self) -> bool {
        *self == Inp::Val52
    }
    #[doc = "LPSPI0 Received Data Word input is selected"]
    #[inline(always)]
    pub fn is_val53(&self) -> bool {
        *self == Inp::Val53
    }
    #[doc = "LPSPI1 End of Frame input is selected"]
    #[inline(always)]
    pub fn is_val54(&self) -> bool {
        *self == Inp::Val54
    }
    #[doc = "LPSPI1 Received Data Word input is selected"]
    #[inline(always)]
    pub fn is_val55(&self) -> bool {
        *self == Inp::Val55
    }
    #[doc = "LPUART0 Received Data Word input is selected"]
    #[inline(always)]
    pub fn is_val56(&self) -> bool {
        *self == Inp::Val56
    }
    #[doc = "LPUART0 Transmitted Data Word input is selected"]
    #[inline(always)]
    pub fn is_val57(&self) -> bool {
        *self == Inp::Val57
    }
    #[doc = "LPUART0 Receive Line Idle input is selected"]
    #[inline(always)]
    pub fn is_val58(&self) -> bool {
        *self == Inp::Val58
    }
    #[doc = "LPUART1 Received Data Word input is selected"]
    #[inline(always)]
    pub fn is_val59(&self) -> bool {
        *self == Inp::Val59
    }
    #[doc = "LPUART1 Transmitted Data Word input is selected"]
    #[inline(always)]
    pub fn is_val60(&self) -> bool {
        *self == Inp::Val60
    }
    #[doc = "LPUART1 Receive Line Idle input is selected"]
    #[inline(always)]
    pub fn is_val61(&self) -> bool {
        *self == Inp::Val61
    }
    #[doc = "LPUART2 Received Data Word input is selected"]
    #[inline(always)]
    pub fn is_val62(&self) -> bool {
        *self == Inp::Val62
    }
    #[doc = "LPUART2 Transmitted Data Word input is selected"]
    #[inline(always)]
    pub fn is_val63(&self) -> bool {
        *self == Inp::Val63
    }
    #[doc = "LPUART2 Receive Line Idle input is selected"]
    #[inline(always)]
    pub fn is_val64(&self) -> bool {
        *self == Inp::Val64
    }
    #[doc = "LPUART3 Received Data Word input is selected"]
    #[inline(always)]
    pub fn is_val65(&self) -> bool {
        *self == Inp::Val65
    }
    #[doc = "LPUART3 Transmitted Data Word input is selected"]
    #[inline(always)]
    pub fn is_val66(&self) -> bool {
        *self == Inp::Val66
    }
    #[doc = "LPUART3 Receive Line Idle input is selected"]
    #[inline(always)]
    pub fn is_val67(&self) -> bool {
        *self == Inp::Val67
    }
    #[doc = "LPUART4 Received Data Word input is selected"]
    #[inline(always)]
    pub fn is_val68(&self) -> bool {
        *self == Inp::Val68
    }
    #[doc = "LPUART4 Transmitted Data Word input is selected"]
    #[inline(always)]
    pub fn is_val69(&self) -> bool {
        *self == Inp::Val69
    }
    #[doc = "LPUART4 Receive Line Idle input is selected"]
    #[inline(always)]
    pub fn is_val70(&self) -> bool {
        *self == Inp::Val70
    }
    #[doc = "AOI1_OUT0 input is selected"]
    #[inline(always)]
    pub fn is_val71(&self) -> bool {
        *self == Inp::Val71
    }
    #[doc = "AOI1_OUT1 input is selected"]
    #[inline(always)]
    pub fn is_val72(&self) -> bool {
        *self == Inp::Val72
    }
    #[doc = "AOI1_OUT2 input is selected"]
    #[inline(always)]
    pub fn is_val73(&self) -> bool {
        *self == Inp::Val73
    }
    #[doc = "AOI1_OUT3 input is selected"]
    #[inline(always)]
    pub fn is_val74(&self) -> bool {
        *self == Inp::Val74
    }
    #[doc = "ADC1_tcomp\\[0\\] input is selected"]
    #[inline(always)]
    pub fn is_val75(&self) -> bool {
        *self == Inp::Val75
    }
    #[doc = "ADC1_tcomp\\[1\\] input is selected"]
    #[inline(always)]
    pub fn is_val76(&self) -> bool {
        *self == Inp::Val76
    }
    #[doc = "ADC1_tcomp\\[2\\] input is selected"]
    #[inline(always)]
    pub fn is_val77(&self) -> bool {
        *self == Inp::Val77
    }
    #[doc = "ADC1_tcomp\\[3\\] input is selected"]
    #[inline(always)]
    pub fn is_val78(&self) -> bool {
        *self == Inp::Val78
    }
    #[doc = "CTimer2_MAT1 input is selected"]
    #[inline(always)]
    pub fn is_val79(&self) -> bool {
        *self == Inp::Val79
    }
    #[doc = "CTimer2_MAT2 input is selected"]
    #[inline(always)]
    pub fn is_val80(&self) -> bool {
        *self == Inp::Val80
    }
    #[doc = "CTimer2_MAT3 input is selected"]
    #[inline(always)]
    pub fn is_val81(&self) -> bool {
        *self == Inp::Val81
    }
    #[doc = "CTimer4_MAT1 input is selected"]
    #[inline(always)]
    pub fn is_val82(&self) -> bool {
        *self == Inp::Val82
    }
    #[doc = "CTimer4_MAT2 input is selected"]
    #[inline(always)]
    pub fn is_val83(&self) -> bool {
        *self == Inp::Val83
    }
    #[doc = "CTimer4_MAT3 input is selected"]
    #[inline(always)]
    pub fn is_val84(&self) -> bool {
        *self == Inp::Val84
    }
    #[doc = "QDC1_CMP_FLAG0 input is selected"]
    #[inline(always)]
    pub fn is_val85(&self) -> bool {
        *self == Inp::Val85
    }
    #[doc = "QDC1_CMP_FLAG1 input is selected"]
    #[inline(always)]
    pub fn is_val86(&self) -> bool {
        *self == Inp::Val86
    }
    #[doc = "QDC1_CMP_FLAG2 input is selected"]
    #[inline(always)]
    pub fn is_val87(&self) -> bool {
        *self == Inp::Val87
    }
    #[doc = "QDC1_CMP_FLAG3 input is selected"]
    #[inline(always)]
    pub fn is_val88(&self) -> bool {
        *self == Inp::Val88
    }
    #[doc = "QDC1_POS_MATCH0 input is selected"]
    #[inline(always)]
    pub fn is_val89(&self) -> bool {
        *self == Inp::Val89
    }
    #[doc = "PWM1_SM0_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn is_val90(&self) -> bool {
        *self == Inp::Val90
    }
    #[doc = "PWM1_SM1_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn is_val91(&self) -> bool {
        *self == Inp::Val91
    }
    #[doc = "PWM1_SM2_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn is_val92(&self) -> bool {
        *self == Inp::Val92
    }
    #[doc = "PWM1_SM2_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn is_val93(&self) -> bool {
        *self == Inp::Val93
    }
    #[doc = "LPI2C2 Master End of Packet input is selected"]
    #[inline(always)]
    pub fn is_val94(&self) -> bool {
        *self == Inp::Val94
    }
    #[doc = "LPI2C2 Slave End of Packet input is selected"]
    #[inline(always)]
    pub fn is_val95(&self) -> bool {
        *self == Inp::Val95
    }
    #[doc = "LPI2C3 Master End of Packet input is selected"]
    #[inline(always)]
    pub fn is_val96(&self) -> bool {
        *self == Inp::Val96
    }
    #[doc = "LPI2C3 Slave End of Packet input is selected"]
    #[inline(always)]
    pub fn is_val97(&self) -> bool {
        *self == Inp::Val97
    }
    #[doc = "LPUART5 Received Data Word input is selected"]
    #[inline(always)]
    pub fn is_val98(&self) -> bool {
        *self == Inp::Val98
    }
    #[doc = "LPUART5 Transmitted Data Word input is selected"]
    #[inline(always)]
    pub fn is_val99(&self) -> bool {
        *self == Inp::Val99
    }
    #[doc = "LPUART5 Receive Line Idle input is selected"]
    #[inline(always)]
    pub fn is_val100(&self) -> bool {
        *self == Inp::Val100
    }
    #[doc = "ADC2_tcomp\\[0\\] input is selected"]
    #[inline(always)]
    pub fn is_val105(&self) -> bool {
        *self == Inp::Val105
    }
    #[doc = "ADC2_tcomp\\[1\\] input is selected"]
    #[inline(always)]
    pub fn is_val106(&self) -> bool {
        *self == Inp::Val106
    }
    #[doc = "ADC2_tcomp\\[2\\] input is selected"]
    #[inline(always)]
    pub fn is_val107(&self) -> bool {
        *self == Inp::Val107
    }
    #[doc = "ADC2_tcomp\\[3\\] input is selected"]
    #[inline(always)]
    pub fn is_val108(&self) -> bool {
        *self == Inp::Val108
    }
    #[doc = "ADC3_tcomp\\[0\\] input is selected"]
    #[inline(always)]
    pub fn is_val109(&self) -> bool {
        *self == Inp::Val109
    }
    #[doc = "ADC3_tcomp\\[1\\] input is selected"]
    #[inline(always)]
    pub fn is_val110(&self) -> bool {
        *self == Inp::Val110
    }
    #[doc = "ADC3_tcomp\\[2\\] input is selected"]
    #[inline(always)]
    pub fn is_val111(&self) -> bool {
        *self == Inp::Val111
    }
    #[doc = "ADC3_tcomp\\[3\\] input is selected"]
    #[inline(always)]
    pub fn is_val112(&self) -> bool {
        *self == Inp::Val112
    }
    #[doc = "TRIG_IN0 input is selected"]
    #[inline(always)]
    pub fn is_val113(&self) -> bool {
        *self == Inp::Val113
    }
    #[doc = "TRIG_IN1 input is selected"]
    #[inline(always)]
    pub fn is_val114(&self) -> bool {
        *self == Inp::Val114
    }
    #[doc = "TRIG_IN2 input is selected"]
    #[inline(always)]
    pub fn is_val115(&self) -> bool {
        *self == Inp::Val115
    }
    #[doc = "TRIG_IN3 input is selected"]
    #[inline(always)]
    pub fn is_val116(&self) -> bool {
        *self == Inp::Val116
    }
    #[doc = "TRIG_IN4 input is selected"]
    #[inline(always)]
    pub fn is_val117(&self) -> bool {
        *self == Inp::Val117
    }
    #[doc = "TRIG_IN5 input is selected"]
    #[inline(always)]
    pub fn is_val118(&self) -> bool {
        *self == Inp::Val118
    }
    #[doc = "TRIG_IN6 input is selected"]
    #[inline(always)]
    pub fn is_val119(&self) -> bool {
        *self == Inp::Val119
    }
    #[doc = "TRIG_IN7 input is selected"]
    #[inline(always)]
    pub fn is_val120(&self) -> bool {
        *self == Inp::Val120
    }
    #[doc = "TRIG_IN8 input is selected"]
    #[inline(always)]
    pub fn is_val121(&self) -> bool {
        *self == Inp::Val121
    }
    #[doc = "TRIG_IN9 input is selected"]
    #[inline(always)]
    pub fn is_val122(&self) -> bool {
        *self == Inp::Val122
    }
    #[doc = "TRIG_IN10 input is selected"]
    #[inline(always)]
    pub fn is_val123(&self) -> bool {
        *self == Inp::Val123
    }
    #[doc = "TRIG_IN11 input is selected"]
    #[inline(always)]
    pub fn is_val124(&self) -> bool {
        *self == Inp::Val124
    }
}
#[doc = "Field `INP` writer - Input number for CTIMER3"]
pub type InpW<'a, REG> = crate::FieldWriter<'a, REG, 7, Inp>;
impl<'a, REG> InpW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "CT_INP0 input is selected"]
    #[inline(always)]
    pub fn val1(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val1)
    }
    #[doc = "CT_INP1 input is selected"]
    #[inline(always)]
    pub fn val2(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val2)
    }
    #[doc = "CT_INP2 input is selected"]
    #[inline(always)]
    pub fn val3(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val3)
    }
    #[doc = "CT_INP3 input is selected"]
    #[inline(always)]
    pub fn val4(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val4)
    }
    #[doc = "CT_INP4 input is selected"]
    #[inline(always)]
    pub fn val5(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val5)
    }
    #[doc = "CT_INP5 input is selected"]
    #[inline(always)]
    pub fn val6(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val6)
    }
    #[doc = "CT_INP6 input is selected"]
    #[inline(always)]
    pub fn val7(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val7)
    }
    #[doc = "CT_INP7 input is selected"]
    #[inline(always)]
    pub fn val8(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val8)
    }
    #[doc = "CT_INP8 input is selected"]
    #[inline(always)]
    pub fn val9(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val9)
    }
    #[doc = "CT_INP9 input is selected"]
    #[inline(always)]
    pub fn val10(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val10)
    }
    #[doc = "CT_INP10 input is selected"]
    #[inline(always)]
    pub fn val11(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val11)
    }
    #[doc = "CT_INP11 input is selected"]
    #[inline(always)]
    pub fn val12(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val12)
    }
    #[doc = "CT_INP12 input is selected"]
    #[inline(always)]
    pub fn val13(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val13)
    }
    #[doc = "CT_INP13 input is selected"]
    #[inline(always)]
    pub fn val14(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val14)
    }
    #[doc = "CT_INP14 input is selected"]
    #[inline(always)]
    pub fn val15(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val15)
    }
    #[doc = "CT_INP15 input is selected"]
    #[inline(always)]
    pub fn val16(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val16)
    }
    #[doc = "CT_INP16 input is selected"]
    #[inline(always)]
    pub fn val17(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val17)
    }
    #[doc = "CT_INP17 input is selected"]
    #[inline(always)]
    pub fn val18(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val18)
    }
    #[doc = "CT_INP18 input is selected"]
    #[inline(always)]
    pub fn val19(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val19)
    }
    #[doc = "CT_INP19 input is selected"]
    #[inline(always)]
    pub fn val20(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val20)
    }
    #[doc = "USB0 usb0 start of frame input is selected"]
    #[inline(always)]
    pub fn val21(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val21)
    }
    #[doc = "AOI0_OUT0 input is selected"]
    #[inline(always)]
    pub fn val22(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val22)
    }
    #[doc = "AOI0_OUT1 input is selected"]
    #[inline(always)]
    pub fn val23(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val23)
    }
    #[doc = "AOI0_OUT2 input is selected"]
    #[inline(always)]
    pub fn val24(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val24)
    }
    #[doc = "AOI0_OUT3 input is selected"]
    #[inline(always)]
    pub fn val25(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val25)
    }
    #[doc = "ADC0_tcomp\\[0\\]"]
    #[inline(always)]
    pub fn val26(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val26)
    }
    #[doc = "ADC0_tcomp\\[1\\]"]
    #[inline(always)]
    pub fn val27(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val27)
    }
    #[doc = "ADC0_tcomp\\[2\\]"]
    #[inline(always)]
    pub fn val28(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val28)
    }
    #[doc = "ADC0_tcomp\\[3\\] input is selected"]
    #[inline(always)]
    pub fn val29(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val29)
    }
    #[doc = "CMP0_OUT is selected"]
    #[inline(always)]
    pub fn val30(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val30)
    }
    #[doc = "CMP1_OUT is selected"]
    #[inline(always)]
    pub fn val31(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val31)
    }
    #[doc = "CMP2_OUT is selected"]
    #[inline(always)]
    pub fn val32(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val32)
    }
    #[doc = "CTimer0_MAT1 input is selected"]
    #[inline(always)]
    pub fn val33(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val33)
    }
    #[doc = "CTimer0_MAT2 input is selected"]
    #[inline(always)]
    pub fn val34(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val34)
    }
    #[doc = "CTimer0_MAT3 input is selected"]
    #[inline(always)]
    pub fn val35(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val35)
    }
    #[doc = "CTimer1_MAT1 input is selected"]
    #[inline(always)]
    pub fn val36(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val36)
    }
    #[doc = "CTimer1_MAT2 input is selected"]
    #[inline(always)]
    pub fn val37(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val37)
    }
    #[doc = "CTimer1_MAT3 input is selected"]
    #[inline(always)]
    pub fn val38(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val38)
    }
    #[doc = "QDC0_CMP_FLAG0 is selected"]
    #[inline(always)]
    pub fn val39(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val39)
    }
    #[doc = "QDC0_CMP_FLAG1 input is selected"]
    #[inline(always)]
    pub fn val40(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val40)
    }
    #[doc = "QDC0_CMP_FLAG2 input is selected"]
    #[inline(always)]
    pub fn val41(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val41)
    }
    #[doc = "QDC0_CMP_FLAG3 input is selected"]
    #[inline(always)]
    pub fn val42(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val42)
    }
    #[doc = "QDC0_POS_MATCH0 input is selected"]
    #[inline(always)]
    pub fn val43(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val43)
    }
    #[doc = "PWM0_SM0_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn val44(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val44)
    }
    #[doc = "PWM0_SM1_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn val45(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val45)
    }
    #[doc = "PWM0_SM2_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn val46(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val46)
    }
    #[doc = "PWM0_SM3_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn val47(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val47)
    }
    #[doc = "LPI2C0 Master End of Packet input is selected"]
    #[inline(always)]
    pub fn val48(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val48)
    }
    #[doc = "LPI2C0 Slave End of Packet input is selected"]
    #[inline(always)]
    pub fn val49(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val49)
    }
    #[doc = "LPI2C1 Master End of Packet input is selected"]
    #[inline(always)]
    pub fn val50(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val50)
    }
    #[doc = "LPI2C1 Slave End of Packet input is selected"]
    #[inline(always)]
    pub fn val51(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val51)
    }
    #[doc = "LPSPI0 End of Frame input is selected"]
    #[inline(always)]
    pub fn val52(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val52)
    }
    #[doc = "LPSPI0 Received Data Word input is selected"]
    #[inline(always)]
    pub fn val53(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val53)
    }
    #[doc = "LPSPI1 End of Frame input is selected"]
    #[inline(always)]
    pub fn val54(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val54)
    }
    #[doc = "LPSPI1 Received Data Word input is selected"]
    #[inline(always)]
    pub fn val55(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val55)
    }
    #[doc = "LPUART0 Received Data Word input is selected"]
    #[inline(always)]
    pub fn val56(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val56)
    }
    #[doc = "LPUART0 Transmitted Data Word input is selected"]
    #[inline(always)]
    pub fn val57(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val57)
    }
    #[doc = "LPUART0 Receive Line Idle input is selected"]
    #[inline(always)]
    pub fn val58(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val58)
    }
    #[doc = "LPUART1 Received Data Word input is selected"]
    #[inline(always)]
    pub fn val59(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val59)
    }
    #[doc = "LPUART1 Transmitted Data Word input is selected"]
    #[inline(always)]
    pub fn val60(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val60)
    }
    #[doc = "LPUART1 Receive Line Idle input is selected"]
    #[inline(always)]
    pub fn val61(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val61)
    }
    #[doc = "LPUART2 Received Data Word input is selected"]
    #[inline(always)]
    pub fn val62(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val62)
    }
    #[doc = "LPUART2 Transmitted Data Word input is selected"]
    #[inline(always)]
    pub fn val63(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val63)
    }
    #[doc = "LPUART2 Receive Line Idle input is selected"]
    #[inline(always)]
    pub fn val64(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val64)
    }
    #[doc = "LPUART3 Received Data Word input is selected"]
    #[inline(always)]
    pub fn val65(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val65)
    }
    #[doc = "LPUART3 Transmitted Data Word input is selected"]
    #[inline(always)]
    pub fn val66(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val66)
    }
    #[doc = "LPUART3 Receive Line Idle input is selected"]
    #[inline(always)]
    pub fn val67(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val67)
    }
    #[doc = "LPUART4 Received Data Word input is selected"]
    #[inline(always)]
    pub fn val68(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val68)
    }
    #[doc = "LPUART4 Transmitted Data Word input is selected"]
    #[inline(always)]
    pub fn val69(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val69)
    }
    #[doc = "LPUART4 Receive Line Idle input is selected"]
    #[inline(always)]
    pub fn val70(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val70)
    }
    #[doc = "AOI1_OUT0 input is selected"]
    #[inline(always)]
    pub fn val71(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val71)
    }
    #[doc = "AOI1_OUT1 input is selected"]
    #[inline(always)]
    pub fn val72(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val72)
    }
    #[doc = "AOI1_OUT2 input is selected"]
    #[inline(always)]
    pub fn val73(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val73)
    }
    #[doc = "AOI1_OUT3 input is selected"]
    #[inline(always)]
    pub fn val74(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val74)
    }
    #[doc = "ADC1_tcomp\\[0\\] input is selected"]
    #[inline(always)]
    pub fn val75(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val75)
    }
    #[doc = "ADC1_tcomp\\[1\\] input is selected"]
    #[inline(always)]
    pub fn val76(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val76)
    }
    #[doc = "ADC1_tcomp\\[2\\] input is selected"]
    #[inline(always)]
    pub fn val77(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val77)
    }
    #[doc = "ADC1_tcomp\\[3\\] input is selected"]
    #[inline(always)]
    pub fn val78(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val78)
    }
    #[doc = "CTimer2_MAT1 input is selected"]
    #[inline(always)]
    pub fn val79(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val79)
    }
    #[doc = "CTimer2_MAT2 input is selected"]
    #[inline(always)]
    pub fn val80(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val80)
    }
    #[doc = "CTimer2_MAT3 input is selected"]
    #[inline(always)]
    pub fn val81(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val81)
    }
    #[doc = "CTimer4_MAT1 input is selected"]
    #[inline(always)]
    pub fn val82(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val82)
    }
    #[doc = "CTimer4_MAT2 input is selected"]
    #[inline(always)]
    pub fn val83(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val83)
    }
    #[doc = "CTimer4_MAT3 input is selected"]
    #[inline(always)]
    pub fn val84(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val84)
    }
    #[doc = "QDC1_CMP_FLAG0 input is selected"]
    #[inline(always)]
    pub fn val85(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val85)
    }
    #[doc = "QDC1_CMP_FLAG1 input is selected"]
    #[inline(always)]
    pub fn val86(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val86)
    }
    #[doc = "QDC1_CMP_FLAG2 input is selected"]
    #[inline(always)]
    pub fn val87(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val87)
    }
    #[doc = "QDC1_CMP_FLAG3 input is selected"]
    #[inline(always)]
    pub fn val88(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val88)
    }
    #[doc = "QDC1_POS_MATCH0 input is selected"]
    #[inline(always)]
    pub fn val89(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val89)
    }
    #[doc = "PWM1_SM0_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn val90(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val90)
    }
    #[doc = "PWM1_SM1_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn val91(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val91)
    }
    #[doc = "PWM1_SM2_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn val92(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val92)
    }
    #[doc = "PWM1_SM2_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn val93(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val93)
    }
    #[doc = "LPI2C2 Master End of Packet input is selected"]
    #[inline(always)]
    pub fn val94(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val94)
    }
    #[doc = "LPI2C2 Slave End of Packet input is selected"]
    #[inline(always)]
    pub fn val95(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val95)
    }
    #[doc = "LPI2C3 Master End of Packet input is selected"]
    #[inline(always)]
    pub fn val96(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val96)
    }
    #[doc = "LPI2C3 Slave End of Packet input is selected"]
    #[inline(always)]
    pub fn val97(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val97)
    }
    #[doc = "LPUART5 Received Data Word input is selected"]
    #[inline(always)]
    pub fn val98(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val98)
    }
    #[doc = "LPUART5 Transmitted Data Word input is selected"]
    #[inline(always)]
    pub fn val99(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val99)
    }
    #[doc = "LPUART5 Receive Line Idle input is selected"]
    #[inline(always)]
    pub fn val100(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val100)
    }
    #[doc = "ADC2_tcomp\\[0\\] input is selected"]
    #[inline(always)]
    pub fn val105(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val105)
    }
    #[doc = "ADC2_tcomp\\[1\\] input is selected"]
    #[inline(always)]
    pub fn val106(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val106)
    }
    #[doc = "ADC2_tcomp\\[2\\] input is selected"]
    #[inline(always)]
    pub fn val107(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val107)
    }
    #[doc = "ADC2_tcomp\\[3\\] input is selected"]
    #[inline(always)]
    pub fn val108(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val108)
    }
    #[doc = "ADC3_tcomp\\[0\\] input is selected"]
    #[inline(always)]
    pub fn val109(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val109)
    }
    #[doc = "ADC3_tcomp\\[1\\] input is selected"]
    #[inline(always)]
    pub fn val110(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val110)
    }
    #[doc = "ADC3_tcomp\\[2\\] input is selected"]
    #[inline(always)]
    pub fn val111(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val111)
    }
    #[doc = "ADC3_tcomp\\[3\\] input is selected"]
    #[inline(always)]
    pub fn val112(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val112)
    }
    #[doc = "TRIG_IN0 input is selected"]
    #[inline(always)]
    pub fn val113(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val113)
    }
    #[doc = "TRIG_IN1 input is selected"]
    #[inline(always)]
    pub fn val114(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val114)
    }
    #[doc = "TRIG_IN2 input is selected"]
    #[inline(always)]
    pub fn val115(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val115)
    }
    #[doc = "TRIG_IN3 input is selected"]
    #[inline(always)]
    pub fn val116(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val116)
    }
    #[doc = "TRIG_IN4 input is selected"]
    #[inline(always)]
    pub fn val117(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val117)
    }
    #[doc = "TRIG_IN5 input is selected"]
    #[inline(always)]
    pub fn val118(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val118)
    }
    #[doc = "TRIG_IN6 input is selected"]
    #[inline(always)]
    pub fn val119(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val119)
    }
    #[doc = "TRIG_IN7 input is selected"]
    #[inline(always)]
    pub fn val120(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val120)
    }
    #[doc = "TRIG_IN8 input is selected"]
    #[inline(always)]
    pub fn val121(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val121)
    }
    #[doc = "TRIG_IN9 input is selected"]
    #[inline(always)]
    pub fn val122(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val122)
    }
    #[doc = "TRIG_IN10 input is selected"]
    #[inline(always)]
    pub fn val123(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val123)
    }
    #[doc = "TRIG_IN11 input is selected"]
    #[inline(always)]
    pub fn val124(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val124)
    }
}
impl R {
    #[doc = "Bits 0:6 - Input number for CTIMER3"]
    #[inline(always)]
    pub fn inp(&self) -> InpR {
        InpR::new((self.bits & 0x7f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:6 - Input number for CTIMER3"]
    #[inline(always)]
    pub fn inp(&mut self) -> InpW<Timer3trigSpec> {
        InpW::new(self, 0)
    }
}
#[doc = "Trigger register for TIMER3\n\nYou can [`read`](crate::Reg::read) this register and get [`timer3trig::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`timer3trig::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Timer3trigSpec;
impl crate::RegisterSpec for Timer3trigSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`timer3trig::R`](R) reader structure"]
impl crate::Readable for Timer3trigSpec {}
#[doc = "`write(|w| ..)` method takes [`timer3trig::W`](W) writer structure"]
impl crate::Writable for Timer3trigSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TIMER3TRIG to value 0x7f"]
impl crate::Resettable for Timer3trigSpec {
    const RESET_VALUE: u32 = 0x7f;
}

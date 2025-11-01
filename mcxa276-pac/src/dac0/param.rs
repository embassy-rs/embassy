#[doc = "Register `PARAM` reader"]
pub type R = crate::R<ParamSpec>;
#[doc = "FIFO Size\n\nValue on reset: 3"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Fifosz {
    #[doc = "1: FIFO depth is 4"]
    Val1 = 1,
    #[doc = "2: FIFO depth is 8"]
    Val2 = 2,
    #[doc = "3: FIFO depth is 16"]
    Val3 = 3,
    #[doc = "4: FIFO depth is 32"]
    Val4 = 4,
    #[doc = "5: FIFO depth is 64"]
    Val5 = 5,
    #[doc = "6: FIFO depth is 128"]
    Val6 = 6,
    #[doc = "7: FIFO depth is 256"]
    Val7 = 7,
}
impl From<Fifosz> for u8 {
    #[inline(always)]
    fn from(variant: Fifosz) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Fifosz {
    type Ux = u8;
}
impl crate::IsEnum for Fifosz {}
#[doc = "Field `FIFOSZ` reader - FIFO Size"]
pub type FifoszR = crate::FieldReader<Fifosz>;
impl FifoszR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Fifosz> {
        match self.bits {
            1 => Some(Fifosz::Val1),
            2 => Some(Fifosz::Val2),
            3 => Some(Fifosz::Val3),
            4 => Some(Fifosz::Val4),
            5 => Some(Fifosz::Val5),
            6 => Some(Fifosz::Val6),
            7 => Some(Fifosz::Val7),
            _ => None,
        }
    }
    #[doc = "FIFO depth is 4"]
    #[inline(always)]
    pub fn is_val_1(&self) -> bool {
        *self == Fifosz::Val1
    }
    #[doc = "FIFO depth is 8"]
    #[inline(always)]
    pub fn is_val_2(&self) -> bool {
        *self == Fifosz::Val2
    }
    #[doc = "FIFO depth is 16"]
    #[inline(always)]
    pub fn is_val_3(&self) -> bool {
        *self == Fifosz::Val3
    }
    #[doc = "FIFO depth is 32"]
    #[inline(always)]
    pub fn is_val_4(&self) -> bool {
        *self == Fifosz::Val4
    }
    #[doc = "FIFO depth is 64"]
    #[inline(always)]
    pub fn is_val_5(&self) -> bool {
        *self == Fifosz::Val5
    }
    #[doc = "FIFO depth is 128"]
    #[inline(always)]
    pub fn is_val_6(&self) -> bool {
        *self == Fifosz::Val6
    }
    #[doc = "FIFO depth is 256"]
    #[inline(always)]
    pub fn is_val_7(&self) -> bool {
        *self == Fifosz::Val7
    }
}
impl R {
    #[doc = "Bits 0:2 - FIFO Size"]
    #[inline(always)]
    pub fn fifosz(&self) -> FifoszR {
        FifoszR::new((self.bits & 7) as u8)
    }
}
#[doc = "Parameter\n\nYou can [`read`](crate::Reg::read) this register and get [`param::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ParamSpec;
impl crate::RegisterSpec for ParamSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`param::R`](R) reader structure"]
impl crate::Readable for ParamSpec {}
#[doc = "`reset()` method sets PARAM to value 0x03"]
impl crate::Resettable for ParamSpec {
    const RESET_VALUE: u32 = 0x03;
}

#[doc = "Register `OP_CTRL` reader"]
pub type R = crate::R<OpCtrlSpec>;
#[doc = "Register `OP_CTRL` writer"]
pub type W = crate::W<OpCtrlSpec>;
#[doc = "Override RES0 Data Type Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OvdtEnRes0 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<OvdtEnRes0> for bool {
    #[inline(always)]
    fn from(variant: OvdtEnRes0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OVDT_EN_RES0` reader - Override RES0 Data Type Enable"]
pub type OvdtEnRes0R = crate::BitReader<OvdtEnRes0>;
impl OvdtEnRes0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> OvdtEnRes0 {
        match self.bits {
            false => OvdtEnRes0::Disable,
            true => OvdtEnRes0::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == OvdtEnRes0::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == OvdtEnRes0::Enable
    }
}
#[doc = "Field `OVDT_EN_RES0` writer - Override RES0 Data Type Enable"]
pub type OvdtEnRes0W<'a, REG> = crate::BitWriter<'a, REG, OvdtEnRes0>;
impl<'a, REG> OvdtEnRes0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(OvdtEnRes0::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(OvdtEnRes0::Enable)
    }
}
#[doc = "Override RES0 Data Type\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum OvdtRes0 {
    #[doc = "0: UINT"]
    Uint = 0,
    #[doc = "1: INT"]
    Int = 1,
    #[doc = "2: Q1.X"]
    Q1X = 2,
    #[doc = "3: FLOAT"]
    Float = 3,
}
impl From<OvdtRes0> for u8 {
    #[inline(always)]
    fn from(variant: OvdtRes0) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for OvdtRes0 {
    type Ux = u8;
}
impl crate::IsEnum for OvdtRes0 {}
#[doc = "Field `OVDT_RES0` reader - Override RES0 Data Type"]
pub type OvdtRes0R = crate::FieldReader<OvdtRes0>;
impl OvdtRes0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> OvdtRes0 {
        match self.bits {
            0 => OvdtRes0::Uint,
            1 => OvdtRes0::Int,
            2 => OvdtRes0::Q1X,
            3 => OvdtRes0::Float,
            _ => unreachable!(),
        }
    }
    #[doc = "UINT"]
    #[inline(always)]
    pub fn is_uint(&self) -> bool {
        *self == OvdtRes0::Uint
    }
    #[doc = "INT"]
    #[inline(always)]
    pub fn is_int(&self) -> bool {
        *self == OvdtRes0::Int
    }
    #[doc = "Q1.X"]
    #[inline(always)]
    pub fn is_q1_x(&self) -> bool {
        *self == OvdtRes0::Q1X
    }
    #[doc = "FLOAT"]
    #[inline(always)]
    pub fn is_float(&self) -> bool {
        *self == OvdtRes0::Float
    }
}
#[doc = "Field `OVDT_RES0` writer - Override RES0 Data Type"]
pub type OvdtRes0W<'a, REG> = crate::FieldWriter<'a, REG, 2, OvdtRes0, crate::Safe>;
impl<'a, REG> OvdtRes0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "UINT"]
    #[inline(always)]
    pub fn uint(self) -> &'a mut crate::W<REG> {
        self.variant(OvdtRes0::Uint)
    }
    #[doc = "INT"]
    #[inline(always)]
    pub fn int(self) -> &'a mut crate::W<REG> {
        self.variant(OvdtRes0::Int)
    }
    #[doc = "Q1.X"]
    #[inline(always)]
    pub fn q1_x(self) -> &'a mut crate::W<REG> {
        self.variant(OvdtRes0::Q1X)
    }
    #[doc = "FLOAT"]
    #[inline(always)]
    pub fn float(self) -> &'a mut crate::W<REG> {
        self.variant(OvdtRes0::Float)
    }
}
#[doc = "Override RES1 Data Type Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OvdtEnRes1 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<OvdtEnRes1> for bool {
    #[inline(always)]
    fn from(variant: OvdtEnRes1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OVDT_EN_RES1` reader - Override RES1 Data Type Enable"]
pub type OvdtEnRes1R = crate::BitReader<OvdtEnRes1>;
impl OvdtEnRes1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> OvdtEnRes1 {
        match self.bits {
            false => OvdtEnRes1::Disable,
            true => OvdtEnRes1::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == OvdtEnRes1::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == OvdtEnRes1::Enable
    }
}
#[doc = "Field `OVDT_EN_RES1` writer - Override RES1 Data Type Enable"]
pub type OvdtEnRes1W<'a, REG> = crate::BitWriter<'a, REG, OvdtEnRes1>;
impl<'a, REG> OvdtEnRes1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(OvdtEnRes1::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(OvdtEnRes1::Enable)
    }
}
#[doc = "Override RES1 Data Type\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum OvdtRes1 {
    #[doc = "0: UINT"]
    Uint = 0,
    #[doc = "1: INT"]
    Int = 1,
    #[doc = "2: Q1.X"]
    Q1X = 2,
    #[doc = "3: FLOAT"]
    Float = 3,
}
impl From<OvdtRes1> for u8 {
    #[inline(always)]
    fn from(variant: OvdtRes1) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for OvdtRes1 {
    type Ux = u8;
}
impl crate::IsEnum for OvdtRes1 {}
#[doc = "Field `OVDT_RES1` reader - Override RES1 Data Type"]
pub type OvdtRes1R = crate::FieldReader<OvdtRes1>;
impl OvdtRes1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> OvdtRes1 {
        match self.bits {
            0 => OvdtRes1::Uint,
            1 => OvdtRes1::Int,
            2 => OvdtRes1::Q1X,
            3 => OvdtRes1::Float,
            _ => unreachable!(),
        }
    }
    #[doc = "UINT"]
    #[inline(always)]
    pub fn is_uint(&self) -> bool {
        *self == OvdtRes1::Uint
    }
    #[doc = "INT"]
    #[inline(always)]
    pub fn is_int(&self) -> bool {
        *self == OvdtRes1::Int
    }
    #[doc = "Q1.X"]
    #[inline(always)]
    pub fn is_q1_x(&self) -> bool {
        *self == OvdtRes1::Q1X
    }
    #[doc = "FLOAT"]
    #[inline(always)]
    pub fn is_float(&self) -> bool {
        *self == OvdtRes1::Float
    }
}
#[doc = "Field `OVDT_RES1` writer - Override RES1 Data Type"]
pub type OvdtRes1W<'a, REG> = crate::FieldWriter<'a, REG, 2, OvdtRes1, crate::Safe>;
impl<'a, REG> OvdtRes1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "UINT"]
    #[inline(always)]
    pub fn uint(self) -> &'a mut crate::W<REG> {
        self.variant(OvdtRes1::Uint)
    }
    #[doc = "INT"]
    #[inline(always)]
    pub fn int(self) -> &'a mut crate::W<REG> {
        self.variant(OvdtRes1::Int)
    }
    #[doc = "Q1.X"]
    #[inline(always)]
    pub fn q1_x(self) -> &'a mut crate::W<REG> {
        self.variant(OvdtRes1::Q1X)
    }
    #[doc = "FLOAT"]
    #[inline(always)]
    pub fn float(self) -> &'a mut crate::W<REG> {
        self.variant(OvdtRes1::Float)
    }
}
#[doc = "Override RES2 Data Type Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OvdtEnRes2 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<OvdtEnRes2> for bool {
    #[inline(always)]
    fn from(variant: OvdtEnRes2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OVDT_EN_RES2` reader - Override RES2 Data Type Enable"]
pub type OvdtEnRes2R = crate::BitReader<OvdtEnRes2>;
impl OvdtEnRes2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> OvdtEnRes2 {
        match self.bits {
            false => OvdtEnRes2::Disable,
            true => OvdtEnRes2::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == OvdtEnRes2::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == OvdtEnRes2::Enable
    }
}
#[doc = "Field `OVDT_EN_RES2` writer - Override RES2 Data Type Enable"]
pub type OvdtEnRes2W<'a, REG> = crate::BitWriter<'a, REG, OvdtEnRes2>;
impl<'a, REG> OvdtEnRes2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(OvdtEnRes2::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(OvdtEnRes2::Enable)
    }
}
#[doc = "Override RES2 Data Type\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum OvdtRes2 {
    #[doc = "0: UINT"]
    Uint = 0,
    #[doc = "1: INT"]
    Int = 1,
    #[doc = "2: Q1.X"]
    Q1X = 2,
    #[doc = "3: FLOAT"]
    Float = 3,
}
impl From<OvdtRes2> for u8 {
    #[inline(always)]
    fn from(variant: OvdtRes2) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for OvdtRes2 {
    type Ux = u8;
}
impl crate::IsEnum for OvdtRes2 {}
#[doc = "Field `OVDT_RES2` reader - Override RES2 Data Type"]
pub type OvdtRes2R = crate::FieldReader<OvdtRes2>;
impl OvdtRes2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> OvdtRes2 {
        match self.bits {
            0 => OvdtRes2::Uint,
            1 => OvdtRes2::Int,
            2 => OvdtRes2::Q1X,
            3 => OvdtRes2::Float,
            _ => unreachable!(),
        }
    }
    #[doc = "UINT"]
    #[inline(always)]
    pub fn is_uint(&self) -> bool {
        *self == OvdtRes2::Uint
    }
    #[doc = "INT"]
    #[inline(always)]
    pub fn is_int(&self) -> bool {
        *self == OvdtRes2::Int
    }
    #[doc = "Q1.X"]
    #[inline(always)]
    pub fn is_q1_x(&self) -> bool {
        *self == OvdtRes2::Q1X
    }
    #[doc = "FLOAT"]
    #[inline(always)]
    pub fn is_float(&self) -> bool {
        *self == OvdtRes2::Float
    }
}
#[doc = "Field `OVDT_RES2` writer - Override RES2 Data Type"]
pub type OvdtRes2W<'a, REG> = crate::FieldWriter<'a, REG, 2, OvdtRes2, crate::Safe>;
impl<'a, REG> OvdtRes2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "UINT"]
    #[inline(always)]
    pub fn uint(self) -> &'a mut crate::W<REG> {
        self.variant(OvdtRes2::Uint)
    }
    #[doc = "INT"]
    #[inline(always)]
    pub fn int(self) -> &'a mut crate::W<REG> {
        self.variant(OvdtRes2::Int)
    }
    #[doc = "Q1.X"]
    #[inline(always)]
    pub fn q1_x(self) -> &'a mut crate::W<REG> {
        self.variant(OvdtRes2::Q1X)
    }
    #[doc = "FLOAT"]
    #[inline(always)]
    pub fn float(self) -> &'a mut crate::W<REG> {
        self.variant(OvdtRes2::Float)
    }
}
#[doc = "Override RES3 Data Type Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OvdtEnRes3 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<OvdtEnRes3> for bool {
    #[inline(always)]
    fn from(variant: OvdtEnRes3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OVDT_EN_RES3` reader - Override RES3 Data Type Enable"]
pub type OvdtEnRes3R = crate::BitReader<OvdtEnRes3>;
impl OvdtEnRes3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> OvdtEnRes3 {
        match self.bits {
            false => OvdtEnRes3::Disable,
            true => OvdtEnRes3::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == OvdtEnRes3::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == OvdtEnRes3::Enable
    }
}
#[doc = "Field `OVDT_EN_RES3` writer - Override RES3 Data Type Enable"]
pub type OvdtEnRes3W<'a, REG> = crate::BitWriter<'a, REG, OvdtEnRes3>;
impl<'a, REG> OvdtEnRes3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(OvdtEnRes3::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(OvdtEnRes3::Enable)
    }
}
#[doc = "Override RES3 Data Type\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum OvdtRes3 {
    #[doc = "0: UINT"]
    Uint = 0,
    #[doc = "1: INT"]
    Int = 1,
    #[doc = "2: Q1.X"]
    Q1X = 2,
    #[doc = "3: FLOAT"]
    Float = 3,
}
impl From<OvdtRes3> for u8 {
    #[inline(always)]
    fn from(variant: OvdtRes3) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for OvdtRes3 {
    type Ux = u8;
}
impl crate::IsEnum for OvdtRes3 {}
#[doc = "Field `OVDT_RES3` reader - Override RES3 Data Type"]
pub type OvdtRes3R = crate::FieldReader<OvdtRes3>;
impl OvdtRes3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> OvdtRes3 {
        match self.bits {
            0 => OvdtRes3::Uint,
            1 => OvdtRes3::Int,
            2 => OvdtRes3::Q1X,
            3 => OvdtRes3::Float,
            _ => unreachable!(),
        }
    }
    #[doc = "UINT"]
    #[inline(always)]
    pub fn is_uint(&self) -> bool {
        *self == OvdtRes3::Uint
    }
    #[doc = "INT"]
    #[inline(always)]
    pub fn is_int(&self) -> bool {
        *self == OvdtRes3::Int
    }
    #[doc = "Q1.X"]
    #[inline(always)]
    pub fn is_q1_x(&self) -> bool {
        *self == OvdtRes3::Q1X
    }
    #[doc = "FLOAT"]
    #[inline(always)]
    pub fn is_float(&self) -> bool {
        *self == OvdtRes3::Float
    }
}
#[doc = "Field `OVDT_RES3` writer - Override RES3 Data Type"]
pub type OvdtRes3W<'a, REG> = crate::FieldWriter<'a, REG, 2, OvdtRes3, crate::Safe>;
impl<'a, REG> OvdtRes3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "UINT"]
    #[inline(always)]
    pub fn uint(self) -> &'a mut crate::W<REG> {
        self.variant(OvdtRes3::Uint)
    }
    #[doc = "INT"]
    #[inline(always)]
    pub fn int(self) -> &'a mut crate::W<REG> {
        self.variant(OvdtRes3::Int)
    }
    #[doc = "Q1.X"]
    #[inline(always)]
    pub fn q1_x(self) -> &'a mut crate::W<REG> {
        self.variant(OvdtRes3::Q1X)
    }
    #[doc = "FLOAT"]
    #[inline(always)]
    pub fn float(self) -> &'a mut crate::W<REG> {
        self.variant(OvdtRes3::Float)
    }
}
impl R {
    #[doc = "Bit 0 - Override RES0 Data Type Enable"]
    #[inline(always)]
    pub fn ovdt_en_res0(&self) -> OvdtEnRes0R {
        OvdtEnRes0R::new((self.bits & 1) != 0)
    }
    #[doc = "Bits 1:2 - Override RES0 Data Type"]
    #[inline(always)]
    pub fn ovdt_res0(&self) -> OvdtRes0R {
        OvdtRes0R::new(((self.bits >> 1) & 3) as u8)
    }
    #[doc = "Bit 8 - Override RES1 Data Type Enable"]
    #[inline(always)]
    pub fn ovdt_en_res1(&self) -> OvdtEnRes1R {
        OvdtEnRes1R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bits 9:10 - Override RES1 Data Type"]
    #[inline(always)]
    pub fn ovdt_res1(&self) -> OvdtRes1R {
        OvdtRes1R::new(((self.bits >> 9) & 3) as u8)
    }
    #[doc = "Bit 16 - Override RES2 Data Type Enable"]
    #[inline(always)]
    pub fn ovdt_en_res2(&self) -> OvdtEnRes2R {
        OvdtEnRes2R::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bits 17:18 - Override RES2 Data Type"]
    #[inline(always)]
    pub fn ovdt_res2(&self) -> OvdtRes2R {
        OvdtRes2R::new(((self.bits >> 17) & 3) as u8)
    }
    #[doc = "Bit 24 - Override RES3 Data Type Enable"]
    #[inline(always)]
    pub fn ovdt_en_res3(&self) -> OvdtEnRes3R {
        OvdtEnRes3R::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bits 25:26 - Override RES3 Data Type"]
    #[inline(always)]
    pub fn ovdt_res3(&self) -> OvdtRes3R {
        OvdtRes3R::new(((self.bits >> 25) & 3) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - Override RES0 Data Type Enable"]
    #[inline(always)]
    pub fn ovdt_en_res0(&mut self) -> OvdtEnRes0W<OpCtrlSpec> {
        OvdtEnRes0W::new(self, 0)
    }
    #[doc = "Bits 1:2 - Override RES0 Data Type"]
    #[inline(always)]
    pub fn ovdt_res0(&mut self) -> OvdtRes0W<OpCtrlSpec> {
        OvdtRes0W::new(self, 1)
    }
    #[doc = "Bit 8 - Override RES1 Data Type Enable"]
    #[inline(always)]
    pub fn ovdt_en_res1(&mut self) -> OvdtEnRes1W<OpCtrlSpec> {
        OvdtEnRes1W::new(self, 8)
    }
    #[doc = "Bits 9:10 - Override RES1 Data Type"]
    #[inline(always)]
    pub fn ovdt_res1(&mut self) -> OvdtRes1W<OpCtrlSpec> {
        OvdtRes1W::new(self, 9)
    }
    #[doc = "Bit 16 - Override RES2 Data Type Enable"]
    #[inline(always)]
    pub fn ovdt_en_res2(&mut self) -> OvdtEnRes2W<OpCtrlSpec> {
        OvdtEnRes2W::new(self, 16)
    }
    #[doc = "Bits 17:18 - Override RES2 Data Type"]
    #[inline(always)]
    pub fn ovdt_res2(&mut self) -> OvdtRes2W<OpCtrlSpec> {
        OvdtRes2W::new(self, 17)
    }
    #[doc = "Bit 24 - Override RES3 Data Type Enable"]
    #[inline(always)]
    pub fn ovdt_en_res3(&mut self) -> OvdtEnRes3W<OpCtrlSpec> {
        OvdtEnRes3W::new(self, 24)
    }
    #[doc = "Bits 25:26 - Override RES3 Data Type"]
    #[inline(always)]
    pub fn ovdt_res3(&mut self) -> OvdtRes3W<OpCtrlSpec> {
        OvdtRes3W::new(self, 25)
    }
}
#[doc = "Operation Control\n\nYou can [`read`](crate::Reg::read) this register and get [`op_ctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`op_ctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct OpCtrlSpec;
impl crate::RegisterSpec for OpCtrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`op_ctrl::R`](R) reader structure"]
impl crate::Readable for OpCtrlSpec {}
#[doc = "`write(|w| ..)` method takes [`op_ctrl::W`](W) writer structure"]
impl crate::Writable for OpCtrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets OP_CTRL to value 0"]
impl crate::Resettable for OpCtrlSpec {}

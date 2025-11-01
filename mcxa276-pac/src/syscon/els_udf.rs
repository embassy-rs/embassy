#[doc = "Register `ELS_UDF` reader"]
pub type R = crate::R<ElsUdfSpec>;
#[doc = "Register `ELS_UDF` writer"]
pub type W = crate::W<ElsUdfSpec>;
#[doc = "UDF KEY Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum KeySel {
    #[doc = "0: DUK: UID\\[127:0\\]^RTL_CONST1\\[127:0\\]"]
    Duk0 = 0,
    #[doc = "1: DUK: UID\\[127:0\\]^RTL_CONST1\\[127:0\\]"]
    Duk1 = 1,
    #[doc = "2: DeviceHSM"]
    DeviceHsm = 2,
    #[doc = "3: NXP_mRoT"]
    NxpMRoT = 3,
}
impl From<KeySel> for u8 {
    #[inline(always)]
    fn from(variant: KeySel) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for KeySel {
    type Ux = u8;
}
impl crate::IsEnum for KeySel {}
#[doc = "Field `KEY_SEL` reader - UDF KEY Select"]
pub type KeySelR = crate::FieldReader<KeySel>;
impl KeySelR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> KeySel {
        match self.bits {
            0 => KeySel::Duk0,
            1 => KeySel::Duk1,
            2 => KeySel::DeviceHsm,
            3 => KeySel::NxpMRoT,
            _ => unreachable!(),
        }
    }
    #[doc = "DUK: UID\\[127:0\\]^RTL_CONST1\\[127:0\\]"]
    #[inline(always)]
    pub fn is_duk_0(&self) -> bool {
        *self == KeySel::Duk0
    }
    #[doc = "DUK: UID\\[127:0\\]^RTL_CONST1\\[127:0\\]"]
    #[inline(always)]
    pub fn is_duk_1(&self) -> bool {
        *self == KeySel::Duk1
    }
    #[doc = "DeviceHSM"]
    #[inline(always)]
    pub fn is_device_hsm(&self) -> bool {
        *self == KeySel::DeviceHsm
    }
    #[doc = "NXP_mRoT"]
    #[inline(always)]
    pub fn is_nxp_m_ro_t(&self) -> bool {
        *self == KeySel::NxpMRoT
    }
}
#[doc = "Field `KEY_SEL` writer - UDF KEY Select"]
pub type KeySelW<'a, REG> = crate::FieldWriter<'a, REG, 2, KeySel, crate::Safe>;
impl<'a, REG> KeySelW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "DUK: UID\\[127:0\\]^RTL_CONST1\\[127:0\\]"]
    #[inline(always)]
    pub fn duk_0(self) -> &'a mut crate::W<REG> {
        self.variant(KeySel::Duk0)
    }
    #[doc = "DUK: UID\\[127:0\\]^RTL_CONST1\\[127:0\\]"]
    #[inline(always)]
    pub fn duk_1(self) -> &'a mut crate::W<REG> {
        self.variant(KeySel::Duk1)
    }
    #[doc = "DeviceHSM"]
    #[inline(always)]
    pub fn device_hsm(self) -> &'a mut crate::W<REG> {
        self.variant(KeySel::DeviceHsm)
    }
    #[doc = "NXP_mRoT"]
    #[inline(always)]
    pub fn nxp_m_ro_t(self) -> &'a mut crate::W<REG> {
        self.variant(KeySel::NxpMRoT)
    }
}
#[doc = "UID register hidden control. Write values other than 1010b, locked the write of UID_HIDDEN until a system reset.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum UidHidden {
    #[doc = "10: Enable the access of UID\\[127:0\\] register. All other value, disable the read/write of UID\\[127:0\\] register."]
    UidHidden = 10,
}
impl From<UidHidden> for u8 {
    #[inline(always)]
    fn from(variant: UidHidden) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for UidHidden {
    type Ux = u8;
}
impl crate::IsEnum for UidHidden {}
#[doc = "Field `UID_HIDDEN` reader - UID register hidden control. Write values other than 1010b, locked the write of UID_HIDDEN until a system reset."]
pub type UidHiddenR = crate::FieldReader<UidHidden>;
impl UidHiddenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<UidHidden> {
        match self.bits {
            10 => Some(UidHidden::UidHidden),
            _ => None,
        }
    }
    #[doc = "Enable the access of UID\\[127:0\\] register. All other value, disable the read/write of UID\\[127:0\\] register."]
    #[inline(always)]
    pub fn is_uid_hidden(&self) -> bool {
        *self == UidHidden::UidHidden
    }
}
#[doc = "Field `UID_HIDDEN` writer - UID register hidden control. Write values other than 1010b, locked the write of UID_HIDDEN until a system reset."]
pub type UidHiddenW<'a, REG> = crate::FieldWriter<'a, REG, 4, UidHidden>;
impl<'a, REG> UidHiddenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Enable the access of UID\\[127:0\\] register. All other value, disable the read/write of UID\\[127:0\\] register."]
    #[inline(always)]
    pub fn uid_hidden(self) -> &'a mut crate::W<REG> {
        self.variant(UidHidden::UidHidden)
    }
}
#[doc = "UDF register hidden control. Write values other than 1010b, locked the write of UDF_HIDDEN until a system reset.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum UdfHidden {
    #[doc = "10: Enable the access of UDF register from APB bus. All other value, disable the read/write of UDF register from UDF APB bus."]
    UdfHidden = 10,
}
impl From<UdfHidden> for u8 {
    #[inline(always)]
    fn from(variant: UdfHidden) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for UdfHidden {
    type Ux = u8;
}
impl crate::IsEnum for UdfHidden {}
#[doc = "Field `UDF_HIDDEN` reader - UDF register hidden control. Write values other than 1010b, locked the write of UDF_HIDDEN until a system reset."]
pub type UdfHiddenR = crate::FieldReader<UdfHidden>;
impl UdfHiddenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<UdfHidden> {
        match self.bits {
            10 => Some(UdfHidden::UdfHidden),
            _ => None,
        }
    }
    #[doc = "Enable the access of UDF register from APB bus. All other value, disable the read/write of UDF register from UDF APB bus."]
    #[inline(always)]
    pub fn is_udf_hidden(&self) -> bool {
        *self == UdfHidden::UdfHidden
    }
}
#[doc = "Field `UDF_HIDDEN` writer - UDF register hidden control. Write values other than 1010b, locked the write of UDF_HIDDEN until a system reset."]
pub type UdfHiddenW<'a, REG> = crate::FieldWriter<'a, REG, 4, UdfHidden>;
impl<'a, REG> UdfHiddenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Enable the access of UDF register from APB bus. All other value, disable the read/write of UDF register from UDF APB bus."]
    #[inline(always)]
    pub fn udf_hidden(self) -> &'a mut crate::W<REG> {
        self.variant(UdfHidden::UdfHidden)
    }
}
impl R {
    #[doc = "Bits 0:1 - UDF KEY Select"]
    #[inline(always)]
    pub fn key_sel(&self) -> KeySelR {
        KeySelR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 24:27 - UID register hidden control. Write values other than 1010b, locked the write of UID_HIDDEN until a system reset."]
    #[inline(always)]
    pub fn uid_hidden(&self) -> UidHiddenR {
        UidHiddenR::new(((self.bits >> 24) & 0x0f) as u8)
    }
    #[doc = "Bits 28:31 - UDF register hidden control. Write values other than 1010b, locked the write of UDF_HIDDEN until a system reset."]
    #[inline(always)]
    pub fn udf_hidden(&self) -> UdfHiddenR {
        UdfHiddenR::new(((self.bits >> 28) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - UDF KEY Select"]
    #[inline(always)]
    pub fn key_sel(&mut self) -> KeySelW<ElsUdfSpec> {
        KeySelW::new(self, 0)
    }
    #[doc = "Bits 24:27 - UID register hidden control. Write values other than 1010b, locked the write of UID_HIDDEN until a system reset."]
    #[inline(always)]
    pub fn uid_hidden(&mut self) -> UidHiddenW<ElsUdfSpec> {
        UidHiddenW::new(self, 24)
    }
    #[doc = "Bits 28:31 - UDF register hidden control. Write values other than 1010b, locked the write of UDF_HIDDEN until a system reset."]
    #[inline(always)]
    pub fn udf_hidden(&mut self) -> UdfHiddenW<ElsUdfSpec> {
        UdfHiddenW::new(self, 28)
    }
}
#[doc = "UDF Control\n\nYou can [`read`](crate::Reg::read) this register and get [`els_udf::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`els_udf::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ElsUdfSpec;
impl crate::RegisterSpec for ElsUdfSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`els_udf::R`](R) reader structure"]
impl crate::Readable for ElsUdfSpec {}
#[doc = "`write(|w| ..)` method takes [`els_udf::W`](W) writer structure"]
impl crate::Writable for ElsUdfSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets ELS_UDF to value 0"]
impl crate::Resettable for ElsUdfSpec {}

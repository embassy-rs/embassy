#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    _reserved0: [u8; 0x0200],
    sgi_datin0a: SgiDatin0a,
    sgi_datin0b: SgiDatin0b,
    sgi_datin0c: SgiDatin0c,
    sgi_datin0d: SgiDatin0d,
    sgi_datin1a: SgiDatin1a,
    sgi_datin1b: SgiDatin1b,
    sgi_datin1c: SgiDatin1c,
    sgi_datin1d: SgiDatin1d,
    sgi_datin2a: SgiDatin2a,
    sgi_datin2b: SgiDatin2b,
    sgi_datin2c: SgiDatin2c,
    sgi_datin2d: SgiDatin2d,
    sgi_datin3a: SgiDatin3a,
    sgi_datin3b: SgiDatin3b,
    sgi_datin3c: SgiDatin3c,
    sgi_datin3d: SgiDatin3d,
    sgi_key0a: SgiKey0a,
    sgi_key0b: SgiKey0b,
    sgi_key0c: SgiKey0c,
    sgi_key0d: SgiKey0d,
    sgi_key1a: SgiKey1a,
    sgi_key1b: SgiKey1b,
    sgi_key1c: SgiKey1c,
    sgi_key1d: SgiKey1d,
    sgi_key2a: SgiKey2a,
    sgi_key2b: SgiKey2b,
    sgi_key2c: SgiKey2c,
    sgi_key2d: SgiKey2d,
    sgi_key3a: SgiKey3a,
    sgi_key3b: SgiKey3b,
    sgi_key3c: SgiKey3c,
    sgi_key3d: SgiKey3d,
    sgi_key4a: SgiKey4a,
    sgi_key4b: SgiKey4b,
    sgi_key4c: SgiKey4c,
    sgi_key4d: SgiKey4d,
    sgi_key5a: SgiKey5a,
    sgi_key5b: SgiKey5b,
    sgi_key5c: SgiKey5c,
    sgi_key5d: SgiKey5d,
    sgi_key6a: SgiKey6a,
    sgi_key6b: SgiKey6b,
    sgi_key6c: SgiKey6c,
    sgi_key6d: SgiKey6d,
    sgi_key7a: SgiKey7a,
    sgi_key7b: SgiKey7b,
    sgi_key7c: SgiKey7c,
    sgi_key7d: SgiKey7d,
    sgi_datouta: SgiDatouta,
    sgi_datoutb: SgiDatoutb,
    sgi_datoutc: SgiDatoutc,
    sgi_datoutd: SgiDatoutd,
    _reserved52: [u8; 0x0930],
    sgi_status: SgiStatus,
    sgi_count: SgiCount,
    sgi_keychk: SgiKeychk,
    _reserved55: [u8; 0xf4],
    sgi_ctrl: SgiCtrl,
    sgi_ctrl2: SgiCtrl2,
    sgi_dummy_ctrl: SgiDummyCtrl,
    sgi_sfr_sw_mask: SgiSfrSwMask,
    sgi_sfrseed: SgiSfrseed,
    sgi_sha2_ctrl: SgiSha2Ctrl,
    sgi_sha_fifo: SgiShaFifo,
    sgi_config: SgiConfig,
    sgi_config2: SgiConfig2,
    sgi_auto_mode: SgiAutoMode,
    sgi_auto_dma_ctrl: SgiAutoDmaCtrl,
    _reserved66: [u8; 0x04],
    sgi_prng_sw_seed: SgiPrngSwSeed,
    _reserved67: [u8; 0x0c],
    sgi_key_ctrl: SgiKeyCtrl,
    _reserved68: [u8; 0x0c],
    sgi_key_wrap: SgiKeyWrap,
    _reserved69: [u8; 0x01b4],
    sgi_version: SgiVersion,
    _reserved70: [u8; 0xb4],
    sgi_access_err: SgiAccessErr,
    sgi_access_err_clr: SgiAccessErrClr,
    _reserved72: [u8; 0x18],
    sgi_int_status: SgiIntStatus,
    sgi_int_enable: SgiIntEnable,
    sgi_int_status_clr: SgiIntStatusClr,
    sgi_int_status_set: SgiIntStatusSet,
    _reserved76: [u8; 0x0c],
    sgi_module_id: SgiModuleId,
}
impl RegisterBlock {
    #[doc = "0x200 - Input Data register 0 - Word-3"]
    #[inline(always)]
    pub const fn sgi_datin0a(&self) -> &SgiDatin0a {
        &self.sgi_datin0a
    }
    #[doc = "0x204 - Input Data register 0 - Word-2"]
    #[inline(always)]
    pub const fn sgi_datin0b(&self) -> &SgiDatin0b {
        &self.sgi_datin0b
    }
    #[doc = "0x208 - Input Data register 0 - Word-1"]
    #[inline(always)]
    pub const fn sgi_datin0c(&self) -> &SgiDatin0c {
        &self.sgi_datin0c
    }
    #[doc = "0x20c - Input Data register 0 - Word-0"]
    #[inline(always)]
    pub const fn sgi_datin0d(&self) -> &SgiDatin0d {
        &self.sgi_datin0d
    }
    #[doc = "0x210 - Input Data register 1 - Word-3"]
    #[inline(always)]
    pub const fn sgi_datin1a(&self) -> &SgiDatin1a {
        &self.sgi_datin1a
    }
    #[doc = "0x214 - Input Data register 1 - Word-2"]
    #[inline(always)]
    pub const fn sgi_datin1b(&self) -> &SgiDatin1b {
        &self.sgi_datin1b
    }
    #[doc = "0x218 - Input Data register 1 - Word-1"]
    #[inline(always)]
    pub const fn sgi_datin1c(&self) -> &SgiDatin1c {
        &self.sgi_datin1c
    }
    #[doc = "0x21c - Input Data register 1 - Word-0"]
    #[inline(always)]
    pub const fn sgi_datin1d(&self) -> &SgiDatin1d {
        &self.sgi_datin1d
    }
    #[doc = "0x220 - Input Data register 2 - Word-3"]
    #[inline(always)]
    pub const fn sgi_datin2a(&self) -> &SgiDatin2a {
        &self.sgi_datin2a
    }
    #[doc = "0x224 - Input Data register 2 - Word-2"]
    #[inline(always)]
    pub const fn sgi_datin2b(&self) -> &SgiDatin2b {
        &self.sgi_datin2b
    }
    #[doc = "0x228 - Input Data register 2 - Word-1"]
    #[inline(always)]
    pub const fn sgi_datin2c(&self) -> &SgiDatin2c {
        &self.sgi_datin2c
    }
    #[doc = "0x22c - Input Data register 2 - Word-0"]
    #[inline(always)]
    pub const fn sgi_datin2d(&self) -> &SgiDatin2d {
        &self.sgi_datin2d
    }
    #[doc = "0x230 - Input Data register 3 - Word-3"]
    #[inline(always)]
    pub const fn sgi_datin3a(&self) -> &SgiDatin3a {
        &self.sgi_datin3a
    }
    #[doc = "0x234 - Input Data register 3 - Word-2"]
    #[inline(always)]
    pub const fn sgi_datin3b(&self) -> &SgiDatin3b {
        &self.sgi_datin3b
    }
    #[doc = "0x238 - Input Data register 3 - Word-1"]
    #[inline(always)]
    pub const fn sgi_datin3c(&self) -> &SgiDatin3c {
        &self.sgi_datin3c
    }
    #[doc = "0x23c - Input Data register 3 - Word-0"]
    #[inline(always)]
    pub const fn sgi_datin3d(&self) -> &SgiDatin3d {
        &self.sgi_datin3d
    }
    #[doc = "0x240 - Input Key register 0 - Word-3"]
    #[inline(always)]
    pub const fn sgi_key0a(&self) -> &SgiKey0a {
        &self.sgi_key0a
    }
    #[doc = "0x244 - Input Key register 0 - Word-2"]
    #[inline(always)]
    pub const fn sgi_key0b(&self) -> &SgiKey0b {
        &self.sgi_key0b
    }
    #[doc = "0x248 - Input Key register 0 - Word-1"]
    #[inline(always)]
    pub const fn sgi_key0c(&self) -> &SgiKey0c {
        &self.sgi_key0c
    }
    #[doc = "0x24c - Input Key register 0 - Word-0"]
    #[inline(always)]
    pub const fn sgi_key0d(&self) -> &SgiKey0d {
        &self.sgi_key0d
    }
    #[doc = "0x250 - Input Key register 1 - Word-3"]
    #[inline(always)]
    pub const fn sgi_key1a(&self) -> &SgiKey1a {
        &self.sgi_key1a
    }
    #[doc = "0x254 - Input Key register 1 - Word-2"]
    #[inline(always)]
    pub const fn sgi_key1b(&self) -> &SgiKey1b {
        &self.sgi_key1b
    }
    #[doc = "0x258 - Input Key register 1 - Word-1"]
    #[inline(always)]
    pub const fn sgi_key1c(&self) -> &SgiKey1c {
        &self.sgi_key1c
    }
    #[doc = "0x25c - Input Key register 1 - Word-0"]
    #[inline(always)]
    pub const fn sgi_key1d(&self) -> &SgiKey1d {
        &self.sgi_key1d
    }
    #[doc = "0x260 - Input Key register 2 - Word-3"]
    #[inline(always)]
    pub const fn sgi_key2a(&self) -> &SgiKey2a {
        &self.sgi_key2a
    }
    #[doc = "0x264 - Input Key register 2 - Word-2"]
    #[inline(always)]
    pub const fn sgi_key2b(&self) -> &SgiKey2b {
        &self.sgi_key2b
    }
    #[doc = "0x268 - Input Key register 2 - Word-1"]
    #[inline(always)]
    pub const fn sgi_key2c(&self) -> &SgiKey2c {
        &self.sgi_key2c
    }
    #[doc = "0x26c - Input Key register 2 - Word-0"]
    #[inline(always)]
    pub const fn sgi_key2d(&self) -> &SgiKey2d {
        &self.sgi_key2d
    }
    #[doc = "0x270 - Input Key register 3 - Word-3"]
    #[inline(always)]
    pub const fn sgi_key3a(&self) -> &SgiKey3a {
        &self.sgi_key3a
    }
    #[doc = "0x274 - Input Key register 3 - Word-2"]
    #[inline(always)]
    pub const fn sgi_key3b(&self) -> &SgiKey3b {
        &self.sgi_key3b
    }
    #[doc = "0x278 - Input Key register 3 - Word-1"]
    #[inline(always)]
    pub const fn sgi_key3c(&self) -> &SgiKey3c {
        &self.sgi_key3c
    }
    #[doc = "0x27c - Input Key register 3 - Word-0"]
    #[inline(always)]
    pub const fn sgi_key3d(&self) -> &SgiKey3d {
        &self.sgi_key3d
    }
    #[doc = "0x280 - Input Key register 4 - Word-3"]
    #[inline(always)]
    pub const fn sgi_key4a(&self) -> &SgiKey4a {
        &self.sgi_key4a
    }
    #[doc = "0x284 - Input Key register 4 - Word-2"]
    #[inline(always)]
    pub const fn sgi_key4b(&self) -> &SgiKey4b {
        &self.sgi_key4b
    }
    #[doc = "0x288 - Input Key register 4 - Word-1"]
    #[inline(always)]
    pub const fn sgi_key4c(&self) -> &SgiKey4c {
        &self.sgi_key4c
    }
    #[doc = "0x28c - Input Key register 4 - Word-0"]
    #[inline(always)]
    pub const fn sgi_key4d(&self) -> &SgiKey4d {
        &self.sgi_key4d
    }
    #[doc = "0x290 - Input Key register 5 - Word-3"]
    #[inline(always)]
    pub const fn sgi_key5a(&self) -> &SgiKey5a {
        &self.sgi_key5a
    }
    #[doc = "0x294 - Input Key register 5 - Word-2"]
    #[inline(always)]
    pub const fn sgi_key5b(&self) -> &SgiKey5b {
        &self.sgi_key5b
    }
    #[doc = "0x298 - Input Key register 5 - Word-1"]
    #[inline(always)]
    pub const fn sgi_key5c(&self) -> &SgiKey5c {
        &self.sgi_key5c
    }
    #[doc = "0x29c - Input Key register 5 - Word-0"]
    #[inline(always)]
    pub const fn sgi_key5d(&self) -> &SgiKey5d {
        &self.sgi_key5d
    }
    #[doc = "0x2a0 - Input Key register 6 - Word-3"]
    #[inline(always)]
    pub const fn sgi_key6a(&self) -> &SgiKey6a {
        &self.sgi_key6a
    }
    #[doc = "0x2a4 - Input Key register 6 - Word-2"]
    #[inline(always)]
    pub const fn sgi_key6b(&self) -> &SgiKey6b {
        &self.sgi_key6b
    }
    #[doc = "0x2a8 - Input Key register 6 - Word-1"]
    #[inline(always)]
    pub const fn sgi_key6c(&self) -> &SgiKey6c {
        &self.sgi_key6c
    }
    #[doc = "0x2ac - Input Key register 6 - Word-0"]
    #[inline(always)]
    pub const fn sgi_key6d(&self) -> &SgiKey6d {
        &self.sgi_key6d
    }
    #[doc = "0x2b0 - Input Key register 7 - Word-3"]
    #[inline(always)]
    pub const fn sgi_key7a(&self) -> &SgiKey7a {
        &self.sgi_key7a
    }
    #[doc = "0x2b4 - Input Key register 7 - Word-2"]
    #[inline(always)]
    pub const fn sgi_key7b(&self) -> &SgiKey7b {
        &self.sgi_key7b
    }
    #[doc = "0x2b8 - Input Key register 7 - Word-1"]
    #[inline(always)]
    pub const fn sgi_key7c(&self) -> &SgiKey7c {
        &self.sgi_key7c
    }
    #[doc = "0x2bc - Input Key register 7 - Word-0"]
    #[inline(always)]
    pub const fn sgi_key7d(&self) -> &SgiKey7d {
        &self.sgi_key7d
    }
    #[doc = "0x2c0 - Output Data register - Word-3"]
    #[inline(always)]
    pub const fn sgi_datouta(&self) -> &SgiDatouta {
        &self.sgi_datouta
    }
    #[doc = "0x2c4 - Output Data register - Word-2"]
    #[inline(always)]
    pub const fn sgi_datoutb(&self) -> &SgiDatoutb {
        &self.sgi_datoutb
    }
    #[doc = "0x2c8 - Output Data register - Word-1"]
    #[inline(always)]
    pub const fn sgi_datoutc(&self) -> &SgiDatoutc {
        &self.sgi_datoutc
    }
    #[doc = "0x2cc - Output Data register - Word-0"]
    #[inline(always)]
    pub const fn sgi_datoutd(&self) -> &SgiDatoutd {
        &self.sgi_datoutd
    }
    #[doc = "0xc00 - Status register"]
    #[inline(always)]
    pub const fn sgi_status(&self) -> &SgiStatus {
        &self.sgi_status
    }
    #[doc = "0xc04 - Calculation counter"]
    #[inline(always)]
    pub const fn sgi_count(&self) -> &SgiCount {
        &self.sgi_count
    }
    #[doc = "0xc08 - Key checksum register"]
    #[inline(always)]
    pub const fn sgi_keychk(&self) -> &SgiKeychk {
        &self.sgi_keychk
    }
    #[doc = "0xd00 - SGI Control register"]
    #[inline(always)]
    pub const fn sgi_ctrl(&self) -> &SgiCtrl {
        &self.sgi_ctrl
    }
    #[doc = "0xd04 - SGI Control register 2"]
    #[inline(always)]
    pub const fn sgi_ctrl2(&self) -> &SgiCtrl2 {
        &self.sgi_ctrl2
    }
    #[doc = "0xd08 - Configuration of dummy controls"]
    #[inline(always)]
    pub const fn sgi_dummy_ctrl(&self) -> &SgiDummyCtrl {
        &self.sgi_dummy_ctrl
    }
    #[doc = "0xd0c - Sofware Assisted Masking register ."]
    #[inline(always)]
    pub const fn sgi_sfr_sw_mask(&self) -> &SgiSfrSwMask {
        &self.sgi_sfr_sw_mask
    }
    #[doc = "0xd10 - SFRSEED register for SFRMASK feature."]
    #[inline(always)]
    pub const fn sgi_sfrseed(&self) -> &SgiSfrseed {
        &self.sgi_sfrseed
    }
    #[doc = "0xd14 - SHA Control Register"]
    #[inline(always)]
    pub const fn sgi_sha2_ctrl(&self) -> &SgiSha2Ctrl {
        &self.sgi_sha2_ctrl
    }
    #[doc = "0xd18 - SHA FIFO lower-bank low"]
    #[inline(always)]
    pub const fn sgi_sha_fifo(&self) -> &SgiShaFifo {
        &self.sgi_sha_fifo
    }
    #[doc = "0xd1c - SHA Configuration Reg"]
    #[inline(always)]
    pub const fn sgi_config(&self) -> &SgiConfig {
        &self.sgi_config
    }
    #[doc = "0xd20 - SHA Configuration 2 Reg"]
    #[inline(always)]
    pub const fn sgi_config2(&self) -> &SgiConfig2 {
        &self.sgi_config2
    }
    #[doc = "0xd24 - SGI Auto Mode Control register"]
    #[inline(always)]
    pub const fn sgi_auto_mode(&self) -> &SgiAutoMode {
        &self.sgi_auto_mode
    }
    #[doc = "0xd28 - SGI Auto Mode Control register"]
    #[inline(always)]
    pub const fn sgi_auto_dma_ctrl(&self) -> &SgiAutoDmaCtrl {
        &self.sgi_auto_dma_ctrl
    }
    #[doc = "0xd30 - SGI internal PRNG SW seeding register"]
    #[inline(always)]
    pub const fn sgi_prng_sw_seed(&self) -> &SgiPrngSwSeed {
        &self.sgi_prng_sw_seed
    }
    #[doc = "0xd40 - SGI Key Control SFR"]
    #[inline(always)]
    pub const fn sgi_key_ctrl(&self) -> &SgiKeyCtrl {
        &self.sgi_key_ctrl
    }
    #[doc = "0xd50 - Wrapped key read SFR"]
    #[inline(always)]
    pub const fn sgi_key_wrap(&self) -> &SgiKeyWrap {
        &self.sgi_key_wrap
    }
    #[doc = "0xf08 - SGI Version"]
    #[inline(always)]
    pub const fn sgi_version(&self) -> &SgiVersion {
        &self.sgi_version
    }
    #[doc = "0xfc0 - Access Error"]
    #[inline(always)]
    pub const fn sgi_access_err(&self) -> &SgiAccessErr {
        &self.sgi_access_err
    }
    #[doc = "0xfc4 - Clear Access Error"]
    #[inline(always)]
    pub const fn sgi_access_err_clr(&self) -> &SgiAccessErrClr {
        &self.sgi_access_err_clr
    }
    #[doc = "0xfe0 - Interrupt status"]
    #[inline(always)]
    pub const fn sgi_int_status(&self) -> &SgiIntStatus {
        &self.sgi_int_status
    }
    #[doc = "0xfe4 - Interrupt enable"]
    #[inline(always)]
    pub const fn sgi_int_enable(&self) -> &SgiIntEnable {
        &self.sgi_int_enable
    }
    #[doc = "0xfe8 - Interrupt status clear"]
    #[inline(always)]
    pub const fn sgi_int_status_clr(&self) -> &SgiIntStatusClr {
        &self.sgi_int_status_clr
    }
    #[doc = "0xfec - Interrupt status set"]
    #[inline(always)]
    pub const fn sgi_int_status_set(&self) -> &SgiIntStatusSet {
        &self.sgi_int_status_set
    }
    #[doc = "0xffc - Module ID"]
    #[inline(always)]
    pub const fn sgi_module_id(&self) -> &SgiModuleId {
        &self.sgi_module_id
    }
}
#[doc = "sgi_datin0a (rw) register accessor: Input Data register 0 - Word-3\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin0a::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin0a::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_datin0a`] module"]
#[doc(alias = "sgi_datin0a")]
pub type SgiDatin0a = crate::Reg<sgi_datin0a::SgiDatin0aSpec>;
#[doc = "Input Data register 0 - Word-3"]
pub mod sgi_datin0a;
#[doc = "sgi_datin0b (rw) register accessor: Input Data register 0 - Word-2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin0b::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin0b::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_datin0b`] module"]
#[doc(alias = "sgi_datin0b")]
pub type SgiDatin0b = crate::Reg<sgi_datin0b::SgiDatin0bSpec>;
#[doc = "Input Data register 0 - Word-2"]
pub mod sgi_datin0b;
#[doc = "sgi_datin0c (rw) register accessor: Input Data register 0 - Word-1\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin0c::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin0c::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_datin0c`] module"]
#[doc(alias = "sgi_datin0c")]
pub type SgiDatin0c = crate::Reg<sgi_datin0c::SgiDatin0cSpec>;
#[doc = "Input Data register 0 - Word-1"]
pub mod sgi_datin0c;
#[doc = "sgi_datin0d (rw) register accessor: Input Data register 0 - Word-0\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin0d::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin0d::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_datin0d`] module"]
#[doc(alias = "sgi_datin0d")]
pub type SgiDatin0d = crate::Reg<sgi_datin0d::SgiDatin0dSpec>;
#[doc = "Input Data register 0 - Word-0"]
pub mod sgi_datin0d;
#[doc = "sgi_datin1a (rw) register accessor: Input Data register 1 - Word-3\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin1a::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin1a::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_datin1a`] module"]
#[doc(alias = "sgi_datin1a")]
pub type SgiDatin1a = crate::Reg<sgi_datin1a::SgiDatin1aSpec>;
#[doc = "Input Data register 1 - Word-3"]
pub mod sgi_datin1a;
#[doc = "sgi_datin1b (rw) register accessor: Input Data register 1 - Word-2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin1b::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin1b::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_datin1b`] module"]
#[doc(alias = "sgi_datin1b")]
pub type SgiDatin1b = crate::Reg<sgi_datin1b::SgiDatin1bSpec>;
#[doc = "Input Data register 1 - Word-2"]
pub mod sgi_datin1b;
#[doc = "sgi_datin1c (rw) register accessor: Input Data register 1 - Word-1\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin1c::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin1c::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_datin1c`] module"]
#[doc(alias = "sgi_datin1c")]
pub type SgiDatin1c = crate::Reg<sgi_datin1c::SgiDatin1cSpec>;
#[doc = "Input Data register 1 - Word-1"]
pub mod sgi_datin1c;
#[doc = "sgi_datin1d (rw) register accessor: Input Data register 1 - Word-0\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin1d::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin1d::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_datin1d`] module"]
#[doc(alias = "sgi_datin1d")]
pub type SgiDatin1d = crate::Reg<sgi_datin1d::SgiDatin1dSpec>;
#[doc = "Input Data register 1 - Word-0"]
pub mod sgi_datin1d;
#[doc = "sgi_datin2a (rw) register accessor: Input Data register 2 - Word-3\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin2a::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin2a::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_datin2a`] module"]
#[doc(alias = "sgi_datin2a")]
pub type SgiDatin2a = crate::Reg<sgi_datin2a::SgiDatin2aSpec>;
#[doc = "Input Data register 2 - Word-3"]
pub mod sgi_datin2a;
#[doc = "sgi_datin2b (rw) register accessor: Input Data register 2 - Word-2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin2b::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin2b::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_datin2b`] module"]
#[doc(alias = "sgi_datin2b")]
pub type SgiDatin2b = crate::Reg<sgi_datin2b::SgiDatin2bSpec>;
#[doc = "Input Data register 2 - Word-2"]
pub mod sgi_datin2b;
#[doc = "sgi_datin2c (rw) register accessor: Input Data register 2 - Word-1\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin2c::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin2c::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_datin2c`] module"]
#[doc(alias = "sgi_datin2c")]
pub type SgiDatin2c = crate::Reg<sgi_datin2c::SgiDatin2cSpec>;
#[doc = "Input Data register 2 - Word-1"]
pub mod sgi_datin2c;
#[doc = "sgi_datin2d (rw) register accessor: Input Data register 2 - Word-0\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin2d::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin2d::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_datin2d`] module"]
#[doc(alias = "sgi_datin2d")]
pub type SgiDatin2d = crate::Reg<sgi_datin2d::SgiDatin2dSpec>;
#[doc = "Input Data register 2 - Word-0"]
pub mod sgi_datin2d;
#[doc = "sgi_datin3a (rw) register accessor: Input Data register 3 - Word-3\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin3a::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin3a::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_datin3a`] module"]
#[doc(alias = "sgi_datin3a")]
pub type SgiDatin3a = crate::Reg<sgi_datin3a::SgiDatin3aSpec>;
#[doc = "Input Data register 3 - Word-3"]
pub mod sgi_datin3a;
#[doc = "sgi_datin3b (rw) register accessor: Input Data register 3 - Word-2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin3b::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin3b::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_datin3b`] module"]
#[doc(alias = "sgi_datin3b")]
pub type SgiDatin3b = crate::Reg<sgi_datin3b::SgiDatin3bSpec>;
#[doc = "Input Data register 3 - Word-2"]
pub mod sgi_datin3b;
#[doc = "sgi_datin3c (rw) register accessor: Input Data register 3 - Word-1\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin3c::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin3c::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_datin3c`] module"]
#[doc(alias = "sgi_datin3c")]
pub type SgiDatin3c = crate::Reg<sgi_datin3c::SgiDatin3cSpec>;
#[doc = "Input Data register 3 - Word-1"]
pub mod sgi_datin3c;
#[doc = "sgi_datin3d (rw) register accessor: Input Data register 3 - Word-0\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin3d::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin3d::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_datin3d`] module"]
#[doc(alias = "sgi_datin3d")]
pub type SgiDatin3d = crate::Reg<sgi_datin3d::SgiDatin3dSpec>;
#[doc = "Input Data register 3 - Word-0"]
pub mod sgi_datin3d;
#[doc = "sgi_key0a (rw) register accessor: Input Key register 0 - Word-3\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key0a::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key0a::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key0a`] module"]
#[doc(alias = "sgi_key0a")]
pub type SgiKey0a = crate::Reg<sgi_key0a::SgiKey0aSpec>;
#[doc = "Input Key register 0 - Word-3"]
pub mod sgi_key0a;
#[doc = "sgi_key0b (rw) register accessor: Input Key register 0 - Word-2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key0b::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key0b::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key0b`] module"]
#[doc(alias = "sgi_key0b")]
pub type SgiKey0b = crate::Reg<sgi_key0b::SgiKey0bSpec>;
#[doc = "Input Key register 0 - Word-2"]
pub mod sgi_key0b;
#[doc = "sgi_key0c (rw) register accessor: Input Key register 0 - Word-1\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key0c::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key0c::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key0c`] module"]
#[doc(alias = "sgi_key0c")]
pub type SgiKey0c = crate::Reg<sgi_key0c::SgiKey0cSpec>;
#[doc = "Input Key register 0 - Word-1"]
pub mod sgi_key0c;
#[doc = "sgi_key0d (rw) register accessor: Input Key register 0 - Word-0\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key0d::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key0d::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key0d`] module"]
#[doc(alias = "sgi_key0d")]
pub type SgiKey0d = crate::Reg<sgi_key0d::SgiKey0dSpec>;
#[doc = "Input Key register 0 - Word-0"]
pub mod sgi_key0d;
#[doc = "sgi_key1a (rw) register accessor: Input Key register 1 - Word-3\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key1a::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key1a::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key1a`] module"]
#[doc(alias = "sgi_key1a")]
pub type SgiKey1a = crate::Reg<sgi_key1a::SgiKey1aSpec>;
#[doc = "Input Key register 1 - Word-3"]
pub mod sgi_key1a;
#[doc = "sgi_key1b (rw) register accessor: Input Key register 1 - Word-2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key1b::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key1b::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key1b`] module"]
#[doc(alias = "sgi_key1b")]
pub type SgiKey1b = crate::Reg<sgi_key1b::SgiKey1bSpec>;
#[doc = "Input Key register 1 - Word-2"]
pub mod sgi_key1b;
#[doc = "sgi_key1c (rw) register accessor: Input Key register 1 - Word-1\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key1c::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key1c::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key1c`] module"]
#[doc(alias = "sgi_key1c")]
pub type SgiKey1c = crate::Reg<sgi_key1c::SgiKey1cSpec>;
#[doc = "Input Key register 1 - Word-1"]
pub mod sgi_key1c;
#[doc = "sgi_key1d (rw) register accessor: Input Key register 1 - Word-0\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key1d::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key1d::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key1d`] module"]
#[doc(alias = "sgi_key1d")]
pub type SgiKey1d = crate::Reg<sgi_key1d::SgiKey1dSpec>;
#[doc = "Input Key register 1 - Word-0"]
pub mod sgi_key1d;
#[doc = "sgi_key2a (rw) register accessor: Input Key register 2 - Word-3\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key2a::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key2a::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key2a`] module"]
#[doc(alias = "sgi_key2a")]
pub type SgiKey2a = crate::Reg<sgi_key2a::SgiKey2aSpec>;
#[doc = "Input Key register 2 - Word-3"]
pub mod sgi_key2a;
#[doc = "sgi_key2b (rw) register accessor: Input Key register 2 - Word-2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key2b::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key2b::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key2b`] module"]
#[doc(alias = "sgi_key2b")]
pub type SgiKey2b = crate::Reg<sgi_key2b::SgiKey2bSpec>;
#[doc = "Input Key register 2 - Word-2"]
pub mod sgi_key2b;
#[doc = "sgi_key2c (rw) register accessor: Input Key register 2 - Word-1\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key2c::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key2c::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key2c`] module"]
#[doc(alias = "sgi_key2c")]
pub type SgiKey2c = crate::Reg<sgi_key2c::SgiKey2cSpec>;
#[doc = "Input Key register 2 - Word-1"]
pub mod sgi_key2c;
#[doc = "sgi_key2d (rw) register accessor: Input Key register 2 - Word-0\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key2d::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key2d::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key2d`] module"]
#[doc(alias = "sgi_key2d")]
pub type SgiKey2d = crate::Reg<sgi_key2d::SgiKey2dSpec>;
#[doc = "Input Key register 2 - Word-0"]
pub mod sgi_key2d;
#[doc = "sgi_key3a (rw) register accessor: Input Key register 3 - Word-3\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key3a::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key3a::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key3a`] module"]
#[doc(alias = "sgi_key3a")]
pub type SgiKey3a = crate::Reg<sgi_key3a::SgiKey3aSpec>;
#[doc = "Input Key register 3 - Word-3"]
pub mod sgi_key3a;
#[doc = "sgi_key3b (rw) register accessor: Input Key register 3 - Word-2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key3b::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key3b::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key3b`] module"]
#[doc(alias = "sgi_key3b")]
pub type SgiKey3b = crate::Reg<sgi_key3b::SgiKey3bSpec>;
#[doc = "Input Key register 3 - Word-2"]
pub mod sgi_key3b;
#[doc = "sgi_key3c (rw) register accessor: Input Key register 3 - Word-1\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key3c::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key3c::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key3c`] module"]
#[doc(alias = "sgi_key3c")]
pub type SgiKey3c = crate::Reg<sgi_key3c::SgiKey3cSpec>;
#[doc = "Input Key register 3 - Word-1"]
pub mod sgi_key3c;
#[doc = "sgi_key3d (rw) register accessor: Input Key register 3 - Word-0\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key3d::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key3d::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key3d`] module"]
#[doc(alias = "sgi_key3d")]
pub type SgiKey3d = crate::Reg<sgi_key3d::SgiKey3dSpec>;
#[doc = "Input Key register 3 - Word-0"]
pub mod sgi_key3d;
#[doc = "sgi_key4a (rw) register accessor: Input Key register 4 - Word-3\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key4a::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key4a::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key4a`] module"]
#[doc(alias = "sgi_key4a")]
pub type SgiKey4a = crate::Reg<sgi_key4a::SgiKey4aSpec>;
#[doc = "Input Key register 4 - Word-3"]
pub mod sgi_key4a;
#[doc = "sgi_key4b (rw) register accessor: Input Key register 4 - Word-2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key4b::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key4b::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key4b`] module"]
#[doc(alias = "sgi_key4b")]
pub type SgiKey4b = crate::Reg<sgi_key4b::SgiKey4bSpec>;
#[doc = "Input Key register 4 - Word-2"]
pub mod sgi_key4b;
#[doc = "sgi_key4c (rw) register accessor: Input Key register 4 - Word-1\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key4c::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key4c::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key4c`] module"]
#[doc(alias = "sgi_key4c")]
pub type SgiKey4c = crate::Reg<sgi_key4c::SgiKey4cSpec>;
#[doc = "Input Key register 4 - Word-1"]
pub mod sgi_key4c;
#[doc = "sgi_key4d (rw) register accessor: Input Key register 4 - Word-0\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key4d::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key4d::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key4d`] module"]
#[doc(alias = "sgi_key4d")]
pub type SgiKey4d = crate::Reg<sgi_key4d::SgiKey4dSpec>;
#[doc = "Input Key register 4 - Word-0"]
pub mod sgi_key4d;
#[doc = "sgi_key5a (rw) register accessor: Input Key register 5 - Word-3\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key5a::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key5a::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key5a`] module"]
#[doc(alias = "sgi_key5a")]
pub type SgiKey5a = crate::Reg<sgi_key5a::SgiKey5aSpec>;
#[doc = "Input Key register 5 - Word-3"]
pub mod sgi_key5a;
#[doc = "sgi_key5b (rw) register accessor: Input Key register 5 - Word-2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key5b::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key5b::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key5b`] module"]
#[doc(alias = "sgi_key5b")]
pub type SgiKey5b = crate::Reg<sgi_key5b::SgiKey5bSpec>;
#[doc = "Input Key register 5 - Word-2"]
pub mod sgi_key5b;
#[doc = "sgi_key5c (rw) register accessor: Input Key register 5 - Word-1\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key5c::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key5c::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key5c`] module"]
#[doc(alias = "sgi_key5c")]
pub type SgiKey5c = crate::Reg<sgi_key5c::SgiKey5cSpec>;
#[doc = "Input Key register 5 - Word-1"]
pub mod sgi_key5c;
#[doc = "sgi_key5d (rw) register accessor: Input Key register 5 - Word-0\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key5d::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key5d::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key5d`] module"]
#[doc(alias = "sgi_key5d")]
pub type SgiKey5d = crate::Reg<sgi_key5d::SgiKey5dSpec>;
#[doc = "Input Key register 5 - Word-0"]
pub mod sgi_key5d;
#[doc = "sgi_key6a (rw) register accessor: Input Key register 6 - Word-3\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key6a::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key6a::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key6a`] module"]
#[doc(alias = "sgi_key6a")]
pub type SgiKey6a = crate::Reg<sgi_key6a::SgiKey6aSpec>;
#[doc = "Input Key register 6 - Word-3"]
pub mod sgi_key6a;
#[doc = "sgi_key6b (rw) register accessor: Input Key register 6 - Word-2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key6b::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key6b::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key6b`] module"]
#[doc(alias = "sgi_key6b")]
pub type SgiKey6b = crate::Reg<sgi_key6b::SgiKey6bSpec>;
#[doc = "Input Key register 6 - Word-2"]
pub mod sgi_key6b;
#[doc = "sgi_key6c (rw) register accessor: Input Key register 6 - Word-1\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key6c::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key6c::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key6c`] module"]
#[doc(alias = "sgi_key6c")]
pub type SgiKey6c = crate::Reg<sgi_key6c::SgiKey6cSpec>;
#[doc = "Input Key register 6 - Word-1"]
pub mod sgi_key6c;
#[doc = "sgi_key6d (rw) register accessor: Input Key register 6 - Word-0\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key6d::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key6d::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key6d`] module"]
#[doc(alias = "sgi_key6d")]
pub type SgiKey6d = crate::Reg<sgi_key6d::SgiKey6dSpec>;
#[doc = "Input Key register 6 - Word-0"]
pub mod sgi_key6d;
#[doc = "sgi_key7a (rw) register accessor: Input Key register 7 - Word-3\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key7a::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key7a::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key7a`] module"]
#[doc(alias = "sgi_key7a")]
pub type SgiKey7a = crate::Reg<sgi_key7a::SgiKey7aSpec>;
#[doc = "Input Key register 7 - Word-3"]
pub mod sgi_key7a;
#[doc = "sgi_key7b (rw) register accessor: Input Key register 7 - Word-2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key7b::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key7b::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key7b`] module"]
#[doc(alias = "sgi_key7b")]
pub type SgiKey7b = crate::Reg<sgi_key7b::SgiKey7bSpec>;
#[doc = "Input Key register 7 - Word-2"]
pub mod sgi_key7b;
#[doc = "sgi_key7c (rw) register accessor: Input Key register 7 - Word-1\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key7c::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key7c::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key7c`] module"]
#[doc(alias = "sgi_key7c")]
pub type SgiKey7c = crate::Reg<sgi_key7c::SgiKey7cSpec>;
#[doc = "Input Key register 7 - Word-1"]
pub mod sgi_key7c;
#[doc = "sgi_key7d (rw) register accessor: Input Key register 7 - Word-0\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key7d::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key7d::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key7d`] module"]
#[doc(alias = "sgi_key7d")]
pub type SgiKey7d = crate::Reg<sgi_key7d::SgiKey7dSpec>;
#[doc = "Input Key register 7 - Word-0"]
pub mod sgi_key7d;
#[doc = "sgi_datouta (rw) register accessor: Output Data register - Word-3\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datouta::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datouta::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_datouta`] module"]
#[doc(alias = "sgi_datouta")]
pub type SgiDatouta = crate::Reg<sgi_datouta::SgiDatoutaSpec>;
#[doc = "Output Data register - Word-3"]
pub mod sgi_datouta;
#[doc = "sgi_datoutb (rw) register accessor: Output Data register - Word-2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datoutb::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datoutb::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_datoutb`] module"]
#[doc(alias = "sgi_datoutb")]
pub type SgiDatoutb = crate::Reg<sgi_datoutb::SgiDatoutbSpec>;
#[doc = "Output Data register - Word-2"]
pub mod sgi_datoutb;
#[doc = "sgi_datoutc (rw) register accessor: Output Data register - Word-1\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datoutc::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datoutc::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_datoutc`] module"]
#[doc(alias = "sgi_datoutc")]
pub type SgiDatoutc = crate::Reg<sgi_datoutc::SgiDatoutcSpec>;
#[doc = "Output Data register - Word-1"]
pub mod sgi_datoutc;
#[doc = "sgi_datoutd (rw) register accessor: Output Data register - Word-0\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datoutd::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datoutd::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_datoutd`] module"]
#[doc(alias = "sgi_datoutd")]
pub type SgiDatoutd = crate::Reg<sgi_datoutd::SgiDatoutdSpec>;
#[doc = "Output Data register - Word-0"]
pub mod sgi_datoutd;
#[doc = "sgi_status (rw) register accessor: Status register\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_status::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_status::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_status`] module"]
#[doc(alias = "sgi_status")]
pub type SgiStatus = crate::Reg<sgi_status::SgiStatusSpec>;
#[doc = "Status register"]
pub mod sgi_status;
#[doc = "sgi_count (rw) register accessor: Calculation counter\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_count::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_count::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_count`] module"]
#[doc(alias = "sgi_count")]
pub type SgiCount = crate::Reg<sgi_count::SgiCountSpec>;
#[doc = "Calculation counter"]
pub mod sgi_count;
#[doc = "sgi_keychk (rw) register accessor: Key checksum register\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_keychk::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_keychk::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_keychk`] module"]
#[doc(alias = "sgi_keychk")]
pub type SgiKeychk = crate::Reg<sgi_keychk::SgiKeychkSpec>;
#[doc = "Key checksum register"]
pub mod sgi_keychk;
#[doc = "sgi_ctrl (rw) register accessor: SGI Control register\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_ctrl`] module"]
#[doc(alias = "sgi_ctrl")]
pub type SgiCtrl = crate::Reg<sgi_ctrl::SgiCtrlSpec>;
#[doc = "SGI Control register"]
pub mod sgi_ctrl;
#[doc = "sgi_ctrl2 (rw) register accessor: SGI Control register 2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_ctrl2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_ctrl2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_ctrl2`] module"]
#[doc(alias = "sgi_ctrl2")]
pub type SgiCtrl2 = crate::Reg<sgi_ctrl2::SgiCtrl2Spec>;
#[doc = "SGI Control register 2"]
pub mod sgi_ctrl2;
#[doc = "sgi_dummy_ctrl (rw) register accessor: Configuration of dummy controls\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_dummy_ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_dummy_ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_dummy_ctrl`] module"]
#[doc(alias = "sgi_dummy_ctrl")]
pub type SgiDummyCtrl = crate::Reg<sgi_dummy_ctrl::SgiDummyCtrlSpec>;
#[doc = "Configuration of dummy controls"]
pub mod sgi_dummy_ctrl;
#[doc = "sgi_sfr_sw_mask (rw) register accessor: Sofware Assisted Masking register .\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_sfr_sw_mask::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_sfr_sw_mask::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_sfr_sw_mask`] module"]
#[doc(alias = "sgi_sfr_sw_mask")]
pub type SgiSfrSwMask = crate::Reg<sgi_sfr_sw_mask::SgiSfrSwMaskSpec>;
#[doc = "Sofware Assisted Masking register ."]
pub mod sgi_sfr_sw_mask;
#[doc = "sgi_sfrseed (rw) register accessor: SFRSEED register for SFRMASK feature.\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_sfrseed::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_sfrseed::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_sfrseed`] module"]
#[doc(alias = "sgi_sfrseed")]
pub type SgiSfrseed = crate::Reg<sgi_sfrseed::SgiSfrseedSpec>;
#[doc = "SFRSEED register for SFRMASK feature."]
pub mod sgi_sfrseed;
#[doc = "sgi_sha2_ctrl (rw) register accessor: SHA Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_sha2_ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_sha2_ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_sha2_ctrl`] module"]
#[doc(alias = "sgi_sha2_ctrl")]
pub type SgiSha2Ctrl = crate::Reg<sgi_sha2_ctrl::SgiSha2CtrlSpec>;
#[doc = "SHA Control Register"]
pub mod sgi_sha2_ctrl;
#[doc = "sgi_sha_fifo (rw) register accessor: SHA FIFO lower-bank low\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_sha_fifo::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_sha_fifo::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_sha_fifo`] module"]
#[doc(alias = "sgi_sha_fifo")]
pub type SgiShaFifo = crate::Reg<sgi_sha_fifo::SgiShaFifoSpec>;
#[doc = "SHA FIFO lower-bank low"]
pub mod sgi_sha_fifo;
#[doc = "sgi_config (r) register accessor: SHA Configuration Reg\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_config::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_config`] module"]
#[doc(alias = "sgi_config")]
pub type SgiConfig = crate::Reg<sgi_config::SgiConfigSpec>;
#[doc = "SHA Configuration Reg"]
pub mod sgi_config;
#[doc = "sgi_config2 (r) register accessor: SHA Configuration 2 Reg\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_config2::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_config2`] module"]
#[doc(alias = "sgi_config2")]
pub type SgiConfig2 = crate::Reg<sgi_config2::SgiConfig2Spec>;
#[doc = "SHA Configuration 2 Reg"]
pub mod sgi_config2;
#[doc = "sgi_auto_mode (rw) register accessor: SGI Auto Mode Control register\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_auto_mode::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_auto_mode::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_auto_mode`] module"]
#[doc(alias = "sgi_auto_mode")]
pub type SgiAutoMode = crate::Reg<sgi_auto_mode::SgiAutoModeSpec>;
#[doc = "SGI Auto Mode Control register"]
pub mod sgi_auto_mode;
#[doc = "sgi_auto_dma_ctrl (rw) register accessor: SGI Auto Mode Control register\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_auto_dma_ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_auto_dma_ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_auto_dma_ctrl`] module"]
#[doc(alias = "sgi_auto_dma_ctrl")]
pub type SgiAutoDmaCtrl = crate::Reg<sgi_auto_dma_ctrl::SgiAutoDmaCtrlSpec>;
#[doc = "SGI Auto Mode Control register"]
pub mod sgi_auto_dma_ctrl;
#[doc = "sgi_prng_sw_seed (rw) register accessor: SGI internal PRNG SW seeding register\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_prng_sw_seed::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_prng_sw_seed::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_prng_sw_seed`] module"]
#[doc(alias = "sgi_prng_sw_seed")]
pub type SgiPrngSwSeed = crate::Reg<sgi_prng_sw_seed::SgiPrngSwSeedSpec>;
#[doc = "SGI internal PRNG SW seeding register"]
pub mod sgi_prng_sw_seed;
#[doc = "sgi_key_ctrl (rw) register accessor: SGI Key Control SFR\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key_ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_key_ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key_ctrl`] module"]
#[doc(alias = "sgi_key_ctrl")]
pub type SgiKeyCtrl = crate::Reg<sgi_key_ctrl::SgiKeyCtrlSpec>;
#[doc = "SGI Key Control SFR"]
pub mod sgi_key_ctrl;
#[doc = "sgi_key_wrap (r) register accessor: Wrapped key read SFR\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key_wrap::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_key_wrap`] module"]
#[doc(alias = "sgi_key_wrap")]
pub type SgiKeyWrap = crate::Reg<sgi_key_wrap::SgiKeyWrapSpec>;
#[doc = "Wrapped key read SFR"]
pub mod sgi_key_wrap;
#[doc = "sgi_version (r) register accessor: SGI Version\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_version::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_version`] module"]
#[doc(alias = "sgi_version")]
pub type SgiVersion = crate::Reg<sgi_version::SgiVersionSpec>;
#[doc = "SGI Version"]
pub mod sgi_version;
#[doc = "sgi_access_err (rw) register accessor: Access Error\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_access_err::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_access_err::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_access_err`] module"]
#[doc(alias = "sgi_access_err")]
pub type SgiAccessErr = crate::Reg<sgi_access_err::SgiAccessErrSpec>;
#[doc = "Access Error"]
pub mod sgi_access_err;
#[doc = "sgi_access_err_clr (rw) register accessor: Clear Access Error\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_access_err_clr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_access_err_clr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_access_err_clr`] module"]
#[doc(alias = "sgi_access_err_clr")]
pub type SgiAccessErrClr = crate::Reg<sgi_access_err_clr::SgiAccessErrClrSpec>;
#[doc = "Clear Access Error"]
pub mod sgi_access_err_clr;
#[doc = "sgi_int_status (r) register accessor: Interrupt status\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_int_status::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_int_status`] module"]
#[doc(alias = "sgi_int_status")]
pub type SgiIntStatus = crate::Reg<sgi_int_status::SgiIntStatusSpec>;
#[doc = "Interrupt status"]
pub mod sgi_int_status;
#[doc = "sgi_int_enable (rw) register accessor: Interrupt enable\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_int_enable::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_int_enable::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_int_enable`] module"]
#[doc(alias = "sgi_int_enable")]
pub type SgiIntEnable = crate::Reg<sgi_int_enable::SgiIntEnableSpec>;
#[doc = "Interrupt enable"]
pub mod sgi_int_enable;
#[doc = "sgi_int_status_clr (rw) register accessor: Interrupt status clear\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_int_status_clr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_int_status_clr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_int_status_clr`] module"]
#[doc(alias = "sgi_int_status_clr")]
pub type SgiIntStatusClr = crate::Reg<sgi_int_status_clr::SgiIntStatusClrSpec>;
#[doc = "Interrupt status clear"]
pub mod sgi_int_status_clr;
#[doc = "sgi_int_status_set (rw) register accessor: Interrupt status set\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_int_status_set::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_int_status_set::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_int_status_set`] module"]
#[doc(alias = "sgi_int_status_set")]
pub type SgiIntStatusSet = crate::Reg<sgi_int_status_set::SgiIntStatusSetSpec>;
#[doc = "Interrupt status set"]
pub mod sgi_int_status_set;
#[doc = "sgi_module_id (r) register accessor: Module ID\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_module_id::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sgi_module_id`] module"]
#[doc(alias = "sgi_module_id")]
pub type SgiModuleId = crate::Reg<sgi_module_id::SgiModuleIdSpec>;
#[doc = "Module ID"]
pub mod sgi_module_id;

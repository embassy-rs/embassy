#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    mcr: Mcr,
    ctrl1: Ctrl1,
    timer: Timer,
    _reserved3: [u8; 0x04],
    rxmgmask: Rxmgmask,
    rx14mask: Rx14mask,
    rx15mask: Rx15mask,
    ecr: Ecr,
    esr1: Esr1,
    _reserved8: [u8; 0x04],
    imask1: Imask1,
    _reserved9: [u8; 0x04],
    iflag1: Iflag1,
    ctrl2: Ctrl2,
    esr2: Esr2,
    _reserved12: [u8; 0x08],
    crcr: Crcr,
    rxfgmask: Rxfgmask,
    rxfir: Rxfir,
    cbt: Cbt,
    _reserved16: [u8; 0x2c],
    _reserved_16_mb_: [u8; 0x04],
    _reserved_17_mb_: [u8; 0x04],
    _reserved_18_mb: [u8; 0x04],
    _reserved_19_mb: [u8; 0x04],
    _reserved_20_mb_: [u8; 0x04],
    _reserved_21_mb_: [u8; 0x04],
    _reserved_22_mb: [u8; 0x04],
    _reserved_23_mb: [u8; 0x04],
    _reserved_24_mb_: [u8; 0x04],
    _reserved_25_mb_: [u8; 0x04],
    _reserved_26_mb: [u8; 0x04],
    _reserved_27_mb: [u8; 0x04],
    _reserved_28_mb_: [u8; 0x04],
    _reserved_29_mb_: [u8; 0x04],
    _reserved_30_mb: [u8; 0x04],
    _reserved_31_mb: [u8; 0x04],
    _reserved_32_mb_: [u8; 0x04],
    _reserved_33_mb_: [u8; 0x04],
    _reserved_34_mb: [u8; 0x04],
    _reserved_35_mb: [u8; 0x04],
    _reserved_36_mb_: [u8; 0x04],
    _reserved_37_mb_: [u8; 0x04],
    _reserved_38_mb: [u8; 0x04],
    _reserved_39_mb: [u8; 0x04],
    _reserved_40_mb_: [u8; 0x04],
    _reserved_41_mb_: [u8; 0x04],
    _reserved_42_mb: [u8; 0x04],
    _reserved_43_mb: [u8; 0x04],
    _reserved_44_mb_: [u8; 0x04],
    _reserved_45_mb_: [u8; 0x04],
    _reserved_46_mb: [u8; 0x04],
    _reserved_47_mb: [u8; 0x04],
    _reserved_48_mb_: [u8; 0x04],
    _reserved_49_mb_: [u8; 0x04],
    _reserved_50_mb: [u8; 0x04],
    _reserved_51_mb: [u8; 0x04],
    _reserved_52_mb_: [u8; 0x04],
    _reserved_53_mb_: [u8; 0x04],
    _reserved_54_mb: [u8; 0x04],
    _reserved_55_mb: [u8; 0x04],
    _reserved_56_mb_: [u8; 0x04],
    _reserved_57_mb_: [u8; 0x04],
    _reserved_58_mb: [u8; 0x04],
    _reserved_59_mb: [u8; 0x04],
    _reserved_60_mb_: [u8; 0x04],
    _reserved_61_mb_: [u8; 0x04],
    _reserved_62_mb: [u8; 0x04],
    _reserved_63_mb: [u8; 0x04],
    _reserved_64_mb_: [u8; 0x04],
    _reserved_65_mb_: [u8; 0x04],
    _reserved_66_mb: [u8; 0x04],
    _reserved_67_mb: [u8; 0x04],
    _reserved_68_mb_: [u8; 0x04],
    _reserved_69_mb_: [u8; 0x04],
    _reserved_70_mb: [u8; 0x04],
    _reserved_71_mb: [u8; 0x04],
    _reserved_72_mb_: [u8; 0x04],
    _reserved_73_mb_: [u8; 0x04],
    _reserved_74_mb: [u8; 0x04],
    _reserved_75_mb: [u8; 0x04],
    _reserved_76_mb_: [u8; 0x04],
    _reserved_77_mb_: [u8; 0x04],
    _reserved_78_mb: [u8; 0x04],
    _reserved_79_mb: [u8; 0x04],
    _reserved_80_mb_: [u8; 0x04],
    _reserved_81_mb_: [u8; 0x04],
    _reserved_82_mb: [u8; 0x04],
    _reserved_83_mb: [u8; 0x04],
    _reserved_84_mb_: [u8; 0x04],
    _reserved_85_mb_: [u8; 0x04],
    _reserved_86_mb: [u8; 0x04],
    _reserved_87_mb: [u8; 0x04],
    _reserved_88_mb_: [u8; 0x04],
    _reserved_89_mb_: [u8; 0x04],
    _reserved_90_mb: [u8; 0x04],
    _reserved_91_mb: [u8; 0x04],
    _reserved_92_mb_: [u8; 0x04],
    _reserved_93_mb_: [u8; 0x04],
    _reserved_94_mb: [u8; 0x04],
    _reserved_95_mb: [u8; 0x04],
    _reserved_96_mb_: [u8; 0x04],
    _reserved_97_mb_: [u8; 0x04],
    _reserved_98_mb: [u8; 0x04],
    _reserved_99_mb: [u8; 0x04],
    _reserved_100_mb_: [u8; 0x04],
    _reserved_101_mb_: [u8; 0x04],
    _reserved_102_mb: [u8; 0x04],
    _reserved_103_mb: [u8; 0x04],
    _reserved_104_mb_: [u8; 0x04],
    _reserved_105_mb_: [u8; 0x04],
    _reserved_106_mb: [u8; 0x04],
    _reserved_107_mb: [u8; 0x04],
    _reserved_108_mb_: [u8; 0x04],
    _reserved_109_mb_: [u8; 0x04],
    _reserved_110_mb: [u8; 0x04],
    _reserved_111_mb: [u8; 0x04],
    _reserved_112_mb_: [u8; 0x04],
    _reserved_113_mb_: [u8; 0x04],
    _reserved_114_mb: [u8; 0x04],
    _reserved_115_mb: [u8; 0x04],
    _reserved_116_mb_: [u8; 0x04],
    _reserved_117_mb_: [u8; 0x04],
    _reserved_118_mb: [u8; 0x04],
    _reserved_119_mb: [u8; 0x04],
    _reserved_120_mb_: [u8; 0x04],
    _reserved_121_mb_: [u8; 0x04],
    _reserved_122_mb: [u8; 0x04],
    _reserved_123_mb: [u8; 0x04],
    _reserved_124_mb_: [u8; 0x04],
    _reserved_125_mb_: [u8; 0x04],
    _reserved_126_mb: [u8; 0x04],
    _reserved_127_mb: [u8; 0x04],
    _reserved_128_mb_: [u8; 0x04],
    _reserved_129_mb_: [u8; 0x04],
    _reserved_130_mb: [u8; 0x04],
    _reserved_131_mb: [u8; 0x04],
    _reserved_132_mb_: [u8; 0x04],
    _reserved_133_mb_: [u8; 0x04],
    _reserved_134_mb: [u8; 0x04],
    _reserved_135_mb: [u8; 0x04],
    _reserved_136_mb_: [u8; 0x04],
    _reserved_137_mb_: [u8; 0x04],
    _reserved_138_mb: [u8; 0x04],
    _reserved_139_mb: [u8; 0x04],
    _reserved_140_mb_: [u8; 0x04],
    _reserved_141_mb_: [u8; 0x04],
    _reserved_142_mb: [u8; 0x04],
    _reserved_143_mb: [u8; 0x04],
    _reserved144: [u8; 0x0600],
    rximr: [Rximr; 32],
    _reserved145: [u8; 0x0200],
    ctrl1_pn: Ctrl1Pn,
    ctrl2_pn: Ctrl2Pn,
    wu_mtc: WuMtc,
    flt_id1: FltId1,
    flt_dlc: FltDlc,
    pl1_lo: Pl1Lo,
    pl1_hi: Pl1Hi,
    flt_id2_idmask: FltId2Idmask,
    pl2_plmask_lo: Pl2PlmaskLo,
    pl2_plmask_hi: Pl2PlmaskHi,
    _reserved155: [u8; 0x18],
    wmb: [Wmb; 4],
    _reserved156: [u8; 0x70],
    eprs: Eprs,
    encbt: Encbt,
    edcbt: Edcbt,
    etdc: Etdc,
    fdctrl: Fdctrl,
    fdcbt: Fdcbt,
    fdcrc: Fdcrc,
    erfcr: Erfcr,
    erfier: Erfier,
    erfsr: Erfsr,
    _reserved166: [u8; 0x23e8],
    erffel: [Erffel; 32],
}
impl RegisterBlock {
    #[doc = "0x00 - Module Configuration"]
    #[inline(always)]
    pub const fn mcr(&self) -> &Mcr {
        &self.mcr
    }
    #[doc = "0x04 - Control 1"]
    #[inline(always)]
    pub const fn ctrl1(&self) -> &Ctrl1 {
        &self.ctrl1
    }
    #[doc = "0x08 - Free-Running Timer"]
    #[inline(always)]
    pub const fn timer(&self) -> &Timer {
        &self.timer
    }
    #[doc = "0x10 - RX Message Buffers Global Mask"]
    #[inline(always)]
    pub const fn rxmgmask(&self) -> &Rxmgmask {
        &self.rxmgmask
    }
    #[doc = "0x14 - Receive 14 Mask"]
    #[inline(always)]
    pub const fn rx14mask(&self) -> &Rx14mask {
        &self.rx14mask
    }
    #[doc = "0x18 - Receive 15 Mask"]
    #[inline(always)]
    pub const fn rx15mask(&self) -> &Rx15mask {
        &self.rx15mask
    }
    #[doc = "0x1c - Error Counter"]
    #[inline(always)]
    pub const fn ecr(&self) -> &Ecr {
        &self.ecr
    }
    #[doc = "0x20 - Error and Status 1"]
    #[inline(always)]
    pub const fn esr1(&self) -> &Esr1 {
        &self.esr1
    }
    #[doc = "0x28 - Interrupt Masks 1"]
    #[inline(always)]
    pub const fn imask1(&self) -> &Imask1 {
        &self.imask1
    }
    #[doc = "0x30 - Interrupt Flags 1"]
    #[inline(always)]
    pub const fn iflag1(&self) -> &Iflag1 {
        &self.iflag1
    }
    #[doc = "0x34 - Control 2"]
    #[inline(always)]
    pub const fn ctrl2(&self) -> &Ctrl2 {
        &self.ctrl2
    }
    #[doc = "0x38 - Error and Status 2"]
    #[inline(always)]
    pub const fn esr2(&self) -> &Esr2 {
        &self.esr2
    }
    #[doc = "0x44 - Cyclic Redundancy Check"]
    #[inline(always)]
    pub const fn crcr(&self) -> &Crcr {
        &self.crcr
    }
    #[doc = "0x48 - Legacy RX FIFO Global Mask"]
    #[inline(always)]
    pub const fn rxfgmask(&self) -> &Rxfgmask {
        &self.rxfgmask
    }
    #[doc = "0x4c - Legacy RX FIFO Information"]
    #[inline(always)]
    pub const fn rxfir(&self) -> &Rxfir {
        &self.rxfir
    }
    #[doc = "0x50 - CAN Bit Timing"]
    #[inline(always)]
    pub const fn cbt(&self) -> &Cbt {
        &self.cbt
    }
    #[doc = "0x80 - Message Buffer 0 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb0_8b_cs(&self) -> &Mb8bGroupMb0_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(128).cast() }
    }
    #[doc = "0x80 - Message Buffer 0 CS Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb0_64b_cs(&self) -> &Mb64bGroupMb0_64bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(128).cast() }
    }
    #[doc = "0x80 - Message Buffer 0 CS Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb0_32b_cs(&self) -> &Mb32bGroupMb0_32bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(128).cast() }
    }
    #[doc = "0x80 - Message Buffer 0 CS Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb0_16b_cs(&self) -> &Mb16bGroupMb0_16bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(128).cast() }
    }
    #[doc = "0x80 - Message Buffer 0 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs0(&self) -> &MbGroupCs0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(128).cast() }
    }
    #[doc = "0x84 - Message Buffer 0 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb0_8b_id(&self) -> &Mb8bGroupMb0_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(132).cast() }
    }
    #[doc = "0x84 - Message Buffer 0 ID Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb0_64b_id(&self) -> &Mb64bGroupMb0_64bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(132).cast() }
    }
    #[doc = "0x84 - Message Buffer 0 ID Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb0_32b_id(&self) -> &Mb32bGroupMb0_32bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(132).cast() }
    }
    #[doc = "0x84 - Message Buffer 0 ID Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb0_16b_id(&self) -> &Mb16bGroupMb0_16bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(132).cast() }
    }
    #[doc = "0x84 - Message Buffer 0 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id0(&self) -> &MbGroupId0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(132).cast() }
    }
    #[doc = "0x88 - Message Buffer 0 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word00(&self) -> &MbGroupWord00 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(136).cast() }
    }
    #[doc = "0x88 - Message Buffer 0 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb0_8b_word0(&self) -> &Mb8bGroupMb0_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(136).cast() }
    }
    #[doc = "0x88 - Message Buffer 0 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb0_64b_word0(&self) -> &Mb64bGroupMb0_64bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(136).cast() }
    }
    #[doc = "0x88 - Message Buffer 0 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb0_32b_word0(&self) -> &Mb32bGroupMb0_32bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(136).cast() }
    }
    #[doc = "0x88 - Message Buffer 0 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb0_16b_word0(&self) -> &Mb16bGroupMb0_16bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(136).cast() }
    }
    #[doc = "0x8c - Message Buffer 0 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word10(&self) -> &MbGroupWord10 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(140).cast() }
    }
    #[doc = "0x8c - Message Buffer 0 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb0_8b_word1(&self) -> &Mb8bGroupMb0_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(140).cast() }
    }
    #[doc = "0x8c - Message Buffer 0 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb0_64b_word1(&self) -> &Mb64bGroupMb0_64bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(140).cast() }
    }
    #[doc = "0x8c - Message Buffer 0 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb0_32b_word1(&self) -> &Mb32bGroupMb0_32bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(140).cast() }
    }
    #[doc = "0x8c - Message Buffer 0 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb0_16b_word1(&self) -> &Mb16bGroupMb0_16bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(140).cast() }
    }
    #[doc = "0x90 - Message Buffer 1 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb1_8b_cs(&self) -> &Mb8bGroupMb1_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(144).cast() }
    }
    #[doc = "0x90 - Message Buffer 0 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb0_64b_word2(&self) -> &Mb64bGroupMb0_64bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(144).cast() }
    }
    #[doc = "0x90 - Message Buffer 0 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb0_32b_word2(&self) -> &Mb32bGroupMb0_32bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(144).cast() }
    }
    #[doc = "0x90 - Message Buffer 0 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb0_16b_word2(&self) -> &Mb16bGroupMb0_16bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(144).cast() }
    }
    #[doc = "0x90 - Message Buffer 1 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs1(&self) -> &MbGroupCs1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(144).cast() }
    }
    #[doc = "0x94 - Message Buffer 1 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb1_8b_id(&self) -> &Mb8bGroupMb1_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(148).cast() }
    }
    #[doc = "0x94 - Message Buffer 0 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb0_64b_word3(&self) -> &Mb64bGroupMb0_64bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(148).cast() }
    }
    #[doc = "0x94 - Message Buffer 0 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb0_32b_word3(&self) -> &Mb32bGroupMb0_32bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(148).cast() }
    }
    #[doc = "0x94 - Message Buffer 0 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb0_16b_word3(&self) -> &Mb16bGroupMb0_16bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(148).cast() }
    }
    #[doc = "0x94 - Message Buffer 1 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id1(&self) -> &MbGroupId1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(148).cast() }
    }
    #[doc = "0x98 - Message Buffer 1 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word01(&self) -> &MbGroupWord01 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(152).cast() }
    }
    #[doc = "0x98 - Message Buffer 1 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb1_8b_word0(&self) -> &Mb8bGroupMb1_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(152).cast() }
    }
    #[doc = "0x98 - Message Buffer 1 CS Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb1_16b_cs(&self) -> &Mb16bGroupMb1_16bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(152).cast() }
    }
    #[doc = "0x98 - Message Buffer 0 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb0_64b_word4(&self) -> &Mb64bGroupMb0_64bWord4 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(152).cast() }
    }
    #[doc = "0x98 - Message Buffer 0 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb0_32b_word4(&self) -> &Mb32bGroupMb0_32bWord4 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(152).cast() }
    }
    #[doc = "0x9c - Message Buffer 1 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word11(&self) -> &MbGroupWord11 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(156).cast() }
    }
    #[doc = "0x9c - Message Buffer 1 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb1_8b_word1(&self) -> &Mb8bGroupMb1_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(156).cast() }
    }
    #[doc = "0x9c - Message Buffer 1 ID Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb1_16b_id(&self) -> &Mb16bGroupMb1_16bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(156).cast() }
    }
    #[doc = "0x9c - Message Buffer 0 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb0_64b_word5(&self) -> &Mb64bGroupMb0_64bWord5 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(156).cast() }
    }
    #[doc = "0x9c - Message Buffer 0 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb0_32b_word5(&self) -> &Mb32bGroupMb0_32bWord5 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(156).cast() }
    }
    #[doc = "0xa0 - Message Buffer 2 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb2_8b_cs(&self) -> &Mb8bGroupMb2_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(160).cast() }
    }
    #[doc = "0xa0 - Message Buffer 1 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb1_16b_word0(&self) -> &Mb16bGroupMb1_16bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(160).cast() }
    }
    #[doc = "0xa0 - Message Buffer 0 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb0_64b_word6(&self) -> &Mb64bGroupMb0_64bWord6 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(160).cast() }
    }
    #[doc = "0xa0 - Message Buffer 0 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb0_32b_word6(&self) -> &Mb32bGroupMb0_32bWord6 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(160).cast() }
    }
    #[doc = "0xa0 - Message Buffer 2 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs2(&self) -> &MbGroupCs2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(160).cast() }
    }
    #[doc = "0xa4 - Message Buffer 2 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb2_8b_id(&self) -> &Mb8bGroupMb2_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(164).cast() }
    }
    #[doc = "0xa4 - Message Buffer 1 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb1_16b_word1(&self) -> &Mb16bGroupMb1_16bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(164).cast() }
    }
    #[doc = "0xa4 - Message Buffer 0 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb0_64b_word7(&self) -> &Mb64bGroupMb0_64bWord7 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(164).cast() }
    }
    #[doc = "0xa4 - Message Buffer 0 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb0_32b_word7(&self) -> &Mb32bGroupMb0_32bWord7 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(164).cast() }
    }
    #[doc = "0xa4 - Message Buffer 2 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id2(&self) -> &MbGroupId2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(164).cast() }
    }
    #[doc = "0xa8 - Message Buffer 2 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word02(&self) -> &MbGroupWord02 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(168).cast() }
    }
    #[doc = "0xa8 - Message Buffer 2 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb2_8b_word0(&self) -> &Mb8bGroupMb2_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(168).cast() }
    }
    #[doc = "0xa8 - Message Buffer 1 CS Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb1_32b_cs(&self) -> &Mb32bGroupMb1_32bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(168).cast() }
    }
    #[doc = "0xa8 - Message Buffer 1 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb1_16b_word2(&self) -> &Mb16bGroupMb1_16bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(168).cast() }
    }
    #[doc = "0xa8 - Message Buffer 0 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb0_64b_word8(&self) -> &Mb64bGroupMb0_64bWord8 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(168).cast() }
    }
    #[doc = "0xac - Message Buffer 2 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word12(&self) -> &MbGroupWord12 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(172).cast() }
    }
    #[doc = "0xac - Message Buffer 2 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb2_8b_word1(&self) -> &Mb8bGroupMb2_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(172).cast() }
    }
    #[doc = "0xac - Message Buffer 1 ID Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb1_32b_id(&self) -> &Mb32bGroupMb1_32bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(172).cast() }
    }
    #[doc = "0xac - Message Buffer 1 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb1_16b_word3(&self) -> &Mb16bGroupMb1_16bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(172).cast() }
    }
    #[doc = "0xac - Message Buffer 0 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb0_64b_word9(&self) -> &Mb64bGroupMb0_64bWord9 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(172).cast() }
    }
    #[doc = "0xb0 - Message Buffer 3 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb3_8b_cs(&self) -> &Mb8bGroupMb3_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(176).cast() }
    }
    #[doc = "0xb0 - Message Buffer 2 CS Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb2_16b_cs(&self) -> &Mb16bGroupMb2_16bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(176).cast() }
    }
    #[doc = "0xb0 - Message Buffer 1 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb1_32b_word0(&self) -> &Mb32bGroupMb1_32bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(176).cast() }
    }
    #[doc = "0xb0 - Message Buffer 0 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb0_64b_word10(&self) -> &Mb64bGroupMb0_64bWord10 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(176).cast() }
    }
    #[doc = "0xb0 - Message Buffer 3 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs3(&self) -> &MbGroupCs3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(176).cast() }
    }
    #[doc = "0xb4 - Message Buffer 3 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb3_8b_id(&self) -> &Mb8bGroupMb3_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(180).cast() }
    }
    #[doc = "0xb4 - Message Buffer 2 ID Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb2_16b_id(&self) -> &Mb16bGroupMb2_16bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(180).cast() }
    }
    #[doc = "0xb4 - Message Buffer 1 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb1_32b_word1(&self) -> &Mb32bGroupMb1_32bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(180).cast() }
    }
    #[doc = "0xb4 - Message Buffer 0 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb0_64b_word11(&self) -> &Mb64bGroupMb0_64bWord11 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(180).cast() }
    }
    #[doc = "0xb4 - Message Buffer 3 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id3(&self) -> &MbGroupId3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(180).cast() }
    }
    #[doc = "0xb8 - Message Buffer 3 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word03(&self) -> &MbGroupWord03 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(184).cast() }
    }
    #[doc = "0xb8 - Message Buffer 3 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb3_8b_word0(&self) -> &Mb8bGroupMb3_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(184).cast() }
    }
    #[doc = "0xb8 - Message Buffer 2 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb2_16b_word0(&self) -> &Mb16bGroupMb2_16bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(184).cast() }
    }
    #[doc = "0xb8 - Message Buffer 1 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb1_32b_word2(&self) -> &Mb32bGroupMb1_32bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(184).cast() }
    }
    #[doc = "0xb8 - Message Buffer 0 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb0_64b_word12(&self) -> &Mb64bGroupMb0_64bWord12 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(184).cast() }
    }
    #[doc = "0xbc - Message Buffer 3 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word13(&self) -> &MbGroupWord13 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(188).cast() }
    }
    #[doc = "0xbc - Message Buffer 3 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb3_8b_word1(&self) -> &Mb8bGroupMb3_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(188).cast() }
    }
    #[doc = "0xbc - Message Buffer 2 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb2_16b_word1(&self) -> &Mb16bGroupMb2_16bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(188).cast() }
    }
    #[doc = "0xbc - Message Buffer 1 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb1_32b_word3(&self) -> &Mb32bGroupMb1_32bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(188).cast() }
    }
    #[doc = "0xbc - Message Buffer 0 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb0_64b_word13(&self) -> &Mb64bGroupMb0_64bWord13 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(188).cast() }
    }
    #[doc = "0xc0 - Message Buffer 4 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb4_8b_cs(&self) -> &Mb8bGroupMb4_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(192).cast() }
    }
    #[doc = "0xc0 - Message Buffer 2 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb2_16b_word2(&self) -> &Mb16bGroupMb2_16bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(192).cast() }
    }
    #[doc = "0xc0 - Message Buffer 1 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb1_32b_word4(&self) -> &Mb32bGroupMb1_32bWord4 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(192).cast() }
    }
    #[doc = "0xc0 - Message Buffer 0 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb0_64b_word14(&self) -> &Mb64bGroupMb0_64bWord14 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(192).cast() }
    }
    #[doc = "0xc0 - Message Buffer 4 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs4(&self) -> &MbGroupCs4 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(192).cast() }
    }
    #[doc = "0xc4 - Message Buffer 4 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb4_8b_id(&self) -> &Mb8bGroupMb4_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(196).cast() }
    }
    #[doc = "0xc4 - Message Buffer 2 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb2_16b_word3(&self) -> &Mb16bGroupMb2_16bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(196).cast() }
    }
    #[doc = "0xc4 - Message Buffer 1 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb1_32b_word5(&self) -> &Mb32bGroupMb1_32bWord5 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(196).cast() }
    }
    #[doc = "0xc4 - Message Buffer 0 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb0_64b_word15(&self) -> &Mb64bGroupMb0_64bWord15 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(196).cast() }
    }
    #[doc = "0xc4 - Message Buffer 4 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id4(&self) -> &MbGroupId4 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(196).cast() }
    }
    #[doc = "0xc8 - Message Buffer 4 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word04(&self) -> &MbGroupWord04 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(200).cast() }
    }
    #[doc = "0xc8 - Message Buffer 4 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb4_8b_word0(&self) -> &Mb8bGroupMb4_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(200).cast() }
    }
    #[doc = "0xc8 - Message Buffer 3 CS Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb3_16b_cs(&self) -> &Mb16bGroupMb3_16bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(200).cast() }
    }
    #[doc = "0xc8 - Message Buffer 1 CS Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb1_64b_cs(&self) -> &Mb64bGroupMb1_64bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(200).cast() }
    }
    #[doc = "0xc8 - Message Buffer 1 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb1_32b_word6(&self) -> &Mb32bGroupMb1_32bWord6 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(200).cast() }
    }
    #[doc = "0xcc - Message Buffer 4 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word14(&self) -> &MbGroupWord14 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(204).cast() }
    }
    #[doc = "0xcc - Message Buffer 4 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb4_8b_word1(&self) -> &Mb8bGroupMb4_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(204).cast() }
    }
    #[doc = "0xcc - Message Buffer 3 ID Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb3_16b_id(&self) -> &Mb16bGroupMb3_16bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(204).cast() }
    }
    #[doc = "0xcc - Message Buffer 1 ID Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb1_64b_id(&self) -> &Mb64bGroupMb1_64bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(204).cast() }
    }
    #[doc = "0xcc - Message Buffer 1 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb1_32b_word7(&self) -> &Mb32bGroupMb1_32bWord7 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(204).cast() }
    }
    #[doc = "0xd0 - Message Buffer 5 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb5_8b_cs(&self) -> &Mb8bGroupMb5_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(208).cast() }
    }
    #[doc = "0xd0 - Message Buffer 3 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb3_16b_word0(&self) -> &Mb16bGroupMb3_16bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(208).cast() }
    }
    #[doc = "0xd0 - Message Buffer 2 CS Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb2_32b_cs(&self) -> &Mb32bGroupMb2_32bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(208).cast() }
    }
    #[doc = "0xd0 - Message Buffer 1 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb1_64b_word0(&self) -> &Mb64bGroupMb1_64bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(208).cast() }
    }
    #[doc = "0xd0 - Message Buffer 5 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs5(&self) -> &MbGroupCs5 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(208).cast() }
    }
    #[doc = "0xd4 - Message Buffer 5 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb5_8b_id(&self) -> &Mb8bGroupMb5_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(212).cast() }
    }
    #[doc = "0xd4 - Message Buffer 3 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb3_16b_word1(&self) -> &Mb16bGroupMb3_16bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(212).cast() }
    }
    #[doc = "0xd4 - Message Buffer 2 ID Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb2_32b_id(&self) -> &Mb32bGroupMb2_32bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(212).cast() }
    }
    #[doc = "0xd4 - Message Buffer 1 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb1_64b_word1(&self) -> &Mb64bGroupMb1_64bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(212).cast() }
    }
    #[doc = "0xd4 - Message Buffer 5 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id5(&self) -> &MbGroupId5 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(212).cast() }
    }
    #[doc = "0xd8 - Message Buffer 5 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word05(&self) -> &MbGroupWord05 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(216).cast() }
    }
    #[doc = "0xd8 - Message Buffer 5 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb5_8b_word0(&self) -> &Mb8bGroupMb5_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(216).cast() }
    }
    #[doc = "0xd8 - Message Buffer 3 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb3_16b_word2(&self) -> &Mb16bGroupMb3_16bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(216).cast() }
    }
    #[doc = "0xd8 - Message Buffer 2 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb2_32b_word0(&self) -> &Mb32bGroupMb2_32bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(216).cast() }
    }
    #[doc = "0xd8 - Message Buffer 1 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb1_64b_word2(&self) -> &Mb64bGroupMb1_64bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(216).cast() }
    }
    #[doc = "0xdc - Message Buffer 5 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word15(&self) -> &MbGroupWord15 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(220).cast() }
    }
    #[doc = "0xdc - Message Buffer 5 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb5_8b_word1(&self) -> &Mb8bGroupMb5_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(220).cast() }
    }
    #[doc = "0xdc - Message Buffer 3 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb3_16b_word3(&self) -> &Mb16bGroupMb3_16bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(220).cast() }
    }
    #[doc = "0xdc - Message Buffer 2 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb2_32b_word1(&self) -> &Mb32bGroupMb2_32bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(220).cast() }
    }
    #[doc = "0xdc - Message Buffer 1 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb1_64b_word3(&self) -> &Mb64bGroupMb1_64bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(220).cast() }
    }
    #[doc = "0xe0 - Message Buffer 6 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb6_8b_cs(&self) -> &Mb8bGroupMb6_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(224).cast() }
    }
    #[doc = "0xe0 - Message Buffer 4 CS Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb4_16b_cs(&self) -> &Mb16bGroupMb4_16bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(224).cast() }
    }
    #[doc = "0xe0 - Message Buffer 2 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb2_32b_word2(&self) -> &Mb32bGroupMb2_32bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(224).cast() }
    }
    #[doc = "0xe0 - Message Buffer 1 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb1_64b_word4(&self) -> &Mb64bGroupMb1_64bWord4 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(224).cast() }
    }
    #[doc = "0xe0 - Message Buffer 6 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs6(&self) -> &MbGroupCs6 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(224).cast() }
    }
    #[doc = "0xe4 - Message Buffer 6 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb6_8b_id(&self) -> &Mb8bGroupMb6_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(228).cast() }
    }
    #[doc = "0xe4 - Message Buffer 4 ID Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb4_16b_id(&self) -> &Mb16bGroupMb4_16bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(228).cast() }
    }
    #[doc = "0xe4 - Message Buffer 2 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb2_32b_word3(&self) -> &Mb32bGroupMb2_32bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(228).cast() }
    }
    #[doc = "0xe4 - Message Buffer 1 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb1_64b_word5(&self) -> &Mb64bGroupMb1_64bWord5 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(228).cast() }
    }
    #[doc = "0xe4 - Message Buffer 6 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id6(&self) -> &MbGroupId6 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(228).cast() }
    }
    #[doc = "0xe8 - Message Buffer 6 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word06(&self) -> &MbGroupWord06 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(232).cast() }
    }
    #[doc = "0xe8 - Message Buffer 6 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb6_8b_word0(&self) -> &Mb8bGroupMb6_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(232).cast() }
    }
    #[doc = "0xe8 - Message Buffer 4 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb4_16b_word0(&self) -> &Mb16bGroupMb4_16bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(232).cast() }
    }
    #[doc = "0xe8 - Message Buffer 2 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb2_32b_word4(&self) -> &Mb32bGroupMb2_32bWord4 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(232).cast() }
    }
    #[doc = "0xe8 - Message Buffer 1 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb1_64b_word6(&self) -> &Mb64bGroupMb1_64bWord6 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(232).cast() }
    }
    #[doc = "0xec - Message Buffer 6 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word16(&self) -> &MbGroupWord16 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(236).cast() }
    }
    #[doc = "0xec - Message Buffer 6 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb6_8b_word1(&self) -> &Mb8bGroupMb6_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(236).cast() }
    }
    #[doc = "0xec - Message Buffer 4 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb4_16b_word1(&self) -> &Mb16bGroupMb4_16bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(236).cast() }
    }
    #[doc = "0xec - Message Buffer 2 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb2_32b_word5(&self) -> &Mb32bGroupMb2_32bWord5 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(236).cast() }
    }
    #[doc = "0xec - Message Buffer 1 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb1_64b_word7(&self) -> &Mb64bGroupMb1_64bWord7 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(236).cast() }
    }
    #[doc = "0xf0 - Message Buffer 7 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb7_8b_cs(&self) -> &Mb8bGroupMb7_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(240).cast() }
    }
    #[doc = "0xf0 - Message Buffer 4 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb4_16b_word2(&self) -> &Mb16bGroupMb4_16bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(240).cast() }
    }
    #[doc = "0xf0 - Message Buffer 2 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb2_32b_word6(&self) -> &Mb32bGroupMb2_32bWord6 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(240).cast() }
    }
    #[doc = "0xf0 - Message Buffer 1 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb1_64b_word8(&self) -> &Mb64bGroupMb1_64bWord8 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(240).cast() }
    }
    #[doc = "0xf0 - Message Buffer 7 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs7(&self) -> &MbGroupCs7 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(240).cast() }
    }
    #[doc = "0xf4 - Message Buffer 7 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb7_8b_id(&self) -> &Mb8bGroupMb7_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(244).cast() }
    }
    #[doc = "0xf4 - Message Buffer 4 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb4_16b_word3(&self) -> &Mb16bGroupMb4_16bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(244).cast() }
    }
    #[doc = "0xf4 - Message Buffer 2 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb2_32b_word7(&self) -> &Mb32bGroupMb2_32bWord7 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(244).cast() }
    }
    #[doc = "0xf4 - Message Buffer 1 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb1_64b_word9(&self) -> &Mb64bGroupMb1_64bWord9 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(244).cast() }
    }
    #[doc = "0xf4 - Message Buffer 7 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id7(&self) -> &MbGroupId7 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(244).cast() }
    }
    #[doc = "0xf8 - Message Buffer 7 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word07(&self) -> &MbGroupWord07 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(248).cast() }
    }
    #[doc = "0xf8 - Message Buffer 7 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb7_8b_word0(&self) -> &Mb8bGroupMb7_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(248).cast() }
    }
    #[doc = "0xf8 - Message Buffer 5 CS Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb5_16b_cs(&self) -> &Mb16bGroupMb5_16bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(248).cast() }
    }
    #[doc = "0xf8 - Message Buffer 3 CS Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb3_32b_cs(&self) -> &Mb32bGroupMb3_32bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(248).cast() }
    }
    #[doc = "0xf8 - Message Buffer 1 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb1_64b_word10(&self) -> &Mb64bGroupMb1_64bWord10 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(248).cast() }
    }
    #[doc = "0xfc - Message Buffer 7 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word17(&self) -> &MbGroupWord17 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(252).cast() }
    }
    #[doc = "0xfc - Message Buffer 7 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb7_8b_word1(&self) -> &Mb8bGroupMb7_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(252).cast() }
    }
    #[doc = "0xfc - Message Buffer 5 ID Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb5_16b_id(&self) -> &Mb16bGroupMb5_16bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(252).cast() }
    }
    #[doc = "0xfc - Message Buffer 3 ID Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb3_32b_id(&self) -> &Mb32bGroupMb3_32bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(252).cast() }
    }
    #[doc = "0xfc - Message Buffer 1 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb1_64b_word11(&self) -> &Mb64bGroupMb1_64bWord11 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(252).cast() }
    }
    #[doc = "0x100 - Message Buffer 8 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb8_8b_cs(&self) -> &Mb8bGroupMb8_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(256).cast() }
    }
    #[doc = "0x100 - Message Buffer 5 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb5_16b_word0(&self) -> &Mb16bGroupMb5_16bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(256).cast() }
    }
    #[doc = "0x100 - Message Buffer 3 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb3_32b_word0(&self) -> &Mb32bGroupMb3_32bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(256).cast() }
    }
    #[doc = "0x100 - Message Buffer 1 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb1_64b_word12(&self) -> &Mb64bGroupMb1_64bWord12 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(256).cast() }
    }
    #[doc = "0x100 - Message Buffer 8 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs8(&self) -> &MbGroupCs8 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(256).cast() }
    }
    #[doc = "0x104 - Message Buffer 8 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb8_8b_id(&self) -> &Mb8bGroupMb8_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(260).cast() }
    }
    #[doc = "0x104 - Message Buffer 5 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb5_16b_word1(&self) -> &Mb16bGroupMb5_16bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(260).cast() }
    }
    #[doc = "0x104 - Message Buffer 3 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb3_32b_word1(&self) -> &Mb32bGroupMb3_32bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(260).cast() }
    }
    #[doc = "0x104 - Message Buffer 1 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb1_64b_word13(&self) -> &Mb64bGroupMb1_64bWord13 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(260).cast() }
    }
    #[doc = "0x104 - Message Buffer 8 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id8(&self) -> &MbGroupId8 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(260).cast() }
    }
    #[doc = "0x108 - Message Buffer 8 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word08(&self) -> &MbGroupWord08 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(264).cast() }
    }
    #[doc = "0x108 - Message Buffer 8 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb8_8b_word0(&self) -> &Mb8bGroupMb8_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(264).cast() }
    }
    #[doc = "0x108 - Message Buffer 5 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb5_16b_word2(&self) -> &Mb16bGroupMb5_16bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(264).cast() }
    }
    #[doc = "0x108 - Message Buffer 3 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb3_32b_word2(&self) -> &Mb32bGroupMb3_32bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(264).cast() }
    }
    #[doc = "0x108 - Message Buffer 1 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb1_64b_word14(&self) -> &Mb64bGroupMb1_64bWord14 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(264).cast() }
    }
    #[doc = "0x10c - Message Buffer 8 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word18(&self) -> &MbGroupWord18 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(268).cast() }
    }
    #[doc = "0x10c - Message Buffer 8 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb8_8b_word1(&self) -> &Mb8bGroupMb8_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(268).cast() }
    }
    #[doc = "0x10c - Message Buffer 5 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb5_16b_word3(&self) -> &Mb16bGroupMb5_16bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(268).cast() }
    }
    #[doc = "0x10c - Message Buffer 3 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb3_32b_word3(&self) -> &Mb32bGroupMb3_32bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(268).cast() }
    }
    #[doc = "0x10c - Message Buffer 1 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb1_64b_word15(&self) -> &Mb64bGroupMb1_64bWord15 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(268).cast() }
    }
    #[doc = "0x110 - Message Buffer 9 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb9_8b_cs(&self) -> &Mb8bGroupMb9_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(272).cast() }
    }
    #[doc = "0x110 - Message Buffer 6 CS Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb6_16b_cs(&self) -> &Mb16bGroupMb6_16bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(272).cast() }
    }
    #[doc = "0x110 - Message Buffer 3 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb3_32b_word4(&self) -> &Mb32bGroupMb3_32bWord4 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(272).cast() }
    }
    #[doc = "0x110 - Message Buffer 2 CS Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb2_64b_cs(&self) -> &Mb64bGroupMb2_64bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(272).cast() }
    }
    #[doc = "0x110 - Message Buffer 9 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs9(&self) -> &MbGroupCs9 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(272).cast() }
    }
    #[doc = "0x114 - Message Buffer 9 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb9_8b_id(&self) -> &Mb8bGroupMb9_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(276).cast() }
    }
    #[doc = "0x114 - Message Buffer 6 ID Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb6_16b_id(&self) -> &Mb16bGroupMb6_16bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(276).cast() }
    }
    #[doc = "0x114 - Message Buffer 3 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb3_32b_word5(&self) -> &Mb32bGroupMb3_32bWord5 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(276).cast() }
    }
    #[doc = "0x114 - Message Buffer 2 ID Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb2_64b_id(&self) -> &Mb64bGroupMb2_64bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(276).cast() }
    }
    #[doc = "0x114 - Message Buffer 9 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id9(&self) -> &MbGroupId9 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(276).cast() }
    }
    #[doc = "0x118 - Message Buffer 9 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word09(&self) -> &MbGroupWord09 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(280).cast() }
    }
    #[doc = "0x118 - Message Buffer 9 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb9_8b_word0(&self) -> &Mb8bGroupMb9_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(280).cast() }
    }
    #[doc = "0x118 - Message Buffer 6 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb6_16b_word0(&self) -> &Mb16bGroupMb6_16bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(280).cast() }
    }
    #[doc = "0x118 - Message Buffer 3 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb3_32b_word6(&self) -> &Mb32bGroupMb3_32bWord6 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(280).cast() }
    }
    #[doc = "0x118 - Message Buffer 2 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb2_64b_word0(&self) -> &Mb64bGroupMb2_64bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(280).cast() }
    }
    #[doc = "0x11c - Message Buffer 9 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word19(&self) -> &MbGroupWord19 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(284).cast() }
    }
    #[doc = "0x11c - Message Buffer 9 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb9_8b_word1(&self) -> &Mb8bGroupMb9_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(284).cast() }
    }
    #[doc = "0x11c - Message Buffer 6 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb6_16b_word1(&self) -> &Mb16bGroupMb6_16bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(284).cast() }
    }
    #[doc = "0x11c - Message Buffer 3 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb3_32b_word7(&self) -> &Mb32bGroupMb3_32bWord7 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(284).cast() }
    }
    #[doc = "0x11c - Message Buffer 2 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb2_64b_word1(&self) -> &Mb64bGroupMb2_64bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(284).cast() }
    }
    #[doc = "0x120 - Message Buffer 6 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb6_16b_word2(&self) -> &Mb16bGroupMb6_16bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(288).cast() }
    }
    #[doc = "0x120 - Message Buffer 4 CS Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb4_32b_cs(&self) -> &Mb32bGroupMb4_32bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(288).cast() }
    }
    #[doc = "0x120 - Message Buffer 2 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb2_64b_word2(&self) -> &Mb64bGroupMb2_64bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(288).cast() }
    }
    #[doc = "0x120 - Message Buffer 10 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb10_8b_cs(&self) -> &Mb8bGroupMb10_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(288).cast() }
    }
    #[doc = "0x120 - Message Buffer 10 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs10(&self) -> &MbGroupCs10 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(288).cast() }
    }
    #[doc = "0x124 - Message Buffer 6 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb6_16b_word3(&self) -> &Mb16bGroupMb6_16bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(292).cast() }
    }
    #[doc = "0x124 - Message Buffer 4 ID Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb4_32b_id(&self) -> &Mb32bGroupMb4_32bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(292).cast() }
    }
    #[doc = "0x124 - Message Buffer 2 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb2_64b_word3(&self) -> &Mb64bGroupMb2_64bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(292).cast() }
    }
    #[doc = "0x124 - Message Buffer 10 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb10_8b_id(&self) -> &Mb8bGroupMb10_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(292).cast() }
    }
    #[doc = "0x124 - Message Buffer 10 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id10(&self) -> &MbGroupId10 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(292).cast() }
    }
    #[doc = "0x128 - Message Buffer 10 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word010(&self) -> &MbGroupWord010 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(296).cast() }
    }
    #[doc = "0x128 - Message Buffer 7 CS Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb7_16b_cs(&self) -> &Mb16bGroupMb7_16bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(296).cast() }
    }
    #[doc = "0x128 - Message Buffer 4 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb4_32b_word0(&self) -> &Mb32bGroupMb4_32bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(296).cast() }
    }
    #[doc = "0x128 - Message Buffer 2 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb2_64b_word4(&self) -> &Mb64bGroupMb2_64bWord4 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(296).cast() }
    }
    #[doc = "0x128 - Message Buffer 10 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb10_8b_word0(&self) -> &Mb8bGroupMb10_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(296).cast() }
    }
    #[doc = "0x12c - Message Buffer 10 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word110(&self) -> &MbGroupWord110 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(300).cast() }
    }
    #[doc = "0x12c - Message Buffer 7 ID Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb7_16b_id(&self) -> &Mb16bGroupMb7_16bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(300).cast() }
    }
    #[doc = "0x12c - Message Buffer 4 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb4_32b_word1(&self) -> &Mb32bGroupMb4_32bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(300).cast() }
    }
    #[doc = "0x12c - Message Buffer 2 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb2_64b_word5(&self) -> &Mb64bGroupMb2_64bWord5 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(300).cast() }
    }
    #[doc = "0x12c - Message Buffer 10 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb10_8b_word1(&self) -> &Mb8bGroupMb10_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(300).cast() }
    }
    #[doc = "0x130 - Message Buffer 7 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb7_16b_word0(&self) -> &Mb16bGroupMb7_16bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(304).cast() }
    }
    #[doc = "0x130 - Message Buffer 4 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb4_32b_word2(&self) -> &Mb32bGroupMb4_32bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(304).cast() }
    }
    #[doc = "0x130 - Message Buffer 2 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb2_64b_word6(&self) -> &Mb64bGroupMb2_64bWord6 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(304).cast() }
    }
    #[doc = "0x130 - Message Buffer 11 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb11_8b_cs(&self) -> &Mb8bGroupMb11_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(304).cast() }
    }
    #[doc = "0x130 - Message Buffer 11 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs11(&self) -> &MbGroupCs11 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(304).cast() }
    }
    #[doc = "0x134 - Message Buffer 7 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb7_16b_word1(&self) -> &Mb16bGroupMb7_16bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(308).cast() }
    }
    #[doc = "0x134 - Message Buffer 4 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb4_32b_word3(&self) -> &Mb32bGroupMb4_32bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(308).cast() }
    }
    #[doc = "0x134 - Message Buffer 2 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb2_64b_word7(&self) -> &Mb64bGroupMb2_64bWord7 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(308).cast() }
    }
    #[doc = "0x134 - Message Buffer 11 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb11_8b_id(&self) -> &Mb8bGroupMb11_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(308).cast() }
    }
    #[doc = "0x134 - Message Buffer 11 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id11(&self) -> &MbGroupId11 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(308).cast() }
    }
    #[doc = "0x138 - Message Buffer 11 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word011(&self) -> &MbGroupWord011 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(312).cast() }
    }
    #[doc = "0x138 - Message Buffer 7 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb7_16b_word2(&self) -> &Mb16bGroupMb7_16bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(312).cast() }
    }
    #[doc = "0x138 - Message Buffer 4 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb4_32b_word4(&self) -> &Mb32bGroupMb4_32bWord4 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(312).cast() }
    }
    #[doc = "0x138 - Message Buffer 2 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb2_64b_word8(&self) -> &Mb64bGroupMb2_64bWord8 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(312).cast() }
    }
    #[doc = "0x138 - Message Buffer 11 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb11_8b_word0(&self) -> &Mb8bGroupMb11_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(312).cast() }
    }
    #[doc = "0x13c - Message Buffer 11 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word111(&self) -> &MbGroupWord111 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(316).cast() }
    }
    #[doc = "0x13c - Message Buffer 7 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb7_16b_word3(&self) -> &Mb16bGroupMb7_16bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(316).cast() }
    }
    #[doc = "0x13c - Message Buffer 4 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb4_32b_word5(&self) -> &Mb32bGroupMb4_32bWord5 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(316).cast() }
    }
    #[doc = "0x13c - Message Buffer 2 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb2_64b_word9(&self) -> &Mb64bGroupMb2_64bWord9 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(316).cast() }
    }
    #[doc = "0x13c - Message Buffer 11 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb11_8b_word1(&self) -> &Mb8bGroupMb11_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(316).cast() }
    }
    #[doc = "0x140 - Message Buffer 8 CS Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb8_16b_cs(&self) -> &Mb16bGroupMb8_16bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(320).cast() }
    }
    #[doc = "0x140 - Message Buffer 4 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb4_32b_word6(&self) -> &Mb32bGroupMb4_32bWord6 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(320).cast() }
    }
    #[doc = "0x140 - Message Buffer 2 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb2_64b_word10(&self) -> &Mb64bGroupMb2_64bWord10 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(320).cast() }
    }
    #[doc = "0x140 - Message Buffer 12 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb12_8b_cs(&self) -> &Mb8bGroupMb12_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(320).cast() }
    }
    #[doc = "0x140 - Message Buffer 12 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs12(&self) -> &MbGroupCs12 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(320).cast() }
    }
    #[doc = "0x144 - Message Buffer 8 ID Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb8_16b_id(&self) -> &Mb16bGroupMb8_16bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(324).cast() }
    }
    #[doc = "0x144 - Message Buffer 4 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb4_32b_word7(&self) -> &Mb32bGroupMb4_32bWord7 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(324).cast() }
    }
    #[doc = "0x144 - Message Buffer 2 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb2_64b_word11(&self) -> &Mb64bGroupMb2_64bWord11 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(324).cast() }
    }
    #[doc = "0x144 - Message Buffer 12 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb12_8b_id(&self) -> &Mb8bGroupMb12_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(324).cast() }
    }
    #[doc = "0x144 - Message Buffer 12 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id12(&self) -> &MbGroupId12 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(324).cast() }
    }
    #[doc = "0x148 - Message Buffer 12 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word012(&self) -> &MbGroupWord012 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(328).cast() }
    }
    #[doc = "0x148 - Message Buffer 8 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb8_16b_word0(&self) -> &Mb16bGroupMb8_16bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(328).cast() }
    }
    #[doc = "0x148 - Message Buffer 5 CS Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb5_32b_cs(&self) -> &Mb32bGroupMb5_32bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(328).cast() }
    }
    #[doc = "0x148 - Message Buffer 2 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb2_64b_word12(&self) -> &Mb64bGroupMb2_64bWord12 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(328).cast() }
    }
    #[doc = "0x148 - Message Buffer 12 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb12_8b_word0(&self) -> &Mb8bGroupMb12_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(328).cast() }
    }
    #[doc = "0x14c - Message Buffer 12 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word112(&self) -> &MbGroupWord112 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(332).cast() }
    }
    #[doc = "0x14c - Message Buffer 8 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb8_16b_word1(&self) -> &Mb16bGroupMb8_16bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(332).cast() }
    }
    #[doc = "0x14c - Message Buffer 5 ID Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb5_32b_id(&self) -> &Mb32bGroupMb5_32bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(332).cast() }
    }
    #[doc = "0x14c - Message Buffer 2 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb2_64b_word13(&self) -> &Mb64bGroupMb2_64bWord13 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(332).cast() }
    }
    #[doc = "0x14c - Message Buffer 12 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb12_8b_word1(&self) -> &Mb8bGroupMb12_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(332).cast() }
    }
    #[doc = "0x150 - Message Buffer 8 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb8_16b_word2(&self) -> &Mb16bGroupMb8_16bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(336).cast() }
    }
    #[doc = "0x150 - Message Buffer 5 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb5_32b_word0(&self) -> &Mb32bGroupMb5_32bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(336).cast() }
    }
    #[doc = "0x150 - Message Buffer 2 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb2_64b_word14(&self) -> &Mb64bGroupMb2_64bWord14 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(336).cast() }
    }
    #[doc = "0x150 - Message Buffer 13 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb13_8b_cs(&self) -> &Mb8bGroupMb13_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(336).cast() }
    }
    #[doc = "0x150 - Message Buffer 13 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs13(&self) -> &MbGroupCs13 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(336).cast() }
    }
    #[doc = "0x154 - Message Buffer 8 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb8_16b_word3(&self) -> &Mb16bGroupMb8_16bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(340).cast() }
    }
    #[doc = "0x154 - Message Buffer 5 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb5_32b_word1(&self) -> &Mb32bGroupMb5_32bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(340).cast() }
    }
    #[doc = "0x154 - Message Buffer 2 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb2_64b_word15(&self) -> &Mb64bGroupMb2_64bWord15 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(340).cast() }
    }
    #[doc = "0x154 - Message Buffer 13 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb13_8b_id(&self) -> &Mb8bGroupMb13_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(340).cast() }
    }
    #[doc = "0x154 - Message Buffer 13 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id13(&self) -> &MbGroupId13 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(340).cast() }
    }
    #[doc = "0x158 - Message Buffer 13 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word013(&self) -> &MbGroupWord013 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(344).cast() }
    }
    #[doc = "0x158 - Message Buffer 9 CS Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb9_16b_cs(&self) -> &Mb16bGroupMb9_16bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(344).cast() }
    }
    #[doc = "0x158 - Message Buffer 5 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb5_32b_word2(&self) -> &Mb32bGroupMb5_32bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(344).cast() }
    }
    #[doc = "0x158 - Message Buffer 3 CS Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb3_64b_cs(&self) -> &Mb64bGroupMb3_64bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(344).cast() }
    }
    #[doc = "0x158 - Message Buffer 13 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb13_8b_word0(&self) -> &Mb8bGroupMb13_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(344).cast() }
    }
    #[doc = "0x15c - Message Buffer 13 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word113(&self) -> &MbGroupWord113 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(348).cast() }
    }
    #[doc = "0x15c - Message Buffer 9 ID Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb9_16b_id(&self) -> &Mb16bGroupMb9_16bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(348).cast() }
    }
    #[doc = "0x15c - Message Buffer 5 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb5_32b_word3(&self) -> &Mb32bGroupMb5_32bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(348).cast() }
    }
    #[doc = "0x15c - Message Buffer 3 ID Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb3_64b_id(&self) -> &Mb64bGroupMb3_64bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(348).cast() }
    }
    #[doc = "0x15c - Message Buffer 13 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb13_8b_word1(&self) -> &Mb8bGroupMb13_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(348).cast() }
    }
    #[doc = "0x160 - Message Buffer 9 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb9_16b_word0(&self) -> &Mb16bGroupMb9_16bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(352).cast() }
    }
    #[doc = "0x160 - Message Buffer 5 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb5_32b_word4(&self) -> &Mb32bGroupMb5_32bWord4 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(352).cast() }
    }
    #[doc = "0x160 - Message Buffer 3 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb3_64b_word0(&self) -> &Mb64bGroupMb3_64bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(352).cast() }
    }
    #[doc = "0x160 - Message Buffer 14 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb14_8b_cs(&self) -> &Mb8bGroupMb14_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(352).cast() }
    }
    #[doc = "0x160 - Message Buffer 14 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs14(&self) -> &MbGroupCs14 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(352).cast() }
    }
    #[doc = "0x164 - Message Buffer 9 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb9_16b_word1(&self) -> &Mb16bGroupMb9_16bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(356).cast() }
    }
    #[doc = "0x164 - Message Buffer 5 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb5_32b_word5(&self) -> &Mb32bGroupMb5_32bWord5 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(356).cast() }
    }
    #[doc = "0x164 - Message Buffer 3 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb3_64b_word1(&self) -> &Mb64bGroupMb3_64bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(356).cast() }
    }
    #[doc = "0x164 - Message Buffer 14 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb14_8b_id(&self) -> &Mb8bGroupMb14_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(356).cast() }
    }
    #[doc = "0x164 - Message Buffer 14 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id14(&self) -> &MbGroupId14 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(356).cast() }
    }
    #[doc = "0x168 - Message Buffer 14 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word014(&self) -> &MbGroupWord014 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(360).cast() }
    }
    #[doc = "0x168 - Message Buffer 9 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb9_16b_word2(&self) -> &Mb16bGroupMb9_16bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(360).cast() }
    }
    #[doc = "0x168 - Message Buffer 5 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb5_32b_word6(&self) -> &Mb32bGroupMb5_32bWord6 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(360).cast() }
    }
    #[doc = "0x168 - Message Buffer 3 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb3_64b_word2(&self) -> &Mb64bGroupMb3_64bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(360).cast() }
    }
    #[doc = "0x168 - Message Buffer 14 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb14_8b_word0(&self) -> &Mb8bGroupMb14_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(360).cast() }
    }
    #[doc = "0x16c - Message Buffer 14 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word114(&self) -> &MbGroupWord114 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(364).cast() }
    }
    #[doc = "0x16c - Message Buffer 9 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb9_16b_word3(&self) -> &Mb16bGroupMb9_16bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(364).cast() }
    }
    #[doc = "0x16c - Message Buffer 5 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb5_32b_word7(&self) -> &Mb32bGroupMb5_32bWord7 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(364).cast() }
    }
    #[doc = "0x16c - Message Buffer 3 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb3_64b_word3(&self) -> &Mb64bGroupMb3_64bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(364).cast() }
    }
    #[doc = "0x16c - Message Buffer 14 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb14_8b_word1(&self) -> &Mb8bGroupMb14_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(364).cast() }
    }
    #[doc = "0x170 - Message Buffer 6 CS Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb6_32b_cs(&self) -> &Mb32bGroupMb6_32bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(368).cast() }
    }
    #[doc = "0x170 - Message Buffer 3 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb3_64b_word4(&self) -> &Mb64bGroupMb3_64bWord4 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(368).cast() }
    }
    #[doc = "0x170 - Message Buffer 15 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb15_8b_cs(&self) -> &Mb8bGroupMb15_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(368).cast() }
    }
    #[doc = "0x170 - Message Buffer 10 CS Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb10_16b_cs(&self) -> &Mb16bGroupMb10_16bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(368).cast() }
    }
    #[doc = "0x170 - Message Buffer 15 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs15(&self) -> &MbGroupCs15 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(368).cast() }
    }
    #[doc = "0x174 - Message Buffer 6 ID Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb6_32b_id(&self) -> &Mb32bGroupMb6_32bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(372).cast() }
    }
    #[doc = "0x174 - Message Buffer 3 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb3_64b_word5(&self) -> &Mb64bGroupMb3_64bWord5 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(372).cast() }
    }
    #[doc = "0x174 - Message Buffer 15 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb15_8b_id(&self) -> &Mb8bGroupMb15_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(372).cast() }
    }
    #[doc = "0x174 - Message Buffer 10 ID Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb10_16b_id(&self) -> &Mb16bGroupMb10_16bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(372).cast() }
    }
    #[doc = "0x174 - Message Buffer 15 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id15(&self) -> &MbGroupId15 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(372).cast() }
    }
    #[doc = "0x178 - Message Buffer 15 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word015(&self) -> &MbGroupWord015 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(376).cast() }
    }
    #[doc = "0x178 - Message Buffer 6 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb6_32b_word0(&self) -> &Mb32bGroupMb6_32bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(376).cast() }
    }
    #[doc = "0x178 - Message Buffer 3 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb3_64b_word6(&self) -> &Mb64bGroupMb3_64bWord6 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(376).cast() }
    }
    #[doc = "0x178 - Message Buffer 15 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb15_8b_word0(&self) -> &Mb8bGroupMb15_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(376).cast() }
    }
    #[doc = "0x178 - Message Buffer 10 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb10_16b_word0(&self) -> &Mb16bGroupMb10_16bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(376).cast() }
    }
    #[doc = "0x17c - Message Buffer 15 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word115(&self) -> &MbGroupWord115 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(380).cast() }
    }
    #[doc = "0x17c - Message Buffer 6 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb6_32b_word1(&self) -> &Mb32bGroupMb6_32bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(380).cast() }
    }
    #[doc = "0x17c - Message Buffer 3 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb3_64b_word7(&self) -> &Mb64bGroupMb3_64bWord7 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(380).cast() }
    }
    #[doc = "0x17c - Message Buffer 15 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb15_8b_word1(&self) -> &Mb8bGroupMb15_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(380).cast() }
    }
    #[doc = "0x17c - Message Buffer 10 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb10_16b_word1(&self) -> &Mb16bGroupMb10_16bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(380).cast() }
    }
    #[doc = "0x180 - Message Buffer 6 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb6_32b_word2(&self) -> &Mb32bGroupMb6_32bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(384).cast() }
    }
    #[doc = "0x180 - Message Buffer 3 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb3_64b_word8(&self) -> &Mb64bGroupMb3_64bWord8 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(384).cast() }
    }
    #[doc = "0x180 - Message Buffer 16 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb16_8b_cs(&self) -> &Mb8bGroupMb16_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(384).cast() }
    }
    #[doc = "0x180 - Message Buffer 10 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb10_16b_word2(&self) -> &Mb16bGroupMb10_16bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(384).cast() }
    }
    #[doc = "0x180 - Message Buffer 16 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs16(&self) -> &MbGroupCs16 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(384).cast() }
    }
    #[doc = "0x184 - Message Buffer 6 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb6_32b_word3(&self) -> &Mb32bGroupMb6_32bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(388).cast() }
    }
    #[doc = "0x184 - Message Buffer 3 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb3_64b_word9(&self) -> &Mb64bGroupMb3_64bWord9 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(388).cast() }
    }
    #[doc = "0x184 - Message Buffer 16 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb16_8b_id(&self) -> &Mb8bGroupMb16_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(388).cast() }
    }
    #[doc = "0x184 - Message Buffer 10 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb10_16b_word3(&self) -> &Mb16bGroupMb10_16bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(388).cast() }
    }
    #[doc = "0x184 - Message Buffer 16 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id16(&self) -> &MbGroupId16 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(388).cast() }
    }
    #[doc = "0x188 - Message Buffer 16 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word016(&self) -> &MbGroupWord016 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(392).cast() }
    }
    #[doc = "0x188 - Message Buffer 6 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb6_32b_word4(&self) -> &Mb32bGroupMb6_32bWord4 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(392).cast() }
    }
    #[doc = "0x188 - Message Buffer 3 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb3_64b_word10(&self) -> &Mb64bGroupMb3_64bWord10 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(392).cast() }
    }
    #[doc = "0x188 - Message Buffer 16 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb16_8b_word0(&self) -> &Mb8bGroupMb16_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(392).cast() }
    }
    #[doc = "0x188 - Message Buffer 11 CS Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb11_16b_cs(&self) -> &Mb16bGroupMb11_16bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(392).cast() }
    }
    #[doc = "0x18c - Message Buffer 16 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word116(&self) -> &MbGroupWord116 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(396).cast() }
    }
    #[doc = "0x18c - Message Buffer 6 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb6_32b_word5(&self) -> &Mb32bGroupMb6_32bWord5 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(396).cast() }
    }
    #[doc = "0x18c - Message Buffer 3 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb3_64b_word11(&self) -> &Mb64bGroupMb3_64bWord11 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(396).cast() }
    }
    #[doc = "0x18c - Message Buffer 16 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb16_8b_word1(&self) -> &Mb8bGroupMb16_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(396).cast() }
    }
    #[doc = "0x18c - Message Buffer 11 ID Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb11_16b_id(&self) -> &Mb16bGroupMb11_16bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(396).cast() }
    }
    #[doc = "0x190 - Message Buffer 6 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb6_32b_word6(&self) -> &Mb32bGroupMb6_32bWord6 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(400).cast() }
    }
    #[doc = "0x190 - Message Buffer 3 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb3_64b_word12(&self) -> &Mb64bGroupMb3_64bWord12 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(400).cast() }
    }
    #[doc = "0x190 - Message Buffer 17 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb17_8b_cs(&self) -> &Mb8bGroupMb17_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(400).cast() }
    }
    #[doc = "0x190 - Message Buffer 11 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb11_16b_word0(&self) -> &Mb16bGroupMb11_16bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(400).cast() }
    }
    #[doc = "0x190 - Message Buffer 17 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs17(&self) -> &MbGroupCs17 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(400).cast() }
    }
    #[doc = "0x194 - Message Buffer 6 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb6_32b_word7(&self) -> &Mb32bGroupMb6_32bWord7 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(404).cast() }
    }
    #[doc = "0x194 - Message Buffer 3 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb3_64b_word13(&self) -> &Mb64bGroupMb3_64bWord13 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(404).cast() }
    }
    #[doc = "0x194 - Message Buffer 17 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb17_8b_id(&self) -> &Mb8bGroupMb17_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(404).cast() }
    }
    #[doc = "0x194 - Message Buffer 11 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb11_16b_word1(&self) -> &Mb16bGroupMb11_16bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(404).cast() }
    }
    #[doc = "0x194 - Message Buffer 17 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id17(&self) -> &MbGroupId17 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(404).cast() }
    }
    #[doc = "0x198 - Message Buffer 17 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word017(&self) -> &MbGroupWord017 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(408).cast() }
    }
    #[doc = "0x198 - Message Buffer 7 CS Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb7_32b_cs(&self) -> &Mb32bGroupMb7_32bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(408).cast() }
    }
    #[doc = "0x198 - Message Buffer 3 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb3_64b_word14(&self) -> &Mb64bGroupMb3_64bWord14 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(408).cast() }
    }
    #[doc = "0x198 - Message Buffer 17 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb17_8b_word0(&self) -> &Mb8bGroupMb17_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(408).cast() }
    }
    #[doc = "0x198 - Message Buffer 11 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb11_16b_word2(&self) -> &Mb16bGroupMb11_16bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(408).cast() }
    }
    #[doc = "0x19c - Message Buffer 17 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word117(&self) -> &MbGroupWord117 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(412).cast() }
    }
    #[doc = "0x19c - Message Buffer 7 ID Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb7_32b_id(&self) -> &Mb32bGroupMb7_32bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(412).cast() }
    }
    #[doc = "0x19c - Message Buffer 3 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb3_64b_word15(&self) -> &Mb64bGroupMb3_64bWord15 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(412).cast() }
    }
    #[doc = "0x19c - Message Buffer 17 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb17_8b_word1(&self) -> &Mb8bGroupMb17_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(412).cast() }
    }
    #[doc = "0x19c - Message Buffer 11 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb11_16b_word3(&self) -> &Mb16bGroupMb11_16bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(412).cast() }
    }
    #[doc = "0x1a0 - Message Buffer 7 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb7_32b_word0(&self) -> &Mb32bGroupMb7_32bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(416).cast() }
    }
    #[doc = "0x1a0 - Message Buffer 4 CS Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb4_64b_cs(&self) -> &Mb64bGroupMb4_64bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(416).cast() }
    }
    #[doc = "0x1a0 - Message Buffer 18 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb18_8b_cs(&self) -> &Mb8bGroupMb18_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(416).cast() }
    }
    #[doc = "0x1a0 - Message Buffer 12 CS Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb12_16b_cs(&self) -> &Mb16bGroupMb12_16bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(416).cast() }
    }
    #[doc = "0x1a0 - Message Buffer 18 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs18(&self) -> &MbGroupCs18 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(416).cast() }
    }
    #[doc = "0x1a4 - Message Buffer 7 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb7_32b_word1(&self) -> &Mb32bGroupMb7_32bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(420).cast() }
    }
    #[doc = "0x1a4 - Message Buffer 4 ID Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb4_64b_id(&self) -> &Mb64bGroupMb4_64bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(420).cast() }
    }
    #[doc = "0x1a4 - Message Buffer 18 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb18_8b_id(&self) -> &Mb8bGroupMb18_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(420).cast() }
    }
    #[doc = "0x1a4 - Message Buffer 12 ID Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb12_16b_id(&self) -> &Mb16bGroupMb12_16bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(420).cast() }
    }
    #[doc = "0x1a4 - Message Buffer 18 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id18(&self) -> &MbGroupId18 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(420).cast() }
    }
    #[doc = "0x1a8 - Message Buffer 18 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word018(&self) -> &MbGroupWord018 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(424).cast() }
    }
    #[doc = "0x1a8 - Message Buffer 7 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb7_32b_word2(&self) -> &Mb32bGroupMb7_32bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(424).cast() }
    }
    #[doc = "0x1a8 - Message Buffer 4 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb4_64b_word0(&self) -> &Mb64bGroupMb4_64bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(424).cast() }
    }
    #[doc = "0x1a8 - Message Buffer 18 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb18_8b_word0(&self) -> &Mb8bGroupMb18_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(424).cast() }
    }
    #[doc = "0x1a8 - Message Buffer 12 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb12_16b_word0(&self) -> &Mb16bGroupMb12_16bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(424).cast() }
    }
    #[doc = "0x1ac - Message Buffer 18 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word118(&self) -> &MbGroupWord118 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(428).cast() }
    }
    #[doc = "0x1ac - Message Buffer 7 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb7_32b_word3(&self) -> &Mb32bGroupMb7_32bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(428).cast() }
    }
    #[doc = "0x1ac - Message Buffer 4 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb4_64b_word1(&self) -> &Mb64bGroupMb4_64bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(428).cast() }
    }
    #[doc = "0x1ac - Message Buffer 18 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb18_8b_word1(&self) -> &Mb8bGroupMb18_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(428).cast() }
    }
    #[doc = "0x1ac - Message Buffer 12 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb12_16b_word1(&self) -> &Mb16bGroupMb12_16bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(428).cast() }
    }
    #[doc = "0x1b0 - Message Buffer 7 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb7_32b_word4(&self) -> &Mb32bGroupMb7_32bWord4 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(432).cast() }
    }
    #[doc = "0x1b0 - Message Buffer 4 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb4_64b_word2(&self) -> &Mb64bGroupMb4_64bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(432).cast() }
    }
    #[doc = "0x1b0 - Message Buffer 19 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb19_8b_cs(&self) -> &Mb8bGroupMb19_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(432).cast() }
    }
    #[doc = "0x1b0 - Message Buffer 12 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb12_16b_word2(&self) -> &Mb16bGroupMb12_16bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(432).cast() }
    }
    #[doc = "0x1b0 - Message Buffer 19 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs19(&self) -> &MbGroupCs19 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(432).cast() }
    }
    #[doc = "0x1b4 - Message Buffer 7 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb7_32b_word5(&self) -> &Mb32bGroupMb7_32bWord5 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(436).cast() }
    }
    #[doc = "0x1b4 - Message Buffer 4 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb4_64b_word3(&self) -> &Mb64bGroupMb4_64bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(436).cast() }
    }
    #[doc = "0x1b4 - Message Buffer 19 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb19_8b_id(&self) -> &Mb8bGroupMb19_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(436).cast() }
    }
    #[doc = "0x1b4 - Message Buffer 12 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb12_16b_word3(&self) -> &Mb16bGroupMb12_16bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(436).cast() }
    }
    #[doc = "0x1b4 - Message Buffer 19 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id19(&self) -> &MbGroupId19 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(436).cast() }
    }
    #[doc = "0x1b8 - Message Buffer 19 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word019(&self) -> &MbGroupWord019 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(440).cast() }
    }
    #[doc = "0x1b8 - Message Buffer 7 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb7_32b_word6(&self) -> &Mb32bGroupMb7_32bWord6 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(440).cast() }
    }
    #[doc = "0x1b8 - Message Buffer 4 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb4_64b_word4(&self) -> &Mb64bGroupMb4_64bWord4 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(440).cast() }
    }
    #[doc = "0x1b8 - Message Buffer 19 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb19_8b_word0(&self) -> &Mb8bGroupMb19_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(440).cast() }
    }
    #[doc = "0x1b8 - Message Buffer 13 CS Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb13_16b_cs(&self) -> &Mb16bGroupMb13_16bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(440).cast() }
    }
    #[doc = "0x1bc - Message Buffer 19 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word119(&self) -> &MbGroupWord119 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(444).cast() }
    }
    #[doc = "0x1bc - Message Buffer 7 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb7_32b_word7(&self) -> &Mb32bGroupMb7_32bWord7 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(444).cast() }
    }
    #[doc = "0x1bc - Message Buffer 4 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb4_64b_word5(&self) -> &Mb64bGroupMb4_64bWord5 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(444).cast() }
    }
    #[doc = "0x1bc - Message Buffer 19 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb19_8b_word1(&self) -> &Mb8bGroupMb19_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(444).cast() }
    }
    #[doc = "0x1bc - Message Buffer 13 ID Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb13_16b_id(&self) -> &Mb16bGroupMb13_16bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(444).cast() }
    }
    #[doc = "0x1c0 - Message Buffer 8 CS Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb8_32b_cs(&self) -> &Mb32bGroupMb8_32bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(448).cast() }
    }
    #[doc = "0x1c0 - Message Buffer 4 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb4_64b_word6(&self) -> &Mb64bGroupMb4_64bWord6 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(448).cast() }
    }
    #[doc = "0x1c0 - Message Buffer 20 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb20_8b_cs(&self) -> &Mb8bGroupMb20_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(448).cast() }
    }
    #[doc = "0x1c0 - Message Buffer 13 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb13_16b_word0(&self) -> &Mb16bGroupMb13_16bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(448).cast() }
    }
    #[doc = "0x1c0 - Message Buffer 20 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs20(&self) -> &MbGroupCs20 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(448).cast() }
    }
    #[doc = "0x1c4 - Message Buffer 8 ID Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb8_32b_id(&self) -> &Mb32bGroupMb8_32bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(452).cast() }
    }
    #[doc = "0x1c4 - Message Buffer 4 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb4_64b_word7(&self) -> &Mb64bGroupMb4_64bWord7 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(452).cast() }
    }
    #[doc = "0x1c4 - Message Buffer 20 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb20_8b_id(&self) -> &Mb8bGroupMb20_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(452).cast() }
    }
    #[doc = "0x1c4 - Message Buffer 13 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb13_16b_word1(&self) -> &Mb16bGroupMb13_16bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(452).cast() }
    }
    #[doc = "0x1c4 - Message Buffer 20 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id20(&self) -> &MbGroupId20 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(452).cast() }
    }
    #[doc = "0x1c8 - Message Buffer 20 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word020(&self) -> &MbGroupWord020 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(456).cast() }
    }
    #[doc = "0x1c8 - Message Buffer 8 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb8_32b_word0(&self) -> &Mb32bGroupMb8_32bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(456).cast() }
    }
    #[doc = "0x1c8 - Message Buffer 4 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb4_64b_word8(&self) -> &Mb64bGroupMb4_64bWord8 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(456).cast() }
    }
    #[doc = "0x1c8 - Message Buffer 20 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb20_8b_word0(&self) -> &Mb8bGroupMb20_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(456).cast() }
    }
    #[doc = "0x1c8 - Message Buffer 13 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb13_16b_word2(&self) -> &Mb16bGroupMb13_16bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(456).cast() }
    }
    #[doc = "0x1cc - Message Buffer 20 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word120(&self) -> &MbGroupWord120 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(460).cast() }
    }
    #[doc = "0x1cc - Message Buffer 8 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb8_32b_word1(&self) -> &Mb32bGroupMb8_32bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(460).cast() }
    }
    #[doc = "0x1cc - Message Buffer 4 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb4_64b_word9(&self) -> &Mb64bGroupMb4_64bWord9 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(460).cast() }
    }
    #[doc = "0x1cc - Message Buffer 20 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb20_8b_word1(&self) -> &Mb8bGroupMb20_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(460).cast() }
    }
    #[doc = "0x1cc - Message Buffer 13 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb13_16b_word3(&self) -> &Mb16bGroupMb13_16bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(460).cast() }
    }
    #[doc = "0x1d0 - Message Buffer 8 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb8_32b_word2(&self) -> &Mb32bGroupMb8_32bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(464).cast() }
    }
    #[doc = "0x1d0 - Message Buffer 4 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb4_64b_word10(&self) -> &Mb64bGroupMb4_64bWord10 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(464).cast() }
    }
    #[doc = "0x1d0 - Message Buffer 21 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb21_8b_cs(&self) -> &Mb8bGroupMb21_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(464).cast() }
    }
    #[doc = "0x1d0 - Message Buffer 14 CS Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb14_16b_cs(&self) -> &Mb16bGroupMb14_16bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(464).cast() }
    }
    #[doc = "0x1d0 - Message Buffer 21 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs21(&self) -> &MbGroupCs21 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(464).cast() }
    }
    #[doc = "0x1d4 - Message Buffer 8 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb8_32b_word3(&self) -> &Mb32bGroupMb8_32bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(468).cast() }
    }
    #[doc = "0x1d4 - Message Buffer 4 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb4_64b_word11(&self) -> &Mb64bGroupMb4_64bWord11 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(468).cast() }
    }
    #[doc = "0x1d4 - Message Buffer 21 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb21_8b_id(&self) -> &Mb8bGroupMb21_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(468).cast() }
    }
    #[doc = "0x1d4 - Message Buffer 14 ID Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb14_16b_id(&self) -> &Mb16bGroupMb14_16bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(468).cast() }
    }
    #[doc = "0x1d4 - Message Buffer 21 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id21(&self) -> &MbGroupId21 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(468).cast() }
    }
    #[doc = "0x1d8 - Message Buffer 21 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word021(&self) -> &MbGroupWord021 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(472).cast() }
    }
    #[doc = "0x1d8 - Message Buffer 8 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb8_32b_word4(&self) -> &Mb32bGroupMb8_32bWord4 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(472).cast() }
    }
    #[doc = "0x1d8 - Message Buffer 4 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb4_64b_word12(&self) -> &Mb64bGroupMb4_64bWord12 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(472).cast() }
    }
    #[doc = "0x1d8 - Message Buffer 21 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb21_8b_word0(&self) -> &Mb8bGroupMb21_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(472).cast() }
    }
    #[doc = "0x1d8 - Message Buffer 14 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb14_16b_word0(&self) -> &Mb16bGroupMb14_16bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(472).cast() }
    }
    #[doc = "0x1dc - Message Buffer 21 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word121(&self) -> &MbGroupWord121 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(476).cast() }
    }
    #[doc = "0x1dc - Message Buffer 8 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb8_32b_word5(&self) -> &Mb32bGroupMb8_32bWord5 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(476).cast() }
    }
    #[doc = "0x1dc - Message Buffer 4 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb4_64b_word13(&self) -> &Mb64bGroupMb4_64bWord13 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(476).cast() }
    }
    #[doc = "0x1dc - Message Buffer 21 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb21_8b_word1(&self) -> &Mb8bGroupMb21_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(476).cast() }
    }
    #[doc = "0x1dc - Message Buffer 14 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb14_16b_word1(&self) -> &Mb16bGroupMb14_16bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(476).cast() }
    }
    #[doc = "0x1e0 - Message Buffer 8 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb8_32b_word6(&self) -> &Mb32bGroupMb8_32bWord6 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(480).cast() }
    }
    #[doc = "0x1e0 - Message Buffer 4 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb4_64b_word14(&self) -> &Mb64bGroupMb4_64bWord14 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(480).cast() }
    }
    #[doc = "0x1e0 - Message Buffer 22 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb22_8b_cs(&self) -> &Mb8bGroupMb22_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(480).cast() }
    }
    #[doc = "0x1e0 - Message Buffer 14 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb14_16b_word2(&self) -> &Mb16bGroupMb14_16bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(480).cast() }
    }
    #[doc = "0x1e0 - Message Buffer 22 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs22(&self) -> &MbGroupCs22 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(480).cast() }
    }
    #[doc = "0x1e4 - Message Buffer 8 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb8_32b_word7(&self) -> &Mb32bGroupMb8_32bWord7 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(484).cast() }
    }
    #[doc = "0x1e4 - Message Buffer 4 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb4_64b_word15(&self) -> &Mb64bGroupMb4_64bWord15 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(484).cast() }
    }
    #[doc = "0x1e4 - Message Buffer 22 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb22_8b_id(&self) -> &Mb8bGroupMb22_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(484).cast() }
    }
    #[doc = "0x1e4 - Message Buffer 14 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb14_16b_word3(&self) -> &Mb16bGroupMb14_16bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(484).cast() }
    }
    #[doc = "0x1e4 - Message Buffer 22 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id22(&self) -> &MbGroupId22 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(484).cast() }
    }
    #[doc = "0x1e8 - Message Buffer 22 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word022(&self) -> &MbGroupWord022 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(488).cast() }
    }
    #[doc = "0x1e8 - Message Buffer 9 CS Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb9_32b_cs(&self) -> &Mb32bGroupMb9_32bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(488).cast() }
    }
    #[doc = "0x1e8 - Message Buffer 5 CS Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb5_64b_cs(&self) -> &Mb64bGroupMb5_64bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(488).cast() }
    }
    #[doc = "0x1e8 - Message Buffer 22 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb22_8b_word0(&self) -> &Mb8bGroupMb22_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(488).cast() }
    }
    #[doc = "0x1e8 - Message Buffer 15 CS Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb15_16b_cs(&self) -> &Mb16bGroupMb15_16bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(488).cast() }
    }
    #[doc = "0x1ec - Message Buffer 22 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word122(&self) -> &MbGroupWord122 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(492).cast() }
    }
    #[doc = "0x1ec - Message Buffer 9 ID Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb9_32b_id(&self) -> &Mb32bGroupMb9_32bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(492).cast() }
    }
    #[doc = "0x1ec - Message Buffer 5 ID Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb5_64b_id(&self) -> &Mb64bGroupMb5_64bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(492).cast() }
    }
    #[doc = "0x1ec - Message Buffer 22 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb22_8b_word1(&self) -> &Mb8bGroupMb22_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(492).cast() }
    }
    #[doc = "0x1ec - Message Buffer 15 ID Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb15_16b_id(&self) -> &Mb16bGroupMb15_16bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(492).cast() }
    }
    #[doc = "0x1f0 - Message Buffer 9 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb9_32b_word0(&self) -> &Mb32bGroupMb9_32bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(496).cast() }
    }
    #[doc = "0x1f0 - Message Buffer 5 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb5_64b_word0(&self) -> &Mb64bGroupMb5_64bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(496).cast() }
    }
    #[doc = "0x1f0 - Message Buffer 23 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb23_8b_cs(&self) -> &Mb8bGroupMb23_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(496).cast() }
    }
    #[doc = "0x1f0 - Message Buffer 15 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb15_16b_word0(&self) -> &Mb16bGroupMb15_16bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(496).cast() }
    }
    #[doc = "0x1f0 - Message Buffer 23 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs23(&self) -> &MbGroupCs23 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(496).cast() }
    }
    #[doc = "0x1f4 - Message Buffer 9 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb9_32b_word1(&self) -> &Mb32bGroupMb9_32bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(500).cast() }
    }
    #[doc = "0x1f4 - Message Buffer 5 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb5_64b_word1(&self) -> &Mb64bGroupMb5_64bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(500).cast() }
    }
    #[doc = "0x1f4 - Message Buffer 23 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb23_8b_id(&self) -> &Mb8bGroupMb23_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(500).cast() }
    }
    #[doc = "0x1f4 - Message Buffer 15 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb15_16b_word1(&self) -> &Mb16bGroupMb15_16bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(500).cast() }
    }
    #[doc = "0x1f4 - Message Buffer 23 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id23(&self) -> &MbGroupId23 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(500).cast() }
    }
    #[doc = "0x1f8 - Message Buffer 23 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word023(&self) -> &MbGroupWord023 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(504).cast() }
    }
    #[doc = "0x1f8 - Message Buffer 9 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb9_32b_word2(&self) -> &Mb32bGroupMb9_32bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(504).cast() }
    }
    #[doc = "0x1f8 - Message Buffer 5 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb5_64b_word2(&self) -> &Mb64bGroupMb5_64bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(504).cast() }
    }
    #[doc = "0x1f8 - Message Buffer 23 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb23_8b_word0(&self) -> &Mb8bGroupMb23_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(504).cast() }
    }
    #[doc = "0x1f8 - Message Buffer 15 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb15_16b_word2(&self) -> &Mb16bGroupMb15_16bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(504).cast() }
    }
    #[doc = "0x1fc - Message Buffer 23 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word123(&self) -> &MbGroupWord123 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(508).cast() }
    }
    #[doc = "0x1fc - Message Buffer 9 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb9_32b_word3(&self) -> &Mb32bGroupMb9_32bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(508).cast() }
    }
    #[doc = "0x1fc - Message Buffer 5 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb5_64b_word3(&self) -> &Mb64bGroupMb5_64bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(508).cast() }
    }
    #[doc = "0x1fc - Message Buffer 23 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb23_8b_word1(&self) -> &Mb8bGroupMb23_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(508).cast() }
    }
    #[doc = "0x1fc - Message Buffer 15 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb15_16b_word3(&self) -> &Mb16bGroupMb15_16bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(508).cast() }
    }
    #[doc = "0x200 - Message Buffer 9 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb9_32b_word4(&self) -> &Mb32bGroupMb9_32bWord4 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(512).cast() }
    }
    #[doc = "0x200 - Message Buffer 5 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb5_64b_word4(&self) -> &Mb64bGroupMb5_64bWord4 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(512).cast() }
    }
    #[doc = "0x200 - Message Buffer 24 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb24_8b_cs(&self) -> &Mb8bGroupMb24_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(512).cast() }
    }
    #[doc = "0x200 - Message Buffer 16 CS Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb16_16b_cs(&self) -> &Mb16bGroupMb16_16bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(512).cast() }
    }
    #[doc = "0x200 - Message Buffer 24 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs24(&self) -> &MbGroupCs24 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(512).cast() }
    }
    #[doc = "0x204 - Message Buffer 9 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb9_32b_word5(&self) -> &Mb32bGroupMb9_32bWord5 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(516).cast() }
    }
    #[doc = "0x204 - Message Buffer 5 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb5_64b_word5(&self) -> &Mb64bGroupMb5_64bWord5 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(516).cast() }
    }
    #[doc = "0x204 - Message Buffer 24 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb24_8b_id(&self) -> &Mb8bGroupMb24_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(516).cast() }
    }
    #[doc = "0x204 - Message Buffer 16 ID Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb16_16b_id(&self) -> &Mb16bGroupMb16_16bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(516).cast() }
    }
    #[doc = "0x204 - Message Buffer 24 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id24(&self) -> &MbGroupId24 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(516).cast() }
    }
    #[doc = "0x208 - Message Buffer 24 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word024(&self) -> &MbGroupWord024 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(520).cast() }
    }
    #[doc = "0x208 - Message Buffer 9 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb9_32b_word6(&self) -> &Mb32bGroupMb9_32bWord6 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(520).cast() }
    }
    #[doc = "0x208 - Message Buffer 5 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb5_64b_word6(&self) -> &Mb64bGroupMb5_64bWord6 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(520).cast() }
    }
    #[doc = "0x208 - Message Buffer 24 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb24_8b_word0(&self) -> &Mb8bGroupMb24_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(520).cast() }
    }
    #[doc = "0x208 - Message Buffer 16 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb16_16b_word0(&self) -> &Mb16bGroupMb16_16bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(520).cast() }
    }
    #[doc = "0x20c - Message Buffer 24 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word124(&self) -> &MbGroupWord124 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(524).cast() }
    }
    #[doc = "0x20c - Message Buffer 9 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb9_32b_word7(&self) -> &Mb32bGroupMb9_32bWord7 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(524).cast() }
    }
    #[doc = "0x20c - Message Buffer 5 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb5_64b_word7(&self) -> &Mb64bGroupMb5_64bWord7 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(524).cast() }
    }
    #[doc = "0x20c - Message Buffer 24 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb24_8b_word1(&self) -> &Mb8bGroupMb24_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(524).cast() }
    }
    #[doc = "0x20c - Message Buffer 16 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb16_16b_word1(&self) -> &Mb16bGroupMb16_16bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(524).cast() }
    }
    #[doc = "0x210 - Message Buffer 5 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb5_64b_word8(&self) -> &Mb64bGroupMb5_64bWord8 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(528).cast() }
    }
    #[doc = "0x210 - Message Buffer 25 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb25_8b_cs(&self) -> &Mb8bGroupMb25_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(528).cast() }
    }
    #[doc = "0x210 - Message Buffer 16 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb16_16b_word2(&self) -> &Mb16bGroupMb16_16bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(528).cast() }
    }
    #[doc = "0x210 - Message Buffer 10 CS Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb10_32b_cs(&self) -> &Mb32bGroupMb10_32bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(528).cast() }
    }
    #[doc = "0x210 - Message Buffer 25 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs25(&self) -> &MbGroupCs25 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(528).cast() }
    }
    #[doc = "0x214 - Message Buffer 5 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb5_64b_word9(&self) -> &Mb64bGroupMb5_64bWord9 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(532).cast() }
    }
    #[doc = "0x214 - Message Buffer 25 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb25_8b_id(&self) -> &Mb8bGroupMb25_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(532).cast() }
    }
    #[doc = "0x214 - Message Buffer 16 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb16_16b_word3(&self) -> &Mb16bGroupMb16_16bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(532).cast() }
    }
    #[doc = "0x214 - Message Buffer 10 ID Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb10_32b_id(&self) -> &Mb32bGroupMb10_32bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(532).cast() }
    }
    #[doc = "0x214 - Message Buffer 25 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id25(&self) -> &MbGroupId25 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(532).cast() }
    }
    #[doc = "0x218 - Message Buffer 25 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word025(&self) -> &MbGroupWord025 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(536).cast() }
    }
    #[doc = "0x218 - Message Buffer 5 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb5_64b_word10(&self) -> &Mb64bGroupMb5_64bWord10 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(536).cast() }
    }
    #[doc = "0x218 - Message Buffer 25 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb25_8b_word0(&self) -> &Mb8bGroupMb25_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(536).cast() }
    }
    #[doc = "0x218 - Message Buffer 17 CS Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb17_16b_cs(&self) -> &Mb16bGroupMb17_16bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(536).cast() }
    }
    #[doc = "0x218 - Message Buffer 10 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb10_32b_word0(&self) -> &Mb32bGroupMb10_32bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(536).cast() }
    }
    #[doc = "0x21c - Message Buffer 25 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word125(&self) -> &MbGroupWord125 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(540).cast() }
    }
    #[doc = "0x21c - Message Buffer 5 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb5_64b_word11(&self) -> &Mb64bGroupMb5_64bWord11 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(540).cast() }
    }
    #[doc = "0x21c - Message Buffer 25 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb25_8b_word1(&self) -> &Mb8bGroupMb25_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(540).cast() }
    }
    #[doc = "0x21c - Message Buffer 17 ID Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb17_16b_id(&self) -> &Mb16bGroupMb17_16bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(540).cast() }
    }
    #[doc = "0x21c - Message Buffer 10 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb10_32b_word1(&self) -> &Mb32bGroupMb10_32bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(540).cast() }
    }
    #[doc = "0x220 - Message Buffer 5 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb5_64b_word12(&self) -> &Mb64bGroupMb5_64bWord12 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(544).cast() }
    }
    #[doc = "0x220 - Message Buffer 26 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb26_8b_cs(&self) -> &Mb8bGroupMb26_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(544).cast() }
    }
    #[doc = "0x220 - Message Buffer 17 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb17_16b_word0(&self) -> &Mb16bGroupMb17_16bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(544).cast() }
    }
    #[doc = "0x220 - Message Buffer 10 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb10_32b_word2(&self) -> &Mb32bGroupMb10_32bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(544).cast() }
    }
    #[doc = "0x220 - Message Buffer 26 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs26(&self) -> &MbGroupCs26 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(544).cast() }
    }
    #[doc = "0x224 - Message Buffer 5 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb5_64b_word13(&self) -> &Mb64bGroupMb5_64bWord13 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(548).cast() }
    }
    #[doc = "0x224 - Message Buffer 26 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb26_8b_id(&self) -> &Mb8bGroupMb26_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(548).cast() }
    }
    #[doc = "0x224 - Message Buffer 17 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb17_16b_word1(&self) -> &Mb16bGroupMb17_16bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(548).cast() }
    }
    #[doc = "0x224 - Message Buffer 10 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb10_32b_word3(&self) -> &Mb32bGroupMb10_32bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(548).cast() }
    }
    #[doc = "0x224 - Message Buffer 26 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id26(&self) -> &MbGroupId26 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(548).cast() }
    }
    #[doc = "0x228 - Message Buffer 26 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word026(&self) -> &MbGroupWord026 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(552).cast() }
    }
    #[doc = "0x228 - Message Buffer 5 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb5_64b_word14(&self) -> &Mb64bGroupMb5_64bWord14 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(552).cast() }
    }
    #[doc = "0x228 - Message Buffer 26 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb26_8b_word0(&self) -> &Mb8bGroupMb26_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(552).cast() }
    }
    #[doc = "0x228 - Message Buffer 17 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb17_16b_word2(&self) -> &Mb16bGroupMb17_16bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(552).cast() }
    }
    #[doc = "0x228 - Message Buffer 10 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb10_32b_word4(&self) -> &Mb32bGroupMb10_32bWord4 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(552).cast() }
    }
    #[doc = "0x22c - Message Buffer 26 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word126(&self) -> &MbGroupWord126 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(556).cast() }
    }
    #[doc = "0x22c - Message Buffer 5 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb5_64b_word15(&self) -> &Mb64bGroupMb5_64bWord15 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(556).cast() }
    }
    #[doc = "0x22c - Message Buffer 26 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb26_8b_word1(&self) -> &Mb8bGroupMb26_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(556).cast() }
    }
    #[doc = "0x22c - Message Buffer 17 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb17_16b_word3(&self) -> &Mb16bGroupMb17_16bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(556).cast() }
    }
    #[doc = "0x22c - Message Buffer 10 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb10_32b_word5(&self) -> &Mb32bGroupMb10_32bWord5 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(556).cast() }
    }
    #[doc = "0x230 - Message Buffer 6 CS Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb6_64b_cs(&self) -> &Mb64bGroupMb6_64bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(560).cast() }
    }
    #[doc = "0x230 - Message Buffer 27 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb27_8b_cs(&self) -> &Mb8bGroupMb27_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(560).cast() }
    }
    #[doc = "0x230 - Message Buffer 18 CS Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb18_16b_cs(&self) -> &Mb16bGroupMb18_16bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(560).cast() }
    }
    #[doc = "0x230 - Message Buffer 10 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb10_32b_word6(&self) -> &Mb32bGroupMb10_32bWord6 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(560).cast() }
    }
    #[doc = "0x230 - Message Buffer 27 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs27(&self) -> &MbGroupCs27 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(560).cast() }
    }
    #[doc = "0x234 - Message Buffer 6 ID Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb6_64b_id(&self) -> &Mb64bGroupMb6_64bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(564).cast() }
    }
    #[doc = "0x234 - Message Buffer 27 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb27_8b_id(&self) -> &Mb8bGroupMb27_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(564).cast() }
    }
    #[doc = "0x234 - Message Buffer 18 ID Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb18_16b_id(&self) -> &Mb16bGroupMb18_16bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(564).cast() }
    }
    #[doc = "0x234 - Message Buffer 10 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb10_32b_word7(&self) -> &Mb32bGroupMb10_32bWord7 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(564).cast() }
    }
    #[doc = "0x234 - Message Buffer 27 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id27(&self) -> &MbGroupId27 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(564).cast() }
    }
    #[doc = "0x238 - Message Buffer 27 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word027(&self) -> &MbGroupWord027 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(568).cast() }
    }
    #[doc = "0x238 - Message Buffer 6 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb6_64b_word0(&self) -> &Mb64bGroupMb6_64bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(568).cast() }
    }
    #[doc = "0x238 - Message Buffer 27 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb27_8b_word0(&self) -> &Mb8bGroupMb27_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(568).cast() }
    }
    #[doc = "0x238 - Message Buffer 18 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb18_16b_word0(&self) -> &Mb16bGroupMb18_16bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(568).cast() }
    }
    #[doc = "0x238 - Message Buffer 11 CS Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb11_32b_cs(&self) -> &Mb32bGroupMb11_32bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(568).cast() }
    }
    #[doc = "0x23c - Message Buffer 27 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word127(&self) -> &MbGroupWord127 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(572).cast() }
    }
    #[doc = "0x23c - Message Buffer 6 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb6_64b_word1(&self) -> &Mb64bGroupMb6_64bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(572).cast() }
    }
    #[doc = "0x23c - Message Buffer 27 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb27_8b_word1(&self) -> &Mb8bGroupMb27_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(572).cast() }
    }
    #[doc = "0x23c - Message Buffer 18 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb18_16b_word1(&self) -> &Mb16bGroupMb18_16bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(572).cast() }
    }
    #[doc = "0x23c - Message Buffer 11 ID Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb11_32b_id(&self) -> &Mb32bGroupMb11_32bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(572).cast() }
    }
    #[doc = "0x240 - Message Buffer 6 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb6_64b_word2(&self) -> &Mb64bGroupMb6_64bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(576).cast() }
    }
    #[doc = "0x240 - Message Buffer 28 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb28_8b_cs(&self) -> &Mb8bGroupMb28_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(576).cast() }
    }
    #[doc = "0x240 - Message Buffer 18 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb18_16b_word2(&self) -> &Mb16bGroupMb18_16bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(576).cast() }
    }
    #[doc = "0x240 - Message Buffer 11 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb11_32b_word0(&self) -> &Mb32bGroupMb11_32bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(576).cast() }
    }
    #[doc = "0x240 - Message Buffer 28 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs28(&self) -> &MbGroupCs28 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(576).cast() }
    }
    #[doc = "0x244 - Message Buffer 6 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb6_64b_word3(&self) -> &Mb64bGroupMb6_64bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(580).cast() }
    }
    #[doc = "0x244 - Message Buffer 28 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb28_8b_id(&self) -> &Mb8bGroupMb28_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(580).cast() }
    }
    #[doc = "0x244 - Message Buffer 18 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb18_16b_word3(&self) -> &Mb16bGroupMb18_16bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(580).cast() }
    }
    #[doc = "0x244 - Message Buffer 11 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb11_32b_word1(&self) -> &Mb32bGroupMb11_32bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(580).cast() }
    }
    #[doc = "0x244 - Message Buffer 28 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id28(&self) -> &MbGroupId28 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(580).cast() }
    }
    #[doc = "0x248 - Message Buffer 28 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word028(&self) -> &MbGroupWord028 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(584).cast() }
    }
    #[doc = "0x248 - Message Buffer 6 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb6_64b_word4(&self) -> &Mb64bGroupMb6_64bWord4 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(584).cast() }
    }
    #[doc = "0x248 - Message Buffer 28 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb28_8b_word0(&self) -> &Mb8bGroupMb28_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(584).cast() }
    }
    #[doc = "0x248 - Message Buffer 19 CS Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb19_16b_cs(&self) -> &Mb16bGroupMb19_16bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(584).cast() }
    }
    #[doc = "0x248 - Message Buffer 11 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb11_32b_word2(&self) -> &Mb32bGroupMb11_32bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(584).cast() }
    }
    #[doc = "0x24c - Message Buffer 28 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word128(&self) -> &MbGroupWord128 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(588).cast() }
    }
    #[doc = "0x24c - Message Buffer 6 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb6_64b_word5(&self) -> &Mb64bGroupMb6_64bWord5 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(588).cast() }
    }
    #[doc = "0x24c - Message Buffer 28 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb28_8b_word1(&self) -> &Mb8bGroupMb28_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(588).cast() }
    }
    #[doc = "0x24c - Message Buffer 19 ID Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb19_16b_id(&self) -> &Mb16bGroupMb19_16bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(588).cast() }
    }
    #[doc = "0x24c - Message Buffer 11 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb11_32b_word3(&self) -> &Mb32bGroupMb11_32bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(588).cast() }
    }
    #[doc = "0x250 - Message Buffer 6 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb6_64b_word6(&self) -> &Mb64bGroupMb6_64bWord6 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(592).cast() }
    }
    #[doc = "0x250 - Message Buffer 29 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb29_8b_cs(&self) -> &Mb8bGroupMb29_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(592).cast() }
    }
    #[doc = "0x250 - Message Buffer 19 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb19_16b_word0(&self) -> &Mb16bGroupMb19_16bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(592).cast() }
    }
    #[doc = "0x250 - Message Buffer 11 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb11_32b_word4(&self) -> &Mb32bGroupMb11_32bWord4 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(592).cast() }
    }
    #[doc = "0x250 - Message Buffer 29 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs29(&self) -> &MbGroupCs29 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(592).cast() }
    }
    #[doc = "0x254 - Message Buffer 6 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb6_64b_word7(&self) -> &Mb64bGroupMb6_64bWord7 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(596).cast() }
    }
    #[doc = "0x254 - Message Buffer 29 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb29_8b_id(&self) -> &Mb8bGroupMb29_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(596).cast() }
    }
    #[doc = "0x254 - Message Buffer 19 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb19_16b_word1(&self) -> &Mb16bGroupMb19_16bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(596).cast() }
    }
    #[doc = "0x254 - Message Buffer 11 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb11_32b_word5(&self) -> &Mb32bGroupMb11_32bWord5 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(596).cast() }
    }
    #[doc = "0x254 - Message Buffer 29 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id29(&self) -> &MbGroupId29 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(596).cast() }
    }
    #[doc = "0x258 - Message Buffer 29 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word029(&self) -> &MbGroupWord029 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(600).cast() }
    }
    #[doc = "0x258 - Message Buffer 6 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb6_64b_word8(&self) -> &Mb64bGroupMb6_64bWord8 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(600).cast() }
    }
    #[doc = "0x258 - Message Buffer 29 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb29_8b_word0(&self) -> &Mb8bGroupMb29_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(600).cast() }
    }
    #[doc = "0x258 - Message Buffer 19 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb19_16b_word2(&self) -> &Mb16bGroupMb19_16bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(600).cast() }
    }
    #[doc = "0x258 - Message Buffer 11 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb11_32b_word6(&self) -> &Mb32bGroupMb11_32bWord6 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(600).cast() }
    }
    #[doc = "0x25c - Message Buffer 29 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word129(&self) -> &MbGroupWord129 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(604).cast() }
    }
    #[doc = "0x25c - Message Buffer 6 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb6_64b_word9(&self) -> &Mb64bGroupMb6_64bWord9 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(604).cast() }
    }
    #[doc = "0x25c - Message Buffer 29 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb29_8b_word1(&self) -> &Mb8bGroupMb29_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(604).cast() }
    }
    #[doc = "0x25c - Message Buffer 19 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb19_16b_word3(&self) -> &Mb16bGroupMb19_16bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(604).cast() }
    }
    #[doc = "0x25c - Message Buffer 11 WORD_32B Register"]
    #[inline(always)]
    pub const fn mb_32b_group_mb11_32b_word7(&self) -> &Mb32bGroupMb11_32bWord7 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(604).cast() }
    }
    #[doc = "0x260 - Message Buffer 6 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb6_64b_word10(&self) -> &Mb64bGroupMb6_64bWord10 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(608).cast() }
    }
    #[doc = "0x260 - Message Buffer 30 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb30_8b_cs(&self) -> &Mb8bGroupMb30_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(608).cast() }
    }
    #[doc = "0x260 - Message Buffer 20 CS Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb20_16b_cs(&self) -> &Mb16bGroupMb20_16bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(608).cast() }
    }
    #[doc = "0x260 - Message Buffer 30 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs30(&self) -> &MbGroupCs30 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(608).cast() }
    }
    #[doc = "0x264 - Message Buffer 6 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb6_64b_word11(&self) -> &Mb64bGroupMb6_64bWord11 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(612).cast() }
    }
    #[doc = "0x264 - Message Buffer 30 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb30_8b_id(&self) -> &Mb8bGroupMb30_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(612).cast() }
    }
    #[doc = "0x264 - Message Buffer 20 ID Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb20_16b_id(&self) -> &Mb16bGroupMb20_16bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(612).cast() }
    }
    #[doc = "0x264 - Message Buffer 30 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id30(&self) -> &MbGroupId30 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(612).cast() }
    }
    #[doc = "0x268 - Message Buffer 30 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word030(&self) -> &MbGroupWord030 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(616).cast() }
    }
    #[doc = "0x268 - Message Buffer 6 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb6_64b_word12(&self) -> &Mb64bGroupMb6_64bWord12 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(616).cast() }
    }
    #[doc = "0x268 - Message Buffer 30 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb30_8b_word0(&self) -> &Mb8bGroupMb30_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(616).cast() }
    }
    #[doc = "0x268 - Message Buffer 20 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb20_16b_word0(&self) -> &Mb16bGroupMb20_16bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(616).cast() }
    }
    #[doc = "0x26c - Message Buffer 30 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word130(&self) -> &MbGroupWord130 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(620).cast() }
    }
    #[doc = "0x26c - Message Buffer 6 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb6_64b_word13(&self) -> &Mb64bGroupMb6_64bWord13 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(620).cast() }
    }
    #[doc = "0x26c - Message Buffer 30 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb30_8b_word1(&self) -> &Mb8bGroupMb30_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(620).cast() }
    }
    #[doc = "0x26c - Message Buffer 20 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb20_16b_word1(&self) -> &Mb16bGroupMb20_16bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(620).cast() }
    }
    #[doc = "0x270 - Message Buffer 6 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb6_64b_word14(&self) -> &Mb64bGroupMb6_64bWord14 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(624).cast() }
    }
    #[doc = "0x270 - Message Buffer 31 CS Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb31_8b_cs(&self) -> &Mb8bGroupMb31_8bCs {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(624).cast() }
    }
    #[doc = "0x270 - Message Buffer 20 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb20_16b_word2(&self) -> &Mb16bGroupMb20_16bWord2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(624).cast() }
    }
    #[doc = "0x270 - Message Buffer 31 CS Register"]
    #[inline(always)]
    pub const fn mb_group_cs31(&self) -> &MbGroupCs31 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(624).cast() }
    }
    #[doc = "0x274 - Message Buffer 6 WORD_64B Register"]
    #[inline(always)]
    pub const fn mb_64b_group_mb6_64b_word15(&self) -> &Mb64bGroupMb6_64bWord15 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(628).cast() }
    }
    #[doc = "0x274 - Message Buffer 31 ID Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb31_8b_id(&self) -> &Mb8bGroupMb31_8bId {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(628).cast() }
    }
    #[doc = "0x274 - Message Buffer 20 WORD_16B Register"]
    #[inline(always)]
    pub const fn mb_16b_group_mb20_16b_word3(&self) -> &Mb16bGroupMb20_16bWord3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(628).cast() }
    }
    #[doc = "0x274 - Message Buffer 31 ID Register"]
    #[inline(always)]
    pub const fn mb_group_id31(&self) -> &MbGroupId31 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(628).cast() }
    }
    #[doc = "0x278 - Message Buffer 31 WORD0 Register"]
    #[inline(always)]
    pub const fn mb_group_word031(&self) -> &MbGroupWord031 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(632).cast() }
    }
    #[doc = "0x278 - Message Buffer 31 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb31_8b_word0(&self) -> &Mb8bGroupMb31_8bWord0 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(632).cast() }
    }
    #[doc = "0x27c - Message Buffer 31 WORD1 Register"]
    #[inline(always)]
    pub const fn mb_group_word131(&self) -> &MbGroupWord131 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(636).cast() }
    }
    #[doc = "0x27c - Message Buffer 31 WORD_8B Register"]
    #[inline(always)]
    pub const fn mb_8b_group_mb31_8b_word1(&self) -> &Mb8bGroupMb31_8bWord1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(636).cast() }
    }
    #[doc = "0x880..0x900 - Receive Individual Mask"]
    #[inline(always)]
    pub const fn rximr(&self, n: usize) -> &Rximr {
        &self.rximr[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x880..0x900 - Receive Individual Mask"]
    #[inline(always)]
    pub fn rximr_iter(&self) -> impl Iterator<Item = &Rximr> {
        self.rximr.iter()
    }
    #[doc = "0xb00 - Pretended Networking Control 1"]
    #[inline(always)]
    pub const fn ctrl1_pn(&self) -> &Ctrl1Pn {
        &self.ctrl1_pn
    }
    #[doc = "0xb04 - Pretended Networking Control 2"]
    #[inline(always)]
    pub const fn ctrl2_pn(&self) -> &Ctrl2Pn {
        &self.ctrl2_pn
    }
    #[doc = "0xb08 - Pretended Networking Wake-Up Match"]
    #[inline(always)]
    pub const fn wu_mtc(&self) -> &WuMtc {
        &self.wu_mtc
    }
    #[doc = "0xb0c - Pretended Networking ID Filter 1"]
    #[inline(always)]
    pub const fn flt_id1(&self) -> &FltId1 {
        &self.flt_id1
    }
    #[doc = "0xb10 - Pretended Networking Data Length Code (DLC) Filter"]
    #[inline(always)]
    pub const fn flt_dlc(&self) -> &FltDlc {
        &self.flt_dlc
    }
    #[doc = "0xb14 - Pretended Networking Payload Low Filter 1"]
    #[inline(always)]
    pub const fn pl1_lo(&self) -> &Pl1Lo {
        &self.pl1_lo
    }
    #[doc = "0xb18 - Pretended Networking Payload High Filter 1"]
    #[inline(always)]
    pub const fn pl1_hi(&self) -> &Pl1Hi {
        &self.pl1_hi
    }
    #[doc = "0xb1c - Pretended Networking ID Filter 2 or ID Mask"]
    #[inline(always)]
    pub const fn flt_id2_idmask(&self) -> &FltId2Idmask {
        &self.flt_id2_idmask
    }
    #[doc = "0xb20 - Pretended Networking Payload Low Filter 2 and Payload Low Mask"]
    #[inline(always)]
    pub const fn pl2_plmask_lo(&self) -> &Pl2PlmaskLo {
        &self.pl2_plmask_lo
    }
    #[doc = "0xb24 - Pretended Networking Payload High Filter 2 and Payload High Mask"]
    #[inline(always)]
    pub const fn pl2_plmask_hi(&self) -> &Pl2PlmaskHi {
        &self.pl2_plmask_hi
    }
    #[doc = "0xb40..0xb80 - Array of registers: WMB_CS, WMB_D03, WMB_D47, WMB_ID"]
    #[inline(always)]
    pub const fn wmb(&self, n: usize) -> &Wmb {
        &self.wmb[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0xb40..0xb80 - Array of registers: WMB_CS, WMB_D03, WMB_D47, WMB_ID"]
    #[inline(always)]
    pub fn wmb_iter(&self) -> impl Iterator<Item = &Wmb> {
        self.wmb.iter()
    }
    #[doc = "0xbf0 - Enhanced CAN Bit Timing Prescalers"]
    #[inline(always)]
    pub const fn eprs(&self) -> &Eprs {
        &self.eprs
    }
    #[doc = "0xbf4 - Enhanced Nominal CAN Bit Timing"]
    #[inline(always)]
    pub const fn encbt(&self) -> &Encbt {
        &self.encbt
    }
    #[doc = "0xbf8 - Enhanced Data Phase CAN Bit Timing"]
    #[inline(always)]
    pub const fn edcbt(&self) -> &Edcbt {
        &self.edcbt
    }
    #[doc = "0xbfc - Enhanced Transceiver Delay Compensation"]
    #[inline(always)]
    pub const fn etdc(&self) -> &Etdc {
        &self.etdc
    }
    #[doc = "0xc00 - CAN FD Control"]
    #[inline(always)]
    pub const fn fdctrl(&self) -> &Fdctrl {
        &self.fdctrl
    }
    #[doc = "0xc04 - CAN FD Bit Timing"]
    #[inline(always)]
    pub const fn fdcbt(&self) -> &Fdcbt {
        &self.fdcbt
    }
    #[doc = "0xc08 - CAN FD CRC"]
    #[inline(always)]
    pub const fn fdcrc(&self) -> &Fdcrc {
        &self.fdcrc
    }
    #[doc = "0xc0c - Enhanced RX FIFO Control"]
    #[inline(always)]
    pub const fn erfcr(&self) -> &Erfcr {
        &self.erfcr
    }
    #[doc = "0xc10 - Enhanced RX FIFO Interrupt Enable"]
    #[inline(always)]
    pub const fn erfier(&self) -> &Erfier {
        &self.erfier
    }
    #[doc = "0xc14 - Enhanced RX FIFO Status"]
    #[inline(always)]
    pub const fn erfsr(&self) -> &Erfsr {
        &self.erfsr
    }
    #[doc = "0x3000..0x3080 - Enhanced RX FIFO Filter Element"]
    #[inline(always)]
    pub const fn erffel(&self, n: usize) -> &Erffel {
        &self.erffel[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x3000..0x3080 - Enhanced RX FIFO Filter Element"]
    #[inline(always)]
    pub fn erffel_iter(&self) -> impl Iterator<Item = &Erffel> {
        self.erffel.iter()
    }
}
#[doc = "MCR (rw) register accessor: Module Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`mcr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mcr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mcr`] module"]
#[doc(alias = "MCR")]
pub type Mcr = crate::Reg<mcr::McrSpec>;
#[doc = "Module Configuration"]
pub mod mcr;
#[doc = "CTRL1 (rw) register accessor: Control 1\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ctrl1`] module"]
#[doc(alias = "CTRL1")]
pub type Ctrl1 = crate::Reg<ctrl1::Ctrl1Spec>;
#[doc = "Control 1"]
pub mod ctrl1;
#[doc = "TIMER (rw) register accessor: Free-Running Timer\n\nYou can [`read`](crate::Reg::read) this register and get [`timer::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`timer::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@timer`] module"]
#[doc(alias = "TIMER")]
pub type Timer = crate::Reg<timer::TimerSpec>;
#[doc = "Free-Running Timer"]
pub mod timer;
#[doc = "RXMGMASK (rw) register accessor: RX Message Buffers Global Mask\n\nYou can [`read`](crate::Reg::read) this register and get [`rxmgmask::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rxmgmask::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@rxmgmask`] module"]
#[doc(alias = "RXMGMASK")]
pub type Rxmgmask = crate::Reg<rxmgmask::RxmgmaskSpec>;
#[doc = "RX Message Buffers Global Mask"]
pub mod rxmgmask;
#[doc = "RX14MASK (rw) register accessor: Receive 14 Mask\n\nYou can [`read`](crate::Reg::read) this register and get [`rx14mask::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rx14mask::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@rx14mask`] module"]
#[doc(alias = "RX14MASK")]
pub type Rx14mask = crate::Reg<rx14mask::Rx14maskSpec>;
#[doc = "Receive 14 Mask"]
pub mod rx14mask;
#[doc = "RX15MASK (rw) register accessor: Receive 15 Mask\n\nYou can [`read`](crate::Reg::read) this register and get [`rx15mask::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rx15mask::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@rx15mask`] module"]
#[doc(alias = "RX15MASK")]
pub type Rx15mask = crate::Reg<rx15mask::Rx15maskSpec>;
#[doc = "Receive 15 Mask"]
pub mod rx15mask;
#[doc = "ECR (rw) register accessor: Error Counter\n\nYou can [`read`](crate::Reg::read) this register and get [`ecr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ecr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ecr`] module"]
#[doc(alias = "ECR")]
pub type Ecr = crate::Reg<ecr::EcrSpec>;
#[doc = "Error Counter"]
pub mod ecr;
#[doc = "ESR1 (rw) register accessor: Error and Status 1\n\nYou can [`read`](crate::Reg::read) this register and get [`esr1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`esr1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@esr1`] module"]
#[doc(alias = "ESR1")]
pub type Esr1 = crate::Reg<esr1::Esr1Spec>;
#[doc = "Error and Status 1"]
pub mod esr1;
#[doc = "IMASK1 (rw) register accessor: Interrupt Masks 1\n\nYou can [`read`](crate::Reg::read) this register and get [`imask1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`imask1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@imask1`] module"]
#[doc(alias = "IMASK1")]
pub type Imask1 = crate::Reg<imask1::Imask1Spec>;
#[doc = "Interrupt Masks 1"]
pub mod imask1;
#[doc = "IFLAG1 (rw) register accessor: Interrupt Flags 1\n\nYou can [`read`](crate::Reg::read) this register and get [`iflag1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`iflag1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@iflag1`] module"]
#[doc(alias = "IFLAG1")]
pub type Iflag1 = crate::Reg<iflag1::Iflag1Spec>;
#[doc = "Interrupt Flags 1"]
pub mod iflag1;
#[doc = "CTRL2 (rw) register accessor: Control 2\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ctrl2`] module"]
#[doc(alias = "CTRL2")]
pub type Ctrl2 = crate::Reg<ctrl2::Ctrl2Spec>;
#[doc = "Control 2"]
pub mod ctrl2;
#[doc = "ESR2 (r) register accessor: Error and Status 2\n\nYou can [`read`](crate::Reg::read) this register and get [`esr2::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@esr2`] module"]
#[doc(alias = "ESR2")]
pub type Esr2 = crate::Reg<esr2::Esr2Spec>;
#[doc = "Error and Status 2"]
pub mod esr2;
#[doc = "CRCR (r) register accessor: Cyclic Redundancy Check\n\nYou can [`read`](crate::Reg::read) this register and get [`crcr::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@crcr`] module"]
#[doc(alias = "CRCR")]
pub type Crcr = crate::Reg<crcr::CrcrSpec>;
#[doc = "Cyclic Redundancy Check"]
pub mod crcr;
#[doc = "RXFGMASK (rw) register accessor: Legacy RX FIFO Global Mask\n\nYou can [`read`](crate::Reg::read) this register and get [`rxfgmask::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rxfgmask::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@rxfgmask`] module"]
#[doc(alias = "RXFGMASK")]
pub type Rxfgmask = crate::Reg<rxfgmask::RxfgmaskSpec>;
#[doc = "Legacy RX FIFO Global Mask"]
pub mod rxfgmask;
#[doc = "RXFIR (r) register accessor: Legacy RX FIFO Information\n\nYou can [`read`](crate::Reg::read) this register and get [`rxfir::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@rxfir`] module"]
#[doc(alias = "RXFIR")]
pub type Rxfir = crate::Reg<rxfir::RxfirSpec>;
#[doc = "Legacy RX FIFO Information"]
pub mod rxfir;
#[doc = "CBT (rw) register accessor: CAN Bit Timing\n\nYou can [`read`](crate::Reg::read) this register and get [`cbt::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cbt::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cbt`] module"]
#[doc(alias = "CBT")]
pub type Cbt = crate::Reg<cbt::CbtSpec>;
#[doc = "CAN Bit Timing"]
pub mod cbt;
#[doc = "MB_GROUP_CS0 (rw) register accessor: Message Buffer 0 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs0`] module"]
#[doc(alias = "MB_GROUP_CS0")]
pub type MbGroupCs0 = crate::Reg<mb_group_cs0::MbGroupCs0Spec>;
#[doc = "Message Buffer 0 CS Register"]
pub mod mb_group_cs0;
#[doc = "MB_16B_GROUP_MB0_16B_CS (rw) register accessor: Message Buffer 0 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb0_16b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb0_16b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb0_16b_cs`] module"]
#[doc(alias = "MB_16B_GROUP_MB0_16B_CS")]
pub type Mb16bGroupMb0_16bCs = crate::Reg<mb_16b_group_mb0_16b_cs::Mb16bGroupMb0_16bCsSpec>;
#[doc = "Message Buffer 0 CS Register"]
pub mod mb_16b_group_mb0_16b_cs;
#[doc = "MB_32B_GROUP_MB0_32B_CS (rw) register accessor: Message Buffer 0 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb0_32b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb0_32b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb0_32b_cs`] module"]
#[doc(alias = "MB_32B_GROUP_MB0_32B_CS")]
pub type Mb32bGroupMb0_32bCs = crate::Reg<mb_32b_group_mb0_32b_cs::Mb32bGroupMb0_32bCsSpec>;
#[doc = "Message Buffer 0 CS Register"]
pub mod mb_32b_group_mb0_32b_cs;
#[doc = "MB_64B_GROUP_MB0_64B_CS (rw) register accessor: Message Buffer 0 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb0_64b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb0_64b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb0_64b_cs`] module"]
#[doc(alias = "MB_64B_GROUP_MB0_64B_CS")]
pub type Mb64bGroupMb0_64bCs = crate::Reg<mb_64b_group_mb0_64b_cs::Mb64bGroupMb0_64bCsSpec>;
#[doc = "Message Buffer 0 CS Register"]
pub mod mb_64b_group_mb0_64b_cs;
#[doc = "MB_8B_GROUP_MB0_8B_CS (rw) register accessor: Message Buffer 0 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb0_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb0_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb0_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB0_8B_CS")]
pub type Mb8bGroupMb0_8bCs = crate::Reg<mb_8b_group_mb0_8b_cs::Mb8bGroupMb0_8bCsSpec>;
#[doc = "Message Buffer 0 CS Register"]
pub mod mb_8b_group_mb0_8b_cs;
#[doc = "MB_GROUP_ID0 (rw) register accessor: Message Buffer 0 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id0`] module"]
#[doc(alias = "MB_GROUP_ID0")]
pub type MbGroupId0 = crate::Reg<mb_group_id0::MbGroupId0Spec>;
#[doc = "Message Buffer 0 ID Register"]
pub mod mb_group_id0;
#[doc = "MB_16B_GROUP_MB0_16B_ID (rw) register accessor: Message Buffer 0 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb0_16b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb0_16b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb0_16b_id`] module"]
#[doc(alias = "MB_16B_GROUP_MB0_16B_ID")]
pub type Mb16bGroupMb0_16bId = crate::Reg<mb_16b_group_mb0_16b_id::Mb16bGroupMb0_16bIdSpec>;
#[doc = "Message Buffer 0 ID Register"]
pub mod mb_16b_group_mb0_16b_id;
#[doc = "MB_32B_GROUP_MB0_32B_ID (rw) register accessor: Message Buffer 0 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb0_32b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb0_32b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb0_32b_id`] module"]
#[doc(alias = "MB_32B_GROUP_MB0_32B_ID")]
pub type Mb32bGroupMb0_32bId = crate::Reg<mb_32b_group_mb0_32b_id::Mb32bGroupMb0_32bIdSpec>;
#[doc = "Message Buffer 0 ID Register"]
pub mod mb_32b_group_mb0_32b_id;
#[doc = "MB_64B_GROUP_MB0_64B_ID (rw) register accessor: Message Buffer 0 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb0_64b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb0_64b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb0_64b_id`] module"]
#[doc(alias = "MB_64B_GROUP_MB0_64B_ID")]
pub type Mb64bGroupMb0_64bId = crate::Reg<mb_64b_group_mb0_64b_id::Mb64bGroupMb0_64bIdSpec>;
#[doc = "Message Buffer 0 ID Register"]
pub mod mb_64b_group_mb0_64b_id;
#[doc = "MB_8B_GROUP_MB0_8B_ID (rw) register accessor: Message Buffer 0 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb0_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb0_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb0_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB0_8B_ID")]
pub type Mb8bGroupMb0_8bId = crate::Reg<mb_8b_group_mb0_8b_id::Mb8bGroupMb0_8bIdSpec>;
#[doc = "Message Buffer 0 ID Register"]
pub mod mb_8b_group_mb0_8b_id;
#[doc = "MB_16B_GROUP_MB0_16B_WORD0 (rw) register accessor: Message Buffer 0 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb0_16b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb0_16b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb0_16b_word0`] module"]
#[doc(alias = "MB_16B_GROUP_MB0_16B_WORD0")]
pub type Mb16bGroupMb0_16bWord0 =
    crate::Reg<mb_16b_group_mb0_16b_word0::Mb16bGroupMb0_16bWord0Spec>;
#[doc = "Message Buffer 0 WORD_16B Register"]
pub mod mb_16b_group_mb0_16b_word0;
#[doc = "MB_32B_GROUP_MB0_32B_WORD0 (rw) register accessor: Message Buffer 0 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb0_32b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb0_32b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb0_32b_word0`] module"]
#[doc(alias = "MB_32B_GROUP_MB0_32B_WORD0")]
pub type Mb32bGroupMb0_32bWord0 =
    crate::Reg<mb_32b_group_mb0_32b_word0::Mb32bGroupMb0_32bWord0Spec>;
#[doc = "Message Buffer 0 WORD_32B Register"]
pub mod mb_32b_group_mb0_32b_word0;
#[doc = "MB_64B_GROUP_MB0_64B_WORD0 (rw) register accessor: Message Buffer 0 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb0_64b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb0_64b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb0_64b_word0`] module"]
#[doc(alias = "MB_64B_GROUP_MB0_64B_WORD0")]
pub type Mb64bGroupMb0_64bWord0 =
    crate::Reg<mb_64b_group_mb0_64b_word0::Mb64bGroupMb0_64bWord0Spec>;
#[doc = "Message Buffer 0 WORD_64B Register"]
pub mod mb_64b_group_mb0_64b_word0;
#[doc = "MB_8B_GROUP_MB0_8B_WORD0 (rw) register accessor: Message Buffer 0 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb0_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb0_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb0_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB0_8B_WORD0")]
pub type Mb8bGroupMb0_8bWord0 = crate::Reg<mb_8b_group_mb0_8b_word0::Mb8bGroupMb0_8bWord0Spec>;
#[doc = "Message Buffer 0 WORD_8B Register"]
pub mod mb_8b_group_mb0_8b_word0;
#[doc = "MB_GROUP_WORD00 (rw) register accessor: Message Buffer 0 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word00::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word00::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word00`] module"]
#[doc(alias = "MB_GROUP_WORD00")]
pub type MbGroupWord00 = crate::Reg<mb_group_word00::MbGroupWord00Spec>;
#[doc = "Message Buffer 0 WORD0 Register"]
pub mod mb_group_word00;
#[doc = "MB_16B_GROUP_MB0_16B_WORD1 (rw) register accessor: Message Buffer 0 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb0_16b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb0_16b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb0_16b_word1`] module"]
#[doc(alias = "MB_16B_GROUP_MB0_16B_WORD1")]
pub type Mb16bGroupMb0_16bWord1 =
    crate::Reg<mb_16b_group_mb0_16b_word1::Mb16bGroupMb0_16bWord1Spec>;
#[doc = "Message Buffer 0 WORD_16B Register"]
pub mod mb_16b_group_mb0_16b_word1;
#[doc = "MB_32B_GROUP_MB0_32B_WORD1 (rw) register accessor: Message Buffer 0 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb0_32b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb0_32b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb0_32b_word1`] module"]
#[doc(alias = "MB_32B_GROUP_MB0_32B_WORD1")]
pub type Mb32bGroupMb0_32bWord1 =
    crate::Reg<mb_32b_group_mb0_32b_word1::Mb32bGroupMb0_32bWord1Spec>;
#[doc = "Message Buffer 0 WORD_32B Register"]
pub mod mb_32b_group_mb0_32b_word1;
#[doc = "MB_64B_GROUP_MB0_64B_WORD1 (rw) register accessor: Message Buffer 0 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb0_64b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb0_64b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb0_64b_word1`] module"]
#[doc(alias = "MB_64B_GROUP_MB0_64B_WORD1")]
pub type Mb64bGroupMb0_64bWord1 =
    crate::Reg<mb_64b_group_mb0_64b_word1::Mb64bGroupMb0_64bWord1Spec>;
#[doc = "Message Buffer 0 WORD_64B Register"]
pub mod mb_64b_group_mb0_64b_word1;
#[doc = "MB_8B_GROUP_MB0_8B_WORD1 (rw) register accessor: Message Buffer 0 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb0_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb0_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb0_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB0_8B_WORD1")]
pub type Mb8bGroupMb0_8bWord1 = crate::Reg<mb_8b_group_mb0_8b_word1::Mb8bGroupMb0_8bWord1Spec>;
#[doc = "Message Buffer 0 WORD_8B Register"]
pub mod mb_8b_group_mb0_8b_word1;
#[doc = "MB_GROUP_WORD10 (rw) register accessor: Message Buffer 0 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word10::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word10::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word10`] module"]
#[doc(alias = "MB_GROUP_WORD10")]
pub type MbGroupWord10 = crate::Reg<mb_group_word10::MbGroupWord10Spec>;
#[doc = "Message Buffer 0 WORD1 Register"]
pub mod mb_group_word10;
#[doc = "MB_GROUP_CS1 (rw) register accessor: Message Buffer 1 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs1`] module"]
#[doc(alias = "MB_GROUP_CS1")]
pub type MbGroupCs1 = crate::Reg<mb_group_cs1::MbGroupCs1Spec>;
#[doc = "Message Buffer 1 CS Register"]
pub mod mb_group_cs1;
#[doc = "MB_16B_GROUP_MB0_16B_WORD2 (rw) register accessor: Message Buffer 0 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb0_16b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb0_16b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb0_16b_word2`] module"]
#[doc(alias = "MB_16B_GROUP_MB0_16B_WORD2")]
pub type Mb16bGroupMb0_16bWord2 =
    crate::Reg<mb_16b_group_mb0_16b_word2::Mb16bGroupMb0_16bWord2Spec>;
#[doc = "Message Buffer 0 WORD_16B Register"]
pub mod mb_16b_group_mb0_16b_word2;
#[doc = "MB_32B_GROUP_MB0_32B_WORD2 (rw) register accessor: Message Buffer 0 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb0_32b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb0_32b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb0_32b_word2`] module"]
#[doc(alias = "MB_32B_GROUP_MB0_32B_WORD2")]
pub type Mb32bGroupMb0_32bWord2 =
    crate::Reg<mb_32b_group_mb0_32b_word2::Mb32bGroupMb0_32bWord2Spec>;
#[doc = "Message Buffer 0 WORD_32B Register"]
pub mod mb_32b_group_mb0_32b_word2;
#[doc = "MB_64B_GROUP_MB0_64B_WORD2 (rw) register accessor: Message Buffer 0 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb0_64b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb0_64b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb0_64b_word2`] module"]
#[doc(alias = "MB_64B_GROUP_MB0_64B_WORD2")]
pub type Mb64bGroupMb0_64bWord2 =
    crate::Reg<mb_64b_group_mb0_64b_word2::Mb64bGroupMb0_64bWord2Spec>;
#[doc = "Message Buffer 0 WORD_64B Register"]
pub mod mb_64b_group_mb0_64b_word2;
#[doc = "MB_8B_GROUP_MB1_8B_CS (rw) register accessor: Message Buffer 1 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb1_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb1_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb1_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB1_8B_CS")]
pub type Mb8bGroupMb1_8bCs = crate::Reg<mb_8b_group_mb1_8b_cs::Mb8bGroupMb1_8bCsSpec>;
#[doc = "Message Buffer 1 CS Register"]
pub mod mb_8b_group_mb1_8b_cs;
#[doc = "MB_GROUP_ID1 (rw) register accessor: Message Buffer 1 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id1`] module"]
#[doc(alias = "MB_GROUP_ID1")]
pub type MbGroupId1 = crate::Reg<mb_group_id1::MbGroupId1Spec>;
#[doc = "Message Buffer 1 ID Register"]
pub mod mb_group_id1;
#[doc = "MB_16B_GROUP_MB0_16B_WORD3 (rw) register accessor: Message Buffer 0 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb0_16b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb0_16b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb0_16b_word3`] module"]
#[doc(alias = "MB_16B_GROUP_MB0_16B_WORD3")]
pub type Mb16bGroupMb0_16bWord3 =
    crate::Reg<mb_16b_group_mb0_16b_word3::Mb16bGroupMb0_16bWord3Spec>;
#[doc = "Message Buffer 0 WORD_16B Register"]
pub mod mb_16b_group_mb0_16b_word3;
#[doc = "MB_32B_GROUP_MB0_32B_WORD3 (rw) register accessor: Message Buffer 0 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb0_32b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb0_32b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb0_32b_word3`] module"]
#[doc(alias = "MB_32B_GROUP_MB0_32B_WORD3")]
pub type Mb32bGroupMb0_32bWord3 =
    crate::Reg<mb_32b_group_mb0_32b_word3::Mb32bGroupMb0_32bWord3Spec>;
#[doc = "Message Buffer 0 WORD_32B Register"]
pub mod mb_32b_group_mb0_32b_word3;
#[doc = "MB_64B_GROUP_MB0_64B_WORD3 (rw) register accessor: Message Buffer 0 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb0_64b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb0_64b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb0_64b_word3`] module"]
#[doc(alias = "MB_64B_GROUP_MB0_64B_WORD3")]
pub type Mb64bGroupMb0_64bWord3 =
    crate::Reg<mb_64b_group_mb0_64b_word3::Mb64bGroupMb0_64bWord3Spec>;
#[doc = "Message Buffer 0 WORD_64B Register"]
pub mod mb_64b_group_mb0_64b_word3;
#[doc = "MB_8B_GROUP_MB1_8B_ID (rw) register accessor: Message Buffer 1 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb1_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb1_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb1_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB1_8B_ID")]
pub type Mb8bGroupMb1_8bId = crate::Reg<mb_8b_group_mb1_8b_id::Mb8bGroupMb1_8bIdSpec>;
#[doc = "Message Buffer 1 ID Register"]
pub mod mb_8b_group_mb1_8b_id;
#[doc = "MB_32B_GROUP_MB0_32B_WORD4 (rw) register accessor: Message Buffer 0 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb0_32b_word4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb0_32b_word4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb0_32b_word4`] module"]
#[doc(alias = "MB_32B_GROUP_MB0_32B_WORD4")]
pub type Mb32bGroupMb0_32bWord4 =
    crate::Reg<mb_32b_group_mb0_32b_word4::Mb32bGroupMb0_32bWord4Spec>;
#[doc = "Message Buffer 0 WORD_32B Register"]
pub mod mb_32b_group_mb0_32b_word4;
#[doc = "MB_64B_GROUP_MB0_64B_WORD4 (rw) register accessor: Message Buffer 0 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb0_64b_word4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb0_64b_word4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb0_64b_word4`] module"]
#[doc(alias = "MB_64B_GROUP_MB0_64B_WORD4")]
pub type Mb64bGroupMb0_64bWord4 =
    crate::Reg<mb_64b_group_mb0_64b_word4::Mb64bGroupMb0_64bWord4Spec>;
#[doc = "Message Buffer 0 WORD_64B Register"]
pub mod mb_64b_group_mb0_64b_word4;
#[doc = "MB_16B_GROUP_MB1_16B_CS (rw) register accessor: Message Buffer 1 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb1_16b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb1_16b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb1_16b_cs`] module"]
#[doc(alias = "MB_16B_GROUP_MB1_16B_CS")]
pub type Mb16bGroupMb1_16bCs = crate::Reg<mb_16b_group_mb1_16b_cs::Mb16bGroupMb1_16bCsSpec>;
#[doc = "Message Buffer 1 CS Register"]
pub mod mb_16b_group_mb1_16b_cs;
#[doc = "MB_8B_GROUP_MB1_8B_WORD0 (rw) register accessor: Message Buffer 1 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb1_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb1_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb1_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB1_8B_WORD0")]
pub type Mb8bGroupMb1_8bWord0 = crate::Reg<mb_8b_group_mb1_8b_word0::Mb8bGroupMb1_8bWord0Spec>;
#[doc = "Message Buffer 1 WORD_8B Register"]
pub mod mb_8b_group_mb1_8b_word0;
#[doc = "MB_GROUP_WORD01 (rw) register accessor: Message Buffer 1 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word01::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word01::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word01`] module"]
#[doc(alias = "MB_GROUP_WORD01")]
pub type MbGroupWord01 = crate::Reg<mb_group_word01::MbGroupWord01Spec>;
#[doc = "Message Buffer 1 WORD0 Register"]
pub mod mb_group_word01;
#[doc = "MB_32B_GROUP_MB0_32B_WORD5 (rw) register accessor: Message Buffer 0 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb0_32b_word5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb0_32b_word5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb0_32b_word5`] module"]
#[doc(alias = "MB_32B_GROUP_MB0_32B_WORD5")]
pub type Mb32bGroupMb0_32bWord5 =
    crate::Reg<mb_32b_group_mb0_32b_word5::Mb32bGroupMb0_32bWord5Spec>;
#[doc = "Message Buffer 0 WORD_32B Register"]
pub mod mb_32b_group_mb0_32b_word5;
#[doc = "MB_64B_GROUP_MB0_64B_WORD5 (rw) register accessor: Message Buffer 0 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb0_64b_word5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb0_64b_word5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb0_64b_word5`] module"]
#[doc(alias = "MB_64B_GROUP_MB0_64B_WORD5")]
pub type Mb64bGroupMb0_64bWord5 =
    crate::Reg<mb_64b_group_mb0_64b_word5::Mb64bGroupMb0_64bWord5Spec>;
#[doc = "Message Buffer 0 WORD_64B Register"]
pub mod mb_64b_group_mb0_64b_word5;
#[doc = "MB_16B_GROUP_MB1_16B_ID (rw) register accessor: Message Buffer 1 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb1_16b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb1_16b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb1_16b_id`] module"]
#[doc(alias = "MB_16B_GROUP_MB1_16B_ID")]
pub type Mb16bGroupMb1_16bId = crate::Reg<mb_16b_group_mb1_16b_id::Mb16bGroupMb1_16bIdSpec>;
#[doc = "Message Buffer 1 ID Register"]
pub mod mb_16b_group_mb1_16b_id;
#[doc = "MB_8B_GROUP_MB1_8B_WORD1 (rw) register accessor: Message Buffer 1 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb1_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb1_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb1_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB1_8B_WORD1")]
pub type Mb8bGroupMb1_8bWord1 = crate::Reg<mb_8b_group_mb1_8b_word1::Mb8bGroupMb1_8bWord1Spec>;
#[doc = "Message Buffer 1 WORD_8B Register"]
pub mod mb_8b_group_mb1_8b_word1;
#[doc = "MB_GROUP_WORD11 (rw) register accessor: Message Buffer 1 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word11::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word11::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word11`] module"]
#[doc(alias = "MB_GROUP_WORD11")]
pub type MbGroupWord11 = crate::Reg<mb_group_word11::MbGroupWord11Spec>;
#[doc = "Message Buffer 1 WORD1 Register"]
pub mod mb_group_word11;
#[doc = "MB_GROUP_CS2 (rw) register accessor: Message Buffer 2 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs2`] module"]
#[doc(alias = "MB_GROUP_CS2")]
pub type MbGroupCs2 = crate::Reg<mb_group_cs2::MbGroupCs2Spec>;
#[doc = "Message Buffer 2 CS Register"]
pub mod mb_group_cs2;
#[doc = "MB_32B_GROUP_MB0_32B_WORD6 (rw) register accessor: Message Buffer 0 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb0_32b_word6::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb0_32b_word6::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb0_32b_word6`] module"]
#[doc(alias = "MB_32B_GROUP_MB0_32B_WORD6")]
pub type Mb32bGroupMb0_32bWord6 =
    crate::Reg<mb_32b_group_mb0_32b_word6::Mb32bGroupMb0_32bWord6Spec>;
#[doc = "Message Buffer 0 WORD_32B Register"]
pub mod mb_32b_group_mb0_32b_word6;
#[doc = "MB_64B_GROUP_MB0_64B_WORD6 (rw) register accessor: Message Buffer 0 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb0_64b_word6::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb0_64b_word6::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb0_64b_word6`] module"]
#[doc(alias = "MB_64B_GROUP_MB0_64B_WORD6")]
pub type Mb64bGroupMb0_64bWord6 =
    crate::Reg<mb_64b_group_mb0_64b_word6::Mb64bGroupMb0_64bWord6Spec>;
#[doc = "Message Buffer 0 WORD_64B Register"]
pub mod mb_64b_group_mb0_64b_word6;
#[doc = "MB_16B_GROUP_MB1_16B_WORD0 (rw) register accessor: Message Buffer 1 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb1_16b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb1_16b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb1_16b_word0`] module"]
#[doc(alias = "MB_16B_GROUP_MB1_16B_WORD0")]
pub type Mb16bGroupMb1_16bWord0 =
    crate::Reg<mb_16b_group_mb1_16b_word0::Mb16bGroupMb1_16bWord0Spec>;
#[doc = "Message Buffer 1 WORD_16B Register"]
pub mod mb_16b_group_mb1_16b_word0;
#[doc = "MB_8B_GROUP_MB2_8B_CS (rw) register accessor: Message Buffer 2 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb2_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb2_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb2_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB2_8B_CS")]
pub type Mb8bGroupMb2_8bCs = crate::Reg<mb_8b_group_mb2_8b_cs::Mb8bGroupMb2_8bCsSpec>;
#[doc = "Message Buffer 2 CS Register"]
pub mod mb_8b_group_mb2_8b_cs;
#[doc = "MB_GROUP_ID2 (rw) register accessor: Message Buffer 2 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id2`] module"]
#[doc(alias = "MB_GROUP_ID2")]
pub type MbGroupId2 = crate::Reg<mb_group_id2::MbGroupId2Spec>;
#[doc = "Message Buffer 2 ID Register"]
pub mod mb_group_id2;
#[doc = "MB_32B_GROUP_MB0_32B_WORD7 (rw) register accessor: Message Buffer 0 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb0_32b_word7::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb0_32b_word7::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb0_32b_word7`] module"]
#[doc(alias = "MB_32B_GROUP_MB0_32B_WORD7")]
pub type Mb32bGroupMb0_32bWord7 =
    crate::Reg<mb_32b_group_mb0_32b_word7::Mb32bGroupMb0_32bWord7Spec>;
#[doc = "Message Buffer 0 WORD_32B Register"]
pub mod mb_32b_group_mb0_32b_word7;
#[doc = "MB_64B_GROUP_MB0_64B_WORD7 (rw) register accessor: Message Buffer 0 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb0_64b_word7::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb0_64b_word7::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb0_64b_word7`] module"]
#[doc(alias = "MB_64B_GROUP_MB0_64B_WORD7")]
pub type Mb64bGroupMb0_64bWord7 =
    crate::Reg<mb_64b_group_mb0_64b_word7::Mb64bGroupMb0_64bWord7Spec>;
#[doc = "Message Buffer 0 WORD_64B Register"]
pub mod mb_64b_group_mb0_64b_word7;
#[doc = "MB_16B_GROUP_MB1_16B_WORD1 (rw) register accessor: Message Buffer 1 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb1_16b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb1_16b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb1_16b_word1`] module"]
#[doc(alias = "MB_16B_GROUP_MB1_16B_WORD1")]
pub type Mb16bGroupMb1_16bWord1 =
    crate::Reg<mb_16b_group_mb1_16b_word1::Mb16bGroupMb1_16bWord1Spec>;
#[doc = "Message Buffer 1 WORD_16B Register"]
pub mod mb_16b_group_mb1_16b_word1;
#[doc = "MB_8B_GROUP_MB2_8B_ID (rw) register accessor: Message Buffer 2 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb2_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb2_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb2_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB2_8B_ID")]
pub type Mb8bGroupMb2_8bId = crate::Reg<mb_8b_group_mb2_8b_id::Mb8bGroupMb2_8bIdSpec>;
#[doc = "Message Buffer 2 ID Register"]
pub mod mb_8b_group_mb2_8b_id;
#[doc = "MB_64B_GROUP_MB0_64B_WORD8 (rw) register accessor: Message Buffer 0 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb0_64b_word8::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb0_64b_word8::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb0_64b_word8`] module"]
#[doc(alias = "MB_64B_GROUP_MB0_64B_WORD8")]
pub type Mb64bGroupMb0_64bWord8 =
    crate::Reg<mb_64b_group_mb0_64b_word8::Mb64bGroupMb0_64bWord8Spec>;
#[doc = "Message Buffer 0 WORD_64B Register"]
pub mod mb_64b_group_mb0_64b_word8;
#[doc = "MB_16B_GROUP_MB1_16B_WORD2 (rw) register accessor: Message Buffer 1 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb1_16b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb1_16b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb1_16b_word2`] module"]
#[doc(alias = "MB_16B_GROUP_MB1_16B_WORD2")]
pub type Mb16bGroupMb1_16bWord2 =
    crate::Reg<mb_16b_group_mb1_16b_word2::Mb16bGroupMb1_16bWord2Spec>;
#[doc = "Message Buffer 1 WORD_16B Register"]
pub mod mb_16b_group_mb1_16b_word2;
#[doc = "MB_32B_GROUP_MB1_32B_CS (rw) register accessor: Message Buffer 1 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb1_32b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb1_32b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb1_32b_cs`] module"]
#[doc(alias = "MB_32B_GROUP_MB1_32B_CS")]
pub type Mb32bGroupMb1_32bCs = crate::Reg<mb_32b_group_mb1_32b_cs::Mb32bGroupMb1_32bCsSpec>;
#[doc = "Message Buffer 1 CS Register"]
pub mod mb_32b_group_mb1_32b_cs;
#[doc = "MB_8B_GROUP_MB2_8B_WORD0 (rw) register accessor: Message Buffer 2 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb2_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb2_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb2_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB2_8B_WORD0")]
pub type Mb8bGroupMb2_8bWord0 = crate::Reg<mb_8b_group_mb2_8b_word0::Mb8bGroupMb2_8bWord0Spec>;
#[doc = "Message Buffer 2 WORD_8B Register"]
pub mod mb_8b_group_mb2_8b_word0;
#[doc = "MB_GROUP_WORD02 (rw) register accessor: Message Buffer 2 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word02::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word02::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word02`] module"]
#[doc(alias = "MB_GROUP_WORD02")]
pub type MbGroupWord02 = crate::Reg<mb_group_word02::MbGroupWord02Spec>;
#[doc = "Message Buffer 2 WORD0 Register"]
pub mod mb_group_word02;
#[doc = "MB_64B_GROUP_MB0_64B_WORD9 (rw) register accessor: Message Buffer 0 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb0_64b_word9::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb0_64b_word9::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb0_64b_word9`] module"]
#[doc(alias = "MB_64B_GROUP_MB0_64B_WORD9")]
pub type Mb64bGroupMb0_64bWord9 =
    crate::Reg<mb_64b_group_mb0_64b_word9::Mb64bGroupMb0_64bWord9Spec>;
#[doc = "Message Buffer 0 WORD_64B Register"]
pub mod mb_64b_group_mb0_64b_word9;
#[doc = "MB_16B_GROUP_MB1_16B_WORD3 (rw) register accessor: Message Buffer 1 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb1_16b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb1_16b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb1_16b_word3`] module"]
#[doc(alias = "MB_16B_GROUP_MB1_16B_WORD3")]
pub type Mb16bGroupMb1_16bWord3 =
    crate::Reg<mb_16b_group_mb1_16b_word3::Mb16bGroupMb1_16bWord3Spec>;
#[doc = "Message Buffer 1 WORD_16B Register"]
pub mod mb_16b_group_mb1_16b_word3;
#[doc = "MB_32B_GROUP_MB1_32B_ID (rw) register accessor: Message Buffer 1 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb1_32b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb1_32b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb1_32b_id`] module"]
#[doc(alias = "MB_32B_GROUP_MB1_32B_ID")]
pub type Mb32bGroupMb1_32bId = crate::Reg<mb_32b_group_mb1_32b_id::Mb32bGroupMb1_32bIdSpec>;
#[doc = "Message Buffer 1 ID Register"]
pub mod mb_32b_group_mb1_32b_id;
#[doc = "MB_8B_GROUP_MB2_8B_WORD1 (rw) register accessor: Message Buffer 2 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb2_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb2_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb2_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB2_8B_WORD1")]
pub type Mb8bGroupMb2_8bWord1 = crate::Reg<mb_8b_group_mb2_8b_word1::Mb8bGroupMb2_8bWord1Spec>;
#[doc = "Message Buffer 2 WORD_8B Register"]
pub mod mb_8b_group_mb2_8b_word1;
#[doc = "MB_GROUP_WORD12 (rw) register accessor: Message Buffer 2 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word12::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word12::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word12`] module"]
#[doc(alias = "MB_GROUP_WORD12")]
pub type MbGroupWord12 = crate::Reg<mb_group_word12::MbGroupWord12Spec>;
#[doc = "Message Buffer 2 WORD1 Register"]
pub mod mb_group_word12;
#[doc = "MB_GROUP_CS3 (rw) register accessor: Message Buffer 3 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs3`] module"]
#[doc(alias = "MB_GROUP_CS3")]
pub type MbGroupCs3 = crate::Reg<mb_group_cs3::MbGroupCs3Spec>;
#[doc = "Message Buffer 3 CS Register"]
pub mod mb_group_cs3;
#[doc = "MB_64B_GROUP_MB0_64B_WORD10 (rw) register accessor: Message Buffer 0 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb0_64b_word10::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb0_64b_word10::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb0_64b_word10`] module"]
#[doc(alias = "MB_64B_GROUP_MB0_64B_WORD10")]
pub type Mb64bGroupMb0_64bWord10 =
    crate::Reg<mb_64b_group_mb0_64b_word10::Mb64bGroupMb0_64bWord10Spec>;
#[doc = "Message Buffer 0 WORD_64B Register"]
pub mod mb_64b_group_mb0_64b_word10;
#[doc = "MB_32B_GROUP_MB1_32B_WORD0 (rw) register accessor: Message Buffer 1 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb1_32b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb1_32b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb1_32b_word0`] module"]
#[doc(alias = "MB_32B_GROUP_MB1_32B_WORD0")]
pub type Mb32bGroupMb1_32bWord0 =
    crate::Reg<mb_32b_group_mb1_32b_word0::Mb32bGroupMb1_32bWord0Spec>;
#[doc = "Message Buffer 1 WORD_32B Register"]
pub mod mb_32b_group_mb1_32b_word0;
#[doc = "MB_16B_GROUP_MB2_16B_CS (rw) register accessor: Message Buffer 2 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb2_16b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb2_16b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb2_16b_cs`] module"]
#[doc(alias = "MB_16B_GROUP_MB2_16B_CS")]
pub type Mb16bGroupMb2_16bCs = crate::Reg<mb_16b_group_mb2_16b_cs::Mb16bGroupMb2_16bCsSpec>;
#[doc = "Message Buffer 2 CS Register"]
pub mod mb_16b_group_mb2_16b_cs;
#[doc = "MB_8B_GROUP_MB3_8B_CS (rw) register accessor: Message Buffer 3 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb3_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb3_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb3_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB3_8B_CS")]
pub type Mb8bGroupMb3_8bCs = crate::Reg<mb_8b_group_mb3_8b_cs::Mb8bGroupMb3_8bCsSpec>;
#[doc = "Message Buffer 3 CS Register"]
pub mod mb_8b_group_mb3_8b_cs;
#[doc = "MB_GROUP_ID3 (rw) register accessor: Message Buffer 3 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id3`] module"]
#[doc(alias = "MB_GROUP_ID3")]
pub type MbGroupId3 = crate::Reg<mb_group_id3::MbGroupId3Spec>;
#[doc = "Message Buffer 3 ID Register"]
pub mod mb_group_id3;
#[doc = "MB_64B_GROUP_MB0_64B_WORD11 (rw) register accessor: Message Buffer 0 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb0_64b_word11::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb0_64b_word11::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb0_64b_word11`] module"]
#[doc(alias = "MB_64B_GROUP_MB0_64B_WORD11")]
pub type Mb64bGroupMb0_64bWord11 =
    crate::Reg<mb_64b_group_mb0_64b_word11::Mb64bGroupMb0_64bWord11Spec>;
#[doc = "Message Buffer 0 WORD_64B Register"]
pub mod mb_64b_group_mb0_64b_word11;
#[doc = "MB_32B_GROUP_MB1_32B_WORD1 (rw) register accessor: Message Buffer 1 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb1_32b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb1_32b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb1_32b_word1`] module"]
#[doc(alias = "MB_32B_GROUP_MB1_32B_WORD1")]
pub type Mb32bGroupMb1_32bWord1 =
    crate::Reg<mb_32b_group_mb1_32b_word1::Mb32bGroupMb1_32bWord1Spec>;
#[doc = "Message Buffer 1 WORD_32B Register"]
pub mod mb_32b_group_mb1_32b_word1;
#[doc = "MB_16B_GROUP_MB2_16B_ID (rw) register accessor: Message Buffer 2 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb2_16b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb2_16b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb2_16b_id`] module"]
#[doc(alias = "MB_16B_GROUP_MB2_16B_ID")]
pub type Mb16bGroupMb2_16bId = crate::Reg<mb_16b_group_mb2_16b_id::Mb16bGroupMb2_16bIdSpec>;
#[doc = "Message Buffer 2 ID Register"]
pub mod mb_16b_group_mb2_16b_id;
#[doc = "MB_8B_GROUP_MB3_8B_ID (rw) register accessor: Message Buffer 3 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb3_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb3_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb3_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB3_8B_ID")]
pub type Mb8bGroupMb3_8bId = crate::Reg<mb_8b_group_mb3_8b_id::Mb8bGroupMb3_8bIdSpec>;
#[doc = "Message Buffer 3 ID Register"]
pub mod mb_8b_group_mb3_8b_id;
#[doc = "MB_64B_GROUP_MB0_64B_WORD12 (rw) register accessor: Message Buffer 0 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb0_64b_word12::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb0_64b_word12::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb0_64b_word12`] module"]
#[doc(alias = "MB_64B_GROUP_MB0_64B_WORD12")]
pub type Mb64bGroupMb0_64bWord12 =
    crate::Reg<mb_64b_group_mb0_64b_word12::Mb64bGroupMb0_64bWord12Spec>;
#[doc = "Message Buffer 0 WORD_64B Register"]
pub mod mb_64b_group_mb0_64b_word12;
#[doc = "MB_32B_GROUP_MB1_32B_WORD2 (rw) register accessor: Message Buffer 1 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb1_32b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb1_32b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb1_32b_word2`] module"]
#[doc(alias = "MB_32B_GROUP_MB1_32B_WORD2")]
pub type Mb32bGroupMb1_32bWord2 =
    crate::Reg<mb_32b_group_mb1_32b_word2::Mb32bGroupMb1_32bWord2Spec>;
#[doc = "Message Buffer 1 WORD_32B Register"]
pub mod mb_32b_group_mb1_32b_word2;
#[doc = "MB_16B_GROUP_MB2_16B_WORD0 (rw) register accessor: Message Buffer 2 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb2_16b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb2_16b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb2_16b_word0`] module"]
#[doc(alias = "MB_16B_GROUP_MB2_16B_WORD0")]
pub type Mb16bGroupMb2_16bWord0 =
    crate::Reg<mb_16b_group_mb2_16b_word0::Mb16bGroupMb2_16bWord0Spec>;
#[doc = "Message Buffer 2 WORD_16B Register"]
pub mod mb_16b_group_mb2_16b_word0;
#[doc = "MB_8B_GROUP_MB3_8B_WORD0 (rw) register accessor: Message Buffer 3 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb3_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb3_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb3_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB3_8B_WORD0")]
pub type Mb8bGroupMb3_8bWord0 = crate::Reg<mb_8b_group_mb3_8b_word0::Mb8bGroupMb3_8bWord0Spec>;
#[doc = "Message Buffer 3 WORD_8B Register"]
pub mod mb_8b_group_mb3_8b_word0;
#[doc = "MB_GROUP_WORD03 (rw) register accessor: Message Buffer 3 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word03::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word03::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word03`] module"]
#[doc(alias = "MB_GROUP_WORD03")]
pub type MbGroupWord03 = crate::Reg<mb_group_word03::MbGroupWord03Spec>;
#[doc = "Message Buffer 3 WORD0 Register"]
pub mod mb_group_word03;
#[doc = "MB_64B_GROUP_MB0_64B_WORD13 (rw) register accessor: Message Buffer 0 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb0_64b_word13::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb0_64b_word13::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb0_64b_word13`] module"]
#[doc(alias = "MB_64B_GROUP_MB0_64B_WORD13")]
pub type Mb64bGroupMb0_64bWord13 =
    crate::Reg<mb_64b_group_mb0_64b_word13::Mb64bGroupMb0_64bWord13Spec>;
#[doc = "Message Buffer 0 WORD_64B Register"]
pub mod mb_64b_group_mb0_64b_word13;
#[doc = "MB_32B_GROUP_MB1_32B_WORD3 (rw) register accessor: Message Buffer 1 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb1_32b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb1_32b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb1_32b_word3`] module"]
#[doc(alias = "MB_32B_GROUP_MB1_32B_WORD3")]
pub type Mb32bGroupMb1_32bWord3 =
    crate::Reg<mb_32b_group_mb1_32b_word3::Mb32bGroupMb1_32bWord3Spec>;
#[doc = "Message Buffer 1 WORD_32B Register"]
pub mod mb_32b_group_mb1_32b_word3;
#[doc = "MB_16B_GROUP_MB2_16B_WORD1 (rw) register accessor: Message Buffer 2 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb2_16b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb2_16b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb2_16b_word1`] module"]
#[doc(alias = "MB_16B_GROUP_MB2_16B_WORD1")]
pub type Mb16bGroupMb2_16bWord1 =
    crate::Reg<mb_16b_group_mb2_16b_word1::Mb16bGroupMb2_16bWord1Spec>;
#[doc = "Message Buffer 2 WORD_16B Register"]
pub mod mb_16b_group_mb2_16b_word1;
#[doc = "MB_8B_GROUP_MB3_8B_WORD1 (rw) register accessor: Message Buffer 3 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb3_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb3_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb3_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB3_8B_WORD1")]
pub type Mb8bGroupMb3_8bWord1 = crate::Reg<mb_8b_group_mb3_8b_word1::Mb8bGroupMb3_8bWord1Spec>;
#[doc = "Message Buffer 3 WORD_8B Register"]
pub mod mb_8b_group_mb3_8b_word1;
#[doc = "MB_GROUP_WORD13 (rw) register accessor: Message Buffer 3 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word13::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word13::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word13`] module"]
#[doc(alias = "MB_GROUP_WORD13")]
pub type MbGroupWord13 = crate::Reg<mb_group_word13::MbGroupWord13Spec>;
#[doc = "Message Buffer 3 WORD1 Register"]
pub mod mb_group_word13;
#[doc = "MB_GROUP_CS4 (rw) register accessor: Message Buffer 4 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs4`] module"]
#[doc(alias = "MB_GROUP_CS4")]
pub type MbGroupCs4 = crate::Reg<mb_group_cs4::MbGroupCs4Spec>;
#[doc = "Message Buffer 4 CS Register"]
pub mod mb_group_cs4;
#[doc = "MB_64B_GROUP_MB0_64B_WORD14 (rw) register accessor: Message Buffer 0 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb0_64b_word14::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb0_64b_word14::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb0_64b_word14`] module"]
#[doc(alias = "MB_64B_GROUP_MB0_64B_WORD14")]
pub type Mb64bGroupMb0_64bWord14 =
    crate::Reg<mb_64b_group_mb0_64b_word14::Mb64bGroupMb0_64bWord14Spec>;
#[doc = "Message Buffer 0 WORD_64B Register"]
pub mod mb_64b_group_mb0_64b_word14;
#[doc = "MB_32B_GROUP_MB1_32B_WORD4 (rw) register accessor: Message Buffer 1 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb1_32b_word4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb1_32b_word4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb1_32b_word4`] module"]
#[doc(alias = "MB_32B_GROUP_MB1_32B_WORD4")]
pub type Mb32bGroupMb1_32bWord4 =
    crate::Reg<mb_32b_group_mb1_32b_word4::Mb32bGroupMb1_32bWord4Spec>;
#[doc = "Message Buffer 1 WORD_32B Register"]
pub mod mb_32b_group_mb1_32b_word4;
#[doc = "MB_16B_GROUP_MB2_16B_WORD2 (rw) register accessor: Message Buffer 2 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb2_16b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb2_16b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb2_16b_word2`] module"]
#[doc(alias = "MB_16B_GROUP_MB2_16B_WORD2")]
pub type Mb16bGroupMb2_16bWord2 =
    crate::Reg<mb_16b_group_mb2_16b_word2::Mb16bGroupMb2_16bWord2Spec>;
#[doc = "Message Buffer 2 WORD_16B Register"]
pub mod mb_16b_group_mb2_16b_word2;
#[doc = "MB_8B_GROUP_MB4_8B_CS (rw) register accessor: Message Buffer 4 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb4_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb4_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb4_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB4_8B_CS")]
pub type Mb8bGroupMb4_8bCs = crate::Reg<mb_8b_group_mb4_8b_cs::Mb8bGroupMb4_8bCsSpec>;
#[doc = "Message Buffer 4 CS Register"]
pub mod mb_8b_group_mb4_8b_cs;
#[doc = "MB_GROUP_ID4 (rw) register accessor: Message Buffer 4 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id4`] module"]
#[doc(alias = "MB_GROUP_ID4")]
pub type MbGroupId4 = crate::Reg<mb_group_id4::MbGroupId4Spec>;
#[doc = "Message Buffer 4 ID Register"]
pub mod mb_group_id4;
#[doc = "MB_64B_GROUP_MB0_64B_WORD15 (rw) register accessor: Message Buffer 0 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb0_64b_word15::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb0_64b_word15::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb0_64b_word15`] module"]
#[doc(alias = "MB_64B_GROUP_MB0_64B_WORD15")]
pub type Mb64bGroupMb0_64bWord15 =
    crate::Reg<mb_64b_group_mb0_64b_word15::Mb64bGroupMb0_64bWord15Spec>;
#[doc = "Message Buffer 0 WORD_64B Register"]
pub mod mb_64b_group_mb0_64b_word15;
#[doc = "MB_32B_GROUP_MB1_32B_WORD5 (rw) register accessor: Message Buffer 1 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb1_32b_word5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb1_32b_word5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb1_32b_word5`] module"]
#[doc(alias = "MB_32B_GROUP_MB1_32B_WORD5")]
pub type Mb32bGroupMb1_32bWord5 =
    crate::Reg<mb_32b_group_mb1_32b_word5::Mb32bGroupMb1_32bWord5Spec>;
#[doc = "Message Buffer 1 WORD_32B Register"]
pub mod mb_32b_group_mb1_32b_word5;
#[doc = "MB_16B_GROUP_MB2_16B_WORD3 (rw) register accessor: Message Buffer 2 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb2_16b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb2_16b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb2_16b_word3`] module"]
#[doc(alias = "MB_16B_GROUP_MB2_16B_WORD3")]
pub type Mb16bGroupMb2_16bWord3 =
    crate::Reg<mb_16b_group_mb2_16b_word3::Mb16bGroupMb2_16bWord3Spec>;
#[doc = "Message Buffer 2 WORD_16B Register"]
pub mod mb_16b_group_mb2_16b_word3;
#[doc = "MB_8B_GROUP_MB4_8B_ID (rw) register accessor: Message Buffer 4 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb4_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb4_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb4_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB4_8B_ID")]
pub type Mb8bGroupMb4_8bId = crate::Reg<mb_8b_group_mb4_8b_id::Mb8bGroupMb4_8bIdSpec>;
#[doc = "Message Buffer 4 ID Register"]
pub mod mb_8b_group_mb4_8b_id;
#[doc = "MB_32B_GROUP_MB1_32B_WORD6 (rw) register accessor: Message Buffer 1 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb1_32b_word6::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb1_32b_word6::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb1_32b_word6`] module"]
#[doc(alias = "MB_32B_GROUP_MB1_32B_WORD6")]
pub type Mb32bGroupMb1_32bWord6 =
    crate::Reg<mb_32b_group_mb1_32b_word6::Mb32bGroupMb1_32bWord6Spec>;
#[doc = "Message Buffer 1 WORD_32B Register"]
pub mod mb_32b_group_mb1_32b_word6;
#[doc = "MB_64B_GROUP_MB1_64B_CS (rw) register accessor: Message Buffer 1 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb1_64b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb1_64b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb1_64b_cs`] module"]
#[doc(alias = "MB_64B_GROUP_MB1_64B_CS")]
pub type Mb64bGroupMb1_64bCs = crate::Reg<mb_64b_group_mb1_64b_cs::Mb64bGroupMb1_64bCsSpec>;
#[doc = "Message Buffer 1 CS Register"]
pub mod mb_64b_group_mb1_64b_cs;
#[doc = "MB_16B_GROUP_MB3_16B_CS (rw) register accessor: Message Buffer 3 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb3_16b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb3_16b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb3_16b_cs`] module"]
#[doc(alias = "MB_16B_GROUP_MB3_16B_CS")]
pub type Mb16bGroupMb3_16bCs = crate::Reg<mb_16b_group_mb3_16b_cs::Mb16bGroupMb3_16bCsSpec>;
#[doc = "Message Buffer 3 CS Register"]
pub mod mb_16b_group_mb3_16b_cs;
#[doc = "MB_8B_GROUP_MB4_8B_WORD0 (rw) register accessor: Message Buffer 4 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb4_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb4_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb4_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB4_8B_WORD0")]
pub type Mb8bGroupMb4_8bWord0 = crate::Reg<mb_8b_group_mb4_8b_word0::Mb8bGroupMb4_8bWord0Spec>;
#[doc = "Message Buffer 4 WORD_8B Register"]
pub mod mb_8b_group_mb4_8b_word0;
#[doc = "MB_GROUP_WORD04 (rw) register accessor: Message Buffer 4 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word04::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word04::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word04`] module"]
#[doc(alias = "MB_GROUP_WORD04")]
pub type MbGroupWord04 = crate::Reg<mb_group_word04::MbGroupWord04Spec>;
#[doc = "Message Buffer 4 WORD0 Register"]
pub mod mb_group_word04;
#[doc = "MB_32B_GROUP_MB1_32B_WORD7 (rw) register accessor: Message Buffer 1 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb1_32b_word7::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb1_32b_word7::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb1_32b_word7`] module"]
#[doc(alias = "MB_32B_GROUP_MB1_32B_WORD7")]
pub type Mb32bGroupMb1_32bWord7 =
    crate::Reg<mb_32b_group_mb1_32b_word7::Mb32bGroupMb1_32bWord7Spec>;
#[doc = "Message Buffer 1 WORD_32B Register"]
pub mod mb_32b_group_mb1_32b_word7;
#[doc = "MB_64B_GROUP_MB1_64B_ID (rw) register accessor: Message Buffer 1 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb1_64b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb1_64b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb1_64b_id`] module"]
#[doc(alias = "MB_64B_GROUP_MB1_64B_ID")]
pub type Mb64bGroupMb1_64bId = crate::Reg<mb_64b_group_mb1_64b_id::Mb64bGroupMb1_64bIdSpec>;
#[doc = "Message Buffer 1 ID Register"]
pub mod mb_64b_group_mb1_64b_id;
#[doc = "MB_16B_GROUP_MB3_16B_ID (rw) register accessor: Message Buffer 3 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb3_16b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb3_16b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb3_16b_id`] module"]
#[doc(alias = "MB_16B_GROUP_MB3_16B_ID")]
pub type Mb16bGroupMb3_16bId = crate::Reg<mb_16b_group_mb3_16b_id::Mb16bGroupMb3_16bIdSpec>;
#[doc = "Message Buffer 3 ID Register"]
pub mod mb_16b_group_mb3_16b_id;
#[doc = "MB_8B_GROUP_MB4_8B_WORD1 (rw) register accessor: Message Buffer 4 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb4_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb4_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb4_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB4_8B_WORD1")]
pub type Mb8bGroupMb4_8bWord1 = crate::Reg<mb_8b_group_mb4_8b_word1::Mb8bGroupMb4_8bWord1Spec>;
#[doc = "Message Buffer 4 WORD_8B Register"]
pub mod mb_8b_group_mb4_8b_word1;
#[doc = "MB_GROUP_WORD14 (rw) register accessor: Message Buffer 4 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word14::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word14::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word14`] module"]
#[doc(alias = "MB_GROUP_WORD14")]
pub type MbGroupWord14 = crate::Reg<mb_group_word14::MbGroupWord14Spec>;
#[doc = "Message Buffer 4 WORD1 Register"]
pub mod mb_group_word14;
#[doc = "MB_GROUP_CS5 (rw) register accessor: Message Buffer 5 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs5`] module"]
#[doc(alias = "MB_GROUP_CS5")]
pub type MbGroupCs5 = crate::Reg<mb_group_cs5::MbGroupCs5Spec>;
#[doc = "Message Buffer 5 CS Register"]
pub mod mb_group_cs5;
#[doc = "MB_64B_GROUP_MB1_64B_WORD0 (rw) register accessor: Message Buffer 1 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb1_64b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb1_64b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb1_64b_word0`] module"]
#[doc(alias = "MB_64B_GROUP_MB1_64B_WORD0")]
pub type Mb64bGroupMb1_64bWord0 =
    crate::Reg<mb_64b_group_mb1_64b_word0::Mb64bGroupMb1_64bWord0Spec>;
#[doc = "Message Buffer 1 WORD_64B Register"]
pub mod mb_64b_group_mb1_64b_word0;
#[doc = "MB_32B_GROUP_MB2_32B_CS (rw) register accessor: Message Buffer 2 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb2_32b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb2_32b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb2_32b_cs`] module"]
#[doc(alias = "MB_32B_GROUP_MB2_32B_CS")]
pub type Mb32bGroupMb2_32bCs = crate::Reg<mb_32b_group_mb2_32b_cs::Mb32bGroupMb2_32bCsSpec>;
#[doc = "Message Buffer 2 CS Register"]
pub mod mb_32b_group_mb2_32b_cs;
#[doc = "MB_16B_GROUP_MB3_16B_WORD0 (rw) register accessor: Message Buffer 3 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb3_16b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb3_16b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb3_16b_word0`] module"]
#[doc(alias = "MB_16B_GROUP_MB3_16B_WORD0")]
pub type Mb16bGroupMb3_16bWord0 =
    crate::Reg<mb_16b_group_mb3_16b_word0::Mb16bGroupMb3_16bWord0Spec>;
#[doc = "Message Buffer 3 WORD_16B Register"]
pub mod mb_16b_group_mb3_16b_word0;
#[doc = "MB_8B_GROUP_MB5_8B_CS (rw) register accessor: Message Buffer 5 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb5_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb5_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb5_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB5_8B_CS")]
pub type Mb8bGroupMb5_8bCs = crate::Reg<mb_8b_group_mb5_8b_cs::Mb8bGroupMb5_8bCsSpec>;
#[doc = "Message Buffer 5 CS Register"]
pub mod mb_8b_group_mb5_8b_cs;
#[doc = "MB_GROUP_ID5 (rw) register accessor: Message Buffer 5 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id5`] module"]
#[doc(alias = "MB_GROUP_ID5")]
pub type MbGroupId5 = crate::Reg<mb_group_id5::MbGroupId5Spec>;
#[doc = "Message Buffer 5 ID Register"]
pub mod mb_group_id5;
#[doc = "MB_64B_GROUP_MB1_64B_WORD1 (rw) register accessor: Message Buffer 1 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb1_64b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb1_64b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb1_64b_word1`] module"]
#[doc(alias = "MB_64B_GROUP_MB1_64B_WORD1")]
pub type Mb64bGroupMb1_64bWord1 =
    crate::Reg<mb_64b_group_mb1_64b_word1::Mb64bGroupMb1_64bWord1Spec>;
#[doc = "Message Buffer 1 WORD_64B Register"]
pub mod mb_64b_group_mb1_64b_word1;
#[doc = "MB_32B_GROUP_MB2_32B_ID (rw) register accessor: Message Buffer 2 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb2_32b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb2_32b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb2_32b_id`] module"]
#[doc(alias = "MB_32B_GROUP_MB2_32B_ID")]
pub type Mb32bGroupMb2_32bId = crate::Reg<mb_32b_group_mb2_32b_id::Mb32bGroupMb2_32bIdSpec>;
#[doc = "Message Buffer 2 ID Register"]
pub mod mb_32b_group_mb2_32b_id;
#[doc = "MB_16B_GROUP_MB3_16B_WORD1 (rw) register accessor: Message Buffer 3 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb3_16b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb3_16b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb3_16b_word1`] module"]
#[doc(alias = "MB_16B_GROUP_MB3_16B_WORD1")]
pub type Mb16bGroupMb3_16bWord1 =
    crate::Reg<mb_16b_group_mb3_16b_word1::Mb16bGroupMb3_16bWord1Spec>;
#[doc = "Message Buffer 3 WORD_16B Register"]
pub mod mb_16b_group_mb3_16b_word1;
#[doc = "MB_8B_GROUP_MB5_8B_ID (rw) register accessor: Message Buffer 5 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb5_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb5_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb5_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB5_8B_ID")]
pub type Mb8bGroupMb5_8bId = crate::Reg<mb_8b_group_mb5_8b_id::Mb8bGroupMb5_8bIdSpec>;
#[doc = "Message Buffer 5 ID Register"]
pub mod mb_8b_group_mb5_8b_id;
#[doc = "MB_64B_GROUP_MB1_64B_WORD2 (rw) register accessor: Message Buffer 1 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb1_64b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb1_64b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb1_64b_word2`] module"]
#[doc(alias = "MB_64B_GROUP_MB1_64B_WORD2")]
pub type Mb64bGroupMb1_64bWord2 =
    crate::Reg<mb_64b_group_mb1_64b_word2::Mb64bGroupMb1_64bWord2Spec>;
#[doc = "Message Buffer 1 WORD_64B Register"]
pub mod mb_64b_group_mb1_64b_word2;
#[doc = "MB_32B_GROUP_MB2_32B_WORD0 (rw) register accessor: Message Buffer 2 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb2_32b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb2_32b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb2_32b_word0`] module"]
#[doc(alias = "MB_32B_GROUP_MB2_32B_WORD0")]
pub type Mb32bGroupMb2_32bWord0 =
    crate::Reg<mb_32b_group_mb2_32b_word0::Mb32bGroupMb2_32bWord0Spec>;
#[doc = "Message Buffer 2 WORD_32B Register"]
pub mod mb_32b_group_mb2_32b_word0;
#[doc = "MB_16B_GROUP_MB3_16B_WORD2 (rw) register accessor: Message Buffer 3 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb3_16b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb3_16b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb3_16b_word2`] module"]
#[doc(alias = "MB_16B_GROUP_MB3_16B_WORD2")]
pub type Mb16bGroupMb3_16bWord2 =
    crate::Reg<mb_16b_group_mb3_16b_word2::Mb16bGroupMb3_16bWord2Spec>;
#[doc = "Message Buffer 3 WORD_16B Register"]
pub mod mb_16b_group_mb3_16b_word2;
#[doc = "MB_8B_GROUP_MB5_8B_WORD0 (rw) register accessor: Message Buffer 5 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb5_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb5_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb5_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB5_8B_WORD0")]
pub type Mb8bGroupMb5_8bWord0 = crate::Reg<mb_8b_group_mb5_8b_word0::Mb8bGroupMb5_8bWord0Spec>;
#[doc = "Message Buffer 5 WORD_8B Register"]
pub mod mb_8b_group_mb5_8b_word0;
#[doc = "MB_GROUP_WORD05 (rw) register accessor: Message Buffer 5 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word05::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word05::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word05`] module"]
#[doc(alias = "MB_GROUP_WORD05")]
pub type MbGroupWord05 = crate::Reg<mb_group_word05::MbGroupWord05Spec>;
#[doc = "Message Buffer 5 WORD0 Register"]
pub mod mb_group_word05;
#[doc = "MB_64B_GROUP_MB1_64B_WORD3 (rw) register accessor: Message Buffer 1 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb1_64b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb1_64b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb1_64b_word3`] module"]
#[doc(alias = "MB_64B_GROUP_MB1_64B_WORD3")]
pub type Mb64bGroupMb1_64bWord3 =
    crate::Reg<mb_64b_group_mb1_64b_word3::Mb64bGroupMb1_64bWord3Spec>;
#[doc = "Message Buffer 1 WORD_64B Register"]
pub mod mb_64b_group_mb1_64b_word3;
#[doc = "MB_32B_GROUP_MB2_32B_WORD1 (rw) register accessor: Message Buffer 2 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb2_32b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb2_32b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb2_32b_word1`] module"]
#[doc(alias = "MB_32B_GROUP_MB2_32B_WORD1")]
pub type Mb32bGroupMb2_32bWord1 =
    crate::Reg<mb_32b_group_mb2_32b_word1::Mb32bGroupMb2_32bWord1Spec>;
#[doc = "Message Buffer 2 WORD_32B Register"]
pub mod mb_32b_group_mb2_32b_word1;
#[doc = "MB_16B_GROUP_MB3_16B_WORD3 (rw) register accessor: Message Buffer 3 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb3_16b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb3_16b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb3_16b_word3`] module"]
#[doc(alias = "MB_16B_GROUP_MB3_16B_WORD3")]
pub type Mb16bGroupMb3_16bWord3 =
    crate::Reg<mb_16b_group_mb3_16b_word3::Mb16bGroupMb3_16bWord3Spec>;
#[doc = "Message Buffer 3 WORD_16B Register"]
pub mod mb_16b_group_mb3_16b_word3;
#[doc = "MB_8B_GROUP_MB5_8B_WORD1 (rw) register accessor: Message Buffer 5 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb5_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb5_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb5_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB5_8B_WORD1")]
pub type Mb8bGroupMb5_8bWord1 = crate::Reg<mb_8b_group_mb5_8b_word1::Mb8bGroupMb5_8bWord1Spec>;
#[doc = "Message Buffer 5 WORD_8B Register"]
pub mod mb_8b_group_mb5_8b_word1;
#[doc = "MB_GROUP_WORD15 (rw) register accessor: Message Buffer 5 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word15::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word15::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word15`] module"]
#[doc(alias = "MB_GROUP_WORD15")]
pub type MbGroupWord15 = crate::Reg<mb_group_word15::MbGroupWord15Spec>;
#[doc = "Message Buffer 5 WORD1 Register"]
pub mod mb_group_word15;
#[doc = "MB_GROUP_CS6 (rw) register accessor: Message Buffer 6 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs6::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs6::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs6`] module"]
#[doc(alias = "MB_GROUP_CS6")]
pub type MbGroupCs6 = crate::Reg<mb_group_cs6::MbGroupCs6Spec>;
#[doc = "Message Buffer 6 CS Register"]
pub mod mb_group_cs6;
#[doc = "MB_64B_GROUP_MB1_64B_WORD4 (rw) register accessor: Message Buffer 1 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb1_64b_word4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb1_64b_word4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb1_64b_word4`] module"]
#[doc(alias = "MB_64B_GROUP_MB1_64B_WORD4")]
pub type Mb64bGroupMb1_64bWord4 =
    crate::Reg<mb_64b_group_mb1_64b_word4::Mb64bGroupMb1_64bWord4Spec>;
#[doc = "Message Buffer 1 WORD_64B Register"]
pub mod mb_64b_group_mb1_64b_word4;
#[doc = "MB_32B_GROUP_MB2_32B_WORD2 (rw) register accessor: Message Buffer 2 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb2_32b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb2_32b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb2_32b_word2`] module"]
#[doc(alias = "MB_32B_GROUP_MB2_32B_WORD2")]
pub type Mb32bGroupMb2_32bWord2 =
    crate::Reg<mb_32b_group_mb2_32b_word2::Mb32bGroupMb2_32bWord2Spec>;
#[doc = "Message Buffer 2 WORD_32B Register"]
pub mod mb_32b_group_mb2_32b_word2;
#[doc = "MB_16B_GROUP_MB4_16B_CS (rw) register accessor: Message Buffer 4 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb4_16b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb4_16b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb4_16b_cs`] module"]
#[doc(alias = "MB_16B_GROUP_MB4_16B_CS")]
pub type Mb16bGroupMb4_16bCs = crate::Reg<mb_16b_group_mb4_16b_cs::Mb16bGroupMb4_16bCsSpec>;
#[doc = "Message Buffer 4 CS Register"]
pub mod mb_16b_group_mb4_16b_cs;
#[doc = "MB_8B_GROUP_MB6_8B_CS (rw) register accessor: Message Buffer 6 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb6_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb6_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb6_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB6_8B_CS")]
pub type Mb8bGroupMb6_8bCs = crate::Reg<mb_8b_group_mb6_8b_cs::Mb8bGroupMb6_8bCsSpec>;
#[doc = "Message Buffer 6 CS Register"]
pub mod mb_8b_group_mb6_8b_cs;
#[doc = "MB_GROUP_ID6 (rw) register accessor: Message Buffer 6 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id6::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id6::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id6`] module"]
#[doc(alias = "MB_GROUP_ID6")]
pub type MbGroupId6 = crate::Reg<mb_group_id6::MbGroupId6Spec>;
#[doc = "Message Buffer 6 ID Register"]
pub mod mb_group_id6;
#[doc = "MB_64B_GROUP_MB1_64B_WORD5 (rw) register accessor: Message Buffer 1 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb1_64b_word5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb1_64b_word5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb1_64b_word5`] module"]
#[doc(alias = "MB_64B_GROUP_MB1_64B_WORD5")]
pub type Mb64bGroupMb1_64bWord5 =
    crate::Reg<mb_64b_group_mb1_64b_word5::Mb64bGroupMb1_64bWord5Spec>;
#[doc = "Message Buffer 1 WORD_64B Register"]
pub mod mb_64b_group_mb1_64b_word5;
#[doc = "MB_32B_GROUP_MB2_32B_WORD3 (rw) register accessor: Message Buffer 2 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb2_32b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb2_32b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb2_32b_word3`] module"]
#[doc(alias = "MB_32B_GROUP_MB2_32B_WORD3")]
pub type Mb32bGroupMb2_32bWord3 =
    crate::Reg<mb_32b_group_mb2_32b_word3::Mb32bGroupMb2_32bWord3Spec>;
#[doc = "Message Buffer 2 WORD_32B Register"]
pub mod mb_32b_group_mb2_32b_word3;
#[doc = "MB_16B_GROUP_MB4_16B_ID (rw) register accessor: Message Buffer 4 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb4_16b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb4_16b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb4_16b_id`] module"]
#[doc(alias = "MB_16B_GROUP_MB4_16B_ID")]
pub type Mb16bGroupMb4_16bId = crate::Reg<mb_16b_group_mb4_16b_id::Mb16bGroupMb4_16bIdSpec>;
#[doc = "Message Buffer 4 ID Register"]
pub mod mb_16b_group_mb4_16b_id;
#[doc = "MB_8B_GROUP_MB6_8B_ID (rw) register accessor: Message Buffer 6 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb6_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb6_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb6_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB6_8B_ID")]
pub type Mb8bGroupMb6_8bId = crate::Reg<mb_8b_group_mb6_8b_id::Mb8bGroupMb6_8bIdSpec>;
#[doc = "Message Buffer 6 ID Register"]
pub mod mb_8b_group_mb6_8b_id;
#[doc = "MB_64B_GROUP_MB1_64B_WORD6 (rw) register accessor: Message Buffer 1 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb1_64b_word6::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb1_64b_word6::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb1_64b_word6`] module"]
#[doc(alias = "MB_64B_GROUP_MB1_64B_WORD6")]
pub type Mb64bGroupMb1_64bWord6 =
    crate::Reg<mb_64b_group_mb1_64b_word6::Mb64bGroupMb1_64bWord6Spec>;
#[doc = "Message Buffer 1 WORD_64B Register"]
pub mod mb_64b_group_mb1_64b_word6;
#[doc = "MB_32B_GROUP_MB2_32B_WORD4 (rw) register accessor: Message Buffer 2 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb2_32b_word4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb2_32b_word4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb2_32b_word4`] module"]
#[doc(alias = "MB_32B_GROUP_MB2_32B_WORD4")]
pub type Mb32bGroupMb2_32bWord4 =
    crate::Reg<mb_32b_group_mb2_32b_word4::Mb32bGroupMb2_32bWord4Spec>;
#[doc = "Message Buffer 2 WORD_32B Register"]
pub mod mb_32b_group_mb2_32b_word4;
#[doc = "MB_16B_GROUP_MB4_16B_WORD0 (rw) register accessor: Message Buffer 4 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb4_16b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb4_16b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb4_16b_word0`] module"]
#[doc(alias = "MB_16B_GROUP_MB4_16B_WORD0")]
pub type Mb16bGroupMb4_16bWord0 =
    crate::Reg<mb_16b_group_mb4_16b_word0::Mb16bGroupMb4_16bWord0Spec>;
#[doc = "Message Buffer 4 WORD_16B Register"]
pub mod mb_16b_group_mb4_16b_word0;
#[doc = "MB_8B_GROUP_MB6_8B_WORD0 (rw) register accessor: Message Buffer 6 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb6_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb6_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb6_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB6_8B_WORD0")]
pub type Mb8bGroupMb6_8bWord0 = crate::Reg<mb_8b_group_mb6_8b_word0::Mb8bGroupMb6_8bWord0Spec>;
#[doc = "Message Buffer 6 WORD_8B Register"]
pub mod mb_8b_group_mb6_8b_word0;
#[doc = "MB_GROUP_WORD06 (rw) register accessor: Message Buffer 6 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word06::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word06::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word06`] module"]
#[doc(alias = "MB_GROUP_WORD06")]
pub type MbGroupWord06 = crate::Reg<mb_group_word06::MbGroupWord06Spec>;
#[doc = "Message Buffer 6 WORD0 Register"]
pub mod mb_group_word06;
#[doc = "MB_64B_GROUP_MB1_64B_WORD7 (rw) register accessor: Message Buffer 1 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb1_64b_word7::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb1_64b_word7::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb1_64b_word7`] module"]
#[doc(alias = "MB_64B_GROUP_MB1_64B_WORD7")]
pub type Mb64bGroupMb1_64bWord7 =
    crate::Reg<mb_64b_group_mb1_64b_word7::Mb64bGroupMb1_64bWord7Spec>;
#[doc = "Message Buffer 1 WORD_64B Register"]
pub mod mb_64b_group_mb1_64b_word7;
#[doc = "MB_32B_GROUP_MB2_32B_WORD5 (rw) register accessor: Message Buffer 2 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb2_32b_word5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb2_32b_word5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb2_32b_word5`] module"]
#[doc(alias = "MB_32B_GROUP_MB2_32B_WORD5")]
pub type Mb32bGroupMb2_32bWord5 =
    crate::Reg<mb_32b_group_mb2_32b_word5::Mb32bGroupMb2_32bWord5Spec>;
#[doc = "Message Buffer 2 WORD_32B Register"]
pub mod mb_32b_group_mb2_32b_word5;
#[doc = "MB_16B_GROUP_MB4_16B_WORD1 (rw) register accessor: Message Buffer 4 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb4_16b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb4_16b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb4_16b_word1`] module"]
#[doc(alias = "MB_16B_GROUP_MB4_16B_WORD1")]
pub type Mb16bGroupMb4_16bWord1 =
    crate::Reg<mb_16b_group_mb4_16b_word1::Mb16bGroupMb4_16bWord1Spec>;
#[doc = "Message Buffer 4 WORD_16B Register"]
pub mod mb_16b_group_mb4_16b_word1;
#[doc = "MB_8B_GROUP_MB6_8B_WORD1 (rw) register accessor: Message Buffer 6 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb6_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb6_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb6_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB6_8B_WORD1")]
pub type Mb8bGroupMb6_8bWord1 = crate::Reg<mb_8b_group_mb6_8b_word1::Mb8bGroupMb6_8bWord1Spec>;
#[doc = "Message Buffer 6 WORD_8B Register"]
pub mod mb_8b_group_mb6_8b_word1;
#[doc = "MB_GROUP_WORD16 (rw) register accessor: Message Buffer 6 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word16::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word16::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word16`] module"]
#[doc(alias = "MB_GROUP_WORD16")]
pub type MbGroupWord16 = crate::Reg<mb_group_word16::MbGroupWord16Spec>;
#[doc = "Message Buffer 6 WORD1 Register"]
pub mod mb_group_word16;
#[doc = "MB_GROUP_CS7 (rw) register accessor: Message Buffer 7 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs7::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs7::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs7`] module"]
#[doc(alias = "MB_GROUP_CS7")]
pub type MbGroupCs7 = crate::Reg<mb_group_cs7::MbGroupCs7Spec>;
#[doc = "Message Buffer 7 CS Register"]
pub mod mb_group_cs7;
#[doc = "MB_64B_GROUP_MB1_64B_WORD8 (rw) register accessor: Message Buffer 1 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb1_64b_word8::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb1_64b_word8::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb1_64b_word8`] module"]
#[doc(alias = "MB_64B_GROUP_MB1_64B_WORD8")]
pub type Mb64bGroupMb1_64bWord8 =
    crate::Reg<mb_64b_group_mb1_64b_word8::Mb64bGroupMb1_64bWord8Spec>;
#[doc = "Message Buffer 1 WORD_64B Register"]
pub mod mb_64b_group_mb1_64b_word8;
#[doc = "MB_32B_GROUP_MB2_32B_WORD6 (rw) register accessor: Message Buffer 2 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb2_32b_word6::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb2_32b_word6::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb2_32b_word6`] module"]
#[doc(alias = "MB_32B_GROUP_MB2_32B_WORD6")]
pub type Mb32bGroupMb2_32bWord6 =
    crate::Reg<mb_32b_group_mb2_32b_word6::Mb32bGroupMb2_32bWord6Spec>;
#[doc = "Message Buffer 2 WORD_32B Register"]
pub mod mb_32b_group_mb2_32b_word6;
#[doc = "MB_16B_GROUP_MB4_16B_WORD2 (rw) register accessor: Message Buffer 4 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb4_16b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb4_16b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb4_16b_word2`] module"]
#[doc(alias = "MB_16B_GROUP_MB4_16B_WORD2")]
pub type Mb16bGroupMb4_16bWord2 =
    crate::Reg<mb_16b_group_mb4_16b_word2::Mb16bGroupMb4_16bWord2Spec>;
#[doc = "Message Buffer 4 WORD_16B Register"]
pub mod mb_16b_group_mb4_16b_word2;
#[doc = "MB_8B_GROUP_MB7_8B_CS (rw) register accessor: Message Buffer 7 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb7_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb7_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb7_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB7_8B_CS")]
pub type Mb8bGroupMb7_8bCs = crate::Reg<mb_8b_group_mb7_8b_cs::Mb8bGroupMb7_8bCsSpec>;
#[doc = "Message Buffer 7 CS Register"]
pub mod mb_8b_group_mb7_8b_cs;
#[doc = "MB_GROUP_ID7 (rw) register accessor: Message Buffer 7 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id7::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id7::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id7`] module"]
#[doc(alias = "MB_GROUP_ID7")]
pub type MbGroupId7 = crate::Reg<mb_group_id7::MbGroupId7Spec>;
#[doc = "Message Buffer 7 ID Register"]
pub mod mb_group_id7;
#[doc = "MB_64B_GROUP_MB1_64B_WORD9 (rw) register accessor: Message Buffer 1 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb1_64b_word9::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb1_64b_word9::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb1_64b_word9`] module"]
#[doc(alias = "MB_64B_GROUP_MB1_64B_WORD9")]
pub type Mb64bGroupMb1_64bWord9 =
    crate::Reg<mb_64b_group_mb1_64b_word9::Mb64bGroupMb1_64bWord9Spec>;
#[doc = "Message Buffer 1 WORD_64B Register"]
pub mod mb_64b_group_mb1_64b_word9;
#[doc = "MB_32B_GROUP_MB2_32B_WORD7 (rw) register accessor: Message Buffer 2 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb2_32b_word7::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb2_32b_word7::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb2_32b_word7`] module"]
#[doc(alias = "MB_32B_GROUP_MB2_32B_WORD7")]
pub type Mb32bGroupMb2_32bWord7 =
    crate::Reg<mb_32b_group_mb2_32b_word7::Mb32bGroupMb2_32bWord7Spec>;
#[doc = "Message Buffer 2 WORD_32B Register"]
pub mod mb_32b_group_mb2_32b_word7;
#[doc = "MB_16B_GROUP_MB4_16B_WORD3 (rw) register accessor: Message Buffer 4 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb4_16b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb4_16b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb4_16b_word3`] module"]
#[doc(alias = "MB_16B_GROUP_MB4_16B_WORD3")]
pub type Mb16bGroupMb4_16bWord3 =
    crate::Reg<mb_16b_group_mb4_16b_word3::Mb16bGroupMb4_16bWord3Spec>;
#[doc = "Message Buffer 4 WORD_16B Register"]
pub mod mb_16b_group_mb4_16b_word3;
#[doc = "MB_8B_GROUP_MB7_8B_ID (rw) register accessor: Message Buffer 7 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb7_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb7_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb7_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB7_8B_ID")]
pub type Mb8bGroupMb7_8bId = crate::Reg<mb_8b_group_mb7_8b_id::Mb8bGroupMb7_8bIdSpec>;
#[doc = "Message Buffer 7 ID Register"]
pub mod mb_8b_group_mb7_8b_id;
#[doc = "MB_64B_GROUP_MB1_64B_WORD10 (rw) register accessor: Message Buffer 1 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb1_64b_word10::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb1_64b_word10::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb1_64b_word10`] module"]
#[doc(alias = "MB_64B_GROUP_MB1_64B_WORD10")]
pub type Mb64bGroupMb1_64bWord10 =
    crate::Reg<mb_64b_group_mb1_64b_word10::Mb64bGroupMb1_64bWord10Spec>;
#[doc = "Message Buffer 1 WORD_64B Register"]
pub mod mb_64b_group_mb1_64b_word10;
#[doc = "MB_32B_GROUP_MB3_32B_CS (rw) register accessor: Message Buffer 3 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb3_32b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb3_32b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb3_32b_cs`] module"]
#[doc(alias = "MB_32B_GROUP_MB3_32B_CS")]
pub type Mb32bGroupMb3_32bCs = crate::Reg<mb_32b_group_mb3_32b_cs::Mb32bGroupMb3_32bCsSpec>;
#[doc = "Message Buffer 3 CS Register"]
pub mod mb_32b_group_mb3_32b_cs;
#[doc = "MB_16B_GROUP_MB5_16B_CS (rw) register accessor: Message Buffer 5 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb5_16b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb5_16b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb5_16b_cs`] module"]
#[doc(alias = "MB_16B_GROUP_MB5_16B_CS")]
pub type Mb16bGroupMb5_16bCs = crate::Reg<mb_16b_group_mb5_16b_cs::Mb16bGroupMb5_16bCsSpec>;
#[doc = "Message Buffer 5 CS Register"]
pub mod mb_16b_group_mb5_16b_cs;
#[doc = "MB_8B_GROUP_MB7_8B_WORD0 (rw) register accessor: Message Buffer 7 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb7_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb7_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb7_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB7_8B_WORD0")]
pub type Mb8bGroupMb7_8bWord0 = crate::Reg<mb_8b_group_mb7_8b_word0::Mb8bGroupMb7_8bWord0Spec>;
#[doc = "Message Buffer 7 WORD_8B Register"]
pub mod mb_8b_group_mb7_8b_word0;
#[doc = "MB_GROUP_WORD07 (rw) register accessor: Message Buffer 7 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word07::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word07::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word07`] module"]
#[doc(alias = "MB_GROUP_WORD07")]
pub type MbGroupWord07 = crate::Reg<mb_group_word07::MbGroupWord07Spec>;
#[doc = "Message Buffer 7 WORD0 Register"]
pub mod mb_group_word07;
#[doc = "MB_64B_GROUP_MB1_64B_WORD11 (rw) register accessor: Message Buffer 1 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb1_64b_word11::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb1_64b_word11::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb1_64b_word11`] module"]
#[doc(alias = "MB_64B_GROUP_MB1_64B_WORD11")]
pub type Mb64bGroupMb1_64bWord11 =
    crate::Reg<mb_64b_group_mb1_64b_word11::Mb64bGroupMb1_64bWord11Spec>;
#[doc = "Message Buffer 1 WORD_64B Register"]
pub mod mb_64b_group_mb1_64b_word11;
#[doc = "MB_32B_GROUP_MB3_32B_ID (rw) register accessor: Message Buffer 3 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb3_32b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb3_32b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb3_32b_id`] module"]
#[doc(alias = "MB_32B_GROUP_MB3_32B_ID")]
pub type Mb32bGroupMb3_32bId = crate::Reg<mb_32b_group_mb3_32b_id::Mb32bGroupMb3_32bIdSpec>;
#[doc = "Message Buffer 3 ID Register"]
pub mod mb_32b_group_mb3_32b_id;
#[doc = "MB_16B_GROUP_MB5_16B_ID (rw) register accessor: Message Buffer 5 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb5_16b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb5_16b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb5_16b_id`] module"]
#[doc(alias = "MB_16B_GROUP_MB5_16B_ID")]
pub type Mb16bGroupMb5_16bId = crate::Reg<mb_16b_group_mb5_16b_id::Mb16bGroupMb5_16bIdSpec>;
#[doc = "Message Buffer 5 ID Register"]
pub mod mb_16b_group_mb5_16b_id;
#[doc = "MB_8B_GROUP_MB7_8B_WORD1 (rw) register accessor: Message Buffer 7 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb7_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb7_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb7_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB7_8B_WORD1")]
pub type Mb8bGroupMb7_8bWord1 = crate::Reg<mb_8b_group_mb7_8b_word1::Mb8bGroupMb7_8bWord1Spec>;
#[doc = "Message Buffer 7 WORD_8B Register"]
pub mod mb_8b_group_mb7_8b_word1;
#[doc = "MB_GROUP_WORD17 (rw) register accessor: Message Buffer 7 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word17::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word17::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word17`] module"]
#[doc(alias = "MB_GROUP_WORD17")]
pub type MbGroupWord17 = crate::Reg<mb_group_word17::MbGroupWord17Spec>;
#[doc = "Message Buffer 7 WORD1 Register"]
pub mod mb_group_word17;
#[doc = "MB_GROUP_CS8 (rw) register accessor: Message Buffer 8 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs8::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs8::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs8`] module"]
#[doc(alias = "MB_GROUP_CS8")]
pub type MbGroupCs8 = crate::Reg<mb_group_cs8::MbGroupCs8Spec>;
#[doc = "Message Buffer 8 CS Register"]
pub mod mb_group_cs8;
#[doc = "MB_64B_GROUP_MB1_64B_WORD12 (rw) register accessor: Message Buffer 1 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb1_64b_word12::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb1_64b_word12::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb1_64b_word12`] module"]
#[doc(alias = "MB_64B_GROUP_MB1_64B_WORD12")]
pub type Mb64bGroupMb1_64bWord12 =
    crate::Reg<mb_64b_group_mb1_64b_word12::Mb64bGroupMb1_64bWord12Spec>;
#[doc = "Message Buffer 1 WORD_64B Register"]
pub mod mb_64b_group_mb1_64b_word12;
#[doc = "MB_32B_GROUP_MB3_32B_WORD0 (rw) register accessor: Message Buffer 3 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb3_32b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb3_32b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb3_32b_word0`] module"]
#[doc(alias = "MB_32B_GROUP_MB3_32B_WORD0")]
pub type Mb32bGroupMb3_32bWord0 =
    crate::Reg<mb_32b_group_mb3_32b_word0::Mb32bGroupMb3_32bWord0Spec>;
#[doc = "Message Buffer 3 WORD_32B Register"]
pub mod mb_32b_group_mb3_32b_word0;
#[doc = "MB_16B_GROUP_MB5_16B_WORD0 (rw) register accessor: Message Buffer 5 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb5_16b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb5_16b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb5_16b_word0`] module"]
#[doc(alias = "MB_16B_GROUP_MB5_16B_WORD0")]
pub type Mb16bGroupMb5_16bWord0 =
    crate::Reg<mb_16b_group_mb5_16b_word0::Mb16bGroupMb5_16bWord0Spec>;
#[doc = "Message Buffer 5 WORD_16B Register"]
pub mod mb_16b_group_mb5_16b_word0;
#[doc = "MB_8B_GROUP_MB8_8B_CS (rw) register accessor: Message Buffer 8 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb8_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb8_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb8_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB8_8B_CS")]
pub type Mb8bGroupMb8_8bCs = crate::Reg<mb_8b_group_mb8_8b_cs::Mb8bGroupMb8_8bCsSpec>;
#[doc = "Message Buffer 8 CS Register"]
pub mod mb_8b_group_mb8_8b_cs;
#[doc = "MB_GROUP_ID8 (rw) register accessor: Message Buffer 8 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id8::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id8::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id8`] module"]
#[doc(alias = "MB_GROUP_ID8")]
pub type MbGroupId8 = crate::Reg<mb_group_id8::MbGroupId8Spec>;
#[doc = "Message Buffer 8 ID Register"]
pub mod mb_group_id8;
#[doc = "MB_64B_GROUP_MB1_64B_WORD13 (rw) register accessor: Message Buffer 1 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb1_64b_word13::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb1_64b_word13::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb1_64b_word13`] module"]
#[doc(alias = "MB_64B_GROUP_MB1_64B_WORD13")]
pub type Mb64bGroupMb1_64bWord13 =
    crate::Reg<mb_64b_group_mb1_64b_word13::Mb64bGroupMb1_64bWord13Spec>;
#[doc = "Message Buffer 1 WORD_64B Register"]
pub mod mb_64b_group_mb1_64b_word13;
#[doc = "MB_32B_GROUP_MB3_32B_WORD1 (rw) register accessor: Message Buffer 3 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb3_32b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb3_32b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb3_32b_word1`] module"]
#[doc(alias = "MB_32B_GROUP_MB3_32B_WORD1")]
pub type Mb32bGroupMb3_32bWord1 =
    crate::Reg<mb_32b_group_mb3_32b_word1::Mb32bGroupMb3_32bWord1Spec>;
#[doc = "Message Buffer 3 WORD_32B Register"]
pub mod mb_32b_group_mb3_32b_word1;
#[doc = "MB_16B_GROUP_MB5_16B_WORD1 (rw) register accessor: Message Buffer 5 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb5_16b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb5_16b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb5_16b_word1`] module"]
#[doc(alias = "MB_16B_GROUP_MB5_16B_WORD1")]
pub type Mb16bGroupMb5_16bWord1 =
    crate::Reg<mb_16b_group_mb5_16b_word1::Mb16bGroupMb5_16bWord1Spec>;
#[doc = "Message Buffer 5 WORD_16B Register"]
pub mod mb_16b_group_mb5_16b_word1;
#[doc = "MB_8B_GROUP_MB8_8B_ID (rw) register accessor: Message Buffer 8 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb8_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb8_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb8_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB8_8B_ID")]
pub type Mb8bGroupMb8_8bId = crate::Reg<mb_8b_group_mb8_8b_id::Mb8bGroupMb8_8bIdSpec>;
#[doc = "Message Buffer 8 ID Register"]
pub mod mb_8b_group_mb8_8b_id;
#[doc = "MB_64B_GROUP_MB1_64B_WORD14 (rw) register accessor: Message Buffer 1 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb1_64b_word14::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb1_64b_word14::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb1_64b_word14`] module"]
#[doc(alias = "MB_64B_GROUP_MB1_64B_WORD14")]
pub type Mb64bGroupMb1_64bWord14 =
    crate::Reg<mb_64b_group_mb1_64b_word14::Mb64bGroupMb1_64bWord14Spec>;
#[doc = "Message Buffer 1 WORD_64B Register"]
pub mod mb_64b_group_mb1_64b_word14;
#[doc = "MB_32B_GROUP_MB3_32B_WORD2 (rw) register accessor: Message Buffer 3 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb3_32b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb3_32b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb3_32b_word2`] module"]
#[doc(alias = "MB_32B_GROUP_MB3_32B_WORD2")]
pub type Mb32bGroupMb3_32bWord2 =
    crate::Reg<mb_32b_group_mb3_32b_word2::Mb32bGroupMb3_32bWord2Spec>;
#[doc = "Message Buffer 3 WORD_32B Register"]
pub mod mb_32b_group_mb3_32b_word2;
#[doc = "MB_16B_GROUP_MB5_16B_WORD2 (rw) register accessor: Message Buffer 5 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb5_16b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb5_16b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb5_16b_word2`] module"]
#[doc(alias = "MB_16B_GROUP_MB5_16B_WORD2")]
pub type Mb16bGroupMb5_16bWord2 =
    crate::Reg<mb_16b_group_mb5_16b_word2::Mb16bGroupMb5_16bWord2Spec>;
#[doc = "Message Buffer 5 WORD_16B Register"]
pub mod mb_16b_group_mb5_16b_word2;
#[doc = "MB_8B_GROUP_MB8_8B_WORD0 (rw) register accessor: Message Buffer 8 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb8_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb8_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb8_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB8_8B_WORD0")]
pub type Mb8bGroupMb8_8bWord0 = crate::Reg<mb_8b_group_mb8_8b_word0::Mb8bGroupMb8_8bWord0Spec>;
#[doc = "Message Buffer 8 WORD_8B Register"]
pub mod mb_8b_group_mb8_8b_word0;
#[doc = "MB_GROUP_WORD08 (rw) register accessor: Message Buffer 8 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word08::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word08::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word08`] module"]
#[doc(alias = "MB_GROUP_WORD08")]
pub type MbGroupWord08 = crate::Reg<mb_group_word08::MbGroupWord08Spec>;
#[doc = "Message Buffer 8 WORD0 Register"]
pub mod mb_group_word08;
#[doc = "MB_64B_GROUP_MB1_64B_WORD15 (rw) register accessor: Message Buffer 1 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb1_64b_word15::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb1_64b_word15::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb1_64b_word15`] module"]
#[doc(alias = "MB_64B_GROUP_MB1_64B_WORD15")]
pub type Mb64bGroupMb1_64bWord15 =
    crate::Reg<mb_64b_group_mb1_64b_word15::Mb64bGroupMb1_64bWord15Spec>;
#[doc = "Message Buffer 1 WORD_64B Register"]
pub mod mb_64b_group_mb1_64b_word15;
#[doc = "MB_32B_GROUP_MB3_32B_WORD3 (rw) register accessor: Message Buffer 3 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb3_32b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb3_32b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb3_32b_word3`] module"]
#[doc(alias = "MB_32B_GROUP_MB3_32B_WORD3")]
pub type Mb32bGroupMb3_32bWord3 =
    crate::Reg<mb_32b_group_mb3_32b_word3::Mb32bGroupMb3_32bWord3Spec>;
#[doc = "Message Buffer 3 WORD_32B Register"]
pub mod mb_32b_group_mb3_32b_word3;
#[doc = "MB_16B_GROUP_MB5_16B_WORD3 (rw) register accessor: Message Buffer 5 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb5_16b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb5_16b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb5_16b_word3`] module"]
#[doc(alias = "MB_16B_GROUP_MB5_16B_WORD3")]
pub type Mb16bGroupMb5_16bWord3 =
    crate::Reg<mb_16b_group_mb5_16b_word3::Mb16bGroupMb5_16bWord3Spec>;
#[doc = "Message Buffer 5 WORD_16B Register"]
pub mod mb_16b_group_mb5_16b_word3;
#[doc = "MB_8B_GROUP_MB8_8B_WORD1 (rw) register accessor: Message Buffer 8 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb8_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb8_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb8_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB8_8B_WORD1")]
pub type Mb8bGroupMb8_8bWord1 = crate::Reg<mb_8b_group_mb8_8b_word1::Mb8bGroupMb8_8bWord1Spec>;
#[doc = "Message Buffer 8 WORD_8B Register"]
pub mod mb_8b_group_mb8_8b_word1;
#[doc = "MB_GROUP_WORD18 (rw) register accessor: Message Buffer 8 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word18::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word18::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word18`] module"]
#[doc(alias = "MB_GROUP_WORD18")]
pub type MbGroupWord18 = crate::Reg<mb_group_word18::MbGroupWord18Spec>;
#[doc = "Message Buffer 8 WORD1 Register"]
pub mod mb_group_word18;
#[doc = "MB_GROUP_CS9 (rw) register accessor: Message Buffer 9 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs9::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs9::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs9`] module"]
#[doc(alias = "MB_GROUP_CS9")]
pub type MbGroupCs9 = crate::Reg<mb_group_cs9::MbGroupCs9Spec>;
#[doc = "Message Buffer 9 CS Register"]
pub mod mb_group_cs9;
#[doc = "MB_64B_GROUP_MB2_64B_CS (rw) register accessor: Message Buffer 2 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb2_64b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb2_64b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb2_64b_cs`] module"]
#[doc(alias = "MB_64B_GROUP_MB2_64B_CS")]
pub type Mb64bGroupMb2_64bCs = crate::Reg<mb_64b_group_mb2_64b_cs::Mb64bGroupMb2_64bCsSpec>;
#[doc = "Message Buffer 2 CS Register"]
pub mod mb_64b_group_mb2_64b_cs;
#[doc = "MB_32B_GROUP_MB3_32B_WORD4 (rw) register accessor: Message Buffer 3 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb3_32b_word4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb3_32b_word4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb3_32b_word4`] module"]
#[doc(alias = "MB_32B_GROUP_MB3_32B_WORD4")]
pub type Mb32bGroupMb3_32bWord4 =
    crate::Reg<mb_32b_group_mb3_32b_word4::Mb32bGroupMb3_32bWord4Spec>;
#[doc = "Message Buffer 3 WORD_32B Register"]
pub mod mb_32b_group_mb3_32b_word4;
#[doc = "MB_16B_GROUP_MB6_16B_CS (rw) register accessor: Message Buffer 6 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb6_16b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb6_16b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb6_16b_cs`] module"]
#[doc(alias = "MB_16B_GROUP_MB6_16B_CS")]
pub type Mb16bGroupMb6_16bCs = crate::Reg<mb_16b_group_mb6_16b_cs::Mb16bGroupMb6_16bCsSpec>;
#[doc = "Message Buffer 6 CS Register"]
pub mod mb_16b_group_mb6_16b_cs;
#[doc = "MB_8B_GROUP_MB9_8B_CS (rw) register accessor: Message Buffer 9 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb9_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb9_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb9_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB9_8B_CS")]
pub type Mb8bGroupMb9_8bCs = crate::Reg<mb_8b_group_mb9_8b_cs::Mb8bGroupMb9_8bCsSpec>;
#[doc = "Message Buffer 9 CS Register"]
pub mod mb_8b_group_mb9_8b_cs;
#[doc = "MB_GROUP_ID9 (rw) register accessor: Message Buffer 9 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id9::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id9::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id9`] module"]
#[doc(alias = "MB_GROUP_ID9")]
pub type MbGroupId9 = crate::Reg<mb_group_id9::MbGroupId9Spec>;
#[doc = "Message Buffer 9 ID Register"]
pub mod mb_group_id9;
#[doc = "MB_64B_GROUP_MB2_64B_ID (rw) register accessor: Message Buffer 2 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb2_64b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb2_64b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb2_64b_id`] module"]
#[doc(alias = "MB_64B_GROUP_MB2_64B_ID")]
pub type Mb64bGroupMb2_64bId = crate::Reg<mb_64b_group_mb2_64b_id::Mb64bGroupMb2_64bIdSpec>;
#[doc = "Message Buffer 2 ID Register"]
pub mod mb_64b_group_mb2_64b_id;
#[doc = "MB_32B_GROUP_MB3_32B_WORD5 (rw) register accessor: Message Buffer 3 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb3_32b_word5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb3_32b_word5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb3_32b_word5`] module"]
#[doc(alias = "MB_32B_GROUP_MB3_32B_WORD5")]
pub type Mb32bGroupMb3_32bWord5 =
    crate::Reg<mb_32b_group_mb3_32b_word5::Mb32bGroupMb3_32bWord5Spec>;
#[doc = "Message Buffer 3 WORD_32B Register"]
pub mod mb_32b_group_mb3_32b_word5;
#[doc = "MB_16B_GROUP_MB6_16B_ID (rw) register accessor: Message Buffer 6 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb6_16b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb6_16b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb6_16b_id`] module"]
#[doc(alias = "MB_16B_GROUP_MB6_16B_ID")]
pub type Mb16bGroupMb6_16bId = crate::Reg<mb_16b_group_mb6_16b_id::Mb16bGroupMb6_16bIdSpec>;
#[doc = "Message Buffer 6 ID Register"]
pub mod mb_16b_group_mb6_16b_id;
#[doc = "MB_8B_GROUP_MB9_8B_ID (rw) register accessor: Message Buffer 9 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb9_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb9_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb9_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB9_8B_ID")]
pub type Mb8bGroupMb9_8bId = crate::Reg<mb_8b_group_mb9_8b_id::Mb8bGroupMb9_8bIdSpec>;
#[doc = "Message Buffer 9 ID Register"]
pub mod mb_8b_group_mb9_8b_id;
#[doc = "MB_64B_GROUP_MB2_64B_WORD0 (rw) register accessor: Message Buffer 2 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb2_64b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb2_64b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb2_64b_word0`] module"]
#[doc(alias = "MB_64B_GROUP_MB2_64B_WORD0")]
pub type Mb64bGroupMb2_64bWord0 =
    crate::Reg<mb_64b_group_mb2_64b_word0::Mb64bGroupMb2_64bWord0Spec>;
#[doc = "Message Buffer 2 WORD_64B Register"]
pub mod mb_64b_group_mb2_64b_word0;
#[doc = "MB_32B_GROUP_MB3_32B_WORD6 (rw) register accessor: Message Buffer 3 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb3_32b_word6::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb3_32b_word6::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb3_32b_word6`] module"]
#[doc(alias = "MB_32B_GROUP_MB3_32B_WORD6")]
pub type Mb32bGroupMb3_32bWord6 =
    crate::Reg<mb_32b_group_mb3_32b_word6::Mb32bGroupMb3_32bWord6Spec>;
#[doc = "Message Buffer 3 WORD_32B Register"]
pub mod mb_32b_group_mb3_32b_word6;
#[doc = "MB_16B_GROUP_MB6_16B_WORD0 (rw) register accessor: Message Buffer 6 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb6_16b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb6_16b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb6_16b_word0`] module"]
#[doc(alias = "MB_16B_GROUP_MB6_16B_WORD0")]
pub type Mb16bGroupMb6_16bWord0 =
    crate::Reg<mb_16b_group_mb6_16b_word0::Mb16bGroupMb6_16bWord0Spec>;
#[doc = "Message Buffer 6 WORD_16B Register"]
pub mod mb_16b_group_mb6_16b_word0;
#[doc = "MB_8B_GROUP_MB9_8B_WORD0 (rw) register accessor: Message Buffer 9 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb9_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb9_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb9_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB9_8B_WORD0")]
pub type Mb8bGroupMb9_8bWord0 = crate::Reg<mb_8b_group_mb9_8b_word0::Mb8bGroupMb9_8bWord0Spec>;
#[doc = "Message Buffer 9 WORD_8B Register"]
pub mod mb_8b_group_mb9_8b_word0;
#[doc = "MB_GROUP_WORD09 (rw) register accessor: Message Buffer 9 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word09::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word09::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word09`] module"]
#[doc(alias = "MB_GROUP_WORD09")]
pub type MbGroupWord09 = crate::Reg<mb_group_word09::MbGroupWord09Spec>;
#[doc = "Message Buffer 9 WORD0 Register"]
pub mod mb_group_word09;
#[doc = "MB_64B_GROUP_MB2_64B_WORD1 (rw) register accessor: Message Buffer 2 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb2_64b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb2_64b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb2_64b_word1`] module"]
#[doc(alias = "MB_64B_GROUP_MB2_64B_WORD1")]
pub type Mb64bGroupMb2_64bWord1 =
    crate::Reg<mb_64b_group_mb2_64b_word1::Mb64bGroupMb2_64bWord1Spec>;
#[doc = "Message Buffer 2 WORD_64B Register"]
pub mod mb_64b_group_mb2_64b_word1;
#[doc = "MB_32B_GROUP_MB3_32B_WORD7 (rw) register accessor: Message Buffer 3 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb3_32b_word7::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb3_32b_word7::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb3_32b_word7`] module"]
#[doc(alias = "MB_32B_GROUP_MB3_32B_WORD7")]
pub type Mb32bGroupMb3_32bWord7 =
    crate::Reg<mb_32b_group_mb3_32b_word7::Mb32bGroupMb3_32bWord7Spec>;
#[doc = "Message Buffer 3 WORD_32B Register"]
pub mod mb_32b_group_mb3_32b_word7;
#[doc = "MB_16B_GROUP_MB6_16B_WORD1 (rw) register accessor: Message Buffer 6 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb6_16b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb6_16b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb6_16b_word1`] module"]
#[doc(alias = "MB_16B_GROUP_MB6_16B_WORD1")]
pub type Mb16bGroupMb6_16bWord1 =
    crate::Reg<mb_16b_group_mb6_16b_word1::Mb16bGroupMb6_16bWord1Spec>;
#[doc = "Message Buffer 6 WORD_16B Register"]
pub mod mb_16b_group_mb6_16b_word1;
#[doc = "MB_8B_GROUP_MB9_8B_WORD1 (rw) register accessor: Message Buffer 9 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb9_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb9_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb9_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB9_8B_WORD1")]
pub type Mb8bGroupMb9_8bWord1 = crate::Reg<mb_8b_group_mb9_8b_word1::Mb8bGroupMb9_8bWord1Spec>;
#[doc = "Message Buffer 9 WORD_8B Register"]
pub mod mb_8b_group_mb9_8b_word1;
#[doc = "MB_GROUP_WORD19 (rw) register accessor: Message Buffer 9 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word19::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word19::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word19`] module"]
#[doc(alias = "MB_GROUP_WORD19")]
pub type MbGroupWord19 = crate::Reg<mb_group_word19::MbGroupWord19Spec>;
#[doc = "Message Buffer 9 WORD1 Register"]
pub mod mb_group_word19;
#[doc = "MB_GROUP_CS10 (rw) register accessor: Message Buffer 10 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs10::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs10::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs10`] module"]
#[doc(alias = "MB_GROUP_CS10")]
pub type MbGroupCs10 = crate::Reg<mb_group_cs10::MbGroupCs10Spec>;
#[doc = "Message Buffer 10 CS Register"]
pub mod mb_group_cs10;
#[doc = "MB_8B_GROUP_MB10_8B_CS (rw) register accessor: Message Buffer 10 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb10_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb10_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb10_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB10_8B_CS")]
pub type Mb8bGroupMb10_8bCs = crate::Reg<mb_8b_group_mb10_8b_cs::Mb8bGroupMb10_8bCsSpec>;
#[doc = "Message Buffer 10 CS Register"]
pub mod mb_8b_group_mb10_8b_cs;
#[doc = "MB_64B_GROUP_MB2_64B_WORD2 (rw) register accessor: Message Buffer 2 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb2_64b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb2_64b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb2_64b_word2`] module"]
#[doc(alias = "MB_64B_GROUP_MB2_64B_WORD2")]
pub type Mb64bGroupMb2_64bWord2 =
    crate::Reg<mb_64b_group_mb2_64b_word2::Mb64bGroupMb2_64bWord2Spec>;
#[doc = "Message Buffer 2 WORD_64B Register"]
pub mod mb_64b_group_mb2_64b_word2;
#[doc = "MB_32B_GROUP_MB4_32B_CS (rw) register accessor: Message Buffer 4 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb4_32b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb4_32b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb4_32b_cs`] module"]
#[doc(alias = "MB_32B_GROUP_MB4_32B_CS")]
pub type Mb32bGroupMb4_32bCs = crate::Reg<mb_32b_group_mb4_32b_cs::Mb32bGroupMb4_32bCsSpec>;
#[doc = "Message Buffer 4 CS Register"]
pub mod mb_32b_group_mb4_32b_cs;
#[doc = "MB_16B_GROUP_MB6_16B_WORD2 (rw) register accessor: Message Buffer 6 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb6_16b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb6_16b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb6_16b_word2`] module"]
#[doc(alias = "MB_16B_GROUP_MB6_16B_WORD2")]
pub type Mb16bGroupMb6_16bWord2 =
    crate::Reg<mb_16b_group_mb6_16b_word2::Mb16bGroupMb6_16bWord2Spec>;
#[doc = "Message Buffer 6 WORD_16B Register"]
pub mod mb_16b_group_mb6_16b_word2;
#[doc = "MB_GROUP_ID10 (rw) register accessor: Message Buffer 10 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id10::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id10::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id10`] module"]
#[doc(alias = "MB_GROUP_ID10")]
pub type MbGroupId10 = crate::Reg<mb_group_id10::MbGroupId10Spec>;
#[doc = "Message Buffer 10 ID Register"]
pub mod mb_group_id10;
#[doc = "MB_8B_GROUP_MB10_8B_ID (rw) register accessor: Message Buffer 10 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb10_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb10_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb10_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB10_8B_ID")]
pub type Mb8bGroupMb10_8bId = crate::Reg<mb_8b_group_mb10_8b_id::Mb8bGroupMb10_8bIdSpec>;
#[doc = "Message Buffer 10 ID Register"]
pub mod mb_8b_group_mb10_8b_id;
#[doc = "MB_64B_GROUP_MB2_64B_WORD3 (rw) register accessor: Message Buffer 2 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb2_64b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb2_64b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb2_64b_word3`] module"]
#[doc(alias = "MB_64B_GROUP_MB2_64B_WORD3")]
pub type Mb64bGroupMb2_64bWord3 =
    crate::Reg<mb_64b_group_mb2_64b_word3::Mb64bGroupMb2_64bWord3Spec>;
#[doc = "Message Buffer 2 WORD_64B Register"]
pub mod mb_64b_group_mb2_64b_word3;
#[doc = "MB_32B_GROUP_MB4_32B_ID (rw) register accessor: Message Buffer 4 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb4_32b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb4_32b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb4_32b_id`] module"]
#[doc(alias = "MB_32B_GROUP_MB4_32B_ID")]
pub type Mb32bGroupMb4_32bId = crate::Reg<mb_32b_group_mb4_32b_id::Mb32bGroupMb4_32bIdSpec>;
#[doc = "Message Buffer 4 ID Register"]
pub mod mb_32b_group_mb4_32b_id;
#[doc = "MB_16B_GROUP_MB6_16B_WORD3 (rw) register accessor: Message Buffer 6 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb6_16b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb6_16b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb6_16b_word3`] module"]
#[doc(alias = "MB_16B_GROUP_MB6_16B_WORD3")]
pub type Mb16bGroupMb6_16bWord3 =
    crate::Reg<mb_16b_group_mb6_16b_word3::Mb16bGroupMb6_16bWord3Spec>;
#[doc = "Message Buffer 6 WORD_16B Register"]
pub mod mb_16b_group_mb6_16b_word3;
#[doc = "MB_8B_GROUP_MB10_8B_WORD0 (rw) register accessor: Message Buffer 10 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb10_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb10_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb10_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB10_8B_WORD0")]
pub type Mb8bGroupMb10_8bWord0 = crate::Reg<mb_8b_group_mb10_8b_word0::Mb8bGroupMb10_8bWord0Spec>;
#[doc = "Message Buffer 10 WORD_8B Register"]
pub mod mb_8b_group_mb10_8b_word0;
#[doc = "MB_64B_GROUP_MB2_64B_WORD4 (rw) register accessor: Message Buffer 2 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb2_64b_word4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb2_64b_word4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb2_64b_word4`] module"]
#[doc(alias = "MB_64B_GROUP_MB2_64B_WORD4")]
pub type Mb64bGroupMb2_64bWord4 =
    crate::Reg<mb_64b_group_mb2_64b_word4::Mb64bGroupMb2_64bWord4Spec>;
#[doc = "Message Buffer 2 WORD_64B Register"]
pub mod mb_64b_group_mb2_64b_word4;
#[doc = "MB_32B_GROUP_MB4_32B_WORD0 (rw) register accessor: Message Buffer 4 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb4_32b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb4_32b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb4_32b_word0`] module"]
#[doc(alias = "MB_32B_GROUP_MB4_32B_WORD0")]
pub type Mb32bGroupMb4_32bWord0 =
    crate::Reg<mb_32b_group_mb4_32b_word0::Mb32bGroupMb4_32bWord0Spec>;
#[doc = "Message Buffer 4 WORD_32B Register"]
pub mod mb_32b_group_mb4_32b_word0;
#[doc = "MB_16B_GROUP_MB7_16B_CS (rw) register accessor: Message Buffer 7 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb7_16b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb7_16b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb7_16b_cs`] module"]
#[doc(alias = "MB_16B_GROUP_MB7_16B_CS")]
pub type Mb16bGroupMb7_16bCs = crate::Reg<mb_16b_group_mb7_16b_cs::Mb16bGroupMb7_16bCsSpec>;
#[doc = "Message Buffer 7 CS Register"]
pub mod mb_16b_group_mb7_16b_cs;
#[doc = "MB_GROUP_WORD010 (rw) register accessor: Message Buffer 10 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word010::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word010::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word010`] module"]
#[doc(alias = "MB_GROUP_WORD010")]
pub type MbGroupWord010 = crate::Reg<mb_group_word010::MbGroupWord010Spec>;
#[doc = "Message Buffer 10 WORD0 Register"]
pub mod mb_group_word010;
#[doc = "MB_8B_GROUP_MB10_8B_WORD1 (rw) register accessor: Message Buffer 10 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb10_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb10_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb10_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB10_8B_WORD1")]
pub type Mb8bGroupMb10_8bWord1 = crate::Reg<mb_8b_group_mb10_8b_word1::Mb8bGroupMb10_8bWord1Spec>;
#[doc = "Message Buffer 10 WORD_8B Register"]
pub mod mb_8b_group_mb10_8b_word1;
#[doc = "MB_64B_GROUP_MB2_64B_WORD5 (rw) register accessor: Message Buffer 2 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb2_64b_word5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb2_64b_word5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb2_64b_word5`] module"]
#[doc(alias = "MB_64B_GROUP_MB2_64B_WORD5")]
pub type Mb64bGroupMb2_64bWord5 =
    crate::Reg<mb_64b_group_mb2_64b_word5::Mb64bGroupMb2_64bWord5Spec>;
#[doc = "Message Buffer 2 WORD_64B Register"]
pub mod mb_64b_group_mb2_64b_word5;
#[doc = "MB_32B_GROUP_MB4_32B_WORD1 (rw) register accessor: Message Buffer 4 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb4_32b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb4_32b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb4_32b_word1`] module"]
#[doc(alias = "MB_32B_GROUP_MB4_32B_WORD1")]
pub type Mb32bGroupMb4_32bWord1 =
    crate::Reg<mb_32b_group_mb4_32b_word1::Mb32bGroupMb4_32bWord1Spec>;
#[doc = "Message Buffer 4 WORD_32B Register"]
pub mod mb_32b_group_mb4_32b_word1;
#[doc = "MB_16B_GROUP_MB7_16B_ID (rw) register accessor: Message Buffer 7 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb7_16b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb7_16b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb7_16b_id`] module"]
#[doc(alias = "MB_16B_GROUP_MB7_16B_ID")]
pub type Mb16bGroupMb7_16bId = crate::Reg<mb_16b_group_mb7_16b_id::Mb16bGroupMb7_16bIdSpec>;
#[doc = "Message Buffer 7 ID Register"]
pub mod mb_16b_group_mb7_16b_id;
#[doc = "MB_GROUP_WORD110 (rw) register accessor: Message Buffer 10 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word110::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word110::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word110`] module"]
#[doc(alias = "MB_GROUP_WORD110")]
pub type MbGroupWord110 = crate::Reg<mb_group_word110::MbGroupWord110Spec>;
#[doc = "Message Buffer 10 WORD1 Register"]
pub mod mb_group_word110;
#[doc = "MB_GROUP_CS11 (rw) register accessor: Message Buffer 11 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs11::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs11::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs11`] module"]
#[doc(alias = "MB_GROUP_CS11")]
pub type MbGroupCs11 = crate::Reg<mb_group_cs11::MbGroupCs11Spec>;
#[doc = "Message Buffer 11 CS Register"]
pub mod mb_group_cs11;
#[doc = "MB_8B_GROUP_MB11_8B_CS (rw) register accessor: Message Buffer 11 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb11_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb11_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb11_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB11_8B_CS")]
pub type Mb8bGroupMb11_8bCs = crate::Reg<mb_8b_group_mb11_8b_cs::Mb8bGroupMb11_8bCsSpec>;
#[doc = "Message Buffer 11 CS Register"]
pub mod mb_8b_group_mb11_8b_cs;
#[doc = "MB_64B_GROUP_MB2_64B_WORD6 (rw) register accessor: Message Buffer 2 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb2_64b_word6::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb2_64b_word6::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb2_64b_word6`] module"]
#[doc(alias = "MB_64B_GROUP_MB2_64B_WORD6")]
pub type Mb64bGroupMb2_64bWord6 =
    crate::Reg<mb_64b_group_mb2_64b_word6::Mb64bGroupMb2_64bWord6Spec>;
#[doc = "Message Buffer 2 WORD_64B Register"]
pub mod mb_64b_group_mb2_64b_word6;
#[doc = "MB_32B_GROUP_MB4_32B_WORD2 (rw) register accessor: Message Buffer 4 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb4_32b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb4_32b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb4_32b_word2`] module"]
#[doc(alias = "MB_32B_GROUP_MB4_32B_WORD2")]
pub type Mb32bGroupMb4_32bWord2 =
    crate::Reg<mb_32b_group_mb4_32b_word2::Mb32bGroupMb4_32bWord2Spec>;
#[doc = "Message Buffer 4 WORD_32B Register"]
pub mod mb_32b_group_mb4_32b_word2;
#[doc = "MB_16B_GROUP_MB7_16B_WORD0 (rw) register accessor: Message Buffer 7 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb7_16b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb7_16b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb7_16b_word0`] module"]
#[doc(alias = "MB_16B_GROUP_MB7_16B_WORD0")]
pub type Mb16bGroupMb7_16bWord0 =
    crate::Reg<mb_16b_group_mb7_16b_word0::Mb16bGroupMb7_16bWord0Spec>;
#[doc = "Message Buffer 7 WORD_16B Register"]
pub mod mb_16b_group_mb7_16b_word0;
#[doc = "MB_GROUP_ID11 (rw) register accessor: Message Buffer 11 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id11::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id11::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id11`] module"]
#[doc(alias = "MB_GROUP_ID11")]
pub type MbGroupId11 = crate::Reg<mb_group_id11::MbGroupId11Spec>;
#[doc = "Message Buffer 11 ID Register"]
pub mod mb_group_id11;
#[doc = "MB_8B_GROUP_MB11_8B_ID (rw) register accessor: Message Buffer 11 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb11_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb11_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb11_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB11_8B_ID")]
pub type Mb8bGroupMb11_8bId = crate::Reg<mb_8b_group_mb11_8b_id::Mb8bGroupMb11_8bIdSpec>;
#[doc = "Message Buffer 11 ID Register"]
pub mod mb_8b_group_mb11_8b_id;
#[doc = "MB_64B_GROUP_MB2_64B_WORD7 (rw) register accessor: Message Buffer 2 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb2_64b_word7::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb2_64b_word7::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb2_64b_word7`] module"]
#[doc(alias = "MB_64B_GROUP_MB2_64B_WORD7")]
pub type Mb64bGroupMb2_64bWord7 =
    crate::Reg<mb_64b_group_mb2_64b_word7::Mb64bGroupMb2_64bWord7Spec>;
#[doc = "Message Buffer 2 WORD_64B Register"]
pub mod mb_64b_group_mb2_64b_word7;
#[doc = "MB_32B_GROUP_MB4_32B_WORD3 (rw) register accessor: Message Buffer 4 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb4_32b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb4_32b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb4_32b_word3`] module"]
#[doc(alias = "MB_32B_GROUP_MB4_32B_WORD3")]
pub type Mb32bGroupMb4_32bWord3 =
    crate::Reg<mb_32b_group_mb4_32b_word3::Mb32bGroupMb4_32bWord3Spec>;
#[doc = "Message Buffer 4 WORD_32B Register"]
pub mod mb_32b_group_mb4_32b_word3;
#[doc = "MB_16B_GROUP_MB7_16B_WORD1 (rw) register accessor: Message Buffer 7 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb7_16b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb7_16b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb7_16b_word1`] module"]
#[doc(alias = "MB_16B_GROUP_MB7_16B_WORD1")]
pub type Mb16bGroupMb7_16bWord1 =
    crate::Reg<mb_16b_group_mb7_16b_word1::Mb16bGroupMb7_16bWord1Spec>;
#[doc = "Message Buffer 7 WORD_16B Register"]
pub mod mb_16b_group_mb7_16b_word1;
#[doc = "MB_8B_GROUP_MB11_8B_WORD0 (rw) register accessor: Message Buffer 11 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb11_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb11_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb11_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB11_8B_WORD0")]
pub type Mb8bGroupMb11_8bWord0 = crate::Reg<mb_8b_group_mb11_8b_word0::Mb8bGroupMb11_8bWord0Spec>;
#[doc = "Message Buffer 11 WORD_8B Register"]
pub mod mb_8b_group_mb11_8b_word0;
#[doc = "MB_64B_GROUP_MB2_64B_WORD8 (rw) register accessor: Message Buffer 2 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb2_64b_word8::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb2_64b_word8::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb2_64b_word8`] module"]
#[doc(alias = "MB_64B_GROUP_MB2_64B_WORD8")]
pub type Mb64bGroupMb2_64bWord8 =
    crate::Reg<mb_64b_group_mb2_64b_word8::Mb64bGroupMb2_64bWord8Spec>;
#[doc = "Message Buffer 2 WORD_64B Register"]
pub mod mb_64b_group_mb2_64b_word8;
#[doc = "MB_32B_GROUP_MB4_32B_WORD4 (rw) register accessor: Message Buffer 4 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb4_32b_word4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb4_32b_word4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb4_32b_word4`] module"]
#[doc(alias = "MB_32B_GROUP_MB4_32B_WORD4")]
pub type Mb32bGroupMb4_32bWord4 =
    crate::Reg<mb_32b_group_mb4_32b_word4::Mb32bGroupMb4_32bWord4Spec>;
#[doc = "Message Buffer 4 WORD_32B Register"]
pub mod mb_32b_group_mb4_32b_word4;
#[doc = "MB_16B_GROUP_MB7_16B_WORD2 (rw) register accessor: Message Buffer 7 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb7_16b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb7_16b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb7_16b_word2`] module"]
#[doc(alias = "MB_16B_GROUP_MB7_16B_WORD2")]
pub type Mb16bGroupMb7_16bWord2 =
    crate::Reg<mb_16b_group_mb7_16b_word2::Mb16bGroupMb7_16bWord2Spec>;
#[doc = "Message Buffer 7 WORD_16B Register"]
pub mod mb_16b_group_mb7_16b_word2;
#[doc = "MB_GROUP_WORD011 (rw) register accessor: Message Buffer 11 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word011::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word011::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word011`] module"]
#[doc(alias = "MB_GROUP_WORD011")]
pub type MbGroupWord011 = crate::Reg<mb_group_word011::MbGroupWord011Spec>;
#[doc = "Message Buffer 11 WORD0 Register"]
pub mod mb_group_word011;
#[doc = "MB_8B_GROUP_MB11_8B_WORD1 (rw) register accessor: Message Buffer 11 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb11_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb11_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb11_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB11_8B_WORD1")]
pub type Mb8bGroupMb11_8bWord1 = crate::Reg<mb_8b_group_mb11_8b_word1::Mb8bGroupMb11_8bWord1Spec>;
#[doc = "Message Buffer 11 WORD_8B Register"]
pub mod mb_8b_group_mb11_8b_word1;
#[doc = "MB_64B_GROUP_MB2_64B_WORD9 (rw) register accessor: Message Buffer 2 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb2_64b_word9::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb2_64b_word9::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb2_64b_word9`] module"]
#[doc(alias = "MB_64B_GROUP_MB2_64B_WORD9")]
pub type Mb64bGroupMb2_64bWord9 =
    crate::Reg<mb_64b_group_mb2_64b_word9::Mb64bGroupMb2_64bWord9Spec>;
#[doc = "Message Buffer 2 WORD_64B Register"]
pub mod mb_64b_group_mb2_64b_word9;
#[doc = "MB_32B_GROUP_MB4_32B_WORD5 (rw) register accessor: Message Buffer 4 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb4_32b_word5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb4_32b_word5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb4_32b_word5`] module"]
#[doc(alias = "MB_32B_GROUP_MB4_32B_WORD5")]
pub type Mb32bGroupMb4_32bWord5 =
    crate::Reg<mb_32b_group_mb4_32b_word5::Mb32bGroupMb4_32bWord5Spec>;
#[doc = "Message Buffer 4 WORD_32B Register"]
pub mod mb_32b_group_mb4_32b_word5;
#[doc = "MB_16B_GROUP_MB7_16B_WORD3 (rw) register accessor: Message Buffer 7 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb7_16b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb7_16b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb7_16b_word3`] module"]
#[doc(alias = "MB_16B_GROUP_MB7_16B_WORD3")]
pub type Mb16bGroupMb7_16bWord3 =
    crate::Reg<mb_16b_group_mb7_16b_word3::Mb16bGroupMb7_16bWord3Spec>;
#[doc = "Message Buffer 7 WORD_16B Register"]
pub mod mb_16b_group_mb7_16b_word3;
#[doc = "MB_GROUP_WORD111 (rw) register accessor: Message Buffer 11 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word111::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word111::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word111`] module"]
#[doc(alias = "MB_GROUP_WORD111")]
pub type MbGroupWord111 = crate::Reg<mb_group_word111::MbGroupWord111Spec>;
#[doc = "Message Buffer 11 WORD1 Register"]
pub mod mb_group_word111;
#[doc = "MB_GROUP_CS12 (rw) register accessor: Message Buffer 12 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs12::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs12::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs12`] module"]
#[doc(alias = "MB_GROUP_CS12")]
pub type MbGroupCs12 = crate::Reg<mb_group_cs12::MbGroupCs12Spec>;
#[doc = "Message Buffer 12 CS Register"]
pub mod mb_group_cs12;
#[doc = "MB_8B_GROUP_MB12_8B_CS (rw) register accessor: Message Buffer 12 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb12_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb12_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb12_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB12_8B_CS")]
pub type Mb8bGroupMb12_8bCs = crate::Reg<mb_8b_group_mb12_8b_cs::Mb8bGroupMb12_8bCsSpec>;
#[doc = "Message Buffer 12 CS Register"]
pub mod mb_8b_group_mb12_8b_cs;
#[doc = "MB_64B_GROUP_MB2_64B_WORD10 (rw) register accessor: Message Buffer 2 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb2_64b_word10::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb2_64b_word10::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb2_64b_word10`] module"]
#[doc(alias = "MB_64B_GROUP_MB2_64B_WORD10")]
pub type Mb64bGroupMb2_64bWord10 =
    crate::Reg<mb_64b_group_mb2_64b_word10::Mb64bGroupMb2_64bWord10Spec>;
#[doc = "Message Buffer 2 WORD_64B Register"]
pub mod mb_64b_group_mb2_64b_word10;
#[doc = "MB_32B_GROUP_MB4_32B_WORD6 (rw) register accessor: Message Buffer 4 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb4_32b_word6::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb4_32b_word6::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb4_32b_word6`] module"]
#[doc(alias = "MB_32B_GROUP_MB4_32B_WORD6")]
pub type Mb32bGroupMb4_32bWord6 =
    crate::Reg<mb_32b_group_mb4_32b_word6::Mb32bGroupMb4_32bWord6Spec>;
#[doc = "Message Buffer 4 WORD_32B Register"]
pub mod mb_32b_group_mb4_32b_word6;
#[doc = "MB_16B_GROUP_MB8_16B_CS (rw) register accessor: Message Buffer 8 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb8_16b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb8_16b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb8_16b_cs`] module"]
#[doc(alias = "MB_16B_GROUP_MB8_16B_CS")]
pub type Mb16bGroupMb8_16bCs = crate::Reg<mb_16b_group_mb8_16b_cs::Mb16bGroupMb8_16bCsSpec>;
#[doc = "Message Buffer 8 CS Register"]
pub mod mb_16b_group_mb8_16b_cs;
#[doc = "MB_GROUP_ID12 (rw) register accessor: Message Buffer 12 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id12::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id12::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id12`] module"]
#[doc(alias = "MB_GROUP_ID12")]
pub type MbGroupId12 = crate::Reg<mb_group_id12::MbGroupId12Spec>;
#[doc = "Message Buffer 12 ID Register"]
pub mod mb_group_id12;
#[doc = "MB_8B_GROUP_MB12_8B_ID (rw) register accessor: Message Buffer 12 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb12_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb12_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb12_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB12_8B_ID")]
pub type Mb8bGroupMb12_8bId = crate::Reg<mb_8b_group_mb12_8b_id::Mb8bGroupMb12_8bIdSpec>;
#[doc = "Message Buffer 12 ID Register"]
pub mod mb_8b_group_mb12_8b_id;
#[doc = "MB_64B_GROUP_MB2_64B_WORD11 (rw) register accessor: Message Buffer 2 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb2_64b_word11::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb2_64b_word11::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb2_64b_word11`] module"]
#[doc(alias = "MB_64B_GROUP_MB2_64B_WORD11")]
pub type Mb64bGroupMb2_64bWord11 =
    crate::Reg<mb_64b_group_mb2_64b_word11::Mb64bGroupMb2_64bWord11Spec>;
#[doc = "Message Buffer 2 WORD_64B Register"]
pub mod mb_64b_group_mb2_64b_word11;
#[doc = "MB_32B_GROUP_MB4_32B_WORD7 (rw) register accessor: Message Buffer 4 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb4_32b_word7::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb4_32b_word7::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb4_32b_word7`] module"]
#[doc(alias = "MB_32B_GROUP_MB4_32B_WORD7")]
pub type Mb32bGroupMb4_32bWord7 =
    crate::Reg<mb_32b_group_mb4_32b_word7::Mb32bGroupMb4_32bWord7Spec>;
#[doc = "Message Buffer 4 WORD_32B Register"]
pub mod mb_32b_group_mb4_32b_word7;
#[doc = "MB_16B_GROUP_MB8_16B_ID (rw) register accessor: Message Buffer 8 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb8_16b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb8_16b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb8_16b_id`] module"]
#[doc(alias = "MB_16B_GROUP_MB8_16B_ID")]
pub type Mb16bGroupMb8_16bId = crate::Reg<mb_16b_group_mb8_16b_id::Mb16bGroupMb8_16bIdSpec>;
#[doc = "Message Buffer 8 ID Register"]
pub mod mb_16b_group_mb8_16b_id;
#[doc = "MB_8B_GROUP_MB12_8B_WORD0 (rw) register accessor: Message Buffer 12 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb12_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb12_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb12_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB12_8B_WORD0")]
pub type Mb8bGroupMb12_8bWord0 = crate::Reg<mb_8b_group_mb12_8b_word0::Mb8bGroupMb12_8bWord0Spec>;
#[doc = "Message Buffer 12 WORD_8B Register"]
pub mod mb_8b_group_mb12_8b_word0;
#[doc = "MB_64B_GROUP_MB2_64B_WORD12 (rw) register accessor: Message Buffer 2 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb2_64b_word12::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb2_64b_word12::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb2_64b_word12`] module"]
#[doc(alias = "MB_64B_GROUP_MB2_64B_WORD12")]
pub type Mb64bGroupMb2_64bWord12 =
    crate::Reg<mb_64b_group_mb2_64b_word12::Mb64bGroupMb2_64bWord12Spec>;
#[doc = "Message Buffer 2 WORD_64B Register"]
pub mod mb_64b_group_mb2_64b_word12;
#[doc = "MB_32B_GROUP_MB5_32B_CS (rw) register accessor: Message Buffer 5 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb5_32b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb5_32b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb5_32b_cs`] module"]
#[doc(alias = "MB_32B_GROUP_MB5_32B_CS")]
pub type Mb32bGroupMb5_32bCs = crate::Reg<mb_32b_group_mb5_32b_cs::Mb32bGroupMb5_32bCsSpec>;
#[doc = "Message Buffer 5 CS Register"]
pub mod mb_32b_group_mb5_32b_cs;
#[doc = "MB_16B_GROUP_MB8_16B_WORD0 (rw) register accessor: Message Buffer 8 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb8_16b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb8_16b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb8_16b_word0`] module"]
#[doc(alias = "MB_16B_GROUP_MB8_16B_WORD0")]
pub type Mb16bGroupMb8_16bWord0 =
    crate::Reg<mb_16b_group_mb8_16b_word0::Mb16bGroupMb8_16bWord0Spec>;
#[doc = "Message Buffer 8 WORD_16B Register"]
pub mod mb_16b_group_mb8_16b_word0;
#[doc = "MB_GROUP_WORD012 (rw) register accessor: Message Buffer 12 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word012::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word012::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word012`] module"]
#[doc(alias = "MB_GROUP_WORD012")]
pub type MbGroupWord012 = crate::Reg<mb_group_word012::MbGroupWord012Spec>;
#[doc = "Message Buffer 12 WORD0 Register"]
pub mod mb_group_word012;
#[doc = "MB_8B_GROUP_MB12_8B_WORD1 (rw) register accessor: Message Buffer 12 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb12_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb12_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb12_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB12_8B_WORD1")]
pub type Mb8bGroupMb12_8bWord1 = crate::Reg<mb_8b_group_mb12_8b_word1::Mb8bGroupMb12_8bWord1Spec>;
#[doc = "Message Buffer 12 WORD_8B Register"]
pub mod mb_8b_group_mb12_8b_word1;
#[doc = "MB_64B_GROUP_MB2_64B_WORD13 (rw) register accessor: Message Buffer 2 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb2_64b_word13::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb2_64b_word13::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb2_64b_word13`] module"]
#[doc(alias = "MB_64B_GROUP_MB2_64B_WORD13")]
pub type Mb64bGroupMb2_64bWord13 =
    crate::Reg<mb_64b_group_mb2_64b_word13::Mb64bGroupMb2_64bWord13Spec>;
#[doc = "Message Buffer 2 WORD_64B Register"]
pub mod mb_64b_group_mb2_64b_word13;
#[doc = "MB_32B_GROUP_MB5_32B_ID (rw) register accessor: Message Buffer 5 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb5_32b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb5_32b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb5_32b_id`] module"]
#[doc(alias = "MB_32B_GROUP_MB5_32B_ID")]
pub type Mb32bGroupMb5_32bId = crate::Reg<mb_32b_group_mb5_32b_id::Mb32bGroupMb5_32bIdSpec>;
#[doc = "Message Buffer 5 ID Register"]
pub mod mb_32b_group_mb5_32b_id;
#[doc = "MB_16B_GROUP_MB8_16B_WORD1 (rw) register accessor: Message Buffer 8 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb8_16b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb8_16b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb8_16b_word1`] module"]
#[doc(alias = "MB_16B_GROUP_MB8_16B_WORD1")]
pub type Mb16bGroupMb8_16bWord1 =
    crate::Reg<mb_16b_group_mb8_16b_word1::Mb16bGroupMb8_16bWord1Spec>;
#[doc = "Message Buffer 8 WORD_16B Register"]
pub mod mb_16b_group_mb8_16b_word1;
#[doc = "MB_GROUP_WORD112 (rw) register accessor: Message Buffer 12 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word112::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word112::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word112`] module"]
#[doc(alias = "MB_GROUP_WORD112")]
pub type MbGroupWord112 = crate::Reg<mb_group_word112::MbGroupWord112Spec>;
#[doc = "Message Buffer 12 WORD1 Register"]
pub mod mb_group_word112;
#[doc = "MB_GROUP_CS13 (rw) register accessor: Message Buffer 13 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs13::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs13::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs13`] module"]
#[doc(alias = "MB_GROUP_CS13")]
pub type MbGroupCs13 = crate::Reg<mb_group_cs13::MbGroupCs13Spec>;
#[doc = "Message Buffer 13 CS Register"]
pub mod mb_group_cs13;
#[doc = "MB_8B_GROUP_MB13_8B_CS (rw) register accessor: Message Buffer 13 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb13_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb13_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb13_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB13_8B_CS")]
pub type Mb8bGroupMb13_8bCs = crate::Reg<mb_8b_group_mb13_8b_cs::Mb8bGroupMb13_8bCsSpec>;
#[doc = "Message Buffer 13 CS Register"]
pub mod mb_8b_group_mb13_8b_cs;
#[doc = "MB_64B_GROUP_MB2_64B_WORD14 (rw) register accessor: Message Buffer 2 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb2_64b_word14::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb2_64b_word14::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb2_64b_word14`] module"]
#[doc(alias = "MB_64B_GROUP_MB2_64B_WORD14")]
pub type Mb64bGroupMb2_64bWord14 =
    crate::Reg<mb_64b_group_mb2_64b_word14::Mb64bGroupMb2_64bWord14Spec>;
#[doc = "Message Buffer 2 WORD_64B Register"]
pub mod mb_64b_group_mb2_64b_word14;
#[doc = "MB_32B_GROUP_MB5_32B_WORD0 (rw) register accessor: Message Buffer 5 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb5_32b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb5_32b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb5_32b_word0`] module"]
#[doc(alias = "MB_32B_GROUP_MB5_32B_WORD0")]
pub type Mb32bGroupMb5_32bWord0 =
    crate::Reg<mb_32b_group_mb5_32b_word0::Mb32bGroupMb5_32bWord0Spec>;
#[doc = "Message Buffer 5 WORD_32B Register"]
pub mod mb_32b_group_mb5_32b_word0;
#[doc = "MB_16B_GROUP_MB8_16B_WORD2 (rw) register accessor: Message Buffer 8 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb8_16b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb8_16b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb8_16b_word2`] module"]
#[doc(alias = "MB_16B_GROUP_MB8_16B_WORD2")]
pub type Mb16bGroupMb8_16bWord2 =
    crate::Reg<mb_16b_group_mb8_16b_word2::Mb16bGroupMb8_16bWord2Spec>;
#[doc = "Message Buffer 8 WORD_16B Register"]
pub mod mb_16b_group_mb8_16b_word2;
#[doc = "MB_GROUP_ID13 (rw) register accessor: Message Buffer 13 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id13::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id13::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id13`] module"]
#[doc(alias = "MB_GROUP_ID13")]
pub type MbGroupId13 = crate::Reg<mb_group_id13::MbGroupId13Spec>;
#[doc = "Message Buffer 13 ID Register"]
pub mod mb_group_id13;
#[doc = "MB_8B_GROUP_MB13_8B_ID (rw) register accessor: Message Buffer 13 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb13_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb13_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb13_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB13_8B_ID")]
pub type Mb8bGroupMb13_8bId = crate::Reg<mb_8b_group_mb13_8b_id::Mb8bGroupMb13_8bIdSpec>;
#[doc = "Message Buffer 13 ID Register"]
pub mod mb_8b_group_mb13_8b_id;
#[doc = "MB_64B_GROUP_MB2_64B_WORD15 (rw) register accessor: Message Buffer 2 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb2_64b_word15::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb2_64b_word15::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb2_64b_word15`] module"]
#[doc(alias = "MB_64B_GROUP_MB2_64B_WORD15")]
pub type Mb64bGroupMb2_64bWord15 =
    crate::Reg<mb_64b_group_mb2_64b_word15::Mb64bGroupMb2_64bWord15Spec>;
#[doc = "Message Buffer 2 WORD_64B Register"]
pub mod mb_64b_group_mb2_64b_word15;
#[doc = "MB_32B_GROUP_MB5_32B_WORD1 (rw) register accessor: Message Buffer 5 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb5_32b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb5_32b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb5_32b_word1`] module"]
#[doc(alias = "MB_32B_GROUP_MB5_32B_WORD1")]
pub type Mb32bGroupMb5_32bWord1 =
    crate::Reg<mb_32b_group_mb5_32b_word1::Mb32bGroupMb5_32bWord1Spec>;
#[doc = "Message Buffer 5 WORD_32B Register"]
pub mod mb_32b_group_mb5_32b_word1;
#[doc = "MB_16B_GROUP_MB8_16B_WORD3 (rw) register accessor: Message Buffer 8 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb8_16b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb8_16b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb8_16b_word3`] module"]
#[doc(alias = "MB_16B_GROUP_MB8_16B_WORD3")]
pub type Mb16bGroupMb8_16bWord3 =
    crate::Reg<mb_16b_group_mb8_16b_word3::Mb16bGroupMb8_16bWord3Spec>;
#[doc = "Message Buffer 8 WORD_16B Register"]
pub mod mb_16b_group_mb8_16b_word3;
#[doc = "MB_8B_GROUP_MB13_8B_WORD0 (rw) register accessor: Message Buffer 13 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb13_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb13_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb13_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB13_8B_WORD0")]
pub type Mb8bGroupMb13_8bWord0 = crate::Reg<mb_8b_group_mb13_8b_word0::Mb8bGroupMb13_8bWord0Spec>;
#[doc = "Message Buffer 13 WORD_8B Register"]
pub mod mb_8b_group_mb13_8b_word0;
#[doc = "MB_64B_GROUP_MB3_64B_CS (rw) register accessor: Message Buffer 3 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb3_64b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb3_64b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb3_64b_cs`] module"]
#[doc(alias = "MB_64B_GROUP_MB3_64B_CS")]
pub type Mb64bGroupMb3_64bCs = crate::Reg<mb_64b_group_mb3_64b_cs::Mb64bGroupMb3_64bCsSpec>;
#[doc = "Message Buffer 3 CS Register"]
pub mod mb_64b_group_mb3_64b_cs;
#[doc = "MB_32B_GROUP_MB5_32B_WORD2 (rw) register accessor: Message Buffer 5 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb5_32b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb5_32b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb5_32b_word2`] module"]
#[doc(alias = "MB_32B_GROUP_MB5_32B_WORD2")]
pub type Mb32bGroupMb5_32bWord2 =
    crate::Reg<mb_32b_group_mb5_32b_word2::Mb32bGroupMb5_32bWord2Spec>;
#[doc = "Message Buffer 5 WORD_32B Register"]
pub mod mb_32b_group_mb5_32b_word2;
#[doc = "MB_16B_GROUP_MB9_16B_CS (rw) register accessor: Message Buffer 9 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb9_16b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb9_16b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb9_16b_cs`] module"]
#[doc(alias = "MB_16B_GROUP_MB9_16B_CS")]
pub type Mb16bGroupMb9_16bCs = crate::Reg<mb_16b_group_mb9_16b_cs::Mb16bGroupMb9_16bCsSpec>;
#[doc = "Message Buffer 9 CS Register"]
pub mod mb_16b_group_mb9_16b_cs;
#[doc = "MB_GROUP_WORD013 (rw) register accessor: Message Buffer 13 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word013::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word013::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word013`] module"]
#[doc(alias = "MB_GROUP_WORD013")]
pub type MbGroupWord013 = crate::Reg<mb_group_word013::MbGroupWord013Spec>;
#[doc = "Message Buffer 13 WORD0 Register"]
pub mod mb_group_word013;
#[doc = "MB_8B_GROUP_MB13_8B_WORD1 (rw) register accessor: Message Buffer 13 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb13_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb13_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb13_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB13_8B_WORD1")]
pub type Mb8bGroupMb13_8bWord1 = crate::Reg<mb_8b_group_mb13_8b_word1::Mb8bGroupMb13_8bWord1Spec>;
#[doc = "Message Buffer 13 WORD_8B Register"]
pub mod mb_8b_group_mb13_8b_word1;
#[doc = "MB_64B_GROUP_MB3_64B_ID (rw) register accessor: Message Buffer 3 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb3_64b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb3_64b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb3_64b_id`] module"]
#[doc(alias = "MB_64B_GROUP_MB3_64B_ID")]
pub type Mb64bGroupMb3_64bId = crate::Reg<mb_64b_group_mb3_64b_id::Mb64bGroupMb3_64bIdSpec>;
#[doc = "Message Buffer 3 ID Register"]
pub mod mb_64b_group_mb3_64b_id;
#[doc = "MB_32B_GROUP_MB5_32B_WORD3 (rw) register accessor: Message Buffer 5 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb5_32b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb5_32b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb5_32b_word3`] module"]
#[doc(alias = "MB_32B_GROUP_MB5_32B_WORD3")]
pub type Mb32bGroupMb5_32bWord3 =
    crate::Reg<mb_32b_group_mb5_32b_word3::Mb32bGroupMb5_32bWord3Spec>;
#[doc = "Message Buffer 5 WORD_32B Register"]
pub mod mb_32b_group_mb5_32b_word3;
#[doc = "MB_16B_GROUP_MB9_16B_ID (rw) register accessor: Message Buffer 9 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb9_16b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb9_16b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb9_16b_id`] module"]
#[doc(alias = "MB_16B_GROUP_MB9_16B_ID")]
pub type Mb16bGroupMb9_16bId = crate::Reg<mb_16b_group_mb9_16b_id::Mb16bGroupMb9_16bIdSpec>;
#[doc = "Message Buffer 9 ID Register"]
pub mod mb_16b_group_mb9_16b_id;
#[doc = "MB_GROUP_WORD113 (rw) register accessor: Message Buffer 13 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word113::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word113::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word113`] module"]
#[doc(alias = "MB_GROUP_WORD113")]
pub type MbGroupWord113 = crate::Reg<mb_group_word113::MbGroupWord113Spec>;
#[doc = "Message Buffer 13 WORD1 Register"]
pub mod mb_group_word113;
#[doc = "MB_GROUP_CS14 (rw) register accessor: Message Buffer 14 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs14::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs14::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs14`] module"]
#[doc(alias = "MB_GROUP_CS14")]
pub type MbGroupCs14 = crate::Reg<mb_group_cs14::MbGroupCs14Spec>;
#[doc = "Message Buffer 14 CS Register"]
pub mod mb_group_cs14;
#[doc = "MB_8B_GROUP_MB14_8B_CS (rw) register accessor: Message Buffer 14 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb14_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb14_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb14_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB14_8B_CS")]
pub type Mb8bGroupMb14_8bCs = crate::Reg<mb_8b_group_mb14_8b_cs::Mb8bGroupMb14_8bCsSpec>;
#[doc = "Message Buffer 14 CS Register"]
pub mod mb_8b_group_mb14_8b_cs;
#[doc = "MB_64B_GROUP_MB3_64B_WORD0 (rw) register accessor: Message Buffer 3 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb3_64b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb3_64b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb3_64b_word0`] module"]
#[doc(alias = "MB_64B_GROUP_MB3_64B_WORD0")]
pub type Mb64bGroupMb3_64bWord0 =
    crate::Reg<mb_64b_group_mb3_64b_word0::Mb64bGroupMb3_64bWord0Spec>;
#[doc = "Message Buffer 3 WORD_64B Register"]
pub mod mb_64b_group_mb3_64b_word0;
#[doc = "MB_32B_GROUP_MB5_32B_WORD4 (rw) register accessor: Message Buffer 5 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb5_32b_word4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb5_32b_word4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb5_32b_word4`] module"]
#[doc(alias = "MB_32B_GROUP_MB5_32B_WORD4")]
pub type Mb32bGroupMb5_32bWord4 =
    crate::Reg<mb_32b_group_mb5_32b_word4::Mb32bGroupMb5_32bWord4Spec>;
#[doc = "Message Buffer 5 WORD_32B Register"]
pub mod mb_32b_group_mb5_32b_word4;
#[doc = "MB_16B_GROUP_MB9_16B_WORD0 (rw) register accessor: Message Buffer 9 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb9_16b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb9_16b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb9_16b_word0`] module"]
#[doc(alias = "MB_16B_GROUP_MB9_16B_WORD0")]
pub type Mb16bGroupMb9_16bWord0 =
    crate::Reg<mb_16b_group_mb9_16b_word0::Mb16bGroupMb9_16bWord0Spec>;
#[doc = "Message Buffer 9 WORD_16B Register"]
pub mod mb_16b_group_mb9_16b_word0;
#[doc = "MB_GROUP_ID14 (rw) register accessor: Message Buffer 14 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id14::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id14::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id14`] module"]
#[doc(alias = "MB_GROUP_ID14")]
pub type MbGroupId14 = crate::Reg<mb_group_id14::MbGroupId14Spec>;
#[doc = "Message Buffer 14 ID Register"]
pub mod mb_group_id14;
#[doc = "MB_8B_GROUP_MB14_8B_ID (rw) register accessor: Message Buffer 14 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb14_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb14_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb14_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB14_8B_ID")]
pub type Mb8bGroupMb14_8bId = crate::Reg<mb_8b_group_mb14_8b_id::Mb8bGroupMb14_8bIdSpec>;
#[doc = "Message Buffer 14 ID Register"]
pub mod mb_8b_group_mb14_8b_id;
#[doc = "MB_64B_GROUP_MB3_64B_WORD1 (rw) register accessor: Message Buffer 3 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb3_64b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb3_64b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb3_64b_word1`] module"]
#[doc(alias = "MB_64B_GROUP_MB3_64B_WORD1")]
pub type Mb64bGroupMb3_64bWord1 =
    crate::Reg<mb_64b_group_mb3_64b_word1::Mb64bGroupMb3_64bWord1Spec>;
#[doc = "Message Buffer 3 WORD_64B Register"]
pub mod mb_64b_group_mb3_64b_word1;
#[doc = "MB_32B_GROUP_MB5_32B_WORD5 (rw) register accessor: Message Buffer 5 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb5_32b_word5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb5_32b_word5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb5_32b_word5`] module"]
#[doc(alias = "MB_32B_GROUP_MB5_32B_WORD5")]
pub type Mb32bGroupMb5_32bWord5 =
    crate::Reg<mb_32b_group_mb5_32b_word5::Mb32bGroupMb5_32bWord5Spec>;
#[doc = "Message Buffer 5 WORD_32B Register"]
pub mod mb_32b_group_mb5_32b_word5;
#[doc = "MB_16B_GROUP_MB9_16B_WORD1 (rw) register accessor: Message Buffer 9 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb9_16b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb9_16b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb9_16b_word1`] module"]
#[doc(alias = "MB_16B_GROUP_MB9_16B_WORD1")]
pub type Mb16bGroupMb9_16bWord1 =
    crate::Reg<mb_16b_group_mb9_16b_word1::Mb16bGroupMb9_16bWord1Spec>;
#[doc = "Message Buffer 9 WORD_16B Register"]
pub mod mb_16b_group_mb9_16b_word1;
#[doc = "MB_8B_GROUP_MB14_8B_WORD0 (rw) register accessor: Message Buffer 14 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb14_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb14_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb14_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB14_8B_WORD0")]
pub type Mb8bGroupMb14_8bWord0 = crate::Reg<mb_8b_group_mb14_8b_word0::Mb8bGroupMb14_8bWord0Spec>;
#[doc = "Message Buffer 14 WORD_8B Register"]
pub mod mb_8b_group_mb14_8b_word0;
#[doc = "MB_64B_GROUP_MB3_64B_WORD2 (rw) register accessor: Message Buffer 3 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb3_64b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb3_64b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb3_64b_word2`] module"]
#[doc(alias = "MB_64B_GROUP_MB3_64B_WORD2")]
pub type Mb64bGroupMb3_64bWord2 =
    crate::Reg<mb_64b_group_mb3_64b_word2::Mb64bGroupMb3_64bWord2Spec>;
#[doc = "Message Buffer 3 WORD_64B Register"]
pub mod mb_64b_group_mb3_64b_word2;
#[doc = "MB_32B_GROUP_MB5_32B_WORD6 (rw) register accessor: Message Buffer 5 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb5_32b_word6::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb5_32b_word6::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb5_32b_word6`] module"]
#[doc(alias = "MB_32B_GROUP_MB5_32B_WORD6")]
pub type Mb32bGroupMb5_32bWord6 =
    crate::Reg<mb_32b_group_mb5_32b_word6::Mb32bGroupMb5_32bWord6Spec>;
#[doc = "Message Buffer 5 WORD_32B Register"]
pub mod mb_32b_group_mb5_32b_word6;
#[doc = "MB_16B_GROUP_MB9_16B_WORD2 (rw) register accessor: Message Buffer 9 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb9_16b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb9_16b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb9_16b_word2`] module"]
#[doc(alias = "MB_16B_GROUP_MB9_16B_WORD2")]
pub type Mb16bGroupMb9_16bWord2 =
    crate::Reg<mb_16b_group_mb9_16b_word2::Mb16bGroupMb9_16bWord2Spec>;
#[doc = "Message Buffer 9 WORD_16B Register"]
pub mod mb_16b_group_mb9_16b_word2;
#[doc = "MB_GROUP_WORD014 (rw) register accessor: Message Buffer 14 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word014::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word014::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word014`] module"]
#[doc(alias = "MB_GROUP_WORD014")]
pub type MbGroupWord014 = crate::Reg<mb_group_word014::MbGroupWord014Spec>;
#[doc = "Message Buffer 14 WORD0 Register"]
pub mod mb_group_word014;
#[doc = "MB_8B_GROUP_MB14_8B_WORD1 (rw) register accessor: Message Buffer 14 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb14_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb14_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb14_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB14_8B_WORD1")]
pub type Mb8bGroupMb14_8bWord1 = crate::Reg<mb_8b_group_mb14_8b_word1::Mb8bGroupMb14_8bWord1Spec>;
#[doc = "Message Buffer 14 WORD_8B Register"]
pub mod mb_8b_group_mb14_8b_word1;
#[doc = "MB_64B_GROUP_MB3_64B_WORD3 (rw) register accessor: Message Buffer 3 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb3_64b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb3_64b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb3_64b_word3`] module"]
#[doc(alias = "MB_64B_GROUP_MB3_64B_WORD3")]
pub type Mb64bGroupMb3_64bWord3 =
    crate::Reg<mb_64b_group_mb3_64b_word3::Mb64bGroupMb3_64bWord3Spec>;
#[doc = "Message Buffer 3 WORD_64B Register"]
pub mod mb_64b_group_mb3_64b_word3;
#[doc = "MB_32B_GROUP_MB5_32B_WORD7 (rw) register accessor: Message Buffer 5 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb5_32b_word7::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb5_32b_word7::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb5_32b_word7`] module"]
#[doc(alias = "MB_32B_GROUP_MB5_32B_WORD7")]
pub type Mb32bGroupMb5_32bWord7 =
    crate::Reg<mb_32b_group_mb5_32b_word7::Mb32bGroupMb5_32bWord7Spec>;
#[doc = "Message Buffer 5 WORD_32B Register"]
pub mod mb_32b_group_mb5_32b_word7;
#[doc = "MB_16B_GROUP_MB9_16B_WORD3 (rw) register accessor: Message Buffer 9 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb9_16b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb9_16b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb9_16b_word3`] module"]
#[doc(alias = "MB_16B_GROUP_MB9_16B_WORD3")]
pub type Mb16bGroupMb9_16bWord3 =
    crate::Reg<mb_16b_group_mb9_16b_word3::Mb16bGroupMb9_16bWord3Spec>;
#[doc = "Message Buffer 9 WORD_16B Register"]
pub mod mb_16b_group_mb9_16b_word3;
#[doc = "MB_GROUP_WORD114 (rw) register accessor: Message Buffer 14 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word114::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word114::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word114`] module"]
#[doc(alias = "MB_GROUP_WORD114")]
pub type MbGroupWord114 = crate::Reg<mb_group_word114::MbGroupWord114Spec>;
#[doc = "Message Buffer 14 WORD1 Register"]
pub mod mb_group_word114;
#[doc = "MB_GROUP_CS15 (rw) register accessor: Message Buffer 15 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs15::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs15::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs15`] module"]
#[doc(alias = "MB_GROUP_CS15")]
pub type MbGroupCs15 = crate::Reg<mb_group_cs15::MbGroupCs15Spec>;
#[doc = "Message Buffer 15 CS Register"]
pub mod mb_group_cs15;
#[doc = "MB_16B_GROUP_MB10_16B_CS (rw) register accessor: Message Buffer 10 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb10_16b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb10_16b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb10_16b_cs`] module"]
#[doc(alias = "MB_16B_GROUP_MB10_16B_CS")]
pub type Mb16bGroupMb10_16bCs = crate::Reg<mb_16b_group_mb10_16b_cs::Mb16bGroupMb10_16bCsSpec>;
#[doc = "Message Buffer 10 CS Register"]
pub mod mb_16b_group_mb10_16b_cs;
#[doc = "MB_8B_GROUP_MB15_8B_CS (rw) register accessor: Message Buffer 15 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb15_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb15_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb15_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB15_8B_CS")]
pub type Mb8bGroupMb15_8bCs = crate::Reg<mb_8b_group_mb15_8b_cs::Mb8bGroupMb15_8bCsSpec>;
#[doc = "Message Buffer 15 CS Register"]
pub mod mb_8b_group_mb15_8b_cs;
#[doc = "MB_64B_GROUP_MB3_64B_WORD4 (rw) register accessor: Message Buffer 3 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb3_64b_word4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb3_64b_word4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb3_64b_word4`] module"]
#[doc(alias = "MB_64B_GROUP_MB3_64B_WORD4")]
pub type Mb64bGroupMb3_64bWord4 =
    crate::Reg<mb_64b_group_mb3_64b_word4::Mb64bGroupMb3_64bWord4Spec>;
#[doc = "Message Buffer 3 WORD_64B Register"]
pub mod mb_64b_group_mb3_64b_word4;
#[doc = "MB_32B_GROUP_MB6_32B_CS (rw) register accessor: Message Buffer 6 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb6_32b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb6_32b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb6_32b_cs`] module"]
#[doc(alias = "MB_32B_GROUP_MB6_32B_CS")]
pub type Mb32bGroupMb6_32bCs = crate::Reg<mb_32b_group_mb6_32b_cs::Mb32bGroupMb6_32bCsSpec>;
#[doc = "Message Buffer 6 CS Register"]
pub mod mb_32b_group_mb6_32b_cs;
#[doc = "MB_GROUP_ID15 (rw) register accessor: Message Buffer 15 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id15::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id15::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id15`] module"]
#[doc(alias = "MB_GROUP_ID15")]
pub type MbGroupId15 = crate::Reg<mb_group_id15::MbGroupId15Spec>;
#[doc = "Message Buffer 15 ID Register"]
pub mod mb_group_id15;
#[doc = "MB_16B_GROUP_MB10_16B_ID (rw) register accessor: Message Buffer 10 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb10_16b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb10_16b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb10_16b_id`] module"]
#[doc(alias = "MB_16B_GROUP_MB10_16B_ID")]
pub type Mb16bGroupMb10_16bId = crate::Reg<mb_16b_group_mb10_16b_id::Mb16bGroupMb10_16bIdSpec>;
#[doc = "Message Buffer 10 ID Register"]
pub mod mb_16b_group_mb10_16b_id;
#[doc = "MB_8B_GROUP_MB15_8B_ID (rw) register accessor: Message Buffer 15 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb15_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb15_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb15_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB15_8B_ID")]
pub type Mb8bGroupMb15_8bId = crate::Reg<mb_8b_group_mb15_8b_id::Mb8bGroupMb15_8bIdSpec>;
#[doc = "Message Buffer 15 ID Register"]
pub mod mb_8b_group_mb15_8b_id;
#[doc = "MB_64B_GROUP_MB3_64B_WORD5 (rw) register accessor: Message Buffer 3 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb3_64b_word5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb3_64b_word5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb3_64b_word5`] module"]
#[doc(alias = "MB_64B_GROUP_MB3_64B_WORD5")]
pub type Mb64bGroupMb3_64bWord5 =
    crate::Reg<mb_64b_group_mb3_64b_word5::Mb64bGroupMb3_64bWord5Spec>;
#[doc = "Message Buffer 3 WORD_64B Register"]
pub mod mb_64b_group_mb3_64b_word5;
#[doc = "MB_32B_GROUP_MB6_32B_ID (rw) register accessor: Message Buffer 6 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb6_32b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb6_32b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb6_32b_id`] module"]
#[doc(alias = "MB_32B_GROUP_MB6_32B_ID")]
pub type Mb32bGroupMb6_32bId = crate::Reg<mb_32b_group_mb6_32b_id::Mb32bGroupMb6_32bIdSpec>;
#[doc = "Message Buffer 6 ID Register"]
pub mod mb_32b_group_mb6_32b_id;
#[doc = "MB_16B_GROUP_MB10_16B_WORD0 (rw) register accessor: Message Buffer 10 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb10_16b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb10_16b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb10_16b_word0`] module"]
#[doc(alias = "MB_16B_GROUP_MB10_16B_WORD0")]
pub type Mb16bGroupMb10_16bWord0 =
    crate::Reg<mb_16b_group_mb10_16b_word0::Mb16bGroupMb10_16bWord0Spec>;
#[doc = "Message Buffer 10 WORD_16B Register"]
pub mod mb_16b_group_mb10_16b_word0;
#[doc = "MB_8B_GROUP_MB15_8B_WORD0 (rw) register accessor: Message Buffer 15 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb15_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb15_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb15_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB15_8B_WORD0")]
pub type Mb8bGroupMb15_8bWord0 = crate::Reg<mb_8b_group_mb15_8b_word0::Mb8bGroupMb15_8bWord0Spec>;
#[doc = "Message Buffer 15 WORD_8B Register"]
pub mod mb_8b_group_mb15_8b_word0;
#[doc = "MB_64B_GROUP_MB3_64B_WORD6 (rw) register accessor: Message Buffer 3 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb3_64b_word6::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb3_64b_word6::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb3_64b_word6`] module"]
#[doc(alias = "MB_64B_GROUP_MB3_64B_WORD6")]
pub type Mb64bGroupMb3_64bWord6 =
    crate::Reg<mb_64b_group_mb3_64b_word6::Mb64bGroupMb3_64bWord6Spec>;
#[doc = "Message Buffer 3 WORD_64B Register"]
pub mod mb_64b_group_mb3_64b_word6;
#[doc = "MB_32B_GROUP_MB6_32B_WORD0 (rw) register accessor: Message Buffer 6 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb6_32b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb6_32b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb6_32b_word0`] module"]
#[doc(alias = "MB_32B_GROUP_MB6_32B_WORD0")]
pub type Mb32bGroupMb6_32bWord0 =
    crate::Reg<mb_32b_group_mb6_32b_word0::Mb32bGroupMb6_32bWord0Spec>;
#[doc = "Message Buffer 6 WORD_32B Register"]
pub mod mb_32b_group_mb6_32b_word0;
#[doc = "MB_GROUP_WORD015 (rw) register accessor: Message Buffer 15 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word015::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word015::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word015`] module"]
#[doc(alias = "MB_GROUP_WORD015")]
pub type MbGroupWord015 = crate::Reg<mb_group_word015::MbGroupWord015Spec>;
#[doc = "Message Buffer 15 WORD0 Register"]
pub mod mb_group_word015;
#[doc = "MB_16B_GROUP_MB10_16B_WORD1 (rw) register accessor: Message Buffer 10 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb10_16b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb10_16b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb10_16b_word1`] module"]
#[doc(alias = "MB_16B_GROUP_MB10_16B_WORD1")]
pub type Mb16bGroupMb10_16bWord1 =
    crate::Reg<mb_16b_group_mb10_16b_word1::Mb16bGroupMb10_16bWord1Spec>;
#[doc = "Message Buffer 10 WORD_16B Register"]
pub mod mb_16b_group_mb10_16b_word1;
#[doc = "MB_8B_GROUP_MB15_8B_WORD1 (rw) register accessor: Message Buffer 15 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb15_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb15_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb15_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB15_8B_WORD1")]
pub type Mb8bGroupMb15_8bWord1 = crate::Reg<mb_8b_group_mb15_8b_word1::Mb8bGroupMb15_8bWord1Spec>;
#[doc = "Message Buffer 15 WORD_8B Register"]
pub mod mb_8b_group_mb15_8b_word1;
#[doc = "MB_64B_GROUP_MB3_64B_WORD7 (rw) register accessor: Message Buffer 3 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb3_64b_word7::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb3_64b_word7::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb3_64b_word7`] module"]
#[doc(alias = "MB_64B_GROUP_MB3_64B_WORD7")]
pub type Mb64bGroupMb3_64bWord7 =
    crate::Reg<mb_64b_group_mb3_64b_word7::Mb64bGroupMb3_64bWord7Spec>;
#[doc = "Message Buffer 3 WORD_64B Register"]
pub mod mb_64b_group_mb3_64b_word7;
#[doc = "MB_32B_GROUP_MB6_32B_WORD1 (rw) register accessor: Message Buffer 6 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb6_32b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb6_32b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb6_32b_word1`] module"]
#[doc(alias = "MB_32B_GROUP_MB6_32B_WORD1")]
pub type Mb32bGroupMb6_32bWord1 =
    crate::Reg<mb_32b_group_mb6_32b_word1::Mb32bGroupMb6_32bWord1Spec>;
#[doc = "Message Buffer 6 WORD_32B Register"]
pub mod mb_32b_group_mb6_32b_word1;
#[doc = "MB_GROUP_WORD115 (rw) register accessor: Message Buffer 15 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word115::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word115::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word115`] module"]
#[doc(alias = "MB_GROUP_WORD115")]
pub type MbGroupWord115 = crate::Reg<mb_group_word115::MbGroupWord115Spec>;
#[doc = "Message Buffer 15 WORD1 Register"]
pub mod mb_group_word115;
#[doc = "MB_GROUP_CS16 (rw) register accessor: Message Buffer 16 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs16::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs16::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs16`] module"]
#[doc(alias = "MB_GROUP_CS16")]
pub type MbGroupCs16 = crate::Reg<mb_group_cs16::MbGroupCs16Spec>;
#[doc = "Message Buffer 16 CS Register"]
pub mod mb_group_cs16;
#[doc = "MB_16B_GROUP_MB10_16B_WORD2 (rw) register accessor: Message Buffer 10 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb10_16b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb10_16b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb10_16b_word2`] module"]
#[doc(alias = "MB_16B_GROUP_MB10_16B_WORD2")]
pub type Mb16bGroupMb10_16bWord2 =
    crate::Reg<mb_16b_group_mb10_16b_word2::Mb16bGroupMb10_16bWord2Spec>;
#[doc = "Message Buffer 10 WORD_16B Register"]
pub mod mb_16b_group_mb10_16b_word2;
#[doc = "MB_8B_GROUP_MB16_8B_CS (rw) register accessor: Message Buffer 16 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb16_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb16_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb16_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB16_8B_CS")]
pub type Mb8bGroupMb16_8bCs = crate::Reg<mb_8b_group_mb16_8b_cs::Mb8bGroupMb16_8bCsSpec>;
#[doc = "Message Buffer 16 CS Register"]
pub mod mb_8b_group_mb16_8b_cs;
#[doc = "MB_64B_GROUP_MB3_64B_WORD8 (rw) register accessor: Message Buffer 3 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb3_64b_word8::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb3_64b_word8::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb3_64b_word8`] module"]
#[doc(alias = "MB_64B_GROUP_MB3_64B_WORD8")]
pub type Mb64bGroupMb3_64bWord8 =
    crate::Reg<mb_64b_group_mb3_64b_word8::Mb64bGroupMb3_64bWord8Spec>;
#[doc = "Message Buffer 3 WORD_64B Register"]
pub mod mb_64b_group_mb3_64b_word8;
#[doc = "MB_32B_GROUP_MB6_32B_WORD2 (rw) register accessor: Message Buffer 6 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb6_32b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb6_32b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb6_32b_word2`] module"]
#[doc(alias = "MB_32B_GROUP_MB6_32B_WORD2")]
pub type Mb32bGroupMb6_32bWord2 =
    crate::Reg<mb_32b_group_mb6_32b_word2::Mb32bGroupMb6_32bWord2Spec>;
#[doc = "Message Buffer 6 WORD_32B Register"]
pub mod mb_32b_group_mb6_32b_word2;
#[doc = "MB_GROUP_ID16 (rw) register accessor: Message Buffer 16 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id16::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id16::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id16`] module"]
#[doc(alias = "MB_GROUP_ID16")]
pub type MbGroupId16 = crate::Reg<mb_group_id16::MbGroupId16Spec>;
#[doc = "Message Buffer 16 ID Register"]
pub mod mb_group_id16;
#[doc = "MB_16B_GROUP_MB10_16B_WORD3 (rw) register accessor: Message Buffer 10 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb10_16b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb10_16b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb10_16b_word3`] module"]
#[doc(alias = "MB_16B_GROUP_MB10_16B_WORD3")]
pub type Mb16bGroupMb10_16bWord3 =
    crate::Reg<mb_16b_group_mb10_16b_word3::Mb16bGroupMb10_16bWord3Spec>;
#[doc = "Message Buffer 10 WORD_16B Register"]
pub mod mb_16b_group_mb10_16b_word3;
#[doc = "MB_8B_GROUP_MB16_8B_ID (rw) register accessor: Message Buffer 16 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb16_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb16_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb16_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB16_8B_ID")]
pub type Mb8bGroupMb16_8bId = crate::Reg<mb_8b_group_mb16_8b_id::Mb8bGroupMb16_8bIdSpec>;
#[doc = "Message Buffer 16 ID Register"]
pub mod mb_8b_group_mb16_8b_id;
#[doc = "MB_64B_GROUP_MB3_64B_WORD9 (rw) register accessor: Message Buffer 3 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb3_64b_word9::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb3_64b_word9::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb3_64b_word9`] module"]
#[doc(alias = "MB_64B_GROUP_MB3_64B_WORD9")]
pub type Mb64bGroupMb3_64bWord9 =
    crate::Reg<mb_64b_group_mb3_64b_word9::Mb64bGroupMb3_64bWord9Spec>;
#[doc = "Message Buffer 3 WORD_64B Register"]
pub mod mb_64b_group_mb3_64b_word9;
#[doc = "MB_32B_GROUP_MB6_32B_WORD3 (rw) register accessor: Message Buffer 6 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb6_32b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb6_32b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb6_32b_word3`] module"]
#[doc(alias = "MB_32B_GROUP_MB6_32B_WORD3")]
pub type Mb32bGroupMb6_32bWord3 =
    crate::Reg<mb_32b_group_mb6_32b_word3::Mb32bGroupMb6_32bWord3Spec>;
#[doc = "Message Buffer 6 WORD_32B Register"]
pub mod mb_32b_group_mb6_32b_word3;
#[doc = "MB_16B_GROUP_MB11_16B_CS (rw) register accessor: Message Buffer 11 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb11_16b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb11_16b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb11_16b_cs`] module"]
#[doc(alias = "MB_16B_GROUP_MB11_16B_CS")]
pub type Mb16bGroupMb11_16bCs = crate::Reg<mb_16b_group_mb11_16b_cs::Mb16bGroupMb11_16bCsSpec>;
#[doc = "Message Buffer 11 CS Register"]
pub mod mb_16b_group_mb11_16b_cs;
#[doc = "MB_8B_GROUP_MB16_8B_WORD0 (rw) register accessor: Message Buffer 16 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb16_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb16_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb16_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB16_8B_WORD0")]
pub type Mb8bGroupMb16_8bWord0 = crate::Reg<mb_8b_group_mb16_8b_word0::Mb8bGroupMb16_8bWord0Spec>;
#[doc = "Message Buffer 16 WORD_8B Register"]
pub mod mb_8b_group_mb16_8b_word0;
#[doc = "MB_64B_GROUP_MB3_64B_WORD10 (rw) register accessor: Message Buffer 3 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb3_64b_word10::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb3_64b_word10::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb3_64b_word10`] module"]
#[doc(alias = "MB_64B_GROUP_MB3_64B_WORD10")]
pub type Mb64bGroupMb3_64bWord10 =
    crate::Reg<mb_64b_group_mb3_64b_word10::Mb64bGroupMb3_64bWord10Spec>;
#[doc = "Message Buffer 3 WORD_64B Register"]
pub mod mb_64b_group_mb3_64b_word10;
#[doc = "MB_32B_GROUP_MB6_32B_WORD4 (rw) register accessor: Message Buffer 6 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb6_32b_word4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb6_32b_word4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb6_32b_word4`] module"]
#[doc(alias = "MB_32B_GROUP_MB6_32B_WORD4")]
pub type Mb32bGroupMb6_32bWord4 =
    crate::Reg<mb_32b_group_mb6_32b_word4::Mb32bGroupMb6_32bWord4Spec>;
#[doc = "Message Buffer 6 WORD_32B Register"]
pub mod mb_32b_group_mb6_32b_word4;
#[doc = "MB_GROUP_WORD016 (rw) register accessor: Message Buffer 16 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word016::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word016::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word016`] module"]
#[doc(alias = "MB_GROUP_WORD016")]
pub type MbGroupWord016 = crate::Reg<mb_group_word016::MbGroupWord016Spec>;
#[doc = "Message Buffer 16 WORD0 Register"]
pub mod mb_group_word016;
#[doc = "MB_16B_GROUP_MB11_16B_ID (rw) register accessor: Message Buffer 11 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb11_16b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb11_16b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb11_16b_id`] module"]
#[doc(alias = "MB_16B_GROUP_MB11_16B_ID")]
pub type Mb16bGroupMb11_16bId = crate::Reg<mb_16b_group_mb11_16b_id::Mb16bGroupMb11_16bIdSpec>;
#[doc = "Message Buffer 11 ID Register"]
pub mod mb_16b_group_mb11_16b_id;
#[doc = "MB_8B_GROUP_MB16_8B_WORD1 (rw) register accessor: Message Buffer 16 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb16_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb16_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb16_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB16_8B_WORD1")]
pub type Mb8bGroupMb16_8bWord1 = crate::Reg<mb_8b_group_mb16_8b_word1::Mb8bGroupMb16_8bWord1Spec>;
#[doc = "Message Buffer 16 WORD_8B Register"]
pub mod mb_8b_group_mb16_8b_word1;
#[doc = "MB_64B_GROUP_MB3_64B_WORD11 (rw) register accessor: Message Buffer 3 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb3_64b_word11::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb3_64b_word11::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb3_64b_word11`] module"]
#[doc(alias = "MB_64B_GROUP_MB3_64B_WORD11")]
pub type Mb64bGroupMb3_64bWord11 =
    crate::Reg<mb_64b_group_mb3_64b_word11::Mb64bGroupMb3_64bWord11Spec>;
#[doc = "Message Buffer 3 WORD_64B Register"]
pub mod mb_64b_group_mb3_64b_word11;
#[doc = "MB_32B_GROUP_MB6_32B_WORD5 (rw) register accessor: Message Buffer 6 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb6_32b_word5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb6_32b_word5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb6_32b_word5`] module"]
#[doc(alias = "MB_32B_GROUP_MB6_32B_WORD5")]
pub type Mb32bGroupMb6_32bWord5 =
    crate::Reg<mb_32b_group_mb6_32b_word5::Mb32bGroupMb6_32bWord5Spec>;
#[doc = "Message Buffer 6 WORD_32B Register"]
pub mod mb_32b_group_mb6_32b_word5;
#[doc = "MB_GROUP_WORD116 (rw) register accessor: Message Buffer 16 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word116::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word116::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word116`] module"]
#[doc(alias = "MB_GROUP_WORD116")]
pub type MbGroupWord116 = crate::Reg<mb_group_word116::MbGroupWord116Spec>;
#[doc = "Message Buffer 16 WORD1 Register"]
pub mod mb_group_word116;
#[doc = "MB_GROUP_CS17 (rw) register accessor: Message Buffer 17 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs17::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs17::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs17`] module"]
#[doc(alias = "MB_GROUP_CS17")]
pub type MbGroupCs17 = crate::Reg<mb_group_cs17::MbGroupCs17Spec>;
#[doc = "Message Buffer 17 CS Register"]
pub mod mb_group_cs17;
#[doc = "MB_16B_GROUP_MB11_16B_WORD0 (rw) register accessor: Message Buffer 11 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb11_16b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb11_16b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb11_16b_word0`] module"]
#[doc(alias = "MB_16B_GROUP_MB11_16B_WORD0")]
pub type Mb16bGroupMb11_16bWord0 =
    crate::Reg<mb_16b_group_mb11_16b_word0::Mb16bGroupMb11_16bWord0Spec>;
#[doc = "Message Buffer 11 WORD_16B Register"]
pub mod mb_16b_group_mb11_16b_word0;
#[doc = "MB_8B_GROUP_MB17_8B_CS (rw) register accessor: Message Buffer 17 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb17_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb17_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb17_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB17_8B_CS")]
pub type Mb8bGroupMb17_8bCs = crate::Reg<mb_8b_group_mb17_8b_cs::Mb8bGroupMb17_8bCsSpec>;
#[doc = "Message Buffer 17 CS Register"]
pub mod mb_8b_group_mb17_8b_cs;
#[doc = "MB_64B_GROUP_MB3_64B_WORD12 (rw) register accessor: Message Buffer 3 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb3_64b_word12::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb3_64b_word12::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb3_64b_word12`] module"]
#[doc(alias = "MB_64B_GROUP_MB3_64B_WORD12")]
pub type Mb64bGroupMb3_64bWord12 =
    crate::Reg<mb_64b_group_mb3_64b_word12::Mb64bGroupMb3_64bWord12Spec>;
#[doc = "Message Buffer 3 WORD_64B Register"]
pub mod mb_64b_group_mb3_64b_word12;
#[doc = "MB_32B_GROUP_MB6_32B_WORD6 (rw) register accessor: Message Buffer 6 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb6_32b_word6::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb6_32b_word6::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb6_32b_word6`] module"]
#[doc(alias = "MB_32B_GROUP_MB6_32B_WORD6")]
pub type Mb32bGroupMb6_32bWord6 =
    crate::Reg<mb_32b_group_mb6_32b_word6::Mb32bGroupMb6_32bWord6Spec>;
#[doc = "Message Buffer 6 WORD_32B Register"]
pub mod mb_32b_group_mb6_32b_word6;
#[doc = "MB_GROUP_ID17 (rw) register accessor: Message Buffer 17 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id17::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id17::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id17`] module"]
#[doc(alias = "MB_GROUP_ID17")]
pub type MbGroupId17 = crate::Reg<mb_group_id17::MbGroupId17Spec>;
#[doc = "Message Buffer 17 ID Register"]
pub mod mb_group_id17;
#[doc = "MB_16B_GROUP_MB11_16B_WORD1 (rw) register accessor: Message Buffer 11 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb11_16b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb11_16b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb11_16b_word1`] module"]
#[doc(alias = "MB_16B_GROUP_MB11_16B_WORD1")]
pub type Mb16bGroupMb11_16bWord1 =
    crate::Reg<mb_16b_group_mb11_16b_word1::Mb16bGroupMb11_16bWord1Spec>;
#[doc = "Message Buffer 11 WORD_16B Register"]
pub mod mb_16b_group_mb11_16b_word1;
#[doc = "MB_8B_GROUP_MB17_8B_ID (rw) register accessor: Message Buffer 17 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb17_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb17_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb17_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB17_8B_ID")]
pub type Mb8bGroupMb17_8bId = crate::Reg<mb_8b_group_mb17_8b_id::Mb8bGroupMb17_8bIdSpec>;
#[doc = "Message Buffer 17 ID Register"]
pub mod mb_8b_group_mb17_8b_id;
#[doc = "MB_64B_GROUP_MB3_64B_WORD13 (rw) register accessor: Message Buffer 3 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb3_64b_word13::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb3_64b_word13::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb3_64b_word13`] module"]
#[doc(alias = "MB_64B_GROUP_MB3_64B_WORD13")]
pub type Mb64bGroupMb3_64bWord13 =
    crate::Reg<mb_64b_group_mb3_64b_word13::Mb64bGroupMb3_64bWord13Spec>;
#[doc = "Message Buffer 3 WORD_64B Register"]
pub mod mb_64b_group_mb3_64b_word13;
#[doc = "MB_32B_GROUP_MB6_32B_WORD7 (rw) register accessor: Message Buffer 6 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb6_32b_word7::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb6_32b_word7::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb6_32b_word7`] module"]
#[doc(alias = "MB_32B_GROUP_MB6_32B_WORD7")]
pub type Mb32bGroupMb6_32bWord7 =
    crate::Reg<mb_32b_group_mb6_32b_word7::Mb32bGroupMb6_32bWord7Spec>;
#[doc = "Message Buffer 6 WORD_32B Register"]
pub mod mb_32b_group_mb6_32b_word7;
#[doc = "MB_16B_GROUP_MB11_16B_WORD2 (rw) register accessor: Message Buffer 11 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb11_16b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb11_16b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb11_16b_word2`] module"]
#[doc(alias = "MB_16B_GROUP_MB11_16B_WORD2")]
pub type Mb16bGroupMb11_16bWord2 =
    crate::Reg<mb_16b_group_mb11_16b_word2::Mb16bGroupMb11_16bWord2Spec>;
#[doc = "Message Buffer 11 WORD_16B Register"]
pub mod mb_16b_group_mb11_16b_word2;
#[doc = "MB_8B_GROUP_MB17_8B_WORD0 (rw) register accessor: Message Buffer 17 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb17_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb17_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb17_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB17_8B_WORD0")]
pub type Mb8bGroupMb17_8bWord0 = crate::Reg<mb_8b_group_mb17_8b_word0::Mb8bGroupMb17_8bWord0Spec>;
#[doc = "Message Buffer 17 WORD_8B Register"]
pub mod mb_8b_group_mb17_8b_word0;
#[doc = "MB_64B_GROUP_MB3_64B_WORD14 (rw) register accessor: Message Buffer 3 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb3_64b_word14::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb3_64b_word14::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb3_64b_word14`] module"]
#[doc(alias = "MB_64B_GROUP_MB3_64B_WORD14")]
pub type Mb64bGroupMb3_64bWord14 =
    crate::Reg<mb_64b_group_mb3_64b_word14::Mb64bGroupMb3_64bWord14Spec>;
#[doc = "Message Buffer 3 WORD_64B Register"]
pub mod mb_64b_group_mb3_64b_word14;
#[doc = "MB_32B_GROUP_MB7_32B_CS (rw) register accessor: Message Buffer 7 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb7_32b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb7_32b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb7_32b_cs`] module"]
#[doc(alias = "MB_32B_GROUP_MB7_32B_CS")]
pub type Mb32bGroupMb7_32bCs = crate::Reg<mb_32b_group_mb7_32b_cs::Mb32bGroupMb7_32bCsSpec>;
#[doc = "Message Buffer 7 CS Register"]
pub mod mb_32b_group_mb7_32b_cs;
#[doc = "MB_GROUP_WORD017 (rw) register accessor: Message Buffer 17 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word017::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word017::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word017`] module"]
#[doc(alias = "MB_GROUP_WORD017")]
pub type MbGroupWord017 = crate::Reg<mb_group_word017::MbGroupWord017Spec>;
#[doc = "Message Buffer 17 WORD0 Register"]
pub mod mb_group_word017;
#[doc = "MB_16B_GROUP_MB11_16B_WORD3 (rw) register accessor: Message Buffer 11 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb11_16b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb11_16b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb11_16b_word3`] module"]
#[doc(alias = "MB_16B_GROUP_MB11_16B_WORD3")]
pub type Mb16bGroupMb11_16bWord3 =
    crate::Reg<mb_16b_group_mb11_16b_word3::Mb16bGroupMb11_16bWord3Spec>;
#[doc = "Message Buffer 11 WORD_16B Register"]
pub mod mb_16b_group_mb11_16b_word3;
#[doc = "MB_8B_GROUP_MB17_8B_WORD1 (rw) register accessor: Message Buffer 17 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb17_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb17_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb17_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB17_8B_WORD1")]
pub type Mb8bGroupMb17_8bWord1 = crate::Reg<mb_8b_group_mb17_8b_word1::Mb8bGroupMb17_8bWord1Spec>;
#[doc = "Message Buffer 17 WORD_8B Register"]
pub mod mb_8b_group_mb17_8b_word1;
#[doc = "MB_64B_GROUP_MB3_64B_WORD15 (rw) register accessor: Message Buffer 3 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb3_64b_word15::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb3_64b_word15::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb3_64b_word15`] module"]
#[doc(alias = "MB_64B_GROUP_MB3_64B_WORD15")]
pub type Mb64bGroupMb3_64bWord15 =
    crate::Reg<mb_64b_group_mb3_64b_word15::Mb64bGroupMb3_64bWord15Spec>;
#[doc = "Message Buffer 3 WORD_64B Register"]
pub mod mb_64b_group_mb3_64b_word15;
#[doc = "MB_32B_GROUP_MB7_32B_ID (rw) register accessor: Message Buffer 7 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb7_32b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb7_32b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb7_32b_id`] module"]
#[doc(alias = "MB_32B_GROUP_MB7_32B_ID")]
pub type Mb32bGroupMb7_32bId = crate::Reg<mb_32b_group_mb7_32b_id::Mb32bGroupMb7_32bIdSpec>;
#[doc = "Message Buffer 7 ID Register"]
pub mod mb_32b_group_mb7_32b_id;
#[doc = "MB_GROUP_WORD117 (rw) register accessor: Message Buffer 17 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word117::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word117::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word117`] module"]
#[doc(alias = "MB_GROUP_WORD117")]
pub type MbGroupWord117 = crate::Reg<mb_group_word117::MbGroupWord117Spec>;
#[doc = "Message Buffer 17 WORD1 Register"]
pub mod mb_group_word117;
#[doc = "MB_GROUP_CS18 (rw) register accessor: Message Buffer 18 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs18::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs18::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs18`] module"]
#[doc(alias = "MB_GROUP_CS18")]
pub type MbGroupCs18 = crate::Reg<mb_group_cs18::MbGroupCs18Spec>;
#[doc = "Message Buffer 18 CS Register"]
pub mod mb_group_cs18;
#[doc = "MB_16B_GROUP_MB12_16B_CS (rw) register accessor: Message Buffer 12 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb12_16b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb12_16b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb12_16b_cs`] module"]
#[doc(alias = "MB_16B_GROUP_MB12_16B_CS")]
pub type Mb16bGroupMb12_16bCs = crate::Reg<mb_16b_group_mb12_16b_cs::Mb16bGroupMb12_16bCsSpec>;
#[doc = "Message Buffer 12 CS Register"]
pub mod mb_16b_group_mb12_16b_cs;
#[doc = "MB_8B_GROUP_MB18_8B_CS (rw) register accessor: Message Buffer 18 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb18_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb18_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb18_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB18_8B_CS")]
pub type Mb8bGroupMb18_8bCs = crate::Reg<mb_8b_group_mb18_8b_cs::Mb8bGroupMb18_8bCsSpec>;
#[doc = "Message Buffer 18 CS Register"]
pub mod mb_8b_group_mb18_8b_cs;
#[doc = "MB_64B_GROUP_MB4_64B_CS (rw) register accessor: Message Buffer 4 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb4_64b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb4_64b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb4_64b_cs`] module"]
#[doc(alias = "MB_64B_GROUP_MB4_64B_CS")]
pub type Mb64bGroupMb4_64bCs = crate::Reg<mb_64b_group_mb4_64b_cs::Mb64bGroupMb4_64bCsSpec>;
#[doc = "Message Buffer 4 CS Register"]
pub mod mb_64b_group_mb4_64b_cs;
#[doc = "MB_32B_GROUP_MB7_32B_WORD0 (rw) register accessor: Message Buffer 7 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb7_32b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb7_32b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb7_32b_word0`] module"]
#[doc(alias = "MB_32B_GROUP_MB7_32B_WORD0")]
pub type Mb32bGroupMb7_32bWord0 =
    crate::Reg<mb_32b_group_mb7_32b_word0::Mb32bGroupMb7_32bWord0Spec>;
#[doc = "Message Buffer 7 WORD_32B Register"]
pub mod mb_32b_group_mb7_32b_word0;
#[doc = "MB_GROUP_ID18 (rw) register accessor: Message Buffer 18 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id18::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id18::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id18`] module"]
#[doc(alias = "MB_GROUP_ID18")]
pub type MbGroupId18 = crate::Reg<mb_group_id18::MbGroupId18Spec>;
#[doc = "Message Buffer 18 ID Register"]
pub mod mb_group_id18;
#[doc = "MB_16B_GROUP_MB12_16B_ID (rw) register accessor: Message Buffer 12 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb12_16b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb12_16b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb12_16b_id`] module"]
#[doc(alias = "MB_16B_GROUP_MB12_16B_ID")]
pub type Mb16bGroupMb12_16bId = crate::Reg<mb_16b_group_mb12_16b_id::Mb16bGroupMb12_16bIdSpec>;
#[doc = "Message Buffer 12 ID Register"]
pub mod mb_16b_group_mb12_16b_id;
#[doc = "MB_8B_GROUP_MB18_8B_ID (rw) register accessor: Message Buffer 18 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb18_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb18_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb18_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB18_8B_ID")]
pub type Mb8bGroupMb18_8bId = crate::Reg<mb_8b_group_mb18_8b_id::Mb8bGroupMb18_8bIdSpec>;
#[doc = "Message Buffer 18 ID Register"]
pub mod mb_8b_group_mb18_8b_id;
#[doc = "MB_64B_GROUP_MB4_64B_ID (rw) register accessor: Message Buffer 4 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb4_64b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb4_64b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb4_64b_id`] module"]
#[doc(alias = "MB_64B_GROUP_MB4_64B_ID")]
pub type Mb64bGroupMb4_64bId = crate::Reg<mb_64b_group_mb4_64b_id::Mb64bGroupMb4_64bIdSpec>;
#[doc = "Message Buffer 4 ID Register"]
pub mod mb_64b_group_mb4_64b_id;
#[doc = "MB_32B_GROUP_MB7_32B_WORD1 (rw) register accessor: Message Buffer 7 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb7_32b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb7_32b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb7_32b_word1`] module"]
#[doc(alias = "MB_32B_GROUP_MB7_32B_WORD1")]
pub type Mb32bGroupMb7_32bWord1 =
    crate::Reg<mb_32b_group_mb7_32b_word1::Mb32bGroupMb7_32bWord1Spec>;
#[doc = "Message Buffer 7 WORD_32B Register"]
pub mod mb_32b_group_mb7_32b_word1;
#[doc = "MB_16B_GROUP_MB12_16B_WORD0 (rw) register accessor: Message Buffer 12 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb12_16b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb12_16b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb12_16b_word0`] module"]
#[doc(alias = "MB_16B_GROUP_MB12_16B_WORD0")]
pub type Mb16bGroupMb12_16bWord0 =
    crate::Reg<mb_16b_group_mb12_16b_word0::Mb16bGroupMb12_16bWord0Spec>;
#[doc = "Message Buffer 12 WORD_16B Register"]
pub mod mb_16b_group_mb12_16b_word0;
#[doc = "MB_8B_GROUP_MB18_8B_WORD0 (rw) register accessor: Message Buffer 18 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb18_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb18_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb18_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB18_8B_WORD0")]
pub type Mb8bGroupMb18_8bWord0 = crate::Reg<mb_8b_group_mb18_8b_word0::Mb8bGroupMb18_8bWord0Spec>;
#[doc = "Message Buffer 18 WORD_8B Register"]
pub mod mb_8b_group_mb18_8b_word0;
#[doc = "MB_64B_GROUP_MB4_64B_WORD0 (rw) register accessor: Message Buffer 4 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb4_64b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb4_64b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb4_64b_word0`] module"]
#[doc(alias = "MB_64B_GROUP_MB4_64B_WORD0")]
pub type Mb64bGroupMb4_64bWord0 =
    crate::Reg<mb_64b_group_mb4_64b_word0::Mb64bGroupMb4_64bWord0Spec>;
#[doc = "Message Buffer 4 WORD_64B Register"]
pub mod mb_64b_group_mb4_64b_word0;
#[doc = "MB_32B_GROUP_MB7_32B_WORD2 (rw) register accessor: Message Buffer 7 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb7_32b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb7_32b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb7_32b_word2`] module"]
#[doc(alias = "MB_32B_GROUP_MB7_32B_WORD2")]
pub type Mb32bGroupMb7_32bWord2 =
    crate::Reg<mb_32b_group_mb7_32b_word2::Mb32bGroupMb7_32bWord2Spec>;
#[doc = "Message Buffer 7 WORD_32B Register"]
pub mod mb_32b_group_mb7_32b_word2;
#[doc = "MB_GROUP_WORD018 (rw) register accessor: Message Buffer 18 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word018::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word018::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word018`] module"]
#[doc(alias = "MB_GROUP_WORD018")]
pub type MbGroupWord018 = crate::Reg<mb_group_word018::MbGroupWord018Spec>;
#[doc = "Message Buffer 18 WORD0 Register"]
pub mod mb_group_word018;
#[doc = "MB_16B_GROUP_MB12_16B_WORD1 (rw) register accessor: Message Buffer 12 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb12_16b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb12_16b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb12_16b_word1`] module"]
#[doc(alias = "MB_16B_GROUP_MB12_16B_WORD1")]
pub type Mb16bGroupMb12_16bWord1 =
    crate::Reg<mb_16b_group_mb12_16b_word1::Mb16bGroupMb12_16bWord1Spec>;
#[doc = "Message Buffer 12 WORD_16B Register"]
pub mod mb_16b_group_mb12_16b_word1;
#[doc = "MB_8B_GROUP_MB18_8B_WORD1 (rw) register accessor: Message Buffer 18 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb18_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb18_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb18_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB18_8B_WORD1")]
pub type Mb8bGroupMb18_8bWord1 = crate::Reg<mb_8b_group_mb18_8b_word1::Mb8bGroupMb18_8bWord1Spec>;
#[doc = "Message Buffer 18 WORD_8B Register"]
pub mod mb_8b_group_mb18_8b_word1;
#[doc = "MB_64B_GROUP_MB4_64B_WORD1 (rw) register accessor: Message Buffer 4 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb4_64b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb4_64b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb4_64b_word1`] module"]
#[doc(alias = "MB_64B_GROUP_MB4_64B_WORD1")]
pub type Mb64bGroupMb4_64bWord1 =
    crate::Reg<mb_64b_group_mb4_64b_word1::Mb64bGroupMb4_64bWord1Spec>;
#[doc = "Message Buffer 4 WORD_64B Register"]
pub mod mb_64b_group_mb4_64b_word1;
#[doc = "MB_32B_GROUP_MB7_32B_WORD3 (rw) register accessor: Message Buffer 7 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb7_32b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb7_32b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb7_32b_word3`] module"]
#[doc(alias = "MB_32B_GROUP_MB7_32B_WORD3")]
pub type Mb32bGroupMb7_32bWord3 =
    crate::Reg<mb_32b_group_mb7_32b_word3::Mb32bGroupMb7_32bWord3Spec>;
#[doc = "Message Buffer 7 WORD_32B Register"]
pub mod mb_32b_group_mb7_32b_word3;
#[doc = "MB_GROUP_WORD118 (rw) register accessor: Message Buffer 18 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word118::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word118::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word118`] module"]
#[doc(alias = "MB_GROUP_WORD118")]
pub type MbGroupWord118 = crate::Reg<mb_group_word118::MbGroupWord118Spec>;
#[doc = "Message Buffer 18 WORD1 Register"]
pub mod mb_group_word118;
#[doc = "MB_GROUP_CS19 (rw) register accessor: Message Buffer 19 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs19::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs19::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs19`] module"]
#[doc(alias = "MB_GROUP_CS19")]
pub type MbGroupCs19 = crate::Reg<mb_group_cs19::MbGroupCs19Spec>;
#[doc = "Message Buffer 19 CS Register"]
pub mod mb_group_cs19;
#[doc = "MB_16B_GROUP_MB12_16B_WORD2 (rw) register accessor: Message Buffer 12 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb12_16b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb12_16b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb12_16b_word2`] module"]
#[doc(alias = "MB_16B_GROUP_MB12_16B_WORD2")]
pub type Mb16bGroupMb12_16bWord2 =
    crate::Reg<mb_16b_group_mb12_16b_word2::Mb16bGroupMb12_16bWord2Spec>;
#[doc = "Message Buffer 12 WORD_16B Register"]
pub mod mb_16b_group_mb12_16b_word2;
#[doc = "MB_8B_GROUP_MB19_8B_CS (rw) register accessor: Message Buffer 19 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb19_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb19_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb19_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB19_8B_CS")]
pub type Mb8bGroupMb19_8bCs = crate::Reg<mb_8b_group_mb19_8b_cs::Mb8bGroupMb19_8bCsSpec>;
#[doc = "Message Buffer 19 CS Register"]
pub mod mb_8b_group_mb19_8b_cs;
#[doc = "MB_64B_GROUP_MB4_64B_WORD2 (rw) register accessor: Message Buffer 4 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb4_64b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb4_64b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb4_64b_word2`] module"]
#[doc(alias = "MB_64B_GROUP_MB4_64B_WORD2")]
pub type Mb64bGroupMb4_64bWord2 =
    crate::Reg<mb_64b_group_mb4_64b_word2::Mb64bGroupMb4_64bWord2Spec>;
#[doc = "Message Buffer 4 WORD_64B Register"]
pub mod mb_64b_group_mb4_64b_word2;
#[doc = "MB_32B_GROUP_MB7_32B_WORD4 (rw) register accessor: Message Buffer 7 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb7_32b_word4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb7_32b_word4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb7_32b_word4`] module"]
#[doc(alias = "MB_32B_GROUP_MB7_32B_WORD4")]
pub type Mb32bGroupMb7_32bWord4 =
    crate::Reg<mb_32b_group_mb7_32b_word4::Mb32bGroupMb7_32bWord4Spec>;
#[doc = "Message Buffer 7 WORD_32B Register"]
pub mod mb_32b_group_mb7_32b_word4;
#[doc = "MB_GROUP_ID19 (rw) register accessor: Message Buffer 19 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id19::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id19::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id19`] module"]
#[doc(alias = "MB_GROUP_ID19")]
pub type MbGroupId19 = crate::Reg<mb_group_id19::MbGroupId19Spec>;
#[doc = "Message Buffer 19 ID Register"]
pub mod mb_group_id19;
#[doc = "MB_16B_GROUP_MB12_16B_WORD3 (rw) register accessor: Message Buffer 12 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb12_16b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb12_16b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb12_16b_word3`] module"]
#[doc(alias = "MB_16B_GROUP_MB12_16B_WORD3")]
pub type Mb16bGroupMb12_16bWord3 =
    crate::Reg<mb_16b_group_mb12_16b_word3::Mb16bGroupMb12_16bWord3Spec>;
#[doc = "Message Buffer 12 WORD_16B Register"]
pub mod mb_16b_group_mb12_16b_word3;
#[doc = "MB_8B_GROUP_MB19_8B_ID (rw) register accessor: Message Buffer 19 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb19_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb19_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb19_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB19_8B_ID")]
pub type Mb8bGroupMb19_8bId = crate::Reg<mb_8b_group_mb19_8b_id::Mb8bGroupMb19_8bIdSpec>;
#[doc = "Message Buffer 19 ID Register"]
pub mod mb_8b_group_mb19_8b_id;
#[doc = "MB_64B_GROUP_MB4_64B_WORD3 (rw) register accessor: Message Buffer 4 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb4_64b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb4_64b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb4_64b_word3`] module"]
#[doc(alias = "MB_64B_GROUP_MB4_64B_WORD3")]
pub type Mb64bGroupMb4_64bWord3 =
    crate::Reg<mb_64b_group_mb4_64b_word3::Mb64bGroupMb4_64bWord3Spec>;
#[doc = "Message Buffer 4 WORD_64B Register"]
pub mod mb_64b_group_mb4_64b_word3;
#[doc = "MB_32B_GROUP_MB7_32B_WORD5 (rw) register accessor: Message Buffer 7 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb7_32b_word5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb7_32b_word5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb7_32b_word5`] module"]
#[doc(alias = "MB_32B_GROUP_MB7_32B_WORD5")]
pub type Mb32bGroupMb7_32bWord5 =
    crate::Reg<mb_32b_group_mb7_32b_word5::Mb32bGroupMb7_32bWord5Spec>;
#[doc = "Message Buffer 7 WORD_32B Register"]
pub mod mb_32b_group_mb7_32b_word5;
#[doc = "MB_16B_GROUP_MB13_16B_CS (rw) register accessor: Message Buffer 13 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb13_16b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb13_16b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb13_16b_cs`] module"]
#[doc(alias = "MB_16B_GROUP_MB13_16B_CS")]
pub type Mb16bGroupMb13_16bCs = crate::Reg<mb_16b_group_mb13_16b_cs::Mb16bGroupMb13_16bCsSpec>;
#[doc = "Message Buffer 13 CS Register"]
pub mod mb_16b_group_mb13_16b_cs;
#[doc = "MB_8B_GROUP_MB19_8B_WORD0 (rw) register accessor: Message Buffer 19 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb19_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb19_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb19_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB19_8B_WORD0")]
pub type Mb8bGroupMb19_8bWord0 = crate::Reg<mb_8b_group_mb19_8b_word0::Mb8bGroupMb19_8bWord0Spec>;
#[doc = "Message Buffer 19 WORD_8B Register"]
pub mod mb_8b_group_mb19_8b_word0;
#[doc = "MB_64B_GROUP_MB4_64B_WORD4 (rw) register accessor: Message Buffer 4 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb4_64b_word4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb4_64b_word4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb4_64b_word4`] module"]
#[doc(alias = "MB_64B_GROUP_MB4_64B_WORD4")]
pub type Mb64bGroupMb4_64bWord4 =
    crate::Reg<mb_64b_group_mb4_64b_word4::Mb64bGroupMb4_64bWord4Spec>;
#[doc = "Message Buffer 4 WORD_64B Register"]
pub mod mb_64b_group_mb4_64b_word4;
#[doc = "MB_32B_GROUP_MB7_32B_WORD6 (rw) register accessor: Message Buffer 7 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb7_32b_word6::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb7_32b_word6::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb7_32b_word6`] module"]
#[doc(alias = "MB_32B_GROUP_MB7_32B_WORD6")]
pub type Mb32bGroupMb7_32bWord6 =
    crate::Reg<mb_32b_group_mb7_32b_word6::Mb32bGroupMb7_32bWord6Spec>;
#[doc = "Message Buffer 7 WORD_32B Register"]
pub mod mb_32b_group_mb7_32b_word6;
#[doc = "MB_GROUP_WORD019 (rw) register accessor: Message Buffer 19 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word019::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word019::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word019`] module"]
#[doc(alias = "MB_GROUP_WORD019")]
pub type MbGroupWord019 = crate::Reg<mb_group_word019::MbGroupWord019Spec>;
#[doc = "Message Buffer 19 WORD0 Register"]
pub mod mb_group_word019;
#[doc = "MB_16B_GROUP_MB13_16B_ID (rw) register accessor: Message Buffer 13 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb13_16b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb13_16b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb13_16b_id`] module"]
#[doc(alias = "MB_16B_GROUP_MB13_16B_ID")]
pub type Mb16bGroupMb13_16bId = crate::Reg<mb_16b_group_mb13_16b_id::Mb16bGroupMb13_16bIdSpec>;
#[doc = "Message Buffer 13 ID Register"]
pub mod mb_16b_group_mb13_16b_id;
#[doc = "MB_8B_GROUP_MB19_8B_WORD1 (rw) register accessor: Message Buffer 19 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb19_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb19_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb19_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB19_8B_WORD1")]
pub type Mb8bGroupMb19_8bWord1 = crate::Reg<mb_8b_group_mb19_8b_word1::Mb8bGroupMb19_8bWord1Spec>;
#[doc = "Message Buffer 19 WORD_8B Register"]
pub mod mb_8b_group_mb19_8b_word1;
#[doc = "MB_64B_GROUP_MB4_64B_WORD5 (rw) register accessor: Message Buffer 4 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb4_64b_word5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb4_64b_word5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb4_64b_word5`] module"]
#[doc(alias = "MB_64B_GROUP_MB4_64B_WORD5")]
pub type Mb64bGroupMb4_64bWord5 =
    crate::Reg<mb_64b_group_mb4_64b_word5::Mb64bGroupMb4_64bWord5Spec>;
#[doc = "Message Buffer 4 WORD_64B Register"]
pub mod mb_64b_group_mb4_64b_word5;
#[doc = "MB_32B_GROUP_MB7_32B_WORD7 (rw) register accessor: Message Buffer 7 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb7_32b_word7::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb7_32b_word7::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb7_32b_word7`] module"]
#[doc(alias = "MB_32B_GROUP_MB7_32B_WORD7")]
pub type Mb32bGroupMb7_32bWord7 =
    crate::Reg<mb_32b_group_mb7_32b_word7::Mb32bGroupMb7_32bWord7Spec>;
#[doc = "Message Buffer 7 WORD_32B Register"]
pub mod mb_32b_group_mb7_32b_word7;
#[doc = "MB_GROUP_WORD119 (rw) register accessor: Message Buffer 19 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word119::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word119::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word119`] module"]
#[doc(alias = "MB_GROUP_WORD119")]
pub type MbGroupWord119 = crate::Reg<mb_group_word119::MbGroupWord119Spec>;
#[doc = "Message Buffer 19 WORD1 Register"]
pub mod mb_group_word119;
#[doc = "MB_GROUP_CS20 (rw) register accessor: Message Buffer 20 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs20::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs20::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs20`] module"]
#[doc(alias = "MB_GROUP_CS20")]
pub type MbGroupCs20 = crate::Reg<mb_group_cs20::MbGroupCs20Spec>;
#[doc = "Message Buffer 20 CS Register"]
pub mod mb_group_cs20;
#[doc = "MB_16B_GROUP_MB13_16B_WORD0 (rw) register accessor: Message Buffer 13 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb13_16b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb13_16b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb13_16b_word0`] module"]
#[doc(alias = "MB_16B_GROUP_MB13_16B_WORD0")]
pub type Mb16bGroupMb13_16bWord0 =
    crate::Reg<mb_16b_group_mb13_16b_word0::Mb16bGroupMb13_16bWord0Spec>;
#[doc = "Message Buffer 13 WORD_16B Register"]
pub mod mb_16b_group_mb13_16b_word0;
#[doc = "MB_8B_GROUP_MB20_8B_CS (rw) register accessor: Message Buffer 20 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb20_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb20_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb20_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB20_8B_CS")]
pub type Mb8bGroupMb20_8bCs = crate::Reg<mb_8b_group_mb20_8b_cs::Mb8bGroupMb20_8bCsSpec>;
#[doc = "Message Buffer 20 CS Register"]
pub mod mb_8b_group_mb20_8b_cs;
#[doc = "MB_64B_GROUP_MB4_64B_WORD6 (rw) register accessor: Message Buffer 4 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb4_64b_word6::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb4_64b_word6::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb4_64b_word6`] module"]
#[doc(alias = "MB_64B_GROUP_MB4_64B_WORD6")]
pub type Mb64bGroupMb4_64bWord6 =
    crate::Reg<mb_64b_group_mb4_64b_word6::Mb64bGroupMb4_64bWord6Spec>;
#[doc = "Message Buffer 4 WORD_64B Register"]
pub mod mb_64b_group_mb4_64b_word6;
#[doc = "MB_32B_GROUP_MB8_32B_CS (rw) register accessor: Message Buffer 8 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb8_32b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb8_32b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb8_32b_cs`] module"]
#[doc(alias = "MB_32B_GROUP_MB8_32B_CS")]
pub type Mb32bGroupMb8_32bCs = crate::Reg<mb_32b_group_mb8_32b_cs::Mb32bGroupMb8_32bCsSpec>;
#[doc = "Message Buffer 8 CS Register"]
pub mod mb_32b_group_mb8_32b_cs;
#[doc = "MB_GROUP_ID20 (rw) register accessor: Message Buffer 20 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id20::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id20::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id20`] module"]
#[doc(alias = "MB_GROUP_ID20")]
pub type MbGroupId20 = crate::Reg<mb_group_id20::MbGroupId20Spec>;
#[doc = "Message Buffer 20 ID Register"]
pub mod mb_group_id20;
#[doc = "MB_16B_GROUP_MB13_16B_WORD1 (rw) register accessor: Message Buffer 13 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb13_16b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb13_16b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb13_16b_word1`] module"]
#[doc(alias = "MB_16B_GROUP_MB13_16B_WORD1")]
pub type Mb16bGroupMb13_16bWord1 =
    crate::Reg<mb_16b_group_mb13_16b_word1::Mb16bGroupMb13_16bWord1Spec>;
#[doc = "Message Buffer 13 WORD_16B Register"]
pub mod mb_16b_group_mb13_16b_word1;
#[doc = "MB_8B_GROUP_MB20_8B_ID (rw) register accessor: Message Buffer 20 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb20_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb20_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb20_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB20_8B_ID")]
pub type Mb8bGroupMb20_8bId = crate::Reg<mb_8b_group_mb20_8b_id::Mb8bGroupMb20_8bIdSpec>;
#[doc = "Message Buffer 20 ID Register"]
pub mod mb_8b_group_mb20_8b_id;
#[doc = "MB_64B_GROUP_MB4_64B_WORD7 (rw) register accessor: Message Buffer 4 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb4_64b_word7::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb4_64b_word7::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb4_64b_word7`] module"]
#[doc(alias = "MB_64B_GROUP_MB4_64B_WORD7")]
pub type Mb64bGroupMb4_64bWord7 =
    crate::Reg<mb_64b_group_mb4_64b_word7::Mb64bGroupMb4_64bWord7Spec>;
#[doc = "Message Buffer 4 WORD_64B Register"]
pub mod mb_64b_group_mb4_64b_word7;
#[doc = "MB_32B_GROUP_MB8_32B_ID (rw) register accessor: Message Buffer 8 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb8_32b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb8_32b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb8_32b_id`] module"]
#[doc(alias = "MB_32B_GROUP_MB8_32B_ID")]
pub type Mb32bGroupMb8_32bId = crate::Reg<mb_32b_group_mb8_32b_id::Mb32bGroupMb8_32bIdSpec>;
#[doc = "Message Buffer 8 ID Register"]
pub mod mb_32b_group_mb8_32b_id;
#[doc = "MB_16B_GROUP_MB13_16B_WORD2 (rw) register accessor: Message Buffer 13 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb13_16b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb13_16b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb13_16b_word2`] module"]
#[doc(alias = "MB_16B_GROUP_MB13_16B_WORD2")]
pub type Mb16bGroupMb13_16bWord2 =
    crate::Reg<mb_16b_group_mb13_16b_word2::Mb16bGroupMb13_16bWord2Spec>;
#[doc = "Message Buffer 13 WORD_16B Register"]
pub mod mb_16b_group_mb13_16b_word2;
#[doc = "MB_8B_GROUP_MB20_8B_WORD0 (rw) register accessor: Message Buffer 20 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb20_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb20_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb20_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB20_8B_WORD0")]
pub type Mb8bGroupMb20_8bWord0 = crate::Reg<mb_8b_group_mb20_8b_word0::Mb8bGroupMb20_8bWord0Spec>;
#[doc = "Message Buffer 20 WORD_8B Register"]
pub mod mb_8b_group_mb20_8b_word0;
#[doc = "MB_64B_GROUP_MB4_64B_WORD8 (rw) register accessor: Message Buffer 4 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb4_64b_word8::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb4_64b_word8::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb4_64b_word8`] module"]
#[doc(alias = "MB_64B_GROUP_MB4_64B_WORD8")]
pub type Mb64bGroupMb4_64bWord8 =
    crate::Reg<mb_64b_group_mb4_64b_word8::Mb64bGroupMb4_64bWord8Spec>;
#[doc = "Message Buffer 4 WORD_64B Register"]
pub mod mb_64b_group_mb4_64b_word8;
#[doc = "MB_32B_GROUP_MB8_32B_WORD0 (rw) register accessor: Message Buffer 8 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb8_32b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb8_32b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb8_32b_word0`] module"]
#[doc(alias = "MB_32B_GROUP_MB8_32B_WORD0")]
pub type Mb32bGroupMb8_32bWord0 =
    crate::Reg<mb_32b_group_mb8_32b_word0::Mb32bGroupMb8_32bWord0Spec>;
#[doc = "Message Buffer 8 WORD_32B Register"]
pub mod mb_32b_group_mb8_32b_word0;
#[doc = "MB_GROUP_WORD020 (rw) register accessor: Message Buffer 20 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word020::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word020::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word020`] module"]
#[doc(alias = "MB_GROUP_WORD020")]
pub type MbGroupWord020 = crate::Reg<mb_group_word020::MbGroupWord020Spec>;
#[doc = "Message Buffer 20 WORD0 Register"]
pub mod mb_group_word020;
#[doc = "MB_16B_GROUP_MB13_16B_WORD3 (rw) register accessor: Message Buffer 13 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb13_16b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb13_16b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb13_16b_word3`] module"]
#[doc(alias = "MB_16B_GROUP_MB13_16B_WORD3")]
pub type Mb16bGroupMb13_16bWord3 =
    crate::Reg<mb_16b_group_mb13_16b_word3::Mb16bGroupMb13_16bWord3Spec>;
#[doc = "Message Buffer 13 WORD_16B Register"]
pub mod mb_16b_group_mb13_16b_word3;
#[doc = "MB_8B_GROUP_MB20_8B_WORD1 (rw) register accessor: Message Buffer 20 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb20_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb20_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb20_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB20_8B_WORD1")]
pub type Mb8bGroupMb20_8bWord1 = crate::Reg<mb_8b_group_mb20_8b_word1::Mb8bGroupMb20_8bWord1Spec>;
#[doc = "Message Buffer 20 WORD_8B Register"]
pub mod mb_8b_group_mb20_8b_word1;
#[doc = "MB_64B_GROUP_MB4_64B_WORD9 (rw) register accessor: Message Buffer 4 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb4_64b_word9::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb4_64b_word9::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb4_64b_word9`] module"]
#[doc(alias = "MB_64B_GROUP_MB4_64B_WORD9")]
pub type Mb64bGroupMb4_64bWord9 =
    crate::Reg<mb_64b_group_mb4_64b_word9::Mb64bGroupMb4_64bWord9Spec>;
#[doc = "Message Buffer 4 WORD_64B Register"]
pub mod mb_64b_group_mb4_64b_word9;
#[doc = "MB_32B_GROUP_MB8_32B_WORD1 (rw) register accessor: Message Buffer 8 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb8_32b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb8_32b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb8_32b_word1`] module"]
#[doc(alias = "MB_32B_GROUP_MB8_32B_WORD1")]
pub type Mb32bGroupMb8_32bWord1 =
    crate::Reg<mb_32b_group_mb8_32b_word1::Mb32bGroupMb8_32bWord1Spec>;
#[doc = "Message Buffer 8 WORD_32B Register"]
pub mod mb_32b_group_mb8_32b_word1;
#[doc = "MB_GROUP_WORD120 (rw) register accessor: Message Buffer 20 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word120::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word120::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word120`] module"]
#[doc(alias = "MB_GROUP_WORD120")]
pub type MbGroupWord120 = crate::Reg<mb_group_word120::MbGroupWord120Spec>;
#[doc = "Message Buffer 20 WORD1 Register"]
pub mod mb_group_word120;
#[doc = "MB_GROUP_CS21 (rw) register accessor: Message Buffer 21 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs21::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs21::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs21`] module"]
#[doc(alias = "MB_GROUP_CS21")]
pub type MbGroupCs21 = crate::Reg<mb_group_cs21::MbGroupCs21Spec>;
#[doc = "Message Buffer 21 CS Register"]
pub mod mb_group_cs21;
#[doc = "MB_16B_GROUP_MB14_16B_CS (rw) register accessor: Message Buffer 14 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb14_16b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb14_16b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb14_16b_cs`] module"]
#[doc(alias = "MB_16B_GROUP_MB14_16B_CS")]
pub type Mb16bGroupMb14_16bCs = crate::Reg<mb_16b_group_mb14_16b_cs::Mb16bGroupMb14_16bCsSpec>;
#[doc = "Message Buffer 14 CS Register"]
pub mod mb_16b_group_mb14_16b_cs;
#[doc = "MB_8B_GROUP_MB21_8B_CS (rw) register accessor: Message Buffer 21 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb21_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb21_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb21_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB21_8B_CS")]
pub type Mb8bGroupMb21_8bCs = crate::Reg<mb_8b_group_mb21_8b_cs::Mb8bGroupMb21_8bCsSpec>;
#[doc = "Message Buffer 21 CS Register"]
pub mod mb_8b_group_mb21_8b_cs;
#[doc = "MB_64B_GROUP_MB4_64B_WORD10 (rw) register accessor: Message Buffer 4 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb4_64b_word10::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb4_64b_word10::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb4_64b_word10`] module"]
#[doc(alias = "MB_64B_GROUP_MB4_64B_WORD10")]
pub type Mb64bGroupMb4_64bWord10 =
    crate::Reg<mb_64b_group_mb4_64b_word10::Mb64bGroupMb4_64bWord10Spec>;
#[doc = "Message Buffer 4 WORD_64B Register"]
pub mod mb_64b_group_mb4_64b_word10;
#[doc = "MB_32B_GROUP_MB8_32B_WORD2 (rw) register accessor: Message Buffer 8 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb8_32b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb8_32b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb8_32b_word2`] module"]
#[doc(alias = "MB_32B_GROUP_MB8_32B_WORD2")]
pub type Mb32bGroupMb8_32bWord2 =
    crate::Reg<mb_32b_group_mb8_32b_word2::Mb32bGroupMb8_32bWord2Spec>;
#[doc = "Message Buffer 8 WORD_32B Register"]
pub mod mb_32b_group_mb8_32b_word2;
#[doc = "MB_GROUP_ID21 (rw) register accessor: Message Buffer 21 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id21::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id21::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id21`] module"]
#[doc(alias = "MB_GROUP_ID21")]
pub type MbGroupId21 = crate::Reg<mb_group_id21::MbGroupId21Spec>;
#[doc = "Message Buffer 21 ID Register"]
pub mod mb_group_id21;
#[doc = "MB_16B_GROUP_MB14_16B_ID (rw) register accessor: Message Buffer 14 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb14_16b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb14_16b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb14_16b_id`] module"]
#[doc(alias = "MB_16B_GROUP_MB14_16B_ID")]
pub type Mb16bGroupMb14_16bId = crate::Reg<mb_16b_group_mb14_16b_id::Mb16bGroupMb14_16bIdSpec>;
#[doc = "Message Buffer 14 ID Register"]
pub mod mb_16b_group_mb14_16b_id;
#[doc = "MB_8B_GROUP_MB21_8B_ID (rw) register accessor: Message Buffer 21 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb21_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb21_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb21_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB21_8B_ID")]
pub type Mb8bGroupMb21_8bId = crate::Reg<mb_8b_group_mb21_8b_id::Mb8bGroupMb21_8bIdSpec>;
#[doc = "Message Buffer 21 ID Register"]
pub mod mb_8b_group_mb21_8b_id;
#[doc = "MB_64B_GROUP_MB4_64B_WORD11 (rw) register accessor: Message Buffer 4 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb4_64b_word11::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb4_64b_word11::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb4_64b_word11`] module"]
#[doc(alias = "MB_64B_GROUP_MB4_64B_WORD11")]
pub type Mb64bGroupMb4_64bWord11 =
    crate::Reg<mb_64b_group_mb4_64b_word11::Mb64bGroupMb4_64bWord11Spec>;
#[doc = "Message Buffer 4 WORD_64B Register"]
pub mod mb_64b_group_mb4_64b_word11;
#[doc = "MB_32B_GROUP_MB8_32B_WORD3 (rw) register accessor: Message Buffer 8 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb8_32b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb8_32b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb8_32b_word3`] module"]
#[doc(alias = "MB_32B_GROUP_MB8_32B_WORD3")]
pub type Mb32bGroupMb8_32bWord3 =
    crate::Reg<mb_32b_group_mb8_32b_word3::Mb32bGroupMb8_32bWord3Spec>;
#[doc = "Message Buffer 8 WORD_32B Register"]
pub mod mb_32b_group_mb8_32b_word3;
#[doc = "MB_16B_GROUP_MB14_16B_WORD0 (rw) register accessor: Message Buffer 14 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb14_16b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb14_16b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb14_16b_word0`] module"]
#[doc(alias = "MB_16B_GROUP_MB14_16B_WORD0")]
pub type Mb16bGroupMb14_16bWord0 =
    crate::Reg<mb_16b_group_mb14_16b_word0::Mb16bGroupMb14_16bWord0Spec>;
#[doc = "Message Buffer 14 WORD_16B Register"]
pub mod mb_16b_group_mb14_16b_word0;
#[doc = "MB_8B_GROUP_MB21_8B_WORD0 (rw) register accessor: Message Buffer 21 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb21_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb21_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb21_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB21_8B_WORD0")]
pub type Mb8bGroupMb21_8bWord0 = crate::Reg<mb_8b_group_mb21_8b_word0::Mb8bGroupMb21_8bWord0Spec>;
#[doc = "Message Buffer 21 WORD_8B Register"]
pub mod mb_8b_group_mb21_8b_word0;
#[doc = "MB_64B_GROUP_MB4_64B_WORD12 (rw) register accessor: Message Buffer 4 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb4_64b_word12::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb4_64b_word12::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb4_64b_word12`] module"]
#[doc(alias = "MB_64B_GROUP_MB4_64B_WORD12")]
pub type Mb64bGroupMb4_64bWord12 =
    crate::Reg<mb_64b_group_mb4_64b_word12::Mb64bGroupMb4_64bWord12Spec>;
#[doc = "Message Buffer 4 WORD_64B Register"]
pub mod mb_64b_group_mb4_64b_word12;
#[doc = "MB_32B_GROUP_MB8_32B_WORD4 (rw) register accessor: Message Buffer 8 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb8_32b_word4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb8_32b_word4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb8_32b_word4`] module"]
#[doc(alias = "MB_32B_GROUP_MB8_32B_WORD4")]
pub type Mb32bGroupMb8_32bWord4 =
    crate::Reg<mb_32b_group_mb8_32b_word4::Mb32bGroupMb8_32bWord4Spec>;
#[doc = "Message Buffer 8 WORD_32B Register"]
pub mod mb_32b_group_mb8_32b_word4;
#[doc = "MB_GROUP_WORD021 (rw) register accessor: Message Buffer 21 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word021::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word021::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word021`] module"]
#[doc(alias = "MB_GROUP_WORD021")]
pub type MbGroupWord021 = crate::Reg<mb_group_word021::MbGroupWord021Spec>;
#[doc = "Message Buffer 21 WORD0 Register"]
pub mod mb_group_word021;
#[doc = "MB_16B_GROUP_MB14_16B_WORD1 (rw) register accessor: Message Buffer 14 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb14_16b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb14_16b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb14_16b_word1`] module"]
#[doc(alias = "MB_16B_GROUP_MB14_16B_WORD1")]
pub type Mb16bGroupMb14_16bWord1 =
    crate::Reg<mb_16b_group_mb14_16b_word1::Mb16bGroupMb14_16bWord1Spec>;
#[doc = "Message Buffer 14 WORD_16B Register"]
pub mod mb_16b_group_mb14_16b_word1;
#[doc = "MB_8B_GROUP_MB21_8B_WORD1 (rw) register accessor: Message Buffer 21 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb21_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb21_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb21_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB21_8B_WORD1")]
pub type Mb8bGroupMb21_8bWord1 = crate::Reg<mb_8b_group_mb21_8b_word1::Mb8bGroupMb21_8bWord1Spec>;
#[doc = "Message Buffer 21 WORD_8B Register"]
pub mod mb_8b_group_mb21_8b_word1;
#[doc = "MB_64B_GROUP_MB4_64B_WORD13 (rw) register accessor: Message Buffer 4 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb4_64b_word13::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb4_64b_word13::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb4_64b_word13`] module"]
#[doc(alias = "MB_64B_GROUP_MB4_64B_WORD13")]
pub type Mb64bGroupMb4_64bWord13 =
    crate::Reg<mb_64b_group_mb4_64b_word13::Mb64bGroupMb4_64bWord13Spec>;
#[doc = "Message Buffer 4 WORD_64B Register"]
pub mod mb_64b_group_mb4_64b_word13;
#[doc = "MB_32B_GROUP_MB8_32B_WORD5 (rw) register accessor: Message Buffer 8 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb8_32b_word5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb8_32b_word5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb8_32b_word5`] module"]
#[doc(alias = "MB_32B_GROUP_MB8_32B_WORD5")]
pub type Mb32bGroupMb8_32bWord5 =
    crate::Reg<mb_32b_group_mb8_32b_word5::Mb32bGroupMb8_32bWord5Spec>;
#[doc = "Message Buffer 8 WORD_32B Register"]
pub mod mb_32b_group_mb8_32b_word5;
#[doc = "MB_GROUP_WORD121 (rw) register accessor: Message Buffer 21 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word121::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word121::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word121`] module"]
#[doc(alias = "MB_GROUP_WORD121")]
pub type MbGroupWord121 = crate::Reg<mb_group_word121::MbGroupWord121Spec>;
#[doc = "Message Buffer 21 WORD1 Register"]
pub mod mb_group_word121;
#[doc = "MB_GROUP_CS22 (rw) register accessor: Message Buffer 22 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs22::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs22::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs22`] module"]
#[doc(alias = "MB_GROUP_CS22")]
pub type MbGroupCs22 = crate::Reg<mb_group_cs22::MbGroupCs22Spec>;
#[doc = "Message Buffer 22 CS Register"]
pub mod mb_group_cs22;
#[doc = "MB_16B_GROUP_MB14_16B_WORD2 (rw) register accessor: Message Buffer 14 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb14_16b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb14_16b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb14_16b_word2`] module"]
#[doc(alias = "MB_16B_GROUP_MB14_16B_WORD2")]
pub type Mb16bGroupMb14_16bWord2 =
    crate::Reg<mb_16b_group_mb14_16b_word2::Mb16bGroupMb14_16bWord2Spec>;
#[doc = "Message Buffer 14 WORD_16B Register"]
pub mod mb_16b_group_mb14_16b_word2;
#[doc = "MB_8B_GROUP_MB22_8B_CS (rw) register accessor: Message Buffer 22 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb22_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb22_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb22_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB22_8B_CS")]
pub type Mb8bGroupMb22_8bCs = crate::Reg<mb_8b_group_mb22_8b_cs::Mb8bGroupMb22_8bCsSpec>;
#[doc = "Message Buffer 22 CS Register"]
pub mod mb_8b_group_mb22_8b_cs;
#[doc = "MB_64B_GROUP_MB4_64B_WORD14 (rw) register accessor: Message Buffer 4 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb4_64b_word14::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb4_64b_word14::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb4_64b_word14`] module"]
#[doc(alias = "MB_64B_GROUP_MB4_64B_WORD14")]
pub type Mb64bGroupMb4_64bWord14 =
    crate::Reg<mb_64b_group_mb4_64b_word14::Mb64bGroupMb4_64bWord14Spec>;
#[doc = "Message Buffer 4 WORD_64B Register"]
pub mod mb_64b_group_mb4_64b_word14;
#[doc = "MB_32B_GROUP_MB8_32B_WORD6 (rw) register accessor: Message Buffer 8 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb8_32b_word6::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb8_32b_word6::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb8_32b_word6`] module"]
#[doc(alias = "MB_32B_GROUP_MB8_32B_WORD6")]
pub type Mb32bGroupMb8_32bWord6 =
    crate::Reg<mb_32b_group_mb8_32b_word6::Mb32bGroupMb8_32bWord6Spec>;
#[doc = "Message Buffer 8 WORD_32B Register"]
pub mod mb_32b_group_mb8_32b_word6;
#[doc = "MB_GROUP_ID22 (rw) register accessor: Message Buffer 22 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id22::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id22::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id22`] module"]
#[doc(alias = "MB_GROUP_ID22")]
pub type MbGroupId22 = crate::Reg<mb_group_id22::MbGroupId22Spec>;
#[doc = "Message Buffer 22 ID Register"]
pub mod mb_group_id22;
#[doc = "MB_16B_GROUP_MB14_16B_WORD3 (rw) register accessor: Message Buffer 14 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb14_16b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb14_16b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb14_16b_word3`] module"]
#[doc(alias = "MB_16B_GROUP_MB14_16B_WORD3")]
pub type Mb16bGroupMb14_16bWord3 =
    crate::Reg<mb_16b_group_mb14_16b_word3::Mb16bGroupMb14_16bWord3Spec>;
#[doc = "Message Buffer 14 WORD_16B Register"]
pub mod mb_16b_group_mb14_16b_word3;
#[doc = "MB_8B_GROUP_MB22_8B_ID (rw) register accessor: Message Buffer 22 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb22_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb22_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb22_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB22_8B_ID")]
pub type Mb8bGroupMb22_8bId = crate::Reg<mb_8b_group_mb22_8b_id::Mb8bGroupMb22_8bIdSpec>;
#[doc = "Message Buffer 22 ID Register"]
pub mod mb_8b_group_mb22_8b_id;
#[doc = "MB_64B_GROUP_MB4_64B_WORD15 (rw) register accessor: Message Buffer 4 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb4_64b_word15::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb4_64b_word15::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb4_64b_word15`] module"]
#[doc(alias = "MB_64B_GROUP_MB4_64B_WORD15")]
pub type Mb64bGroupMb4_64bWord15 =
    crate::Reg<mb_64b_group_mb4_64b_word15::Mb64bGroupMb4_64bWord15Spec>;
#[doc = "Message Buffer 4 WORD_64B Register"]
pub mod mb_64b_group_mb4_64b_word15;
#[doc = "MB_32B_GROUP_MB8_32B_WORD7 (rw) register accessor: Message Buffer 8 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb8_32b_word7::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb8_32b_word7::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb8_32b_word7`] module"]
#[doc(alias = "MB_32B_GROUP_MB8_32B_WORD7")]
pub type Mb32bGroupMb8_32bWord7 =
    crate::Reg<mb_32b_group_mb8_32b_word7::Mb32bGroupMb8_32bWord7Spec>;
#[doc = "Message Buffer 8 WORD_32B Register"]
pub mod mb_32b_group_mb8_32b_word7;
#[doc = "MB_16B_GROUP_MB15_16B_CS (rw) register accessor: Message Buffer 15 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb15_16b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb15_16b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb15_16b_cs`] module"]
#[doc(alias = "MB_16B_GROUP_MB15_16B_CS")]
pub type Mb16bGroupMb15_16bCs = crate::Reg<mb_16b_group_mb15_16b_cs::Mb16bGroupMb15_16bCsSpec>;
#[doc = "Message Buffer 15 CS Register"]
pub mod mb_16b_group_mb15_16b_cs;
#[doc = "MB_8B_GROUP_MB22_8B_WORD0 (rw) register accessor: Message Buffer 22 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb22_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb22_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb22_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB22_8B_WORD0")]
pub type Mb8bGroupMb22_8bWord0 = crate::Reg<mb_8b_group_mb22_8b_word0::Mb8bGroupMb22_8bWord0Spec>;
#[doc = "Message Buffer 22 WORD_8B Register"]
pub mod mb_8b_group_mb22_8b_word0;
#[doc = "MB_64B_GROUP_MB5_64B_CS (rw) register accessor: Message Buffer 5 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb5_64b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb5_64b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb5_64b_cs`] module"]
#[doc(alias = "MB_64B_GROUP_MB5_64B_CS")]
pub type Mb64bGroupMb5_64bCs = crate::Reg<mb_64b_group_mb5_64b_cs::Mb64bGroupMb5_64bCsSpec>;
#[doc = "Message Buffer 5 CS Register"]
pub mod mb_64b_group_mb5_64b_cs;
#[doc = "MB_32B_GROUP_MB9_32B_CS (rw) register accessor: Message Buffer 9 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb9_32b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb9_32b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb9_32b_cs`] module"]
#[doc(alias = "MB_32B_GROUP_MB9_32B_CS")]
pub type Mb32bGroupMb9_32bCs = crate::Reg<mb_32b_group_mb9_32b_cs::Mb32bGroupMb9_32bCsSpec>;
#[doc = "Message Buffer 9 CS Register"]
pub mod mb_32b_group_mb9_32b_cs;
#[doc = "MB_GROUP_WORD022 (rw) register accessor: Message Buffer 22 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word022::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word022::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word022`] module"]
#[doc(alias = "MB_GROUP_WORD022")]
pub type MbGroupWord022 = crate::Reg<mb_group_word022::MbGroupWord022Spec>;
#[doc = "Message Buffer 22 WORD0 Register"]
pub mod mb_group_word022;
#[doc = "MB_16B_GROUP_MB15_16B_ID (rw) register accessor: Message Buffer 15 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb15_16b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb15_16b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb15_16b_id`] module"]
#[doc(alias = "MB_16B_GROUP_MB15_16B_ID")]
pub type Mb16bGroupMb15_16bId = crate::Reg<mb_16b_group_mb15_16b_id::Mb16bGroupMb15_16bIdSpec>;
#[doc = "Message Buffer 15 ID Register"]
pub mod mb_16b_group_mb15_16b_id;
#[doc = "MB_8B_GROUP_MB22_8B_WORD1 (rw) register accessor: Message Buffer 22 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb22_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb22_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb22_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB22_8B_WORD1")]
pub type Mb8bGroupMb22_8bWord1 = crate::Reg<mb_8b_group_mb22_8b_word1::Mb8bGroupMb22_8bWord1Spec>;
#[doc = "Message Buffer 22 WORD_8B Register"]
pub mod mb_8b_group_mb22_8b_word1;
#[doc = "MB_64B_GROUP_MB5_64B_ID (rw) register accessor: Message Buffer 5 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb5_64b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb5_64b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb5_64b_id`] module"]
#[doc(alias = "MB_64B_GROUP_MB5_64B_ID")]
pub type Mb64bGroupMb5_64bId = crate::Reg<mb_64b_group_mb5_64b_id::Mb64bGroupMb5_64bIdSpec>;
#[doc = "Message Buffer 5 ID Register"]
pub mod mb_64b_group_mb5_64b_id;
#[doc = "MB_32B_GROUP_MB9_32B_ID (rw) register accessor: Message Buffer 9 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb9_32b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb9_32b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb9_32b_id`] module"]
#[doc(alias = "MB_32B_GROUP_MB9_32B_ID")]
pub type Mb32bGroupMb9_32bId = crate::Reg<mb_32b_group_mb9_32b_id::Mb32bGroupMb9_32bIdSpec>;
#[doc = "Message Buffer 9 ID Register"]
pub mod mb_32b_group_mb9_32b_id;
#[doc = "MB_GROUP_WORD122 (rw) register accessor: Message Buffer 22 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word122::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word122::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word122`] module"]
#[doc(alias = "MB_GROUP_WORD122")]
pub type MbGroupWord122 = crate::Reg<mb_group_word122::MbGroupWord122Spec>;
#[doc = "Message Buffer 22 WORD1 Register"]
pub mod mb_group_word122;
#[doc = "MB_GROUP_CS23 (rw) register accessor: Message Buffer 23 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs23::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs23::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs23`] module"]
#[doc(alias = "MB_GROUP_CS23")]
pub type MbGroupCs23 = crate::Reg<mb_group_cs23::MbGroupCs23Spec>;
#[doc = "Message Buffer 23 CS Register"]
pub mod mb_group_cs23;
#[doc = "MB_16B_GROUP_MB15_16B_WORD0 (rw) register accessor: Message Buffer 15 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb15_16b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb15_16b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb15_16b_word0`] module"]
#[doc(alias = "MB_16B_GROUP_MB15_16B_WORD0")]
pub type Mb16bGroupMb15_16bWord0 =
    crate::Reg<mb_16b_group_mb15_16b_word0::Mb16bGroupMb15_16bWord0Spec>;
#[doc = "Message Buffer 15 WORD_16B Register"]
pub mod mb_16b_group_mb15_16b_word0;
#[doc = "MB_8B_GROUP_MB23_8B_CS (rw) register accessor: Message Buffer 23 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb23_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb23_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb23_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB23_8B_CS")]
pub type Mb8bGroupMb23_8bCs = crate::Reg<mb_8b_group_mb23_8b_cs::Mb8bGroupMb23_8bCsSpec>;
#[doc = "Message Buffer 23 CS Register"]
pub mod mb_8b_group_mb23_8b_cs;
#[doc = "MB_64B_GROUP_MB5_64B_WORD0 (rw) register accessor: Message Buffer 5 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb5_64b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb5_64b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb5_64b_word0`] module"]
#[doc(alias = "MB_64B_GROUP_MB5_64B_WORD0")]
pub type Mb64bGroupMb5_64bWord0 =
    crate::Reg<mb_64b_group_mb5_64b_word0::Mb64bGroupMb5_64bWord0Spec>;
#[doc = "Message Buffer 5 WORD_64B Register"]
pub mod mb_64b_group_mb5_64b_word0;
#[doc = "MB_32B_GROUP_MB9_32B_WORD0 (rw) register accessor: Message Buffer 9 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb9_32b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb9_32b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb9_32b_word0`] module"]
#[doc(alias = "MB_32B_GROUP_MB9_32B_WORD0")]
pub type Mb32bGroupMb9_32bWord0 =
    crate::Reg<mb_32b_group_mb9_32b_word0::Mb32bGroupMb9_32bWord0Spec>;
#[doc = "Message Buffer 9 WORD_32B Register"]
pub mod mb_32b_group_mb9_32b_word0;
#[doc = "MB_GROUP_ID23 (rw) register accessor: Message Buffer 23 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id23::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id23::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id23`] module"]
#[doc(alias = "MB_GROUP_ID23")]
pub type MbGroupId23 = crate::Reg<mb_group_id23::MbGroupId23Spec>;
#[doc = "Message Buffer 23 ID Register"]
pub mod mb_group_id23;
#[doc = "MB_16B_GROUP_MB15_16B_WORD1 (rw) register accessor: Message Buffer 15 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb15_16b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb15_16b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb15_16b_word1`] module"]
#[doc(alias = "MB_16B_GROUP_MB15_16B_WORD1")]
pub type Mb16bGroupMb15_16bWord1 =
    crate::Reg<mb_16b_group_mb15_16b_word1::Mb16bGroupMb15_16bWord1Spec>;
#[doc = "Message Buffer 15 WORD_16B Register"]
pub mod mb_16b_group_mb15_16b_word1;
#[doc = "MB_8B_GROUP_MB23_8B_ID (rw) register accessor: Message Buffer 23 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb23_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb23_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb23_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB23_8B_ID")]
pub type Mb8bGroupMb23_8bId = crate::Reg<mb_8b_group_mb23_8b_id::Mb8bGroupMb23_8bIdSpec>;
#[doc = "Message Buffer 23 ID Register"]
pub mod mb_8b_group_mb23_8b_id;
#[doc = "MB_64B_GROUP_MB5_64B_WORD1 (rw) register accessor: Message Buffer 5 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb5_64b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb5_64b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb5_64b_word1`] module"]
#[doc(alias = "MB_64B_GROUP_MB5_64B_WORD1")]
pub type Mb64bGroupMb5_64bWord1 =
    crate::Reg<mb_64b_group_mb5_64b_word1::Mb64bGroupMb5_64bWord1Spec>;
#[doc = "Message Buffer 5 WORD_64B Register"]
pub mod mb_64b_group_mb5_64b_word1;
#[doc = "MB_32B_GROUP_MB9_32B_WORD1 (rw) register accessor: Message Buffer 9 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb9_32b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb9_32b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb9_32b_word1`] module"]
#[doc(alias = "MB_32B_GROUP_MB9_32B_WORD1")]
pub type Mb32bGroupMb9_32bWord1 =
    crate::Reg<mb_32b_group_mb9_32b_word1::Mb32bGroupMb9_32bWord1Spec>;
#[doc = "Message Buffer 9 WORD_32B Register"]
pub mod mb_32b_group_mb9_32b_word1;
#[doc = "MB_16B_GROUP_MB15_16B_WORD2 (rw) register accessor: Message Buffer 15 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb15_16b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb15_16b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb15_16b_word2`] module"]
#[doc(alias = "MB_16B_GROUP_MB15_16B_WORD2")]
pub type Mb16bGroupMb15_16bWord2 =
    crate::Reg<mb_16b_group_mb15_16b_word2::Mb16bGroupMb15_16bWord2Spec>;
#[doc = "Message Buffer 15 WORD_16B Register"]
pub mod mb_16b_group_mb15_16b_word2;
#[doc = "MB_8B_GROUP_MB23_8B_WORD0 (rw) register accessor: Message Buffer 23 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb23_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb23_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb23_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB23_8B_WORD0")]
pub type Mb8bGroupMb23_8bWord0 = crate::Reg<mb_8b_group_mb23_8b_word0::Mb8bGroupMb23_8bWord0Spec>;
#[doc = "Message Buffer 23 WORD_8B Register"]
pub mod mb_8b_group_mb23_8b_word0;
#[doc = "MB_64B_GROUP_MB5_64B_WORD2 (rw) register accessor: Message Buffer 5 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb5_64b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb5_64b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb5_64b_word2`] module"]
#[doc(alias = "MB_64B_GROUP_MB5_64B_WORD2")]
pub type Mb64bGroupMb5_64bWord2 =
    crate::Reg<mb_64b_group_mb5_64b_word2::Mb64bGroupMb5_64bWord2Spec>;
#[doc = "Message Buffer 5 WORD_64B Register"]
pub mod mb_64b_group_mb5_64b_word2;
#[doc = "MB_32B_GROUP_MB9_32B_WORD2 (rw) register accessor: Message Buffer 9 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb9_32b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb9_32b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb9_32b_word2`] module"]
#[doc(alias = "MB_32B_GROUP_MB9_32B_WORD2")]
pub type Mb32bGroupMb9_32bWord2 =
    crate::Reg<mb_32b_group_mb9_32b_word2::Mb32bGroupMb9_32bWord2Spec>;
#[doc = "Message Buffer 9 WORD_32B Register"]
pub mod mb_32b_group_mb9_32b_word2;
#[doc = "MB_GROUP_WORD023 (rw) register accessor: Message Buffer 23 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word023::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word023::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word023`] module"]
#[doc(alias = "MB_GROUP_WORD023")]
pub type MbGroupWord023 = crate::Reg<mb_group_word023::MbGroupWord023Spec>;
#[doc = "Message Buffer 23 WORD0 Register"]
pub mod mb_group_word023;
#[doc = "MB_16B_GROUP_MB15_16B_WORD3 (rw) register accessor: Message Buffer 15 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb15_16b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb15_16b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb15_16b_word3`] module"]
#[doc(alias = "MB_16B_GROUP_MB15_16B_WORD3")]
pub type Mb16bGroupMb15_16bWord3 =
    crate::Reg<mb_16b_group_mb15_16b_word3::Mb16bGroupMb15_16bWord3Spec>;
#[doc = "Message Buffer 15 WORD_16B Register"]
pub mod mb_16b_group_mb15_16b_word3;
#[doc = "MB_8B_GROUP_MB23_8B_WORD1 (rw) register accessor: Message Buffer 23 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb23_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb23_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb23_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB23_8B_WORD1")]
pub type Mb8bGroupMb23_8bWord1 = crate::Reg<mb_8b_group_mb23_8b_word1::Mb8bGroupMb23_8bWord1Spec>;
#[doc = "Message Buffer 23 WORD_8B Register"]
pub mod mb_8b_group_mb23_8b_word1;
#[doc = "MB_64B_GROUP_MB5_64B_WORD3 (rw) register accessor: Message Buffer 5 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb5_64b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb5_64b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb5_64b_word3`] module"]
#[doc(alias = "MB_64B_GROUP_MB5_64B_WORD3")]
pub type Mb64bGroupMb5_64bWord3 =
    crate::Reg<mb_64b_group_mb5_64b_word3::Mb64bGroupMb5_64bWord3Spec>;
#[doc = "Message Buffer 5 WORD_64B Register"]
pub mod mb_64b_group_mb5_64b_word3;
#[doc = "MB_32B_GROUP_MB9_32B_WORD3 (rw) register accessor: Message Buffer 9 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb9_32b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb9_32b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb9_32b_word3`] module"]
#[doc(alias = "MB_32B_GROUP_MB9_32B_WORD3")]
pub type Mb32bGroupMb9_32bWord3 =
    crate::Reg<mb_32b_group_mb9_32b_word3::Mb32bGroupMb9_32bWord3Spec>;
#[doc = "Message Buffer 9 WORD_32B Register"]
pub mod mb_32b_group_mb9_32b_word3;
#[doc = "MB_GROUP_WORD123 (rw) register accessor: Message Buffer 23 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word123::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word123::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word123`] module"]
#[doc(alias = "MB_GROUP_WORD123")]
pub type MbGroupWord123 = crate::Reg<mb_group_word123::MbGroupWord123Spec>;
#[doc = "Message Buffer 23 WORD1 Register"]
pub mod mb_group_word123;
#[doc = "MB_GROUP_CS24 (rw) register accessor: Message Buffer 24 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs24::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs24::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs24`] module"]
#[doc(alias = "MB_GROUP_CS24")]
pub type MbGroupCs24 = crate::Reg<mb_group_cs24::MbGroupCs24Spec>;
#[doc = "Message Buffer 24 CS Register"]
pub mod mb_group_cs24;
#[doc = "MB_16B_GROUP_MB16_16B_CS (rw) register accessor: Message Buffer 16 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb16_16b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb16_16b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb16_16b_cs`] module"]
#[doc(alias = "MB_16B_GROUP_MB16_16B_CS")]
pub type Mb16bGroupMb16_16bCs = crate::Reg<mb_16b_group_mb16_16b_cs::Mb16bGroupMb16_16bCsSpec>;
#[doc = "Message Buffer 16 CS Register"]
pub mod mb_16b_group_mb16_16b_cs;
#[doc = "MB_8B_GROUP_MB24_8B_CS (rw) register accessor: Message Buffer 24 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb24_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb24_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb24_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB24_8B_CS")]
pub type Mb8bGroupMb24_8bCs = crate::Reg<mb_8b_group_mb24_8b_cs::Mb8bGroupMb24_8bCsSpec>;
#[doc = "Message Buffer 24 CS Register"]
pub mod mb_8b_group_mb24_8b_cs;
#[doc = "MB_64B_GROUP_MB5_64B_WORD4 (rw) register accessor: Message Buffer 5 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb5_64b_word4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb5_64b_word4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb5_64b_word4`] module"]
#[doc(alias = "MB_64B_GROUP_MB5_64B_WORD4")]
pub type Mb64bGroupMb5_64bWord4 =
    crate::Reg<mb_64b_group_mb5_64b_word4::Mb64bGroupMb5_64bWord4Spec>;
#[doc = "Message Buffer 5 WORD_64B Register"]
pub mod mb_64b_group_mb5_64b_word4;
#[doc = "MB_32B_GROUP_MB9_32B_WORD4 (rw) register accessor: Message Buffer 9 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb9_32b_word4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb9_32b_word4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb9_32b_word4`] module"]
#[doc(alias = "MB_32B_GROUP_MB9_32B_WORD4")]
pub type Mb32bGroupMb9_32bWord4 =
    crate::Reg<mb_32b_group_mb9_32b_word4::Mb32bGroupMb9_32bWord4Spec>;
#[doc = "Message Buffer 9 WORD_32B Register"]
pub mod mb_32b_group_mb9_32b_word4;
#[doc = "MB_GROUP_ID24 (rw) register accessor: Message Buffer 24 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id24::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id24::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id24`] module"]
#[doc(alias = "MB_GROUP_ID24")]
pub type MbGroupId24 = crate::Reg<mb_group_id24::MbGroupId24Spec>;
#[doc = "Message Buffer 24 ID Register"]
pub mod mb_group_id24;
#[doc = "MB_16B_GROUP_MB16_16B_ID (rw) register accessor: Message Buffer 16 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb16_16b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb16_16b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb16_16b_id`] module"]
#[doc(alias = "MB_16B_GROUP_MB16_16B_ID")]
pub type Mb16bGroupMb16_16bId = crate::Reg<mb_16b_group_mb16_16b_id::Mb16bGroupMb16_16bIdSpec>;
#[doc = "Message Buffer 16 ID Register"]
pub mod mb_16b_group_mb16_16b_id;
#[doc = "MB_8B_GROUP_MB24_8B_ID (rw) register accessor: Message Buffer 24 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb24_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb24_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb24_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB24_8B_ID")]
pub type Mb8bGroupMb24_8bId = crate::Reg<mb_8b_group_mb24_8b_id::Mb8bGroupMb24_8bIdSpec>;
#[doc = "Message Buffer 24 ID Register"]
pub mod mb_8b_group_mb24_8b_id;
#[doc = "MB_64B_GROUP_MB5_64B_WORD5 (rw) register accessor: Message Buffer 5 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb5_64b_word5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb5_64b_word5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb5_64b_word5`] module"]
#[doc(alias = "MB_64B_GROUP_MB5_64B_WORD5")]
pub type Mb64bGroupMb5_64bWord5 =
    crate::Reg<mb_64b_group_mb5_64b_word5::Mb64bGroupMb5_64bWord5Spec>;
#[doc = "Message Buffer 5 WORD_64B Register"]
pub mod mb_64b_group_mb5_64b_word5;
#[doc = "MB_32B_GROUP_MB9_32B_WORD5 (rw) register accessor: Message Buffer 9 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb9_32b_word5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb9_32b_word5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb9_32b_word5`] module"]
#[doc(alias = "MB_32B_GROUP_MB9_32B_WORD5")]
pub type Mb32bGroupMb9_32bWord5 =
    crate::Reg<mb_32b_group_mb9_32b_word5::Mb32bGroupMb9_32bWord5Spec>;
#[doc = "Message Buffer 9 WORD_32B Register"]
pub mod mb_32b_group_mb9_32b_word5;
#[doc = "MB_16B_GROUP_MB16_16B_WORD0 (rw) register accessor: Message Buffer 16 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb16_16b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb16_16b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb16_16b_word0`] module"]
#[doc(alias = "MB_16B_GROUP_MB16_16B_WORD0")]
pub type Mb16bGroupMb16_16bWord0 =
    crate::Reg<mb_16b_group_mb16_16b_word0::Mb16bGroupMb16_16bWord0Spec>;
#[doc = "Message Buffer 16 WORD_16B Register"]
pub mod mb_16b_group_mb16_16b_word0;
#[doc = "MB_8B_GROUP_MB24_8B_WORD0 (rw) register accessor: Message Buffer 24 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb24_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb24_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb24_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB24_8B_WORD0")]
pub type Mb8bGroupMb24_8bWord0 = crate::Reg<mb_8b_group_mb24_8b_word0::Mb8bGroupMb24_8bWord0Spec>;
#[doc = "Message Buffer 24 WORD_8B Register"]
pub mod mb_8b_group_mb24_8b_word0;
#[doc = "MB_64B_GROUP_MB5_64B_WORD6 (rw) register accessor: Message Buffer 5 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb5_64b_word6::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb5_64b_word6::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb5_64b_word6`] module"]
#[doc(alias = "MB_64B_GROUP_MB5_64B_WORD6")]
pub type Mb64bGroupMb5_64bWord6 =
    crate::Reg<mb_64b_group_mb5_64b_word6::Mb64bGroupMb5_64bWord6Spec>;
#[doc = "Message Buffer 5 WORD_64B Register"]
pub mod mb_64b_group_mb5_64b_word6;
#[doc = "MB_32B_GROUP_MB9_32B_WORD6 (rw) register accessor: Message Buffer 9 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb9_32b_word6::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb9_32b_word6::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb9_32b_word6`] module"]
#[doc(alias = "MB_32B_GROUP_MB9_32B_WORD6")]
pub type Mb32bGroupMb9_32bWord6 =
    crate::Reg<mb_32b_group_mb9_32b_word6::Mb32bGroupMb9_32bWord6Spec>;
#[doc = "Message Buffer 9 WORD_32B Register"]
pub mod mb_32b_group_mb9_32b_word6;
#[doc = "MB_GROUP_WORD024 (rw) register accessor: Message Buffer 24 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word024::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word024::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word024`] module"]
#[doc(alias = "MB_GROUP_WORD024")]
pub type MbGroupWord024 = crate::Reg<mb_group_word024::MbGroupWord024Spec>;
#[doc = "Message Buffer 24 WORD0 Register"]
pub mod mb_group_word024;
#[doc = "MB_16B_GROUP_MB16_16B_WORD1 (rw) register accessor: Message Buffer 16 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb16_16b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb16_16b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb16_16b_word1`] module"]
#[doc(alias = "MB_16B_GROUP_MB16_16B_WORD1")]
pub type Mb16bGroupMb16_16bWord1 =
    crate::Reg<mb_16b_group_mb16_16b_word1::Mb16bGroupMb16_16bWord1Spec>;
#[doc = "Message Buffer 16 WORD_16B Register"]
pub mod mb_16b_group_mb16_16b_word1;
#[doc = "MB_8B_GROUP_MB24_8B_WORD1 (rw) register accessor: Message Buffer 24 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb24_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb24_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb24_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB24_8B_WORD1")]
pub type Mb8bGroupMb24_8bWord1 = crate::Reg<mb_8b_group_mb24_8b_word1::Mb8bGroupMb24_8bWord1Spec>;
#[doc = "Message Buffer 24 WORD_8B Register"]
pub mod mb_8b_group_mb24_8b_word1;
#[doc = "MB_64B_GROUP_MB5_64B_WORD7 (rw) register accessor: Message Buffer 5 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb5_64b_word7::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb5_64b_word7::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb5_64b_word7`] module"]
#[doc(alias = "MB_64B_GROUP_MB5_64B_WORD7")]
pub type Mb64bGroupMb5_64bWord7 =
    crate::Reg<mb_64b_group_mb5_64b_word7::Mb64bGroupMb5_64bWord7Spec>;
#[doc = "Message Buffer 5 WORD_64B Register"]
pub mod mb_64b_group_mb5_64b_word7;
#[doc = "MB_32B_GROUP_MB9_32B_WORD7 (rw) register accessor: Message Buffer 9 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb9_32b_word7::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb9_32b_word7::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb9_32b_word7`] module"]
#[doc(alias = "MB_32B_GROUP_MB9_32B_WORD7")]
pub type Mb32bGroupMb9_32bWord7 =
    crate::Reg<mb_32b_group_mb9_32b_word7::Mb32bGroupMb9_32bWord7Spec>;
#[doc = "Message Buffer 9 WORD_32B Register"]
pub mod mb_32b_group_mb9_32b_word7;
#[doc = "MB_GROUP_WORD124 (rw) register accessor: Message Buffer 24 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word124::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word124::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word124`] module"]
#[doc(alias = "MB_GROUP_WORD124")]
pub type MbGroupWord124 = crate::Reg<mb_group_word124::MbGroupWord124Spec>;
#[doc = "Message Buffer 24 WORD1 Register"]
pub mod mb_group_word124;
#[doc = "MB_GROUP_CS25 (rw) register accessor: Message Buffer 25 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs25::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs25::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs25`] module"]
#[doc(alias = "MB_GROUP_CS25")]
pub type MbGroupCs25 = crate::Reg<mb_group_cs25::MbGroupCs25Spec>;
#[doc = "Message Buffer 25 CS Register"]
pub mod mb_group_cs25;
#[doc = "MB_32B_GROUP_MB10_32B_CS (rw) register accessor: Message Buffer 10 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb10_32b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb10_32b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb10_32b_cs`] module"]
#[doc(alias = "MB_32B_GROUP_MB10_32B_CS")]
pub type Mb32bGroupMb10_32bCs = crate::Reg<mb_32b_group_mb10_32b_cs::Mb32bGroupMb10_32bCsSpec>;
#[doc = "Message Buffer 10 CS Register"]
pub mod mb_32b_group_mb10_32b_cs;
#[doc = "MB_16B_GROUP_MB16_16B_WORD2 (rw) register accessor: Message Buffer 16 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb16_16b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb16_16b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb16_16b_word2`] module"]
#[doc(alias = "MB_16B_GROUP_MB16_16B_WORD2")]
pub type Mb16bGroupMb16_16bWord2 =
    crate::Reg<mb_16b_group_mb16_16b_word2::Mb16bGroupMb16_16bWord2Spec>;
#[doc = "Message Buffer 16 WORD_16B Register"]
pub mod mb_16b_group_mb16_16b_word2;
#[doc = "MB_8B_GROUP_MB25_8B_CS (rw) register accessor: Message Buffer 25 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb25_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb25_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb25_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB25_8B_CS")]
pub type Mb8bGroupMb25_8bCs = crate::Reg<mb_8b_group_mb25_8b_cs::Mb8bGroupMb25_8bCsSpec>;
#[doc = "Message Buffer 25 CS Register"]
pub mod mb_8b_group_mb25_8b_cs;
#[doc = "MB_64B_GROUP_MB5_64B_WORD8 (rw) register accessor: Message Buffer 5 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb5_64b_word8::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb5_64b_word8::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb5_64b_word8`] module"]
#[doc(alias = "MB_64B_GROUP_MB5_64B_WORD8")]
pub type Mb64bGroupMb5_64bWord8 =
    crate::Reg<mb_64b_group_mb5_64b_word8::Mb64bGroupMb5_64bWord8Spec>;
#[doc = "Message Buffer 5 WORD_64B Register"]
pub mod mb_64b_group_mb5_64b_word8;
#[doc = "MB_GROUP_ID25 (rw) register accessor: Message Buffer 25 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id25::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id25::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id25`] module"]
#[doc(alias = "MB_GROUP_ID25")]
pub type MbGroupId25 = crate::Reg<mb_group_id25::MbGroupId25Spec>;
#[doc = "Message Buffer 25 ID Register"]
pub mod mb_group_id25;
#[doc = "MB_32B_GROUP_MB10_32B_ID (rw) register accessor: Message Buffer 10 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb10_32b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb10_32b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb10_32b_id`] module"]
#[doc(alias = "MB_32B_GROUP_MB10_32B_ID")]
pub type Mb32bGroupMb10_32bId = crate::Reg<mb_32b_group_mb10_32b_id::Mb32bGroupMb10_32bIdSpec>;
#[doc = "Message Buffer 10 ID Register"]
pub mod mb_32b_group_mb10_32b_id;
#[doc = "MB_16B_GROUP_MB16_16B_WORD3 (rw) register accessor: Message Buffer 16 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb16_16b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb16_16b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb16_16b_word3`] module"]
#[doc(alias = "MB_16B_GROUP_MB16_16B_WORD3")]
pub type Mb16bGroupMb16_16bWord3 =
    crate::Reg<mb_16b_group_mb16_16b_word3::Mb16bGroupMb16_16bWord3Spec>;
#[doc = "Message Buffer 16 WORD_16B Register"]
pub mod mb_16b_group_mb16_16b_word3;
#[doc = "MB_8B_GROUP_MB25_8B_ID (rw) register accessor: Message Buffer 25 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb25_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb25_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb25_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB25_8B_ID")]
pub type Mb8bGroupMb25_8bId = crate::Reg<mb_8b_group_mb25_8b_id::Mb8bGroupMb25_8bIdSpec>;
#[doc = "Message Buffer 25 ID Register"]
pub mod mb_8b_group_mb25_8b_id;
#[doc = "MB_64B_GROUP_MB5_64B_WORD9 (rw) register accessor: Message Buffer 5 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb5_64b_word9::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb5_64b_word9::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb5_64b_word9`] module"]
#[doc(alias = "MB_64B_GROUP_MB5_64B_WORD9")]
pub type Mb64bGroupMb5_64bWord9 =
    crate::Reg<mb_64b_group_mb5_64b_word9::Mb64bGroupMb5_64bWord9Spec>;
#[doc = "Message Buffer 5 WORD_64B Register"]
pub mod mb_64b_group_mb5_64b_word9;
#[doc = "MB_32B_GROUP_MB10_32B_WORD0 (rw) register accessor: Message Buffer 10 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb10_32b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb10_32b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb10_32b_word0`] module"]
#[doc(alias = "MB_32B_GROUP_MB10_32B_WORD0")]
pub type Mb32bGroupMb10_32bWord0 =
    crate::Reg<mb_32b_group_mb10_32b_word0::Mb32bGroupMb10_32bWord0Spec>;
#[doc = "Message Buffer 10 WORD_32B Register"]
pub mod mb_32b_group_mb10_32b_word0;
#[doc = "MB_16B_GROUP_MB17_16B_CS (rw) register accessor: Message Buffer 17 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb17_16b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb17_16b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb17_16b_cs`] module"]
#[doc(alias = "MB_16B_GROUP_MB17_16B_CS")]
pub type Mb16bGroupMb17_16bCs = crate::Reg<mb_16b_group_mb17_16b_cs::Mb16bGroupMb17_16bCsSpec>;
#[doc = "Message Buffer 17 CS Register"]
pub mod mb_16b_group_mb17_16b_cs;
#[doc = "MB_8B_GROUP_MB25_8B_WORD0 (rw) register accessor: Message Buffer 25 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb25_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb25_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb25_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB25_8B_WORD0")]
pub type Mb8bGroupMb25_8bWord0 = crate::Reg<mb_8b_group_mb25_8b_word0::Mb8bGroupMb25_8bWord0Spec>;
#[doc = "Message Buffer 25 WORD_8B Register"]
pub mod mb_8b_group_mb25_8b_word0;
#[doc = "MB_64B_GROUP_MB5_64B_WORD10 (rw) register accessor: Message Buffer 5 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb5_64b_word10::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb5_64b_word10::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb5_64b_word10`] module"]
#[doc(alias = "MB_64B_GROUP_MB5_64B_WORD10")]
pub type Mb64bGroupMb5_64bWord10 =
    crate::Reg<mb_64b_group_mb5_64b_word10::Mb64bGroupMb5_64bWord10Spec>;
#[doc = "Message Buffer 5 WORD_64B Register"]
pub mod mb_64b_group_mb5_64b_word10;
#[doc = "MB_GROUP_WORD025 (rw) register accessor: Message Buffer 25 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word025::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word025::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word025`] module"]
#[doc(alias = "MB_GROUP_WORD025")]
pub type MbGroupWord025 = crate::Reg<mb_group_word025::MbGroupWord025Spec>;
#[doc = "Message Buffer 25 WORD0 Register"]
pub mod mb_group_word025;
#[doc = "MB_32B_GROUP_MB10_32B_WORD1 (rw) register accessor: Message Buffer 10 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb10_32b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb10_32b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb10_32b_word1`] module"]
#[doc(alias = "MB_32B_GROUP_MB10_32B_WORD1")]
pub type Mb32bGroupMb10_32bWord1 =
    crate::Reg<mb_32b_group_mb10_32b_word1::Mb32bGroupMb10_32bWord1Spec>;
#[doc = "Message Buffer 10 WORD_32B Register"]
pub mod mb_32b_group_mb10_32b_word1;
#[doc = "MB_16B_GROUP_MB17_16B_ID (rw) register accessor: Message Buffer 17 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb17_16b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb17_16b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb17_16b_id`] module"]
#[doc(alias = "MB_16B_GROUP_MB17_16B_ID")]
pub type Mb16bGroupMb17_16bId = crate::Reg<mb_16b_group_mb17_16b_id::Mb16bGroupMb17_16bIdSpec>;
#[doc = "Message Buffer 17 ID Register"]
pub mod mb_16b_group_mb17_16b_id;
#[doc = "MB_8B_GROUP_MB25_8B_WORD1 (rw) register accessor: Message Buffer 25 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb25_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb25_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb25_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB25_8B_WORD1")]
pub type Mb8bGroupMb25_8bWord1 = crate::Reg<mb_8b_group_mb25_8b_word1::Mb8bGroupMb25_8bWord1Spec>;
#[doc = "Message Buffer 25 WORD_8B Register"]
pub mod mb_8b_group_mb25_8b_word1;
#[doc = "MB_64B_GROUP_MB5_64B_WORD11 (rw) register accessor: Message Buffer 5 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb5_64b_word11::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb5_64b_word11::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb5_64b_word11`] module"]
#[doc(alias = "MB_64B_GROUP_MB5_64B_WORD11")]
pub type Mb64bGroupMb5_64bWord11 =
    crate::Reg<mb_64b_group_mb5_64b_word11::Mb64bGroupMb5_64bWord11Spec>;
#[doc = "Message Buffer 5 WORD_64B Register"]
pub mod mb_64b_group_mb5_64b_word11;
#[doc = "MB_GROUP_WORD125 (rw) register accessor: Message Buffer 25 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word125::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word125::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word125`] module"]
#[doc(alias = "MB_GROUP_WORD125")]
pub type MbGroupWord125 = crate::Reg<mb_group_word125::MbGroupWord125Spec>;
#[doc = "Message Buffer 25 WORD1 Register"]
pub mod mb_group_word125;
#[doc = "MB_GROUP_CS26 (rw) register accessor: Message Buffer 26 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs26::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs26::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs26`] module"]
#[doc(alias = "MB_GROUP_CS26")]
pub type MbGroupCs26 = crate::Reg<mb_group_cs26::MbGroupCs26Spec>;
#[doc = "Message Buffer 26 CS Register"]
pub mod mb_group_cs26;
#[doc = "MB_32B_GROUP_MB10_32B_WORD2 (rw) register accessor: Message Buffer 10 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb10_32b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb10_32b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb10_32b_word2`] module"]
#[doc(alias = "MB_32B_GROUP_MB10_32B_WORD2")]
pub type Mb32bGroupMb10_32bWord2 =
    crate::Reg<mb_32b_group_mb10_32b_word2::Mb32bGroupMb10_32bWord2Spec>;
#[doc = "Message Buffer 10 WORD_32B Register"]
pub mod mb_32b_group_mb10_32b_word2;
#[doc = "MB_16B_GROUP_MB17_16B_WORD0 (rw) register accessor: Message Buffer 17 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb17_16b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb17_16b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb17_16b_word0`] module"]
#[doc(alias = "MB_16B_GROUP_MB17_16B_WORD0")]
pub type Mb16bGroupMb17_16bWord0 =
    crate::Reg<mb_16b_group_mb17_16b_word0::Mb16bGroupMb17_16bWord0Spec>;
#[doc = "Message Buffer 17 WORD_16B Register"]
pub mod mb_16b_group_mb17_16b_word0;
#[doc = "MB_8B_GROUP_MB26_8B_CS (rw) register accessor: Message Buffer 26 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb26_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb26_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb26_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB26_8B_CS")]
pub type Mb8bGroupMb26_8bCs = crate::Reg<mb_8b_group_mb26_8b_cs::Mb8bGroupMb26_8bCsSpec>;
#[doc = "Message Buffer 26 CS Register"]
pub mod mb_8b_group_mb26_8b_cs;
#[doc = "MB_64B_GROUP_MB5_64B_WORD12 (rw) register accessor: Message Buffer 5 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb5_64b_word12::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb5_64b_word12::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb5_64b_word12`] module"]
#[doc(alias = "MB_64B_GROUP_MB5_64B_WORD12")]
pub type Mb64bGroupMb5_64bWord12 =
    crate::Reg<mb_64b_group_mb5_64b_word12::Mb64bGroupMb5_64bWord12Spec>;
#[doc = "Message Buffer 5 WORD_64B Register"]
pub mod mb_64b_group_mb5_64b_word12;
#[doc = "MB_GROUP_ID26 (rw) register accessor: Message Buffer 26 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id26::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id26::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id26`] module"]
#[doc(alias = "MB_GROUP_ID26")]
pub type MbGroupId26 = crate::Reg<mb_group_id26::MbGroupId26Spec>;
#[doc = "Message Buffer 26 ID Register"]
pub mod mb_group_id26;
#[doc = "MB_32B_GROUP_MB10_32B_WORD3 (rw) register accessor: Message Buffer 10 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb10_32b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb10_32b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb10_32b_word3`] module"]
#[doc(alias = "MB_32B_GROUP_MB10_32B_WORD3")]
pub type Mb32bGroupMb10_32bWord3 =
    crate::Reg<mb_32b_group_mb10_32b_word3::Mb32bGroupMb10_32bWord3Spec>;
#[doc = "Message Buffer 10 WORD_32B Register"]
pub mod mb_32b_group_mb10_32b_word3;
#[doc = "MB_16B_GROUP_MB17_16B_WORD1 (rw) register accessor: Message Buffer 17 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb17_16b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb17_16b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb17_16b_word1`] module"]
#[doc(alias = "MB_16B_GROUP_MB17_16B_WORD1")]
pub type Mb16bGroupMb17_16bWord1 =
    crate::Reg<mb_16b_group_mb17_16b_word1::Mb16bGroupMb17_16bWord1Spec>;
#[doc = "Message Buffer 17 WORD_16B Register"]
pub mod mb_16b_group_mb17_16b_word1;
#[doc = "MB_8B_GROUP_MB26_8B_ID (rw) register accessor: Message Buffer 26 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb26_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb26_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb26_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB26_8B_ID")]
pub type Mb8bGroupMb26_8bId = crate::Reg<mb_8b_group_mb26_8b_id::Mb8bGroupMb26_8bIdSpec>;
#[doc = "Message Buffer 26 ID Register"]
pub mod mb_8b_group_mb26_8b_id;
#[doc = "MB_64B_GROUP_MB5_64B_WORD13 (rw) register accessor: Message Buffer 5 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb5_64b_word13::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb5_64b_word13::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb5_64b_word13`] module"]
#[doc(alias = "MB_64B_GROUP_MB5_64B_WORD13")]
pub type Mb64bGroupMb5_64bWord13 =
    crate::Reg<mb_64b_group_mb5_64b_word13::Mb64bGroupMb5_64bWord13Spec>;
#[doc = "Message Buffer 5 WORD_64B Register"]
pub mod mb_64b_group_mb5_64b_word13;
#[doc = "MB_32B_GROUP_MB10_32B_WORD4 (rw) register accessor: Message Buffer 10 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb10_32b_word4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb10_32b_word4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb10_32b_word4`] module"]
#[doc(alias = "MB_32B_GROUP_MB10_32B_WORD4")]
pub type Mb32bGroupMb10_32bWord4 =
    crate::Reg<mb_32b_group_mb10_32b_word4::Mb32bGroupMb10_32bWord4Spec>;
#[doc = "Message Buffer 10 WORD_32B Register"]
pub mod mb_32b_group_mb10_32b_word4;
#[doc = "MB_16B_GROUP_MB17_16B_WORD2 (rw) register accessor: Message Buffer 17 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb17_16b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb17_16b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb17_16b_word2`] module"]
#[doc(alias = "MB_16B_GROUP_MB17_16B_WORD2")]
pub type Mb16bGroupMb17_16bWord2 =
    crate::Reg<mb_16b_group_mb17_16b_word2::Mb16bGroupMb17_16bWord2Spec>;
#[doc = "Message Buffer 17 WORD_16B Register"]
pub mod mb_16b_group_mb17_16b_word2;
#[doc = "MB_8B_GROUP_MB26_8B_WORD0 (rw) register accessor: Message Buffer 26 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb26_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb26_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb26_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB26_8B_WORD0")]
pub type Mb8bGroupMb26_8bWord0 = crate::Reg<mb_8b_group_mb26_8b_word0::Mb8bGroupMb26_8bWord0Spec>;
#[doc = "Message Buffer 26 WORD_8B Register"]
pub mod mb_8b_group_mb26_8b_word0;
#[doc = "MB_64B_GROUP_MB5_64B_WORD14 (rw) register accessor: Message Buffer 5 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb5_64b_word14::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb5_64b_word14::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb5_64b_word14`] module"]
#[doc(alias = "MB_64B_GROUP_MB5_64B_WORD14")]
pub type Mb64bGroupMb5_64bWord14 =
    crate::Reg<mb_64b_group_mb5_64b_word14::Mb64bGroupMb5_64bWord14Spec>;
#[doc = "Message Buffer 5 WORD_64B Register"]
pub mod mb_64b_group_mb5_64b_word14;
#[doc = "MB_GROUP_WORD026 (rw) register accessor: Message Buffer 26 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word026::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word026::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word026`] module"]
#[doc(alias = "MB_GROUP_WORD026")]
pub type MbGroupWord026 = crate::Reg<mb_group_word026::MbGroupWord026Spec>;
#[doc = "Message Buffer 26 WORD0 Register"]
pub mod mb_group_word026;
#[doc = "MB_32B_GROUP_MB10_32B_WORD5 (rw) register accessor: Message Buffer 10 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb10_32b_word5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb10_32b_word5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb10_32b_word5`] module"]
#[doc(alias = "MB_32B_GROUP_MB10_32B_WORD5")]
pub type Mb32bGroupMb10_32bWord5 =
    crate::Reg<mb_32b_group_mb10_32b_word5::Mb32bGroupMb10_32bWord5Spec>;
#[doc = "Message Buffer 10 WORD_32B Register"]
pub mod mb_32b_group_mb10_32b_word5;
#[doc = "MB_16B_GROUP_MB17_16B_WORD3 (rw) register accessor: Message Buffer 17 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb17_16b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb17_16b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb17_16b_word3`] module"]
#[doc(alias = "MB_16B_GROUP_MB17_16B_WORD3")]
pub type Mb16bGroupMb17_16bWord3 =
    crate::Reg<mb_16b_group_mb17_16b_word3::Mb16bGroupMb17_16bWord3Spec>;
#[doc = "Message Buffer 17 WORD_16B Register"]
pub mod mb_16b_group_mb17_16b_word3;
#[doc = "MB_8B_GROUP_MB26_8B_WORD1 (rw) register accessor: Message Buffer 26 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb26_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb26_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb26_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB26_8B_WORD1")]
pub type Mb8bGroupMb26_8bWord1 = crate::Reg<mb_8b_group_mb26_8b_word1::Mb8bGroupMb26_8bWord1Spec>;
#[doc = "Message Buffer 26 WORD_8B Register"]
pub mod mb_8b_group_mb26_8b_word1;
#[doc = "MB_64B_GROUP_MB5_64B_WORD15 (rw) register accessor: Message Buffer 5 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb5_64b_word15::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb5_64b_word15::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb5_64b_word15`] module"]
#[doc(alias = "MB_64B_GROUP_MB5_64B_WORD15")]
pub type Mb64bGroupMb5_64bWord15 =
    crate::Reg<mb_64b_group_mb5_64b_word15::Mb64bGroupMb5_64bWord15Spec>;
#[doc = "Message Buffer 5 WORD_64B Register"]
pub mod mb_64b_group_mb5_64b_word15;
#[doc = "MB_GROUP_WORD126 (rw) register accessor: Message Buffer 26 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word126::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word126::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word126`] module"]
#[doc(alias = "MB_GROUP_WORD126")]
pub type MbGroupWord126 = crate::Reg<mb_group_word126::MbGroupWord126Spec>;
#[doc = "Message Buffer 26 WORD1 Register"]
pub mod mb_group_word126;
#[doc = "MB_GROUP_CS27 (rw) register accessor: Message Buffer 27 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs27::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs27::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs27`] module"]
#[doc(alias = "MB_GROUP_CS27")]
pub type MbGroupCs27 = crate::Reg<mb_group_cs27::MbGroupCs27Spec>;
#[doc = "Message Buffer 27 CS Register"]
pub mod mb_group_cs27;
#[doc = "MB_32B_GROUP_MB10_32B_WORD6 (rw) register accessor: Message Buffer 10 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb10_32b_word6::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb10_32b_word6::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb10_32b_word6`] module"]
#[doc(alias = "MB_32B_GROUP_MB10_32B_WORD6")]
pub type Mb32bGroupMb10_32bWord6 =
    crate::Reg<mb_32b_group_mb10_32b_word6::Mb32bGroupMb10_32bWord6Spec>;
#[doc = "Message Buffer 10 WORD_32B Register"]
pub mod mb_32b_group_mb10_32b_word6;
#[doc = "MB_16B_GROUP_MB18_16B_CS (rw) register accessor: Message Buffer 18 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb18_16b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb18_16b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb18_16b_cs`] module"]
#[doc(alias = "MB_16B_GROUP_MB18_16B_CS")]
pub type Mb16bGroupMb18_16bCs = crate::Reg<mb_16b_group_mb18_16b_cs::Mb16bGroupMb18_16bCsSpec>;
#[doc = "Message Buffer 18 CS Register"]
pub mod mb_16b_group_mb18_16b_cs;
#[doc = "MB_8B_GROUP_MB27_8B_CS (rw) register accessor: Message Buffer 27 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb27_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb27_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb27_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB27_8B_CS")]
pub type Mb8bGroupMb27_8bCs = crate::Reg<mb_8b_group_mb27_8b_cs::Mb8bGroupMb27_8bCsSpec>;
#[doc = "Message Buffer 27 CS Register"]
pub mod mb_8b_group_mb27_8b_cs;
#[doc = "MB_64B_GROUP_MB6_64B_CS (rw) register accessor: Message Buffer 6 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb6_64b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb6_64b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb6_64b_cs`] module"]
#[doc(alias = "MB_64B_GROUP_MB6_64B_CS")]
pub type Mb64bGroupMb6_64bCs = crate::Reg<mb_64b_group_mb6_64b_cs::Mb64bGroupMb6_64bCsSpec>;
#[doc = "Message Buffer 6 CS Register"]
pub mod mb_64b_group_mb6_64b_cs;
#[doc = "MB_GROUP_ID27 (rw) register accessor: Message Buffer 27 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id27::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id27::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id27`] module"]
#[doc(alias = "MB_GROUP_ID27")]
pub type MbGroupId27 = crate::Reg<mb_group_id27::MbGroupId27Spec>;
#[doc = "Message Buffer 27 ID Register"]
pub mod mb_group_id27;
#[doc = "MB_32B_GROUP_MB10_32B_WORD7 (rw) register accessor: Message Buffer 10 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb10_32b_word7::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb10_32b_word7::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb10_32b_word7`] module"]
#[doc(alias = "MB_32B_GROUP_MB10_32B_WORD7")]
pub type Mb32bGroupMb10_32bWord7 =
    crate::Reg<mb_32b_group_mb10_32b_word7::Mb32bGroupMb10_32bWord7Spec>;
#[doc = "Message Buffer 10 WORD_32B Register"]
pub mod mb_32b_group_mb10_32b_word7;
#[doc = "MB_16B_GROUP_MB18_16B_ID (rw) register accessor: Message Buffer 18 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb18_16b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb18_16b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb18_16b_id`] module"]
#[doc(alias = "MB_16B_GROUP_MB18_16B_ID")]
pub type Mb16bGroupMb18_16bId = crate::Reg<mb_16b_group_mb18_16b_id::Mb16bGroupMb18_16bIdSpec>;
#[doc = "Message Buffer 18 ID Register"]
pub mod mb_16b_group_mb18_16b_id;
#[doc = "MB_8B_GROUP_MB27_8B_ID (rw) register accessor: Message Buffer 27 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb27_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb27_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb27_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB27_8B_ID")]
pub type Mb8bGroupMb27_8bId = crate::Reg<mb_8b_group_mb27_8b_id::Mb8bGroupMb27_8bIdSpec>;
#[doc = "Message Buffer 27 ID Register"]
pub mod mb_8b_group_mb27_8b_id;
#[doc = "MB_64B_GROUP_MB6_64B_ID (rw) register accessor: Message Buffer 6 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb6_64b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb6_64b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb6_64b_id`] module"]
#[doc(alias = "MB_64B_GROUP_MB6_64B_ID")]
pub type Mb64bGroupMb6_64bId = crate::Reg<mb_64b_group_mb6_64b_id::Mb64bGroupMb6_64bIdSpec>;
#[doc = "Message Buffer 6 ID Register"]
pub mod mb_64b_group_mb6_64b_id;
#[doc = "MB_32B_GROUP_MB11_32B_CS (rw) register accessor: Message Buffer 11 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb11_32b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb11_32b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb11_32b_cs`] module"]
#[doc(alias = "MB_32B_GROUP_MB11_32B_CS")]
pub type Mb32bGroupMb11_32bCs = crate::Reg<mb_32b_group_mb11_32b_cs::Mb32bGroupMb11_32bCsSpec>;
#[doc = "Message Buffer 11 CS Register"]
pub mod mb_32b_group_mb11_32b_cs;
#[doc = "MB_16B_GROUP_MB18_16B_WORD0 (rw) register accessor: Message Buffer 18 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb18_16b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb18_16b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb18_16b_word0`] module"]
#[doc(alias = "MB_16B_GROUP_MB18_16B_WORD0")]
pub type Mb16bGroupMb18_16bWord0 =
    crate::Reg<mb_16b_group_mb18_16b_word0::Mb16bGroupMb18_16bWord0Spec>;
#[doc = "Message Buffer 18 WORD_16B Register"]
pub mod mb_16b_group_mb18_16b_word0;
#[doc = "MB_8B_GROUP_MB27_8B_WORD0 (rw) register accessor: Message Buffer 27 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb27_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb27_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb27_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB27_8B_WORD0")]
pub type Mb8bGroupMb27_8bWord0 = crate::Reg<mb_8b_group_mb27_8b_word0::Mb8bGroupMb27_8bWord0Spec>;
#[doc = "Message Buffer 27 WORD_8B Register"]
pub mod mb_8b_group_mb27_8b_word0;
#[doc = "MB_64B_GROUP_MB6_64B_WORD0 (rw) register accessor: Message Buffer 6 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb6_64b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb6_64b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb6_64b_word0`] module"]
#[doc(alias = "MB_64B_GROUP_MB6_64B_WORD0")]
pub type Mb64bGroupMb6_64bWord0 =
    crate::Reg<mb_64b_group_mb6_64b_word0::Mb64bGroupMb6_64bWord0Spec>;
#[doc = "Message Buffer 6 WORD_64B Register"]
pub mod mb_64b_group_mb6_64b_word0;
#[doc = "MB_GROUP_WORD027 (rw) register accessor: Message Buffer 27 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word027::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word027::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word027`] module"]
#[doc(alias = "MB_GROUP_WORD027")]
pub type MbGroupWord027 = crate::Reg<mb_group_word027::MbGroupWord027Spec>;
#[doc = "Message Buffer 27 WORD0 Register"]
pub mod mb_group_word027;
#[doc = "MB_32B_GROUP_MB11_32B_ID (rw) register accessor: Message Buffer 11 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb11_32b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb11_32b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb11_32b_id`] module"]
#[doc(alias = "MB_32B_GROUP_MB11_32B_ID")]
pub type Mb32bGroupMb11_32bId = crate::Reg<mb_32b_group_mb11_32b_id::Mb32bGroupMb11_32bIdSpec>;
#[doc = "Message Buffer 11 ID Register"]
pub mod mb_32b_group_mb11_32b_id;
#[doc = "MB_16B_GROUP_MB18_16B_WORD1 (rw) register accessor: Message Buffer 18 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb18_16b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb18_16b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb18_16b_word1`] module"]
#[doc(alias = "MB_16B_GROUP_MB18_16B_WORD1")]
pub type Mb16bGroupMb18_16bWord1 =
    crate::Reg<mb_16b_group_mb18_16b_word1::Mb16bGroupMb18_16bWord1Spec>;
#[doc = "Message Buffer 18 WORD_16B Register"]
pub mod mb_16b_group_mb18_16b_word1;
#[doc = "MB_8B_GROUP_MB27_8B_WORD1 (rw) register accessor: Message Buffer 27 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb27_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb27_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb27_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB27_8B_WORD1")]
pub type Mb8bGroupMb27_8bWord1 = crate::Reg<mb_8b_group_mb27_8b_word1::Mb8bGroupMb27_8bWord1Spec>;
#[doc = "Message Buffer 27 WORD_8B Register"]
pub mod mb_8b_group_mb27_8b_word1;
#[doc = "MB_64B_GROUP_MB6_64B_WORD1 (rw) register accessor: Message Buffer 6 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb6_64b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb6_64b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb6_64b_word1`] module"]
#[doc(alias = "MB_64B_GROUP_MB6_64B_WORD1")]
pub type Mb64bGroupMb6_64bWord1 =
    crate::Reg<mb_64b_group_mb6_64b_word1::Mb64bGroupMb6_64bWord1Spec>;
#[doc = "Message Buffer 6 WORD_64B Register"]
pub mod mb_64b_group_mb6_64b_word1;
#[doc = "MB_GROUP_WORD127 (rw) register accessor: Message Buffer 27 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word127::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word127::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word127`] module"]
#[doc(alias = "MB_GROUP_WORD127")]
pub type MbGroupWord127 = crate::Reg<mb_group_word127::MbGroupWord127Spec>;
#[doc = "Message Buffer 27 WORD1 Register"]
pub mod mb_group_word127;
#[doc = "MB_GROUP_CS28 (rw) register accessor: Message Buffer 28 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs28::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs28::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs28`] module"]
#[doc(alias = "MB_GROUP_CS28")]
pub type MbGroupCs28 = crate::Reg<mb_group_cs28::MbGroupCs28Spec>;
#[doc = "Message Buffer 28 CS Register"]
pub mod mb_group_cs28;
#[doc = "MB_32B_GROUP_MB11_32B_WORD0 (rw) register accessor: Message Buffer 11 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb11_32b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb11_32b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb11_32b_word0`] module"]
#[doc(alias = "MB_32B_GROUP_MB11_32B_WORD0")]
pub type Mb32bGroupMb11_32bWord0 =
    crate::Reg<mb_32b_group_mb11_32b_word0::Mb32bGroupMb11_32bWord0Spec>;
#[doc = "Message Buffer 11 WORD_32B Register"]
pub mod mb_32b_group_mb11_32b_word0;
#[doc = "MB_16B_GROUP_MB18_16B_WORD2 (rw) register accessor: Message Buffer 18 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb18_16b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb18_16b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb18_16b_word2`] module"]
#[doc(alias = "MB_16B_GROUP_MB18_16B_WORD2")]
pub type Mb16bGroupMb18_16bWord2 =
    crate::Reg<mb_16b_group_mb18_16b_word2::Mb16bGroupMb18_16bWord2Spec>;
#[doc = "Message Buffer 18 WORD_16B Register"]
pub mod mb_16b_group_mb18_16b_word2;
#[doc = "MB_8B_GROUP_MB28_8B_CS (rw) register accessor: Message Buffer 28 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb28_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb28_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb28_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB28_8B_CS")]
pub type Mb8bGroupMb28_8bCs = crate::Reg<mb_8b_group_mb28_8b_cs::Mb8bGroupMb28_8bCsSpec>;
#[doc = "Message Buffer 28 CS Register"]
pub mod mb_8b_group_mb28_8b_cs;
#[doc = "MB_64B_GROUP_MB6_64B_WORD2 (rw) register accessor: Message Buffer 6 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb6_64b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb6_64b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb6_64b_word2`] module"]
#[doc(alias = "MB_64B_GROUP_MB6_64B_WORD2")]
pub type Mb64bGroupMb6_64bWord2 =
    crate::Reg<mb_64b_group_mb6_64b_word2::Mb64bGroupMb6_64bWord2Spec>;
#[doc = "Message Buffer 6 WORD_64B Register"]
pub mod mb_64b_group_mb6_64b_word2;
#[doc = "MB_GROUP_ID28 (rw) register accessor: Message Buffer 28 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id28::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id28::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id28`] module"]
#[doc(alias = "MB_GROUP_ID28")]
pub type MbGroupId28 = crate::Reg<mb_group_id28::MbGroupId28Spec>;
#[doc = "Message Buffer 28 ID Register"]
pub mod mb_group_id28;
#[doc = "MB_32B_GROUP_MB11_32B_WORD1 (rw) register accessor: Message Buffer 11 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb11_32b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb11_32b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb11_32b_word1`] module"]
#[doc(alias = "MB_32B_GROUP_MB11_32B_WORD1")]
pub type Mb32bGroupMb11_32bWord1 =
    crate::Reg<mb_32b_group_mb11_32b_word1::Mb32bGroupMb11_32bWord1Spec>;
#[doc = "Message Buffer 11 WORD_32B Register"]
pub mod mb_32b_group_mb11_32b_word1;
#[doc = "MB_16B_GROUP_MB18_16B_WORD3 (rw) register accessor: Message Buffer 18 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb18_16b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb18_16b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb18_16b_word3`] module"]
#[doc(alias = "MB_16B_GROUP_MB18_16B_WORD3")]
pub type Mb16bGroupMb18_16bWord3 =
    crate::Reg<mb_16b_group_mb18_16b_word3::Mb16bGroupMb18_16bWord3Spec>;
#[doc = "Message Buffer 18 WORD_16B Register"]
pub mod mb_16b_group_mb18_16b_word3;
#[doc = "MB_8B_GROUP_MB28_8B_ID (rw) register accessor: Message Buffer 28 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb28_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb28_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb28_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB28_8B_ID")]
pub type Mb8bGroupMb28_8bId = crate::Reg<mb_8b_group_mb28_8b_id::Mb8bGroupMb28_8bIdSpec>;
#[doc = "Message Buffer 28 ID Register"]
pub mod mb_8b_group_mb28_8b_id;
#[doc = "MB_64B_GROUP_MB6_64B_WORD3 (rw) register accessor: Message Buffer 6 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb6_64b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb6_64b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb6_64b_word3`] module"]
#[doc(alias = "MB_64B_GROUP_MB6_64B_WORD3")]
pub type Mb64bGroupMb6_64bWord3 =
    crate::Reg<mb_64b_group_mb6_64b_word3::Mb64bGroupMb6_64bWord3Spec>;
#[doc = "Message Buffer 6 WORD_64B Register"]
pub mod mb_64b_group_mb6_64b_word3;
#[doc = "MB_32B_GROUP_MB11_32B_WORD2 (rw) register accessor: Message Buffer 11 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb11_32b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb11_32b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb11_32b_word2`] module"]
#[doc(alias = "MB_32B_GROUP_MB11_32B_WORD2")]
pub type Mb32bGroupMb11_32bWord2 =
    crate::Reg<mb_32b_group_mb11_32b_word2::Mb32bGroupMb11_32bWord2Spec>;
#[doc = "Message Buffer 11 WORD_32B Register"]
pub mod mb_32b_group_mb11_32b_word2;
#[doc = "MB_16B_GROUP_MB19_16B_CS (rw) register accessor: Message Buffer 19 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb19_16b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb19_16b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb19_16b_cs`] module"]
#[doc(alias = "MB_16B_GROUP_MB19_16B_CS")]
pub type Mb16bGroupMb19_16bCs = crate::Reg<mb_16b_group_mb19_16b_cs::Mb16bGroupMb19_16bCsSpec>;
#[doc = "Message Buffer 19 CS Register"]
pub mod mb_16b_group_mb19_16b_cs;
#[doc = "MB_8B_GROUP_MB28_8B_WORD0 (rw) register accessor: Message Buffer 28 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb28_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb28_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb28_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB28_8B_WORD0")]
pub type Mb8bGroupMb28_8bWord0 = crate::Reg<mb_8b_group_mb28_8b_word0::Mb8bGroupMb28_8bWord0Spec>;
#[doc = "Message Buffer 28 WORD_8B Register"]
pub mod mb_8b_group_mb28_8b_word0;
#[doc = "MB_64B_GROUP_MB6_64B_WORD4 (rw) register accessor: Message Buffer 6 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb6_64b_word4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb6_64b_word4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb6_64b_word4`] module"]
#[doc(alias = "MB_64B_GROUP_MB6_64B_WORD4")]
pub type Mb64bGroupMb6_64bWord4 =
    crate::Reg<mb_64b_group_mb6_64b_word4::Mb64bGroupMb6_64bWord4Spec>;
#[doc = "Message Buffer 6 WORD_64B Register"]
pub mod mb_64b_group_mb6_64b_word4;
#[doc = "MB_GROUP_WORD028 (rw) register accessor: Message Buffer 28 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word028::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word028::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word028`] module"]
#[doc(alias = "MB_GROUP_WORD028")]
pub type MbGroupWord028 = crate::Reg<mb_group_word028::MbGroupWord028Spec>;
#[doc = "Message Buffer 28 WORD0 Register"]
pub mod mb_group_word028;
#[doc = "MB_32B_GROUP_MB11_32B_WORD3 (rw) register accessor: Message Buffer 11 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb11_32b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb11_32b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb11_32b_word3`] module"]
#[doc(alias = "MB_32B_GROUP_MB11_32B_WORD3")]
pub type Mb32bGroupMb11_32bWord3 =
    crate::Reg<mb_32b_group_mb11_32b_word3::Mb32bGroupMb11_32bWord3Spec>;
#[doc = "Message Buffer 11 WORD_32B Register"]
pub mod mb_32b_group_mb11_32b_word3;
#[doc = "MB_16B_GROUP_MB19_16B_ID (rw) register accessor: Message Buffer 19 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb19_16b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb19_16b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb19_16b_id`] module"]
#[doc(alias = "MB_16B_GROUP_MB19_16B_ID")]
pub type Mb16bGroupMb19_16bId = crate::Reg<mb_16b_group_mb19_16b_id::Mb16bGroupMb19_16bIdSpec>;
#[doc = "Message Buffer 19 ID Register"]
pub mod mb_16b_group_mb19_16b_id;
#[doc = "MB_8B_GROUP_MB28_8B_WORD1 (rw) register accessor: Message Buffer 28 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb28_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb28_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb28_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB28_8B_WORD1")]
pub type Mb8bGroupMb28_8bWord1 = crate::Reg<mb_8b_group_mb28_8b_word1::Mb8bGroupMb28_8bWord1Spec>;
#[doc = "Message Buffer 28 WORD_8B Register"]
pub mod mb_8b_group_mb28_8b_word1;
#[doc = "MB_64B_GROUP_MB6_64B_WORD5 (rw) register accessor: Message Buffer 6 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb6_64b_word5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb6_64b_word5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb6_64b_word5`] module"]
#[doc(alias = "MB_64B_GROUP_MB6_64B_WORD5")]
pub type Mb64bGroupMb6_64bWord5 =
    crate::Reg<mb_64b_group_mb6_64b_word5::Mb64bGroupMb6_64bWord5Spec>;
#[doc = "Message Buffer 6 WORD_64B Register"]
pub mod mb_64b_group_mb6_64b_word5;
#[doc = "MB_GROUP_WORD128 (rw) register accessor: Message Buffer 28 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word128::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word128::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word128`] module"]
#[doc(alias = "MB_GROUP_WORD128")]
pub type MbGroupWord128 = crate::Reg<mb_group_word128::MbGroupWord128Spec>;
#[doc = "Message Buffer 28 WORD1 Register"]
pub mod mb_group_word128;
#[doc = "MB_GROUP_CS29 (rw) register accessor: Message Buffer 29 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs29::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs29::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs29`] module"]
#[doc(alias = "MB_GROUP_CS29")]
pub type MbGroupCs29 = crate::Reg<mb_group_cs29::MbGroupCs29Spec>;
#[doc = "Message Buffer 29 CS Register"]
pub mod mb_group_cs29;
#[doc = "MB_32B_GROUP_MB11_32B_WORD4 (rw) register accessor: Message Buffer 11 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb11_32b_word4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb11_32b_word4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb11_32b_word4`] module"]
#[doc(alias = "MB_32B_GROUP_MB11_32B_WORD4")]
pub type Mb32bGroupMb11_32bWord4 =
    crate::Reg<mb_32b_group_mb11_32b_word4::Mb32bGroupMb11_32bWord4Spec>;
#[doc = "Message Buffer 11 WORD_32B Register"]
pub mod mb_32b_group_mb11_32b_word4;
#[doc = "MB_16B_GROUP_MB19_16B_WORD0 (rw) register accessor: Message Buffer 19 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb19_16b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb19_16b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb19_16b_word0`] module"]
#[doc(alias = "MB_16B_GROUP_MB19_16B_WORD0")]
pub type Mb16bGroupMb19_16bWord0 =
    crate::Reg<mb_16b_group_mb19_16b_word0::Mb16bGroupMb19_16bWord0Spec>;
#[doc = "Message Buffer 19 WORD_16B Register"]
pub mod mb_16b_group_mb19_16b_word0;
#[doc = "MB_8B_GROUP_MB29_8B_CS (rw) register accessor: Message Buffer 29 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb29_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb29_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb29_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB29_8B_CS")]
pub type Mb8bGroupMb29_8bCs = crate::Reg<mb_8b_group_mb29_8b_cs::Mb8bGroupMb29_8bCsSpec>;
#[doc = "Message Buffer 29 CS Register"]
pub mod mb_8b_group_mb29_8b_cs;
#[doc = "MB_64B_GROUP_MB6_64B_WORD6 (rw) register accessor: Message Buffer 6 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb6_64b_word6::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb6_64b_word6::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb6_64b_word6`] module"]
#[doc(alias = "MB_64B_GROUP_MB6_64B_WORD6")]
pub type Mb64bGroupMb6_64bWord6 =
    crate::Reg<mb_64b_group_mb6_64b_word6::Mb64bGroupMb6_64bWord6Spec>;
#[doc = "Message Buffer 6 WORD_64B Register"]
pub mod mb_64b_group_mb6_64b_word6;
#[doc = "MB_GROUP_ID29 (rw) register accessor: Message Buffer 29 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id29::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id29::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id29`] module"]
#[doc(alias = "MB_GROUP_ID29")]
pub type MbGroupId29 = crate::Reg<mb_group_id29::MbGroupId29Spec>;
#[doc = "Message Buffer 29 ID Register"]
pub mod mb_group_id29;
#[doc = "MB_32B_GROUP_MB11_32B_WORD5 (rw) register accessor: Message Buffer 11 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb11_32b_word5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb11_32b_word5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb11_32b_word5`] module"]
#[doc(alias = "MB_32B_GROUP_MB11_32B_WORD5")]
pub type Mb32bGroupMb11_32bWord5 =
    crate::Reg<mb_32b_group_mb11_32b_word5::Mb32bGroupMb11_32bWord5Spec>;
#[doc = "Message Buffer 11 WORD_32B Register"]
pub mod mb_32b_group_mb11_32b_word5;
#[doc = "MB_16B_GROUP_MB19_16B_WORD1 (rw) register accessor: Message Buffer 19 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb19_16b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb19_16b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb19_16b_word1`] module"]
#[doc(alias = "MB_16B_GROUP_MB19_16B_WORD1")]
pub type Mb16bGroupMb19_16bWord1 =
    crate::Reg<mb_16b_group_mb19_16b_word1::Mb16bGroupMb19_16bWord1Spec>;
#[doc = "Message Buffer 19 WORD_16B Register"]
pub mod mb_16b_group_mb19_16b_word1;
#[doc = "MB_8B_GROUP_MB29_8B_ID (rw) register accessor: Message Buffer 29 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb29_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb29_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb29_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB29_8B_ID")]
pub type Mb8bGroupMb29_8bId = crate::Reg<mb_8b_group_mb29_8b_id::Mb8bGroupMb29_8bIdSpec>;
#[doc = "Message Buffer 29 ID Register"]
pub mod mb_8b_group_mb29_8b_id;
#[doc = "MB_64B_GROUP_MB6_64B_WORD7 (rw) register accessor: Message Buffer 6 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb6_64b_word7::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb6_64b_word7::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb6_64b_word7`] module"]
#[doc(alias = "MB_64B_GROUP_MB6_64B_WORD7")]
pub type Mb64bGroupMb6_64bWord7 =
    crate::Reg<mb_64b_group_mb6_64b_word7::Mb64bGroupMb6_64bWord7Spec>;
#[doc = "Message Buffer 6 WORD_64B Register"]
pub mod mb_64b_group_mb6_64b_word7;
#[doc = "MB_32B_GROUP_MB11_32B_WORD6 (rw) register accessor: Message Buffer 11 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb11_32b_word6::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb11_32b_word6::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb11_32b_word6`] module"]
#[doc(alias = "MB_32B_GROUP_MB11_32B_WORD6")]
pub type Mb32bGroupMb11_32bWord6 =
    crate::Reg<mb_32b_group_mb11_32b_word6::Mb32bGroupMb11_32bWord6Spec>;
#[doc = "Message Buffer 11 WORD_32B Register"]
pub mod mb_32b_group_mb11_32b_word6;
#[doc = "MB_16B_GROUP_MB19_16B_WORD2 (rw) register accessor: Message Buffer 19 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb19_16b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb19_16b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb19_16b_word2`] module"]
#[doc(alias = "MB_16B_GROUP_MB19_16B_WORD2")]
pub type Mb16bGroupMb19_16bWord2 =
    crate::Reg<mb_16b_group_mb19_16b_word2::Mb16bGroupMb19_16bWord2Spec>;
#[doc = "Message Buffer 19 WORD_16B Register"]
pub mod mb_16b_group_mb19_16b_word2;
#[doc = "MB_8B_GROUP_MB29_8B_WORD0 (rw) register accessor: Message Buffer 29 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb29_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb29_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb29_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB29_8B_WORD0")]
pub type Mb8bGroupMb29_8bWord0 = crate::Reg<mb_8b_group_mb29_8b_word0::Mb8bGroupMb29_8bWord0Spec>;
#[doc = "Message Buffer 29 WORD_8B Register"]
pub mod mb_8b_group_mb29_8b_word0;
#[doc = "MB_64B_GROUP_MB6_64B_WORD8 (rw) register accessor: Message Buffer 6 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb6_64b_word8::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb6_64b_word8::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb6_64b_word8`] module"]
#[doc(alias = "MB_64B_GROUP_MB6_64B_WORD8")]
pub type Mb64bGroupMb6_64bWord8 =
    crate::Reg<mb_64b_group_mb6_64b_word8::Mb64bGroupMb6_64bWord8Spec>;
#[doc = "Message Buffer 6 WORD_64B Register"]
pub mod mb_64b_group_mb6_64b_word8;
#[doc = "MB_GROUP_WORD029 (rw) register accessor: Message Buffer 29 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word029::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word029::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word029`] module"]
#[doc(alias = "MB_GROUP_WORD029")]
pub type MbGroupWord029 = crate::Reg<mb_group_word029::MbGroupWord029Spec>;
#[doc = "Message Buffer 29 WORD0 Register"]
pub mod mb_group_word029;
#[doc = "MB_32B_GROUP_MB11_32B_WORD7 (rw) register accessor: Message Buffer 11 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb11_32b_word7::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb11_32b_word7::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_32b_group_mb11_32b_word7`] module"]
#[doc(alias = "MB_32B_GROUP_MB11_32B_WORD7")]
pub type Mb32bGroupMb11_32bWord7 =
    crate::Reg<mb_32b_group_mb11_32b_word7::Mb32bGroupMb11_32bWord7Spec>;
#[doc = "Message Buffer 11 WORD_32B Register"]
pub mod mb_32b_group_mb11_32b_word7;
#[doc = "MB_16B_GROUP_MB19_16B_WORD3 (rw) register accessor: Message Buffer 19 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb19_16b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb19_16b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb19_16b_word3`] module"]
#[doc(alias = "MB_16B_GROUP_MB19_16B_WORD3")]
pub type Mb16bGroupMb19_16bWord3 =
    crate::Reg<mb_16b_group_mb19_16b_word3::Mb16bGroupMb19_16bWord3Spec>;
#[doc = "Message Buffer 19 WORD_16B Register"]
pub mod mb_16b_group_mb19_16b_word3;
#[doc = "MB_8B_GROUP_MB29_8B_WORD1 (rw) register accessor: Message Buffer 29 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb29_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb29_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb29_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB29_8B_WORD1")]
pub type Mb8bGroupMb29_8bWord1 = crate::Reg<mb_8b_group_mb29_8b_word1::Mb8bGroupMb29_8bWord1Spec>;
#[doc = "Message Buffer 29 WORD_8B Register"]
pub mod mb_8b_group_mb29_8b_word1;
#[doc = "MB_64B_GROUP_MB6_64B_WORD9 (rw) register accessor: Message Buffer 6 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb6_64b_word9::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb6_64b_word9::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb6_64b_word9`] module"]
#[doc(alias = "MB_64B_GROUP_MB6_64B_WORD9")]
pub type Mb64bGroupMb6_64bWord9 =
    crate::Reg<mb_64b_group_mb6_64b_word9::Mb64bGroupMb6_64bWord9Spec>;
#[doc = "Message Buffer 6 WORD_64B Register"]
pub mod mb_64b_group_mb6_64b_word9;
#[doc = "MB_GROUP_WORD129 (rw) register accessor: Message Buffer 29 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word129::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word129::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word129`] module"]
#[doc(alias = "MB_GROUP_WORD129")]
pub type MbGroupWord129 = crate::Reg<mb_group_word129::MbGroupWord129Spec>;
#[doc = "Message Buffer 29 WORD1 Register"]
pub mod mb_group_word129;
#[doc = "MB_GROUP_CS30 (rw) register accessor: Message Buffer 30 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs30::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs30::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs30`] module"]
#[doc(alias = "MB_GROUP_CS30")]
pub type MbGroupCs30 = crate::Reg<mb_group_cs30::MbGroupCs30Spec>;
#[doc = "Message Buffer 30 CS Register"]
pub mod mb_group_cs30;
#[doc = "MB_16B_GROUP_MB20_16B_CS (rw) register accessor: Message Buffer 20 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb20_16b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb20_16b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb20_16b_cs`] module"]
#[doc(alias = "MB_16B_GROUP_MB20_16B_CS")]
pub type Mb16bGroupMb20_16bCs = crate::Reg<mb_16b_group_mb20_16b_cs::Mb16bGroupMb20_16bCsSpec>;
#[doc = "Message Buffer 20 CS Register"]
pub mod mb_16b_group_mb20_16b_cs;
#[doc = "MB_8B_GROUP_MB30_8B_CS (rw) register accessor: Message Buffer 30 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb30_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb30_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb30_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB30_8B_CS")]
pub type Mb8bGroupMb30_8bCs = crate::Reg<mb_8b_group_mb30_8b_cs::Mb8bGroupMb30_8bCsSpec>;
#[doc = "Message Buffer 30 CS Register"]
pub mod mb_8b_group_mb30_8b_cs;
#[doc = "MB_64B_GROUP_MB6_64B_WORD10 (rw) register accessor: Message Buffer 6 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb6_64b_word10::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb6_64b_word10::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb6_64b_word10`] module"]
#[doc(alias = "MB_64B_GROUP_MB6_64B_WORD10")]
pub type Mb64bGroupMb6_64bWord10 =
    crate::Reg<mb_64b_group_mb6_64b_word10::Mb64bGroupMb6_64bWord10Spec>;
#[doc = "Message Buffer 6 WORD_64B Register"]
pub mod mb_64b_group_mb6_64b_word10;
#[doc = "MB_GROUP_ID30 (rw) register accessor: Message Buffer 30 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id30::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id30::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id30`] module"]
#[doc(alias = "MB_GROUP_ID30")]
pub type MbGroupId30 = crate::Reg<mb_group_id30::MbGroupId30Spec>;
#[doc = "Message Buffer 30 ID Register"]
pub mod mb_group_id30;
#[doc = "MB_16B_GROUP_MB20_16B_ID (rw) register accessor: Message Buffer 20 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb20_16b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb20_16b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb20_16b_id`] module"]
#[doc(alias = "MB_16B_GROUP_MB20_16B_ID")]
pub type Mb16bGroupMb20_16bId = crate::Reg<mb_16b_group_mb20_16b_id::Mb16bGroupMb20_16bIdSpec>;
#[doc = "Message Buffer 20 ID Register"]
pub mod mb_16b_group_mb20_16b_id;
#[doc = "MB_8B_GROUP_MB30_8B_ID (rw) register accessor: Message Buffer 30 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb30_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb30_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb30_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB30_8B_ID")]
pub type Mb8bGroupMb30_8bId = crate::Reg<mb_8b_group_mb30_8b_id::Mb8bGroupMb30_8bIdSpec>;
#[doc = "Message Buffer 30 ID Register"]
pub mod mb_8b_group_mb30_8b_id;
#[doc = "MB_64B_GROUP_MB6_64B_WORD11 (rw) register accessor: Message Buffer 6 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb6_64b_word11::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb6_64b_word11::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb6_64b_word11`] module"]
#[doc(alias = "MB_64B_GROUP_MB6_64B_WORD11")]
pub type Mb64bGroupMb6_64bWord11 =
    crate::Reg<mb_64b_group_mb6_64b_word11::Mb64bGroupMb6_64bWord11Spec>;
#[doc = "Message Buffer 6 WORD_64B Register"]
pub mod mb_64b_group_mb6_64b_word11;
#[doc = "MB_16B_GROUP_MB20_16B_WORD0 (rw) register accessor: Message Buffer 20 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb20_16b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb20_16b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb20_16b_word0`] module"]
#[doc(alias = "MB_16B_GROUP_MB20_16B_WORD0")]
pub type Mb16bGroupMb20_16bWord0 =
    crate::Reg<mb_16b_group_mb20_16b_word0::Mb16bGroupMb20_16bWord0Spec>;
#[doc = "Message Buffer 20 WORD_16B Register"]
pub mod mb_16b_group_mb20_16b_word0;
#[doc = "MB_8B_GROUP_MB30_8B_WORD0 (rw) register accessor: Message Buffer 30 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb30_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb30_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb30_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB30_8B_WORD0")]
pub type Mb8bGroupMb30_8bWord0 = crate::Reg<mb_8b_group_mb30_8b_word0::Mb8bGroupMb30_8bWord0Spec>;
#[doc = "Message Buffer 30 WORD_8B Register"]
pub mod mb_8b_group_mb30_8b_word0;
#[doc = "MB_64B_GROUP_MB6_64B_WORD12 (rw) register accessor: Message Buffer 6 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb6_64b_word12::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb6_64b_word12::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb6_64b_word12`] module"]
#[doc(alias = "MB_64B_GROUP_MB6_64B_WORD12")]
pub type Mb64bGroupMb6_64bWord12 =
    crate::Reg<mb_64b_group_mb6_64b_word12::Mb64bGroupMb6_64bWord12Spec>;
#[doc = "Message Buffer 6 WORD_64B Register"]
pub mod mb_64b_group_mb6_64b_word12;
#[doc = "MB_GROUP_WORD030 (rw) register accessor: Message Buffer 30 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word030::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word030::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word030`] module"]
#[doc(alias = "MB_GROUP_WORD030")]
pub type MbGroupWord030 = crate::Reg<mb_group_word030::MbGroupWord030Spec>;
#[doc = "Message Buffer 30 WORD0 Register"]
pub mod mb_group_word030;
#[doc = "MB_16B_GROUP_MB20_16B_WORD1 (rw) register accessor: Message Buffer 20 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb20_16b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb20_16b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb20_16b_word1`] module"]
#[doc(alias = "MB_16B_GROUP_MB20_16B_WORD1")]
pub type Mb16bGroupMb20_16bWord1 =
    crate::Reg<mb_16b_group_mb20_16b_word1::Mb16bGroupMb20_16bWord1Spec>;
#[doc = "Message Buffer 20 WORD_16B Register"]
pub mod mb_16b_group_mb20_16b_word1;
#[doc = "MB_8B_GROUP_MB30_8B_WORD1 (rw) register accessor: Message Buffer 30 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb30_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb30_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb30_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB30_8B_WORD1")]
pub type Mb8bGroupMb30_8bWord1 = crate::Reg<mb_8b_group_mb30_8b_word1::Mb8bGroupMb30_8bWord1Spec>;
#[doc = "Message Buffer 30 WORD_8B Register"]
pub mod mb_8b_group_mb30_8b_word1;
#[doc = "MB_64B_GROUP_MB6_64B_WORD13 (rw) register accessor: Message Buffer 6 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb6_64b_word13::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb6_64b_word13::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb6_64b_word13`] module"]
#[doc(alias = "MB_64B_GROUP_MB6_64B_WORD13")]
pub type Mb64bGroupMb6_64bWord13 =
    crate::Reg<mb_64b_group_mb6_64b_word13::Mb64bGroupMb6_64bWord13Spec>;
#[doc = "Message Buffer 6 WORD_64B Register"]
pub mod mb_64b_group_mb6_64b_word13;
#[doc = "MB_GROUP_WORD130 (rw) register accessor: Message Buffer 30 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word130::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word130::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word130`] module"]
#[doc(alias = "MB_GROUP_WORD130")]
pub type MbGroupWord130 = crate::Reg<mb_group_word130::MbGroupWord130Spec>;
#[doc = "Message Buffer 30 WORD1 Register"]
pub mod mb_group_word130;
#[doc = "MB_GROUP_CS31 (rw) register accessor: Message Buffer 31 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_cs31::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_cs31::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_cs31`] module"]
#[doc(alias = "MB_GROUP_CS31")]
pub type MbGroupCs31 = crate::Reg<mb_group_cs31::MbGroupCs31Spec>;
#[doc = "Message Buffer 31 CS Register"]
pub mod mb_group_cs31;
#[doc = "MB_16B_GROUP_MB20_16B_WORD2 (rw) register accessor: Message Buffer 20 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb20_16b_word2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb20_16b_word2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb20_16b_word2`] module"]
#[doc(alias = "MB_16B_GROUP_MB20_16B_WORD2")]
pub type Mb16bGroupMb20_16bWord2 =
    crate::Reg<mb_16b_group_mb20_16b_word2::Mb16bGroupMb20_16bWord2Spec>;
#[doc = "Message Buffer 20 WORD_16B Register"]
pub mod mb_16b_group_mb20_16b_word2;
#[doc = "MB_8B_GROUP_MB31_8B_CS (rw) register accessor: Message Buffer 31 CS Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb31_8b_cs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb31_8b_cs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb31_8b_cs`] module"]
#[doc(alias = "MB_8B_GROUP_MB31_8B_CS")]
pub type Mb8bGroupMb31_8bCs = crate::Reg<mb_8b_group_mb31_8b_cs::Mb8bGroupMb31_8bCsSpec>;
#[doc = "Message Buffer 31 CS Register"]
pub mod mb_8b_group_mb31_8b_cs;
#[doc = "MB_64B_GROUP_MB6_64B_WORD14 (rw) register accessor: Message Buffer 6 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb6_64b_word14::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb6_64b_word14::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb6_64b_word14`] module"]
#[doc(alias = "MB_64B_GROUP_MB6_64B_WORD14")]
pub type Mb64bGroupMb6_64bWord14 =
    crate::Reg<mb_64b_group_mb6_64b_word14::Mb64bGroupMb6_64bWord14Spec>;
#[doc = "Message Buffer 6 WORD_64B Register"]
pub mod mb_64b_group_mb6_64b_word14;
#[doc = "MB_GROUP_ID31 (rw) register accessor: Message Buffer 31 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id31::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id31::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_id31`] module"]
#[doc(alias = "MB_GROUP_ID31")]
pub type MbGroupId31 = crate::Reg<mb_group_id31::MbGroupId31Spec>;
#[doc = "Message Buffer 31 ID Register"]
pub mod mb_group_id31;
#[doc = "MB_16B_GROUP_MB20_16B_WORD3 (rw) register accessor: Message Buffer 20 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb20_16b_word3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb20_16b_word3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_16b_group_mb20_16b_word3`] module"]
#[doc(alias = "MB_16B_GROUP_MB20_16B_WORD3")]
pub type Mb16bGroupMb20_16bWord3 =
    crate::Reg<mb_16b_group_mb20_16b_word3::Mb16bGroupMb20_16bWord3Spec>;
#[doc = "Message Buffer 20 WORD_16B Register"]
pub mod mb_16b_group_mb20_16b_word3;
#[doc = "MB_8B_GROUP_MB31_8B_ID (rw) register accessor: Message Buffer 31 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb31_8b_id::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb31_8b_id::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb31_8b_id`] module"]
#[doc(alias = "MB_8B_GROUP_MB31_8B_ID")]
pub type Mb8bGroupMb31_8bId = crate::Reg<mb_8b_group_mb31_8b_id::Mb8bGroupMb31_8bIdSpec>;
#[doc = "Message Buffer 31 ID Register"]
pub mod mb_8b_group_mb31_8b_id;
#[doc = "MB_64B_GROUP_MB6_64B_WORD15 (rw) register accessor: Message Buffer 6 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb6_64b_word15::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb6_64b_word15::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_64b_group_mb6_64b_word15`] module"]
#[doc(alias = "MB_64B_GROUP_MB6_64B_WORD15")]
pub type Mb64bGroupMb6_64bWord15 =
    crate::Reg<mb_64b_group_mb6_64b_word15::Mb64bGroupMb6_64bWord15Spec>;
#[doc = "Message Buffer 6 WORD_64B Register"]
pub mod mb_64b_group_mb6_64b_word15;
#[doc = "MB_8B_GROUP_MB31_8B_WORD0 (rw) register accessor: Message Buffer 31 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb31_8b_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb31_8b_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb31_8b_word0`] module"]
#[doc(alias = "MB_8B_GROUP_MB31_8B_WORD0")]
pub type Mb8bGroupMb31_8bWord0 = crate::Reg<mb_8b_group_mb31_8b_word0::Mb8bGroupMb31_8bWord0Spec>;
#[doc = "Message Buffer 31 WORD_8B Register"]
pub mod mb_8b_group_mb31_8b_word0;
#[doc = "MB_GROUP_WORD031 (rw) register accessor: Message Buffer 31 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word031::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word031::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word031`] module"]
#[doc(alias = "MB_GROUP_WORD031")]
pub type MbGroupWord031 = crate::Reg<mb_group_word031::MbGroupWord031Spec>;
#[doc = "Message Buffer 31 WORD0 Register"]
pub mod mb_group_word031;
#[doc = "MB_8B_GROUP_MB31_8B_WORD1 (rw) register accessor: Message Buffer 31 WORD_8B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_8b_group_mb31_8b_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_8b_group_mb31_8b_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_8b_group_mb31_8b_word1`] module"]
#[doc(alias = "MB_8B_GROUP_MB31_8B_WORD1")]
pub type Mb8bGroupMb31_8bWord1 = crate::Reg<mb_8b_group_mb31_8b_word1::Mb8bGroupMb31_8bWord1Spec>;
#[doc = "Message Buffer 31 WORD_8B Register"]
pub mod mb_8b_group_mb31_8b_word1;
#[doc = "MB_GROUP_WORD131 (rw) register accessor: Message Buffer 31 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word131::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word131::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mb_group_word131`] module"]
#[doc(alias = "MB_GROUP_WORD131")]
pub type MbGroupWord131 = crate::Reg<mb_group_word131::MbGroupWord131Spec>;
#[doc = "Message Buffer 31 WORD1 Register"]
pub mod mb_group_word131;
#[doc = "RXIMR (rw) register accessor: Receive Individual Mask\n\nYou can [`read`](crate::Reg::read) this register and get [`rximr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rximr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@rximr`] module"]
#[doc(alias = "RXIMR")]
pub type Rximr = crate::Reg<rximr::RximrSpec>;
#[doc = "Receive Individual Mask"]
pub mod rximr;
#[doc = "CTRL1_PN (rw) register accessor: Pretended Networking Control 1\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl1_pn::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl1_pn::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ctrl1_pn`] module"]
#[doc(alias = "CTRL1_PN")]
pub type Ctrl1Pn = crate::Reg<ctrl1_pn::Ctrl1PnSpec>;
#[doc = "Pretended Networking Control 1"]
pub mod ctrl1_pn;
#[doc = "CTRL2_PN (rw) register accessor: Pretended Networking Control 2\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl2_pn::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl2_pn::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ctrl2_pn`] module"]
#[doc(alias = "CTRL2_PN")]
pub type Ctrl2Pn = crate::Reg<ctrl2_pn::Ctrl2PnSpec>;
#[doc = "Pretended Networking Control 2"]
pub mod ctrl2_pn;
#[doc = "WU_MTC (rw) register accessor: Pretended Networking Wake-Up Match\n\nYou can [`read`](crate::Reg::read) this register and get [`wu_mtc::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`wu_mtc::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@wu_mtc`] module"]
#[doc(alias = "WU_MTC")]
pub type WuMtc = crate::Reg<wu_mtc::WuMtcSpec>;
#[doc = "Pretended Networking Wake-Up Match"]
pub mod wu_mtc;
#[doc = "FLT_ID1 (rw) register accessor: Pretended Networking ID Filter 1\n\nYou can [`read`](crate::Reg::read) this register and get [`flt_id1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flt_id1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@flt_id1`] module"]
#[doc(alias = "FLT_ID1")]
pub type FltId1 = crate::Reg<flt_id1::FltId1Spec>;
#[doc = "Pretended Networking ID Filter 1"]
pub mod flt_id1;
#[doc = "FLT_DLC (rw) register accessor: Pretended Networking Data Length Code (DLC) Filter\n\nYou can [`read`](crate::Reg::read) this register and get [`flt_dlc::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flt_dlc::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@flt_dlc`] module"]
#[doc(alias = "FLT_DLC")]
pub type FltDlc = crate::Reg<flt_dlc::FltDlcSpec>;
#[doc = "Pretended Networking Data Length Code (DLC) Filter"]
pub mod flt_dlc;
#[doc = "PL1_LO (rw) register accessor: Pretended Networking Payload Low Filter 1\n\nYou can [`read`](crate::Reg::read) this register and get [`pl1_lo::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pl1_lo::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pl1_lo`] module"]
#[doc(alias = "PL1_LO")]
pub type Pl1Lo = crate::Reg<pl1_lo::Pl1LoSpec>;
#[doc = "Pretended Networking Payload Low Filter 1"]
pub mod pl1_lo;
#[doc = "PL1_HI (rw) register accessor: Pretended Networking Payload High Filter 1\n\nYou can [`read`](crate::Reg::read) this register and get [`pl1_hi::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pl1_hi::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pl1_hi`] module"]
#[doc(alias = "PL1_HI")]
pub type Pl1Hi = crate::Reg<pl1_hi::Pl1HiSpec>;
#[doc = "Pretended Networking Payload High Filter 1"]
pub mod pl1_hi;
#[doc = "FLT_ID2_IDMASK (rw) register accessor: Pretended Networking ID Filter 2 or ID Mask\n\nYou can [`read`](crate::Reg::read) this register and get [`flt_id2_idmask::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flt_id2_idmask::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@flt_id2_idmask`] module"]
#[doc(alias = "FLT_ID2_IDMASK")]
pub type FltId2Idmask = crate::Reg<flt_id2_idmask::FltId2IdmaskSpec>;
#[doc = "Pretended Networking ID Filter 2 or ID Mask"]
pub mod flt_id2_idmask;
#[doc = "PL2_PLMASK_LO (rw) register accessor: Pretended Networking Payload Low Filter 2 and Payload Low Mask\n\nYou can [`read`](crate::Reg::read) this register and get [`pl2_plmask_lo::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pl2_plmask_lo::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pl2_plmask_lo`] module"]
#[doc(alias = "PL2_PLMASK_LO")]
pub type Pl2PlmaskLo = crate::Reg<pl2_plmask_lo::Pl2PlmaskLoSpec>;
#[doc = "Pretended Networking Payload Low Filter 2 and Payload Low Mask"]
pub mod pl2_plmask_lo;
#[doc = "PL2_PLMASK_HI (rw) register accessor: Pretended Networking Payload High Filter 2 and Payload High Mask\n\nYou can [`read`](crate::Reg::read) this register and get [`pl2_plmask_hi::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pl2_plmask_hi::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pl2_plmask_hi`] module"]
#[doc(alias = "PL2_PLMASK_HI")]
pub type Pl2PlmaskHi = crate::Reg<pl2_plmask_hi::Pl2PlmaskHiSpec>;
#[doc = "Pretended Networking Payload High Filter 2 and Payload High Mask"]
pub mod pl2_plmask_hi;
#[doc = "Array of registers: WMB_CS, WMB_D03, WMB_D47, WMB_ID"]
pub use self::wmb::Wmb;
#[doc = r"Cluster"]
#[doc = "Array of registers: WMB_CS, WMB_D03, WMB_D47, WMB_ID"]
pub mod wmb;
#[doc = "EPRS (rw) register accessor: Enhanced CAN Bit Timing Prescalers\n\nYou can [`read`](crate::Reg::read) this register and get [`eprs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`eprs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@eprs`] module"]
#[doc(alias = "EPRS")]
pub type Eprs = crate::Reg<eprs::EprsSpec>;
#[doc = "Enhanced CAN Bit Timing Prescalers"]
pub mod eprs;
#[doc = "ENCBT (rw) register accessor: Enhanced Nominal CAN Bit Timing\n\nYou can [`read`](crate::Reg::read) this register and get [`encbt::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`encbt::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@encbt`] module"]
#[doc(alias = "ENCBT")]
pub type Encbt = crate::Reg<encbt::EncbtSpec>;
#[doc = "Enhanced Nominal CAN Bit Timing"]
pub mod encbt;
#[doc = "EDCBT (rw) register accessor: Enhanced Data Phase CAN Bit Timing\n\nYou can [`read`](crate::Reg::read) this register and get [`edcbt::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`edcbt::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@edcbt`] module"]
#[doc(alias = "EDCBT")]
pub type Edcbt = crate::Reg<edcbt::EdcbtSpec>;
#[doc = "Enhanced Data Phase CAN Bit Timing"]
pub mod edcbt;
#[doc = "ETDC (rw) register accessor: Enhanced Transceiver Delay Compensation\n\nYou can [`read`](crate::Reg::read) this register and get [`etdc::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`etdc::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@etdc`] module"]
#[doc(alias = "ETDC")]
pub type Etdc = crate::Reg<etdc::EtdcSpec>;
#[doc = "Enhanced Transceiver Delay Compensation"]
pub mod etdc;
#[doc = "FDCTRL (rw) register accessor: CAN FD Control\n\nYou can [`read`](crate::Reg::read) this register and get [`fdctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fdctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@fdctrl`] module"]
#[doc(alias = "FDCTRL")]
pub type Fdctrl = crate::Reg<fdctrl::FdctrlSpec>;
#[doc = "CAN FD Control"]
pub mod fdctrl;
#[doc = "FDCBT (rw) register accessor: CAN FD Bit Timing\n\nYou can [`read`](crate::Reg::read) this register and get [`fdcbt::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fdcbt::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@fdcbt`] module"]
#[doc(alias = "FDCBT")]
pub type Fdcbt = crate::Reg<fdcbt::FdcbtSpec>;
#[doc = "CAN FD Bit Timing"]
pub mod fdcbt;
#[doc = "FDCRC (r) register accessor: CAN FD CRC\n\nYou can [`read`](crate::Reg::read) this register and get [`fdcrc::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@fdcrc`] module"]
#[doc(alias = "FDCRC")]
pub type Fdcrc = crate::Reg<fdcrc::FdcrcSpec>;
#[doc = "CAN FD CRC"]
pub mod fdcrc;
#[doc = "ERFCR (rw) register accessor: Enhanced RX FIFO Control\n\nYou can [`read`](crate::Reg::read) this register and get [`erfcr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`erfcr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@erfcr`] module"]
#[doc(alias = "ERFCR")]
pub type Erfcr = crate::Reg<erfcr::ErfcrSpec>;
#[doc = "Enhanced RX FIFO Control"]
pub mod erfcr;
#[doc = "ERFIER (rw) register accessor: Enhanced RX FIFO Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`erfier::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`erfier::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@erfier`] module"]
#[doc(alias = "ERFIER")]
pub type Erfier = crate::Reg<erfier::ErfierSpec>;
#[doc = "Enhanced RX FIFO Interrupt Enable"]
pub mod erfier;
#[doc = "ERFSR (rw) register accessor: Enhanced RX FIFO Status\n\nYou can [`read`](crate::Reg::read) this register and get [`erfsr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`erfsr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@erfsr`] module"]
#[doc(alias = "ERFSR")]
pub type Erfsr = crate::Reg<erfsr::ErfsrSpec>;
#[doc = "Enhanced RX FIFO Status"]
pub mod erfsr;
#[doc = "ERFFEL (rw) register accessor: Enhanced RX FIFO Filter Element\n\nYou can [`read`](crate::Reg::read) this register and get [`erffel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`erffel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@erffel`] module"]
#[doc(alias = "ERFFEL")]
pub type Erffel = crate::Reg<erffel::ErffelSpec>;
#[doc = "Enhanced RX FIFO Filter Element"]
pub mod erffel;

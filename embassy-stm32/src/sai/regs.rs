//! SAI register write helpers for PAC variants without typed field setters.

use super::{Config, TxRx};
use crate::pac::sai::Ch;

#[cfg(not(sai_n6))]
pub(crate) fn configure_cr1(ch: Ch, config: &Config, tx_rx: TxRx) {
    let mode = config.mode.mode(tx_rx);

    ch.cr1().modify(|w| {
        w.set_mode(mode);
        w.set_prtcfg(config.protocol.prtcfg());
        w.set_ds(config.data_size.ds());
        w.set_lsbfirst(config.bit_order.lsbfirst());
        w.set_ckstr(config.clock_strobe.ckstr());
        w.set_syncen(config.sync_input.syncen());
        w.set_mono(config.stereo_mono.mono());
        w.set_outdriv(config.output_drive.outdriv());
        w.set_mckdiv(config.master_clock_divider);
        w.set_nodiv(config.nodiv);
        w.set_dmaen(true);
    });
}

#[cfg(sai_n6)]
pub(crate) fn configure_cr1(ch: Ch, config: &Config, tx_rx: TxRx) {
    let mode = config.mode.mode(tx_rx);

    ch.cr1().modify(|w| {
        w.set_mode(mode.to_bits());
        w.set_prtcfg(config.protocol.prtcfg().to_bits());
        w.set_ds(config.data_size.ds().to_bits());
        w.set_lsbfirst(config.bit_order.lsbfirst().to_bits() != 0);
        w.set_ckstr(config.clock_strobe.ckstr().to_bits() != 0);
        w.set_syncen(config.sync_input.syncen().to_bits());
        w.set_mono(config.stereo_mono.mono().to_bits() != 0);
        w.set_outdriv(config.output_drive.outdriv().to_bits() != 0);
        w.set_mckdiv(config.master_clock_divider.to_bits());
        w.set_nodiv(config.nodiv);
        w.set_dmaen(true);
    });
}

#[cfg(not(sai_n6))]
pub(crate) fn configure_cr2(ch: Ch, config: &Config) {
    ch.cr2().modify(|w| {
        w.set_fth(config.fifo_threshold.fth());
        w.set_comp(config.companding.comp());
        w.set_cpl(config.complement_format.cpl());
        w.set_muteval(config.mute_value.muteval());
        w.set_mutecnt(config.mute_detection_counter.0 as u8);
        w.set_tris(config.is_high_impedance_on_inactive_slot);
    });
}

#[cfg(sai_n6)]
pub(crate) fn configure_cr2(ch: Ch, config: &Config) {
    ch.cr2().modify(|w| {
        w.set_fth(config.fifo_threshold.fth().to_bits());
        w.set_comp(config.companding.comp().to_bits());
        w.set_cpl(config.complement_format.cpl().to_bits() != 0);
        w.set_muteval(config.mute_value.muteval().to_bits() != 0);
        w.set_mutecnt(config.mute_detection_counter.0 as u8);
        w.set_tris(config.is_high_impedance_on_inactive_slot);
    });
}

#[cfg(not(sai_n6))]
pub(crate) fn configure_frcr(ch: Ch, config: &Config) {
    ch.frcr().modify(|w| {
        w.set_fsoff(config.frame_sync_offset.fsoff());
        w.set_fspol(config.frame_sync_polarity.fspol());
        w.set_fsdef(config.frame_sync_definition.fsdef());
        w.set_fsall(config.frame_sync_active_level_length.0 as u8 - 1);
        w.set_frl((config.frame_length - 1).try_into().unwrap());
    });
}

#[cfg(sai_n6)]
pub(crate) fn configure_frcr(ch: Ch, config: &Config) {
    ch.frcr().modify(|w| {
        w.set_fsoff(config.frame_sync_offset.fsoff().to_bits() != 0);
        w.set_fspol(config.frame_sync_polarity.fspol().to_bits() != 0);
        w.set_fsdef(config.frame_sync_definition.fsdef());
        w.set_fsall(config.frame_sync_active_level_length.0 as u8 - 1);
        w.set_frl((config.frame_length - 1).try_into().unwrap());
    });
}

#[cfg(not(sai_n6))]
pub(crate) fn configure_slotr(ch: Ch, config: &Config) {
    ch.slotr().modify(|w| {
        w.set_nbslot(config.slot_count.0 as u8 - 1);
        w.set_fboff(config.first_bit_offset.0 as u8);
        w.set_slotsz(config.slot_size.slotsz());
        w.set_sloten(super::vals::Sloten::from_bits(config.slot_enable as u16));
    });
}

#[cfg(sai_n6)]
pub(crate) fn configure_slotr(ch: Ch, config: &Config) {
    ch.slotr().modify(|w| {
        w.set_nbslot(config.slot_count.0 as u8 - 1);
        w.set_fboff(config.first_bit_offset.0 as u8);
        w.set_slotsz(config.slot_size.slotsz().to_bits());
        w.set_sloten(config.slot_enable);
    });
}

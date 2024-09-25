use super::Info;

pub fn calc_ns_per_timer_tick(
    info: &'static Info,
    freq: crate::time::Hertz,
    mode: crate::can::fd::config::CanFdMode,
) -> u64 {
    match mode {
        // Use timestamp from Rx FIFO to adjust timestamp reported to user
        crate::can::fd::config::CanFdMode::ClassicCanOnly => {
            let prescale: u64 = ({ info.low.regs.nbtp().read().nbrp() } + 1) as u64
                * ({ info.low.regs.tscc().read().tcp() } + 1) as u64;
            1_000_000_000 as u64 / (freq.0 as u64 * prescale)
        }
        // For VBR this is too hard because the FDCAN timer switches clock rate you need to configure to use
        // timer3 instead which is too hard to do from this module.
        _ => 0,
    }
}

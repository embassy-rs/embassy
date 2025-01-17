use core::{
    marker::PhantomData,
    sync::atomic::{AtomicU32, Ordering},
};

use bit_field::BitField;

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

/// There is no good way from what I can tell to initalize a
/// `[AtomicU32; N]` for a N const generic parameter.
/// Initializing it as zero should be safe because an atomic is
/// the same size as its underling type, so this should be safe to
/// do.
pub const fn new_atomic_u32_array<const N: usize>() -> [AtomicU32; N] {
    static_assertions::const_assert!(core::mem::size_of::<AtomicU32>() == 4);
    unsafe { core::mem::zeroed() }
}

pub struct AtomicResourceAllocator<const N: usize, const S: usize> {
    _phantom: PhantomData<[(); N]>,
    atomics: [AtomicU32; S],
}

impl<const N: usize, const S: usize> AtomicResourceAllocator<N, S> {
    pub const fn new() -> Self {
        AtomicResourceAllocator {
            _phantom: PhantomData,
            atomics: new_atomic_u32_array(),
        }
    }

    pub fn allocate(&self) -> Option<usize> {
        for n in 0..S {
            let val = self.atomics[n].load(Ordering::Relaxed);
            loop {
                let free_idx = val.leading_ones() as usize;

                if free_idx == 32 {
                    break;
                }

                let max_slots = (N + 31) / 32;
                let max_last_bits = N % 32;
                if n == max_slots && free_idx >= max_last_bits {
                    break;
                }

                let mask = 1 << free_idx;
                match self.atomics[n].compare_exchange(val, val | mask, Ordering::Relaxed, Ordering::Relaxed) {
                    Ok(_) => {
                        let slot_idx = free_idx * n;
                        if slot_idx < N {
                            return Some(slot_idx);
                        }
                    }
                    Err(_) => (),
                }
            }
        }
        None
    }

    pub fn deallocate(&self, index: usize) {
        let atomic_idx = index % 32;
        let bit_idx = index / 32;
        let prev = self.atomics[atomic_idx].fetch_and(!(1 << bit_idx), Ordering::Relaxed);
        assert!(
            prev.get_bit(bit_idx),
            "attempted to deallocate a non allocated resource!"
        );
    }
}

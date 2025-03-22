//!
//! This example shows how to continuously sample from the PDM hardware peripheral
//! on the nRF52840. It uses the double-buffering, zero-copy API. It is efficient
//! and allows use of more than 2 buffers if there is a need to handle
//! bursty/slow data processing.
//!
//! It is compatible with static, pool, and dynamic allocation of buffers.
//!

#![no_std]
#![no_main]

use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::Relaxed;

use embassy_executor::Spawner;
use embassy_nrf::pdm::{self, Config, DoubleBufferSampleState, Frequency, OperationMode, Pdm, Ratio};
use embassy_nrf::{bind_interrupts, peripherals};
use fixed::types::I7F1;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PDM => pdm::InterruptHandler<peripherals::PDM>;
});

const PDM_WORDS: usize = 1024;

// ///
// /// Example dynamic allocation function for the zero-copy API.
// ///
// /// Returns the raw parts of a Vec<i16> that has been leaked:
// /// * A pointer to the buffer
// /// * The capacity of the buffer
// /// * The length of the buffer
// ///
// fn alloc_pdm_vec() -> (*mut i16, usize, usize) {
//     let mut buf = Vec::<i16>::with_capacity(PDM_WORDS);
//     let ptr = buf.as_mut_ptr();
//     buf.leak();
//     (ptr, PDM_WORDS, PDM_WORDS)
// };

// ///
// /// Example dynamic deallocation function for the zero-copy API.
// ///
// fn dealloc_pdm_vec(ptr: *mut i16, length: usize) {
//     unsafe { Vec::from_raw_parts(ptr, length, length) };
//  }

///
/// Example static allocation for the zero-copy API
///
/// Returns a pointer to a static array of i16, where the length is the capacity.
/// * A pointer to the buffer
/// * The capacity of the buffer
/// * The length of the buffer
///
fn alloc_pdm_static() -> (*mut i16, usize, usize) {
    static mut BUF: [[i16; PDM_WORDS]; 10] = [[0; PDM_WORDS]; 10];
    static mut BUF_IDX: usize = 0;
    let ptr = unsafe {
        BUF_IDX = (BUF_IDX + 1) % BUF.len();
        BUF[BUF_IDX].as_mut_ptr()
    };
    (ptr, PDM_WORDS, PDM_WORDS)
}

///
/// Example static deallocation function for the zero-copy API.
///
fn dealloc_pdm_static(_ptr: *mut i16, _capacity: usize, _length: usize) {
    // Nothing to do here, we are passing static buffers
}

#[embassy_executor::main]
async fn main(_p: Spawner) {
    let mut p = embassy_nrf::init(Default::default());
    let mut config = Config::default();
    // The output data rate is the frequency / ratio, so 1.28MHz / 64 = 16kHz
    // Minimum data rate is 1MHz / 80 = 12.5kHz
    // Maximum data rate is 1.33MHz / 64 = 20.8kHz
    config.frequency = Frequency::_1280K;
    config.ratio = Ratio::RATIO80;
    config.operation_mode = OperationMode::Mono;
    // GPDM,default=3.2 dB
    config.gain_left = I7F1::from_num(10); // 10 + 3.2 = 13.2 dB gain
    let mut pdm = Pdm::new(p.PDM, Irqs, &mut p.P0_00, &mut p.P0_01, config);

    // Get buffers from a static, pool, or dynamic allocation
    // Note, these are leaked and will need to be manually deallocated appropriately
    let alloc = alloc_pdm_static;
    let dealloc = dealloc_pdm_static;
    let mut pdm_bufs = [alloc(), alloc()];

    #[allow(unused_mut)]
    // Somewhere else in the program, you may want to be able to stop the PDM
    // The task sampler may return DoubleBufferSampleState::Stop to halt the PDM
    let mut early_stopping_flag: AtomicBool = AtomicBool::new(false);

    pdm.run_double_buffered(&mut pdm_bufs, move |buf| {
        // If dynamically allocated, create a Vec from the raw parts
        // let buf = unsafe { Vec::from_raw_parts(buf.0, buf.1, buf.2) };

        // If statically allocated, create a slice from the raw parts
        let pdm_words: &[i16] = unsafe { core::slice::from_raw_parts_mut(buf.0, buf.1) };

        // Do what you want with data here, or send it to another task to prevent taking too long here
        let (min, max) = pdm_words.iter().fold((0, 0), |(min, max), &x| {
            (core::cmp::min(min, x), core::cmp::max(max, x))
        });
        defmt::info!("ptr: {}, min: {}, max: {}", buf.0, min, max);

        // When stopping, you need to ensure that the buf is dropped appropriately
        if early_stopping_flag.load(Relaxed) {
            return DoubleBufferSampleState::Stop;
        }

        // Give the next buffer to use for double buffering
        let next_buf = alloc();
        DoubleBufferSampleState::Swap(next_buf)
    })
    .await
    .unwrap();

    // On Stop, one of the buffers was not swapped, so we need to deallocate it
    for buf in pdm_bufs {
        if buf.0 as usize != 0 {
            dealloc(buf.0, buf.1, buf.2);
        }
    }
}

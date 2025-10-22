#![no_std]
#![no_main]

#[cfg(feature = "mspm0g3507")]
teleprobe_meta::target!(b"lp-mspm0g3507");

#[cfg(feature = "mspm0g3519")]
teleprobe_meta::target!(b"lp-mspm0g3519");

use core::slice;

use defmt::{assert, assert_eq, *};
use embassy_executor::Spawner;
use embassy_mspm0::Peri;
use embassy_mspm0::dma::{Channel, Transfer, TransferMode, TransferOptions, Word};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_mspm0::init(Default::default());
    info!("Hello World!");

    {
        info!("Single u8 read (blocking)");
        single_read(p.DMA_CH0.reborrow(), 0x41_u8);

        info!("Single u16 read (blocking)");
        single_read(p.DMA_CH0.reborrow(), 0xFF41_u16);

        info!("Single u32 read (blocking)");
        single_read(p.DMA_CH0.reborrow(), 0xFFEE_FF41_u32);

        info!("Single u64 read (blocking)");
        single_read(p.DMA_CH0.reborrow(), 0x0011_2233_FFEE_FF41_u64);
    }

    // Widening transfers
    {
        info!("Single u8 read to u16");
        widening_single_read::<u8, u16>(p.DMA_CH0.reborrow(), 0x41);

        info!("Single u8 read to u32");
        widening_single_read::<u8, u32>(p.DMA_CH0.reborrow(), 0x43);

        info!("Single u8 read to u64");
        widening_single_read::<u8, u64>(p.DMA_CH0.reborrow(), 0x47);

        info!("Single u16 read to u32");
        widening_single_read::<u16, u32>(p.DMA_CH0.reborrow(), 0xAE43);

        info!("Single u16 read to u64");
        widening_single_read::<u16, u64>(p.DMA_CH0.reborrow(), 0xAF47);

        info!("Single u32 read to u64");
        widening_single_read::<u32, u64>(p.DMA_CH0.reborrow(), 0xDEAD_AF47);
    }

    // Narrowing transfers.
    {
        info!("Single u16 read to u8");
        narrowing_single_read::<u16, u8>(p.DMA_CH0.reborrow(), 0x4142);

        info!("Single u32 read to u8");
        narrowing_single_read::<u32, u8>(p.DMA_CH0.reborrow(), 0x4142_2414);

        info!("Single u64 read to u8");
        narrowing_single_read::<u64, u8>(p.DMA_CH0.reborrow(), 0x4142_2414_5153_7776);

        info!("Single u32 read to u16");
        narrowing_single_read::<u32, u16>(p.DMA_CH0.reborrow(), 0x4142_2414);

        info!("Single u64 read to u16");
        narrowing_single_read::<u64, u16>(p.DMA_CH0.reborrow(), 0x4142_2414_5153_7776);

        info!("Single u64 read to u32");
        narrowing_single_read::<u64, u32>(p.DMA_CH0.reborrow(), 0x4142_2414_5153_7776);
    }

    {
        info!("Single u8 read (async)");
        async_single_read(p.DMA_CH0.reborrow(), 0x42_u8).await;

        info!("Single u16 read (async)");
        async_single_read(p.DMA_CH0.reborrow(), 0xAE42_u16).await;

        info!("Single u32 read (async)");
        async_single_read(p.DMA_CH0.reborrow(), 0xFE44_1500_u32).await;

        info!("Single u64 read (async)");
        async_single_read(p.DMA_CH0.reborrow(), 0x8F7F_6F5F_4F3F_2F1F_u64).await;
    }

    {
        info!("Multiple u8 reads (blocking)");
        block_read::<_, 16>(p.DMA_CH0.reborrow(), 0x98_u8);

        info!("Multiple u16 reads (blocking)");
        block_read::<_, 2>(p.DMA_CH0.reborrow(), 0x9801_u16);

        info!("Multiple u32 reads (blocking)");
        block_read::<_, 4>(p.DMA_CH0.reborrow(), 0x9821_9801_u32);

        info!("Multiple u64 reads (blocking)");
        block_read::<_, 4>(p.DMA_CH0.reborrow(), 0xABCD_EF01_2345_6789_u64);
    }

    {
        info!("Multiple u8 reads (async)");
        async_block_read::<_, 8>(p.DMA_CH0.reborrow(), 0x86_u8).await;

        info!("Multiple u16 reads (async)");
        async_block_read::<_, 6>(p.DMA_CH0.reborrow(), 0x7777_u16).await;

        info!("Multiple u32 reads (async)");
        async_block_read::<_, 3>(p.DMA_CH0.reborrow(), 0xA5A5_A5A5_u32).await;

        info!("Multiple u64 reads (async)");
        async_block_read::<_, 14>(p.DMA_CH0.reborrow(), 0x5A5A_5A5A_A5A5_A5A5_u64).await;
    }

    // Intentionally skip testing multiple reads in single transfer mode.
    //
    // If the destination length is greater than 1 and single transfer mode is used then two transfers
    // are performed in a trigger. Similarly with any other length of destination above 2, only 2 transfers
    // are performed. Issuing another trigger (resume) results in no further progress. More than likely
    // the test does not work due to some combination of a hardware bug and the datasheet being unclear
    // regarding what ends a software trigger.
    //
    // However this case works fine with a hardware trigger (such as the ADC hardware trigger).

    {
        info!("Single u8 write (blocking)");
        single_write(p.DMA_CH0.reborrow(), 0x41_u8);

        info!("Single u16 write (blocking)");
        single_write(p.DMA_CH0.reborrow(), 0x4142_u16);

        info!("Single u32 write (blocking)");
        single_write(p.DMA_CH0.reborrow(), 0x4142_4344_u32);

        info!("Single u64 write (blocking)");
        single_write(p.DMA_CH0.reborrow(), 0x4142_4344_4546_4748_u64);
    }

    {
        info!("Single u8 write (async)");
        async_single_write(p.DMA_CH0.reborrow(), 0xAA_u8).await;

        info!("Single u16 write (async)");
        async_single_write(p.DMA_CH0.reborrow(), 0xBBBB_u16).await;

        info!("Single u32 write (async)");
        async_single_write(p.DMA_CH0.reborrow(), 0xCCCC_CCCC_u32).await;

        info!("Single u64 write (async)");
        async_single_write(p.DMA_CH0.reborrow(), 0xDDDD_DDDD_DDDD_DDDD_u64).await;
    }

    {
        info!("Multiple u8 writes (blocking)");
        block_write(p.DMA_CH0.reborrow(), &[0xFF_u8, 0x7F, 0x3F, 0x1F]);

        info!("Multiple u16 writes (blocking)");
        block_write(p.DMA_CH0.reborrow(), &[0xFFFF_u16, 0xFF7F, 0xFF3F, 0xFF1F]);

        info!("Multiple u32 writes (blocking)");
        block_write(
            p.DMA_CH0.reborrow(),
            &[0xFF00_00FF_u32, 0xFF00_007F, 0x0000_FF3F, 0xFF1F_0000],
        );

        info!("Multiple u64 writes (blocking)");
        block_write(
            p.DMA_CH0.reborrow(),
            &[
                0xFF00_0000_0000_00FF_u64,
                0x0000_FF00_007F_0000,
                0x0000_FF3F_0000_0000,
                0xFF1F_0000_1111_837A,
            ],
        );
    }

    {
        info!("Multiple u8 writes (async)");
        async_block_write(p.DMA_CH0.reborrow(), &[0u8, 1, 2, 3]).await;

        info!("Multiple u16 writes (async)");
        async_block_write(p.DMA_CH0.reborrow(), &[0x9801u16, 0x9802, 0x9803, 0x9800, 0x9000]).await;

        info!("Multiple u32 writes (async)");
        async_block_write(p.DMA_CH0.reborrow(), &[0x9801_ABCDu32, 0xFFAC_9802, 0xDEAD_9803]).await;

        info!("Multiple u64 writes (async)");
        async_block_write(
            p.DMA_CH0.reborrow(),
            &[
                0xA55A_1111_3333_5555_u64,
                0x1111_A55A_3333_5555,
                0x5555_A55A_3333_1111,
                0x01234_5678_89AB_CDEF,
            ],
        )
        .await;
    }

    // TODO: Mixed byte and word transfers.

    info!("Test OK");
    cortex_m::asm::bkpt();
}

fn single_read<W: Word + Copy + Default + Eq + defmt::Format>(mut channel: Peri<'_, impl Channel>, mut src: W) {
    let options = TransferOptions::default();
    let mut dst = W::default();

    // SAFETY: src and dst outlive the transfer.
    let transfer = unsafe {
        unwrap!(Transfer::new_read(
            channel.reborrow(),
            Transfer::SOFTWARE_TRIGGER,
            &mut src,
            slice::from_mut(&mut dst),
            options,
        ))
    };
    transfer.blocking_wait();

    assert_eq!(src, dst);
}

async fn async_single_read<W: Word + Copy + Default + Eq + defmt::Format>(
    mut channel: Peri<'_, impl Channel>,
    mut src: W,
) {
    let options = TransferOptions::default();
    let mut dst = W::default();

    // SAFETY: src and dst outlive the transfer.
    let transfer = unsafe {
        unwrap!(Transfer::new_read(
            channel.reborrow(),
            Transfer::SOFTWARE_TRIGGER,
            &mut src,
            slice::from_mut(&mut dst),
            options,
        ))
    };
    transfer.await;

    assert_eq!(src, dst);
}

fn block_read<W: Word + Copy + Default + Eq + defmt::Format, const N: usize>(
    mut channel: Peri<'_, impl Channel>,
    mut src: W,
) {
    let mut options = TransferOptions::default();
    // Complete the entire transfer.
    options.mode = TransferMode::Block;

    let mut dst = [W::default(); N];

    // SAFETY: src and dst outlive the transfer.
    let transfer = unsafe {
        unwrap!(Transfer::new_read(
            channel.reborrow(),
            Transfer::SOFTWARE_TRIGGER,
            &mut src,
            &mut dst[..],
            options,
        ))
    };
    transfer.blocking_wait();

    assert_eq!(dst, [src; N]);
}

async fn async_block_read<W: Word + Copy + Default + Eq + defmt::Format, const N: usize>(
    mut channel: Peri<'_, impl Channel>,
    mut src: W,
) {
    let mut options = TransferOptions::default();
    // Complete the entire transfer.
    options.mode = TransferMode::Block;

    let mut dst = [W::default(); N];

    // SAFETY: src and dst outlive the transfer.
    let transfer = unsafe {
        unwrap!(Transfer::new_read(
            channel.reborrow(),
            Transfer::SOFTWARE_TRIGGER,
            &mut src,
            &mut dst[..],
            options,
        ))
    };
    transfer.await;

    assert_eq!(dst, [src; N]);
}

fn single_write<W: Word + Default + Eq + defmt::Format>(mut channel: Peri<'_, impl Channel>, src: W) {
    let options = TransferOptions::default();
    let mut dst = W::default();

    // SAFETY: src and dst outlive the transfer.
    let transfer = unsafe {
        unwrap!(Transfer::new_write(
            channel.reborrow(),
            Transfer::SOFTWARE_TRIGGER,
            slice::from_ref(&src),
            &mut dst,
            options,
        ))
    };
    transfer.blocking_wait();

    assert_eq!(src, dst);
}

async fn async_single_write<W: Word + Default + Eq + defmt::Format>(mut channel: Peri<'_, impl Channel>, src: W) {
    let options = TransferOptions::default();
    let mut dst = W::default();

    // SAFETY: src and dst outlive the transfer.
    let transfer = unsafe {
        unwrap!(Transfer::new_write(
            channel.reborrow(),
            Transfer::SOFTWARE_TRIGGER,
            slice::from_ref(&src),
            &mut dst,
            options,
        ))
    };
    transfer.await;

    assert_eq!(src, dst);
}

fn block_write<W: Word + Default + Eq + defmt::Format>(mut channel: Peri<'_, impl Channel>, src: &[W]) {
    let mut options = TransferOptions::default();
    // Complete the entire transfer.
    options.mode = TransferMode::Block;

    let mut dst = W::default();

    // Starting from 1 because a zero length transfer does nothing.
    for i in 1..src.len() {
        info!("-> {} write(s)", i);

        // SAFETY: src and dst outlive the transfer.
        let transfer = unsafe {
            unwrap!(Transfer::new_write(
                channel.reborrow(),
                Transfer::SOFTWARE_TRIGGER,
                &src[..i],
                &mut dst,
                options,
            ))
        };
        transfer.blocking_wait();

        // The result will be the last value written.
        assert_eq!(dst, src[i - 1]);
    }
}

async fn async_block_write<W: Word + Default + Eq + defmt::Format>(mut channel: Peri<'_, impl Channel>, src: &[W]) {
    let mut options = TransferOptions::default();
    // Complete the entire transfer.
    options.mode = TransferMode::Block;

    let mut dst = W::default();

    // Starting from 1 because a zero length transfer does nothing.
    for i in 1..src.len() {
        info!("-> {} write(s)", i);
        // SAFETY: src and dst outlive the transfer.
        let transfer = unsafe {
            unwrap!(Transfer::new_write(
                channel.reborrow(),
                Transfer::SOFTWARE_TRIGGER,
                &src[..i],
                &mut dst,
                options,
            ))
        };
        transfer.await;

        // The result will be the last value written.
        assert_eq!(dst, src[i - 1]);
    }
}

/// [`single_read`], but testing when the destination is wider than the source.
///
/// The MSPM0 DMA states that the upper bytes when the destination is longer than the source are zeroed.
/// This matches the behavior in Rust for all unsigned integer types.
fn widening_single_read<SW, DW>(mut channel: Peri<'_, impl Channel>, mut src: SW)
where
    SW: Word + Copy + Default + Eq + defmt::Format,
    DW: Word + Copy + Default + Eq + defmt::Format + From<SW>,
{
    assert!(
        DW::size() > SW::size(),
        "This test only works when the destination is larger than the source"
    );

    let options = TransferOptions::default();
    let mut dst = DW::default();

    // SAFETY: src and dst outlive the transfer.
    let transfer = unsafe {
        unwrap!(Transfer::new_read(
            channel.reborrow(),
            Transfer::SOFTWARE_TRIGGER,
            &mut src,
            slice::from_mut(&mut dst),
            options,
        ))
    };
    transfer.blocking_wait();

    assert_eq!(DW::from(src), dst);
}

/// [`single_read`], but testing when the destination is narrower than the source.
///
/// The MSPM0 DMA states that the upper bytes when the source is longer than the destination are dropped.
/// This matches the behavior in Rust for all unsigned integer types.
fn narrowing_single_read<SW, DW>(mut channel: Peri<'_, impl Channel>, mut src: SW)
where
    SW: Word + Copy + Default + Eq + defmt::Format + From<DW>,
    DW: Word + Copy + Default + Eq + defmt::Format + Narrow<SW>,
{
    assert!(
        SW::size() > DW::size(),
        "This test only works when the source is larger than the destination"
    );

    let options = TransferOptions::default();
    let mut dst = DW::default();

    // SAFETY: src and dst outlive the transfer.
    let transfer = unsafe {
        unwrap!(Transfer::new_read(
            channel.reborrow(),
            Transfer::SOFTWARE_TRIGGER,
            &mut src,
            slice::from_mut(&mut dst),
            options,
        ))
    };
    transfer.blocking_wait();

    // The expected value is the source value masked by the maximum destination value.
    // This is effectively `src as DW as SW` to drop the upper byte(s).
    let expect = SW::from(DW::narrow(src));
    assert_eq!(expect, dst.into());
}

/// A pseudo `as` trait to allow downcasting integer types (TryFrom could fail).
trait Narrow<T> {
    fn narrow(value: T) -> Self;
}

impl Narrow<u16> for u8 {
    fn narrow(value: u16) -> Self {
        value as u8
    }
}

impl Narrow<u32> for u8 {
    fn narrow(value: u32) -> Self {
        value as u8
    }
}

impl Narrow<u64> for u8 {
    fn narrow(value: u64) -> Self {
        value as u8
    }
}

impl Narrow<u32> for u16 {
    fn narrow(value: u32) -> Self {
        value as u16
    }
}

impl Narrow<u64> for u16 {
    fn narrow(value: u64) -> Self {
        value as u16
    }
}

impl Narrow<u64> for u32 {
    fn narrow(value: u64) -> Self {
        value as u32
    }
}

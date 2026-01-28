//! Serial Audio Interface (SAI)
#![macro_use]

use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;

pub use crate::dma::word;
use crate::dma::{
    self, AnyChannel, Channel, ReadableRingBuffer, Request, TransferOptions, TypedChannel, WritableRingBuffer,
    dma_into, ringbuffer,
};
use crate::gpio::{AfType, AnyPin, OutputType, Pull, SealedPin as _, Speed};
pub use crate::pac::sai::vals::Mckdiv as MasterClockDivider;
use crate::pac::sai::{Sai as Regs, vals};
use crate::rcc::{self, RccPeripheral};
use crate::{Peri, interrupt, peripherals};

/// SAI error
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// `write` called on a SAI in receive mode.
    NotATransmitter,
    /// `read` called on a SAI in transmit mode.
    NotAReceiver,
    /// Overrun
    Overrun,
}

impl From<ringbuffer::Error> for Error {
    fn from(#[allow(unused)] err: ringbuffer::Error) -> Self {
        #[cfg(feature = "defmt")]
        {
            if err == ringbuffer::Error::DmaUnsynced {
                defmt::error!("Ringbuffer broken invariants detected!");
            }
        }
        Self::Overrun
    }
}

/// Master/slave mode.
#[derive(Copy, Clone)]
#[allow(missing_docs)]
pub enum Mode {
    Master,
    Slave,
}

impl Mode {
    const fn mode(&self, tx_rx: TxRx) -> vals::Mode {
        match tx_rx {
            TxRx::Transmitter => match self {
                Mode::Master => vals::Mode::MASTER_TX,
                Mode::Slave => vals::Mode::SLAVE_TX,
            },
            TxRx::Receiver => match self {
                Mode::Master => vals::Mode::MASTER_RX,
                Mode::Slave => vals::Mode::SLAVE_RX,
            },
        }
    }
}

/// Direction: transmit or receive
#[derive(Copy, Clone)]
#[allow(missing_docs)]
pub enum TxRx {
    Transmitter,
    Receiver,
}

/// Data slot size.
#[derive(Copy, Clone)]
#[allow(missing_docs)]
pub enum SlotSize {
    DataSize,
    /// 16 bit data length on 16 bit wide channel
    Channel16,
    /// 16 bit data length on 32 bit wide channel
    Channel32,
}

impl SlotSize {
    const fn slotsz(&self) -> vals::Slotsz {
        match self {
            SlotSize::DataSize => vals::Slotsz::DATA_SIZE,
            SlotSize::Channel16 => vals::Slotsz::BIT16,
            SlotSize::Channel32 => vals::Slotsz::BIT32,
        }
    }
}

/// Data size.
#[derive(Copy, Clone)]
#[allow(missing_docs)]
pub enum DataSize {
    Data8,
    Data10,
    Data16,
    Data20,
    Data24,
    Data32,
}

impl DataSize {
    const fn ds(&self) -> vals::Ds {
        match self {
            DataSize::Data8 => vals::Ds::BIT8,
            DataSize::Data10 => vals::Ds::BIT10,
            DataSize::Data16 => vals::Ds::BIT16,
            DataSize::Data20 => vals::Ds::BIT20,
            DataSize::Data24 => vals::Ds::BIT24,
            DataSize::Data32 => vals::Ds::BIT32,
        }
    }
}

/// FIFO threshold level.
#[derive(Copy, Clone)]
#[allow(missing_docs)]
pub enum FifoThreshold {
    Empty,
    Quarter,
    Half,
    ThreeQuarters,
    Full,
}

impl FifoThreshold {
    const fn fth(&self) -> vals::Fth {
        match self {
            FifoThreshold::Empty => vals::Fth::EMPTY,
            FifoThreshold::Quarter => vals::Fth::QUARTER1,
            FifoThreshold::Half => vals::Fth::QUARTER2,
            FifoThreshold::ThreeQuarters => vals::Fth::QUARTER3,
            FifoThreshold::Full => vals::Fth::FULL,
        }
    }
}

/// Output value on mute.
#[derive(Copy, Clone)]
#[allow(missing_docs)]
pub enum MuteValue {
    Zero,
    LastValue,
}

impl MuteValue {
    const fn muteval(&self) -> vals::Muteval {
        match self {
            MuteValue::Zero => vals::Muteval::SEND_ZERO,
            MuteValue::LastValue => vals::Muteval::SEND_LAST,
        }
    }
}

/// Protocol variant to use.
#[derive(Copy, Clone)]
#[allow(missing_docs)]
pub enum Protocol {
    Free,
    Spdif,
    Ac97,
}

impl Protocol {
    const fn prtcfg(&self) -> vals::Prtcfg {
        match self {
            Protocol::Free => vals::Prtcfg::FREE,
            Protocol::Spdif => vals::Prtcfg::SPDIF,
            Protocol::Ac97 => vals::Prtcfg::AC97,
        }
    }
}

/// Sync input between SAI units/blocks.
#[derive(Copy, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum SyncInput {
    /// Not synced to any other SAI unit.
    None,
    /// Syncs with the other A/B sub-block within the SAI unit
    Internal,
    /// Syncs with a sub-block in the other SAI unit
    #[cfg(any(sai_v3, sai_v4))]
    External(SyncInputInstance),
}

impl SyncInput {
    const fn syncen(&self) -> vals::Syncen {
        match self {
            SyncInput::None => vals::Syncen::ASYNCHRONOUS,
            SyncInput::Internal => vals::Syncen::INTERNAL,
            #[cfg(any(sai_v3, sai_v4))]
            SyncInput::External(_) => vals::Syncen::EXTERNAL,
        }
    }
}

/// SAI instance to sync from.
#[cfg(any(sai_v3, sai_v4))]
#[derive(Copy, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum SyncInputInstance {
    #[cfg(peri_sai1)]
    Sai1 = 0,
    #[cfg(peri_sai2)]
    Sai2 = 1,
    #[cfg(peri_sai3)]
    Sai3 = 2,
    #[cfg(peri_sai4)]
    Sai4 = 3,
}

/// Channels (stereo or mono).
#[derive(Copy, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum StereoMono {
    Stereo,
    Mono,
}

impl StereoMono {
    const fn mono(&self) -> vals::Mono {
        match self {
            StereoMono::Stereo => vals::Mono::STEREO,
            StereoMono::Mono => vals::Mono::MONO,
        }
    }
}

/// Bit order
#[derive(Copy, Clone)]
pub enum BitOrder {
    /// Least significant bit first.
    LsbFirst,
    /// Most significant bit first.
    MsbFirst,
}

impl BitOrder {
    const fn lsbfirst(&self) -> vals::Lsbfirst {
        match self {
            BitOrder::LsbFirst => vals::Lsbfirst::LSB_FIRST,
            BitOrder::MsbFirst => vals::Lsbfirst::MSB_FIRST,
        }
    }
}

/// Frame sync offset.
#[derive(Copy, Clone)]
pub enum FrameSyncOffset {
    /// This is used in modes other than standard I2S phillips mode
    OnFirstBit,
    /// This is used in standard I2S phillips mode
    BeforeFirstBit,
}

impl FrameSyncOffset {
    const fn fsoff(&self) -> vals::Fsoff {
        match self {
            FrameSyncOffset::OnFirstBit => vals::Fsoff::ON_FIRST,
            FrameSyncOffset::BeforeFirstBit => vals::Fsoff::BEFORE_FIRST,
        }
    }
}

/// Frame sync polarity
#[derive(Copy, Clone)]
pub enum FrameSyncPolarity {
    /// Sync signal is active low.
    ActiveLow,
    /// Sync signal is active high
    ActiveHigh,
}

impl FrameSyncPolarity {
    const fn fspol(&self) -> vals::Fspol {
        match self {
            FrameSyncPolarity::ActiveLow => vals::Fspol::FALLING_EDGE,
            FrameSyncPolarity::ActiveHigh => vals::Fspol::RISING_EDGE,
        }
    }
}

/// Sync definition.
#[derive(Copy, Clone)]
#[allow(missing_docs)]
pub enum FrameSyncDefinition {
    StartOfFrame,
    ChannelIdentification,
}

impl FrameSyncDefinition {
    const fn fsdef(&self) -> bool {
        match self {
            FrameSyncDefinition::StartOfFrame => false,
            FrameSyncDefinition::ChannelIdentification => true,
        }
    }
}

/// Clock strobe.
#[derive(Copy, Clone)]
#[allow(missing_docs)]
pub enum ClockStrobe {
    Falling,
    Rising,
}

impl ClockStrobe {
    const fn ckstr(&self) -> vals::Ckstr {
        match self {
            ClockStrobe::Falling => vals::Ckstr::FALLING_EDGE,
            ClockStrobe::Rising => vals::Ckstr::RISING_EDGE,
        }
    }
}

/// Complements format for negative samples.
#[derive(Copy, Clone)]
#[allow(missing_docs)]
pub enum ComplementFormat {
    OnesComplement,
    TwosComplement,
}

impl ComplementFormat {
    const fn cpl(&self) -> vals::Cpl {
        match self {
            ComplementFormat::OnesComplement => vals::Cpl::ONES_COMPLEMENT,
            ComplementFormat::TwosComplement => vals::Cpl::TWOS_COMPLEMENT,
        }
    }
}

/// Companding setting.
#[derive(Copy, Clone)]
#[allow(missing_docs)]
pub enum Companding {
    None,
    MuLaw,
    ALaw,
}

impl Companding {
    const fn comp(&self) -> vals::Comp {
        match self {
            Companding::None => vals::Comp::NO_COMPANDING,
            Companding::MuLaw => vals::Comp::MU_LAW,
            Companding::ALaw => vals::Comp::ALAW,
        }
    }
}

/// Output drive
#[derive(Copy, Clone)]
#[allow(missing_docs)]
pub enum OutputDrive {
    OnStart,
    Immediately,
}

impl OutputDrive {
    const fn outdriv(&self) -> vals::Outdriv {
        match self {
            OutputDrive::OnStart => vals::Outdriv::ON_START,
            OutputDrive::Immediately => vals::Outdriv::IMMEDIATELY,
        }
    }
}

/// [`SAI`] configuration.
#[allow(missing_docs)]
#[non_exhaustive]
#[derive(Copy, Clone)]
pub struct Config {
    pub mode: Mode,
    pub tx_rx: TxRx,
    pub sync_input: SyncInput,
    pub sync_output: bool,
    pub protocol: Protocol,
    pub slot_size: SlotSize,
    pub slot_count: word::U4,
    pub slot_enable: u16,
    pub first_bit_offset: word::U5,
    pub data_size: DataSize,
    pub stereo_mono: StereoMono,
    pub bit_order: BitOrder,
    pub frame_sync_offset: FrameSyncOffset,
    pub frame_sync_polarity: FrameSyncPolarity,
    pub frame_sync_active_level_length: word::U7,
    pub frame_sync_definition: FrameSyncDefinition,
    pub frame_length: u16,
    pub clock_strobe: ClockStrobe,
    pub output_drive: OutputDrive,
    pub master_clock_divider: MasterClockDivider,
    pub nodiv: bool,
    pub is_high_impedance_on_inactive_slot: bool,
    pub fifo_threshold: FifoThreshold,
    pub companding: Companding,
    pub complement_format: ComplementFormat,
    pub mute_value: MuteValue,
    pub mute_detection_counter: word::U5,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: Mode::Master,
            tx_rx: TxRx::Transmitter,
            sync_output: false,
            sync_input: SyncInput::None,
            protocol: Protocol::Free,
            slot_size: SlotSize::DataSize,
            slot_count: word::U4(2),
            first_bit_offset: word::U5(0),
            slot_enable: 0b11,
            data_size: DataSize::Data16,
            stereo_mono: StereoMono::Stereo,
            bit_order: BitOrder::LsbFirst,
            frame_sync_offset: FrameSyncOffset::BeforeFirstBit,
            frame_sync_polarity: FrameSyncPolarity::ActiveLow,
            frame_sync_active_level_length: word::U7(16),
            frame_sync_definition: FrameSyncDefinition::ChannelIdentification,
            frame_length: 32,
            master_clock_divider: MasterClockDivider::DIV1,
            nodiv: false,
            clock_strobe: ClockStrobe::Rising,
            output_drive: OutputDrive::Immediately,
            is_high_impedance_on_inactive_slot: false,
            fifo_threshold: FifoThreshold::ThreeQuarters,
            companding: Companding::None,
            complement_format: ComplementFormat::TwosComplement,
            mute_value: MuteValue::Zero,
            mute_detection_counter: word::U5(4),
        }
    }
}

impl Config {
    /// Create a new config with all default values.
    pub fn new() -> Self {
        return Default::default();
    }
}

enum RingBuffer<'d, W: word::Word> {
    Writable(WritableRingBuffer<'d, W>),
    Readable(ReadableRingBuffer<'d, W>),
}

fn dr<W: word::Word>(w: crate::pac::sai::Sai, sub_block: WhichSubBlock) -> *mut W {
    let ch = w.ch(sub_block as usize);
    ch.dr().as_ptr() as _
}

// return the type for (sd, sck)
fn get_af_types(mode: Mode, tx_rx: TxRx) -> (AfType, AfType) {
    (
        //sd is defined by tx/rx mode
        match tx_rx {
            TxRx::Transmitter => AfType::output(OutputType::PushPull, Speed::VeryHigh),
            TxRx::Receiver => AfType::input(Pull::Down), // Ensure mute level when no input is connected.
        },
        //clocks (mclk, sck and fs) are defined by master/slave
        match mode {
            Mode::Master => AfType::output(OutputType::PushPull, Speed::VeryHigh),
            Mode::Slave => AfType::input(Pull::Down), // Ensure no clocks when no input is connected.
        },
    )
}

fn get_ring_buffer<'d, T: Instance, W: word::Word>(
    dma: Peri<'d, AnyChannel>,
    dma_buf: &'d mut [W],
    request: Request,
    sub_block: WhichSubBlock,
    tx_rx: TxRx,
) -> RingBuffer<'d, W> {
    let opts = TransferOptions {
        half_transfer_ir: true,
        //the new_write() and new_read() always use circular mode
        ..Default::default()
    };
    match tx_rx {
        TxRx::Transmitter => RingBuffer::Writable(unsafe {
            WritableRingBuffer::new(dma, request, dr(T::REGS, sub_block), dma_buf, opts)
        }),
        TxRx::Receiver => RingBuffer::Readable(unsafe {
            ReadableRingBuffer::new(dma, request, dr(T::REGS, sub_block), dma_buf, opts)
        }),
    }
}

fn update_synchronous_config(config: &mut Config) {
    config.mode = Mode::Slave;
    config.sync_output = false;

    #[cfg(any(sai_v1, sai_v1_4pdm, sai_v2))]
    {
        config.sync_input = SyncInput::Internal;
    }

    #[cfg(any(sai_v3, sai_v4))]
    {
        //this must either be Internal or External
        //The asynchronous sub-block on the same SAI needs to enable sync_output
        assert!(config.sync_input != SyncInput::None);
    }
}

/// SAI subblock instance.
pub struct SubBlock<'d, T: Instance, S: SubBlockInstance> {
    peri: Peri<'d, T>,
    _phantom: PhantomData<S>,
}

/// Split the main SAIx peripheral into the two subblocks.
///
/// You can then create a [`Sai`] driver for each each half.
pub fn split_subblocks<'d, T: Instance>(peri: Peri<'d, T>) -> (SubBlock<'d, T, A>, SubBlock<'d, T, B>) {
    rcc::enable_and_reset::<T>();

    (
        SubBlock {
            peri: unsafe { peri.clone_unchecked() },
            _phantom: PhantomData,
        },
        SubBlock {
            peri,
            _phantom: PhantomData,
        },
    )
}

/// SAI sub-block driver.
pub struct Sai<'d, T: Instance, W: word::Word> {
    _peri: Peri<'d, T>,
    sd: Option<Peri<'d, AnyPin>>,
    fs: Option<Peri<'d, AnyPin>>,
    sck: Option<Peri<'d, AnyPin>>,
    mclk: Option<Peri<'d, AnyPin>>,
    ring_buffer: RingBuffer<'d, W>,
    sub_block: WhichSubBlock,
}

impl<'d, T: Instance, W: word::Word> Sai<'d, T, W> {
    /// Create a new SAI driver in asynchronous mode with MCLK.
    ///
    /// You can obtain the [`SubBlock`] with [`split_subblocks`].
    pub fn new_asynchronous_with_mclk<S: SubBlockInstance, D: Channel + TypedChannel + Dma<T, S>>(
        peri: SubBlock<'d, T, S>,
        sck: Peri<'d, impl SckPin<T, S>>,
        sd: Peri<'d, impl SdPin<T, S>>,
        fs: Peri<'d, impl FsPin<T, S>>,
        mclk: Peri<'d, impl MclkPin<T, S>>,
        dma: Peri<'d, D>,
        dma_buf: &'d mut [W],
        _irq: impl interrupt::typelevel::Binding<D::Interrupt, dma::InterruptHandler<D>> + 'd,
        config: Config,
    ) -> Self {
        let (_sd_af_type, ck_af_type) = get_af_types(config.mode, config.tx_rx);
        set_as_af!(mclk, ck_af_type);

        Self::new_asynchronous(peri, sck, sd, fs, dma, dma_buf, _irq, config)
    }

    /// Create a new SAI driver in asynchronous mode without MCLK.
    ///
    /// You can obtain the [`SubBlock`] with [`split_subblocks`].
    pub fn new_asynchronous<S: SubBlockInstance, D: Dma<T, S>>(
        peri: SubBlock<'d, T, S>,
        sck: Peri<'d, impl SckPin<T, S>>,
        sd: Peri<'d, impl SdPin<T, S>>,
        fs: Peri<'d, impl FsPin<T, S>>,
        dma: Peri<'d, D>,
        dma_buf: &'d mut [W],
        irq: impl interrupt::typelevel::Binding<D::Interrupt, dma::InterruptHandler<D>> + 'd,
        config: Config,
    ) -> Self {
        let peri = peri.peri;

        let (sd_af_type, ck_af_type) = get_af_types(config.mode, config.tx_rx);
        set_as_af!(sd, sd_af_type);
        set_as_af!(sck, ck_af_type);
        set_as_af!(fs, ck_af_type);

        let sub_block = S::WHICH;
        let request = dma.request();

        Self::new_inner(
            peri,
            sub_block,
            Some(sck.into()),
            None,
            Some(sd.into()),
            Some(fs.into()),
            get_ring_buffer::<T, W>(dma_into(dma, &irq), dma_buf, request, sub_block, config.tx_rx),
            config,
        )
    }

    /// Create a new SAI driver in synchronous mode.
    ///
    /// You can obtain the [`SubBlock`] with [`split_subblocks`].
    pub fn new_synchronous<S: SubBlockInstance, D: Dma<T, S>>(
        peri: SubBlock<'d, T, S>,
        sd: Peri<'d, impl SdPin<T, S>>,
        dma: Peri<'d, D>,
        dma_buf: &'d mut [W],
        irq: impl interrupt::typelevel::Binding<D::Interrupt, dma::InterruptHandler<D>> + 'd,
        mut config: Config,
    ) -> Self {
        update_synchronous_config(&mut config);

        let peri = peri.peri;

        let (sd_af_type, _ck_af_type) = get_af_types(config.mode, config.tx_rx);
        set_as_af!(sd, sd_af_type);

        let sub_block = S::WHICH;
        let request = dma.request();

        Self::new_inner(
            peri,
            sub_block,
            None,
            None,
            Some(sd.into()),
            None,
            get_ring_buffer::<T, W>(dma_into(dma, &irq), dma_buf, request, sub_block, config.tx_rx),
            config,
        )
    }

    fn new_inner(
        peri: Peri<'d, T>,
        sub_block: WhichSubBlock,
        sck: Option<Peri<'d, AnyPin>>,
        mclk: Option<Peri<'d, AnyPin>>,
        sd: Option<Peri<'d, AnyPin>>,
        fs: Option<Peri<'d, AnyPin>>,
        ring_buffer: RingBuffer<'d, W>,
        config: Config,
    ) -> Self {
        let ch = T::REGS.ch(sub_block as usize);

        ch.cr1().modify(|w| w.set_saien(false));

        ch.cr2().modify(|w| w.set_fflush(true));

        #[cfg(any(sai_v3, sai_v4))]
        {
            if let SyncInput::External(i) = config.sync_input {
                T::REGS.gcr().modify(|w| {
                    w.set_syncin(i as u8);
                });
            }

            if config.sync_output {
                let syncout: u8 = match sub_block {
                    WhichSubBlock::A => 0b01,
                    WhichSubBlock::B => 0b10,
                };
                T::REGS.gcr().modify(|w| {
                    w.set_syncout(syncout);
                });
            }
        }

        ch.cr1().modify(|w| {
            w.set_mode(config.mode.mode(if Self::is_transmitter(&ring_buffer) {
                TxRx::Transmitter
            } else {
                TxRx::Receiver
            }));
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

        ch.cr2().modify(|w| {
            w.set_fth(config.fifo_threshold.fth());
            w.set_comp(config.companding.comp());
            w.set_cpl(config.complement_format.cpl());
            w.set_muteval(config.mute_value.muteval());
            w.set_mutecnt(config.mute_detection_counter.0 as u8);
            w.set_tris(config.is_high_impedance_on_inactive_slot);
        });

        ch.frcr().modify(|w| {
            w.set_fsoff(config.frame_sync_offset.fsoff());
            w.set_fspol(config.frame_sync_polarity.fspol());
            w.set_fsdef(config.frame_sync_definition.fsdef());
            w.set_fsall(config.frame_sync_active_level_length.0 as u8 - 1);
            w.set_frl((config.frame_length - 1).try_into().unwrap());
        });

        ch.slotr().modify(|w| {
            w.set_nbslot(config.slot_count.0 as u8 - 1);
            w.set_slotsz(config.slot_size.slotsz());
            w.set_fboff(config.first_bit_offset.0 as u8);
            w.set_sloten(vals::Sloten::from_bits(config.slot_enable as u16));
        });

        ch.cr1().modify(|w| w.set_saien(true));

        if ch.cr1().read().saien() == false {
            panic!("SAI failed to enable. Check that config is valid (frame length, slot count, etc)");
        }

        Self {
            _peri: peri,
            sub_block,
            sck,
            mclk,
            sd,
            fs,
            ring_buffer,
        }
    }

    /// Start the SAI driver.
    ///
    /// Only receivers can be started. Transmitters are started on the first writing operation.
    pub fn start(&mut self) -> Result<(), Error> {
        match self.ring_buffer {
            RingBuffer::Writable(_) => Err(Error::NotAReceiver),
            RingBuffer::Readable(ref mut rb) => {
                rb.start();
                Ok(())
            }
        }
    }

    fn is_transmitter(ring_buffer: &RingBuffer<W>) -> bool {
        match ring_buffer {
            RingBuffer::Writable(_) => true,
            _ => false,
        }
    }

    /// Reset SAI operation.
    pub fn reset() {
        rcc::enable_and_reset::<T>();
    }

    /// Enable or disable mute.
    pub fn set_mute(&mut self, value: bool) {
        let ch = T::REGS.ch(self.sub_block as usize);
        ch.cr2().modify(|w| w.set_mute(value));
    }

    /// Determine the mute state of the receiver.
    ///
    /// Clears the mute state flag in the status register.
    pub fn is_muted(&self) -> Result<bool, Error> {
        match &self.ring_buffer {
            RingBuffer::Readable(_) => {
                let ch = T::REGS.ch(self.sub_block as usize);
                let mute_state = ch.sr().read().mutedet();
                ch.clrfr().write(|w| w.set_cmutedet(true));
                Ok(mute_state)
            }
            _ => Err(Error::NotAReceiver),
        }
    }

    /// Wait until any SAI write error occurs.
    ///
    /// One useful application for this is stopping playback as soon as the SAI
    /// experiences an overrun of the ring buffer. Then, instead of letting
    /// the SAI peripheral play the last written buffer over and over again, SAI
    /// can be muted or dropped instead.
    pub async fn wait_write_error(&mut self) -> Result<(), Error> {
        match &mut self.ring_buffer {
            RingBuffer::Writable(buffer) => {
                buffer.wait_write_error().await?;
                Ok(())
            }
            _ => return Err(Error::NotATransmitter),
        }
    }

    /// Write data to the SAI ringbuffer.
    ///
    /// The first write starts the DMA after filling the ring buffer with the provided data.
    /// This ensures that the DMA does not run before data is available in the ring buffer.
    ///
    /// This appends the data to the buffer and returns immediately. The
    /// data will be transmitted in the background.
    ///
    /// If there's no space in the buffer, this waits until there is.
    pub async fn write(&mut self, data: &[W]) -> Result<(), Error> {
        match &mut self.ring_buffer {
            RingBuffer::Writable(buffer) => {
                if buffer.is_running() {
                    buffer.write_exact(data).await?;
                } else {
                    buffer.write_immediate(data)?;
                    buffer.start();
                }
                Ok(())
            }
            _ => return Err(Error::NotATransmitter),
        }
    }

    /// Read data from the SAI ringbuffer.
    ///
    /// SAI is always receiving data in the background. This function pops already-received
    /// data from the buffer.
    ///
    /// If there's less than `data.len()` data in the buffer, this waits until there is.
    pub async fn read(&mut self, data: &mut [W]) -> Result<(), Error> {
        match &mut self.ring_buffer {
            RingBuffer::Readable(buffer) => {
                buffer.read_exact(data).await?;
                Ok(())
            }
            _ => Err(Error::NotAReceiver),
        }
    }
}

impl<'d, T: Instance, W: word::Word> Drop for Sai<'d, T, W> {
    fn drop(&mut self) {
        let ch = T::REGS.ch(self.sub_block as usize);
        ch.cr1().modify(|w| w.set_saien(false));
        ch.cr2().modify(|w| w.set_fflush(true));
        self.fs.as_ref().map(|x| x.set_as_disconnected());
        self.sd.as_ref().map(|x| x.set_as_disconnected());
        self.sck.as_ref().map(|x| x.set_as_disconnected());
        self.mclk.as_ref().map(|x| x.set_as_disconnected());
    }
}

trait SealedInstance {
    const REGS: Regs;
}

#[derive(Copy, Clone)]
enum WhichSubBlock {
    A = 0,
    B = 1,
}

trait SealedSubBlock {
    const WHICH: WhichSubBlock;
}

/// Sub-block instance trait.
#[allow(private_bounds)]
pub trait SubBlockInstance: SealedSubBlock {}

/// Sub-block A.
pub enum A {}
impl SealedSubBlock for A {
    const WHICH: WhichSubBlock = WhichSubBlock::A;
}
impl SubBlockInstance for A {}

/// Sub-block B.
pub enum B {}
impl SealedSubBlock for B {
    const WHICH: WhichSubBlock = WhichSubBlock::B;
}
impl SubBlockInstance for B {}

/// SAI instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + RccPeripheral {}

pin_trait!(SckPin, Instance, SubBlockInstance);
pin_trait!(FsPin, Instance, SubBlockInstance);
pin_trait!(SdPin, Instance, SubBlockInstance);
pin_trait!(MclkPin, Instance, SubBlockInstance);

dma_trait!(Dma, Instance, SubBlockInstance);

foreach_peripheral!(
    (sai, $inst:ident) => {
        impl SealedInstance for peripherals::$inst {
            const REGS: Regs = crate::pac::$inst;
        }

        impl Instance for peripherals::$inst {}
    };
);

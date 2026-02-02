//! Pio backed Nec Ir driver

// Credit: PIO programs from https://github.com/raspberrypi/pico-examples/tree/master/pio/ir_nec
// Licensed BSD-3-Clause.
// Copyright 2020 (c) 2020 Raspberry Pi (Trading) Ltd.

use fixed::traits::ToFixed;

use crate::Peri;
use crate::clocks::clk_sys_freq;
use crate::gpio::Level;
use crate::pio::{
    Common, Config, Direction as PioDirection, FifoJoin, Instance, LoadedProgram, PioPin, ShiftDirection, StateMachine,
};

/// This struct represents a program for receiving NEC IR codes loaded into pio instruction memory.
pub struct PioIrNecRxProgram<'d, PIO: Instance> {
    prg: LoadedProgram<'d, PIO>,
}

impl<'d, PIO: Instance> PioIrNecRxProgram<'d, PIO> {
    /// Load the program into the given pio
    pub fn new(common: &mut Common<'d, PIO>) -> Self {
        let prg = pio::pio_asm!(
            r#"
            ; Decode IR frames in NEC format and push 32-bit words to the input FIFO.
            ;
            ; The input pin should be connected to an IR detector with an 'active low' output.
            ;
            ; This program expects there to be 10 state machine clock ticks per 'normal' 562.5us burst period
            ; in order to permit timely detection of start of a burst. The initialisation function below sets
            ; the correct divisor to achieve this relative to the system clock.
            ;
            ; Within the 'NEC' protocol frames consists of 32 bits sent least-siginificant bit first; so the
            ; Input Shift Register should be configured to shift right and autopush after 32 bits, as in the
            ; initialisation function below.
            ;
            ; Copyright 2020 (c) 2020 Raspberry Pi (Trading) Ltd.

            .define BURST_LOOP_COUNTER 30     ; the detection threshold for a 'frame sync' burst
            .define BIT_SAMPLE_DELAY 15       ; how long to wait after the end of the burst before sampling

            .wrap_target

            next_burst:
                set X, BURST_LOOP_COUNTER
                wait 0 pin 0                  ; wait for the next burst to start

            burst_loop:
                jmp pin data_bit              ; the burst ended before the counter expired
                jmp X-- burst_loop            ; wait for the burst to end

                                              ; the counter expired - this is a sync burst
                mov ISR, NULL                 ; reset the Input Shift Register
                wait 1 pin 0                  ; wait for the sync burst to finish
                jmp next_burst                ; wait for the first data bit

            data_bit:
                nop [ BIT_SAMPLE_DELAY - 1 ]  ; wait for 1.5 burst periods before sampling the bit value
                in PINS, 1                    ; if the next burst has started then detect a '0' (short gap)
                                              ; otherwise detect a '1' (long gap)
                                              ; after 32 bits the ISR will autopush to the receive FIFO
            .wrap
            "#
        );

        let prg = common.load_program(&prg.program);

        Self { prg }
    }
}

/// NEC data is sent in a 32 bit frame made from the normal and inverted address and data. This
/// struct represents the unencoded data.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NecFrame {
    /// Checked NEC Address
    pub address: u8,
    /// Checked NEC Data
    pub data: u8,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// NEC frames send the address and data as normal and inverted bits. These are the errors you can
/// get when trying to decode a raw Frame
pub enum NecFrameDecodeError {
    /// Address and inverted address did not match
    AddressMismatch,
    /// Data and inverted data did not match
    DataMismatch,
    /// Both address and data do not match the inverted value
    AddressAndDataMismatch,
}

impl NecFrame {
    /// Decode a NEC frame from a raw 32 bit number
    pub fn decode(raw: u32) -> Result<Self, NecFrameDecodeError> {
        let bytes = raw.to_le_bytes();

        let address = bytes[0];
        let inverted_address = bytes[1];
        let data = bytes[2];
        let inverted_data = bytes[3];

        match (address == !inverted_address, data == !inverted_data) {
            (true, true) => Ok(NecFrame { address, data }),
            (false, true) => Err(NecFrameDecodeError::AddressMismatch),
            (true, false) => Err(NecFrameDecodeError::DataMismatch),
            (false, false) => Err(NecFrameDecodeError::AddressAndDataMismatch),
        }
    }

    /// Make the raw 32 bit value for a frame.
    pub fn encode(&self) -> u32 {
        u32::from_le_bytes([self.address, !self.address, self.data, !self.data])
    }
}

/// PIO backed NEC IR receiver
/// This program requires an IR receiver such as the VS1838b
/// that low pass filters IR bursts into long pulses.
pub struct PioIrNecRx<'d, PIO: Instance, const SM: usize> {
    sm_rx: StateMachine<'d, PIO, SM>,
}

impl<'d, PIO: Instance, const SM: usize> PioIrNecRx<'d, PIO, SM> {
    /// Construct a PIO backed NEC IR receiver
    pub fn new(
        common: &mut Common<'d, PIO>,
        mut sm_rx: StateMachine<'d, PIO, SM>,
        rx_pin: Peri<'d, impl PioPin>,
        program: &PioIrNecRxProgram<'d, PIO>,
    ) -> Self {
        let mut cfg = Config::default();
        cfg.use_program(&program.prg, &[]);

        let rx_pin = common.make_pio_pin(rx_pin);

        sm_rx.set_pins(Level::High, &[&rx_pin]);
        cfg.set_in_pins(&[&rx_pin]);
        cfg.set_jmp_pin(&rx_pin);
        sm_rx.set_pin_dirs(PioDirection::In, &[&rx_pin]);

        cfg.clock_divider = (clk_sys_freq() as f32 / 17_777.777).to_fixed();

        cfg.shift_in.auto_fill = true;
        cfg.shift_in.direction = ShiftDirection::Right;
        cfg.shift_in.threshold = 32;
        cfg.fifo_join = FifoJoin::RxOnly;

        sm_rx.set_config(&cfg);
        sm_rx.set_enable(true);

        Self { sm_rx }
    }

    /// Wait for a NEC frame
    pub async fn read(&mut self) -> Result<NecFrame, NecFrameDecodeError> {
        NecFrame::decode(self.read_raw().await)
    }

    /// Wait for a raw 32 bit frame
    pub async fn read_raw(&mut self) -> u32 {
        self.sm_rx.rx().wait_pull().await
    }
}

/// This struct represents both programs needed to send NEC IR frames loaded into pio instruction memory.
pub struct PioIrNecTxProgram<'d, PIO: Instance> {
    burst_prg: LoadedProgram<'d, PIO>,
    control_prg: LoadedProgram<'d, PIO>,
}

impl<'d, PIO: Instance> PioIrNecTxProgram<'d, PIO> {
    /// Load both programs into the given PIO's instruction memory.
    /// WARNING: The irq flag used by this program must not be shared with any other PIO program,
    /// including other instances of this program.
    pub fn new(common: &mut Common<'d, PIO>, irq_flag: u8) -> Self {
        /*
        ; Generate bursts of carrier.
        ;
        ; Repeatedly wait for an IRQ to be set then clear it and generate 21 cycles of
        ; carrier with 25% duty cycle
        ;
        ; Copyright 2020 (c) 2020 Raspberry Pi (Trading) Ltd.

        .define NUM_CYCLES 21               ; how many carrier cycles to generate
        .define BURST_IRQ 7                 ; which IRQ should trigger a carrier burst
        .define public TICKS_PER_LOOP 4     ; the number of instructions in the loop (for timing)

        .wrap_target
            set X, (NUM_CYCLES - 1)         ; initialise the loop counter
            wait 1 irq BURST_IRQ            ; wait for the IRQ then clear it
        cycle_loop:
            set pins, 1                     ; set the pin high (1 cycle)
            set pins, 0 [1]                 ; set the pin low (2 cycles)
            jmp X--, cycle_loop             ; (1 more cycle)
        .wrap
        */
        const NUM_CYCLES: u8 = 21;

        let mut burst_a: pio::Assembler<32> = pio::Assembler::new();

        let mut wrap_target = burst_a.label();
        let mut wrap_source = burst_a.label();
        let mut cycle_loop = burst_a.label();
        burst_a.bind(&mut wrap_target);
        burst_a.set(pio::SetDestination::X, NUM_CYCLES);
        burst_a.wait(1, pio::WaitSource::IRQ, irq_flag, false);
        burst_a.bind(&mut cycle_loop);
        burst_a.set(pio::SetDestination::PINS, 1);
        burst_a.set_with_delay(pio::SetDestination::PINS, 0, 1);
        burst_a.jmp(pio::JmpCondition::XDecNonZero, &mut cycle_loop);
        burst_a.bind(&mut wrap_source);

        let burst_prg = burst_a.assemble_with_wrap(wrap_source, wrap_target);
        let burst_prg = common.load_program(&burst_prg);

        /*
        ; Transmit an encoded 32-bit frame in NEC IR format.
        ;
        ; Accepts 32-bit words from the transmit FIFO and sends them least-significant bit first
        ; using pulse position modulation.
        ;
        ; Carrier bursts are generated using the nec_carrier_burst program, which is expected to be
        ; running on a separate state machine.
        ;
        ; This program expects there to be 2 state machine ticks per 'normal' 562.5us
        ; burst period.
        ;
        ; Copyright 2020 (c) 2020 Raspberry Pi (Trading) Ltd.

        .define BURST_IRQ 7                     ; the IRQ used to trigger a carrier burst
        .define NUM_INITIAL_BURSTS 16           ; how many bursts to transmit for a 'sync burst'

        .wrap_target
            pull                                ; fetch a data word from the transmit FIFO into the
                                                ; output shift register, blocking if the FIFO is empty

            set X, (NUM_INITIAL_BURSTS - 1)     ; send a sync burst (9ms)
        long_burst:
            irq BURST_IRQ
            jmp X-- long_burst

            nop [15]                            ; send a 4.5ms space
            irq BURST_IRQ [1]                   ; send a 562.5us burst to begin the first data bit

        data_bit:
            out X, 1                            ; shift the least-significant bit from the OSR
            jmp !X burst                        ; send a short delay for a '0' bit
            nop [3]                             ; send an additional delay for a '1' bit
        burst:
            irq BURST_IRQ                       ; send a 562.5us burst to end the data bit

        jmp !OSRE data_bit                      ; continue sending bits until the OSR is empty

        .wrap                                   ; fetch another data word from the FIFO
        */

        const NUM_INITIAL_BURSTS: u8 = 16;

        let mut control_a: pio::Assembler<32> = pio::Assembler::new();

        let mut wrap_target = control_a.label();
        let mut wrap_source = control_a.label();
        let mut long_burst = control_a.label();
        let mut data_bit = control_a.label();
        let mut burst = control_a.label();
        control_a.bind(&mut wrap_target);
        control_a.pull(false, true);
        control_a.set(pio::SetDestination::X, NUM_INITIAL_BURSTS - 1);
        control_a.bind(&mut long_burst);
        control_a.irq(false, false, irq_flag, pio::IrqIndexMode::DIRECT);
        control_a.jmp(pio::JmpCondition::XDecNonZero, &mut long_burst);
        control_a.nop_with_delay(15);
        control_a.irq_with_delay(false, false, irq_flag, pio::IrqIndexMode::DIRECT, 1);
        control_a.bind(&mut data_bit);
        control_a.out(pio::OutDestination::X, 1);
        control_a.jmp(pio::JmpCondition::XIsZero, &mut burst);
        control_a.nop_with_delay(3);
        control_a.bind(&mut burst);
        control_a.irq(false, false, irq_flag, pio::IrqIndexMode::DIRECT);
        control_a.jmp(pio::JmpCondition::OutputShiftRegisterNotEmpty, &mut data_bit);

        control_a.bind(&mut wrap_source);

        let control_prg = control_a.assemble_with_wrap(wrap_source, wrap_target);
        let control_prg = common.load_program(&control_prg);

        Self { burst_prg, control_prg }
    }
}

/// PIO backed NEC IR receiver
/// This program only requires an high-side switched IR led.
pub struct PioIrNecTx<'d, PIO: Instance, const SM1: usize, const SM2: usize> {
    _sm_burst: StateMachine<'d, PIO, SM1>,
    sm_control: StateMachine<'d, PIO, SM2>,
}

impl<'d, PIO: Instance, const SM1: usize, const SM2: usize> PioIrNecTx<'d, PIO, SM1, SM2> {
    /// Configure a pio state machine to use the loaded rx program.
    pub fn new(
        common: &mut Common<'d, PIO>,
        mut sm_burst: StateMachine<'d, PIO, SM1>,
        mut sm_control: StateMachine<'d, PIO, SM2>,
        tx_pin: Peri<'d, impl PioPin>,
        program: &PioIrNecTxProgram<'d, PIO>,
    ) -> Self {
        let mut burst_cfg = Config::default();

        let tx_pin = common.make_pio_pin(tx_pin);

        sm_burst.set_pins(Level::High, &[&tx_pin]);
        sm_burst.set_pin_dirs(PioDirection::Out, &[&tx_pin]);

        burst_cfg.set_out_pins(&[&tx_pin]);
        burst_cfg.set_set_pins(&[&tx_pin]);
        burst_cfg.use_program(&program.burst_prg, &[]);
        burst_cfg.clock_divider = (clk_sys_freq() / (4 * 38_222)).to_fixed();
        sm_burst.set_config(&burst_cfg);
        sm_burst.set_enable(true);

        let mut control_cfg = Config::default();

        control_cfg.use_program(&program.control_prg, &[]);
        control_cfg.shift_out.auto_fill = false;
        control_cfg.shift_out.direction = ShiftDirection::Right;
        control_cfg.shift_out.threshold = 32;
        control_cfg.fifo_join = FifoJoin::TxOnly;
        control_cfg.clock_divider = (clk_sys_freq() as f32 / 3_555.555).to_fixed();
        sm_control.set_config(&control_cfg);
        sm_control.set_enable(true);

        Self {
            _sm_burst: sm_burst,
            sm_control,
        }
    }

    /// Encode and send a NEC frame
    pub async fn write(&mut self, frame: NecFrame) {
        let word = frame.encode();
        self.sm_control.tx().wait_push(word).await
    }

    /// Send a raw u32 value
    pub async fn write_raw(&mut self, raw: u32) {
        self.sm_control.tx().wait_push(raw).await
    }
}

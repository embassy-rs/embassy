//! This example shows powerful PIO module in the RP2040 chip to communicate with a HD44780 display.
//! See (https://www.sparkfun.com/datasheets/LCD/HD44780.pdf)

#![no_std]
#![no_main]

use core::fmt::Write;

use embassy_executor::Spawner;
use embassy_rp::dma::{AnyChannel, Channel};
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{
    Config, Direction, FifoJoin, InterruptHandler, Pio, PioPin, ShiftConfig, ShiftDirection, StateMachine,
};
use embassy_rp::pwm::{self, Pwm};
use embassy_rp::{bind_interrupts, into_ref, Peripheral, PeripheralRef};
use embassy_time::{Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(pub struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // this test assumes a 2x16 HD44780 display attached as follow:
    //   rs  = PIN0
    //   rw  = PIN1
    //   e   = PIN2
    //   db4 = PIN3
    //   db5 = PIN4
    //   db6 = PIN5
    //   db7 = PIN6
    // additionally a pwm signal for a bias voltage charge pump is provided on pin 15,
    // allowing direct connection of the display to the RP2040 without level shifters.
    let p = embassy_rp::init(Default::default());

    let _pwm = Pwm::new_output_b(p.PWM_CH7, p.PIN_15, {
        let mut c = pwm::Config::default();
        c.divider = 125.into();
        c.top = 100;
        c.compare_b = 50;
        c
    });

    let mut hd = HD44780::new(
        p.PIO0, Irqs, p.DMA_CH3, p.PIN_0, p.PIN_1, p.PIN_2, p.PIN_3, p.PIN_4, p.PIN_5, p.PIN_6,
    )
    .await;

    loop {
        struct Buf<const N: usize>([u8; N], usize);
        impl<const N: usize> Write for Buf<N> {
            fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
                for b in s.as_bytes() {
                    if self.1 >= N {
                        return Err(core::fmt::Error);
                    }
                    self.0[self.1] = *b;
                    self.1 += 1;
                }
                Ok(())
            }
        }
        let mut buf = Buf([0; 16], 0);
        write!(buf, "up {}s", Instant::now().as_micros() as f32 / 1e6).unwrap();
        hd.add_line(&buf.0[0..buf.1]).await;
        Timer::after_secs(1).await;
    }
}

pub struct HD44780<'l> {
    dma: PeripheralRef<'l, AnyChannel>,
    sm: StateMachine<'l, PIO0, 0>,

    buf: [u8; 40],
}

impl<'l> HD44780<'l> {
    pub async fn new(
        pio: impl Peripheral<P = PIO0> + 'l,
        irq: Irqs,
        dma: impl Peripheral<P = impl Channel> + 'l,
        rs: impl PioPin,
        rw: impl PioPin,
        e: impl PioPin,
        db4: impl PioPin,
        db5: impl PioPin,
        db6: impl PioPin,
        db7: impl PioPin,
    ) -> HD44780<'l> {
        into_ref!(dma);

        let Pio {
            mut common,
            mut irq0,
            mut sm0,
            ..
        } = Pio::new(pio, irq);

        // takes command words (<wait:24> <command:4> <0:4>)
        let prg = pio_proc::pio_asm!(
            r#"
                .side_set 1 opt
                .origin 20

                loop:
                    out x,     24
                delay:
                    jmp x--,   delay
                    out pins,  4     side 1
                    out null,  4     side 0
                    jmp !osre, loop
                irq 0
            "#,
        );

        let rs = common.make_pio_pin(rs);
        let rw = common.make_pio_pin(rw);
        let e = common.make_pio_pin(e);
        let db4 = common.make_pio_pin(db4);
        let db5 = common.make_pio_pin(db5);
        let db6 = common.make_pio_pin(db6);
        let db7 = common.make_pio_pin(db7);

        sm0.set_pin_dirs(Direction::Out, &[&rs, &rw, &e, &db4, &db5, &db6, &db7]);

        let mut cfg = Config::default();
        cfg.use_program(&common.load_program(&prg.program), &[&e]);
        cfg.clock_divider = 125u8.into();
        cfg.set_out_pins(&[&db4, &db5, &db6, &db7]);
        cfg.shift_out = ShiftConfig {
            auto_fill: true,
            direction: ShiftDirection::Left,
            threshold: 32,
        };
        cfg.fifo_join = FifoJoin::TxOnly;
        sm0.set_config(&cfg);

        sm0.set_enable(true);
        // init to 8 bit thrice
        sm0.tx().push((50000 << 8) | 0x30);
        sm0.tx().push((5000 << 8) | 0x30);
        sm0.tx().push((200 << 8) | 0x30);
        // init 4 bit
        sm0.tx().push((200 << 8) | 0x20);
        // set font and lines
        sm0.tx().push((50 << 8) | 0x20);
        sm0.tx().push(0b1100_0000);

        irq0.wait().await;
        sm0.set_enable(false);

        // takes command sequences (<rs:1> <count:7>, data...)
        // many side sets are only there to free up a delay bit!
        let prg = pio_proc::pio_asm!(
            r#"
                .origin 27
                .side_set 1

                .wrap_target
                pull     side 0
                out  x 1 side 0 ; !rs
                out  y 7 side 0 ; #data - 1

                ; rs/rw to e: >= 60ns
                ; e high time: >= 500ns
                ; e low time: >= 500ns
                ; read data valid after e falling: ~5ns
                ; write data hold after e falling: ~10ns

                loop:
                    pull                 side 0
                    jmp  !x       data   side 0
                command:
                    set  pins     0b00   side 0
                    jmp  shift           side 0
                data:
                    set  pins     0b01   side 0
                shift:
                    out  pins     4      side 1 [9]
                    nop                  side 0 [9]
                    out  pins     4      side 1 [9]
                    mov  osr      null   side 0 [7]
                    out  pindirs  4      side 0
                    set  pins     0b10   side 0
                busy:
                    nop                  side 1 [9]
                    jmp  pin      more   side 0 [9]
                    mov  osr      ~osr   side 1 [9]
                    nop                  side 0 [4]
                    out  pindirs  4      side 0
                    jmp  y--      loop   side 0
                .wrap
                more:
                    nop                  side 1 [9]
                    jmp busy             side 0 [9]
            "#
        );

        let mut cfg = Config::default();
        cfg.use_program(&common.load_program(&prg.program), &[&e]);
        cfg.clock_divider = 8u8.into(); // ~64ns/insn
        cfg.set_jmp_pin(&db7);
        cfg.set_set_pins(&[&rs, &rw]);
        cfg.set_out_pins(&[&db4, &db5, &db6, &db7]);
        cfg.shift_out.direction = ShiftDirection::Left;
        cfg.fifo_join = FifoJoin::TxOnly;
        sm0.set_config(&cfg);

        sm0.set_enable(true);

        // display on and cursor on and blinking, reset display
        sm0.tx().dma_push(dma.reborrow(), &[0x81u8, 0x0f, 1]).await;

        Self {
            dma: dma.map_into(),
            sm: sm0,
            buf: [0x20; 40],
        }
    }

    pub async fn add_line(&mut self, s: &[u8]) {
        // move cursor to 0:0, prepare 16 characters
        self.buf[..3].copy_from_slice(&[0x80, 0x80, 15]);
        // move line 2 up
        self.buf.copy_within(22..38, 3);
        // move cursor to 1:0, prepare 16 characters
        self.buf[19..22].copy_from_slice(&[0x80, 0xc0, 15]);
        // file line 2 with spaces
        self.buf[22..38].fill(0x20);
        // copy input line
        let len = s.len().min(16);
        self.buf[22..22 + len].copy_from_slice(&s[0..len]);
        // set cursor to 1:15
        self.buf[38..].copy_from_slice(&[0x80, 0xcf]);

        self.sm.tx().dma_push(self.dma.reborrow(), &self.buf).await;
    }
}

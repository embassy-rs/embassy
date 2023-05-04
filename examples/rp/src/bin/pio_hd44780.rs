#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::fmt::Write;

use embassy_executor::Spawner;
use embassy_rp::dma::{AnyChannel, Channel};
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{FifoJoin, Pio, PioPin, PioStateMachine, ShiftDirection};
use embassy_rp::pwm::{Config, Pwm};
use embassy_rp::relocate::RelocatedProgram;
use embassy_rp::{into_ref, Peripheral, PeripheralRef};
use embassy_time::{Duration, Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

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
        let mut c = Config::default();
        c.divider = 125.into();
        c.top = 100;
        c.compare_b = 50;
        c
    });

    let mut hd = HD44780::new(
        p.PIO0, p.DMA_CH3, p.PIN_0, p.PIN_1, p.PIN_2, p.PIN_3, p.PIN_4, p.PIN_5, p.PIN_6,
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
        Timer::after(Duration::from_secs(1)).await;
    }
}

pub struct HD44780<'l> {
    dma: PeripheralRef<'l, AnyChannel>,
    sm: PioStateMachine<'l, PIO0, 0>,

    buf: [u8; 40],
}

impl<'l> HD44780<'l> {
    pub async fn new(
        pio: impl Peripheral<P = PIO0> + 'l,
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

        let db7pin = db7.pin();
        let Pio {
            mut common,
            mut irq0,
            mut sm0,
            ..
        } = Pio::new(pio);

        // takes command words (<wait:24> <command:4> <0:4>)
        let prg = pio_proc::pio_asm!(
            r#"
                .side_set 1 opt

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

        sm0.set_set_pins(&[&rs, &rw]);
        embassy_rp::pio_instr_util::set_pindir(&mut sm0, 0b11);
        sm0.set_set_pins(&[&e]);
        embassy_rp::pio_instr_util::set_pindir(&mut sm0, 0b1);
        sm0.set_set_pins(&[&db4, &db5, &db6, &db7]);
        embassy_rp::pio_instr_util::set_pindir(&mut sm0, 0b11111);

        let relocated = RelocatedProgram::new(&prg.program);
        common.write_instr(relocated.origin() as usize, relocated.code());
        embassy_rp::pio_instr_util::exec_jmp(&mut sm0, relocated.origin());
        sm0.set_clkdiv(125 * 256);
        let pio::Wrap { source, target } = relocated.wrap();
        sm0.set_wrap(source, target);
        sm0.set_side_enable(true);
        sm0.set_out_pins(&[&db4, &db5, &db6, &db7]);
        sm0.set_sideset_base_pin(&e);
        sm0.set_sideset_count(2);
        sm0.set_out_shift_dir(ShiftDirection::Left);
        sm0.set_fifo_join(FifoJoin::TxOnly);
        sm0.set_autopull(true);
        sm0.set_pull_threshold(32);

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
                .origin 7
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

        let relocated = RelocatedProgram::new(&prg.program);
        common.write_instr(relocated.origin() as usize, relocated.code());
        embassy_rp::pio_instr_util::exec_jmp(&mut sm0, relocated.origin());
        let pio::Wrap { source, target } = relocated.wrap();
        sm0.set_clkdiv(8 * 256); // ~64ns/insn
        sm0.set_side_enable(false);
        sm0.set_jmp_pin(db7pin);
        sm0.set_wrap(source, target);
        sm0.set_set_pins(&[&rs, &rw]);
        sm0.set_out_pins(&[&db4, &db5, &db6, &db7]);
        sm0.set_sideset_base_pin(&e);
        sm0.set_sideset_count(1);
        sm0.set_out_shift_dir(ShiftDirection::Left);
        sm0.set_fifo_join(FifoJoin::TxOnly);

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

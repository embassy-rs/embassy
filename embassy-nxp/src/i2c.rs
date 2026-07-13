#![no_std]
#![no_main]

use core::future::poll_fn;
use core::panic::PanicInfo;
use core::task::Poll;

use embassy_sync::waitqueue::AtomicWaker;
use nxp_pac::*;

// waker for the i2c task
// static so the isr can reach it and call wake on it
static I2C0_WAKER: AtomicWaker = AtomicWaker::new();

// good states from mststate bits 3:1 in stat reg (0x804) um11126 table 623
// idle 0x0 master free can start new transaction
// receiveready 0x1 got a byte waiting in mstdat
// transmitready 0x2 can send next byte
pub enum Good_States {
    Idle,
    ReceiveReady,
    TransmitReady,
}

// error codes also from mststate
// nackaddress 0x3 slave didnt ack the address
// nackdata 0x4 slave nacked a data byte
// undefined anything else not in the table
pub enum Errors {
    NackAddress,
    NackData,
    Undefined,
}

// reads stat once and figures out the state
// only call this after wait_ready_async so mstpending is already 1
// otherwise mststate doesnt mean anything yet master is still busy
pub async fn i2c0_status_bytes() -> Result<Good_States, Errors> {
    let state = I2C0.stat().read().mststate();
    match state {
        0x0 => Ok(Good_States::Idle),
        0x1 => Ok(Good_States::ReceiveReady),
        0x2 => Ok(Good_States::TransmitReady),
        0x3 => Err(Errors::NackAddress),
        0x4 => Err(Errors::NackData),
        _ => Err(Errors::Undefined),
    }
}

// isr for flexcomm0 set up as i2c in i2c0_init
// jumps here when mstpending goes to 1 and interrupt was enabled
#[no_mangle]
pub unsafe extern "C" fn FLEXCOMM0() {
    // turn off the interrupt right away
    // dont want to keep coming back here until we ask for it again
    I2C0.intenclr().write(|w| {
        w.mstpendingen().set_bit();
    });

    // wake up a task that was waiting
    I2C0_WAKER.wake();
}

// waits till master is ready mstpending goes 1 in stat table 623
pub async fn wait_ready_async() {
    poll_fn(|cx| {
        // register waker first before checking anything
        // if we checked first and registered after theres a small gap
        // hw could finish right in that window isr fires wake but nobody registered yet
        // so the wake just gets dropped and we sit there stuck forever
        I2C0_WAKER.register(cx.waker());

        // read the reg once then pull out mstpending bit
        let status = I2C0.stat().read().mstpending();

        match status {
            // 1 means ready to go
            true => Poll::Ready(()),
            // 0 means still busy so we wait
            false => {
                // turn on the interrupt so we get told when its done
                I2C0.intenset().write(|w| w.mstpendingen().set_bit());
                Poll::Pending
            }
        }
    })
    .await;
}

// sends start condition then the address byte
// address shifted left one spot rw bit goes in bit 0
// is_read true for read false for write
pub async unsafe fn i2c0_start(addr: u8, is_read: bool) -> Result<(), Errors> {
    // wait for master to be free before starting anything new
    wait_ready_async().await;

    let base_address = (addr << 1) as u16;

    let tx_data = match is_read {
        true => base_address | (1 << 0),
        false => base_address | (0 << 0),
    };

    // load the address byte into mstdat
    I2C0.mstdat().write(|w| {
        w.set_data(tx_data);
    });

    // mststart 1 tells hw to actually put start + address on the bus
    // see mstctl master control register section 33.6.9
    I2C0.mstctl().write(|w| {
        w.set_mststart(true);
        w.set_mstcontinue(false);
        w.set_mststop(false);
        w
    });

    // wait again for hw to finish sending address and get ack or nack back
    wait_ready_async().await;

    // now mstpending is 1 again -> so mststate is trustworthy
    // either slave acked (good states) or nacked the address
    let result = i2c0_status_bytes().await;
    match result {
        Ok(_) => Ok(()),
        Err(e) => return Err(e),
    }
}

// sends one data byte
// assumes start and address already went out fine through i2c0_start
pub async unsafe fn i2c0_write(data: u8) {
    // wait til master wants the next byte
    wait_ready_async().await;

    I2C0.mstdat().write(|w| {
        w.set_data(data as u16);
    });

    // mstcontinue 1 no start no stop just keep going with this byte
    I2C0.mstctl().write(|w| {
        w.set_mststart(false);
        w.set_mstcontinue(true);
        w.set_mststop(false);
        w
    });

    // wait til the byte actually goes out
    wait_ready_async().await;
}

// sends stop condition ends the transaction
pub async unsafe fn i2c0_stop() {
    // wait for master to be ready before sending stop
    wait_ready_async().await;

    // mststop 1 tells hw to put the stop condition on the bus
    I2C0.mstctl().write(|w| {
        w.set_mststart(false);
        w.set_mstcontinue(false);
        w.set_mststop(true);
        w
    });
}

// sets up pins clocks and master mode for flexcomm0 as i2c0
pub unsafe fn i2c0_init() {
    // need iocon clock on before touching pio0_13 pio0_14 below
    SYSCON.ahbclkctrl0().modify(|_, w| w.iocon().set_bit());

    // pin 13 as sda func 1 picks the i2c alt function
    // opendrain needed for i2c so devices can pull the line low without fighting
    // pullup turns on internal pull up resistor
    IOCON.pio0_13().modify(|_, w| {
        w.func().bits(1);
        w.opendrain().enabled();
        w.mode().pullup();
        w
    });

    // pin 14 as scl same settings as sda
    IOCON.pio0_14().modify(|_, w| {
        w.func().bits(1);
        w.opendrain().enabled();
        w.mode().pullup();
        w
    });

    // clock on for flexcomm0 fc0 this is the peripheral we set as i2c
    SYSCON.ahbclkctrlset1().modify(|_, w| {
        w.fc0().set_bit();
    });

    // persel 3 (persel 0x3 -> i2c FROM Table 614 and ALSO pselid FROM Table 613) picks i2c function for flexcomm0
    FLEXCOMM0.pselid().modify(|_, w| {
        w.persel().bits(3);
    });

    // clkdiv divval sets how fclk gets divided down for i2c timing
    I2C0.clkdiv().write(|w| {
        w.divval().bits(2);
    });

    // msten 1 turns on master mode see cfg register table 622
    I2C0.cfg().write(|w| {
        w.msten().set_bit();
    });
}

// writes a bunch of bytes to a slave start address write each byte then stop
pub async unsafe fn i2c0_write_bytes_async(addr: u8, bytes: &[u8]) {
    i2c0_start(addr, false).await;
    for &b in bytes {
        i2c0_write(b).await;
    }
    i2c0_stop().await;
}

// reads a bunch of bytes from a slave start address read then one byte at a time
pub async unsafe fn i2c0_read_bytes_async(addr: u8, bytes: &mut [u8]) {
    i2c0_start(addr, true).await;

    let length = bytes.len();
    for (i, b) in bytes.iter_mut().enumerate() {
        // wait for a byte to show up receiveready
        wait_ready_async().await;

        // grab the byte from mstdat
        *b = I2C0.mstdat().read().data().bits() as u8;

        if i == length - 1 {
            // last one send stop after reading it to close things out
            I2C0.mstctl().write(|w| {
                w.set_mststart(false);
                w.set_mstcontinue(false);
                w.set_mststop(true);
                w
            });
        } else {
            // more coming so send continue instead of stop
            I2C0.mstctl().write(|w| {
                w.set_mststart(false);
                w.set_mstcontinue(true);
                w.set_mststop(false);
                w
            });
        }
    }

    // wait for the stop to actually go through
    wait_ready_async().await;
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
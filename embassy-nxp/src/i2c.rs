#![no_std] 
#![no_main] 

use core::future::poll_fn;
use core::task::Poll;
use core::panic::PanicInfo;
use nxp_pac::*;
use embassy_sync::waitqueue::AtomicWaker;

static I2C0_WAKER : AtomicWaker = AtomicWaker::new();


pub enum Good_States{
    Idle , 
    ReceiveReady,
    TransmitReady,  
}

pub enum Errors {
    NackAddress,
    NackData,
    Undefined,
}

pub async fn i2c0_status_bytes() -> Result<Good_States,Errors>
{
    let state = I2C0.stat().read().mststate();
    match state {
        0x0 => {Ok(Good_States::Idle)}
        0x1 => {Ok(Good_States::ReceiveReady)}
        0x2 => {Ok(Good_States::TransmitReady)}
        0x3 => {Err(Errors::NackAddress)}
        0x4 => {Err(Errors::NackData)}
        _ => {Err(Errors::Undefined)}
    }
}

#[no_mangle]
pub unsafe extern "C" fn FLEXCOMM0() {
    
    I2C0.intenclr().write(|w|{
        w.mstpendingen().set_bit();
    });

    I2C0_WAKER.wake();
}

pub async fn wait_ready_async() 
{

    poll_fn(|cx| {
        I2C0_WAKER.register(cx.waker());
        let status = I2C0.stat().read().mstpending();
        match status {
            true => {
                Poll::Ready(())
            }
            false => {
                I2C0.intenset().write(|w| {
                    w.mstpendingen().set_bit()
                });
                Poll::Pending
            }
        }
    }).await;
}

pub async unsafe fn i2c0_start(addr: u8, is_read: bool) -> Result<(), Errors> {
    wait_ready_async().await;

    let base_address = (addr << 1) as u16; 
    
    let tx_data = match is_read {
        true => base_address | (1 << 0),
        false => base_address | (0 << 0),
    };

    I2C0.mstdat().write(|w| {
         w.set_data(tx_data);
    });

    I2C0.mstctl().write(|w| {
        w.set_mststart(true);
        w.set_mstcontinue(false);
        w.set_mststop(false);
        w
    });

    wait_ready_async().await; 
    // we wanna see if we reached the slave or not 
    let result = i2c0_status_bytes().await;
    match result {
        Ok(_) => {Ok(())}
        Err(e) => {return Err(e)}
    }
}

pub async unsafe fn i2c0_write(data : u8) {
    wait_ready_async().await;
    I2C0.mstdat().write(|w| {
        w.set_data(data as u16);
    });
    
    I2C0.mstctl().write(|w| {
        w.set_mststart(false);
        w.set_mstcontinue(true);
        w.set_mststop(false);
        w
    });

    wait_ready_async().await;
}

pub async unsafe fn i2c0_stop() {
    wait_ready_async().await;
    I2C0.mstctl().write(|w| {
        w.set_mststart(false);
        w.set_mstcontinue(false);
        w.set_mststop(true);
        w
    });
}

pub unsafe fn i2c0_init() {

    // clock for iocon
    SYSCON.ahbclkctrl0().modify(|_ , w| w.iocon().set_bit());
    
    // we set the pin13 
    IOCON.pio0_13().modify(|_,w| {
       w.func().bits(1);
       w.opendrain().enabled();
       w.mode().pullup();
       w
    });

    // we set the pin14
    IOCON.pio0_14().modify( |_,w| {
        w.func().bits(1);
        w.opendrain().enabled();
        w.mode().pullup();
        w
    });


    SYSCON.ahbclkctrlset1().modify(|_, w| {
        w.fc0().set_bit();
    });

    FLEXCOMM0.pselid().modify(|_, w| {
        w.persel().bits(3);
    });

    I2C0.clkdiv().write(|w| {
        w.divval().bits(2);
    });

    I2C0.cfg().write(|w| {
        w.msten().set_bit();
    });
}

pub async unsafe fn i2c0_write_bytes_async(addr:u8 , bytes : &[u8])
{
    i2c0_start(addr , false).await;
    for &b in bytes 
    {
        i2c0_write(b).await;
    }
    i2c0_stop().await;
}

pub async unsafe fn i2c0_read_bytes_async(addr:u8 , bytes : &mut[u8])
{

    i2c0_start(addr,true).await;
    let length = bytes.len();
    for (i , b) in bytes.iter_mut().enumerate() 
    {
        wait_ready_async().await;
        *b = I2C0.mstdat().read().data().bits() as u8;
        if i == length - 1 {    
            I2C0.mstctl().write(|w| {
                w.set_mststart(false);
                w.set_mstcontinue(false);
                w.set_mststop(true);
                w
            });
        } 
        else {
            I2C0.mstctl().write( |w| {
                w.set_mststart(false);
                w.set_mstcontinue(true);
                w.set_mststop(false);
                w
            });
        }
    }   
    wait_ready_async().await;
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
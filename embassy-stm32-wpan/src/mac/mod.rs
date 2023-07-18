pub mod commands;
mod consts;
pub mod control;
mod driver;
pub mod event;
pub mod indications;
mod macros;
mod opcodes;
pub mod responses;
pub mod runner;
pub mod typedefs;

use core::slice;

pub use crate::mac::control::{Control, Error as ControlError};
use crate::mac::driver::Driver;
pub use crate::mac::runner::Runner;
use crate::sub::mac::Mac;

const MTU: usize = 127;

pub async fn new<'a>(mac: Mac) -> (Runner, Control<'a>, Driver<'a>) {
    let runner = Runner::new(mac);
    let control = Control::new(&runner);
    let driver = Driver::new(&runner);

    (runner, control, driver)
}

fn slice8_mut(x: &mut [u32]) -> &mut [u8] {
    let len = x.len() * 4;
    unsafe { slice::from_raw_parts_mut(x.as_mut_ptr() as _, len) }
}

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

pub use crate::mac::control::Control;
use crate::mac::driver::Driver;
pub use crate::mac::runner::Runner;

const MTU: usize = 127;

pub async fn new<'a>(runner: &'a Runner<'a>) -> (Control<'a>, Driver<'a>) {
    (Control::new(runner), Driver::new(runner))
}

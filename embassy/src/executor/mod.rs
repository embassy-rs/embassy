mod executor;
mod timer_executor;

// for time::Timer
pub(crate) use timer_executor::current_timer_queue;

pub use embassy_macros::task;
pub use executor::{Executor, SpawnError, SpawnToken, Task};
pub use timer_executor::TimerExecutor;

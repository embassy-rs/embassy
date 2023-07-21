use core::future::Future;
use core::task;
use core::task::Poll;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::MutexGuard;
use embassy_sync::signal::Signal;
use futures::FutureExt;

use super::commands::MacCommand;
use super::event::MacEvent;
use super::typedefs::MacError;
use crate::mac::runner::Runner;

pub struct Control<'a> {
    runner: &'a Runner<'a>,
}

impl<'a> Control<'a> {
    pub(crate) fn new(runner: &'a Runner<'a>) -> Self {
        Self { runner: runner }
    }

    pub async fn send_command<T>(&self, cmd: &T) -> Result<(), MacError>
    where
        T: MacCommand,
    {
        let _wm = self.runner.write_mutex.lock().await;

        self.runner.mac_subsystem.send_command(cmd).await
    }

    pub async fn send_command_and_get_response<T>(&self, cmd: &T) -> Result<EventToken<'a>, MacError>
    where
        T: MacCommand,
    {
        let rm = self.runner.read_mutex.lock().await;
        let _wm = self.runner.write_mutex.lock().await;
        let token = EventToken::new(self.runner, rm);

        self.runner.mac_subsystem.send_command(cmd).await?;

        Ok(token)
    }
}

pub struct EventToken<'a> {
    runner: &'a Runner<'a>,
    _mutex_guard: MutexGuard<'a, CriticalSectionRawMutex, ()>,
}

impl<'a> EventToken<'a> {
    pub(crate) fn new(runner: &'a Runner<'a>, mutex_guard: MutexGuard<'a, CriticalSectionRawMutex, ()>) -> Self {
        // Enable event receiving
        runner.rx_event_channel.lock(|s| {
            *s.borrow_mut() = Some(Signal::new());
        });

        Self {
            runner: runner,
            _mutex_guard: mutex_guard,
        }
    }
}

impl<'a> Future for EventToken<'a> {
    type Output = MacEvent<'a>;

    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        self.get_mut().runner.rx_event_channel.lock(|s| {
            let signal = s.borrow_mut();
            let signal = match &*signal {
                Some(s) => s,
                _ => unreachable!(),
            };

            let result = match signal.wait().poll_unpin(cx) {
                Poll::Ready(mac_event) => Poll::Ready(mac_event),
                Poll::Pending => Poll::Pending,
            };

            result
        })
    }
}

impl<'a> Drop for EventToken<'a> {
    fn drop(&mut self) {
        // Disable event receiving
        // This will also drop the contained event, if it exists, and will free up receiving the next event
        self.runner.rx_event_channel.lock(|s| {
            *s.borrow_mut() = None;
        });
    }
}

//! Persistent NemaGFX command list management.

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::coherency::{SurfaceSyncInfo, sync_after_gpu, sync_before_gpu};
use crate::error::Error;
#[cfg(not(feature = "stub-gpu2d"))]
use crate::ffi::nema_gfx::nema_cl_submit_no_irq;
use crate::ffi::nema_gfx::{
    nema_cl_bind_circular, nema_cl_create_sized, nema_cl_submit, nema_cl_wait, nema_cmdlist_t, nema_get_error,
};

/// Default circular command-list capacity (matches ST Resize_GPU example).
const COMMAND_LIST_BYTES: i32 = 8 * 1024;

pub(crate) struct CommandList {
    cl: nema_cmdlist_t,
}

impl CommandList {
    pub(crate) fn new() -> Self {
        unsafe {
            let mut cl = nema_cl_create_sized(COMMAND_LIST_BYTES);
            nema_cl_bind_circular(&mut cl);
            Self { cl }
        }
    }

    pub(crate) fn bind(&mut self) {
        unsafe { nema_cl_bind_circular(&mut self.cl) };
    }

    pub(crate) fn submit_and_wait(&mut self, surfaces: &[SurfaceSyncInfo]) -> Result<(), Error> {
        check_bridge_error()?;
        sync_before_gpu(surfaces);
        unsafe {
            nema_cl_submit(&mut self.cl);
            if nema_cl_wait(&mut self.cl) != 0 {
                return Err(Error::CommandListWait);
            }
        }
        finish_submission(surfaces)
    }

    pub(crate) async fn submit_and_wait_async(&mut self, surfaces: &[SurfaceSyncInfo]) -> Result<(), Error> {
        check_bridge_error()?;
        sync_before_gpu(surfaces);

        #[cfg(feature = "stub-gpu2d")]
        {
            unsafe {
                nema_cl_submit(&mut self.cl);
                if nema_cl_wait(&mut self.cl) != 0 {
                    return Err(Error::CommandListWait);
                }
            }
            return finish_submission(surfaces);
        }

        #[cfg(not(feature = "stub-gpu2d"))]
        {
            unsafe { nema_cl_submit_no_irq(&mut self.cl) };
            WaitGpuCompletion.await?;
            finish_submission(surfaces)
        }
    }
}

fn finish_submission(surfaces: &[SurfaceSyncInfo]) -> Result<(), Error> {
    check_nema_error()?;
    check_bridge_error()?;
    sync_after_gpu(surfaces);
    Ok(())
}

fn check_nema_error() -> Result<(), Error> {
    let code = unsafe { nema_get_error() };
    if code != 0 {
        Err(Error::NemaGfx { code })
    } else {
        Ok(())
    }
}

fn check_bridge_error() -> Result<(), Error> {
    #[cfg(not(feature = "stub-gpu2d"))]
    if crate::gpu2d_bridge::take_system_error() {
        return Err(Error::SystemError);
    }
    Ok(())
}

#[cfg(not(feature = "stub-gpu2d"))]
struct WaitGpuCompletion;

#[cfg(not(feature = "stub-gpu2d"))]
impl Future for WaitGpuCompletion {
    type Output = Result<(), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        crate::gpu2d_bridge::poll_wait(cx);
        if crate::gpu2d_bridge::command_list_complete() {
            crate::gpu2d_bridge::complete_command_list();
            return Poll::Ready(Ok(()));
        }
        Poll::Pending
    }
}

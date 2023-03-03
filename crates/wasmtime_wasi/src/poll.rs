use crate::{
    wasi_io::{InputStream, OutputStream},
    wasi_monotonic_clock::{Instant, MonotonicClock},
    wasi_poll::{Host, Pollable},
    WasiCtx,
};

/// A pollable resource table entry.
#[derive(Copy, Clone)]
pub(crate) enum PollableEntry {
    /// Poll for read events.
    Read(InputStream),
    /// Poll for write events.
    Write(OutputStream),
    /// Poll for a monotonic-clock timer.
    MonotonicClock(MonotonicClock, Instant, bool),
}

impl Host for WasiCtx {
    fn drop_pollable(&mut self, _pollable: Pollable) -> anyhow::Result<()> {
        anyhow::bail!("unsupported");
    }

    fn poll_oneoff(&mut self, _futures: Vec<Pollable>) -> anyhow::Result<Vec<u8>> {
        anyhow::bail!("unsupported");
    }
}

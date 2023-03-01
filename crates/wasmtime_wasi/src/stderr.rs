use pollster::FutureExt;
use wasi_common::stream::TableStreamExt;

use crate::{wasi_stderr, WasiCtx};

impl wasi_stderr::WasiStderr for WasiCtx {
    fn print(&mut self, message: String) -> anyhow::Result<()> {
        let s: &mut Box<dyn wasi_common::OutputStream> = self
            .table_mut()
            .get_output_stream_mut(2 /* not sure if guaranteed? */)?;

        s.write(message.as_bytes()).block_on()?;

        Ok(())
    }

    fn is_terminal(&mut self) -> anyhow::Result<bool> {
        Ok(false)
    }

    fn num_columns(&mut self) -> anyhow::Result<Option<u16>> {
        Ok(None)
    }
}

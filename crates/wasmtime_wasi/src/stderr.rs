use wasi_common::stream::TableStreamExt;

use crate::{wasi::stderr, WasiCtx};

impl stderr::Host for WasiCtx {
    fn print(&mut self, message: String) -> anyhow::Result<()> {
        let s: &mut Box<dyn wasi_common::OutputStream> = self
            .table_mut()
            .get_output_stream_mut(2 /* not sure if guaranteed? */)?;

        pollster::block_on(s.write(message.as_bytes()))?;
        Ok(())
    }
}

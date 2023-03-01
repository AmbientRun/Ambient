use crate::{wasi_environment, WasiCtx};

impl wasi_environment::WasiEnvironment for WasiCtx {
    fn get_environment(&mut self) -> anyhow::Result<Vec<(String, String)>> {
        Ok(self.env.clone())
    }
}

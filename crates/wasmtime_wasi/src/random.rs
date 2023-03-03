use cap_rand::{distributions::Standard, Rng};

use crate::{wasi_random, WasiCtx};

impl wasi_random::Host for WasiCtx {
    fn get_random_bytes(&mut self, len: u32) -> anyhow::Result<Vec<u8>> {
        Ok((&mut self.random)
            .sample_iter(Standard)
            .take(len as usize)
            .collect())
    }

    fn get_random_u64(&mut self) -> anyhow::Result<u64> {
        Ok(self.random.sample(Standard))
    }
}

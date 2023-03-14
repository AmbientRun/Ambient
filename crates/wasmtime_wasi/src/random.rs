use cap_rand::{distributions::Standard, Rng};

use crate::{wasi::random, WasiCtx};

impl random::Host for WasiCtx {
    fn get_random_bytes(&mut self, len: u64) -> anyhow::Result<Vec<u8>> {
        Ok((&mut self.random)
            .sample_iter(Standard)
            .take(len as usize)
            .collect())
    }

    fn get_random_u64(&mut self) -> anyhow::Result<u64> {
        Ok(self.random.sample(Standard))
    }

    fn insecure_random(&mut self) -> anyhow::Result<(u64, u64)> {
        Ok((0, 0))
    }
}

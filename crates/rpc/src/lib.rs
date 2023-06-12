use std::{
    collections::HashMap,
    io::{BufRead, Cursor, Write},
    sync::Arc,
};

use futures::{future::BoxFuture, Future, FutureExt};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

#[allow(clippy::type_complexity)]
#[derive(Clone)]
pub struct RpcRegistry<Args> {
    registry: HashMap<
        String,
        Arc<dyn Fn(Args, &[u8]) -> BoxFuture<Result<Vec<u8>, RpcError>> + Send + Sync>,
    >,
}
impl<Args: Send + 'static> RpcRegistry<Args> {
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
        }
    }
    pub fn register<
        Req: Serialize + DeserializeOwned + Send + 'static,
        Resp: Serialize + DeserializeOwned + Send,
        F: Fn(Args, Req) -> L + Send + Sync + Copy + 'static,
        L: Future<Output = Resp> + Send,
    >(
        &mut self,
        func: F,
    ) {
        let name = std::any::type_name::<F>().to_string();
        self.registry.insert(
            name,
            Arc::new(move |args, req| {
                async move {
                    let req = match bincode::deserialize(req) {
                        Ok(req) => req,
                        Err(err) => {
                            return Err(RpcError::BincodeError(err));
                        }
                    };
                    let resp = func(args, req).await;
                    Ok(bincode::serialize(&resp).unwrap())
                }
                .boxed()
            }),
        );
    }
    pub fn serialize_req<
        Req: Serialize + DeserializeOwned,
        Resp: Serialize + DeserializeOwned,
        F: Fn(Args, Req) -> L + 'static,
        L: Future<Output = Resp> + Send,
    >(
        &self,
        _func: F,
        req: Req,
    ) -> Vec<u8> {
        let name = std::any::type_name::<F>().to_string();
        let mut res = Vec::new();
        writeln!(&mut res, "{name}").unwrap();
        let req = bincode::serialize(&req).unwrap();
        res.write_all(&req).unwrap();
        res
    }
    pub async fn run_req(&self, args: Args, req: &[u8]) -> Result<Vec<u8>, RpcError> {
        let mut reader = Cursor::new(req);
        let mut name = String::new();
        reader.read_line(&mut name)?;
        if name.is_empty() {
            return Err(RpcError::IOError(std::io::ErrorKind::InvalidData.into()));
        }
        let name = name[0..(name.len() - 1)].to_string();
        match self.registry.get(&name) {
            Some(func) => {
                let buf = reader.get_ref();
                let pos = (reader.position() as usize).min(buf.len());
                func(args, &buf[pos..]).await
            }
            None => Err(RpcError::NoSuchFunction(name)),
        }
    }
    pub fn deserialize_resp<
        Req: Serialize + DeserializeOwned,
        Resp: Serialize + DeserializeOwned,
        F: Fn(Args, Req) -> L + 'static,
        L: Future<Output = Resp> + Send,
    >(
        &self,
        _func: F,
        resp: &[u8],
    ) -> Result<Resp, bincode::Error> {
        bincode::deserialize(resp)
    }
}
impl<T> std::fmt::Debug for RpcRegistry<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RpcRegistry").finish()
    }
}

#[derive(Debug, Error)]
pub enum RpcError {
    #[error(transparent)]
    BincodeError(#[from] bincode::Error),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("No such function {0}")]
    NoSuchFunction(String),
}

#[cfg(test)]
#[cfg(not(target_os = "unknown"))]
mod tests {
    use crate::RpcRegistry;

    async fn testy(_args: (), req: i32) -> i32 {
        req * 2
    }

    #[tokio::test]
    async fn it_works() {
        let mut reg = RpcRegistry::new();
        reg.register(testy);
        let req = reg.serialize_req(testy, 6);
        let resp = reg.run_req((), &req).await.unwrap();
        let resp = reg.deserialize_resp(testy, &resp).unwrap();
        println!("resp={resp:?}");
    }
}

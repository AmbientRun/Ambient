use ambient_network::client::GameRpcArgs;
use ambient_rpc::RpcRegistry;

pub mod components;
pub mod player;

pub fn create_rpc_registry() -> RpcRegistry<GameRpcArgs> {
    let mut reg = RpcRegistry::new();
    ambient_network::rpc::register_rpcs(&mut reg);
    ambient_debugger::register_rpcs(&mut reg);
    reg
}

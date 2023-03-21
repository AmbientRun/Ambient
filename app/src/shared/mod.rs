use ambient_network::server;
use ambient_rpc::RpcRegistry;

pub mod components;
pub mod player;

pub fn create_server_rpc_registry() -> RpcRegistry<server::RpcArgs> {
    let mut reg = RpcRegistry::new();
    ambient_network::rpc::register_server_rpcs(&mut reg);
    ambient_debugger::register_server_rpcs(&mut reg);
    reg
}

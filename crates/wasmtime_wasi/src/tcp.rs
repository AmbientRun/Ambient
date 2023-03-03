#![allow(unused_variables)]

use crate::{
    wasi_io::{InputStream, OutputStream},
    wasi_network::{Error, IpAddressFamily, Network},
    wasi_poll::Pollable,
    wasi_tcp::{Host, IpSocketAddress, ShutdownType, TcpSocket},
    HostResult, WasiCtx,
};

impl Host for WasiCtx {
    fn listen(&mut self, socket: TcpSocket, backlog: Option<u64>) -> HostResult<(), Error> {
        todo!()
    }

    fn accept(
        &mut self,
        socket: TcpSocket,
    ) -> HostResult<(TcpSocket, InputStream, OutputStream), Error> {
        todo!()
    }

    fn connect(
        &mut self,
        socket: TcpSocket,
        remote_address: IpSocketAddress,
    ) -> HostResult<(InputStream, OutputStream), Error> {
        todo!()
    }

    fn receive_buffer_size(&mut self, socket: TcpSocket) -> HostResult<u64, Error> {
        todo!()
    }

    fn set_receive_buffer_size(&mut self, socket: TcpSocket, value: u64) -> HostResult<(), Error> {
        todo!()
    }

    fn send_buffer_size(&mut self, socket: TcpSocket) -> HostResult<u64, Error> {
        todo!()
    }

    fn set_send_buffer_size(&mut self, socket: TcpSocket, value: u64) -> HostResult<(), Error> {
        todo!()
    }

    fn create_tcp_socket(
        &mut self,
        network: Network,
        address_family: IpAddressFamily,
    ) -> HostResult<TcpSocket, Error> {
        todo!()
    }

    fn bind(&mut self, this: TcpSocket, local_address: IpSocketAddress) -> HostResult<(), Error> {
        todo!()
    }

    fn local_address(&mut self, this: TcpSocket) -> HostResult<IpSocketAddress, Error> {
        todo!()
    }

    fn shutdown(&mut self, this: TcpSocket, shutdown_type: ShutdownType) -> HostResult<(), Error> {
        todo!()
    }

    fn remote_address(&mut self, this: TcpSocket) -> HostResult<IpSocketAddress, Error> {
        todo!()
    }

    fn keep_alive(&mut self, this: TcpSocket) -> HostResult<bool, Error> {
        todo!()
    }

    fn set_keep_alive(&mut self, this: TcpSocket, value: bool) -> HostResult<(), Error> {
        todo!()
    }

    fn no_delay(&mut self, this: TcpSocket) -> HostResult<bool, Error> {
        todo!()
    }

    fn set_no_delay(&mut self, this: TcpSocket, value: bool) -> HostResult<(), Error> {
        todo!()
    }

    fn address_family(&mut self, this: TcpSocket) -> anyhow::Result<IpAddressFamily> {
        todo!()
    }

    fn unicast_hop_limit(&mut self, this: TcpSocket) -> HostResult<u8, Error> {
        todo!()
    }

    fn set_unicast_hop_limit(&mut self, this: TcpSocket, value: u8) -> HostResult<(), Error> {
        todo!()
    }

    fn ipv6_only(&mut self, this: TcpSocket) -> HostResult<bool, Error> {
        todo!()
    }

    fn set_ipv6_only(&mut self, this: TcpSocket, value: bool) -> HostResult<(), Error> {
        todo!()
    }

    fn non_blocking(&mut self, this: TcpSocket) -> HostResult<bool, Error> {
        todo!()
    }

    fn set_non_blocking(&mut self, this: TcpSocket, value: bool) -> HostResult<(), Error> {
        todo!()
    }

    fn subscribe(&mut self, this: TcpSocket) -> anyhow::Result<Pollable> {
        todo!()
    }

    /* TODO: Revisit after https://github.com/WebAssembly/wasi-sockets/issues/17
    fn bytes_readable(&mut self, socket: Connection) -> HostResult<(u64, bool), Error> {
        drop(socket);
        todo!()
    }

    fn bytes_writable(&mut self, socket: Connection) -> HostResult<(u64, bool), Error> {
        drop(socket);
        todo!()
    }
    */

    fn drop_tcp_socket(&mut self, socket: TcpSocket) -> anyhow::Result<()> {
        drop(socket);
        todo!()
    }
}

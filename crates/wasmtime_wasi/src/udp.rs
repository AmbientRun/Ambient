#![allow(unused_variables)]

use crate::{
    wasi::network::{Error, IpAddressFamily, Network},
    wasi::poll::Pollable,
    wasi::udp::{self, Datagram, IpSocketAddress, UdpSocket},
    wasi::udp_create_socket,
    HostResult, WasiCtx,
};
use wasi_common::udp_socket::TableUdpSocketExt;

impl udp::Host for WasiCtx {
    fn connect(
        &mut self,
        udp_socket: UdpSocket,
        network: Network,
        remote_address: IpSocketAddress,
    ) -> HostResult<(), Error> {
        todo!()
    }

    fn send(&mut self, socket: UdpSocket, datagram: Datagram) -> HostResult<(), Error> {
        todo!()
    }

    fn receive(&mut self, socket: UdpSocket) -> HostResult<Datagram, Error> {
        todo!()
    }

    fn receive_buffer_size(&mut self, socket: UdpSocket) -> HostResult<u64, Error> {
        todo!()
    }

    fn set_receive_buffer_size(&mut self, socket: UdpSocket, value: u64) -> HostResult<(), Error> {
        todo!()
    }

    fn send_buffer_size(&mut self, socket: UdpSocket) -> HostResult<u64, Error> {
        todo!()
    }

    fn set_send_buffer_size(&mut self, socket: UdpSocket, value: u64) -> HostResult<(), Error> {
        todo!()
    }

    fn bind(
        &mut self,
        this: UdpSocket,
        network: Network,
        local_address: IpSocketAddress,
    ) -> HostResult<(), Error> {
        todo!()
    }

    fn local_address(&mut self, this: UdpSocket) -> HostResult<IpSocketAddress, Error> {
        todo!()
    }

    fn remote_address(&mut self, this: UdpSocket) -> HostResult<IpSocketAddress, Error> {
        todo!()
    }

    fn address_family(&mut self, this: UdpSocket) -> HostResult<IpAddressFamily, Error> {
        todo!()
    }

    fn unicast_hop_limit(&mut self, this: UdpSocket) -> HostResult<u8, Error> {
        todo!()
    }

    fn set_unicast_hop_limit(&mut self, this: UdpSocket, value: u8) -> HostResult<(), Error> {
        todo!()
    }

    fn ipv6_only(&mut self, this: UdpSocket) -> HostResult<bool, Error> {
        todo!()
    }

    fn set_ipv6_only(&mut self, this: UdpSocket, value: bool) -> HostResult<(), Error> {
        todo!()
    }

    fn non_blocking(&mut self, this: UdpSocket) -> HostResult<bool, Error> {
        todo!()
    }

    fn set_non_blocking(&mut self, this: UdpSocket, value: bool) -> HostResult<(), Error> {
        let this = self.table.get_udp_socket_mut(this)?;
        this.set_nonblocking(value)?;
        Ok(Ok(()))
    }

    fn subscribe(&mut self, this: UdpSocket) -> anyhow::Result<Pollable> {
        todo!()
    }

    /* TODO: Revisit after https://github.com/WebAssembly/wasi-sockets/issues/17
    fn bytes_readable(&mut self, socket: UdpSocket) -> HostResult<(u64, bool), Error> {
        drop(socket);
        todo!()
    }

    fn bytes_writable(&mut self, socket: UdpSocket) -> HostResult<(u64, bool), Error> {
        drop(socket);
        todo!()
    }
    */

    fn drop_udp_socket(&mut self, socket: UdpSocket) -> anyhow::Result<()> {
        drop(socket);
        todo!()
    }
}

impl udp_create_socket::Host for WasiCtx {
    fn create_udp_socket(
        &mut self,
        address_family: IpAddressFamily,
    ) -> HostResult<UdpSocket, Error> {
        todo!()
    }
}

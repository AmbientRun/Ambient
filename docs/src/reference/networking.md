# Networking

Networking is a critical component of Ambient, as it enables communication between the client and the server. This document explains some of the specifics behind the current protocol.

## Protocol

Currently, the Ambient runtime only supports desktop clients and uses QUIC through the `quinn` library as its communication protocol. We are actively working on web deployments and plan to use WebTransport as soon as possible.

The HTTP (TCP) port is `8999`, and the QUIC (UDP) port is `9000`.

## Entities

The Ambient runtime synchronizes all entities with at least one component marked with the `Networked` attribute. Only components marked as `Networked` will be sent to the client. Most core components are `Networked`, but custom components are not by default; this is something developers have to opt into. It is important to note that this may have unintended ramifications in terms of cheating, especially for hostile clients.

The client is fundamentally designed around runtime flexibility of logic, which is non-ideal for avoiding cheaters. Further research and development are required, but it is likely that there is no silver bullet, and the solution will be game-dependent.

If on 0.2 or above, consult the [clientside](https://github.com/AmbientRun/Ambient/blob/main/guest/rust/examples/basics/clientside/ambient.toml) example to see how to define networked components.

## Logic and Prediction

All gameplay logic is currently server-authoritative. We currently do not have any form of latency-hiding, including prediction, rollback, or clientside logic. We previously had rollback but it was removed due to its relative inflexibility (the solution would have to be different for each class of game.)

Our plan is to introduce clientside and shared logic that can be used for user-defined prediction with runtime assistance, but this will take some time.

## Messaging

The Ambient runtime supports messaging from the client to the server and vice versa through structured messages. These messages are defined ahead of time in `ambient.toml` and made accessible to code that consumes that `ambient.toml`. This messaging can be reliable (QUIC unistream) or unreliable (QUIC datagram). Developers can use this to define their networked behavior, including customized prediction.

If on 0.2 or above, consult the [messaging](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/basics/messaging) example to see how to use the messaging functionality.

## Proxy

Since 0.2, Ambient will establish a connection to a NAT traversal proxy by default (this can be turned off with `--no-proxy`). This proxy allows users to connect to an Ambient server, even when the server is behind NAT or similar. Check the [AmbientProxy repository](https://github.com/AmbientRun/AmbientProxy) for more details about the proxy itself.

The Ambient server (i.e. Ambient when started with `run` or `serve`) connects to the proxy using QUIC (using the `quinn` library) and allocates a proxy endpoint. In response, the proxy provides the endpoint's details as well as an URL for asset downloading. The allocated proxy endpoint can be used by players to connect (`ambient join ...`) to the game server, even if it is running behind a NAT.

Communication between the proxy and players uses the same protocol as with a direct connection to the Ambient server; the only difference is the proxy acting as an intermediary.

## Certificates

By default, Ambient bundles a self-signed certificate that is used by the server and trusted by the client.

To use your own certificate:

- specify `--cert` and `--key` for the server:
  ```sh
  ambient serve --cert ./localhost.crt --key ./localhost.key
  ```
- specify `--ca` for the client if the certificate authority that signed the certificate is not present within the client's system roots
  ```sh
  ambient join 127.0.0.1:9000
  ```

If a custom certificate is specified, the bundled certificates will _not_ be used as a fallback.

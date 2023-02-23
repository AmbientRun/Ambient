# Networking

## Synchronization

For entities to be syncronized it must have at least one component marked with the `Networked` attribute. And only these components will be sent to the client. Let us take `transform` components as an example. if we look into crates/core/src/transform.rs we will see

```rust
components!("transform", {
    @[
        MakeDefault, Debuggable, Networked, Store,
        Name["Translation"],
        Description["The translation/position of this entity."]
    ]
    translation: Vec3,
    ...
```

## Protocol

The network protocol used is Quic. If Quic is new to you, and you want a complete walkthrought this is a very nice and complete video: https://www.youtube.com/watch?v=31J8PoLW9iM

If you want a tl;dr;, there is quinn about section that is pasted below:

> A QUIC connection is an association between two endpoints. [...] To communicate application data, each endpoint may open streams up to a limit dictated by its peer.
>
> Streams may be unidirectional or bidirectional, and are cheap to create and disposable. [...] Bidirectional streams behave much like a traditional TCP connection, and are useful for sending messages that have an immediate response, such as an HTTP request. Stream data is delivered reliably, and there is no ordering enforced between data on different streams.
>
> By avoiding head-of-line blocking and providing unified congestion control across all streams of a connection, QUIC is able to provide higher throughput and lower latency than one or multiple TCP connections between the same two hosts, while providing more useful behavior than raw UDP sockets.
>
> Quinn also exposes unreliable datagrams, which are a low-level primitive preferred when automatic fragmentation and retransmission of certain data is not desired.
>
> https://docs.rs/quinn/latest/quinn/index.html#about-quic

To understand the protocol let us run the following simple example:

```
> cd guest/rust/examples/multiplayer
> RUST_LOG=ambient_network=trace ambient serve &> server.txt
```

in another terminal run

```
> cd guest/rust/examples/multiplayer
> RUST_LOG=off,ambient_network=trace ambient serve &> client.txt
```

Wait for a cube to appear and close the window.
Hit Ctrl+C on the server terminal to close it.

Let us peek at the server log:

```
[2023-02-23T17:54:21Z INFO  ambient_network::server] GameServer listening on port 9000
[2023-02-23T17:54:24Z INFO  ambient_network::server] Received connection
[2023-02-23T17:54:24Z INFO  ambient_network::server] Accepted connection
[2023-02-23T17:54:24Z INFO  ambient_network::server] Connecting to client
[2023-02-23T17:54:24Z INFO  ambient_network::protocol] Received handshake from "user_6KrYuPxMd3yvYuiM1MYOg5"
[2023-02-23T17:54:24Z INFO  ambient_network::protocol] Responding with: ClientInfo { user_id: "user_6KrYuPxMd3yvYuiM1MYOg5", .. }
[2023-02-23T17:54:24Z DEBUG ambient_network::server] Client loop starting
[2023-02-23T17:54:24Z INFO  ambient_network::server] Locking world
[2023-02-23T17:54:24Z INFO  ambient_network::server] Broadcasting diffs
[2023-02-23T17:54:24Z INFO  ambient_network::server] Creating init diff
[2023-02-23T17:54:24Z INFO  ambient_network::server] Init diff sent
[2023-02-23T17:54:24Z INFO  ambient_network::server] Player spawned
[2023-02-23T17:54:31Z INFO  ambient_network::server] Closed server side connection for
[2023-02-23T17:54:31Z INFO  ambient_network::server] [user_6KrYuPxMd3yvYuiM1MYOg5] Disconnecting
[2023-02-23T17:54:31Z INFO  ambient_network::server] [user_6KrYuPxMd3yvYuiM1MYOg5] Disconnected
```

and the client log is:


```
[2023-02-23T17:54:24Z INFO  ambient_network::client] Connecting to server at: 127.0.0.1:9000
[2023-02-23T17:54:24Z INFO  ambient_network::client] open_connection; server_addr=127.0.0.1:9000
[2023-02-23T17:54:24Z INFO  ambient_network::client] Connecting to world instance: 127.0.0.1:9000
[2023-02-23T17:54:24Z INFO  ambient_network::client] Got endpoint
[2023-02-23T17:54:24Z INFO  ambient_network::client] Got connection
[2023-02-23T17:54:24Z INFO  ambient_network::protocol] Setup client side protocol
```


### Server Perspective

```
DEBUG ambient_network::server] Client loop starting
```

The message above is means that the client connection is already setup, and the server will start accepting messages. For that the high level code is

```rust
 loop {
    tokio::select! {
        Some(msg) = entities_rx.next() => { ... }
        Some(msg) = stats_rx.next() => { ... }
        Some(msg) = events_rx.next() => { ... }
        Some(Ok(datagram)) = proto.conn.datagrams.next() => { ... }
        Some(Ok((tx, mut rx))) = proto.conn.bi_streams.next() => { ... }
    }
}
```

The first three are `RecvStream` (https://docs.rs/flume/latest/flume/async/struct.RecvStream.html) that the engine send and this loop send to each client.

Entities diff and stats are similar in the sense that each send its messages on its own connection stream (see https://docs.rs/quinn/latest/quinn/struct.SendStream.html).

```rust
Some(msg) = entities_rx.next() => {
    let span = tracing::debug_span!("world diff");
    proto.diff_stream.send_bytes(msg).instrument(span).await?;
}
Some(msg) = stats_rx.next() => {
    let span = tracing::debug_span!("stats");
    proto.stat_stream.send(&msg).instrument(span).await?;
}
```

These two streams were open at `crates/network/src/protocol.rs` inside `ServerProtocol::new()`.

Events are a little bit different. When there is a new event to be sent to a client, this event is sent on its own new stream. Given that streams are the point of blocking inside Quic, in this way, one event does not block another event.

```rust
Some(msg) = events_rx.next() => {
    let span = tracing::debug_span!("server_event");
    let mut stream = proto.connection().open_uni().instrument(span).await?;
    stream.write(&msg).await?;
}
```

The next arm of the `select` is actually the first receiving data.

```rust
 Some(Ok(datagram)) = proto.conn.datagrams.next() => {
    let _span = tracing::debug_span!("datagram").entered();
    tokio::task::block_in_place(|| (self.on_datagram)(&user_id, datagram))
}
```
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

The next arm of the `select` is actually the first receiving data. This allows the client to send data to the server using traditional UDP datagrams.

```rust
 Some(Ok(datagram)) = proto.conn.datagrams.next() => {
    let _span = tracing::debug_span!("datagram").entered();
    tokio::task::block_in_place(|| (self.on_datagram)(&user_id, datagram))
}
```

With the actualy datagram from the client, we call the `on_datagram` callback. Which reads the first four bytes if the diagrams as the datagram handler.id.

```rust
 let on_datagram = |user_id: &String, mut bytes: Bytes| {
    let data = bytes.split_off(4);
    let handler_id = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
    let state = state.clone();
    let handler = {
        let state = state.lock();
        let world = match state.get_player_world(user_id) {
            Some(world) => world,
            None => {
                log::warn!("Player missing for datagram."); // Probably disconnected
                return;
            }
        };
        world.resource(datagram_handlers()).get(&handler_id).cloned()
    };
    match handler {
        Some(handler) => {
            handler(state, assets.clone(), user_id, data);
        }
        None => {
            log::error!("No such datagram handler: {:?}", handler_id);
        }
    }
};
```

The last arm is the one that allows the client to call specific procedures in the server. Each RPC call is a new stream because they are "free" and they are dispatched to the `on_rpc` function as soon as they are received.

After this we need to lock the server state, which shared using an `Arc<Mutex<ServerState>>`. With access to the server state, we get the player world using the player_id, in the code below called `user_id`.

The `world` give us access to its singleton repository throught the `resource` method. In this case we ask for the resource `datagram_handlers` which are the handlers for all the possible RPCs. `datagram_handlers` are actually a `HashMap<u32, Arc<dyn Fn(...)>>`. So we are actually getting a callback that is cloned into the variable `handler`.

Then we just call this handler.

```rust
Some(Ok((tx, mut rx))) = proto.conn.bi_streams.next() => {
    let span = tracing::debug_span!("rpc");
    let stream_id = rx.read_u32().instrument(span).await;
    if let Ok(stream_id) = stream_id {
        tokio::task::block_in_place(|| { (self.on_rpc)(&user_id, stream_id, tx, rx); })
    }
}
```

The implementation of the `on_rpc` is at crates/network/src/server.rs:409. The first thing is to lock the server state, which shared using an `Arc<Mutex<ServerState>>`. With access to the server state, we get the player world using the player_id, in the code below called `user_id`.

The `world` give us access to its singleton repository throught the `resource` method. In this case we ask for the resource `bi_stream_handlers` which are the handlers for all the possible RPCs. `bi_stream_handlers` are actually a `HashMap<u32, Arc<dyn Fn(...)>>`. So we are actually getting a callback that is cloned into the variable `handler`.

Then we just call this handler.

```rust
let on_rpc = |user_id: &String, stream_id, tx, rx| {
    let _span = debug_span!("on_rpc").entered();
    let handler = {
        let state = state.lock();
        let world = match state.get_player_world(user_id) {
            Some(world) => world,
            None => {
                log::error!("Player missing for rpc."); // Probably disconnected
                return;
            }
        };

        world.resource(bi_stream_handlers()).get(&stream_id).cloned()
    };
    if let Some(handler) = handler {
        handler(state.clone(), assets.clone(), user_id, tx, rx);
    } else {
        log::error!("Unrecognized stream type: {}", stream_id);
    }
};
```

It is interesting to note that both datagrams and RPCs, including the actual handler logic, run under `block_in_place`, which means that they are blocking the network task, which will block the network loop.

# Server Sending diffs

When the client connects, the server runs the method `initial_diff` at crates/network/src/server.rs:370. This first initial diff, will iterate over all filtered entities using the method `all_entities`. For each will create a `Spawn` change, which is a variant of the enum `WorldChange`. This variant, of course, asks the client to spawn a entity with the specified `id` and initialize it with the specified data, in the case below this data is coming from `self.read_entity_components(world, id)`.

The interesting part of `read_entity_components` is that it does not collect all entity components, but only those, that pass a specified filter. The default filter, as state above, is for components with the `Networked` metadata.

```rust
pub fn initial_diff(&self, world: &World) -> WorldDiff {
    WorldDiff {
        changes: self
            .all_entities(world)
            .map(|id| WorldChange::Spawn(Some(id), self.read_entity_components(world, id).into()))
            .collect_vec(),
    }
}
```

After all this we have an instance of `WorldDiff` which is serialized using `bincode` and then sent to this `diffs_tx`. This is the sender side of the channel that we saw above as the firm arm of the network loop. This means that this `bincode` serialized version of the diff will reach the client.

```rust
let diff = bincode::serialize(&diff).unwrap();
diffs_tx.send(diff)
```

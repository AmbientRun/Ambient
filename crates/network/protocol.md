# Protocol

Defines the _quinn_/_webtransport_ agnostic protocol which allows the client to connect and communicate with the server in such a way that both peers are resistant to minor changes or _additions_.

This is to allows clients and servers of slightly different versions to still work together.

## Initiation

Accept connection.

The client opens a unidirectional stream of type `ServerControlFrame`.

The server accepts the `ServerControlFrame`, control stream.

Once the unidirectional client control stream has been accepted, the server initiates `3` unidirectional streams of the following types.

- control: `ClientControlFrame`,
- diff: `WorldDiff`
- stats: `ServerStats`

Client accepts the streams in order.

## Connect

Once the client wishes to connect and initiate a physical presence in the server ECS world it sends a `Connect` request on the `ServerControl` stream with the chosen `user_id`

The server receives this requests creates an entity in the `MAIN_INSTANCE_ID` world.

If the server for any reason chooses to reject the connection, the underlying transport is closed. In any other case, the _client_ assumes it is connected as soon as the request has been sent.

It is considered an _error_ if multiple connect requests are sent or received.

## Disconnect

The client can at any time send a `Disconnect` request.

The server will not accept any new requests or streams, and will _despawn_ the
player from the world, and then close the connection.

## Reconnect

If the player initiates a _different_ connection while a player of the given
_user_id_ already exists in the world, the previous player will be despawned and
sent a server initiated `Disconnect` whereafter the old connection is closed.

## Authentication (TODO)

Method to ensure that the `user_id` is authentic and can not be spoofed by
another client.

### Exploits

Currently, nothing is done to ensure the connecting client is who they say they
are. This allow someone else to connect with the _same_ `user_id` and kick the
"real" player from the world, and taking over their position and player data.

## World data

The server periodically sends a _diff_ of the world to the each connected client
on the `diff` stream.

## Reliability

The client and server implementations ensure to their greatest ability that
any error is localized and which is encountered can be recovered, either by ignoring the request,
or assuming a _good-enough_ default value.

Most commonly, these errors arise from unknown `enum` variants or missing fields
due to _small_ version mismatches between the client and server. In such a case,
the frame should be discard, and the stream should _not_ be closed.

Other kinds of errors arise from delegates of opened streams or datagrams. These
are most often a result of faulty logic or incorrect data, or in other cases a
result of race condition between _concurrently_ ongoing unidirectional streams
within or between clients.

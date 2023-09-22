# Messages

Ambient supports message passing. You define your message types in your `ambient.toml`, and you can then subscribe to and dispatch them as you wish.

## Subscribing to messages

Use the `MessageName::subscribe` method to subscribe to messages.

## Dispatching a message

First construct the message and then send it: `MyMessage { some_field: 4. }.send_local_broadcast();`

## Defining new messages

You define your messages in your `ambient.toml`:

```toml
[messages.MyMessage]
fields = { some_field = "F32" }
```

Read more in the [package documentation](./package.md)

## Using message from other packages

Simply define a [dependency](./package.md#dependencies--dependencies) to the other package,
and then use the message as if it was defined in your package.

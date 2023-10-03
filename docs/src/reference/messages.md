# Messages

Ambient supports message passing between client and server, and from package to package. Message types are defined in the `ambient.toml` (see [the reference](./package.md#messages--messages)); these types can then be subscribed to and sent as needed.

## Subscribing to messages

Use the `MessageName::subscribe` method to subscribe to messages. This method is part of the [ModuleMessage](https://docs.rs/ambient_api/latest/ambient_api/message/trait.ModuleMessage.html) and [RuntimeMessage](https://docs.rs/ambient_api/latest/ambient_api/message/trait.RuntimeMessage.html) traits, and has a slightly different syntax depending on whether you are subscribing to a module or runtime message.

## Dispatching a message

Construct the message (a struct) and send it using one of the appropriate methods for your class of message. As an example, to send a package-defined `MyMessage` to all local packages (i.e. packages on "this side"):

```rust
MyMessage { some_field: 4. }.send_local_broadcast();
```

## Defining new messages

New messages can be defined in `ambient.toml`:

```toml
[messages.MyMessage]
fields = { some_field = "F32" }
```

Read more in the [package documentation](./package.md#messages--messages).

## Using messages from other packages

Add a [dependency](./package.md#dependencies--dependencies) to your package manifest,
pointing to the other package. It will then be available to your package to use underneath
the `packages` module; there is no difference in use between a message defined in your
package and one defined in another package.

Of note is that you can get the entity representing a package using the `entity` function
defined for all packages (i.e. `packages::my_dependency::entity()`); messages can then be
sent to that entity, ensuring that only it will handle the message.

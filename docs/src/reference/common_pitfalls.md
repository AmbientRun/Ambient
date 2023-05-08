# Common pitfalls

## Logic

### My clientside WASM module crashes when accessing a component from the server and unwrapping it

Your clientside WASM can run before the server has finished running its WASM, so the component you're trying to access may not have been created yet.

To fix this, consider using `entity::wait_for_component`, which is an async helper that will stall execution until the component is available.

## Rendering

### My object with a random color is black sometimes

The `color` component is a `Vec4`. Using `rand::random` to populate it will
result in the `w`/alpha channel also being between 0 and 1, which means your
object may be black and/or disappear if the alpha is below the default alpha
cut-off.

To fix this, use a random `Vec3` for your color and then extend it to a `Vec4`:

```rust
let color = rand::random::<Vec3>().extend(1.0);
```

## Running

### Fails to start on Linux (Error in Surface::configure: parent device is lost)

If you're running Wayland, you may have to start ambient with: `WAYLAND_DISPLAY=wayland-1 ambient run`.
See [this issue](https://github.com/gfx-rs/wgpu/issues/2519) for details.

### Runtime error: import `...` has the wrong type

This can occur when you have `.wasm` files in your `build` folder that are using an old version of the Ambient API.
Delete the `build` folder and try again - this should force them to be regenerated.

### Failed to download file / error trying to connect: tcp connect error: _etc_ (OS error 10060)

This can happen if your anti-virus or firewall is blocking the connection to the Ambient runtime.
Try deactivating it, then run the Ambient project again with 'ambient run'.
If this fixes it, you'll need to add an exception to your anti-virus/firewall to allow Ambient to connect.
We do not recommend leaving your anti-virus/firewall disabled.

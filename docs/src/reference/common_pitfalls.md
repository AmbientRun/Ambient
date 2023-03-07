# Common pitfalls

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

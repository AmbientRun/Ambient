# Asset cache

The `AssetCache` is very central concept to the internals of the engine. It should be thought of as a way to _cache the result of slow operations_.

For example, let's say we have:

```rust
fn generate_fractral(param1: bool, param2: u32) -> Image {
    // ...
}

let fractal = generate_fractral(true, 5);
```

The output of that function is always the same, so instead of re-running it every time, we can use the asset cache to cache the result:

```rust
#[derive(Debug, Clone)]
struct GenerateFractal { param1: bool, param2: u32 }
impl SyncAssetKey<Arc<Image>> for GenerateFractal {
    fn load(&self, assets: AssetCache) -> Arc<Arc<Image>> {
        // ..
    }
}

let fractal = GenerateFractal { param2: true, param2: 5 }.get(&assets);
```

The cache key is the debug format of `GenerateFractal`.

## Async

This also works with `async`, so you can for instance have:

```rust
#[derive(Debug, Clone)]
struct LoadImageFlipY { url: String }
#[async_trait]
impl AsyncAssetKey<Arc<Image>> for LoadImageFlipY {
    async fn load(&self, assets: AssetCache) -> Arc<Image> {
        let image = ImageFromUrl { url: self.url.clone() }.get(&assets).await.unwrap();
        // ..
    }
}
```

Note that this will internally make sure that each unique key is only loaded once, i.e. `load` above will always
only be called once (as long as the result is still in the cache).

## Keepalive policies

You can also set different keepalive policies: `AssetKeepalive` which can be `None`, `Timeout` or `Forever`.

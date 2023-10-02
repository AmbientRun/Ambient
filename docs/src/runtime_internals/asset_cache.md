# Asset cache

The `AssetCache` is a very central concept to the internals of the engine. It should be thought of as a way to _cache the result of slow operations_ - that is, a memoization cache.

For example, let's say we have:

```rust
fn generate_fractal(param1: bool, param2: u32) -> Image {
    // ...
}

let fractal = generate_fractal(true, 5);
```

This function is entirely pure - that is, its output is only dependent on its inputs, and it will always return the same output for the same inputs. However, it's also very slow to run. Instead of running it every time, the asset cache can be used to cache the result:

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

The cache key is the debug format of `GenerateFractal`. This may change in the future, but it offers a simple way to construct a cache key for now.

## Async

The asset cache also works with `async`. For example:

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

Note that this will internally make sure that each unique key is only loaded once; that is, `load` above will be called once and persisted in the cache, and then the result will be returned for all subsequent calls, until it is evicted from the cache.

## Keep-alive policies

Different keep-alive policies for your cache key can be set by specifying the `keepalive` method in your trait implementation. This returns an `AssetKeepalive` which can be `None`, `Timeout`, or `Forever`.

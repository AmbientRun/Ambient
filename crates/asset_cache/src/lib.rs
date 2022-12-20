mod background;

use std::{
    any::Any, collections::{hash_map::Entry, HashMap}, ops::Deref, pin::Pin, sync::{Arc, Weak}, task::{Context, Poll}, time::Duration
};

use abort_on_drop::ChildTask;
use async_trait::async_trait;
use background::BackgroundKey;
use futures::{
    future::{pending, BoxFuture, Shared, WeakShared}, Future, FutureExt
};
use parking_lot::Mutex;
use pin_project::{pin_project, pinned_drop};
use serde::{Deserialize, Serialize};

trait AssetHolder: as_any::AsAny + Sync + Send {}
impl<T: Clone + Sync + Send + Any + 'static> AssetHolder for T {}

impl as_any::Downcast for dyn AssetHolder {}
impl as_any::Downcast for dyn AssetHolder + Send {}
impl as_any::Downcast for dyn AssetHolder + Sync {}
impl as_any::Downcast for dyn AssetHolder + Send + Sync {}

#[derive(Clone)]
struct LoadPayload {
    asset_key: AssetKey,
    strong: Arc<dyn AssetHolder>,
}

#[derive(Debug, Clone)]
pub enum AssetLoadDropPolicy {
    StopLoading,
    KeepLoading,
}

#[derive(Debug, Clone)]
pub enum AssetKeepalive {
    /// The asset will be unloaded once the last reference to it is released
    None,
    Timeout(Duration),
    Forever,
}

impl AssetKeepalive {
    fn is_active(&self) -> bool {
        !matches!(self, AssetKeepalive::None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetKey(Arc<String>);
impl AssetKey {
    fn new(key: impl Into<String>) -> Self {
        Self(Arc::new(key.into()))
    }
}
impl Deref for AssetKey {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
pub(crate) enum ContentState {
    Loading { fut: WeakShared<BoxFuture<'static, LoadPayload>> },
    Loaded { value: Arc<dyn AssetHolder>, check_alive: Arc<dyn Fn() -> bool + Send + Sync> },
    Aborted,
    Expired,
}

impl std::fmt::Debug for ContentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Loading { fut } => f.debug_struct("Loading").field("fut", fut).finish(),
            Self::Loaded { .. } => f.debug_struct("Loaded").finish(),
            Self::Aborted => write!(f, "Aborted"),
            Self::Expired => write!(f, "Expired"),
        }
    }
}

struct AsyncAssetLoc {
    key: AssetKey,
    /// Since there may be multiple tasks for keepalive, keepalive_end should only be called when
    /// **All** keepalive tasks are done.
    keepalive_guard: Weak<KeepaliveGuard>,
    content: ContentState,
    keepalive_task: Option<ChildTask<()>>,
}

impl AsyncAssetLoc {
    /// Checks if the resource has been dropped since this method was called last time
    fn state(&mut self) -> AsyncAssetState {
        match &mut self.content {
            ContentState::Loading { .. } => AsyncAssetState::Loading,
            ContentState::Loaded { check_alive, .. } => {
                if !check_alive() {
                    self.content = ContentState::Expired;
                    AsyncAssetState::Died
                } else {
                    AsyncAssetState::Alive
                }
            }
            ContentState::Aborted => AsyncAssetState::Aborted,
            ContentState::Expired => AsyncAssetState::Dead,
        }
    }
}

impl ContentState {
    /// Returns the concrete loaded value if loaded and kept alive (strong count).
    fn get_loaded_value<T: Asset + Clone + Sync + Send + 'static>(&self) -> Option<T> {
        if let ContentState::Loaded { value, .. } = &self {
            let content = value.as_any().downcast_ref::<<T as Asset>::WeakType>().unwrap();
            let content = T::from_weak(content)?;
            Some(content)
        } else {
            None
        }
    }

    /// Returns `true` if the content state is [`Loading`].
    ///
    /// [`Loading`]: ContentState::Loading
    #[must_use]
    pub(crate) fn is_loading(&self) -> bool {
        matches!(self, Self::Loading { .. })
    }
}

#[derive(Debug, Clone)]
enum AsyncAssetState {
    Loading,
    Alive,
    Died,
    Dead,
    Aborted,
}

#[derive(Clone)]
struct SyncAssetLoc {
    _key: AssetKey,
    content: Arc<Mutex<Option<Arc<dyn AssetHolder>>>>,
}

#[derive(Clone)]
pub struct AssetCache {
    async_cache: Arc<Mutex<HashMap<AssetKey, AsyncAssetLoc>>>,
    sync: Arc<Mutex<HashMap<AssetKey, SyncAssetLoc>>>,
    pub timeline: Arc<Mutex<AssetsTimeline>>,
    runtime: tokio::runtime::Handle,
    max_keepalive: Option<Duration>,
    /// stack is used for nested asset loading, to visualize for the timeline who loaded what
    stack: Vec<AssetKey>,
}
impl AssetCache {
    pub fn new(runtime: tokio::runtime::Handle) -> Self {
        Self::new_with_config(runtime, None)
    }
    pub fn new_with_config(runtime: tokio::runtime::Handle, max_keepalive: Option<Duration>) -> Self {
        let assets = Self {
            async_cache: Arc::new(Mutex::new(HashMap::new())),
            sync: Arc::new(Mutex::new(HashMap::new())),
            timeline: Arc::new(Mutex::new(AssetsTimeline::new())),
            runtime: runtime.clone(),
            max_keepalive,
            stack: Vec::new(),
        };
        {
            let assets = assets.clone();
            runtime.spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs_f32(1.)).await;
                    assets.clean_up_dropped();
                }
            });
        }
        assets
    }
    #[deprecated(note = "Use a SyncAssetKey instead")]
    pub fn get_sync<K: Into<String>, T: Clone + Sync + Send + 'static>(
        &self,
        key: K,
        loader: impl FnOnce(AssetCache) -> T + Sync + Send,
    ) -> T {
        let loc = {
            let mut cache = self.sync.lock();
            let key = AssetKey::new(key);
            cache.entry(key.clone()).or_insert_with(|| SyncAssetLoc { _key: key, content: Arc::new(Mutex::new(None)) }).clone()
        };
        let mut content = loc.content.lock();
        if content.is_none() {
            *content = Some(Arc::new(loader(self.clone())) as Arc<dyn AssetHolder>);
        }
        content.as_ref().unwrap().as_any().downcast_ref::<T>().unwrap().clone()
    }

    #[deprecated(note = "Use a SyncAssetKey instead")]
    pub fn try_get_sync<K: Into<String>, T: Clone + Sync + Send + 'static>(&self, key: K) -> Option<T> {
        let cache = self.sync.lock();
        let key = AssetKey::new(key);
        if let Some(entry) = cache.get(&key) {
            let content = entry.content.lock();
            Some(content.as_ref().unwrap().as_any().downcast_ref::<T>().unwrap().clone())
        } else {
            None
        }
    }

    #[deprecated(note = "Use a SyncAssetKey instead")]
    pub fn contains_sync<K: Into<String>>(&self, key: K) -> bool {
        let key = AssetKey::new(key);
        let cache = self.sync.lock();
        cache.contains_key(&key)
    }

    /// Use try_get_sync to retreive again
    #[deprecated(note = "Use a SyncAssetKey instead")]
    pub fn insert<K: Into<String>, T: Clone + Sync + Send + 'static>(&self, key: K, asset: T) {
        let key = AssetKey::new(key);
        let mut cache = self.sync.lock();
        cache.insert(key.clone(), SyncAssetLoc { _key: key, content: Arc::new(Mutex::new(Some(Arc::new(asset) as Arc<dyn AssetHolder>))) });
    }

    fn clean_up_dropped(&self) {
        let mut async_ = self.async_cache.lock();
        for (key, asset) in &mut *async_ {
            let state = asset.state();
            match state {
                AsyncAssetState::Died => self.timeline.lock().dropped(key),
                AsyncAssetState::Aborted => self.timeline.lock().aborted(key),
                _ => {}
            }
        }
    }

    /// Returns a snapshot of the current state of the asset
    pub(crate) fn content_state<T: 'static + Clone + Asset + Send + Sync, K: AsyncAssetKeyExt<T>>(&self, key: &K) -> Option<ContentState> {
        let key = AssetKey::new(key.key());
        let cache = self.async_cache.lock();

        cache.get(&key).map(|v| v.content.clone())
    }

    fn fork(&self, key: AssetKey) -> Self {
        let mut cache = self.clone();
        cache.stack.push(key);
        cache
    }

    /// Returns the asset or a future for loading the asset
    fn get_asset_future<K, T>(&self, key: K) -> Result<(AssetKey, T), Shared<Pin<Box<dyn Future<Output = LoadPayload> + Send>>>>
    where
        K: 'static + Clone + AsyncAssetKey<T>,
        T: 'static + Asset + Clone + Sync + Send,
    {
        let mut cache = self.async_cache.lock();

        let keepalive = key.keepalive();

        let timeline = self.timeline.clone();

        let asset_key = AssetKey::new(key.key());

        let load = || {
            tracing::info!("Loading asset: {asset_key:?}");
            // No future loading the value was found.
            //
            // Initiate the loading

            timeline.lock().start_load(asset_key.clone(), key.long_name(), self.stack.clone(), keepalive.is_active());

            let fork = self.fork(asset_key.clone());
            let drop_policy = key.drop_policy();

            let fut = (Box::pin(AssetLoadFuture {
                cache: self.async_cache.clone(),
                key: key.clone(),
                completed: false,
                timeline: timeline.clone(),
                asset_key: asset_key.clone(),
                fut: async move { key.load(fork).await },
            }) as BoxFuture<'static, LoadPayload>)
                .shared();

            let content = ContentState::Loading { fut: Shared::downgrade(&fut).unwrap() };

            // Spawn a task to keep running the shared future even if the key holder drops
            // their part of the future.
            let keepalive = if let AssetLoadDropPolicy::KeepLoading = drop_policy {
                Some(self.runtime.spawn(fut.clone().map(|_| {})).into())
            } else {
                None
            };
            (fut, content, keepalive)
        };

        // Acquire or start a future for loading this asset
        let fut = match cache.entry(asset_key.clone()) {
            Entry::Occupied(mut slot) => {
                let mut loc = slot.get_mut();

                match &mut loc.content {
                    ContentState::Loading { fut } => {
                        if let Some(fut) = fut.upgrade() {
                            fut
                        } else {
                            // Start the loading, and update the content state yet again with the
                            // fresh future
                            let (fut, c, k) = load();
                            loc.content = c;
                            loc.keepalive_task = k;
                            fut
                        }
                    }
                    ContentState::Loaded { value, .. } => {
                        // Loaded and referenced

                        let content = value.as_any().downcast_ref::<<T as Asset>::WeakType>().unwrap();
                        if let Some(content) = T::from_weak(content) {
                            return Ok((asset_key, content));
                        }

                        let (fut, c, k) = load();
                        loc.content = c;
                        loc.keepalive_task = k;
                        fut
                    }
                    ContentState::Aborted | ContentState::Expired => {
                        let (fut, c, k) = load();
                        loc.content = c;
                        loc.keepalive_task = k;
                        fut
                    }
                }
            }
            Entry::Vacant(slot) => {
                // There is not slot for this asset yet.
                //
                // Acquire a future and insert a loading content
                let (fut, content, keepalive_task) = load();
                let key = slot.key().clone();

                slot.insert(AsyncAssetLoc { key, content, keepalive_task, keepalive_guard: Weak::new() });

                fut
            }
        };

        Err(fut)
    }

    async fn get_async<K, T>(&self, key: K) -> T
    where
        K: 'static + Clone + AsyncAssetKey<T>,
        T: 'static + Asset + Clone + Sync + Send,
    {
        let keepalive = key.keepalive();

        let (asset_key, value) = match self.get_asset_future(key) {
            Ok(value) => value,
            Err(fut) => {
                let LoadPayload { asset_key, strong } = fut.await;

                let value = strong.as_any().downcast_ref::<T>().unwrap().clone();
                (asset_key, value)
            }
        };

        let mut cache = self.async_cache.lock();
        let loc = cache.get_mut(&asset_key).expect("Asset loc was removed during loading");

        // Start or replace the keepalive task

        let keepalive_ref = value.clone();
        // Use a drop impl since cancelling a task causes the task to not reach the end, and
        // therefore not registering that the keepalive ended.
        //
        // This is after all what RAII is all about
        //
        // Always get a fresh guard
        let guard = loc.keepalive_guard.upgrade().unwrap_or_else(|| Arc::new(KeepaliveGuard::begin(asset_key, self.timeline.clone())));
        loc.keepalive_guard = Arc::downgrade(&guard);

        match keepalive {
            AssetKeepalive::Timeout(mut dur) => {
                if let Some(max_keepalive) = self.max_keepalive {
                    dur = dur.min(max_keepalive);
                }

                let task = self.runtime.spawn(async move {
                    tokio::time::sleep(dur).await;
                    tracing::info!("Keepalive timed out");
                    drop((keepalive_ref, guard));
                });

                loc.keepalive_task = Some(task.into());
            }
            AssetKeepalive::Forever => {
                let task = self.runtime.spawn(async move {
                    pending::<()>().await;
                    drop((keepalive_ref, guard));
                });

                loc.keepalive_task = Some(task.into());
            }
            _ => (),
        }

        value
    }

    pub fn runtime(&self) -> &tokio::runtime::Handle {
        &self.runtime
    }
}

impl std::fmt::Debug for AssetCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssetCache").finish_non_exhaustive()
    }
}

pub trait SyncAssetKey<T: Clone + Sync + Send + 'static>: Sync + Send + std::fmt::Debug {
    fn load(&self, _assets: AssetCache) -> T {
        panic!("This resource doesn't implement the load method")
    }
}
pub trait SyncAssetKeyExt<T: Clone + Sync + Send + 'static>: SyncAssetKey<T> {
    fn key(&self) -> String;
    fn get(&self, assets: &AssetCache) -> T;
    fn try_get(&self, assets: &AssetCache) -> Option<T>;
    fn insert(&self, assets: &AssetCache, value: T);
    fn exists(&self, assets: &AssetCache) -> bool;
}
#[allow(deprecated)]
impl<T: Clone + Sync + Send + 'static, K: SyncAssetKey<T>> SyncAssetKeyExt<T> for K {
    fn key(&self) -> String {
        format!("{self:?}")
    }
    fn get(&self, assets: &AssetCache) -> T {
        let assets = assets.clone();
        assets.get_sync(self.key(), |assets| self.load(assets))
    }
    fn try_get(&self, assets: &AssetCache) -> Option<T> {
        let assets = assets.clone();
        assets.try_get_sync(self.key())
    }
    fn insert(&self, assets: &AssetCache, value: T) {
        assets.insert(self.key(), value);
    }
    fn exists(&self, assets: &AssetCache) -> bool {
        assets.contains_sync(self.key())
    }
}

#[async_trait]
pub trait AsyncAssetKey<T: Asset + Clone + Sync + Send + 'static>: Sync + Send + std::fmt::Debug {
    async fn load(self, assets: AssetCache) -> T;

    /// Adapter to make the key load in a background task.
    ///
    /// This allows `get` and `load` to work outside the tokio runtime.
    ///
    /// It also prevents the loading from being aborted
    fn in_background(self) -> BackgroundKey<Self>
    where
        Self: Sized,
    {
        BackgroundKey(self)
    }

    fn keepalive(&self) -> AssetKeepalive {
        AssetKeepalive::Timeout(Duration::from_secs_f32(60.))
    }

    fn drop_policy(&self) -> AssetLoadDropPolicy {
        AssetLoadDropPolicy::StopLoading
    }

    fn cpu_size(&self, _asset: &T) -> Option<usize> {
        None
    }
    fn gpu_size(&self, _asset: &T) -> Option<usize> {
        None
    }
}
#[async_trait]
pub trait AsyncAssetKeyExt<T: Asset + Clone + Sync + Send + 'static>: AsyncAssetKey<T> {
    fn key(&self) -> String;
    fn long_name(&self) -> String;
    async fn get(&self, assets: &AssetCache) -> T;
    /// Returns `Some(T)` if the asset is currently loaded, alive, and well.
    ///
    /// Does not attempt to load the asset in any way
    fn is_loaded(&self, assets: &AssetCache) -> Option<T>;
    /// If the asset is loaded, it will be returned. Otherwise, the loading will start loading in the background, and None will be returned
    fn peek(&self, assets: &AssetCache) -> Option<T>;
}

#[async_trait]
impl<T: Asset + Clone + Sync + Send + 'static, K: AsyncAssetKey<T> + Clone + 'static> AsyncAssetKeyExt<T> for K {
    fn key(&self) -> String {
        format!("{self:?}")
    }
    fn long_name(&self) -> String {
        format!("{self:#?}")
    }

    #[tracing::instrument(skip(assets), level = "debug")]
    async fn get(&self, assets: &AssetCache) -> T {
        assets.get_async(self.clone()).await
    }

    fn is_loaded(&self, assets: &AssetCache) -> Option<T> {
        if let Some(content) = assets.content_state(self) {
            if let Some(value) = content.get_loaded_value::<T>() {
                return Some(value);
            }
        }

        None
    }

    #[tracing::instrument(skip(assets), level = "debug")]
    fn peek(&self, assets: &AssetCache) -> Option<T> {
        // Use of `in_background` start a task that keeps loading
        self.clone().in_background().get(assets).now_or_never()
    }
}

pub trait Asset {
    type WeakType: Clone + Sync + Send;
    fn to_weak(strong: &Self) -> Self::WeakType;
    fn from_weak(weak: &Self::WeakType) -> Option<Self>
    where
        Self: Sized;
}
impl<T: Sync + Send + ?Sized> Asset for Arc<T> {
    type WeakType = Weak<T>;
    fn to_weak(strong: &Self) -> Self::WeakType {
        Arc::downgrade(strong)
    }
    fn from_weak(weak: &Self::WeakType) -> Option<Self> {
        Weak::upgrade(weak)
    }
}

impl<T: Asset + Sync + Send, E: Clone + Sync + Send> Asset for Result<T, E> {
    type WeakType = Result<T::WeakType, E>;
    fn to_weak(strong: &Self) -> Self::WeakType {
        match strong {
            Ok(val) => Ok(T::to_weak(val)),
            Err(err) => Err(err.clone()),
        }
    }
    fn from_weak(weak: &Self::WeakType) -> Option<Self> {
        match weak {
            Ok(val) => T::from_weak(val).map(|x| Ok(x)),
            Err(err) => Some(Err(err.clone())),
        }
    }
}

impl<T: Asset + Sync + Send> Asset for Option<T> {
    type WeakType = Option<T::WeakType>;
    fn to_weak(strong: &Self) -> Self::WeakType {
        strong.as_ref().map(|val| T::to_weak(val))
    }
    fn from_weak(weak: &Self::WeakType) -> Option<Self> {
        match weak {
            Some(val) => T::from_weak(val).map(|x| Some(x)),
            None => Some(None),
        }
    }
}

impl<T0: Asset + Sync + Send, T1: Asset + Sync + Send> Asset for (T0, T1) {
    type WeakType = (T0::WeakType, T1::WeakType);
    fn to_weak((a, b): &Self) -> Self::WeakType {
        (T0::to_weak(a), T1::to_weak(b))
    }
    fn from_weak((a, b): &Self::WeakType) -> Option<Self> {
        Some((T0::from_weak(a)?, T1::from_weak(b)?))
    }
}

impl<T: Asset + Sync + Send> Asset for Vec<T> {
    type WeakType = Vec<T::WeakType>;
    fn to_weak(v: &Self) -> Self::WeakType {
        v.iter().map(|x| T::to_weak(x)).collect()
    }
    fn from_weak(v: &Self::WeakType) -> Option<Self> {
        v.iter().map(|x| T::from_weak(x)).collect::<Option<Vec<_>>>()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetLifetime {
    pub start_load: chrono::DateTime<chrono::Utc>,
    pub end_load: Option<chrono::DateTime<chrono::Utc>>,
    pub keepalive_start: Option<chrono::DateTime<chrono::Utc>>,
    pub keepalive_end: Option<chrono::DateTime<chrono::Utc>>,
    pub dropped: Option<chrono::DateTime<chrono::Utc>>,
    pub aborted: Option<chrono::DateTime<chrono::Utc>>,
    pub keepalive: bool,
}
impl AssetLifetime {
    pub fn end_time(&self) -> chrono::DateTime<chrono::Utc> {
        if let Some(aborted) = self.aborted {
            aborted
        } else if let Some(dropped) = self.dropped {
            dropped
        } else {
            chrono::Utc::now()
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssetTimeline {
    pub long_name: String,
    pub stack: Vec<AssetKey>,
    pub cpu_size: Option<usize>,
    pub gpu_size: Option<usize>,
    pub lifetimes: Vec<AssetLifetime>,
    pub is_alive: bool,
}
impl AssetTimeline {
    pub fn is_loading(&self) -> bool {
        self.lifetimes.last().map(|x| x.end_load.is_none() && x.aborted.is_none()).unwrap_or(false)
    }
    pub fn is_aborted(&self) -> bool {
        self.lifetimes.last().map(|x| x.aborted.is_some()).unwrap_or(false)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssetsTimeline {
    pub assets: HashMap<AssetKey, AssetTimeline>,
    pub start_time: chrono::DateTime<chrono::Utc>,
}
impl AssetsTimeline {
    pub fn new() -> Self {
        Self { assets: Default::default(), start_time: chrono::Utc::now() }
    }
    pub fn n_loading(&self) -> usize {
        self.assets.values().filter(|x| x.is_loading()).count()
    }

    fn start_load(&mut self, key: AssetKey, long_name: String, stack: Vec<AssetKey>, keepalive: bool) {
        let asset = self.assets.entry(key).or_default();
        asset.long_name = long_name;
        asset.stack = stack;
        asset.is_alive = true;
        asset.lifetimes.push(AssetLifetime {
            start_load: chrono::Utc::now(),
            end_load: None,
            keepalive_start: None,
            keepalive_end: None,
            dropped: None,
            aborted: None,
            keepalive,
        });
    }
    fn last_lifetime(&mut self, key: &AssetKey) -> &mut AssetLifetime {
        self.assets.get_mut(key).unwrap().lifetimes.last_mut().unwrap()
    }
    fn end_load(&mut self, key: &AssetKey, cpu_size: Option<usize>, gpu_size: Option<usize>) {
        let asset = self.assets.get_mut(key).unwrap();
        asset.lifetimes.last_mut().unwrap().end_load = Some(chrono::Utc::now());
        asset.cpu_size = cpu_size;
        asset.gpu_size = gpu_size;
    }

    fn keepalive_end(&mut self, key: &AssetKey) {
        self.last_lifetime(key).keepalive_end = Some(chrono::Utc::now());
    }

    fn keepalive_start(&mut self, key: &AssetKey) {
        let lf = self.last_lifetime(key);
        lf.keepalive_end = None;
        // Keep the oldest keepalive start
        lf.keepalive_start.get_or_insert(chrono::Utc::now());
    }

    fn dropped(&mut self, key: &AssetKey) {
        let asset = self.assets.get_mut(key).unwrap();
        asset.lifetimes.last_mut().unwrap().dropped = Some(chrono::Utc::now());
        asset.is_alive = false;
    }
    fn aborted(&mut self, key: &AssetKey) {
        let asset = self.assets.get_mut(key).unwrap();
        let lf = asset.lifetimes.last_mut().unwrap();

        if lf.aborted.is_none() {
            lf.aborted = Some(chrono::Utc::now())
        }

        asset.is_alive = false;
    }
}

#[pin_project(PinnedDrop)]
struct AssetLoadFuture<F, K> {
    // Where to store the result
    cache: Arc<Mutex<HashMap<AssetKey, AsyncAssetLoc>>>,
    asset_key: AssetKey,
    timeline: Arc<Mutex<AssetsTimeline>>,
    #[pin]
    fut: F,
    completed: bool,
    key: K,
}

impl<K, F, T> Future for AssetLoadFuture<F, K>
where
    K: AsyncAssetKey<T>,
    F: Future<Output = T>,
    T: 'static + Asset + Clone + Send + Sync,
{
    /// Returns the strong variant
    type Output = LoadPayload;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let p = self.project();

        if let Poll::Ready(res) = p.fut.poll(cx) {
            *p.completed = true;

            // Update the timeline with the size
            let cpu_size = p.key.cpu_size(&res);
            let gpu_size = p.key.gpu_size(&res);
            p.timeline.lock().end_load(p.asset_key, cpu_size, gpu_size);

            let weak_res = Arc::new(T::to_weak(&res)) as Arc<dyn AssetHolder>;

            let check_alive = Arc::new({
                let weak_res = T::to_weak(&res);
                move || T::from_weak(&weak_res).is_some()
            });

            // Type erase
            let value = Arc::new(res) as Arc<dyn AssetHolder>;

            // Update the content state
            let mut cache = p.cache.lock();
            let mut loc = cache.get_mut(p.asset_key).expect("Asset loc was removed during loading");

            // Replace the loading state with the loaded state
            assert!(loc.content.is_loading());
            loc.content = ContentState::Loaded { value: weak_res, check_alive };

            Poll::Ready(LoadPayload { asset_key: p.asset_key.clone(), strong: value })
        } else {
            Poll::Pending
        }
    }
}

#[pinned_drop]
impl<F, K> PinnedDrop for AssetLoadFuture<F, K> {
    fn drop(self: Pin<&mut Self>) {
        if !self.completed {
            let mut cache = self.cache.lock();
            let loc = cache.get_mut(&self.asset_key).expect("Asset loc was removed during loading");
            assert!(loc.content.is_loading());
            loc.content = ContentState::Aborted;
        }
    }
}

#[cfg(test)]
mod test {
    use tokio::{runtime, time::timeout};

    use super::*;

    #[derive(PartialEq, Eq, Debug)]
    struct TestAsset {
        name: String,
    }

    #[derive(Debug, Clone)]
    struct TestAssetKey {
        name: String,
    }

    #[async_trait]
    impl AsyncAssetKey<Arc<TestAsset>> for TestAssetKey {
        async fn load(self, _: AssetCache) -> Arc<TestAsset> {
            tokio::time::sleep(Duration::from_secs(1)).await;

            Arc::new(TestAsset { name: self.name })
        }
    }

    #[tokio::test]
    async fn load_aborted() {
        let assets = AssetCache::new(runtime::Handle::current());

        let asset = timeout(Duration::from_millis(200), TestAssetKey { name: "foo".into() }.get(&assets)).await;

        assert!(asset.is_err());

        let state = assets.content_state(&TestAssetKey { name: "foo".into() });

        assert!(matches!(state, Some(ContentState::Aborted)));

        let state = assets.content_state(&TestAssetKey { name: "bar".into() });
        assert!(matches!(state, None));

        let asset = TestAssetKey { name: "foo".into() }.get(&assets).await;

        assert_eq!(&*asset, &TestAsset { name: "foo".into() });

        let state = assets.content_state(&TestAssetKey { name: "foo".into() });
        assert!(matches!(state, Some(ContentState::Loaded { .. })));
    }

    #[tokio::test]
    async fn test_weak_asset() {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(1);
        #[derive(Debug, Clone)]
        struct Key;
        #[async_trait]
        impl AsyncAssetKey<Result<Arc<u32>, u32>> for Key {
            async fn load(self, _assets: AssetCache) -> Result<Arc<u32>, u32> {
                let val = COUNTER.fetch_add(1, Ordering::SeqCst);
                if val <= 2 {
                    Ok(Arc::new(val))
                } else {
                    Err(val)
                }
            }
            fn keepalive(&self) -> AssetKeepalive {
                AssetKeepalive::None
            }
        }

        let assets = AssetCache::new(tokio::runtime::Handle::current());
        {
            let val = Key.get(&assets).await.unwrap();
            assert_eq!(*val, 1);
            let val2 = Key.get(&assets).await.unwrap();
            assert_eq!(*val2, 1);
        }
        {
            let val = Key.get(&assets).await.unwrap();
            assert_eq!(*val, 2);
        }
        {
            let val = Key.get(&assets).await.unwrap_err();
            assert_eq!(val, 3);
        }
        {
            let val = Key.get(&assets).await.unwrap_err();
            assert_eq!(val, 3);
        }
    }
}

struct KeepaliveGuard {
    key: AssetKey,
    timeline: Arc<Mutex<AssetsTimeline>>,
}

impl KeepaliveGuard {
    fn begin(key: AssetKey, timeline: Arc<Mutex<AssetsTimeline>>) -> Self {
        timeline.lock().keepalive_start(&key);
        Self { key, timeline }
    }
}

impl Drop for KeepaliveGuard {
    fn drop(&mut self) {
        self.timeline.lock().keepalive_end(&self.key)
    }
}

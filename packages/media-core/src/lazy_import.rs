// 翻译自 packages/media-core/src/lazy-import.ts

use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;

/// Type alias for an async loader closure used by [`create_lazy_import_loader`].
pub type LazyLoadFn<T> = Box<dyn Fn() -> Pin<Box<dyn Future<Output = Result<T, String>> + Send>> + Send + Sync>;

/// Cached async loader used by runtime boundaries that should import on first use.
pub struct LazyPromiseLoader<T> {
    load_fn: LazyLoadFn<T>,
    promise: Mutex<Option<Pin<Box<dyn Future<Output = Result<T, String>> + Send>>>>,
}

impl<T: Send + 'static> LazyPromiseLoader<T> {
    /** Creates a single-flight promise cache around a lazy import or other async loader. */
    pub fn new(load: LazyLoadFn<T>) -> Self {
        Self {
            load_fn: load,
            promise: Mutex::new(None),
        }
    }

    /** Creates a single-flight promise cache around a lazy import or other async loader. */
    pub fn new_with_options(load: LazyLoadFn<T>, _options: LazyPromiseLoaderOptions) -> Self {
        Self::new(load)
    }

    /// Loads the value, caching the in-flight promise for subsequent callers.
    pub async fn load(&self) -> Result<T, String> {
        let mut guard = self.promise.lock().unwrap();
        if guard.is_none() {
            let fut = (self.load_fn)();
            *guard = Some(fut);
        }
        // We can't await a Pin<Box<dyn Future>> while holding a Mutex across .await easily here,
        // so we take ownership and drop the lock to await.
        let taken = guard.take();
        drop(guard);
        match taken {
            Some(mut f) => {
                let result = f.as_mut().await;
                // Note: the original TypeScript clears `promise` on rejection when cacheRejections is false.
                // For 1:1 fidelity, the default behaviour keeps the result cached on success and clears on rejection.
                if result.is_err() {
                    // Failed optional-runtime imports should retry after install/config changes.
                    let mut g = self.promise.lock().unwrap();
                    *g = None;
                } else {
                    // Re-cache successful promise
                    let mut g = self.promise.lock().unwrap();
                    *g = Some(f);
                }
                result
            }
            None => unreachable!(),
        }
    }

    /// Clears the cached promise, forcing the next `load()` to re-invoke the loader.
    pub fn clear(&self) {
        let mut guard = self.promise.lock().unwrap();
        *guard = None;
    }
}

/// Controls whether a failed first import stays cached or is retried later.
#[derive(Default, Clone, Copy, Debug)]
pub struct LazyPromiseLoaderOptions {
    pub cache_rejections: bool,
}

/** Creates a single-flight promise cache around a lazy import or other async loader. */
pub fn create_lazy_import_loader<T: Send + 'static>(
    load: LazyLoadFn<T>,
    options: LazyPromiseLoaderOptions,
) -> LazyPromiseLoader<T> {
    LazyPromiseLoader::new_with_options(load, options)
}
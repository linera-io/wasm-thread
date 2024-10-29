use std::{
    io,
    num::NonZeroUsize,
    sync::{LockResult, Mutex, MutexGuard, TryLockError},
};

use wasm_bindgen::prelude::*;
use web_sys::{DedicatedWorkerGlobalScope, WorkerGlobalScope};

pub fn available_parallelism() -> io::Result<NonZeroUsize> {
    if let Some(window) = web_sys::window() {
        return Ok(NonZeroUsize::new(window.navigator().hardware_concurrency() as usize).unwrap());
    }

    if let Ok(worker) = js_sys::global().dyn_into::<WorkerGlobalScope>() {
        return Ok(NonZeroUsize::new(worker.navigator().hardware_concurrency() as usize).unwrap());
    }

    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "hardware_concurrency unsupported",
    ))
}

pub fn is_web_worker_thread() -> bool {
    js_sys::global().dyn_into::<DedicatedWorkerGlobalScope>().is_ok()
}

/// Extracts path of the `wasm_bindgen` generated .js shim script.
pub fn get_wasm_bindgen_shim_script_path() -> String {
    #[wasm_bindgen]
    #[allow(non_snake_case)]
    extern "C" {
        #[wasm_bindgen(thread_local, js_namespace = ["import", "meta"], js_name = url)]
        static IMPORT_META_URL: String;
    }

    IMPORT_META_URL.with(|s| s.clone())
}

/// A spin lock mutex extension.
///
/// Atomic wait panics in wasm main thread so we can't use `Mutex::lock()`.
/// This is a helper, which implement spinlock by calling `Mutex::try_lock()` in a loop.
/// Care must be taken not to introduce deadlocks when using this trait.
pub trait SpinLockMutex {
    type Inner;

    fn lock_spin<'a>(&'a self) -> LockResult<MutexGuard<'a, Self::Inner>>;
}

impl<T> SpinLockMutex for Mutex<T> {
    type Inner = T;

    fn lock_spin<'a>(&'a self) -> LockResult<MutexGuard<'a, Self::Inner>> {
        loop {
            match self.try_lock() {
                Ok(guard) => break Ok(guard),
                Err(TryLockError::WouldBlock) => {}
                Err(TryLockError::Poisoned(e)) => break Err(e),
            }
        }
    }
}

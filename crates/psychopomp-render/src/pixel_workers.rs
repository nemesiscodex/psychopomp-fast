//! Bounded CPU workers for independent pixel rows, shared by text and exposure.
use std::sync::OnceLock;

use rayon::{ThreadPool, ThreadPoolBuilder};

pub(crate) fn pool() -> Option<&'static ThreadPool> {
    static POOL: OnceLock<Option<ThreadPool>> = OnceLock::new();
    POOL.get_or_init(|| {
        let threads = std::thread::available_parallelism()
            .map_or(1, usize::from)
            .min(4);
        (threads > 1)
            .then(|| ThreadPoolBuilder::new().num_threads(threads).build().ok())
            .flatten()
    })
    .as_ref()
}

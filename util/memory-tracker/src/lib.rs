mod config;

#[cfg(all(
    not(target_env = "msvc"),
    not(target_os = "macos"),
    feature = "profiling"
))]
mod jemalloc;
#[cfg(not(all(
    not(target_env = "msvc"),
    not(target_os = "macos"),
    feature = "profiling"
)))]
mod jemalloc {
    use ckb_logger::warn;

    pub fn jemalloc_profiling_dump(_: String) {
        warn!("jemalloc profiling dump: unsupported");
    }
}

#[cfg(all(not(target_env = "msvc"), not(target_os = "macos")))]
mod process;
#[cfg(not(all(not(target_env = "msvc"), not(target_os = "macos"))))]
mod process {
    use ckb_logger::info;
    use ckb_shared::shared::Shared;

    pub fn track_current_process(_: u64, _: Option<Shared>) {
        info!("track current process: unsupported");
    }
}
pub(crate) mod rocksdb;
pub(crate) mod utils;

pub use config::Config;
pub use jemalloc::jemalloc_profiling_dump;
pub use process::track_current_process;

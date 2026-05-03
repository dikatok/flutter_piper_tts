use std::sync::atomic::{AtomicBool, Ordering};

use log::warn;

static LOGGER_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub(crate) fn init_logger() {
    if LOGGER_INITIALIZED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        warn!("Logger already initialized");
        return;
    }

    #[cfg(target_os = "android")]
    {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Debug)
                .with_tag("flutter_piper_tts_native"),
        );
    }

    #[cfg(any(target_os = "ios"))]
    {
        oslog::OsLogger::new("flutter_piper_tts_native")
            .level_filter(log::LevelFilter::Debug)
            .init()
            .unwrap();
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let mut builder = env_logger::Builder::new();

        if std::env::var("RUST_LOG").is_ok() {
            builder.parse_env("RUST_LOG");
        } else {
            builder.filter_level(log::LevelFilter::Debug);
        }

        builder.init();
    }
}

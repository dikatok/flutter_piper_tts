pub(crate) fn init_logger() {
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
        env_logger::init();
    }
}

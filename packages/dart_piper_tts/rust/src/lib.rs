use std::{
    collections::HashMap,
    ffi::CString,
    os::raw::c_char,
    path::Path,
    sync::{
        Arc, LazyLock, Mutex, OnceLock, RwLock,
        atomic::{AtomicBool, AtomicI32, Ordering},
    },
};

use espeak_ng::install_bundled_data;
use log::{debug, warn};

use crate::{audio::AudioPlayer, helpers::c_char_to_str, instance::Instance, logging::init_logger};

pub mod audio;
pub mod config;
pub mod error;
pub mod helpers;
pub mod inference;
pub mod instance;
pub mod logging;

static INSTANCES: LazyLock<RwLock<HashMap<i32, Instance>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

static FD: AtomicI32 = AtomicI32::new(0);

static ESPEAK_DATA_DIR: LazyLock<RwLock<String>> = LazyLock::new(|| RwLock::new("".to_string()));

static AUDIO_PLAYER: OnceLock<Mutex<AudioPlayer>> = OnceLock::new();

pub type DartPort = i64;
pub type CompletionCallback = unsafe extern "C" fn(port: DartPort);
static COMPLETION_CB: OnceLock<Mutex<Option<CompletionCallback>>> = OnceLock::new();

#[unsafe(no_mangle)]
pub extern "C" fn init(
    data_dir: *const c_char,
    completion_cb: CompletionCallback,
) -> FFIInitResponse {
    init_logger();

    let mut global_data_dir = ESPEAK_DATA_DIR.write().unwrap();
    match global_data_dir.as_str() {
        "" => {
            let binding = Path::new(c_char_to_str(data_dir)).join("espeak-ng-data");
            let path = binding.as_path();
            debug!("espeak data dir: {}", path.display());

            match install_bundled_data(path) {
                Ok(_) => (),
                Err(err) => {
                    return FFIInitResponse {
                        error_message: convert_string_to_cstring(&err.to_string()),
                    };
                }
            };
            *global_data_dir = path.to_str().unwrap().to_string();
            debug!("espeak bundled data installed");
        }
        _ => {
            warn!("espeak data already installed, skipping");
        }
    }

    *COMPLETION_CB
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap() = Some(completion_cb);

    AUDIO_PLAYER.get_or_init(|| Mutex::new(AudioPlayer::default()));

    FFIInitResponse {
        error_message: convert_string_to_cstring(""),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn create_instance(
    model_path: *const c_char,
    config_path: *const c_char,
) -> FFICreateInstanceResponse {
    let model = match Instance::new(
        Path::new(c_char_to_str(model_path)),
        Path::new(c_char_to_str(config_path)),
    ) {
        Ok(model) => model,
        Err(err) => {
            return FFICreateInstanceResponse {
                fd: -1,
                error_message: convert_string_to_cstring(&err.message),
            };
        }
    };
    let fd = FD.fetch_add(1, Ordering::SeqCst);
    INSTANCES.write().unwrap().insert(fd, model);
    FFICreateInstanceResponse {
        fd: fd,
        error_message: convert_string_to_cstring(""),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn speak(fd: i32, text: *const c_char, port: DartPort) -> FFISpeakResponse {
    with_instance_mut(
        fd,
        FFISpeakResponse {
            error_message: convert_string_to_cstring("Instance not found"),
        },
        |instance| match instance.speak(c_char_to_str(text), port.clamp(-1, i64::MAX)) {
            Ok(_) => FFISpeakResponse {
                error_message: convert_string_to_cstring(""),
            },
            Err(err) => FFISpeakResponse {
                error_message: convert_string_to_cstring(&err.message),
            },
        },
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn pause(fd: i32) -> FFIPauseResponse {
    with_instance_mut(
        fd,
        FFIPauseResponse {
            error_message: convert_string_to_cstring("Instance not found"),
        },
        |instance| {
            instance.pause();
            FFIPauseResponse {
                error_message: convert_string_to_cstring(""),
            }
        },
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn resume(fd: i32) -> FFIResumeResponse {
    with_instance_mut(
        fd,
        FFIResumeResponse {
            error_message: convert_string_to_cstring("Instance not found"),
        },
        |instance| {
            instance.resume();
            FFIResumeResponse {
                error_message: convert_string_to_cstring(""),
            }
        },
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn stop(fd: i32) -> FFIStopResponse {
    with_instance_mut(
        fd,
        FFIStopResponse {
            error_message: convert_string_to_cstring("Instance not found"),
        },
        |instance| {
            instance.stop();
            FFIStopResponse {
                error_message: convert_string_to_cstring(""),
            }
        },
    )
}

fn with_instance_mut<T, F>(fd: i32, not_found: T, f: F) -> T
where
    F: FnOnce(&mut Instance) -> T,
{
    let mut binding = INSTANCES.write().unwrap();
    match binding.get_mut(&fd) {
        Some(instance) => f(instance),
        None => not_found,
    }
}

fn convert_string_to_cstring(s: &str) -> *mut c_char {
    CString::new(s).unwrap().into_raw()
}

#[repr(C)]
pub struct FFIInitResponse {
    pub error_message: *mut c_char,
}

#[repr(C)]
pub struct FFICreateInstanceResponse {
    pub fd: i32,
    pub error_message: *mut c_char,
}

#[repr(C)]
pub struct FFISpeakResponse {
    pub error_message: *mut c_char,
}

#[repr(C)]
pub struct FFIPauseResponse {
    pub error_message: *mut c_char,
}

#[repr(C)]
pub struct FFIResumeResponse {
    pub error_message: *mut c_char,
}

#[repr(C)]
pub struct FFIStopResponse {
    pub error_message: *mut c_char,
}

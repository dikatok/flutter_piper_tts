use std::{
    collections::HashMap,
    ffi::CString,
    os::raw::c_char,
    path::Path,
    sync::{
        LazyLock, Mutex, OnceLock, RwLock,
        atomic::{AtomicI32, Ordering},
    },
};

use crate::{
    audio::AudioPlayer, helpers::c_char_to_str, instance::Instance, logging::init_logger,
    phonemizer::Phonemizer,
};

pub mod audio;
pub mod config;
pub mod error;
pub mod helpers;
pub mod inference;
pub mod instance;
pub mod logging;
pub mod phonemizer;
pub mod tokenizer;

static INSTANCES: LazyLock<RwLock<HashMap<i32, Instance>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

static FD: AtomicI32 = AtomicI32::new(0);

static AUDIO_PLAYER: OnceLock<Mutex<AudioPlayer>> = OnceLock::new();

pub type DartPort = i64;
pub type CompletionCallback = unsafe extern "C" fn(port: DartPort);
static COMPLETION_CB: RwLock<Option<Mutex<CompletionCallback>>> = RwLock::new(None);

static PHONEMIZER_SESSION: OnceLock<Mutex<Phonemizer>> = OnceLock::new();

#[unsafe(no_mangle)]
pub extern "C" fn init(
    phonemizer_model_path: *const c_char,
    completion_cb: CompletionCallback,
    is_debug: bool,
) -> FFIInitResponse {
    init_logger(is_debug);

    let mut cb_guard = COMPLETION_CB.write().unwrap();
    *cb_guard = Some(Mutex::new(completion_cb));

    PHONEMIZER_SESSION.get_or_init(|| {
        Mutex::new(Phonemizer::load(Path::new(c_char_to_str(phonemizer_model_path))).unwrap())
    });

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
pub extern "C" fn speak(
    fd: i32,
    text: *const c_char,
    is_phonemes: bool,
    port: DartPort,
) -> FFISpeakResponse {
    with_instance_mut(
        fd,
        FFISpeakResponse {
            error_message: convert_string_to_cstring("instance not initialized"),
        },
        |instance| match instance.speak(c_char_to_str(text), is_phonemes, port.clamp(-1, i64::MAX))
        {
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
            error_message: convert_string_to_cstring("instance not initialized"),
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
            error_message: convert_string_to_cstring("instance not initialized"),
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
            error_message: convert_string_to_cstring("instance not initialized"),
        },
        |instance| {
            instance.stop();
            FFIStopResponse {
                error_message: convert_string_to_cstring(""),
            }
        },
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn dispose(fd: i32) -> FFIDisposeResponse {
    match INSTANCES.write().unwrap().remove(&fd) {
        Some(_) => FFIDisposeResponse {
            error_message: convert_string_to_cstring(""),
        },
        None => FFIDisposeResponse {
            error_message: convert_string_to_cstring("instance not initialized"),
        },
    }
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

#[repr(C)]
pub struct FFIDisposeResponse {
    pub error_message: *mut c_char,
}

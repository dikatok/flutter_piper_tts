use std::{
    ffi::{CString, c_char, c_void},
    path::Path,
    sync::{Mutex, OnceLock, RwLock},
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
        error_message: std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn create_instance(
    model_path: *const c_char,
    config_path: *const c_char,
) -> FFICreateInstanceResponse {
    let instance = match Instance::new(
        Path::new(c_char_to_str(model_path)),
        Path::new(c_char_to_str(config_path)),
    ) {
        Ok(instance) => instance,
        Err(err) => {
            return FFICreateInstanceResponse {
                instance: std::ptr::null_mut(),
                error_message: CString::new(err.message).unwrap().into_raw(),
            };
        }
    };
    FFICreateInstanceResponse {
        instance: Box::into_raw(Box::new(instance)) as *mut c_void,
        error_message: std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn speak(
    instance_ptr: *mut c_void,
    text: *const c_char,
    is_phonemes: bool,
    port: DartPort,
) -> FFISpeakResponse {
    let instance = unsafe { &mut *(instance_ptr as *mut Instance) };
    match instance.speak(c_char_to_str(text), is_phonemes, port.clamp(-1, i64::MAX)) {
        Ok(_) => FFISpeakResponse {
            error_message: std::ptr::null_mut(),
        },
        Err(err) => FFISpeakResponse {
            error_message: CString::new(err.message).unwrap().into_raw(),
        },
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn pause(instance_ptr: *mut c_void) -> FFIPauseResponse {
    let instance = unsafe { &mut *(instance_ptr as *mut Instance) };
    instance.pause();
    FFIPauseResponse {
        error_message: std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn resume(instance_ptr: *mut c_void) -> FFIResumeResponse {
    let instance = unsafe { &mut *(instance_ptr as *mut Instance) };
    instance.resume();
    FFIResumeResponse {
        error_message: std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn stop(instance_ptr: *mut c_void) -> FFIStopResponse {
    let instance = unsafe { &mut *(instance_ptr as *mut Instance) };
    instance.stop();
    FFIStopResponse {
        error_message: std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dispose(instance_ptr: *mut c_void) {
    if !instance_ptr.is_null() {
        // Taking ownership back means Rust will automatically
        // drop it when this block ends.
        let _ = unsafe { Box::from_raw(instance_ptr as *mut Instance) };
    }
}

#[repr(C)]
pub struct FFIInitResponse {
    pub error_message: *mut c_char,
}

#[repr(C)]
pub struct FFICreateInstanceResponse {
    pub instance: *mut c_void,
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

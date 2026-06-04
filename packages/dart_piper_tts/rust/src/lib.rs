use std::{
    ffi::{CStr, CString, c_char, c_void},
    path::Path,
    str::FromStr,
};

use crate::{
    audio::{AudioPlayer, AudioPlayerConfig, CompletionCallback, DartPort},
    instance::Instance,
    logging::init_logger,
    phonemizer::phonemizer::{PhonemizationStrategy, Phonemizer},
};

pub(crate) mod audio;
pub(crate) mod config;
pub(crate) mod error;
pub(crate) mod inference;
pub(crate) mod instance;
pub(crate) mod logging;
pub(crate) mod phonemizer;

#[unsafe(no_mangle)]
pub extern "C" fn init(completion_cb: CompletionCallback, is_debug: bool) -> FFIInitResponse {
    init_logger(is_debug);

    AudioPlayer::init(AudioPlayerConfig::new(None, None, completion_cb));

    Phonemizer::init();

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
    phonemization_strategy: *const c_char,
) -> FFISpeakResponse {
    let instance = unsafe { &mut *(instance_ptr as *mut Instance) };
    match instance.speak(
        c_char_to_str(text),
        is_phonemes,
        port.clamp(-1, i64::MAX),
        PhonemizationStrategy::from_str(c_char_to_str(phonemization_strategy)).unwrap(),
    ) {
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

fn c_char_to_str(ptr: *const c_char) -> &'static str {
    // 1. Safety check for null
    if ptr.is_null() { /* handle error */ }

    // 2. Convert raw pointer to CStr
    let c_str = unsafe { CStr::from_ptr(ptr) };

    c_str.to_str().unwrap()
}

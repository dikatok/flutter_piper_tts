use std::{
    collections::HashMap,
    ffi::CString,
    os::raw::c_char,
    path::Path,
    sync::{
        LazyLock, RwLock,
        atomic::{AtomicI32, Ordering},
    },
};

use espeak_ng::install_bundled_data;
use log::{debug, info};

use crate::{helpers::c_char_to_str, instance::Instance};

pub mod config;
pub mod error;
pub mod helpers;
pub mod inference;
pub mod instance;

static INSTANCES: LazyLock<RwLock<HashMap<i32, Instance>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

static FD: AtomicI32 = AtomicI32::new(0);

static ESPEAK_DATA_DIR: LazyLock<RwLock<String>> = LazyLock::new(|| RwLock::new("".to_string()));

#[unsafe(no_mangle)]
pub extern "C" fn init(data_dir: *const c_char) -> FFIInitResponse {
    #[cfg(target_os = "android")]
    {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Debug)
                .with_tag("MY_RUST_APP"),
        );
        debug!("android logger initialized");
    }

    let mut global_data_dir = ESPEAK_DATA_DIR.write().unwrap();
    let binding = Path::new(c_char_to_str(data_dir)).join("espeak-ng-data");
    let path = binding.as_path();
    *global_data_dir = path.to_str().unwrap().to_string();

    debug!("installing espeak bundled data");
    match install_bundled_data(path) {
        Ok(_) => (),
        Err(err) => {
            return FFIInitResponse {
                error_message: convert_string_to_cstring(&err.to_string()),
            };
        }
    };
    info!("espeak bundled data installed");

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
pub extern "C" fn speak(fd: i32, text: *const c_char) -> FFISpeakResponse {
    println!("speak");
    let mut binding = INSTANCES.write().unwrap();
    let instance = match binding.get_mut(&fd) {
        Some(model) => model,
        None => {
            return FFISpeakResponse {
                error_message: convert_string_to_cstring("Instance not found"),
            };
        }
    };
    match instance.speak(c_char_to_str(text)) {
        Ok(_) => FFISpeakResponse {
            error_message: convert_string_to_cstring(""),
        },
        Err(err) => FFISpeakResponse {
            error_message: convert_string_to_cstring(&err.message),
        },
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

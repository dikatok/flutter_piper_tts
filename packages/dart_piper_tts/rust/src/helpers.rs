use std::ffi::{CStr, c_char};

pub(crate) fn c_char_to_str(ptr: *const c_char) -> &'static str {
    // 1. Safety check for null
    if ptr.is_null() { /* handle error */ }

    // 2. Convert raw pointer to CStr
    let c_str = unsafe { CStr::from_ptr(ptr) };

    c_str.to_str().unwrap()
}

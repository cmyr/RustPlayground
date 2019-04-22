#[macro_use]
extern crate serde_json;

use std::ffi::{CStr, CString};

use libc::{c_char, int32_t, size_t, uint32_t};

use playground_utils::list_toolchains;

#[no_mangle]
pub extern "C" fn playgroundGetToolchains() -> *const c_char {
    let response = match list_toolchains() {
        Ok(toolchains) => json!({ "result": toolchains }),
        Err(e) => json!({ "error": e.to_string() }),
    };

    let response_str = serde_json::to_string(&response).unwrap();
    let cstring = CString::new(response_str).expect("nul byte in response json");
    cstring.into_raw()
}

#[no_mangle]
pub extern "C" fn playgroundStringFree(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }

    unsafe {
        CString::from_raw(ptr);
    }
}

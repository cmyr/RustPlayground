#[macro_use]
extern crate serde_json;

use libc::c_char;
use std::ffi::{CStr, CString, OsStr};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use playground_utils::{do_compile_task, list_toolchains, Error, Task};

#[no_mangle]
pub extern "C" fn playgroundGetToolchains() -> *const c_char {
    let response = match list_toolchains() {
        Ok(toolchains) => json!({ "result": toolchains }),
        Err(e) => to_json_error(e),
    };

    let response_str = serde_json::to_string(&response).unwrap();
    let cstring = CString::new(response_str).expect("nul byte in response json");
    cstring.into_raw()
}

#[no_mangle]
pub extern "C" fn playgroundExecuteTask(
    path: *const c_char,
    cmd_json: *const c_char,
) -> *const c_char {
    let path = unsafe { CStr::from_ptr(path) };
    let json = unsafe { CStr::from_ptr(cmd_json) };
    let path = Path::new(OsStr::from_bytes(path.to_bytes()));
    let json = json.to_str().expect("json must be valid utf8");
    let task: Task = serde_json::from_str(json).expect("malformed task json");
    let response = match do_compile_task(path, task) {
        Ok(result) => json!({ "result": result }),
        Err(e) => to_json_error(e),
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

fn to_json_error(e: Error) -> serde_json::Value {
    json!({
        "error": {
            "message": e.to_string(),
            "code": e.error_code(),
        }
    })
}

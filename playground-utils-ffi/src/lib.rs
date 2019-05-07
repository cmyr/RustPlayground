use libc::c_char;
use std::ffi::{CStr, CString, OsStr};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use ffi_support::{call_with_result, ExternError};
use playground_utils::{do_compile_task, list_toolchains, Task};

#[no_mangle]
pub extern "C" fn playgroundGetToolchains(err: &mut ExternError) -> *const c_char {
    call_with_result(err, || {
        list_toolchains()
            .map(|r| serde_json::to_string(&r).unwrap())
    })
}

#[no_mangle]
pub extern "C" fn playgroundExecuteTask(
    path: *const c_char,
    cmd_json: *const c_char,
    std_err_callback: extern "C" fn(*const c_char),
    err: &mut ExternError,
) -> *const c_char {
    call_with_result(err, || {
        eprintln!("playground execute task");
        let path = unsafe { CStr::from_ptr(path) };
        let json = unsafe { CStr::from_ptr(cmd_json) };
        let path = Path::new(OsStr::from_bytes(path.to_bytes()));
        let json = json.to_str().expect("json must be valid utf8");
        let task: Task = serde_json::from_str(json).expect("malformed task json");
        do_compile_task(path, task, |stderr| {
            let cstring = CString::new(stderr)
                .unwrap_or_else(|_| CString::new("null byte in stderr").unwrap());
            std_err_callback(cstring.as_ptr());
        })
        .map(|r| serde_json::to_string(&r).unwrap())
    })
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

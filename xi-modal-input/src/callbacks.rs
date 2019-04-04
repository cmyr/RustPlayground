//! FFI friendly wrappers around extern fns

use std::ffi::CString;
use std::ops::Range;

use libc::{c_char, size_t};
use serde_json::Value;

pub struct RpcCallback(extern "C" fn(*const c_char));

impl RpcCallback {
    pub fn call<V: Into<Option<Value>>>(&self, method: &str, params: V) {
        let params = params.into().unwrap_or(json!({}));
        let rpc = json!({
            "method": method,
            "params": params,
        });

        let asstr = serde_json::to_string(&rpc).expect("to_string failed");
        let cstr = CString::new(asstr).expect("nul byte in string :( :(");
        (self.0)(cstr.into_raw())
    }
}

impl From<extern "C" fn(*const c_char)> for RpcCallback {
    fn from(src: extern "C" fn(*const c_char)) -> RpcCallback {
        RpcCallback(src)
    }
}

pub struct InvalidateCallback(extern "C" fn(size_t, size_t));

impl InvalidateCallback {
    pub fn call(&self, range: Range<usize>) {
        (self.0)(range.start, range.end)
    }
}

impl From<extern "C" fn(size_t, size_t)> for InvalidateCallback {
    fn from(src: extern "C" fn(size_t, size_t)) -> InvalidateCallback {
        InvalidateCallback(src)
    }
}

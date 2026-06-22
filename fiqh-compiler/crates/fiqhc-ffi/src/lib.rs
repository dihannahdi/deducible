//! C-ABI / WebAssembly surface for the fiqhc engine (Open Core pillar #3).
//!
//! The SAME `extern "C"` functions serve two portability targets:
//!   * built to `wasm32-unknown-unknown` → a `.wasm` the browser loads to validate `.fiqh`
//!     instantly, client-side (great for interactive frontends);
//!   * built as a native `cdylib` (`.so`/`.dll`/`.dylib`) → linked by Java (JNI), C#
//!     (P/Invoke), C, Python (ctypes), Node-FFI … so the invariant engine embeds statically
//!     into legacy core-banking systems and databases that never touch a blockchain.
//!
//! Marshaling: the host allocates a buffer via `fiqh_alloc`, writes UTF-8 `.fiqh` source into
//! it, calls `fiqh_check_json(ptr,len)` and gets a NUL-terminated UTF-8 JSON string back
//! (`{ ok, diagnostics:[{code,severity,message,citation,line,col}] }`), then frees both.

use serde_json::json;
use std::os::raw::c_char;

/// Allocate `len` bytes in the module's memory; the host writes input here.
#[no_mangle]
pub extern "C" fn fiqh_alloc(len: usize) -> *mut u8 {
    let mut v = Vec::<u8>::with_capacity(len);
    let ptr = v.as_mut_ptr();
    std::mem::forget(v);
    ptr
}

/// Free a buffer previously returned by `fiqh_alloc`.
#[no_mangle]
pub extern "C" fn fiqh_free(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        unsafe {
            let _ = Vec::from_raw_parts(ptr, 0, len);
        }
    }
}

/// Check `.fiqh` source (UTF-8, `ptr`+`len`) and return a NUL-terminated JSON string.
/// Free the result with `fiqh_free_cstr`.
#[no_mangle]
pub extern "C" fn fiqh_check_json(ptr: *const u8, len: usize) -> *mut c_char {
    let src = if ptr.is_null() {
        ""
    } else {
        let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
        std::str::from_utf8(bytes).unwrap_or("")
    };
    let json = check_to_json(src);
    match std::ffi::CString::new(json) {
        Ok(c) => c.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Free a JSON string returned by `fiqh_check_json`.
#[no_mangle]
pub extern "C" fn fiqh_free_cstr(p: *mut c_char) {
    if !p.is_null() {
        unsafe {
            let _ = std::ffi::CString::from_raw(p);
        }
    }
}

fn check_to_json(src: &str) -> String {
    match fiqhc::compile_check(src) {
        Ok((_spec, diags)) => {
            let arr: Vec<_> = diags
                .iter()
                .map(|d| {
                    json!({
                        "code": d.code,
                        "severity": if d.is_error() { "error" } else { "warning" },
                        "message": d.message,
                        "citation": d.citation,
                        "line": d.span.line,
                        "col": d.span.col,
                    })
                })
                .collect();
            let ok = diags.iter().all(|d| !d.is_error());
            json!({ "ok": ok, "diagnostics": arr }).to_string()
        }
        Err((msg, span)) => json!({
            "ok": false,
            "diagnostics": [{
                "code": "PARSE", "severity": "error", "message": msg,
                "citation": "", "line": span.line, "col": span.col
            }]
        })
        .to_string(),
    }
}

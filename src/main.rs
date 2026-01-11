use cea_sys::*;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

fn create_ceil_array(name: &str) -> Vec<*const c_char> {
    vec![CString::new(name).unwrap().as_ptr()]
}

fn main() {
    unsafe {
        cea_set_log_level(CEA_LOG_DEBUG);
    }
    
}
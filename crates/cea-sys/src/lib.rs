#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code)]
#![allow(improper_ctypes, improper_ctypes_definitions)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub const CEA_SYS_VENDOR_DIR: Option<&str> = option_env!("CEA_SYS_VENDOR_DIR");
pub const CEA_SYS_INCLUDE_DIR: Option<&str> = option_env!("CEA_SYS_INCLUDE_DIR");
pub const CEA_SYS_THERMO_LIB: Option<&str> = option_env!("CEA_SYS_THERMO_LIB");
pub const CEA_SYS_TRANS_LIB: Option<&str> = option_env!("CEA_SYS_TRANS_LIB");

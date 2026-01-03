#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub const CEA_SYS_PREFIX: &str = env!("CEA_SYS_PREFIX");

// These may or may not be set depending on how CEA installed; build.rs tries to find them.
pub const CEA_SYS_THERMO_LIB: Option<&str> = option_env!("CEA_SYS_THERMO_LIB");
pub const CEA_SYS_TRANS_LIB: Option<&str> = option_env!("CEA_SYS_TRANS_LIB");

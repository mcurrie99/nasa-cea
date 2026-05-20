mod eqsolver;
mod error;
mod high_level;
mod mixture;

pub use eqsolver::{EqPartials, EqSolution, EqSolver, SolverOptions};
pub use error::{Error, Result};
pub use high_level::{
    Equilibrium, EquilibriumBuilder, EquilibriumProperties, EquilibriumResult, solve_tp,
    solve_tp_equivalence_moles,
};
pub use mixture::Mixture;

use std::ffi::CString;
use std::path::Path;
use std::sync::OnceLock;

#[derive(Debug, Copy, Clone)]
pub enum LogLevel {
    Critical,
    Error,
    Warning,
    Info,
    Debug,
    None,
}

impl From<LogLevel> for cea_sys::cea_log_level {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Critical => cea_sys::cea_log_level_CEA_LOG_CRITICAL,
            LogLevel::Error => cea_sys::cea_log_level_CEA_LOG_ERROR,
            LogLevel::Warning => cea_sys::cea_log_level_CEA_LOG_WARNING,
            LogLevel::Info => cea_sys::cea_log_level_CEA_LOG_INFO,
            LogLevel::Debug => cea_sys::cea_log_level_CEA_LOG_DEBUG,
            LogLevel::None => cea_sys::cea_log_level_CEA_LOG_NONE,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum EquilibriumType {
    Tp,
    Hp,
    Sp,
    Tv,
    Uv,
    Sv,
}

impl From<EquilibriumType> for cea_sys::cea_equilibrium_type {
    fn from(value: EquilibriumType) -> Self {
        match value {
            EquilibriumType::Tp => cea_sys::cea_equilibrium_type_CEA_TP,
            EquilibriumType::Hp => cea_sys::cea_equilibrium_type_CEA_HP,
            EquilibriumType::Sp => cea_sys::cea_equilibrium_type_CEA_SP,
            EquilibriumType::Tv => cea_sys::cea_equilibrium_type_CEA_TV,
            EquilibriumType::Uv => cea_sys::cea_equilibrium_type_CEA_UV,
            EquilibriumType::Sv => cea_sys::cea_equilibrium_type_CEA_SV,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum PropertyType {
    Temperature,
    Pressure,
    Volume,
    Density,
    M,
    Mw,
    Enthalpy,
    Energy,
    Entropy,
    GibbsEnergy,
    GammaS,
    FrozenCp,
    FrozenCv,
    EquilibriumCp,
    EquilibriumCv,
    Viscosity,
    FrozenConductivity,
    EquilibriumConductivity,
    FrozenPrandtl,
    EquilibriumPrandtl,
}

impl From<PropertyType> for cea_sys::cea_property_type {
    fn from(value: PropertyType) -> Self {
        match value {
            PropertyType::Temperature => cea_sys::cea_property_type_CEA_TEMPERATURE,
            PropertyType::Pressure => cea_sys::cea_property_type_CEA_PRESSURE,
            PropertyType::Volume => cea_sys::cea_property_type_CEA_VOLUME,
            PropertyType::Density => cea_sys::cea_property_type_CEA_DENSITY,
            PropertyType::M => cea_sys::cea_property_type_CEA_M,
            PropertyType::Mw => cea_sys::cea_property_type_CEA_MW,
            PropertyType::Enthalpy => cea_sys::cea_property_type_CEA_ENTHALPY,
            PropertyType::Energy => cea_sys::cea_property_type_CEA_ENERGY,
            PropertyType::Entropy => cea_sys::cea_property_type_CEA_ENTROPY,
            PropertyType::GibbsEnergy => cea_sys::cea_property_type_CEA_GIBBS_ENERGY,
            PropertyType::GammaS => cea_sys::cea_property_type_CEA_GAMMA_S,
            PropertyType::FrozenCp => cea_sys::cea_property_type_CEA_FROZEN_CP,
            PropertyType::FrozenCv => cea_sys::cea_property_type_CEA_FROZEN_CV,
            PropertyType::EquilibriumCp => cea_sys::cea_property_type_CEA_EQUILIBRIUM_CP,
            PropertyType::EquilibriumCv => cea_sys::cea_property_type_CEA_EQUILIBRIUM_CV,
            PropertyType::Viscosity => cea_sys::cea_property_type_CEA_VISCOSITY,
            PropertyType::FrozenConductivity => cea_sys::cea_property_type_CEA_FROZEN_CONDUCTIVITY,
            PropertyType::EquilibriumConductivity => {
                cea_sys::cea_property_type_CEA_EQUILIBRIUM_CONDUCTIVITY
            }
            PropertyType::FrozenPrandtl => cea_sys::cea_property_type_CEA_FROZEN_PRANDTL,
            PropertyType::EquilibriumPrandtl => cea_sys::cea_property_type_CEA_EQUILIBRIUM_PRANDTL,
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum InitStatus {
    Default(cea_sys::cea_err),
    WithData {
        thermo: cea_sys::cea_err,
        transport: cea_sys::cea_err,
    },
}

static INIT: OnceLock<InitStatus> = OnceLock::new();

pub fn init() -> Result<()> {
    if let (Some(thermo), Some(transport)) =
        (cea_sys::CEA_SYS_THERMO_LIB, cea_sys::CEA_SYS_TRANS_LIB)
    {
        init_from_files(thermo, transport)
    } else {
        let status = *INIT.get_or_init(|| InitStatus::Default(unsafe { cea_sys::cea_init() }));
        check_init_status(status)
    }
}

pub fn init_from_files(
    thermo_file: impl AsRef<Path>,
    transport_file: impl AsRef<Path>,
) -> Result<()> {
    let thermo_file = path_to_cstring(thermo_file, "init_from_files thermo_file")?;
    let transport_file = path_to_cstring(transport_file, "init_from_files transport_file")?;
    let status = *INIT.get_or_init(|| unsafe {
        InitStatus::WithData {
            thermo: cea_sys::cea_init_thermo(thermo_file.as_ptr()),
            transport: cea_sys::cea_init_trans(transport_file.as_ptr()),
        }
    });
    check_init_status(status)
}

pub fn set_log_level(level: LogLevel) -> Result<()> {
    error::check(
        unsafe { cea_sys::cea_set_log_level(level.into()) },
        "cea_set_log_level",
    )
}

fn check_init_status(status: InitStatus) -> Result<()> {
    match status {
        InitStatus::Default(err) => error::check(err, "cea_init"),
        InitStatus::WithData { thermo, transport } => {
            error::check(thermo, "cea_init_thermo")?;
            error::check(transport, "cea_init_trans")
        }
    }
}

fn path_to_cstring(path: impl AsRef<Path>, context: &str) -> Result<CString> {
    CString::new(path.as_ref().as_os_str().to_string_lossy().into_owned()).map_err(|_| Error::Nul {
        context: context.to_string(),
    })
}

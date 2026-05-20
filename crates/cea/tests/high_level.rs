use cea::{Equilibrium, LogLevel, solve_tp_equivalence_moles};
use std::sync::{Mutex, OnceLock};

const ATM: f64 = 1.01325;
const REACTANTS: [&str; 2] = ["H2", "Air"];
const PRODUCTS: [&str; 20] = [
    "Ar", "C", "CO", "CO2", "H", "H2", "H2O", "HNO", "HO2", "HNO2", "HNO3", "N", "NH", "NO", "N2",
    "N2O3", "O", "O2", "OH", "O3",
];
const FUEL_MOLES: [f64; 2] = [1.0, 0.0];
const OXIDANT_MOLES: [f64; 2] = [0.0, 1.0];

fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

#[test]
fn one_shot_tp_equivalence_moles_returns_common_properties() {
    let _guard = test_lock();
    cea::set_log_level(LogLevel::None).unwrap();

    let result = solve_tp_equivalence_moles(
        &REACTANTS,
        &PRODUCTS,
        3000.0,
        ATM,
        &FUEL_MOLES,
        &OXIDANT_MOLES,
        1.0,
    )
    .unwrap();

    assert!(result.converged);
    assert_eq!(result.reactant_weights.len(), REACTANTS.len());
    assert_eq!(result.product_species.len(), PRODUCTS.len());
    assert!(result.properties.enthalpy.is_finite());
    assert!(result.properties.equilibrium_cp.is_finite());
    assert!(result.product_mole_fraction("H2O").unwrap().is_finite());
}

#[test]
fn reusable_equilibrium_solves_multiple_states() {
    let _guard = test_lock();
    cea::set_log_level(LogLevel::None).unwrap();

    let mut equilibrium = Equilibrium::builder()
        .reactants(REACTANTS)
        .products(PRODUCTS)
        .build()
        .unwrap();

    let first = equilibrium
        .tp_with_chemical_equivalence_moles(3000.0, ATM, &FUEL_MOLES, &OXIDANT_MOLES, 1.0)
        .unwrap();
    let second = equilibrium
        .tp_with_chemical_equivalence_moles(2000.0, 0.1 * ATM, &FUEL_MOLES, &OXIDANT_MOLES, 1.5)
        .unwrap();

    assert!(first.converged);
    assert!(second.converged);
    assert_ne!(first.properties.temperature, second.properties.temperature);
    assert!(second.product_mass_fraction("N2").unwrap().is_finite());
}

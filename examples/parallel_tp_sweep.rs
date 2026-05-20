use nasa_cea::{Equilibrium, LogLevel, TpEquivalenceMolesCase};

const ATM: f64 = 1.01325;
const REACTANTS: [&str; 2] = ["H2", "Air"];
const PRODUCTS: [&str; 20] = [
    "Ar", "C", "CO", "CO2", "H", "H2", "H2O", "HNO", "HO2", "HNO2", "HNO3", "N", "NH", "NO", "N2",
    "N2O3", "O", "O2", "OH", "O3",
];

fn main() -> Result<(), nasa_cea::Error> {
    nasa_cea::set_log_level(LogLevel::None)?;

    let builder = Equilibrium::builder()
        .reactants(REACTANTS)
        .products(PRODUCTS);
    let cases = [
        TpEquivalenceMolesCase::new(3000.0, ATM, [1.0, 0.0], [0.0, 1.0], 1.0),
        TpEquivalenceMolesCase::new(2000.0, ATM, [1.0, 0.0], [0.0, 1.0], 1.0),
        TpEquivalenceMolesCase::new(3000.0, 0.1 * ATM, [1.0, 0.0], [0.0, 1.0], 1.5),
        TpEquivalenceMolesCase::new(2000.0, 0.1 * ATM, [1.0, 0.0], [0.0, 1.0], 1.5),
    ];

    let results = builder.solve_tp_equivalence_moles_cases_parallel_with_threads(&cases, 2)?;

    for (case, result) in cases.iter().zip(results.iter()) {
        println!(
            "T={:.0} K P={:.4} phi={:.1} converged={} H={:.6}",
            case.temperature,
            case.pressure,
            case.chemical_equivalence_ratio,
            result.converged,
            result.properties.enthalpy
        );
    }

    Ok(())
}

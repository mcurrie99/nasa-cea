use nasa_cea::{Equilibrium, LogLevel};

const ATM: f64 = 1.01325;
const REACTANTS: [&str; 2] = ["H2", "Air"];
const PRODUCTS: [&str; 20] = [
    "Ar", "C", "CO", "CO2", "H", "H2", "H2O", "HNO", "HO2", "HNO2", "HNO3", "N", "NH", "NO", "N2",
    "N2O3", "O", "O2", "OH", "O3",
];

fn main() -> Result<(), nasa_cea::Error> {
    nasa_cea::set_log_level(LogLevel::None)?;

    let mut equilibrium = Equilibrium::builder()
        .reactants(REACTANTS)
        .products(PRODUCTS)
        .build()?;

    let result = equilibrium.tp_with_chemical_equivalence_moles(
        3000.0,
        ATM,
        &[1.0, 0.0],
        &[0.0, 1.0],
        1.0,
    )?;

    println!("converged: {}", result.converged);
    println!("enthalpy: {:.6}", result.properties.enthalpy);
    println!("equilibrium cp: {:.6}", result.properties.equilibrium_cp);
    println!(
        "H2O mole fraction: {:.6}",
        result.product_mole_fraction("H2O").unwrap_or(0.0)
    );

    Ok(())
}

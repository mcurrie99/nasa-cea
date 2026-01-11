use cea::{
    EqPartials, EqSolution, EqSolver, EquilibriumType, LogLevel, Mixture, PropertyType,
    SolverOptions,
};
use std::time::Instant;

fn main() -> Result<(), cea::Error> {
    const ATM: f64 = 1.01325;

    let reactants = ["H2", "Air"];
    let fuel_moles: [f64; 2] = [1.0, 0.0];
    let oxidant_moles: [f64; 2] = [0.0, 1.0];

    let products = [
        "Ar", "C", "CO", "CO2", "H", "H2", "H2O", "HNO", "HO2", "HNO2", "HNO3", "N", "NH", "NO",
        "N2", "N2O3", "O", "O2", "OH", "O3",
    ];

    let pressures: [f64; 3] = [1.00 * ATM, 0.10 * ATM, 0.01 * ATM];
    let temperatures: [f64; 2] = [3000.0, 2000.0];
    let chem_eq_ratios: [f64; 2] = [1.0, 1.5];

    cea::set_log_level(LogLevel::Debug)?;
    cea::init()?;

    let reac = Mixture::new(&reactants)?;
    let prod = Mixture::new(&products)?;

    let mut opts = SolverOptions::default();
    opts.reactants = Some(&reac);
    let solver = EqSolver::with_options(&prod, opts)?;
    let soln = EqSolution::new(&solver)?;
    let partials = EqPartials::new(&solver)?;

    let mut fuel_weights = vec![0.0; reactants.len()];
    reac.moles_to_weights(&fuel_moles, &mut fuel_weights)?;

    let mut oxidant_weights = vec![0.0; reactants.len()];
    reac.moles_to_weights(&oxidant_moles, &mut oxidant_weights)?;

    let mut of_ratios = vec![0.0; chem_eq_ratios.len()];
    for (ir, chem_eq_ratio) in chem_eq_ratios.iter().enumerate() {
        of_ratios[ir] = reac.chem_eq_ratio_to_of_ratio(
            &oxidant_weights,
            &fuel_weights,
            *chem_eq_ratio,
        )?;
    }

    println!(
        "{:>10}  {:>10}  {:>10}  {:>10}  {:>12}  {:>12}  {:>12}  {:>12}",
        "T (K)",
        "P (Pa)",
        "Chem Equiv",
        "O/F Ratio",
        "H2 (wtfrac)",
        "Air (wtfrac)",
        "H (cal/g)",
        "Cp (cal/g-K)"
    );

    let start = Instant::now();
    let mut iterations = 0usize;

    for (ir, of_ratio) in of_ratios.iter().enumerate() {
        let mut weights = vec![0.0; reactants.len()];
        reac.of_ratio_to_weights(&oxidant_weights, &fuel_weights, *of_ratio, &mut weights)?;

        for pressure in pressures.iter() {
            for temperature in temperatures.iter() {
                solver.solve_with_partials(
                    EquilibriumType::Tp,
                    *temperature,
                    *pressure,
                    &mut weights,
                    &soln,
                    &partials,
                )?;

                let mut enthalpy = soln.get_property(PropertyType::Enthalpy)?;
                let mut heat_capacity = soln.get_property(PropertyType::EquilibriumCp)?;
                enthalpy /= 4.184;
                heat_capacity /= 4.184;

                println!(
                    "{:10.2}  {:10.2}  {:10.2}  {:10.6}  {:12.5e}  {:12.5e}  {:12.5e}  {:12.5e}",
                    temperature,
                    pressure,
                    chem_eq_ratios[ir],
                    of_ratio,
                    weights[0],
                    weights[1],
                    enthalpy,
                    heat_capacity
                );

                iterations += 1;
            }
        }
    }

    let elapsed = start.elapsed();
    println!(
        "Benchmark: {} solves in {:.3?} ({:.3} ms/solve)",
        iterations,
        elapsed,
        elapsed.as_secs_f64() * 1000.0 / iterations as f64
    );

    Ok(())
}

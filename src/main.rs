use cea::{
    EqPartials, EqSolution, EqSolver, EquilibriumType, LogLevel, Mixture, PropertyType, Result,
    SolverOptions,
};
use std::time::Instant;
use tokio::task::JoinSet;

#[derive(Debug, Clone)]
struct Row {
    ir: usize,
    ip: usize,
    it: usize,
    temperature: f64,
    pressure: f64,
    chem_eq_ratio: f64,
    of_ratio: f64,
    weight_h2: f64,
    weight_air: f64,
    enthalpy: f64,
    heat_capacity: f64,
}

fn solve_case(
    ir: usize,
    ip: usize,
    it: usize,
    temperature: f64,
    pressure: f64,
    chem_eq_ratio: f64,
    reactants: &[&str],
    products: &[&str],
    fuel_moles: &[f64],
    oxidant_moles: &[f64],
) -> Result<Row> {
    let reac = Mixture::new(reactants)?;
    let prod = Mixture::new(products)?;

    let mut opts = SolverOptions::default();
    opts.reactants = Some(&reac);
    let solver = EqSolver::with_options(&prod, opts)?;
    let soln = EqSolution::new(&solver)?;
    let partials = EqPartials::new(&solver)?;

    let mut fuel_weights = vec![0.0; reactants.len()];
    reac.moles_to_weights(fuel_moles, &mut fuel_weights)?;

    let mut oxidant_weights = vec![0.0; reactants.len()];
    reac.moles_to_weights(oxidant_moles, &mut oxidant_weights)?;

    let of_ratio = reac.chem_eq_ratio_to_of_ratio(&oxidant_weights, &fuel_weights, chem_eq_ratio)?;

    let mut weights = vec![0.0; reactants.len()];
    reac.of_ratio_to_weights(&oxidant_weights, &fuel_weights, of_ratio, &mut weights)?;

    solver.solve_with_partials(
        EquilibriumType::Tp,
        temperature,
        pressure,
        &mut weights,
        &soln,
        &partials,
    )?;

    let mut enthalpy = soln.get_property(PropertyType::Enthalpy)?;
    let mut heat_capacity = soln.get_property(PropertyType::EquilibriumCp)?;
    enthalpy /= 4.184;
    heat_capacity /= 4.184;

    Ok(Row {
        ir,
        ip,
        it,
        temperature,
        pressure,
        chem_eq_ratio,
        of_ratio,
        weight_h2: weights[0],
        weight_air: weights[1],
        enthalpy,
        heat_capacity,
    })
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
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
    let mut set = JoinSet::new();

    for (ir, chem_eq_ratio) in chem_eq_ratios.iter().enumerate() {
        for (ip, pressure) in pressures.iter().enumerate() {
            for (it, temperature) in temperatures.iter().enumerate() {
                let reactants = reactants.to_vec();
                let products = products.to_vec();
                let fuel_moles = fuel_moles.to_vec();
                let oxidant_moles = oxidant_moles.to_vec();
                let chem_eq_ratio = *chem_eq_ratio;
                let temperature = *temperature;
                let pressure = *pressure;

                set.spawn_blocking(move || {
                    solve_case(
                        ir,
                        ip,
                        it,
                        temperature,
                        pressure,
                        chem_eq_ratio,
                        &reactants,
                        &products,
                        &fuel_moles,
                        &oxidant_moles,
                    )
                });
            }
        }
    }

    let mut rows = Vec::new();
    while let Some(result) = set.join_next().await {
        let row_result = match result {
            Ok(row) => row,
            Err(err) => panic!("task join failed: {err}"),
        };
        rows.push(row_result?);
    }

    rows.sort_by(|a, b| (a.ir, a.ip, a.it).cmp(&(b.ir, b.ip, b.it)));

    for row in &rows {
        println!(
            "{:10.2}  {:10.2}  {:10.2}  {:10.6}  {:12.5e}  {:12.5e}  {:12.5e}  {:12.5e}",
            row.temperature,
            row.pressure,
            row.chem_eq_ratio,
            row.of_ratio,
            row.weight_h2,
            row.weight_air,
            row.enthalpy,
            row.heat_capacity
        );
    }

    let elapsed = start.elapsed();
    println!(
        "Benchmark: {} solves in {:.3?} ({:.3} ms/solve)",
        rows.len(),
        elapsed,
        elapsed.as_secs_f64() * 1000.0 / rows.len() as f64
    );

    Ok(())
}

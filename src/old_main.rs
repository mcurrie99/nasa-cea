use cea_sys::*;
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

fn make_c_string_array(names: &[&str]) -> (Vec<CString>, Vec<*const c_char>) {
    let c_strings: Vec<CString> = names
        .iter()
        .map(|name| CString::new(*name).expect("CString::new failed"))
        .collect();
    let ptrs: Vec<*const c_char> = c_strings.iter().map(|s| s.as_ptr()).collect();
    (c_strings, ptrs)
}

fn check(err: cea_err, context: &str) {
    if err != cea_error_code_CEA_SUCCESS {
        panic!("{context}: CEA error code {err}");
    }
}

fn main() {
    const ATM: cea_real = 1.01325;

    let reactants = ["H2", "Air"];
    let fuel_moles: [cea_real; 2] = [1.0, 0.0];
    let oxidant_moles: [cea_real; 2] = [0.0, 1.0];

    let products = [
        "Ar", "C", "CO", "CO2", "H", "H2", "H2O", "HNO", "HO2", "HNO2", "HNO3", "N", "NH", "NO",
        "N2", "N2O3", "O", "O2", "OH", "O3",
    ];

    let pressures: [cea_real; 3] = [1.00 * ATM, 0.10 * ATM, 0.01 * ATM];
    let temperatures: [cea_real; 2] = [3000.0, 2000.0];
    let chem_eq_ratios: [cea_real; 2] = [1.0, 1.5];

    let (_reactant_c, reactant_ptrs) = make_c_string_array(&reactants);
    let (_product_c, product_ptrs) = make_c_string_array(&products);

    unsafe {
        check(cea_set_log_level(cea_log_level_CEA_LOG_DEBUG), "cea_set_log_level");
        check(cea_init(), "cea_init");

        let mut reac: cea_mixture = ptr::null_mut();
        let mut prod: cea_mixture = ptr::null_mut();
        check(
            cea_mixture_create(
                &mut reac,
                reactant_ptrs.len() as cea_int,
                reactant_ptrs.as_ptr(),
            ),
            "cea_mixture_create reactants",
        );
        check(
            cea_mixture_create(
                &mut prod,
                product_ptrs.len() as cea_int,
                product_ptrs.as_ptr(),
            ),
            "cea_mixture_create products",
        );

        let mut opts: cea_solver_opts = std::mem::zeroed();
        check(cea_solver_opts_init(&mut opts), "cea_solver_opts_init");
        opts.reactants = reac;

        let mut solver: cea_eqsolver = ptr::null_mut();
        check(
            cea_eqsolver_create_with_options(&mut solver, prod, opts),
            "cea_eqsolver_create_with_options",
        );

        let mut soln: cea_eqsolution = ptr::null_mut();
        check(cea_eqsolution_create(&mut soln, solver), "cea_eqsolution_create");

        let mut partials: cea_eqpartials = ptr::null_mut();
        check(
            cea_eqpartials_create(&mut partials, solver),
            "cea_eqpartials_create",
        );

        let mut fuel_weights = vec![0.0; reactant_ptrs.len()];
        check(
            cea_mixture_moles_to_weights(
                reac,
                reactant_ptrs.len() as cea_int,
                fuel_moles.as_ptr(),
                fuel_weights.as_mut_ptr(),
            ),
            "cea_mixture_moles_to_weights fuel",
        );

        let mut oxidant_weights = vec![0.0; reactant_ptrs.len()];
        check(
            cea_mixture_moles_to_weights(
                reac,
                reactant_ptrs.len() as cea_int,
                oxidant_moles.as_ptr(),
                oxidant_weights.as_mut_ptr(),
            ),
            "cea_mixture_moles_to_weights oxidant",
        );

        let mut of_ratios = vec![0.0; chem_eq_ratios.len()];
        for (ir, chem_eq_ratio) in chem_eq_ratios.iter().enumerate() {
            check(
                cea_mixture_chem_eq_ratio_to_of_ratio(
                    reac,
                    reactant_ptrs.len() as cea_int,
                    oxidant_weights.as_ptr(),
                    fuel_weights.as_ptr(),
                    *chem_eq_ratio,
                    &mut of_ratios[ir],
                ),
                "cea_mixture_chem_eq_ratio_to_of_ratio",
            );
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

        for (ir, of_ratio) in of_ratios.iter().enumerate() {
            let mut weights = vec![0.0; reactant_ptrs.len()];
            check(
                cea_mixture_of_ratio_to_weights(
                    reac,
                    reactant_ptrs.len() as cea_int,
                    oxidant_weights.as_ptr(),
                    fuel_weights.as_ptr(),
                    *of_ratio,
                    weights.as_mut_ptr(),
                ),
                "cea_mixture_of_ratio_to_weights",
            );

            for pressure in pressures.iter() {
                for temperature in temperatures.iter() {
                    check(
                        cea_eqsolver_solve_with_partials(
                            solver,
                            cea_equilibrium_type_CEA_TP,
                            *temperature,
                            *pressure,
                            weights.as_mut_ptr(),
                            soln,
                            partials,
                        ),
                        "cea_eqsolver_solve_with_partials",
                    );

                    let mut enthalpy = 0.0;
                    let mut heat_capacity = 0.0;
                    check(
                        cea_eqsolution_get_property(
                            soln,
                            cea_property_type_CEA_ENTHALPY,
                            &mut enthalpy,
                        ),
                        "cea_eqsolution_get_property enthalpy",
                    );
                    check(
                        cea_eqsolution_get_property(
                            soln,
                            cea_property_type_CEA_EQUILIBRIUM_CP,
                            &mut heat_capacity,
                        ),
                        "cea_eqsolution_get_property cp",
                    );
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
                }
            }
        }

        check(cea_eqpartials_destroy(&mut partials), "cea_eqpartials_destroy");
        check(cea_eqsolution_destroy(&mut soln), "cea_eqsolution_destroy");
        check(cea_eqsolver_destroy(&mut solver), "cea_eqsolver_destroy");
        check(cea_mixture_destroy(&mut prod), "cea_mixture_destroy products");
        check(cea_mixture_destroy(&mut reac), "cea_mixture_destroy reactants");
    }
}
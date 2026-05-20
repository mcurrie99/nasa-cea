use crate::{
    init, EqPartials, EqSolution, EqSolver, EquilibriumType, Error, Mixture, PropertyType, Result,
    SolverOptions,
};
use std::thread;

#[derive(Debug, Clone)]
pub struct EquilibriumBuilder {
    reactants: Vec<String>,
    products: ProductSelection,
    trace: f64,
    ions: bool,
    transport: bool,
    insert: Vec<String>,
}

impl Default for EquilibriumBuilder {
    fn default() -> Self {
        Self {
            reactants: Vec::new(),
            products: ProductSelection::FromReactants { omit: Vec::new() },
            trace: 0.0,
            ions: false,
            transport: false,
            insert: Vec::new(),
        }
    }
}

impl EquilibriumBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reactants<I, S>(mut self, species: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.reactants = collect_strings(species);
        self
    }

    pub fn products<I, S>(mut self, species: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.products = ProductSelection::Explicit(collect_strings(species));
        self
    }

    pub fn products_from_reactants<I, S>(mut self, omit: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.products = ProductSelection::FromReactants {
            omit: collect_strings(omit),
        };
        self
    }

    pub fn trace(mut self, trace: f64) -> Self {
        self.trace = trace;
        self
    }

    pub fn ions(mut self, ions: bool) -> Self {
        self.ions = ions;
        self
    }

    pub fn transport(mut self, transport: bool) -> Self {
        self.transport = transport;
        self
    }

    pub fn insert<I, S>(mut self, species: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.insert = collect_strings(species);
        self
    }

    pub fn build(self) -> Result<Equilibrium> {
        init()?;
        if self.reactants.is_empty() {
            return Err(invalid_input(
                "EquilibriumBuilder::build",
                "reactants are required",
            ));
        }

        let reactant_refs = borrowed_strings(&self.reactants);
        let reactants = if self.ions {
            Mixture::new_with_ions(&reactant_refs)?
        } else {
            Mixture::new(&reactant_refs)?
        };

        let products = match &self.products {
            ProductSelection::Explicit(species) => {
                if species.is_empty() {
                    return Err(invalid_input(
                        "EquilibriumBuilder::build",
                        "products cannot be empty",
                    ));
                }
                let product_refs = borrowed_strings(species);
                if self.ions {
                    Mixture::new_with_ions(&product_refs)?
                } else {
                    Mixture::new(&product_refs)?
                }
            }
            ProductSelection::FromReactants { omit } => {
                let omit_refs = borrowed_strings(omit);
                if self.ions {
                    Mixture::from_reactants_with_ions(&reactant_refs, &omit_refs)?
                } else {
                    Mixture::from_reactants(&reactant_refs, &omit_refs)?
                }
            }
        };

        let insert_refs = borrowed_strings(&self.insert);
        let options = SolverOptions {
            trace: self.trace,
            ions: self.ions,
            transport: self.transport,
            reactants: Some(&reactants),
            insert: insert_refs,
        };

        let solver = EqSolver::with_options(&products, options)?;
        let solution = EqSolution::new(&solver)?;
        let partials = EqPartials::new(&solver)?;
        let product_species = products.species_names()?;

        Ok(Equilibrium {
            partials,
            solution,
            solver,
            _products: products,
            reactants,
            reactant_species: self.reactants,
            product_species,
            transport: self.transport,
        })
    }

    pub fn solve_tp_cases_parallel(&self, cases: &[TpCase]) -> Result<Vec<EquilibriumResult>> {
        self.solve_tp_cases_parallel_with_threads(cases, default_thread_count(cases.len()))
    }

    pub fn solve_tp_cases_parallel_with_threads(
        &self,
        cases: &[TpCase],
        threads: usize,
    ) -> Result<Vec<EquilibriumResult>> {
        run_parallel_cases(self, cases, threads, |equilibrium, case| {
            equilibrium.tp(case.temperature, case.pressure, &case.reactant_weights)
        })
    }

    pub fn solve_tp_equivalence_moles_cases_parallel(
        &self,
        cases: &[TpEquivalenceMolesCase],
    ) -> Result<Vec<EquilibriumResult>> {
        self.solve_tp_equivalence_moles_cases_parallel_with_threads(
            cases,
            default_thread_count(cases.len()),
        )
    }

    pub fn solve_tp_equivalence_moles_cases_parallel_with_threads(
        &self,
        cases: &[TpEquivalenceMolesCase],
        threads: usize,
    ) -> Result<Vec<EquilibriumResult>> {
        run_parallel_cases(self, cases, threads, |equilibrium, case| {
            equilibrium.tp_with_chemical_equivalence_moles(
                case.temperature,
                case.pressure,
                &case.fuel_moles,
                &case.oxidant_moles,
                case.chemical_equivalence_ratio,
            )
        })
    }
}

#[derive(Debug, Clone)]
enum ProductSelection {
    Explicit(Vec<String>),
    FromReactants { omit: Vec<String> },
}

pub struct Equilibrium {
    partials: EqPartials,
    solution: EqSolution,
    solver: EqSolver,
    _products: Mixture,
    reactants: Mixture,
    reactant_species: Vec<String>,
    product_species: Vec<String>,
    transport: bool,
}

#[derive(Debug, Clone)]
pub struct TpCase {
    pub temperature: f64,
    pub pressure: f64,
    pub reactant_weights: Vec<f64>,
}

impl TpCase {
    pub fn new(temperature: f64, pressure: f64, reactant_weights: impl Into<Vec<f64>>) -> Self {
        Self {
            temperature,
            pressure,
            reactant_weights: reactant_weights.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TpEquivalenceMolesCase {
    pub temperature: f64,
    pub pressure: f64,
    pub fuel_moles: Vec<f64>,
    pub oxidant_moles: Vec<f64>,
    pub chemical_equivalence_ratio: f64,
}

impl TpEquivalenceMolesCase {
    pub fn new(
        temperature: f64,
        pressure: f64,
        fuel_moles: impl Into<Vec<f64>>,
        oxidant_moles: impl Into<Vec<f64>>,
        chemical_equivalence_ratio: f64,
    ) -> Self {
        Self {
            temperature,
            pressure,
            fuel_moles: fuel_moles.into(),
            oxidant_moles: oxidant_moles.into(),
            chemical_equivalence_ratio,
        }
    }
}

impl Equilibrium {
    pub fn builder() -> EquilibriumBuilder {
        EquilibriumBuilder::new()
    }

    pub fn reactant_species(&self) -> &[String] {
        &self.reactant_species
    }

    pub fn product_species(&self) -> &[String] {
        &self.product_species
    }

    pub fn reactant_weights_from_moles(&self, moles: &[f64]) -> Result<Vec<f64>> {
        self.check_reactant_len(moles, "reactant_weights_from_moles")?;
        self.reactants.moles_to_weights_vec(moles)
    }

    pub fn reactant_moles_from_weights(&self, weights: &[f64]) -> Result<Vec<f64>> {
        self.check_reactant_len(weights, "reactant_moles_from_weights")?;
        self.reactants.weights_to_moles_vec(weights)
    }

    pub fn of_ratio_from_chemical_equivalence(
        &self,
        fuel_weights: &[f64],
        oxidant_weights: &[f64],
        chemical_equivalence_ratio: f64,
    ) -> Result<f64> {
        self.check_reactant_len(fuel_weights, "of_ratio_from_chemical_equivalence fuel")?;
        self.check_reactant_len(
            oxidant_weights,
            "of_ratio_from_chemical_equivalence oxidant",
        )?;
        self.reactants.chem_eq_ratio_to_of_ratio(
            oxidant_weights,
            fuel_weights,
            chemical_equivalence_ratio,
        )
    }

    pub fn of_ratio_from_weight_equivalence(
        &self,
        fuel_weights: &[f64],
        oxidant_weights: &[f64],
        weight_equivalence_ratio: f64,
    ) -> Result<f64> {
        self.check_reactant_len(fuel_weights, "of_ratio_from_weight_equivalence fuel")?;
        self.check_reactant_len(oxidant_weights, "of_ratio_from_weight_equivalence oxidant")?;
        self.reactants.weight_eq_ratio_to_of_ratio(
            oxidant_weights,
            fuel_weights,
            weight_equivalence_ratio,
        )
    }

    pub fn reactant_weights_from_of_ratio(
        &self,
        fuel_weights: &[f64],
        oxidant_weights: &[f64],
        of_ratio: f64,
    ) -> Result<Vec<f64>> {
        self.check_reactant_len(fuel_weights, "reactant_weights_from_of_ratio fuel")?;
        self.check_reactant_len(oxidant_weights, "reactant_weights_from_of_ratio oxidant")?;
        self.reactants
            .weights_from_of_ratio(oxidant_weights, fuel_weights, of_ratio)
    }

    pub fn reactant_weights_from_chemical_equivalence(
        &self,
        fuel_weights: &[f64],
        oxidant_weights: &[f64],
        chemical_equivalence_ratio: f64,
    ) -> Result<Vec<f64>> {
        let of_ratio = self.of_ratio_from_chemical_equivalence(
            fuel_weights,
            oxidant_weights,
            chemical_equivalence_ratio,
        )?;
        self.reactant_weights_from_of_ratio(fuel_weights, oxidant_weights, of_ratio)
    }

    pub fn solve(
        &mut self,
        eq_type: EquilibriumType,
        state1: f64,
        state2: f64,
        reactant_weights: &[f64],
    ) -> Result<EquilibriumResult> {
        self.check_reactant_len(reactant_weights, "Equilibrium::solve reactant_weights")?;
        let mut weights = reactant_weights.to_vec();
        self.solver.solve_with_partials(
            eq_type,
            state1,
            state2,
            &mut weights,
            &self.solution,
            &self.partials,
        )?;
        self.snapshot(weights)
    }

    pub fn tp(
        &mut self,
        temperature: f64,
        pressure: f64,
        reactant_weights: &[f64],
    ) -> Result<EquilibriumResult> {
        self.solve(EquilibriumType::Tp, temperature, pressure, reactant_weights)
    }

    pub fn hp(
        &mut self,
        enthalpy: f64,
        pressure: f64,
        reactant_weights: &[f64],
    ) -> Result<EquilibriumResult> {
        self.solve(EquilibriumType::Hp, enthalpy, pressure, reactant_weights)
    }

    pub fn tv(
        &mut self,
        temperature: f64,
        volume: f64,
        reactant_weights: &[f64],
    ) -> Result<EquilibriumResult> {
        self.solve(EquilibriumType::Tv, temperature, volume, reactant_weights)
    }

    pub fn tp_with_of_ratio(
        &mut self,
        temperature: f64,
        pressure: f64,
        fuel_weights: &[f64],
        oxidant_weights: &[f64],
        of_ratio: f64,
    ) -> Result<EquilibriumResult> {
        let weights =
            self.reactant_weights_from_of_ratio(fuel_weights, oxidant_weights, of_ratio)?;
        self.tp(temperature, pressure, &weights)
    }

    pub fn tp_with_chemical_equivalence_weights(
        &mut self,
        temperature: f64,
        pressure: f64,
        fuel_weights: &[f64],
        oxidant_weights: &[f64],
        chemical_equivalence_ratio: f64,
    ) -> Result<EquilibriumResult> {
        let weights = self.reactant_weights_from_chemical_equivalence(
            fuel_weights,
            oxidant_weights,
            chemical_equivalence_ratio,
        )?;
        self.tp(temperature, pressure, &weights)
    }

    pub fn tp_with_chemical_equivalence_moles(
        &mut self,
        temperature: f64,
        pressure: f64,
        fuel_moles: &[f64],
        oxidant_moles: &[f64],
        chemical_equivalence_ratio: f64,
    ) -> Result<EquilibriumResult> {
        let fuel_weights = self.reactant_weights_from_moles(fuel_moles)?;
        let oxidant_weights = self.reactant_weights_from_moles(oxidant_moles)?;
        self.tp_with_chemical_equivalence_weights(
            temperature,
            pressure,
            &fuel_weights,
            &oxidant_weights,
            chemical_equivalence_ratio,
        )
    }

    fn snapshot(&self, reactant_weights: Vec<f64>) -> Result<EquilibriumResult> {
        let product_count = self.product_species.len();
        let product_mass_fractions = self.solution.species_mass_fractions(product_count)?;
        let product_mole_fractions = self.solution.species_mole_fractions(product_count)?;
        let properties = EquilibriumProperties::from_solution(&self.solution, self.transport)?;
        Ok(EquilibriumResult {
            converged: self.solution.converged()?,
            total_moles: self.solution.moles()?,
            reactant_weights,
            product_species: self.product_species.clone(),
            product_mass_fractions,
            product_mole_fractions,
            properties,
        })
    }

    fn check_reactant_len(&self, values: &[f64], context: &str) -> Result<()> {
        if values.len() == self.reactant_species.len() {
            Ok(())
        } else {
            Err(Error::InvalidLength {
                context: context.to_string(),
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct EquilibriumResult {
    pub converged: bool,
    pub total_moles: f64,
    pub reactant_weights: Vec<f64>,
    pub product_species: Vec<String>,
    pub product_mass_fractions: Vec<f64>,
    pub product_mole_fractions: Vec<f64>,
    pub properties: EquilibriumProperties,
}

impl EquilibriumResult {
    pub fn property(&self, property: PropertyType) -> Option<f64> {
        self.properties.get(property)
    }

    pub fn product_mass_fraction(&self, species: &str) -> Option<f64> {
        self.product_fraction(species, &self.product_mass_fractions)
    }

    pub fn product_mole_fraction(&self, species: &str) -> Option<f64> {
        self.product_fraction(species, &self.product_mole_fractions)
    }

    fn product_fraction(&self, species: &str, values: &[f64]) -> Option<f64> {
        self.product_species
            .iter()
            .position(|name| name == species)
            .and_then(|index| values.get(index).copied())
    }
}

#[derive(Debug, Clone)]
pub struct EquilibriumProperties {
    pub temperature: f64,
    pub pressure: f64,
    pub volume: f64,
    pub density: f64,
    pub m: f64,
    pub molecular_weight: f64,
    pub enthalpy: f64,
    pub internal_energy: f64,
    pub entropy: f64,
    pub gibbs_energy: f64,
    pub gamma_s: f64,
    pub frozen_cp: f64,
    pub frozen_cv: f64,
    pub equilibrium_cp: f64,
    pub equilibrium_cv: f64,
    pub viscosity: Option<f64>,
    pub frozen_conductivity: Option<f64>,
    pub equilibrium_conductivity: Option<f64>,
    pub frozen_prandtl: Option<f64>,
    pub equilibrium_prandtl: Option<f64>,
}

impl EquilibriumProperties {
    fn from_solution(solution: &EqSolution, transport: bool) -> Result<Self> {
        Ok(Self {
            temperature: solution.get_property(PropertyType::Temperature)?,
            pressure: solution.get_property(PropertyType::Pressure)?,
            volume: solution.get_property(PropertyType::Volume)?,
            density: solution.get_property(PropertyType::Density)?,
            m: solution.get_property(PropertyType::M)?,
            molecular_weight: solution.get_property(PropertyType::Mw)?,
            enthalpy: solution.get_property(PropertyType::Enthalpy)?,
            internal_energy: solution.get_property(PropertyType::Energy)?,
            entropy: solution.get_property(PropertyType::Entropy)?,
            gibbs_energy: solution.get_property(PropertyType::GibbsEnergy)?,
            gamma_s: solution.get_property(PropertyType::GammaS)?,
            frozen_cp: solution.get_property(PropertyType::FrozenCp)?,
            frozen_cv: solution.get_property(PropertyType::FrozenCv)?,
            equilibrium_cp: solution.get_property(PropertyType::EquilibriumCp)?,
            equilibrium_cv: solution.get_property(PropertyType::EquilibriumCv)?,
            viscosity: optional_transport_property(solution, transport, PropertyType::Viscosity)?,
            frozen_conductivity: optional_transport_property(
                solution,
                transport,
                PropertyType::FrozenConductivity,
            )?,
            equilibrium_conductivity: optional_transport_property(
                solution,
                transport,
                PropertyType::EquilibriumConductivity,
            )?,
            frozen_prandtl: optional_transport_property(
                solution,
                transport,
                PropertyType::FrozenPrandtl,
            )?,
            equilibrium_prandtl: optional_transport_property(
                solution,
                transport,
                PropertyType::EquilibriumPrandtl,
            )?,
        })
    }

    pub fn get(&self, property: PropertyType) -> Option<f64> {
        match property {
            PropertyType::Temperature => Some(self.temperature),
            PropertyType::Pressure => Some(self.pressure),
            PropertyType::Volume => Some(self.volume),
            PropertyType::Density => Some(self.density),
            PropertyType::M => Some(self.m),
            PropertyType::Mw => Some(self.molecular_weight),
            PropertyType::Enthalpy => Some(self.enthalpy),
            PropertyType::Energy => Some(self.internal_energy),
            PropertyType::Entropy => Some(self.entropy),
            PropertyType::GibbsEnergy => Some(self.gibbs_energy),
            PropertyType::GammaS => Some(self.gamma_s),
            PropertyType::FrozenCp => Some(self.frozen_cp),
            PropertyType::FrozenCv => Some(self.frozen_cv),
            PropertyType::EquilibriumCp => Some(self.equilibrium_cp),
            PropertyType::EquilibriumCv => Some(self.equilibrium_cv),
            PropertyType::Viscosity => self.viscosity,
            PropertyType::FrozenConductivity => self.frozen_conductivity,
            PropertyType::EquilibriumConductivity => self.equilibrium_conductivity,
            PropertyType::FrozenPrandtl => self.frozen_prandtl,
            PropertyType::EquilibriumPrandtl => self.equilibrium_prandtl,
        }
    }
}

pub fn solve_tp(
    reactants: &[&str],
    products: &[&str],
    temperature: f64,
    pressure: f64,
    reactant_weights: &[f64],
) -> Result<EquilibriumResult> {
    let mut equilibrium = Equilibrium::builder()
        .reactants(reactants.iter().copied())
        .products(products.iter().copied())
        .build()?;
    equilibrium.tp(temperature, pressure, reactant_weights)
}

pub fn solve_tp_equivalence_moles(
    reactants: &[&str],
    products: &[&str],
    temperature: f64,
    pressure: f64,
    fuel_moles: &[f64],
    oxidant_moles: &[f64],
    chemical_equivalence_ratio: f64,
) -> Result<EquilibriumResult> {
    let mut equilibrium = Equilibrium::builder()
        .reactants(reactants.iter().copied())
        .products(products.iter().copied())
        .build()?;
    equilibrium.tp_with_chemical_equivalence_moles(
        temperature,
        pressure,
        fuel_moles,
        oxidant_moles,
        chemical_equivalence_ratio,
    )
}

fn optional_transport_property(
    solution: &EqSolution,
    transport: bool,
    property: PropertyType,
) -> Result<Option<f64>> {
    if transport {
        solution.get_property(property).map(Some)
    } else {
        Ok(None)
    }
}

fn collect_strings<I, S>(values: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    values.into_iter().map(Into::into).collect()
}

fn borrowed_strings(values: &[String]) -> Vec<&str> {
    values.iter().map(String::as_str).collect()
}

fn invalid_input(context: &str, message: &str) -> Error {
    Error::InvalidInput {
        context: context.to_string(),
        message: message.to_string(),
    }
}

fn default_thread_count(case_count: usize) -> usize {
    thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
        .min(case_count.max(1))
}

fn run_parallel_cases<C, F>(
    builder: &EquilibriumBuilder,
    cases: &[C],
    threads: usize,
    solve: F,
) -> Result<Vec<EquilibriumResult>>
where
    C: Sync,
    F: Fn(&mut Equilibrium, &C) -> Result<EquilibriumResult> + Copy + Send + Sync,
{
    if cases.is_empty() {
        return Ok(Vec::new());
    }
    if threads == 0 {
        return Err(invalid_input(
            "run_parallel_cases",
            "thread count must be greater than zero",
        ));
    }

    init()?;

    let workers = threads.min(cases.len());
    let chunk_size = cases.len().div_ceil(workers);
    let mut results = vec![None; cases.len()];

    thread::scope(|scope| {
        let mut handles = Vec::with_capacity(workers);
        for (worker_index, chunk) in cases.chunks(chunk_size).enumerate() {
            let start = worker_index * chunk_size;
            let builder = builder.clone();
            handles.push(scope.spawn(move || {
                let mut equilibrium = builder.build()?;
                let mut chunk_results = Vec::with_capacity(chunk.len());
                for (offset, case) in chunk.iter().enumerate() {
                    chunk_results.push((start + offset, solve(&mut equilibrium, case)?));
                }
                Ok::<_, Error>(chunk_results)
            }));
        }

        for handle in handles {
            let chunk_results = handle.join().map_err(|_| Error::ThreadPanic {
                context: "run_parallel_cases".to_string(),
            })??;
            for (index, result) in chunk_results {
                results[index] = Some(result);
            }
        }

        Ok::<_, Error>(())
    })?;

    results
        .into_iter()
        .enumerate()
        .map(|(index, result)| {
            result.ok_or_else(|| Error::ThreadPanic {
                context: format!("run_parallel_cases result {index}"),
            })
        })
        .collect()
}

use crate::error::{Error, Result, check};
use crate::mixture::Mixture;
use crate::{EquilibriumType, PropertyType};
use cea_sys::{cea_int, cea_real};
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

pub struct SolverOptions<'a> {
    pub trace: cea_real,
    pub ions: bool,
    pub transport: bool,
    pub reactants: Option<&'a Mixture>,
    pub insert: Vec<&'a str>,
}

impl Default for SolverOptions<'_> {
    fn default() -> Self {
        Self {
            trace: 0.0,
            ions: false,
            transport: false,
            reactants: None,
            insert: Vec::new(),
        }
    }
}

pub struct EqSolver {
    pub(crate) ptr: cea_sys::cea_eqsolver,
}

impl EqSolver {
    pub fn new(products: &Mixture) -> Result<Self> {
        let mut solver = ptr::null_mut();
        unsafe {
            check(
                cea_sys::cea_eqsolver_create(&mut solver, products.ptr),
                "cea_eqsolver_create",
            )?;
        }
        Ok(Self { ptr: solver })
    }

    pub fn with_reactants(products: &Mixture, reactants: &Mixture) -> Result<Self> {
        let mut solver = ptr::null_mut();
        unsafe {
            check(
                cea_sys::cea_eqsolver_create_with_reactants(
                    &mut solver,
                    products.ptr,
                    reactants.ptr,
                ),
                "cea_eqsolver_create_with_reactants",
            )?;
        }
        Ok(Self { ptr: solver })
    }

    pub fn with_options(products: &Mixture, options: SolverOptions<'_>) -> Result<Self> {
        let mut opts: cea_sys::cea_solver_opts = unsafe { std::mem::zeroed() };
        unsafe {
            check(
                cea_sys::cea_solver_opts_init(&mut opts),
                "cea_solver_opts_init",
            )?;
        }
        opts.trace = options.trace;
        opts.ions = options.ions;
        opts.transport = options.transport;
        if let Some(reactants) = options.reactants {
            opts.reactants = reactants.ptr;
        }

        let (_insert_storage, insert_ptrs) =
            make_c_strings(&options.insert, "SolverOptions insert")?;
        if !insert_ptrs.is_empty() {
            opts.ninsert = to_cea_int(insert_ptrs.len(), "SolverOptions insert")?;
            opts.insert = insert_ptrs.as_ptr();
        }

        let mut solver = ptr::null_mut();
        unsafe {
            check(
                cea_sys::cea_eqsolver_create_with_options(&mut solver, products.ptr, opts),
                "cea_eqsolver_create_with_options",
            )?;
        }
        Ok(Self { ptr: solver })
    }

    pub fn solve(
        &self,
        eq_type: EquilibriumType,
        state1: cea_real,
        state2: cea_real,
        amounts: &mut [cea_real],
        soln: &EqSolution,
    ) -> Result<()> {
        unsafe {
            check(
                cea_sys::cea_eqsolver_solve(
                    self.ptr,
                    eq_type.into(),
                    state1,
                    state2,
                    amounts.as_mut_ptr(),
                    soln.ptr,
                ),
                "cea_eqsolver_solve",
            )
        }
    }

    pub fn solve_with_partials(
        &self,
        eq_type: EquilibriumType,
        state1: cea_real,
        state2: cea_real,
        amounts: &mut [cea_real],
        soln: &EqSolution,
        partials: &EqPartials,
    ) -> Result<()> {
        unsafe {
            check(
                cea_sys::cea_eqsolver_solve_with_partials(
                    self.ptr,
                    eq_type.into(),
                    state1,
                    state2,
                    amounts.as_mut_ptr(),
                    soln.ptr,
                    partials.ptr,
                ),
                "cea_eqsolver_solve_with_partials",
            )
        }
    }
}

impl Drop for EqSolver {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                let _ = cea_sys::cea_eqsolver_destroy(&mut self.ptr);
            }
        }
    }
}

pub struct EqSolution {
    pub(crate) ptr: cea_sys::cea_eqsolution,
}

impl EqSolution {
    pub fn new(solver: &EqSolver) -> Result<Self> {
        let mut soln = ptr::null_mut();
        unsafe {
            check(
                cea_sys::cea_eqsolution_create(&mut soln, solver.ptr),
                "cea_eqsolution_create",
            )?;
        }
        Ok(Self { ptr: soln })
    }

    pub fn get_property(&self, property: PropertyType) -> Result<cea_real> {
        let mut value = 0.0;
        unsafe {
            check(
                cea_sys::cea_eqsolution_get_property(self.ptr, property.into(), &mut value),
                "cea_eqsolution_get_property",
            )?;
        }
        Ok(value)
    }

    pub fn species_mass_fractions(&self, len: usize) -> Result<Vec<cea_real>> {
        self.species_amounts(len, true)
    }

    pub fn species_mole_fractions(&self, len: usize) -> Result<Vec<cea_real>> {
        self.species_amounts(len, false)
    }

    pub fn moles(&self) -> Result<cea_real> {
        let mut value = 0.0;
        unsafe {
            check(
                cea_sys::cea_eqsolution_get_moles(self.ptr, &mut value),
                "cea_eqsolution_get_moles",
            )?;
        }
        Ok(value)
    }

    pub fn converged(&self) -> Result<bool> {
        let mut converged = 0;
        unsafe {
            check(
                cea_sys::cea_eqsolution_get_converged(self.ptr, &mut converged),
                "cea_eqsolution_get_converged",
            )?;
        }
        Ok(converged != 0)
    }

    fn species_amounts(&self, len: usize, mass: bool) -> Result<Vec<cea_real>> {
        let len = to_cea_int(len, "species_amounts")?;
        let mut amounts = vec![0.0; len as usize];
        unsafe {
            check(
                cea_sys::cea_eqsolution_get_species_amounts(
                    self.ptr,
                    len,
                    amounts.as_mut_ptr(),
                    mass,
                ),
                "cea_eqsolution_get_species_amounts",
            )?;
        }
        Ok(amounts)
    }
}

impl Drop for EqSolution {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                let _ = cea_sys::cea_eqsolution_destroy(&mut self.ptr);
            }
        }
    }
}

pub struct EqPartials {
    pub(crate) ptr: cea_sys::cea_eqpartials,
}

impl EqPartials {
    pub fn new(solver: &EqSolver) -> Result<Self> {
        let mut partials = ptr::null_mut();
        unsafe {
            check(
                cea_sys::cea_eqpartials_create(&mut partials, solver.ptr),
                "cea_eqpartials_create",
            )?;
        }
        Ok(Self { ptr: partials })
    }
}

impl Drop for EqPartials {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                let _ = cea_sys::cea_eqpartials_destroy(&mut self.ptr);
            }
        }
    }
}

fn make_c_strings(names: &[&str], context: &str) -> Result<(Vec<CString>, Vec<*const c_char>)> {
    let mut storage = Vec::with_capacity(names.len());
    let mut ptrs = Vec::with_capacity(names.len());
    for name in names {
        let cstr = CString::new(*name).map_err(|_| Error::Nul {
            context: context.to_string(),
        })?;
        ptrs.push(cstr.as_ptr());
        storage.push(cstr);
    }
    Ok((storage, ptrs))
}

fn to_cea_int(len: usize, context: &str) -> Result<cea_int> {
    cea_int::try_from(len).map_err(|_| Error::InvalidLength {
        context: context.to_string(),
    })
}

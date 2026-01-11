use crate::error::{check, Error, Result};
use cea_sys::{cea_int, cea_real, cea_string};
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

pub struct Mixture {
    pub(crate) ptr: cea_sys::cea_mixture,
}

impl Mixture {
    pub fn new(species: &[&str]) -> Result<Self> {
        let (_storage, ptrs) = make_c_strings(species, "Mixture::new species")?;
        let mut mix = ptr::null_mut();
        let len = to_cea_int(species.len(), "Mixture::new species")?;
        unsafe {
            check(
                cea_sys::cea_mixture_create(&mut mix, len, ptrs.as_ptr()),
                "cea_mixture_create",
            )?;
        }
        Ok(Self { ptr: mix })
    }

    pub fn new_with_ions(species: &[&str]) -> Result<Self> {
        let (_storage, ptrs) = make_c_strings(species, "Mixture::new_with_ions species")?;
        let mut mix = ptr::null_mut();
        let len = to_cea_int(species.len(), "Mixture::new_with_ions species")?;
        unsafe {
            check(
                cea_sys::cea_mixture_create_w_ions(&mut mix, len, ptrs.as_ptr()),
                "cea_mixture_create_w_ions",
            )?;
        }
        Ok(Self { ptr: mix })
    }

    pub fn from_reactants(reactants: &[&str], omit: &[&str]) -> Result<Self> {
        let (_r_storage, r_ptrs) =
            make_c_strings(reactants, "Mixture::from_reactants reactants")?;
        let (_o_storage, o_ptrs) = make_c_strings(omit, "Mixture::from_reactants omit")?;
        let mut mix = ptr::null_mut();
        let nreactants = to_cea_int(reactants.len(), "Mixture::from_reactants reactants")?;
        let nomit = to_cea_int(omit.len(), "Mixture::from_reactants omit")?;
        let omit_ptr = if omit.is_empty() {
            ptr::null()
        } else {
            o_ptrs.as_ptr()
        };
        unsafe {
            check(
                cea_sys::cea_mixture_create_from_reactants(
                    &mut mix,
                    nreactants,
                    r_ptrs.as_ptr(),
                    nomit,
                    omit_ptr as *const cea_string,
                ),
                "cea_mixture_create_from_reactants",
            )?;
        }
        Ok(Self { ptr: mix })
    }

    pub fn num_species(&self) -> Result<usize> {
        let mut n: cea_int = 0;
        unsafe {
            check(
                cea_sys::cea_mixture_get_num_species(self.ptr, &mut n),
                "cea_mixture_get_num_species",
            )?;
        }
        Ok(n as usize)
    }

    pub fn moles_to_weights(&self, moles: &[cea_real], weights: &mut [cea_real]) -> Result<()> {
        check_len_match(moles.len(), weights.len(), "moles_to_weights")?;
        let len = to_cea_int(moles.len(), "moles_to_weights")?;
        unsafe {
            check(
                cea_sys::cea_mixture_moles_to_weights(
                    self.ptr,
                    len,
                    moles.as_ptr(),
                    weights.as_mut_ptr(),
                ),
                "cea_mixture_moles_to_weights",
            )
        }
    }

    pub fn weights_to_moles(&self, weights: &[cea_real], moles: &mut [cea_real]) -> Result<()> {
        check_len_match(weights.len(), moles.len(), "weights_to_moles")?;
        let len = to_cea_int(weights.len(), "weights_to_moles")?;
        unsafe {
            check(
                cea_sys::cea_mixture_weights_to_moles(
                    self.ptr,
                    len,
                    weights.as_ptr(),
                    moles.as_mut_ptr(),
                ),
                "cea_mixture_weights_to_moles",
            )
        }
    }

    pub fn chem_eq_ratio_to_of_ratio(
        &self,
        oxidant_weights: &[cea_real],
        fuel_weights: &[cea_real],
        chem_eq_ratio: cea_real,
    ) -> Result<cea_real> {
        check_len_match(
            oxidant_weights.len(),
            fuel_weights.len(),
            "chem_eq_ratio_to_of_ratio",
        )?;
        let len = to_cea_int(oxidant_weights.len(), "chem_eq_ratio_to_of_ratio")?;
        let mut of_ratio = 0.0;
        unsafe {
            check(
                cea_sys::cea_mixture_chem_eq_ratio_to_of_ratio(
                    self.ptr,
                    len,
                    oxidant_weights.as_ptr(),
                    fuel_weights.as_ptr(),
                    chem_eq_ratio,
                    &mut of_ratio,
                ),
                "cea_mixture_chem_eq_ratio_to_of_ratio",
            )?;
        }
        Ok(of_ratio)
    }

    pub fn of_ratio_to_weights(
        &self,
        oxidant_weights: &[cea_real],
        fuel_weights: &[cea_real],
        of_ratio: cea_real,
        reactant_weights: &mut [cea_real],
    ) -> Result<()> {
        check_len_match(
            oxidant_weights.len(),
            fuel_weights.len(),
            "of_ratio_to_weights",
        )?;
        check_len_match(
            oxidant_weights.len(),
            reactant_weights.len(),
            "of_ratio_to_weights",
        )?;
        let len = to_cea_int(oxidant_weights.len(), "of_ratio_to_weights")?;
        unsafe {
            check(
                cea_sys::cea_mixture_of_ratio_to_weights(
                    self.ptr,
                    len,
                    oxidant_weights.as_ptr(),
                    fuel_weights.as_ptr(),
                    of_ratio,
                    reactant_weights.as_mut_ptr(),
                ),
                "cea_mixture_of_ratio_to_weights",
            )
        }
    }
}

impl Drop for Mixture {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                let _ = cea_sys::cea_mixture_destroy(&mut self.ptr);
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

fn check_len_match(a: usize, b: usize, context: &str) -> Result<()> {
    if a == b {
        Ok(())
    } else {
        Err(Error::InvalidLength {
            context: context.to_string(),
        })
    }
}

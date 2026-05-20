use crate::PropertyType;
use crate::error::{Error, Result, check};
use cea_sys::{cea_int, cea_real, cea_string};
use std::ffi::CStr;
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
        let (_r_storage, r_ptrs) = make_c_strings(reactants, "Mixture::from_reactants reactants")?;
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

    pub fn from_reactants_with_ions(reactants: &[&str], omit: &[&str]) -> Result<Self> {
        let (_r_storage, r_ptrs) =
            make_c_strings(reactants, "Mixture::from_reactants_with_ions reactants")?;
        let (_o_storage, o_ptrs) = make_c_strings(omit, "Mixture::from_reactants_with_ions omit")?;
        let mut mix = ptr::null_mut();
        let nreactants = to_cea_int(
            reactants.len(),
            "Mixture::from_reactants_with_ions reactants",
        )?;
        let nomit = to_cea_int(omit.len(), "Mixture::from_reactants_with_ions omit")?;
        let omit_ptr = if omit.is_empty() {
            ptr::null()
        } else {
            o_ptrs.as_ptr()
        };
        unsafe {
            check(
                cea_sys::cea_mixture_create_from_reactants_w_ions(
                    &mut mix,
                    nreactants,
                    r_ptrs.as_ptr(),
                    nomit,
                    omit_ptr as *const cea_string,
                ),
                "cea_mixture_create_from_reactants_w_ions",
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

    pub fn species_name(&self, index: usize) -> Result<String> {
        let stride = species_name_stride()?;
        let mut buffer = vec![0 as c_char; stride];
        let index = to_cea_int(index, "species_name")?;
        let stride = to_cea_int(stride, "species_name")?;
        unsafe {
            check(
                cea_sys::cea_mixture_get_species_name_buf(
                    &self.ptr,
                    index,
                    buffer.as_mut_ptr(),
                    stride,
                ),
                "cea_mixture_get_species_name_buf",
            )?;
            Ok(CStr::from_ptr(buffer.as_ptr())
                .to_string_lossy()
                .into_owned())
        }
    }

    pub fn species_names(&self) -> Result<Vec<String>> {
        let count = self.num_species()?;
        if count == 0 {
            return Ok(Vec::new());
        }

        let stride = species_name_stride()?;
        let mut buffer = vec![0 as c_char; count * stride];
        let count_i = to_cea_int(count, "species_names")?;
        let stride_i = to_cea_int(stride, "species_names")?;
        unsafe {
            check(
                cea_sys::cea_mixture_get_species_names_buf(
                    &self.ptr,
                    count_i,
                    buffer.as_mut_ptr(),
                    stride_i,
                ),
                "cea_mixture_get_species_names_buf",
            )?;
        }

        let mut names = Vec::with_capacity(count);
        for i in 0..count {
            let offset = i * stride;
            let name = unsafe { CStr::from_ptr(buffer.as_ptr().add(offset)) }
                .to_string_lossy()
                .into_owned();
            names.push(name);
        }
        Ok(names)
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

    pub fn moles_to_weights_vec(&self, moles: &[cea_real]) -> Result<Vec<cea_real>> {
        let mut weights = vec![0.0; moles.len()];
        self.moles_to_weights(moles, &mut weights)?;
        Ok(weights)
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

    pub fn weights_to_moles_vec(&self, weights: &[cea_real]) -> Result<Vec<cea_real>> {
        let mut moles = vec![0.0; weights.len()];
        self.weights_to_moles(weights, &mut moles)?;
        Ok(moles)
    }

    pub fn per_mole_to_per_weight(
        &self,
        per_mole: &[cea_real],
        per_weight: &mut [cea_real],
    ) -> Result<()> {
        check_len_match(per_mole.len(), per_weight.len(), "per_mole_to_per_weight")?;
        let len = to_cea_int(per_mole.len(), "per_mole_to_per_weight")?;
        unsafe {
            check(
                cea_sys::cea_mixture_per_mole_to_per_weight(
                    self.ptr,
                    len,
                    per_mole.as_ptr(),
                    per_weight.as_mut_ptr(),
                ),
                "cea_mixture_per_mole_to_per_weight",
            )
        }
    }

    pub fn per_mole_to_per_weight_vec(&self, per_mole: &[cea_real]) -> Result<Vec<cea_real>> {
        let mut per_weight = vec![0.0; per_mole.len()];
        self.per_mole_to_per_weight(per_mole, &mut per_weight)?;
        Ok(per_weight)
    }

    pub fn per_weight_to_per_mole(
        &self,
        per_weight: &[cea_real],
        per_mole: &mut [cea_real],
    ) -> Result<()> {
        check_len_match(per_weight.len(), per_mole.len(), "per_weight_to_per_mole")?;
        let len = to_cea_int(per_weight.len(), "per_weight_to_per_mole")?;
        unsafe {
            check(
                cea_sys::cea_mixture_per_weight_to_per_mole(
                    self.ptr,
                    len,
                    per_weight.as_ptr(),
                    per_mole.as_mut_ptr(),
                ),
                "cea_mixture_per_weight_to_per_mole",
            )
        }
    }

    pub fn per_weight_to_per_mole_vec(&self, per_weight: &[cea_real]) -> Result<Vec<cea_real>> {
        let mut per_mole = vec![0.0; per_weight.len()];
        self.per_weight_to_per_mole(per_weight, &mut per_mole)?;
        Ok(per_mole)
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

    pub fn weight_eq_ratio_to_of_ratio(
        &self,
        oxidant_weights: &[cea_real],
        fuel_weights: &[cea_real],
        weight_eq_ratio: cea_real,
    ) -> Result<cea_real> {
        check_len_match(
            oxidant_weights.len(),
            fuel_weights.len(),
            "weight_eq_ratio_to_of_ratio",
        )?;
        let len = to_cea_int(oxidant_weights.len(), "weight_eq_ratio_to_of_ratio")?;
        let mut of_ratio = 0.0;
        unsafe {
            check(
                cea_sys::cea_mixture_weight_eq_ratio_to_of_ratio(
                    self.ptr,
                    len,
                    oxidant_weights.as_ptr(),
                    fuel_weights.as_ptr(),
                    weight_eq_ratio,
                    &mut of_ratio,
                ),
                "cea_mixture_weight_eq_ratio_to_of_ratio",
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

    pub fn weights_from_of_ratio(
        &self,
        oxidant_weights: &[cea_real],
        fuel_weights: &[cea_real],
        of_ratio: cea_real,
    ) -> Result<Vec<cea_real>> {
        let mut weights = vec![0.0; oxidant_weights.len()];
        self.of_ratio_to_weights(oxidant_weights, fuel_weights, of_ratio, &mut weights)?;
        Ok(weights)
    }

    pub fn calc_property(
        &self,
        property: PropertyType,
        weights: &[cea_real],
        temperature: cea_real,
    ) -> Result<cea_real> {
        let mut value = 0.0;
        let len = to_cea_int(weights.len(), "calc_property")?;
        unsafe {
            check(
                cea_sys::cea_mixture_calc_property(
                    self.ptr,
                    property.into(),
                    len,
                    weights.as_ptr(),
                    temperature,
                    &mut value,
                ),
                "cea_mixture_calc_property",
            )?;
        }
        Ok(value)
    }

    pub fn calc_property_multitemp(
        &self,
        property: PropertyType,
        weights: &[cea_real],
        temperatures: &[cea_real],
    ) -> Result<cea_real> {
        check_len_match(weights.len(), temperatures.len(), "calc_property_multitemp")?;
        let mut value = 0.0;
        let len = to_cea_int(weights.len(), "calc_property_multitemp weights")?;
        let temp_len = to_cea_int(temperatures.len(), "calc_property_multitemp temperatures")?;
        unsafe {
            check(
                cea_sys::cea_mixture_calc_property_multitemp(
                    self.ptr,
                    property.into(),
                    len,
                    weights.as_ptr(),
                    temp_len,
                    temperatures.as_ptr(),
                    &mut value,
                ),
                "cea_mixture_calc_property_multitemp",
            )?;
        }
        Ok(value)
    }

    pub fn calc_property_tp(
        &self,
        property: PropertyType,
        weights: &[cea_real],
        temperature: cea_real,
        pressure: cea_real,
    ) -> Result<cea_real> {
        let mut value = 0.0;
        let len = to_cea_int(weights.len(), "calc_property_tp")?;
        unsafe {
            check(
                cea_sys::cea_mixture_calc_property_tp(
                    self.ptr,
                    property.into(),
                    len,
                    weights.as_ptr(),
                    temperature,
                    pressure,
                    &mut value,
                ),
                "cea_mixture_calc_property_tp",
            )?;
        }
        Ok(value)
    }

    pub fn calc_property_tp_multitemp(
        &self,
        property: PropertyType,
        weights: &[cea_real],
        temperatures: &[cea_real],
        pressure: cea_real,
    ) -> Result<cea_real> {
        check_len_match(
            weights.len(),
            temperatures.len(),
            "calc_property_tp_multitemp",
        )?;
        let mut value = 0.0;
        let len = to_cea_int(weights.len(), "calc_property_tp_multitemp weights")?;
        let temp_len = to_cea_int(
            temperatures.len(),
            "calc_property_tp_multitemp temperatures",
        )?;
        unsafe {
            check(
                cea_sys::cea_mixture_calc_property_tp_multitemp(
                    self.ptr,
                    property.into(),
                    len,
                    weights.as_ptr(),
                    temp_len,
                    temperatures.as_ptr(),
                    pressure,
                    &mut value,
                ),
                "cea_mixture_calc_property_tp_multitemp",
            )?;
        }
        Ok(value)
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

fn species_name_stride() -> Result<usize> {
    let mut name_len: cea_int = 0;
    unsafe {
        check(
            cea_sys::cea_species_name_len(&mut name_len),
            "cea_species_name_len",
        )?;
    }
    usize::try_from(name_len + 1).map_err(|_| Error::InvalidLength {
        context: "species_name_len".to_string(),
    })
}

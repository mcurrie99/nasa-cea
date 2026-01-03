# Import the necessary Cython modules
from libc.stdlib cimport malloc, free
from cpython.bytes cimport PyBytes_AsString
import cython
import ctypes
import array
import warnings

# Import numpy
cimport numpy as np
import numpy as np

np.import_array()

# Import the CEA python module
include "../python/CEA.pyx"

# Import the definitions
from cea_def cimport *

# Equilibrium solve interface for CEA in Matlab
def eq_solve(cea_equilibrium_type eq_type,
             list reactants,
             T=None, H=None, S=None, U=None,
             P=None, V=None,
             list T_reac=[], list fuel_amounts=[], list oxid_amounts=[], moles=False,
             of_ratio=None, phi=None, r_eq=None, pct_fuel=None,
             list only=[], list omit=[], list insert=[],
             trace=None, transport=False, ions=False):

    # Required optional args:
    # - one of P, V
    # - one of T, H, S, U, or T_reac

    # Required kwargs:
    # - one of weights, moles, of_ratio, weight_eq_ratio, chem_eq_ratio
    # - if of_ratio, weight_eq_ratio, or chem_eq_ratio: one pair of:
    #   - (fuel_weights, oxid_weights), or
    #   - (fuel_moles, oxid_moles)

    cdef cea_err ierr

    # Create the reactants mixture from input reactants (following main.f90 pattern)
    reactants_mix = Mixture(reactants, ions=ions)

    # Get the products Mixture object (following main.f90 pattern)
    if only:
        # If only specified, use those species
        products_mix = Mixture(only)
    else:
        products_mix = Mixture(reactants, products_from_reactants=True, omit=omit)

    # Check thermodynamic state values
    state1 = 0.0
    needs_state1 = not (T or H or S or U)
    if (T is not None):
        state1 = T
    elif (H is not None):
        state1 = H
    elif (S is not None):
        state1 = S
    elif (U is not None):
        state1 = U

    if (P is not None):
        state2 = P
    elif (V is not None):
        state2 = V
    else:
        raise TypeError("Pressure or volume state not defined")

    # Check that the reactant weights are defined somehow
    weights = np.zeros(len(reactants))
    if fuel_amounts and oxid_amounts:
        # Convert fuel/oxidant amounts to total weights
        if moles:
            fuel_weights = reactants_mix.moles_to_weights(fuel_amounts)
            oxid_weights = reactants_mix.moles_to_weights(oxid_amounts)
        else:
            fuel_weights = fuel_amounts
            oxid_weights = oxid_amounts

        if of_ratio is not None:
            weights = reactants_mix.of_ratio_to_weights(oxid_weights, fuel_weights, of_ratio)
        elif phi is not None:
            weights = reactants_mix.weight_eq_ratio_to_weights(oxid_weights, fuel_weights, phi)
        elif r_eq is not None:
            of_ratio_calc = reactants_mix.chem_eq_ratio_to_of_ratio(oxid_weights, fuel_weights, r_eq)
            weights = reactants_mix.of_ratio_to_weights(oxid_weights, fuel_weights, of_ratio_calc)
        elif pct_fuel is not None:
            of_ratio_calc = (100.0 - pct_fuel) / pct_fuel
            weights = reactants_mix.of_ratio_to_weights(oxid_weights, fuel_weights, of_ratio_calc)
        else:
            # If no ratio specified, add the oxidant and fuel weights
            weights = [f + o for f, o in zip(fuel_weights, oxid_weights)]
    else:
        raise TypeError("Fuel and oxidizer amounts not defined")

    # Compute the fixed thermodynamic state if needed
    if needs_state1:
        if T_reac:
            if len(T_reac) != len(reactants):
                raise ValueError("T_reac must have the same length as reactants")

            if eq_type == TP or eq_type == TV:
                state1 = T_reac[0]
                warnings.warn("Problem temperature not defined; using first reactant temperature")
            elif eq_type == HP:
                state1 = reactants_mix.calc_property(ENTHALPY, weights, T_reac)
            elif eq_type == SP or eq_type == SV:
                state1 = reactants_mix.calc_property(ENTROPY, weights, T_reac)
            elif eq_type == UV:
                state1 = reactants_mix.calc_property(ENERGY, weights, T_reac)
        else:
            raise TypeError("Reactant temperature not defined")

    # Initialize the solver (following main.f90 pattern)
    if trace is not None:
        if transport:
            solver = EqSolver(products_mix, reactants_mix, trace, ions=ions, transport=True, insert=insert)
        else:
            solver = EqSolver(products_mix, reactants_mix, trace, ions=ions, insert=insert)
    else:
        if transport:
            solver = EqSolver(products_mix, reactants_mix, ions=ions, transport=True, insert=insert)
        else:
            solver = EqSolver(products_mix, reactants_mix, ions=ions, insert=insert)

    # Initialize the solution
    soln = EqSolution(solver)

    # Call the solver
    solver.solve(soln, eq_type, state1, state2, weights)

    return soln
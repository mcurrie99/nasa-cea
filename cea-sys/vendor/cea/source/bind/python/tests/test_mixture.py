import numpy as np
import pytest


def test_species_names_round_trip(cea_module):
    mix = cea_module.Mixture(["H2", "O2", "Ar"])
    names = mix.species_names
    assert len(names) == 3
    for name in ("H2", "O2", "Ar"):
        assert name in names


def test_moles_weights_round_trip(cea_module):
    mix = cea_module.Mixture(["H2", "O2", "Ar"])
    moles = np.array([2.0, 1.0, 0.5], dtype=np.float64)
    weights = mix.moles_to_weights(moles)
    roundtrip = mix.weights_to_moles(weights)
    moles_norm = moles / np.sum(moles)
    roundtrip_norm = roundtrip / np.sum(roundtrip)
    assert np.allclose(roundtrip_norm, moles_norm, rtol=1e-12, atol=1e-12)


def test_calc_property_requires_pressure(cea_module):
    mix = cea_module.Mixture(["H2", "O2"])
    weights = np.array([0.4, 0.6], dtype=np.float64)
    with pytest.raises(ValueError):
        mix.calc_property(cea_module.ENTROPY, weights, temperature=3000.0)


def test_calc_property_rejects_unknown_type(cea_module):
    mix = cea_module.Mixture(["H2", "O2"])
    weights = np.array([0.4, 0.6], dtype=np.float64)
    with pytest.raises(ValueError):
        mix.calc_property(999999, weights, temperature=3000.0)

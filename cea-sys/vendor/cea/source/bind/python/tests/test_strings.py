import numpy as np
import pytest

@pytest.mark.smoke
def test_mixture_accepts_python_strings(cea_module) -> None:
    mix = cea_module.Mixture(["H2", "O2", "Ar"])
    weights = mix.moles_to_weights(np.array([2.0, 1.0, 0.5], dtype=np.float64))
    density = mix.calc_property(cea_module.DENSITY, weights, temperature=3000.0, pressure=1.0)
    assert np.isfinite(density)


def test_bytes_inputs_remain_supported(cea_module) -> None:
    text_mix = cea_module.Mixture(["H2", "O2"])
    bytes_mix = cea_module.Mixture([b"H2", b"O2"])
    weights = np.array([0.4, 0.6], dtype=np.float64)

    # ENERGY does not require an explicit pressure argument, keeping the comparison simple.
    energy_text = text_mix.calc_property(cea_module.ENERGY, weights, temperature=3200.0)
    energy_bytes = bytes_mix.calc_property(cea_module.ENERGY, weights, temperature=3200.0)
    assert energy_text == pytest.approx(energy_bytes)

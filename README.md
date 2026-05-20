# nasa-cea

[![CI](https://github.com/mcurrie99/nasa-cea/actions/workflows/ci.yml/badge.svg)](https://github.com/mcurrie99/nasa-cea/actions/workflows/ci.yml)

Rust bindings and a small safe wrapper for NASA CEA.

The workspace is split into two layers:

- `cea-sys`: raw FFI bindings and vendored CEA build.
- `cea`: safe Rust handles plus a higher-level interface for application code.

## Quick Use

For one-off calculations, use the high-level helper:

```rust
use nasa_cea::solve_tp_equivalence_moles;

const ATM: f64 = 1.01325;

fn main() -> Result<(), nasa_cea::Error> {
    let result = solve_tp_equivalence_moles(
        &["H2", "Air"],
        &[
            "Ar", "C", "CO", "CO2", "H", "H2", "H2O", "HNO", "HO2", "HNO2",
            "HNO3", "N", "NH", "NO", "N2", "N2O3", "O", "O2", "OH", "O3",
        ],
        3000.0,
        ATM,
        &[1.0, 0.0],
        &[0.0, 1.0],
        1.0,
    )?;

    println!("H = {}", result.properties.enthalpy);
    println!("Cp = {}", result.properties.equilibrium_cp);
    println!("H2O mole fraction = {:?}", result.product_mole_fraction("H2O"));
    Ok(())
}
```

For repeated calculations, build and reuse an `Equilibrium` model:

```rust
use nasa_cea::Equilibrium;

fn main() -> Result<(), nasa_cea::Error> {
    let mut equilibrium = Equilibrium::builder()
        .reactants(["H2", "Air"])
        .products([
            "Ar", "C", "CO", "CO2", "H", "H2", "H2O", "HNO", "HO2", "HNO2",
            "HNO3", "N", "NH", "NO", "N2", "N2O3", "O", "O2", "OH", "O3",
        ])
        .build()?;

    let result = equilibrium.tp_with_chemical_equivalence_moles(
        3000.0,
        1.01325,
        &[1.0, 0.0],
        &[0.0, 1.0],
        1.0,
    )?;

    println!("T = {}", result.properties.temperature);
    Ok(())
}
```

## Using From Another Project

Point your other project at the GitHub repository:

```toml
[dependencies]
nasa-cea = { git = "https://github.com/mcurrie99/nasa-cea.git" }
```

Then import it as `nasa_cea`.

The lower-level `cea` crate API is still available if you need explicit control over mixtures, solvers, solutions, and individual CEA calls.

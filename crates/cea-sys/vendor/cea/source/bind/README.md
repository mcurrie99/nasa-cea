CEA Language Bindings
=====================
This directory contains the language bindings that wrap the CEA core.

Subdirectories:
- `c/` — C ABI shim and headers (see `c/README.md`)
- `python/` — Python binding (see `python/README.md`)
- `matlab/` — experimental MATLAB interface (see `matlab/README.md`)
- `excel/` — experimental Excel interface (see `excel/README.md`)

Build configuration:
- Use `cmake --preset dev` to enable all bindings.
- Or set `CEA_ENABLE_BIND_C`, `CEA_ENABLE_BIND_PYTHON`, and
  `CEA_ENABLE_BIND_MATLAB` individually.

# SDFX isosurface

This Go library combines the projects:

- *[SDFX](https://github.com/deadsy/sdfx)*: A simple CAD package using Signed Distance Functions.
- *[isosurface](https://github.com/swiftcoder/isosurface)*: implements SDF meshing algorithms

This takes meshing surfaces designed with *SDFX* and uses the Dual Contouring algorithm implementation of
*isosurface* to create a mesh that preserves sharp features.

In order to avoid Cgo (and easily support all of Go's target platforms), *isosurface* is compiled to a wasm module and
executed using [*wazero*](https://github.com/tetratelabs/wazero), the zero dependency WebAssembly runtime for Go
developers.

DO NOT USE: It is slower (~7x if using fast wasm executor, ~47x if using the compatible executor) than the Dual
Contouring [implementation from *SDFX*](https://github.com/deadsy/sdfx/pull/42), and produces worse results.

Then, why build it? Mostly to learn how cross-language interoperability via WebAssembly works:
see [isosurface.go](./isosurface.go) and [lib.rs](isosurface-api/src/lib.rs).

# Summary
Make the periodic-table library buildable as Wasm as a secondary output, so that it can be called from JavaScript or TypeScript code.

## Context
We already have the `periodic-table` library written in Rust. Exposing it as Wasm is a new initiative. Considering the codebase is in Rust, we want to use `cargo-wasi-sdk` to build the Wasm output. The library consists of sub-crates: `pt-cli` depends on `pt-domain`, `pt-services` which is depending om `pt-data` and `pt-domain`. Since `pt-services` is the main entry point to the external world. The element data set for perfiodic table exists in `data/elements` directory. We need to maintain the data sets outside of the wasm output to have the flexibility to modify it externally. Currently, the library assumes the files exist in a folder provided by the application using the library. The relative path to this folder is configurable in the client application. We will package the files using the crate itself, making it self-contained and removing the dependency on the file system. The `pt-services` crate should expose a clear and simple API for the client application to use. The API should be documented and easy to use. We should also consider to use `wasm-bindgen` to generate the Wasm output for ease of use in the client application.

## Work Plan

1. Add `wasm32-wasi-sdk` to the development dependencies of `pt-services` crate
2. Add `wasm-bindgen` to the development dependencies of `pt-services` crate
3. Add `wasm-pack` to the development dependencies of `pt-services` crate
4. Add `wasm-bindgen-futures` to the dev dependencies of `pt-services` crate
5. Create a `Cargo.toml.wasm` file for `pt-services` crate
6. Create a `Cargo.toml.wasm` file for `pt-data` crate
7. Add the element data files to the `pt-data` crate
8. Create a Wasm output for `pt-services` crate
9. Create a Wasm output for `pt-domain` crate
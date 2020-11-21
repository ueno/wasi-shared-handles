# wasi-shared-handles

This is a demo of [shared handles PR] proposed to wasi-common,
providing a simple [memfd]-like facility as a separate WASI context.
The example program (`examples/read-write`) opens a handle backed by
memory, and read/write as if it is a normal file descriptor.

# Preparation

This crate assumes that the patched wasmtime source tree is available
at `../wasmtime`.  Adjust the paths in `Cargo.toml` as needed.

The example requires `wasi_ext` feature only available in nightly.
Install the nightly toolchain for the `wasm32-wasi` target:

```console
rustup toolchain install --target=wasm32-wasi nightly
```

# Compiling the example

```console
cd examples/read-write
cargo +nightly build --target=wasm32-wasi
cd -
```

# Running the example

```console
cargo build
cargo run examples/read-write/target/wasm32-wasi/debug/read-write.wasm
```

You will see "Hello world!".

[shared handles PR]: https://github.com/bytecodealliance/wasmtime/pull/2304
[memfd]: https://man7.org/linux/man-pages/man2/memfd_create.2.html

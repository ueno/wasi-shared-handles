use anyhow::{Context as _, Result};
use std::path::PathBuf;
mod memory;

fn main() -> Result<()> {
    let path: PathBuf = std::env::args().skip(1).next().unwrap().into();

    let store = wasmtime::Store::default();
    let mut linker = wasmtime::Linker::new(&store);

    let mut builder = wasi_c2::WasiCtx::builder();
    builder.inherit_stdio();

    let ctx = builder.build()?;
    let memory_ctx = memory::WasiMemoryCtx::new(&ctx.clone());

    let snapshot1 = wasi_c2_wasmtime::Wasi::new(&store, ctx);
    snapshot1.add_to_linker(&mut linker)?;

    let memory = memory::WasiMemory::new(&store, memory_ctx);
    memory.add_to_linker(&mut linker)?;

    let module = wasmtime::Module::from_file(store.engine(), &path)
        .context("failed to create wasm module")?;

    linker
        .module("", &module)
        .and_then(|m| m.get_default(""))
        .and_then(|f| f.get0::<()>())
        .and_then(|f| f().map_err(Into::into))
        .context(format!("error while testing Wasm module '{:?}'", &path,))
}

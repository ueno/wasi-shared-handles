use anyhow::{Context as _, Result};
use std::path::PathBuf;
use structopt::StructOpt;
mod handle;
mod memory;
mod socket;

#[derive(Debug, StructOpt)]
#[structopt(name = "wasi-shared-handles", about = "An example of shared handles")]
struct Opt {
    #[structopt(short)]
    address: Option<String>,

    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let store = wasmtime::Store::default();
    let mut linker = wasmtime::Linker::new(&store);

    let mut builder = wasi_common::WasiCtxBuilder::new();
    builder.inherit_stdio();

    let ctx = builder.build()?;

    let memory_ctx = memory::WasiMemoryCtx::new(&ctx.clone());
    let memory = memory::WasiMemory::new(&store, memory_ctx);
    memory.add_to_linker(&mut linker)?;

    if let Some(address) = opt.address {
        let listener = std::net::TcpListener::bind(address)?;
        let socket_ctx = socket::WasiSocketCtx::new(&ctx.clone(), listener);
        let socket = socket::WasiSocket::new(&store, socket_ctx);
        socket.add_to_linker(&mut linker)?;
    }

    let snapshot1 = wasmtime_wasi::Wasi::new(&store, ctx);
    snapshot1.add_to_linker(&mut linker)?;

    let module = wasmtime::Module::from_file(store.engine(), &opt.input)
        .context("failed to create wasm module")?;

    linker
        .module("", &module)
        .and_then(|m| m.get_default(""))
        .and_then(|f| f.get0::<()>())
        .and_then(|f| f().map_err(Into::into))
        .context(format!(
            "error while testing Wasm module '{:?}'",
            &opt.input
        ))
}

use anyhow::{Context as _, Result};
use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;
mod handle;
mod memory;
mod socket;

#[derive(Debug, StructOpt)]
#[structopt(name = "wasi-shared-handles", about = "An example of shared handles")]
struct Opt {
    #[structopt(long)]
    address: String,

    #[structopt(long)]
    keys: PathBuf,

    #[structopt(long)]
    certs: PathBuf,

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

    let listener = std::net::TcpListener::bind(opt.address)?;
    let certs = File::open(opt.certs)?;
    let mut reader = io::BufReader::new(certs);
    let certs = rustls::internal::pemfile::certs(&mut reader).unwrap();
    let keys = File::open(opt.keys)?;
    let mut reader = io::BufReader::new(keys);
    let keys = rustls::internal::pemfile::pkcs8_private_keys(&mut reader).unwrap();
    let mut config = rustls::ServerConfig::new(rustls::NoClientAuth::new());
    config.set_single_cert(certs, keys[0].clone())?;
    let config = Arc::new(config);

    let socket_ctx = socket::WasiSocketCtx::new(&ctx.clone(), &config.clone(), listener);
    let socket = socket::WasiSocket::new(&store, socket_ctx);
    socket.add_to_linker(&mut linker)?;

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

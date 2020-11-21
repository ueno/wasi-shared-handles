use std::cell::RefCell;
use wasi_common::WasiCtx;

pub struct WasiMemoryCtx {
    wasi_ctx: RefCell<WasiCtx>,
}

impl WasiMemoryCtx {
    pub fn new(wasi_ctx: &WasiCtx) -> Self {
        Self {
            wasi_ctx: RefCell::new(wasi_ctx.clone()),
        }
    }
}

wiggle::from_witx!({
    witx: ["$CARGO_MANIFEST_DIR/src/memory.witx"],
    ctx: WasiMemoryCtx,
});

impl types::GuestErrorConversion for WasiMemoryCtx {
    fn into_errno(&self, e: wiggle::GuestError) -> types::Errno {
        eprintln!("Guest error: {:?}", e);
        types::Errno::Inval
    }
}

impl wiggle::GuestErrorType for types::Errno {
    fn success() -> Self {
        Self::Success
    }
}

impl wasi_ephemeral_memory::WasiEphemeralMemory for WasiMemoryCtx {
    fn open(&self) -> Result<types::Fd, types::Errno> {
        let handle = wasi_common::virtfs::InMemoryFile::memory_backed();
        self.wasi_ctx
            .borrow_mut()
            .insert_handle(handle)
            .and_then(|fd| Ok(Into::<u32>::into(fd).into()))
            .map_err(|_| types::Errno::Inval)
    }
}

wasmtime_wiggle::wasmtime_integration!({
    target: self,
    witx: ["$CARGO_MANIFEST_DIR/src/memory.witx"],
    ctx: WasiMemoryCtx,
    modules: {
        wasi_ephemeral_memory => {
            name: WasiMemory,
        }
    },
    missing_memory: { wasi_common::wasi::types::Errno::Inval },
});

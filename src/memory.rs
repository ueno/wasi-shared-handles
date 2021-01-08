use std::cell::RefCell;
use wasi_c2::{FileCaps, WasiCtx};

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
        let pipe = wasi_c2::virt::pipe::Pipe::from(vec![]);
        self.wasi_ctx
            .borrow()
            .insert_file(Box::new(pipe),
                         FileCaps::READ |
                         FileCaps::WRITE |
                         FileCaps::SEEK |
                         FileCaps::TELL)
            .and_then(|fd| Ok(fd.into()))
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
    missing_memory: { wasi_c2::snapshots::preview_1::types::Errno::Inval },
});

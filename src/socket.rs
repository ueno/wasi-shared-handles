use rustls::{ServerConfig, Session};
use std::cell::RefCell;
use std::net::TcpListener;
use std::sync::Arc;
use wasi_common::WasiCtx;

pub struct WasiSocketCtx {
    wasi_ctx: RefCell<WasiCtx>,
    config: Arc<ServerConfig>,
    listener: RefCell<TcpListener>,
}

impl WasiSocketCtx {
    pub fn new(wasi_ctx: &WasiCtx, config: &Arc<ServerConfig>, listener: TcpListener) -> Self {
        Self {
            wasi_ctx: RefCell::new(wasi_ctx.clone()),
            config: config.clone(),
            listener: RefCell::new(listener),
        }
    }
}

wiggle::from_witx!({
    witx: ["$CARGO_MANIFEST_DIR/src/socket.witx"],
    ctx: WasiSocketCtx,
});

impl types::GuestErrorConversion for WasiSocketCtx {
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

impl wasi_ephemeral_socket::WasiEphemeralSocket for WasiSocketCtx {
    fn accept(&self) -> Result<types::Fd, types::Errno> {
        let mut socket = self
            .listener
            .borrow()
            .accept()
            .and_then(|(socket, _address)| Ok(socket))
            .map_err(|_| types::Errno::Inval)?;
        let mut session = rustls::ServerSession::new(&self.config.clone());
        while session.is_handshaking() {
            if session.wants_read() {
                session
                    .read_tls(&mut socket)
                    .map_err(|_| types::Errno::Inval)?;
                session
                    .process_new_packets()
                    .map_err(|_| types::Errno::Inval)?;
            }
            if session.wants_write() {
                session
                    .write_tls(&mut socket)
                    .map_err(|_| types::Errno::Inval)?;
            }
        }
        let handle = crate::handle::SocketHandle::new(socket, session);
        self.wasi_ctx
            .borrow()
            .insert_handle(handle)
            .and_then(|fd| Ok(Into::<u32>::into(fd).into()))
            .map_err(|_| types::Errno::Inval)
    }
}

wasmtime_wiggle::wasmtime_integration!({
    target: self,
    witx: ["$CARGO_MANIFEST_DIR/src/socket.witx"],
    ctx: WasiSocketCtx,
    modules: {
        wasi_ephemeral_socket => {
            name: WasiSocket,
        }
    },
    missing_memory: { wasi_common::wasi::types::Errno::Inval },
});

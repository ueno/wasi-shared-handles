#![feature(wasi_ext)]

use std::io::prelude::*;
use std::os::wasi::prelude::*;

fn test_echo_server() {
    // Accept an incoming connection
    let socket_fd = unsafe { wasi_socket::accept().expect("accepting incoming connection") };
    println!("accepted {}", socket_fd);
    let mut socket = unsafe { std::fs::File::from_raw_fd(socket_fd) };

    loop {
        let mut buffer = [0u8; 1024];
        let nread = socket.read(&mut buffer).expect("reading data from connection");
        if nread == 0 {
            break
        }
        let nwritten = socket.write(&buffer[..nread]).expect("writing data to connection");
        assert_eq!(nwritten, nread);
    }
}

fn main() {
    test_echo_server();
}

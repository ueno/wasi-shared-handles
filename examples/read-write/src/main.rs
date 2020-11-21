#![feature(wasi_ext)]

use std::io::prelude::*;
use std::os::wasi::prelude::FromRawFd;

use wasi_memory;

const DATA: &[u8] = b"Hello world!";

fn test_read_write() {
    // Open a file descriptor backed by memory
    let mem_fd = unsafe { wasi_memory::open().expect("opening memory") };

    // Write data to the file descriptor
    let mut file = unsafe { std::fs::File::from_raw_fd(mem_fd) };
    let nwritten = file.write(DATA).expect("writing to a file");
    assert_eq!(nwritten, DATA.len());

    // Seek to the beginning of the file
    let offset = file.seek(std::io::SeekFrom::Start(0)).expect("seeking to the beginning of the file");
    assert_eq!(
        offset, 0,
        "offset after seeking to the beginning of the file should be at 0"
    );

    // Read data from the file descriptor
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).expect("reading file");
    assert_eq!(buffer.as_bytes(), DATA);

    println!("{}", buffer);
}

fn main() {
    test_read_write();
}

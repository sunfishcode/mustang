//! The following is derived from Rust's
//! library/std/src/sys/unix/kernel_copy/tests.rs at revision
//! 5c0d76dbe1669c96f1959d7b0b1d4de7e9a47c43.

#![feature(try_blocks)]

mustang::can_run_this!();

mod sys_common;

use std::fs::OpenOptions;
#[cfg(feature = "bench")]
use std::io;
use std::io::Result;
use std::io::SeekFrom;
use std::io::{BufRead, Read, Seek, Write};
#[cfg(feature = "bench")]
use std::os::unix::io::AsRawFd;
use sys_common::io::test::tmpdir;

#[test]
fn copy_specialization() -> Result<()> {
    use std::io::{BufReader, BufWriter};

    let tmp_path = tmpdir();
    let source_path = tmp_path.join("copy-spec.source");
    let sink_path = tmp_path.join("copy-spec.sink");

    let result: Result<()> = try {
        let mut source = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&source_path)?;
        source.write_all(b"abcdefghiklmnopqr")?;
        source.seek(SeekFrom::Start(8))?;
        let mut source = BufReader::with_capacity(8, source.take(5));
        source.fill_buf()?;
        assert_eq!(source.buffer(), b"iklmn");
        source.get_mut().set_limit(6);
        source.get_mut().get_mut().seek(SeekFrom::Start(1))?; // "bcdefg"
        let mut source = source.take(10); // "iklmnbcdef"

        let mut sink = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&sink_path)?;
        sink.write_all(b"000000")?;
        let mut sink = BufWriter::with_capacity(5, sink);
        sink.write_all(b"wxyz")?;
        assert_eq!(sink.buffer(), b"wxyz");

        let copied = std::io::copy(&mut source, &mut sink)?;
        assert_eq!(copied, 10, "copy obeyed limit imposed by Take");
        assert_eq!(sink.buffer().len(), 0, "sink buffer was flushed");
        assert_eq!(source.limit(), 0, "outer Take was exhausted");
        assert_eq!(
            source.get_ref().buffer().len(),
            0,
            "source buffer should be drained"
        );
        assert_eq!(
            source.get_ref().get_ref().limit(),
            1,
            "inner Take allowed reading beyond end of file, some bytes should be left"
        );

        let mut sink = sink.into_inner()?;
        sink.seek(SeekFrom::Start(0))?;
        let mut copied = Vec::new();
        sink.read_to_end(&mut copied)?;
        assert_eq!(&copied, b"000000wxyziklmnbcdef");
    };

    let rm1 = std::fs::remove_file(source_path);
    let rm2 = std::fs::remove_file(sink_path);

    result.and(rm1).and(rm2)
}

#[test]
fn copies_append_mode_sink() -> Result<()> {
    let tmp_path = tmpdir();
    let source_path = tmp_path.join("copies_append_mode.source");
    let sink_path = tmp_path.join("copies_append_mode.sink");
    let mut source = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .read(true)
        .open(&source_path)?;
    write!(source, "not empty")?;
    source.seek(SeekFrom::Start(0))?;
    let mut sink = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&sink_path)?;

    let copied = std::io::copy(&mut source, &mut sink)?;

    assert_eq!(copied, 9);

    Ok(())
}

#[cfg(feature = "bench")]
#[bench]
fn bench_file_to_file_copy(b: &mut test::Bencher) {
    const BYTES: usize = 128 * 1024;
    let temp_path = tmpdir();
    let src_path = temp_path.join("file-copy-bench-src");
    let mut src = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(src_path)
        .unwrap();
    src.write(&vec![0u8; BYTES]).unwrap();

    let sink_path = temp_path.join("file-copy-bench-sink");
    let mut sink = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(sink_path)
        .unwrap();

    b.bytes = BYTES as u64;
    b.iter(|| {
        src.seek(SeekFrom::Start(0)).unwrap();
        sink.seek(SeekFrom::Start(0)).unwrap();
        assert_eq!(BYTES as u64, io::copy(&mut src, &mut sink).unwrap());
    });
}

#[cfg(feature = "bench")]
#[bench]
fn bench_file_to_socket_copy(b: &mut test::Bencher) {
    const BYTES: usize = 128 * 1024;
    let temp_path = tmpdir();
    let src_path = temp_path.join("pipe-copy-bench-src");
    let mut src = OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(src_path)
        .unwrap();
    src.write(&vec![0u8; BYTES]).unwrap();

    let sink_drainer = std::net::TcpListener::bind("localhost:0").unwrap();
    let mut sink = std::net::TcpStream::connect(sink_drainer.local_addr().unwrap()).unwrap();
    let mut sink_drainer = sink_drainer.accept().unwrap().0;

    std::thread::spawn(move || {
        let mut sink_buf = vec![0u8; 1024 * 1024];
        loop {
            sink_drainer.read(&mut sink_buf[..]).unwrap();
        }
    });

    b.bytes = BYTES as u64;
    b.iter(|| {
        src.seek(SeekFrom::Start(0)).unwrap();
        assert_eq!(BYTES as u64, io::copy(&mut src, &mut sink).unwrap());
    });
}

#[cfg(feature = "bench")]
#[bench]
fn bench_file_to_uds_copy(b: &mut test::Bencher) {
    const BYTES: usize = 128 * 1024;
    let temp_path = tmpdir();
    let src_path = temp_path.join("uds-copy-bench-src");
    let mut src = OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(src_path)
        .unwrap();
    src.write(&vec![0u8; BYTES]).unwrap();

    let (mut sink, mut sink_drainer) = std::os::unix::net::UnixStream::pair().unwrap();

    std::thread::spawn(move || {
        let mut sink_buf = vec![0u8; 1024 * 1024];
        loop {
            sink_drainer.read(&mut sink_buf[..]).unwrap();
        }
    });

    b.bytes = BYTES as u64;
    b.iter(|| {
        src.seek(SeekFrom::Start(0)).unwrap();
        assert_eq!(BYTES as u64, io::copy(&mut src, &mut sink).unwrap());
    });
}

#[cfg(any(target_os = "linux", target_os = "android"))]
#[cfg(feature = "bench")]
#[bench]
fn bench_socket_pipe_socket_copy(b: &mut test::Bencher) {
    use super::CopyResult;
    use std::io::ErrorKind;
    use std::process::{ChildStdin, ChildStdout};
    use std::sys_common::FromInner;

    let (read_end, write_end) = std::sys::pipe::anon_pipe().unwrap();

    let mut read_end = ChildStdout::from_inner(read_end);
    let write_end = ChildStdin::from_inner(write_end);

    let acceptor = std::net::TcpListener::bind("localhost:0").unwrap();
    let mut remote_end = std::net::TcpStream::connect(acceptor.local_addr().unwrap()).unwrap();

    let local_end = std::sync::Arc::new(acceptor.accept().unwrap().0);

    // the data flow in this benchmark:
    //
    //                      socket(tx)  local_source
    // remote_end (write)  +-------->   (splice to)
    //                                  write_end
    //                                     +
    //                                     |
    //                                     | pipe
    //                                     v
    //                                  read_end
    // remote_end (read)   <---------+  (splice to) *
    //                      socket(rx)  local_end
    //
    // * benchmark loop using io::copy

    std::thread::spawn(move || {
        let mut sink_buf = vec![0u8; 1024 * 1024];
        remote_end.set_nonblocking(true).unwrap();
        loop {
            match remote_end.write(&mut sink_buf[..]) {
                Err(err) if err.kind() == ErrorKind::WouldBlock => {}
                Ok(_) => {}
                err => {
                    err.expect("write failed");
                }
            };
            match remote_end.read(&mut sink_buf[..]) {
                Err(err) if err.kind() == ErrorKind::WouldBlock => {}
                Ok(_) => {}
                err => {
                    err.expect("read failed");
                }
            };
        }
    });

    // check that splice works, otherwise the benchmark would hang
    let probe = super::sendfile_splice(
        super::SpliceMode::Splice,
        local_end.as_raw_fd(),
        write_end.as_raw_fd(),
        1,
    );

    match probe {
        CopyResult::Ended(1) => {
            // splice works
        }
        _ => {
            eprintln!("splice failed, skipping benchmark");
            return;
        }
    }

    let local_source = local_end.clone();
    std::thread::spawn(move || loop {
        super::sendfile_splice(
            super::SpliceMode::Splice,
            local_source.as_raw_fd(),
            write_end.as_raw_fd(),
            u64::MAX,
        );
    });

    const BYTES: usize = 128 * 1024;
    b.bytes = BYTES as u64;
    b.iter(|| {
        assert_eq!(
            BYTES as u64,
            io::copy(&mut (&mut read_end).take(BYTES as u64), &mut &*local_end).unwrap()
        );
    });
}

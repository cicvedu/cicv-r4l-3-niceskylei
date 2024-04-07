// SPDX-License-Identifier: GPL-2.0

//! Rust character device sample.

use core::result::Result::Err;
use core::sync::atomic::{AtomicUsize, Ordering};

use kernel::sync::Mutex;
use kernel::{chrdev, file};
use kernel::{prelude::*, PointerWrapper};

const GLOBALMEM_SIZE: usize = 0x1000;

module! {
    type: RustChrdev,
    name: "rust_chrdev",
    author: "Rust for Linux Contributors",
    description: "Rust character device sample",
    license: "GPL",
}

static GLOBALMEM_BUF: Mutex<[u8; GLOBALMEM_SIZE]> = unsafe { Mutex::new([0u8; GLOBALMEM_SIZE]) };
static GLOBAL_BUF_LEN: AtomicUsize = AtomicUsize::new(0);

struct RustFile {
    #[allow(dead_code)]
    inner: &'static Mutex<[u8; GLOBALMEM_SIZE]>,
}

#[vtable]
impl file::Operations for RustFile {
    type Data = Box<Self>;

    fn open(_shared: &(), _file: &file::File) -> Result<Box<Self>> {
        Ok(Box::try_new(RustFile {
            inner: &GLOBALMEM_BUF,
        })?)
    }

    fn write(
        data: <Self::Data as PointerWrapper>::Borrowed<'_>,
        _file: &file::File,
        reader: &mut impl kernel::io_buffer::IoBufferReader,
        offset: u64,
    ) -> Result<usize> {
        let offset = offset as usize;
        let read_size = reader.len();
        let buf_end = offset + read_size;
        if read_size > GLOBALMEM_SIZE {
            return Err(EOVERFLOW);
        }
        let mut buf = data.inner.lock();

        match reader.read_slice(&mut buf[..read_size]) {
            Ok(_) => {
                GLOBAL_BUF_LEN.store(buf_end, Ordering::Release);
                Ok(read_size)
            }
            Err(e) => Err(e),
        }
    }

    fn read(
        data: <Self::Data as PointerWrapper>::Borrowed<'_>,
        _file: &file::File,
        writer: &mut impl kernel::io_buffer::IoBufferWriter,
        offset: u64,
    ) -> Result<usize> {
        let offset = offset as usize;
        let write_len = writer.len();
        let global_buf_len = GLOBAL_BUF_LEN.load(Ordering::SeqCst);
        if offset > global_buf_len {
            return Err(ESPIPE);
        }
        let can_read_len = write_len.min(global_buf_len - offset);
        if can_read_len == 0 {
            return Ok(0);
        }
        let read_end = offset + can_read_len;
        let buf = data.inner.lock();
        let write_buf = &buf[offset..read_end];
        writer.write_slice(write_buf)?;
        Ok(can_read_len)
    }
}

struct RustChrdev {
    _dev: Pin<Box<chrdev::Registration<2>>>,
}

impl kernel::Module for RustChrdev {
    fn init(name: &'static CStr, module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust character device sample (init)\n");

        let mut chrdev_reg = chrdev::Registration::new_pinned(name, 0, module)?;

        // Register the same kind of device twice, we're just demonstrating
        // that you can use multiple minors. There are two minors in this case
        // because its type is `chrdev::Registration<2>`
        chrdev_reg.as_mut().register::<RustFile>()?;
        chrdev_reg.as_mut().register::<RustFile>()?;

        Ok(RustChrdev { _dev: chrdev_reg })
    }
}

impl Drop for RustChrdev {
    fn drop(&mut self) {
        pr_info!("Rust character device sample (exit)\n");
    }
}

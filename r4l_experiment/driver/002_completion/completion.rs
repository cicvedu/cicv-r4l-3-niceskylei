// SPDX-License-Identifier: GPL-2.0
//! Rust minimal sample.

use core::ptr::drop_in_place;

use kernel::bindings::{complete, completion, init_completion, wait_for_completion};
use kernel::task::Task;
use kernel::Opaque;
use kernel::{chrdev, file, prelude::*};
const MODULE_NAME: &'static str = "completion";

static GLOBAL_COMPLETION: CompletionOp = CompletionOp {
    inner: Opaque::uninit(),
};

struct CompletionOp {
    inner: Opaque<completion>,
}

unsafe impl Sync for CompletionOp {}
unsafe impl Send for CompletionOp {}

module! {
  type: CompletionMod,
  name: "completion",
  author: "Tester",
  description: "Example of Kernel's completion mechanism",
  license: "GPL",
}
#[vtable]
impl file::Operations for CompletionOp {
    type Data = ();

    fn open(_context: &Self::OpenData, _file: &file::File) -> Result<Self::Data> {
        Ok(())
    }
    fn read(
        _data: <Self::Data as kernel::PointerWrapper>::Borrowed<'_>,
        _file: &file::File,
        _writer: &mut impl kernel::io_buffer::IoBufferWriter,
        _offset: u64,
    ) -> Result<usize> {
        pr_info!("completion read is invoked\n");
        let pid = {
            let current = Task::current();
            (*current).pid()
        };
        pr_info!("process {} is going to sleep\n", pid);
        unsafe { wait_for_completion(GLOBAL_COMPLETION.inner.get()) };
        pr_info!("awoken {}\n", pid);
        Ok(0)
    }
    fn write(
        _data: <Self::Data as kernel::PointerWrapper>::Borrowed<'_>,
        _file: &file::File,
        reader: &mut impl kernel::io_buffer::IoBufferReader,
        _offset: u64,
    ) -> Result<usize> {
        pr_info!("completion write is invoked\n");
        let pid = {
            let current = Task::current();
            (*current).pid()
        };
        pr_info!("process {}  awakening the readers...\n", pid);
        unsafe { complete(GLOBAL_COMPLETION.inner.get()) };
        Ok(reader.len())
    }
}

struct CompletionMod {
    #[allow(dead_code)]
    reg: Pin<Box<chrdev::Registration<1>>>,
}

impl kernel::Module for CompletionMod {
    fn init(name: &'static CStr, module: &'static ThisModule) -> Result<Self> {
        pr_warn!("{} is loaded\n", MODULE_NAME);
        unsafe { init_completion(GLOBAL_COMPLETION.inner.get()) };

        let mut reg = chrdev::Registration::new_pinned(name, 0, module)?;
        reg.as_mut().register::<CompletionOp>()?;

        Ok(CompletionMod { reg })
    }
}

impl Drop for CompletionMod {
    fn drop(&mut self) {
        unsafe { drop_in_place(GLOBAL_COMPLETION.inner.get()) };
        pr_alert!("{} is dropped!\n", MODULE_NAME);
    }
}

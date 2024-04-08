// SPDX-License-Identifier: GPL-2.0
//! Rust minimal sample.

use kernel::prelude::*;

module! {
  type: HelloWorldMod,
  name: "helloworld",
  author: "Tester",
  description: "Hello World program",
  license: "GPL",
}

struct HelloWorldMod {}

impl kernel::Module for HelloWorldMod {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_alert!("Hello, world!\n");
        Ok(HelloWorldMod {})
    }
}

impl Drop for HelloWorldMod {
    fn drop(&mut self) {
        pr_alert!("Bye, world!\n");
    }
}

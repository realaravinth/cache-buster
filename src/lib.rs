/*
* Copyright (C) 2021  Aravinth Manivannan <realaravinth@batsense.net>
*
* Use of this source code is governed by the Apache 2.0 and/or the MIT
* License.
*/
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]
//! ```no_run
//! use cache_buster::BusterBuilder;
//!
//! fn main() {
//!     // note: add error checking yourself.
//!     //    println!("cargo:rustc-env=GIT_process={}", git_process);
//!     let types = vec![
//!         mime::IMAGE_PNG,
//!         mime::IMAGE_SVG,
//!         mime::IMAGE_JPEG,
//!         mime::IMAGE_GIF,
//!     ];
//!
//!     let config = BusterBuilder::default()
//!         .source("./dist")
//!         .result("./prod")
//!         .mime_types(types)
//!         .copy(true)
//!         .follow_links(true)
//!         .build()
//!         .unwrap();
//!
//!     config.process().unwrap();
//! }
//! ```

pub mod processor;
pub use processor::BusterBuilder;
pub mod filemap;
pub use filemap::Files;

/// env var to which filemap is written during compilation
pub const ENV_VAR_NAME: &str = "CACHE_BUSTER_FILE_MAP";

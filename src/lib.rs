/*
* Copyright (C) 2021  Aravinth Manivannan <realaravinth@batsense.net>
*
* Use of this source code is governed by the Apache 2.0 and/or the MIT
* License.
*/

//! ```rust
//! use cache_buster::BusterBuilder;
//!
//! fn main() {
//!     // note: add error checking yourself.
//!     //    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
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
//!     config.init().unwrap();
//!     config.hash().unwrap();
//! }
//! ```

pub mod hash;
pub use hash::BusterBuilder;
pub mod map;
pub use map::Files;

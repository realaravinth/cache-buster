/*
* Copyright (C) 2021  Aravinth Manivannan <realaravinth@batsense.net>
*
* Use of this source code is governed by the Apache 2.0 and/or the MIT
* License.
*/
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]
//! # What is cache busting?
//!
//! To optimise network load time, browsers cache static files. Caching
//! greatly improves performance but how do you inform browsers to
//! invalidate cache when your files have changed?
//!
//! Cache busting is a simple but effective solution for this issue. There
//! are several ways to achieve this but the way this library does this is
//! by changing file names to include the hash of the files' contents.
//!
//! So if you have `bundle.js`, it will become
//! `bundle.<long-sha256-hash>.js`. This lets you set a super long cache age
//! as, because of the file names changing, the path to the filename, too,
//! will change. So as far as the browser is concerned, you are trying to load
//! a file that it doesn't have. Pretty neat, isn't it?
//!
//! ## Example:
//!
//! - `build.rs`
//! ```no_run
//! use cache_buster::BusterBuilder;
//!
//! // note: add error checking yourself.
//! //    println!("cargo:rustc-env=GIT_process={}", git_process);
//! let types = vec![
//!     mime::IMAGE_PNG,
//!     mime::IMAGE_SVG,
//!     mime::IMAGE_JPEG,
//!     mime::IMAGE_GIF,
//! ];
//!
//! let config = BusterBuilder::default()
//!     .source("./dist")
//!     .result("./prod")
//!     .mime_types(types)
//!     .copy(true)
//!     .follow_links(true)
//!     .build()
//!     .unwrap();
//!
//! config.process().unwrap();
//! ```
//! - `main.rs`:
//!
//! Module describing runtime compoenet for fetching modified filenames
//!
//! Add the following tou your program to load the filemap during compiletime:
//!
//! ```no_run
//! use cache_buster::Files;
//! use cache_buster::CACHE_BUSTER_DATA_FILE;
//!
//! let files = Files::new(CACHE_BUSTER_DATA_FILE);
//! // the path to the file before setting up for cache busting
//! files.get("./dist/github.svg");
//! ```

pub mod processor;
pub use processor::BusterBuilder;
pub use processor::NoHashCategory;
pub mod filemap;
pub use filemap::Files;

/// file to which filemap is written during compilation
/// include this to `.gitignore`
pub const CACHE_BUSTER_DATA_FILE: &str = "./src/cache_buster_data.json";

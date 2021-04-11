/*
* Copyright (C) 2021  Aravinth Manivannan <realaravinth@batsense.net>
*
* Use of this source code is governed by the Apache 2.0 and/or the MIT
* License.
*/
//! Module describing runtime compoenet for fetching modified filenames
//!
//! Add the following tou your program to load the filemap during compiletime:
//!
//! ```no_run
//! use cache_buster::Files;
//!
//! fn main(){
//!    let files = Files::new();
//! }
//! ```

use std::collections::HashMap;
use std::env;

use serde::{Deserialize, Serialize};

const ENV_VAR_NAME: &str = "CACHE_BUSTER_FILE_MAP";

/// Filemap struct
///
/// maps original names to generated names
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Files {
    /// filemap<original-path, modified-path>
    map: HashMap<String, String>,
    base_dir: String,
}

impl Files {
    /// Load filemap in main program. Should be called from main program
    pub fn new() -> Self {
        let env = env::var(ENV_VAR_NAME)
            .expect("unable to read env var, might be a bug in lib. Please report on GitHub");
        let res: Files = serde_json::from_str(&env).unwrap();
        res
    }

    /// Get relative file path
    ///
    /// If the modified filename path is `./prod/test.randomhash.svg`, it will
    /// output `/test.randomhash.svg`. For full path, see [get_full_path][Self::get_full_path].
    pub fn get<'a>(&'a self, path: &'a str) -> Option<&'a str> {
        if let Some(path) = self.map.get(path) {
            Some(&path[self.base_dir.len()..])
        } else {
            None
        }
    }

    /// Get file path
    ///
    /// If the modified filename path is `./prod/test.randomhash.svg`, it will
    /// output `/prod/test.randomhash.svg`. For relative path, see [get][Self::get].
    pub fn get_full_path<'a>(&'a self, path: &'a str) -> Option<&'a String> {
        self.map.get(path)
    }
}

#[cfg(test)]
mod tests {
    use crate::processor::tests::cleanup;
    use crate::processor::*;

    use super::*;
    use std::path::Path;

    #[test]
    fn get_full_path_works() {
        let types = vec![
            mime::IMAGE_PNG,
            mime::IMAGE_SVG,
            mime::IMAGE_JPEG,
            mime::IMAGE_GIF,
        ];

        let config = BusterBuilder::default()
            .source("./dist")
            .result("/tmp/prod2")
            .mime_types(types)
            .copy(true)
            .follow_links(true)
            .build()
            .unwrap();

        config.process().unwrap();

        let files = Files::new();
        assert!(get_full_path_runner("./dist/log-out.svg", &files));
        assert!(get_full_path_runner(
            "./dist/a/b/c/d/s/d/svg/credit-card.svg",
            &files
        ));

        assert!(!get_full_path_runner("dist/log-out.svg", &files));
        assert!(!get_full_path_runner(
            "dist/a/b/c/d/s/d/svg/credit-card.svg",
            &files
        ));
        cleanup(&config);
    }

    fn get_full_path_runner(path: &str, files: &Files) -> bool {
        if let Some(file) = files.get_full_path(path) {
            Path::new(file).exists()
        } else {
            false
        }
    }

    #[test]
    fn get_works() {
        let types = vec![
            mime::IMAGE_PNG,
            mime::IMAGE_SVG,
            mime::IMAGE_JPEG,
            mime::IMAGE_GIF,
        ];

        let config = BusterBuilder::default()
            .source("./dist")
            .result("/tmp/prod5")
            .mime_types(types)
            .copy(true)
            .follow_links(true)
            .build()
            .unwrap();

        config.process().unwrap();

        let files = Files::new();

        assert!(get_runner("./dist/log-out.svg", &files));
        assert!(get_runner("./dist/a/b/c/d/s/d/svg/credit-card.svg", &files));

        assert!(!get_runner("dist/log-out.svg", &files));
        assert!(!get_runner("dist/a/b/c/d/s/d/svg/credit-card.svg", &files));
        cleanup(&config);
    }

    fn get_runner(path: &str, files: &Files) -> bool {
        if let Some(file) = files.get(path) {
            let path = Path::new(&files.base_dir).join(&file[1..]);
            path.exists()
        } else {
            false
        }
    }
}

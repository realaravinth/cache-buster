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
//! use cache_buster::CACHE_BUSTER_DATA_FILE;
//!
//! fn main(){
//!    let files = Files::new(CACHE_BUSTER_DATA_FILE);
//! }
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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
    pub fn new(map: &str) -> Self {
        let res: Files = serde_json::from_str(&map).unwrap();
        res
    }

    /// Get relative file path
    ///
    /// If the modified filename path is `./prod/test.randomhash.svg`, it will
    /// output `/test.randomhash.svg`. For full path, see [get_full_path][Self::get_full_path].
    pub fn get<'a>(&'a self, path: &'a str) -> Option<&'a str> {
        if let Some(path) = self.map.get(path) {
            Some(&path[self.base_dir.len()..])
            // Some(&path)
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
    use std::fs;

    use crate::processor::tests::{cleanup, delete_file, runner as processor_runner};
    use crate::processor::*;
    use crate::CACHE_BUSTER_DATA_FILE;

    use super::*;
    use std::path::Path;

    fn get_full_path_works() {
        delete_file();
        let types = vec![
            mime::IMAGE_PNG,
            mime::IMAGE_SVG,
            mime::IMAGE_JPEG,
            mime::IMAGE_GIF,
        ];

        let config = BusterBuilder::default()
            .source("./dist")
            .result("/tmp/prodsd2")
            .mime_types(types)
            .copy(true)
            .follow_links(true)
            .build()
            .unwrap();

        config.process().unwrap();

        let map = fs::read_to_string(CACHE_BUSTER_DATA_FILE).unwrap();
        let files = Files::new(&map);

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

    fn get_works() {
        delete_file();
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

        let map = fs::read_to_string(CACHE_BUSTER_DATA_FILE).unwrap();
        let files = Files::new(&map);

        assert!(get_runner("./dist/log-out.svg", &files));
        assert!(get_runner("./dist/a/b/c/d/s/d/svg/credit-card.svg", &files));

        assert!(!get_runner("dist/log-out.svg", &files));
        assert!(!get_runner("dist/a/b/c/d/s/d/svg/credit-card.svg", &files));
        cleanup(&config);
    }

    fn get_runner(path: &str, files: &Files) -> bool {
        if let Some(file) = files.get(path) {
            let path = Path::new(&files.base_dir).join(&file[1..]);
            //println!("{}", &file);
            let path = Path::new(&path);
            path.exists()
        } else {
            false
        }
    }

    #[test]
    pub fn runner() {
        get_works();
        get_full_path_works();
        processor_runner();
    }
}
